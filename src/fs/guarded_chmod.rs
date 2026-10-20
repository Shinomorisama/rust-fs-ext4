//! Shared chmod/chown/utimens/flags/removexattr/setxattr dispatch; other families need caller serialization.
use super::*;
use crate::operation::{Health, Operation};
use std::sync::atomic::{AtomicBool, Ordering};

/// Borrowing the mount ties both the guard and device identity to its lifetime.
/// Helpers reuse this context; they never enter the control recursively.
struct MountOperation<'a> {
    fs: &'a Filesystem,
    operation: Operation<'a>,
}

/// Latch before an outer panic catcher can return, even if it keeps the context.
struct EffectFailure<'a> {
    failed: &'a AtomicBool,
    complete: bool,
}

impl Drop for EffectFailure<'_> {
    fn drop(&mut self) {
        if !self.complete {
            self.failed.store(true, Ordering::SeqCst);
        }
    }
}

impl MountOperation<'_> {
    fn effect<T>(&mut self, action: impl FnOnce() -> Result<T>) -> Result<T> {
        let mut failure = EffectFailure {
            failed: &self.fs.operation_failed,
            complete: false,
        };
        let result = self.operation.effect(|_| action());
        failure.complete = result.is_ok();
        result
    }
}

impl Drop for MountOperation<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() || self.operation.status().health != Health::Healthy {
            self.fs.operation_failed.store(true, Ordering::SeqCst);
        }
    }
}

impl Filesystem {
    fn chmod_operation(&self) -> Result<MountOperation<'_>> {
        if self.operation_failed.load(Ordering::SeqCst) {
            return Err(Error::ReadOnly);
        }
        match self.operation.try_enter() {
            Ok(operation) => Ok(MountOperation {
                fs: self,
                operation,
            }),
            Err(error) => {
                // Contention is an untouched refusal, not a mount failure.
                if matches!(error, Error::Corrupt(_)) {
                    self.operation_failed.store(true, Ordering::SeqCst);
                }
                Err(error)
            }
        }
    }

    fn check_chmod_context(&self, context: &MountOperation<'_>) -> Result<()> {
        // Identity precedes even device property callbacks.
        if !std::ptr::eq(self, context.fs) {
            return Err(Error::Stale);
        }
        if self.operation_failed.load(Ordering::SeqCst) {
            return Err(Error::ReadOnly);
        }
        context.operation.stamp()?;
        Ok(())
    }

    pub(crate) fn guarded_chmod_bytes(&self, path: &[u8], mode: u16) -> Result<()> {
        let mut context = self.chmod_operation()?;
        self.chmod_bytes_in(&mut context, path, mode)
    }

    #[allow(dead_code)] // Retained as a direct private-adapter control.
    pub(crate) fn guarded_chmod_ino(&self, r: InodeRef, mode: u16) -> Result<()> {
        self.guarded_chmod_from(r, mode)
    }

    pub(super) fn guarded_chmod_from(&self, r: impl Into<InodeRef>, mode: u16) -> Result<()> {
        let mut context = self.chmod_operation()?;
        // Into can run arbitrary caller code. Entry and verdict must precede it;
        // conversion is planning, before any marker/effect is attempted.
        self.write_refusal()?;
        self.chmod_ino_in(&mut context, r.into(), mode)
    }

    fn chmod_bytes_in(
        &self,
        context: &mut MountOperation<'_>,
        path: &[u8],
        mode: u16,
    ) -> Result<()> {
        self.check_chmod_context(context)?;
        self.write_refusal()?; // verdict only; refuse_write would write a marker
        let ino = self.resolve(path)?;
        self.chmod_ino_in(context, ino.into(), mode)
    }

    fn chmod_ino_in(&self, context: &mut MountOperation<'_>, r: InodeRef, mode: u16) -> Result<()> {
        self.check_chmod_context(context)?;
        self.write_refusal()?;
        let raw = self.plan_chmod(r, mode)?;
        self.commit_metadata_in(context, r.ino, &raw)
    }

    pub(super) fn guarded_chown_bytes(&self, path: &[u8], uid: u32, gid: u32) -> Result<()> {
        let mut context = self.chmod_operation()?;
        self.chown_bytes_in(&mut context, path, uid, gid)
    }

    pub(super) fn guarded_chown_from(
        &self,
        r: impl Into<InodeRef>,
        uid: u32,
        gid: u32,
    ) -> Result<()> {
        let mut context = self.chmod_operation()?;
        self.write_refusal()?;
        self.chown_ino_in(&mut context, r.into(), uid, gid)
    }

    fn chown_bytes_in(
        &self,
        context: &mut MountOperation<'_>,
        path: &[u8],
        uid: u32,
        gid: u32,
    ) -> Result<()> {
        self.check_chmod_context(context)?;
        self.write_refusal()?;
        let ino = self.resolve(path)?;
        self.chown_ino_in(context, ino.into(), uid, gid)
    }

    fn chown_ino_in(
        &self,
        context: &mut MountOperation<'_>,
        r: InodeRef,
        uid: u32,
        gid: u32,
    ) -> Result<()> {
        self.check_chmod_context(context)?;
        self.write_refusal()?;
        let raw = self.plan_chown(r, uid, gid)?;
        self.commit_metadata_in(context, r.ino, &raw)
    }

    fn commit_metadata_in(
        &self,
        context: &mut MountOperation<'_>,
        ino: u32,
        raw: &[u8],
    ) -> Result<()> {
        let mut buf = BlockBuffer::new(self.sb.block_size());
        self.buffer_write_inode(&mut buf, ino, raw)?;
        // Planning refusals above have no uncertain-write effect. Both actual
        // mutation phases include all callbacks, flushes and completion checks.
        if !self.marked_not_clean.load(Ordering::SeqCst) {
            context.effect(|| self.mark_not_clean_once())?;
        }
        context.effect(|| self.commit_block_buffer(buf))
    }

    pub(super) fn guarded_setxattr_bytes(
        &self,
        path: &[u8],
        name: &str,
        value: &[u8],
    ) -> Result<()> {
        let mut context = self.chmod_operation()?;
        self.setxattr_bytes_in(&mut context, path, name, value)
    }

    fn setxattr_bytes_in(
        &self,
        context: &mut MountOperation<'_>,
        path: &[u8],
        name: &str,
        value: &[u8],
    ) -> Result<()> {
        self.check_chmod_context(context)?;
        self.write_refusal()?;
        let Some(buf) = self.plan_setxattr(path, name, value)? else {
            return Ok(());
        };
        self.commit_setxattr_in(context, buf)
    }

    /// Setter-only whole-plan boundary. This is not admission or a sealed plan.
    fn commit_setxattr_in(
        &self,
        context: &mut MountOperation<'_>,
        mut buf: BlockBuffer,
    ) -> Result<()> {
        self.check_chmod_context(context)?;
        self.prepare_setxattr_buffer(&mut buf)?;
        if !self.marked_not_clean.load(Ordering::SeqCst) {
            context.effect(|| self.mark_not_clean_once())?;
        }
        // Includes journal checkpoint, flush and allocation-view publication.
        context.effect(|| self.commit_block_buffer(buf))
    }

    /// The planner may have read the primary superblock while still clean.
    /// Normalize that owned image, preserving its planned counters and all
    /// unrelated bytes. Never replace it with a fresh disk or mount snapshot.
    fn prepare_setxattr_buffer(&self, buf: &mut BlockBuffer) -> Result<()> {
        let bs = u64::from(self.sb.block_size());
        let offset = crate::superblock::SUPERBLOCK_OFFSET;
        let Some(block) = buf.dirty.get_mut(&(offset / bs)) else {
            return Ok(()); // Inode-only plans need no superblock write.
        };
        let start = (offset % bs) as usize;
        if block.len() != bs as usize || start + 1024 > block.len() {
            return Err(Error::Corrupt("setxattr prepared superblock shape"));
        }
        let sb = &mut block[start..start + 1024];
        if u16::from_le_bytes(sb[0x38..0x3a].try_into().unwrap()) != crate::superblock::EXT4_MAGIC {
            return Err(Error::Corrupt("setxattr prepared superblock magic"));
        }
        let state = u16::from_le_bytes(sb[0x3a..0x3c].try_into().unwrap());
        sb[0x3a..0x3c].copy_from_slice(&(state & !crate::superblock::EXT4_VALID_FS).to_le_bytes());
        if self.csum.enabled {
            let checksum = crate::checksum::linux_crc32c(!0, &sb[..0x3fc]);
            sb[0x3fc..].copy_from_slice(&checksum.to_le_bytes());
        }
        Ok(())
    }

    pub(super) fn guarded_removexattr_bytes(&self, path: &[u8], name: &str) -> Result<()> {
        let mut context = self.chmod_operation()?;
        self.removexattr_bytes_in(&mut context, path, name)
    }

    fn removexattr_bytes_in(
        &self,
        context: &mut MountOperation<'_>,
        path: &[u8],
        name: &str,
    ) -> Result<()> {
        self.check_chmod_context(context)?;
        self.write_refusal()?;
        let buf = self.plan_removexattr(path, name)?;
        self.commit_removexattr_in(context, buf)
    }

    /// Only the removal planner calls this whole-buffer boundary. This is not
    /// transaction admission, a sealed plan, or a change to raw commit semantics.
    fn commit_removexattr_in(
        &self,
        context: &mut MountOperation<'_>,
        mut buf: BlockBuffer,
    ) -> Result<()> {
        self.check_chmod_context(context)?;
        self.prepare_removexattr_buffer(&mut buf)?;
        if !self.marked_not_clean.load(Ordering::SeqCst) {
            context.effect(|| self.mark_not_clean_once())?;
        }
        // Include checkpoint/flush and post-durability allocation publication;
        // even a publication unwind must retire later writes and clean close.
        context.effect(|| self.commit_block_buffer(buf))
    }

    /// The planner may have read the primary superblock while still clean.
    /// Normalize that owned image, preserving its planned counters and all
    /// unrelated bytes. Never replace it with a fresh disk or mount snapshot.
    fn prepare_removexattr_buffer(&self, buf: &mut BlockBuffer) -> Result<()> {
        let bs = u64::from(self.sb.block_size());
        let offset = crate::superblock::SUPERBLOCK_OFFSET;
        let Some(block) = buf.dirty.get_mut(&(offset / bs)) else {
            return Ok(()); // Inode-only plans need no superblock write.
        };
        let start = (offset % bs) as usize;
        if block.len() != bs as usize || start + 1024 > block.len() {
            return Err(Error::Corrupt("removexattr prepared superblock shape"));
        }
        let sb = &mut block[start..start + 1024];
        if u16::from_le_bytes(sb[0x38..0x3a].try_into().unwrap()) != crate::superblock::EXT4_MAGIC {
            return Err(Error::Corrupt("removexattr prepared superblock magic"));
        }
        let state = u16::from_le_bytes(sb[0x3a..0x3c].try_into().unwrap());
        sb[0x3a..0x3c].copy_from_slice(&(state & !crate::superblock::EXT4_VALID_FS).to_le_bytes());
        if self.csum.enabled {
            let checksum = crate::checksum::linux_crc32c(!0, &sb[..0x3fc]);
            sb[0x3fc..].copy_from_slice(&checksum.to_le_bytes());
        }
        Ok(())
    }

    pub(super) fn guarded_flags_bytes(&self, path: &[u8], flags: u32) -> Result<()> {
        let mut context = self.chmod_operation()?;
        self.flags_bytes_in(&mut context, path, flags)
    }

    fn flags_bytes_in(
        &self,
        context: &mut MountOperation<'_>,
        path: &[u8],
        flags: u32,
    ) -> Result<()> {
        self.check_chmod_context(context)?;
        self.write_refusal()?;
        // The inode's current flags are needed for policy; missing path still
        // wins over a requested managed-bit change. Neither refusal marks disk.
        let ino = self.resolve(path)?;
        let raw = self.plan_set_flags(ino, flags)?;
        self.commit_metadata_in(context, ino, &raw)
    }

    pub(super) fn guarded_utimens_bytes(&self, path: &[u8], times: [(i64, u32); 2]) -> Result<()> {
        let mut context = self.chmod_operation()?;
        self.utimens_bytes_in(&mut context, path, times)
    }

    pub(super) fn guarded_utimens_from(
        &self,
        r: impl Into<InodeRef>,
        times: [(i64, u32); 2],
    ) -> Result<()> {
        let mut context = self.chmod_operation()?;
        self.write_refusal()?;
        check_times(times)?; // Pure verdict before arbitrary conversion or clock.
        self.utimens_ino_in(&mut context, r.into(), times)
    }

    fn utimens_bytes_in(
        &self,
        context: &mut MountOperation<'_>,
        path: &[u8],
        times: [(i64, u32); 2],
    ) -> Result<()> {
        self.check_chmod_context(context)?;
        self.write_refusal()?;
        // An invalid time remains EINVAL even when the path does not exist.
        check_times(times)?;
        let ino = self.resolve(path)?;
        self.utimens_ino_in(context, ino.into(), times)
    }

    fn utimens_ino_in(
        &self,
        context: &mut MountOperation<'_>,
        r: InodeRef,
        times: [(i64, u32); 2],
    ) -> Result<()> {
        self.check_chmod_context(context)?;
        self.write_refusal()?;
        check_times(times)?;
        let Some(raw) = self.plan_utimens(r, times)? else {
            // Both omitted fields still validated the live target and volume.
            return Ok(());
        };
        self.commit_metadata_in(context, r.ino, &raw)
    }
}

fn check_times(times: [(i64, u32); 2]) -> Result<()> {
    for (sec, nsec) in times {
        TimeUpdate::check(sec, nsec)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::time::Duration;

    struct Device {
        bytes: Mutex<Vec<u8>>,
        calls: [AtomicUsize; 4], // read, write, flush, device properties
        fail_write: AtomicUsize,
        fail_flush: AtomicUsize,
        panic_flush: AtomicUsize,
        writable: AtomicBool,
    }

    impl Device {
        fn counts(&self) -> [usize; 4] {
            std::array::from_fn(|i| self.calls[i].load(Ordering::SeqCst))
        }
        fn reset(&self) {
            for counter in &self.calls {
                counter.store(0, Ordering::SeqCst);
            }
            self.fail_write.store(0, Ordering::SeqCst);
            self.fail_flush.store(0, Ordering::SeqCst);
            self.panic_flush.store(0, Ordering::SeqCst);
        }
        fn bytes(&self) -> Vec<u8> {
            self.bytes.lock().unwrap().clone()
        }
    }

    impl BlockDevice for Device {
        fn read_at(&self, offset: u64, out: &mut [u8]) -> Result<()> {
            self.calls[0].fetch_add(1, Ordering::SeqCst);
            out.copy_from_slice(&self.bytes.lock().unwrap()[offset as usize..][..out.len()]);
            Ok(())
        }
        fn write_at(&self, offset: u64, bytes: &[u8]) -> Result<()> {
            let n = self.calls[1].fetch_add(1, Ordering::SeqCst) + 1;
            self.bytes.lock().unwrap()[offset as usize..][..bytes.len()].copy_from_slice(bytes);
            // Fail after changing bytes: a failed callback need not mean no effect.
            if n == self.fail_write.load(Ordering::SeqCst) {
                return Err(std::io::Error::other("injected write failure").into());
            }
            Ok(())
        }
        fn flush(&self) -> Result<()> {
            let n = self.calls[2].fetch_add(1, Ordering::SeqCst) + 1;
            assert_ne!(
                n,
                self.panic_flush.load(Ordering::SeqCst),
                "injected flush panic"
            );
            if n == self.fail_flush.load(Ordering::SeqCst) {
                return Err(std::io::Error::other("injected flush failure").into());
            }
            Ok(())
        }
        fn size_bytes(&self) -> u64 {
            self.calls[3].fetch_add(1, Ordering::SeqCst);
            self.bytes.lock().unwrap().len() as u64
        }
        fn is_writable(&self) -> bool {
            self.calls[3].fetch_add(1, Ordering::SeqCst);
            self.writable.load(Ordering::SeqCst)
        }
    }

    struct Clock {
        calls: AtomicUsize,
        now: std::sync::atomic::AtomicI64,
        hook: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
    }

    impl crate::runtime::Runtime for Clock {
        fn now_unix_seconds(&self) -> i64 {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let hook = self.hook.lock().unwrap().clone();
            if let Some(hook) = hook {
                hook();
            }
            self.now.load(Ordering::SeqCst)
        }
        fn next_inode_generation(&self) -> u32 {
            0x12345678
        }
    }

    fn scratch(flavor: features::FsFlavor) -> (Arc<Device>, Filesystem, u32) {
        scratch_with_runtime(flavor, Arc::new(crate::runtime::SystemRuntime))
    }

    fn with_clock() -> (Arc<Device>, Filesystem, u32, Arc<Clock>) {
        let clock = Arc::new(Clock {
            calls: AtomicUsize::new(0),
            now: std::sync::atomic::AtomicI64::new(1_700_000_000),
            hook: Mutex::new(None),
        });
        let (dev, fs, ino) = scratch_with_runtime(features::FsFlavor::Ext4, clock.clone());
        clock.now.store((1i64 << 31) + 10, Ordering::SeqCst);
        clock.calls.store(0, Ordering::SeqCst);
        (dev, fs, ino, clock)
    }

    fn scratch_with_runtime(
        flavor: features::FsFlavor,
        runtime: Arc<dyn crate::runtime::Runtime>,
    ) -> (Arc<Device>, Filesystem, u32) {
        let size = 32 * 1024 * 1024;
        let dev = Arc::new(Device {
            bytes: Mutex::new(vec![0; size]),
            calls: std::array::from_fn(|_| AtomicUsize::new(0)),
            fail_write: AtomicUsize::new(0),
            fail_flush: AtomicUsize::new(0),
            panic_flush: AtomicUsize::new(0),
            writable: AtomicBool::new(true),
        });
        crate::mkfs::format_filesystem_with_flavor(
            dev.as_ref(),
            None,
            Some([17; 16]),
            size as u64,
            4096,
            flavor,
        )
        .unwrap();
        let fs = Filesystem::mount_with_runtime(dev.clone(), runtime.clone()).unwrap();
        let ino = fs.apply_create_bytes(b"/f\xff", 0o640).unwrap();
        drop(fs);
        let fs = Filesystem::mount_with_runtime(dev.clone(), runtime).unwrap();
        dev.reset();
        (dev, fs, ino)
    }

    #[test]
    fn marker_flush_failure_blocks_legacy_writes_flush_and_clean_drop() {
        let (dev, fs, ino) = scratch(features::FsFlavor::Ext4);
        dev.fail_flush.store(1, Ordering::SeqCst);
        assert!(fs.guarded_chmod_ino(ino.into(), 0o600).is_err());
        dev.reset();
        let bytes = dev.bytes();
        assert!(
            fs.apply_chmod_ino(ino, 0o777).is_err(),
            "legacy write reused failed mount"
        );
        assert!(fs.flush().is_err(), "flush reused failed mount");
        drop(fs);
        assert_eq!(dev.counts(), [0; 4], "faulted close performed callbacks");
        assert_eq!(dev.bytes(), bytes, "faulted close restored clean state");
    }

    fn retired(dev: &Device, fs: Filesystem, ino: u32) {
        assert!(fs.operation_failed.load(Ordering::SeqCst));
        dev.reset();
        let bytes = dev.bytes();
        assert!(fs.guarded_chmod_bytes(b"/f\xff", 0o777).is_err());
        assert!(fs.guarded_chmod_ino(ino.into(), 0o777).is_err());
        assert!(fs.apply_chmod_ino(ino, 0o777).is_err());
        assert!(fs.flush().is_err());
        drop(fs);
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(dev.bytes(), bytes);
    }

    #[test]
    fn byte_path_and_inode_share_mode_type_checksum_and_ctime_semantics() {
        let (dev, fs, ino, clock) = with_clock();
        let before = fs.read_inode_verified(ino).unwrap().0;
        fs.guarded_chmod_bytes(b"/f\xff", 0o16751).unwrap();
        let after = fs.read_inode_verified(ino).unwrap().0;
        assert_eq!(after.mode, crate::inode::S_IFREG | 0o6751);
        assert_eq!(after.ctime, (1i64 << 31) + 10);
        assert_ne!(after.ctime, before.ctime);
        assert_eq!(
            (after.mtime, after.atime, after.crtime, after.generation),
            (before.mtime, before.atime, before.crtime, before.generation)
        );
        fs.guarded_chmod_ino(InodeRef::new(ino, before.generation), 0o600)
            .unwrap();
        assert_eq!(
            fs.read_inode_verified(ino).unwrap().0.mode,
            crate::inode::S_IFREG | 0o600
        );
        assert_eq!(clock.calls.load(Ordering::SeqCst), 2);
        let context = fs.chmod_operation().unwrap();
        assert_eq!(context.operation.status().epoch, 3); // marker + two commits
        assert_eq!(
            context.operation.status().validity,
            crate::operation::Validity::Invalid
        );
        drop(context);
        drop(fs);
        let reopened = Filesystem::mount(dev).unwrap();
        assert_eq!(
            reopened.read_inode_verified(ino).unwrap().0.mode,
            crate::inode::S_IFREG | 0o600
        );
    }

    #[test]
    fn directory_type_survives_private_chmod() {
        let (_, fs, _) = scratch(features::FsFlavor::Ext4);
        fs.guarded_chmod_ino(2.into(), 0o10640).unwrap();
        assert_eq!(
            fs.read_inode_verified(2).unwrap().0.mode,
            crate::inode::S_IFDIR | 0o640
        );
    }

    #[test]
    fn overlap_refuses_both_entries_before_any_callback_or_buffer_change() {
        let (dev, fs, ino, clock) = with_clock();
        let context = fs.chmod_operation().unwrap();
        let status = context.operation.status();
        let bytes = dev.bytes();
        let flags = fs.uninit_cleared.lock().unwrap().cleared.clone();
        assert!(matches!(
            fs.guarded_chmod_bytes(b"/f\xff", 0o600),
            Err(Error::Unsupported(_))
        ));
        assert!(matches!(
            fs.guarded_chmod_ino(ino.into(), 0o600),
            Err(Error::Unsupported(_))
        ));
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert_eq!(dev.bytes(), bytes);
        assert_eq!(fs.uninit_cleared.lock().unwrap().cleared, flags);
        assert_eq!(context.operation.status(), status);
        drop(context);
        fs.guarded_chmod_ino(ino.into(), 0o600).unwrap();
    }

    #[test]
    fn foreign_mount_context_refuses_before_any_callback() {
        let (a, first, _, clock_a) = with_clock();
        let (b, second, ino, clock_b) = with_clock();
        let mut context = first.chmod_operation().unwrap();
        let status = context.operation.status();
        let bytes_a = a.bytes();
        let bytes_b = b.bytes();
        assert!(matches!(
            second.chmod_bytes_in(&mut context, b"/f\xff", 0o600),
            Err(Error::Stale)
        ));
        assert!(matches!(
            second.chmod_ino_in(&mut context, ino.into(), 0o600),
            Err(Error::Stale)
        ));
        assert_eq!((a.counts(), b.counts()), ([0; 4], [0; 4]));
        assert_eq!(
            (
                clock_a.calls.load(Ordering::SeqCst),
                clock_b.calls.load(Ordering::SeqCst)
            ),
            (0, 0)
        );
        assert_eq!((a.bytes(), b.bytes()), (bytes_a, bytes_b));
        assert_eq!(context.operation.status(), status);
    }

    #[test]
    fn callback_worker_reentry_refuses_without_deadlock_or_extra_callbacks() {
        let (dev, fs, ino, clock) = with_clock();
        let fs = Arc::new(fs);
        let weak = Arc::downgrade(&fs);
        let observed_dev = dev.clone();
        *clock.hook.lock().unwrap() = Some(Arc::new(move || {
            let fs = weak.upgrade().unwrap();
            let before = observed_dev.counts();
            let (send, receive) = mpsc::channel();
            let worker = std::thread::spawn(move || {
                send.send(matches!(
                    fs.guarded_chmod_bytes(b"/f\xff", 0o777),
                    Err(Error::Unsupported(_))
                ))
                .unwrap();
            });
            assert!(receive
                .recv_timeout(Duration::from_secs(5))
                .expect("callback worker blocked"));
            worker.join().unwrap();
            assert_eq!(observed_dev.counts(), before);
        }));
        fs.guarded_chmod_ino(ino.into(), 0o600).unwrap();
        assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn concurrent_mount_entries_allow_only_one_held_context() {
        let (dev, fs, ino) = scratch(features::FsFlavor::Ext4);
        let fs = Arc::new(fs);
        let held = fs.chmod_operation().unwrap();
        let (send, receive) = mpsc::channel();
        let workers: Vec<_> = (0..4)
            .map(|_| {
                let fs = fs.clone();
                let send = send.clone();
                std::thread::spawn(move || {
                    send.send(matches!(
                        fs.guarded_chmod_ino(ino.into(), 0o600),
                        Err(Error::Unsupported(_))
                    ))
                    .unwrap()
                })
            })
            .collect();
        for _ in 0..4 {
            assert!(receive.recv_timeout(Duration::from_secs(5)).unwrap());
        }
        for worker in workers {
            worker.join().unwrap();
        }
        assert_eq!(dev.counts(), [0; 4]);
        drop(held);
        fs.guarded_chmod_ino(ino.into(), 0o600).unwrap();
    }

    #[test]
    fn stale_generation_and_missing_path_leave_device_flags_and_health_untouched() {
        let (dev, fs, ino, clock) = with_clock();
        let bytes = dev.bytes();
        let flags = fs.uninit_cleared.lock().unwrap().cleared.clone();
        assert!(matches!(
            fs.guarded_chmod_ino(InodeRef::new(ino, 0), 0o600),
            Err(Error::Stale)
        ));
        assert!(matches!(
            fs.guarded_chmod_bytes(b"/missing", 0o600),
            Err(Error::NotFound)
        ));
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert_eq!(&dev.counts()[1..3], &[0, 0]);
        assert_eq!(dev.bytes(), bytes);
        assert_eq!(fs.uninit_cleared.lock().unwrap().cleared, flags);
        let context = fs.chmod_operation().unwrap();
        assert_eq!(context.operation.status().epoch, 0);
        drop(context);
        fs.guarded_chmod_ino(ino.into(), 0o600).unwrap();
    }

    #[test]
    fn readonly_casefold_and_unsupported_features_refuse_before_planning() {
        for profile in 0..4 {
            let (dev, mut fs, ino, clock) = with_clock();
            match profile {
                0 => dev.writable.store(false, Ordering::SeqCst),
                1 => fs.sb.feature_incompat |= features::Incompat::CASEFOLD.bits(),
                2 => fs.sb.feature_ro_compat |= features::RoCompat::QUOTA.bits(),
                _ => fs.replay_pending.store(true, Ordering::SeqCst),
            }
            let bytes = dev.bytes();
            assert!(fs.guarded_chmod_ino(ino.into(), 0o600).is_err());
            assert!(fs.guarded_chmod_bytes(b"/f\xff", 0o600).is_err());
            assert_eq!(&dev.counts()[..3], &[0; 3]);
            assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
            assert_eq!(dev.bytes(), bytes);
            assert!(!fs.marked_not_clean.load(Ordering::SeqCst));
            assert!(!fs.operation_failed.load(Ordering::SeqCst));
        }
    }

    fn all_faults(flavor: features::FsFlavor, flush: bool) {
        let (dev, fs, ino) = scratch(flavor);
        fs.guarded_chmod_ino(ino.into(), 0o600).unwrap();
        let total = dev.counts()[if flush { 2 } else { 1 }];
        assert!(total >= 2, "marker and commit must both execute");
        drop(fs);
        for nth in 1..=total {
            let (dev, fs, ino) = scratch(flavor);
            if flush {
                &dev.fail_flush
            } else {
                &dev.fail_write
            }
            .store(nth, Ordering::SeqCst);
            assert!(
                fs.guarded_chmod_ino(ino.into(), 0o600).is_err(),
                "fault {nth}/{total} was not reached"
            );
            if nth == 1 {
                assert!(!fs.direct_commit_failed.load(Ordering::SeqCst));
            }
            retired(&dev, fs, ino);
        }
    }

    #[test]
    fn every_direct_marker_and_commit_write_failure_retires_mount() {
        all_faults(features::FsFlavor::Ext4, false);
    }
    #[test]
    fn every_direct_marker_and_commit_flush_failure_retires_mount() {
        all_faults(features::FsFlavor::Ext4, true);
    }
    #[test]
    fn every_journal_marker_and_commit_write_failure_retires_mount() {
        all_faults(features::FsFlavor::Ext3, false);
    }
    #[test]
    fn every_journal_marker_and_commit_flush_failure_retires_mount() {
        all_faults(features::FsFlavor::Ext3, true);
    }

    #[test]
    fn caught_effect_panic_latches_even_while_context_remains_alive() {
        let (dev, fs, ino) = scratch(features::FsFlavor::Ext4);
        let mut context = fs.chmod_operation().unwrap();
        dev.panic_flush.store(1, Ordering::SeqCst);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fs.chmod_ino_in(
                &mut context,
                ino.into(),
                0o600
            )))
            .is_err()
        );
        assert_eq!(context.operation.status().health, Health::Uncertain);
        assert!(fs.operation_failed.load(Ordering::SeqCst));
        assert!(fs.flush().is_err());
        drop(context); // The mutex need not be poisoned; health still forbids reuse.
        retired(&dev, fs, ino);
    }

    #[test]
    fn marker_and_commit_unwind_latch_before_outer_catcher() {
        for nth in [1, 2] {
            let (dev, fs, ino) = scratch(features::FsFlavor::Ext4);
            dev.panic_flush.store(nth, Ordering::SeqCst);
            assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || fs.guarded_chmod_ino(ino.into(), 0o600)
            ))
            .is_err());
            assert!(matches!(fs.operation.try_enter(), Err(Error::Corrupt(_))));
            retired(&dev, fs, ino);
        }
    }

    #[test]
    fn planning_callback_unwind_also_retires_mount() {
        let (dev, fs, ino, clock) = with_clock();
        *clock.hook.lock().unwrap() = Some(Arc::new(|| panic!("injected clock panic")));
        let before = dev.bytes();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || fs.guarded_chmod_bytes(b"/f\xff", 0o600)
        ))
        .is_err());
        assert_eq!(dev.bytes(), before);
        retired(&dev, fs, ino);
    }

    #[test]
    fn refresh_and_new_control_cannot_clear_mount_failure() {
        let (dev, mut fs, ino) = scratch(features::FsFlavor::Ext4);
        dev.fail_flush.store(1, Ordering::SeqCst);
        assert!(fs.guarded_chmod_ino(ino.into(), 0o600).is_err());
        dev.reset();
        fs.refresh_metadata().unwrap(); // Maintenance remains externally serialized.
        fs.operation = crate::operation::Control::new();
        retired(&dev, fs, ino);
    }

    #[test]
    fn later_planning_refusal_keeps_prior_effect_versions_and_dirty_marker() {
        let (dev, fs, ino) = scratch(features::FsFlavor::Ext4);
        let mut context = fs.chmod_operation().unwrap();
        fs.chmod_ino_in(&mut context, ino.into(), 0o600).unwrap();
        let status = context.operation.status();
        let bytes = dev.bytes();
        assert!(matches!(
            fs.chmod_bytes_in(&mut context, b"/missing", 0o777),
            Err(Error::NotFound)
        ));
        assert_eq!(context.operation.status(), status);
        assert_eq!(dev.bytes(), bytes);
        assert!(fs.marked_not_clean.load(Ordering::SeqCst));
        drop(context);
        assert!(!fs.operation_failed.load(Ordering::SeqCst));
        drop(fs);
        let bytes = dev.bytes();
        let state = u16::from_le_bytes(bytes[1024 + 0x3a..1024 + 0x3c].try_into().unwrap());
        assert_ne!(state & crate::superblock::EXT4_VALID_FS, 0);
    }

    mod chown_calls;
    mod flags_calls;
    mod public_calls;
    mod removexattr_calls;
    mod setxattr_calls;
    mod utimens_calls;
}

#[cfg(test)]
mod needs_host;

#[cfg(test)]
mod owner_report;

#[cfg(test)]
mod timestamp_report;

#[cfg(test)]
mod zero_extra_fixture;
