//! Promoted audit controls: public removal and the guarded whole-buffer boundary.
//! Constructed counter controls do not establish independent oracle qualification.
use super::*;
use crate::capi::*;
use std::ffi::CString;

const NAME: &str = "user.audit";
fn specimen(
    flavor: features::FsFlavor,
    bs: u32,
    external: bool,
) -> (Arc<Device>, Filesystem, u32, Arc<Clock>) {
    let size = if bs == 1024 {
        8 * 1024 * 1024
    } else {
        32 * 1024 * 1024
    };
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
        Some([29; 16]),
        size as u64,
        bs,
        flavor,
    )
    .unwrap();
    let clock = Arc::new(Clock {
        calls: AtomicUsize::new(0),
        now: std::sync::atomic::AtomicI64::new(1_700_000_000),
        hook: Mutex::new(None),
    });
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
    let ino = fs.apply_create_bytes(b"/f\xff", 0o640).unwrap();
    let value = vec![b'v'; if external { 600 } else { 1 }];
    fs.apply_setxattr_bytes(b"/f\xff", NAME, &value).unwrap();
    fs.apply_setxattr("/", NAME, b"root").unwrap();
    if external {
        assert_ne!(fs.read_inode_verified(ino).unwrap().0.file_acl, 0);
    }
    drop(fs);
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
    dev.reset();
    clock.calls.store(0, Ordering::SeqCst);
    assert_ne!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
    (dev, fs, ino, clock)
}
fn state(dev: &Device) -> u16 {
    let b = dev.bytes();
    u16::from_le_bytes(b[1082..1084].try_into().unwrap())
}
fn free_blocks(dev: &Device) -> u64 {
    let b = dev.bytes();
    u64::from(u32::from_le_bytes(b[1036..1040].try_into().unwrap()))
        | (u64::from(u32::from_le_bytes(b[1368..1372].try_into().unwrap())) << 32)
}
fn c_remove(handle: *mut fs_ext4_fs_t, name: &str) -> i32 {
    let path = CString::new(b"/f\xff".as_slice()).unwrap();
    let name = CString::new(name).unwrap();
    unsafe { fs_ext4_removexattr(handle, path.as_ptr(), name.as_ptr()) }
}
#[test]
fn rust_string_removal_must_refuse_held_context_before_callbacks() {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096, false);
    let held = fs.chmod_operation().unwrap();
    assert!(
        matches!(fs.apply_removexattr("/", NAME), Err(Error::Unsupported(_))),
        "public string removal bypassed held context"
    );
    assert_eq!(dev.counts(), [0; 4]);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
    drop(held);
}
#[test]
fn rust_byte_removal_must_refuse_held_context_before_callbacks() {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096, false);
    let held = fs.chmod_operation().unwrap();
    assert!(
        matches!(
            fs.apply_removexattr_bytes(b"/f\xff", NAME),
            Err(Error::Unsupported(_))
        ),
        "public byte removal bypassed held context"
    );
    assert_eq!(dev.counts(), [0; 4]);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
    drop(held);
}
#[test]
fn capi_removal_must_refuse_held_context_before_callbacks() {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096, false);
    let _fs = with_test_mount(fs, |handle, fs| {
        let held = fs.chmod_operation().unwrap();
        assert_eq!(
            c_remove(handle, NAME),
            -1,
            "C removal bypassed held context"
        );
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::ENOTSUP);
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        drop(held);
    });
}
#[test]
fn rust_missing_attribute_must_leave_clean_marker_and_bytes_untouched() {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096, false);
    let before = dev.bytes();
    assert!(matches!(
        fs.apply_removexattr_bytes(b"/f\xff", "user.absent"),
        Err(Error::NotFound)
    ));
    assert_eq!(
        &dev.counts()[1..3],
        &[0, 0],
        "missing attribute wrote/flushed recovery marker"
    );
    assert!(dev.bytes() == before);
    assert!(!fs.marked_not_clean.load(Ordering::SeqCst));
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn capi_missing_attribute_must_leave_clean_marker_and_bytes_untouched() {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096, false);
    let before = dev.bytes();
    let _fs = with_test_mount(fs, |handle, fs| {
        assert_eq!(c_remove(handle, "user.absent"), -1);
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::ENOENT);
        assert_eq!(
            &dev.counts()[1..3],
            &[0, 0],
            "C missing attribute wrote/flushed recovery marker"
        );
        assert!(dev.bytes() == before);
        assert!(!fs.marked_not_clean.load(Ordering::SeqCst));
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
    });
}
fn constructed_order(flavor: features::FsFlavor, bs: u32) {
    let (dev, fs, _, _) = specimen(flavor, bs, false);
    let mut context = fs.chmod_operation().unwrap();
    let mut buf = BlockBuffer::new(bs);
    let free = free_blocks(&dev);
    // Controlled synthetic counter edit only, not a complete xattr transaction.
    // It deliberately snapshots a clean primary SB BEFORE the marker effect.
    fs.buffer_patch_sb_counters(&mut buf, -1, 0).unwrap();
    let block = 1024 / u64::from(bs);
    let offset = (1024 % u64::from(bs)) as usize;
    assert_ne!(
        u16::from_le_bytes(
            buf.dirty[&block][offset + 58..offset + 60]
                .try_into()
                .unwrap()
        ) & crate::superblock::EXT4_VALID_FS,
        0
    );
    // The new boundary owns both effects; the raw primitive is unchanged.
    fs.commit_removexattr_in(&mut context, buf).unwrap();
    let final_state = state(&dev);
    let final_free = free_blocks(&dev);
    let bytes = dev.bytes();
    if fs.csum.enabled {
        assert_eq!(
            u32::from_le_bytes(bytes[2044..2048].try_into().unwrap()),
            crate::checksum::linux_crc32c(!0, &bytes[1024..2044])
        );
    }
    assert_eq!(
        final_free,
        free - 1,
        "the planned counter edit was not committed"
    );
    assert!(fs.marked_not_clean.load(Ordering::SeqCst));
    assert_eq!(context.operation.status().epoch, 2);
    eprintln!("constructed pre-marker buffer: bs={bs} journal={} state={final_state} planned_counter_preserved=true marker_owed=true checksummed={}",fs.journal.is_some(),fs.csum.enabled);
    assert_eq!(
        final_state & crate::superblock::EXT4_VALID_FS,
        0,
        "constructed clean SB overwrote the recovery marker"
    );
}
#[test]
fn constructed_direct_1k_buffer_must_preserve_dirty_marker() {
    constructed_order(features::FsFlavor::Ext4, 1024);
}
#[test]
fn constructed_direct_4k_buffer_must_preserve_dirty_marker() {
    constructed_order(features::FsFlavor::Ext4, 4096);
}
#[test]
fn constructed_journal_1k_buffer_must_preserve_dirty_marker() {
    constructed_order(features::FsFlavor::Ext3, 1024);
}
#[test]
fn constructed_journal_4k_buffer_must_preserve_dirty_marker() {
    constructed_order(features::FsFlavor::Ext3, 4096);
}

#[test]
fn legacy_public_external_removal_retains_dirty_marker_in_all_four_profiles() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            let (dev, fs, ino, _) = specimen(flavor, bs, true);
            let before = fs.read_inode_verified(ino).unwrap().0;
            let free = free_blocks(&dev);
            fs.apply_removexattr_bytes(b"/f\xff", NAME).unwrap();
            let after = fs.read_inode_verified(ino).unwrap().0;
            assert_eq!(after.file_acl, 0);
            assert_eq!(after.blocks, before.blocks - u64::from(bs) / 512);
            assert_eq!(free_blocks(&dev), free + 1);
            assert_eq!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
            eprintln!("legacy public removal: bs={bs} journal={} marker remains dirty, external block freed",fs.journal.is_some());
        }
    }
}

#[test]
fn normalization_preserves_planned_bytes_and_other_state_bits() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            for prior in [false, true] {
                let (dev, fs, ino, _) = specimen(flavor, bs, false);
                if prior {
                    fs.apply_chmod_ino(ino, 0o600).unwrap();
                }
                let mut buf = BlockBuffer::new(bs);
                fs.buffer_patch_sb_counters(&mut buf, -1, -1).unwrap();
                let block = 1024 / u64::from(bs);
                let offset = (1024 % u64::from(bs)) as usize;
                let staged = buf.dirty.get_mut(&block).unwrap();
                staged[offset + 0x3a..offset + 0x3c].copy_from_slice(&0xa5a7u16.to_le_bytes());
                if bs == 4096 {
                    staged[12] = 0x5a;
                    staged[3000] = 0x6b;
                }
                let before = staged.clone();
                let mut wanted = before.clone();
                wanted[offset + 0x3a..offset + 0x3c].copy_from_slice(&0xa5a6u16.to_le_bytes());
                if fs.csum.enabled {
                    let sum = crate::checksum::linux_crc32c(!0, &wanted[offset..offset + 0x3fc]);
                    wanted[offset + 0x3fc..offset + 1024].copy_from_slice(&sum.to_le_bytes());
                }
                dev.reset();
                fs.prepare_removexattr_buffer(&mut buf).unwrap();
                assert_eq!(buf.dirty[&block], wanted);
                assert_eq!(dev.counts(), [0; 4], "normalization read or wrote disk");
                // The same operation with an inode-only plan must add no SB block.
                let mut inode_only = fs.plan_removexattr(b"/f\xff", NAME).unwrap();
                assert!(!inode_only.dirty.contains_key(&block) || fs.sb.inode_size == 128);
                let blocks: Vec<_> = inode_only.dirty.keys().copied().collect();
                fs.prepare_removexattr_buffer(&mut inode_only).unwrap();
                assert_eq!(inode_only.dirty.keys().copied().collect::<Vec<_>>(), blocks);
            }
        }
    }
}

#[test]
fn malformed_buffer_and_foreign_context_refuse_before_effects() {
    let (_, first, _, _) = specimen(features::FsFlavor::Ext4, 4096, true);
    let (dev, second, _, clock) = specimen(features::FsFlavor::Ext4, 4096, true);
    let mut foreign = first.chmod_operation().unwrap();
    let before = dev.bytes();
    assert!(matches!(
        second.removexattr_bytes_in(&mut foreign, b"/f\xff", NAME),
        Err(Error::Stale)
    ));
    assert!(matches!(
        second.commit_removexattr_in(&mut foreign, BlockBuffer::new(4096)),
        Err(Error::Stale)
    ));
    assert_eq!(dev.counts(), [0; 4]);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
    let mut context = second.chmod_operation().unwrap();
    let status = context.operation.status();
    for bytes in [vec![], vec![0; 1024], vec![0; 4096], vec![0; 4097]] {
        let mut buf = BlockBuffer::new(4096);
        buf.put(0, bytes);
        assert!(matches!(
            second.commit_removexattr_in(&mut context, buf),
            Err(Error::Corrupt(_))
        ));
        assert_eq!(context.operation.status(), status);
        assert!(!second.operation_failed.load(Ordering::SeqCst));
    }
    assert_eq!(dev.counts(), [0; 4]);
    assert!(dev.bytes() == before);
}

#[test]
fn owed_marker_is_not_repeated_and_planning_refusals_preserve_state() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            for prior in [false, true] {
                let (dev, fs, ino, clock) = specimen(flavor, bs, true);
                if prior {
                    fs.apply_chown_ino(ino, 3, 4).unwrap();
                }
                let status = fs.chmod_operation().unwrap().operation.status();
                let before = dev.bytes();
                let cleared = fs.uninit_cleared.lock().unwrap().cleared.clone();
                dev.reset();
                clock.calls.store(0, Ordering::SeqCst);
                let fs = with_test_mount(fs, |handle, fs| {
                    for (path, name) in [
                        (b"/missing".as_slice(), "no.prefix"),
                        (b"/f\xff", "user.absent"),
                    ] {
                        assert!(matches!(
                            fs.apply_removexattr_bytes(path, name),
                            Err(Error::NotFound)
                        ));
                    }
                    // Legacy has no inline area, but the external decoder validates prefixes.
                    assert!(matches!(
                        fs.apply_removexattr_bytes(b"/f\xff", "no.prefix"),
                        Err(Error::InvalidArgument(_))
                    ));
                    assert_eq!(c_remove(handle, "user.absent"), -1);
                    assert_eq!(fs_ext4_last_errno(), crate::error::errno::ENOENT);
                    assert!(dev.bytes() == before);
                    assert_eq!(&dev.counts()[1..3], &[0, 0]);
                    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
                    assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
                    assert_eq!(fs.uninit_cleared.lock().unwrap().cleared, cleared);
                    assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), prior);
                    assert_eq!(c_remove(handle, NAME), 0);
                    assert_eq!(fs_ext4_last_errno(), 0);
                    assert!(unsafe { std::ffi::CStr::from_ptr(fs_ext4_last_error()) }
                        .to_bytes()
                        .is_empty());
                    assert_eq!(
                        fs.chmod_operation().unwrap().operation.status().epoch,
                        status.epoch + if prior { 1 } else { 2 }
                    );
                });
                drop(fs);
                let fs = Filesystem::mount(dev.clone()).unwrap();
                assert_eq!(fs.read_inode_verified(ino).unwrap().0.file_acl, 0);
            }
        }
    }
}

#[test]
fn volume_refusals_precede_lookup_and_clock_on_clean_and_dirty_mounts() {
    for prior in [false, true] {
        for profile in 0..4 {
            let (dev, mut fs, ino, clock) = specimen(features::FsFlavor::Ext4, 4096, true);
            if prior {
                fs.apply_chmod_ino(ino, 0o600).unwrap();
            }
            match profile {
                0 => dev.writable.store(false, Ordering::SeqCst),
                1 => fs.sb.feature_incompat |= features::Incompat::CASEFOLD.bits(),
                2 => fs.sb.feature_ro_compat |= features::RoCompat::QUOTA.bits(),
                _ => fs.replay_pending.store(true, Ordering::SeqCst),
            }
            let status = fs.chmod_operation().unwrap().operation.status();
            let before = dev.bytes();
            dev.reset();
            clock.calls.store(0, Ordering::SeqCst);
            let _fs = with_test_mount(fs, |handle, fs| {
                assert!(fs.apply_removexattr_bytes(b"/missing", NAME).is_err());
                assert_eq!(c_remove(handle, NAME), -1);
                assert_eq!(&dev.counts()[..3], &[0; 3]);
                assert!(dev.bytes() == before);
                assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
                assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
                assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), prior);
                assert!(!fs.operation_failed.load(Ordering::SeqCst));
            });
        }
    }
}

#[derive(Default)]
struct Workers(Arc<Mutex<Vec<std::thread::JoinHandle<bool>>>>);
impl Workers {
    fn finish(&self) {
        let workers = std::mem::take(&mut *self.0.lock().unwrap());
        let results: Vec<_> = workers.into_iter().map(|w| w.join()).collect();
        assert!(results.into_iter().all(|r| matches!(r, Ok(true))));
    }
}
impl Drop for Workers {
    fn drop(&mut self) {
        for w in self.0.lock().unwrap().drain(..) {
            let _ = w.join();
        }
    }
}

#[test]
fn rust_workers_refuse_all_five_families_in_both_directions() {
    for outer in 0..6 {
        let (dev, fs, ino, clock) = specimen(features::FsFlavor::Ext4, 4096, true);
        let flags = fs.read_inode_verified(ino).unwrap().0.flags;
        let fs = Arc::new(fs);
        let weak = Arc::downgrade(&fs);
        let workers = Workers::default();
        let handles = workers.0.clone();
        let responsive = Arc::new(AtomicBool::new(true));
        let seen = responsive.clone();
        let observed = dev.clone();
        *clock.hook.lock().unwrap() = Some(Arc::new(move || {
            let fs = weak.upgrade().unwrap();
            let before = observed.counts();
            let observed = observed.clone();
            let (send, recv) = mpsc::channel();
            handles.lock().unwrap().push(std::thread::spawn(move || {
                let results = [
                    fs.apply_chmod("/", 0o777),
                    fs.apply_chmod_bytes(b"/f\xff", 0o777),
                    fs.apply_chmod_ino(ino, 0o777),
                    fs.apply_chown("/", 1, 2),
                    fs.apply_chown_bytes(b"/f\xff", 1, 2),
                    fs.apply_chown_ino(ino, 1, 2),
                    fs.apply_utimens("/", 5, 0, 6, 0),
                    fs.apply_utimens_bytes(b"/f\xff", 5, 0, 6, 0),
                    fs.apply_utimens_ino(ino, 5, 0, 6, 0),
                    fs.apply_set_flags("/", flags),
                    fs.apply_set_flags_bytes(b"/f\xff", flags),
                    fs.apply_removexattr("/", NAME),
                    fs.apply_removexattr_bytes(b"/f\xff", NAME),
                    fs.apply_setxattr("/", NAME, b"x"),
                    fs.apply_setxattr_bytes(b"/f\xff", NAME, b"x"),
                ];
                let ok = results
                    .into_iter()
                    .all(|r| matches!(r, Err(Error::Unsupported(_))))
                    && observed.counts() == before;
                let _ = send.send(());
                ok
            }));
            if recv.recv_timeout(Duration::from_secs(5)).is_err() {
                seen.store(false, Ordering::SeqCst);
            }
        }));
        let result = match outer {
            0 => fs.apply_chmod_ino(ino, 0o600),
            1 => fs.apply_chown_ino(ino, 1, 2),
            2 => fs.apply_utimens_ino(ino, 5, 0, 6, 0),
            3 => fs.apply_set_flags_bytes(b"/f\xff", flags),
            4 => fs.apply_removexattr_bytes(b"/f\xff", NAME),
            _ => fs.apply_setxattr_bytes(b"/f\xff", NAME, b"x"),
        };
        *clock.hook.lock().unwrap() = None;
        workers.finish();
        assert!(responsive.load(Ordering::SeqCst));
        result.unwrap();
    }
}

#[test]
fn capi_workers_refuse_all_five_families_and_keep_thread_local_errors() {
    for outer in 0..6 {
        let (dev, fs, ino, clock) = specimen(features::FsFlavor::Ext4, 4096, true);
        let flags = fs.read_inode_verified(ino).unwrap().0.flags;
        let _fs = with_test_mount(fs, |handle, _| {
            let workers = Workers::default();
            let handles = workers.0.clone();
            let responsive = Arc::new(AtomicBool::new(true));
            let seen = responsive.clone();
            let observed = dev.clone();
            let address = handle as usize;
            *clock.hook.lock().unwrap() = Some(Arc::new(move || {
                let before = observed.counts();
                let observed = observed.clone();
                let (send, recv) = mpsc::channel();
                handles.lock().unwrap().push(std::thread::spawn(move || {
                    let handle = address as *mut fs_ext4_fs_t;
                    let path = CString::new(b"/f\xff".as_slice()).unwrap();
                    let mut ok = true;
                    for entry in 0..9 {
                        let result = unsafe {
                            match entry {
                                0 => fs_ext4_chmod(handle, path.as_ptr(), 0o777),
                                1 => fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o777),
                                2 => fs_ext4_chown(handle, path.as_ptr(), 1, 2),
                                3 => fs_ext4_chown_ino(handle, ino, FS_EXT4_GEN_ANY, 1, 2),
                                4 => fs_ext4_utimens(handle, path.as_ptr(), 5, 0, 6, 0),
                                5 => fs_ext4_utimens_ino(handle, ino, FS_EXT4_GEN_ANY, 5, 0, 6, 0),
                                6 => fs_ext4_set_flags(handle, path.as_ptr(), flags),
                                7 => c_remove(handle, NAME),
                                _ => super::setxattr_calls::c_set(handle, b"/f\xff", NAME, b"x"),
                            }
                        };
                        ok &= result == -1 && fs_ext4_last_errno() == crate::error::errno::ENOTSUP;
                    }
                    ok &= observed.counts() == before;
                    let _ = send.send(());
                    ok
                }));
                if recv.recv_timeout(Duration::from_secs(5)).is_err() {
                    seen.store(false, Ordering::SeqCst);
                }
            }));
            let result = unsafe {
                match outer {
                    0 => fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o600),
                    1 => fs_ext4_chown_ino(handle, ino, FS_EXT4_GEN_ANY, 1, 2),
                    2 => fs_ext4_utimens_ino(handle, ino, FS_EXT4_GEN_ANY, 5, 0, 6, 0),
                    3 => fs_ext4_set_flags(
                        handle,
                        CString::new(b"/f\xff".as_slice()).unwrap().as_ptr(),
                        flags,
                    ),
                    4 => c_remove(handle, NAME),
                    _ => super::setxattr_calls::c_set(handle, b"/f\xff", NAME, b"x"),
                }
            };
            let error = fs_ext4_last_errno();
            *clock.hook.lock().unwrap() = None;
            workers.finish();
            assert!(responsive.load(Ordering::SeqCst));
            assert_eq!((result, error), (0, 0));
        });
    }
}

pub(super) fn retired_five(dev: &Device, fs: Filesystem, ino: u32) {
    assert!(fs.operation_failed.load(Ordering::SeqCst));
    dev.reset();
    let before = dev.bytes();
    let fs = with_test_mount(fs, |handle, fs| {
        assert!(fs.apply_removexattr("/", NAME).is_err());
        assert!(fs.apply_removexattr_bytes(b"/f\xff", NAME).is_err());
        assert!(fs.apply_chmod_ino(ino, 0o777).is_err());
        assert!(fs.apply_chown_ino(ino, 1, 2).is_err());
        assert!(fs.apply_utimens_ino(ino, 5, 0, 6, 0).is_err());
        assert!(fs.apply_set_flags("/", 0).is_err());
        assert!(fs.apply_setxattr("/", NAME, b"x").is_err());
        assert!(fs.apply_setxattr_bytes(b"/f\xff", NAME, b"x").is_err());
        for family in 0..6 {
            let path = CString::new(b"/f\xff".as_slice()).unwrap();
            let result = unsafe {
                match family {
                    0 => fs_ext4_chmod(handle, path.as_ptr(), 0o777),
                    1 => fs_ext4_chown(handle, path.as_ptr(), 1, 2),
                    2 => fs_ext4_utimens(handle, path.as_ptr(), 5, 0, 6, 0),
                    3 => fs_ext4_set_flags(handle, path.as_ptr(), 0),
                    4 => c_remove(handle, NAME),
                    _ => super::setxattr_calls::c_set(handle, b"/f\xff", NAME, b"x"),
                }
            };
            assert_eq!(result, -1);
            assert_eq!(fs_ext4_last_errno(), crate::error::errno::EROFS);
        }
        assert_eq!(unsafe { fs_ext4_flush(handle) }, -1);
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
    });
    unmount_test_mount(fs);
    assert_eq!(dev.counts(), [0; 4]);
    assert!(dev.bytes() == before, "retired close or family wrote disk");
}

fn every_fault(capi: bool, flush: bool) {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            let branches = if matches!(flavor, features::FsFlavor::Ext4) {
                0..6
            } else {
                0..4
            };
            for branch in branches {
                let (dev, fs, _, _, _) = branch_specimen(flavor, bs, branch);
                dev.reset();
                fs.apply_removexattr_bytes(b"/f\xff", NAME).unwrap();
                let index = if flush { 2 } else { 1 };
                let total = dev.counts()[index];
                assert!(total >= 2);
                drop(fs);
                for nth in 1..=total {
                    let (dev, fs, ino, _, _) = branch_specimen(flavor, bs, branch);
                    dev.reset();
                    (if flush {
                        &dev.fail_flush
                    } else {
                        &dev.fail_write
                    })
                    .store(nth, Ordering::SeqCst);
                    let fs = if capi {
                        with_test_mount(fs, |handle, _| {
                            assert_eq!(c_remove(handle, NAME), -1);
                            assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
                        })
                    } else {
                        assert!(fs.apply_removexattr_bytes(b"/f\xff", NAME).is_err());
                        fs
                    };
                    assert!(
                        dev.counts()[index] >= nth,
                        "injected callback was not reached"
                    );
                    retired_five(&dev, fs, ino);
                }
            }
        }
    }
}
#[test]
fn rust_every_multiblock_write_fault_retires_five_families() {
    every_fault(false, false);
}
#[test]
fn rust_every_multiblock_flush_fault_retires_five_families() {
    every_fault(false, true);
}
#[test]
fn capi_every_multiblock_write_fault_retires_five_families() {
    every_fault(true, false);
}
#[test]
fn capi_every_multiblock_flush_fault_retires_five_families() {
    every_fault(true, true);
}

#[test]
fn planning_and_every_effect_flush_panic_retire_five_families_and_c_close() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        let (dev, fs, _, _) = specimen(flavor, 4096, true);
        dev.reset();
        fs.apply_removexattr_bytes(b"/f\xff", NAME).unwrap();
        let total = dev.counts()[2];
        drop(fs);
        for capi in [false, true] {
            for phase in 0..=total {
                let (dev, fs, ino, clock) = specimen(flavor, 4096, true);
                dev.reset();
                let before = dev.bytes();
                if phase == 0 {
                    *clock.hook.lock().unwrap() =
                        Some(Arc::new(|| panic!("removal planning panic")));
                } else {
                    dev.panic_flush.store(phase, Ordering::SeqCst);
                }
                let fs = if capi {
                    with_test_mount(fs, |handle, _| {
                        assert_eq!(c_remove(handle, NAME), -1);
                        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
                        assert!(unsafe { std::ffi::CStr::from_ptr(fs_ext4_last_error()) }
                            .to_string_lossy()
                            .contains("panic:"));
                    })
                } else {
                    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                        || fs.apply_removexattr_bytes(b"/f\xff", NAME)
                    ))
                    .is_err());
                    fs
                };
                if phase == 0 {
                    assert!(dev.bytes() == before);
                    assert_eq!(&dev.counts()[1..3], &[0, 0]);
                } else {
                    assert!(dev.counts()[2] >= phase);
                }
                retired_five(&dev, fs, ino);
            }
        }
    }
}

#[test]
fn publication_unwind_is_inside_effect_even_when_context_is_retained() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        let (dev, fs, ino, _) = specimen(flavor, 4096, false);
        let buf = fs.plan_removexattr(b"/f\xff", NAME).unwrap();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _lock = fs.uninit_cleared.lock().unwrap();
            panic!("poison publication lock");
        }))
        .is_err());
        let mut context = fs.chmod_operation().unwrap();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || fs.commit_removexattr_in(&mut context, buf)
        ))
        .is_err());
        assert_eq!(context.operation.status().health, Health::Uncertain);
        assert!(fs.operation_failed.load(Ordering::SeqCst));
        assert!(fs.flush().is_err());
        assert_eq!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
        drop(context);
        retired_five(&dev, fs, ino);
    }
}

#[test]
fn earlier_family_failure_blocks_removal_and_clean_or_dirty_planning_panic_blocks_close() {
    for family in 0..4 {
        let (dev, fs, ino, _) = specimen(features::FsFlavor::Ext4, 4096, true);
        dev.fail_flush.store(1, Ordering::SeqCst);
        let result = match family {
            0 => fs.apply_chmod_ino(ino, 0o600),
            1 => fs.apply_chown_ino(ino, 1, 2),
            2 => fs.apply_utimens_ino(ino, 5, 0, 6, 0),
            _ => fs.apply_set_flags("/", fs.read_inode_verified(2).unwrap().0.flags | 0x40),
        };
        assert!(result.is_err());
        retired_five(&dev, fs, ino);
    }
    for prior in [false, true] {
        let (dev, fs, ino, clock) = specimen(features::FsFlavor::Ext4, 4096, true);
        if prior {
            fs.apply_chown_ino(ino, 1, 2).unwrap();
        }
        let before = dev.bytes();
        dev.reset();
        *clock.hook.lock().unwrap() = Some(Arc::new(|| panic!("close planning panic")));
        let fs = with_test_mount(fs, |handle, _| {
            assert_eq!(c_remove(handle, NAME), -1);
        });
        assert!(dev.bytes() == before);
        assert_eq!(&dev.counts()[1..3], &[0, 0]);
        assert_eq!(state(&dev) & crate::superblock::EXT4_VALID_FS != 0, !prior);
        retired_five(&dev, fs, ino);
    }
}

#[test]
fn corrupt_and_ea_inode_external_blocks_refuse_without_partial_inline_commit() {
    for prior in [false, true] {
        for profile in 0..4 {
            let (dev, fs, ino, clock) = specimen(features::FsFlavor::Ext4, 4096, true);
            // Deliberately seed the same name inline while retaining its external
            // copy: a later external refusal must discard this local inline edit.
            let (inode, mut raw) = fs.read_inode_verified(ino).unwrap();
            let start = 128 + u16::from_le_bytes(raw[128..130].try_into().unwrap()) as usize;
            crate::xattr::plan_set_in_inode_region(&mut raw[start..], NAME, b"duplicate").unwrap();
            fs.finalize_inode_raw(ino, inode.generation, &mut raw)
                .unwrap();
            fs.write_inode_raw(ino, &raw).unwrap();
            let nr = inode.file_acl;
            let mut block = fs.read_block(nr).unwrap();
            match profile {
                0 => block[0] ^= 1,
                1 => block[8..12].copy_from_slice(&2u32.to_le_bytes()),
                2 => block[4095] ^= 1,
                _ => block[36..40].copy_from_slice(&ino.to_le_bytes()),
            }
            if profile != 2 {
                fs.csum.patch_xattr_block(nr, &mut block);
            }
            fs.dev.write_at(nr * 4096, &block).unwrap();
            drop(fs); // End fixture writes before testing a clean mount.
            let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
            if prior {
                fs.apply_chown_ino(ino, 1, 2).unwrap();
            }
            let before = dev.bytes();
            let status = fs.chmod_operation().unwrap().operation.status();
            let cleared = fs.uninit_cleared.lock().unwrap().cleared.clone();
            dev.reset();
            clock.calls.store(0, Ordering::SeqCst);
            let _fs = with_test_mount(fs, |handle, fs| {
                assert!(fs.apply_removexattr_bytes(b"/f\xff", NAME).is_err());
                assert_eq!(c_remove(handle, NAME), -1);
                assert!(dev.bytes() == before);
                assert_eq!(&dev.counts()[1..3], &[0, 0]);
                assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
                assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
                assert_eq!(fs.uninit_cleared.lock().unwrap().cleared, cleared);
                assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), prior);
            });
        }
    }
}

pub(super) fn branch_specimen(
    flavor: features::FsFlavor,
    bs: u32,
    branch: u8,
) -> (Arc<Device>, Filesystem, u32, Arc<Clock>, Option<u32>) {
    let (dev, fs, ino, clock) = specimen(flavor, bs, true);
    if branch <= 1 {
        fs.apply_setxattr_bytes(b"/f\xff", "user.keep", &[b'k'; 200])
            .unwrap();
    }
    let peer = if branch == 1 || branch == 2 {
        let peer = fs.apply_create("/peer", 0o600).unwrap();
        let (inode, _) = fs.read_inode_verified(ino).unwrap();
        let (other, mut raw) = fs.read_inode_verified(peer).unwrap();
        let mut block = fs.read_block(inode.file_acl).unwrap();
        block[4..8].copy_from_slice(&2u32.to_le_bytes());
        fs.csum.patch_xattr_block(inode.file_acl, &mut block);
        fs.dev
            .write_at(inode.file_acl * u64::from(bs), &block)
            .unwrap();
        Filesystem::write_file_acl(&mut raw, inode.file_acl).unwrap();
        Filesystem::patch_inode_size_and_blocks(
            &mut raw,
            other.size,
            other.blocks + u64::from(bs) / 512,
        )
        .unwrap();
        fs.finalize_inode_raw(peer, other.generation, &mut raw)
            .unwrap();
        fs.write_inode_raw(peer, &raw).unwrap();
        Some(peer)
    } else {
        None
    };
    if branch == 4 || branch == 5 {
        let (inode, mut raw) = fs.read_inode_verified(ino).unwrap();
        assert_eq!(raw.len(), 256);
        if branch == 4 {
            let start = 128 + u16::from_le_bytes(raw[128..130].try_into().unwrap()) as usize;
            crate::xattr::plan_set_in_inode_region(&mut raw[start..], NAME, b"duplicate").unwrap();
        } else {
            super::super::zero_extra_fixture::prepare(&mut raw).unwrap();
        }
        fs.finalize_inode_raw(ino, inode.generation, &mut raw)
            .unwrap();
        fs.write_inode_raw(ino, &raw).unwrap();
    }
    drop(fs);
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
    dev.reset();
    clock.calls.store(0, Ordering::SeqCst);
    (dev, fs, ino, clock, peer)
}
#[test]
fn actual_removal_branches_preserve_counters_sharing_metadata_and_dirty_state() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            // Legacy inodes have no inline area or extra fields to construct.
            let branches = if matches!(flavor, features::FsFlavor::Ext4) {
                0..6
            } else {
                0..4
            };
            for branch in branches {
                for capi in [false, true] {
                    let (dev, fs, ino, clock, peer) = branch_specimen(flavor, bs, branch);
                    let before = fs.read_inode_verified(ino).unwrap().0;
                    let free = free_blocks(&dev);
                    let peer_raw = peer.map(|p| fs.read_inode_verified(p).unwrap().1);
                    clock.now.store(1_700_000_100, Ordering::SeqCst);
                    let fs = with_test_mount(fs, |handle, fs| {
                        if capi {
                            assert_eq!(c_remove(handle, NAME), 0);
                            assert_eq!(fs_ext4_last_errno(), 0);
                        } else {
                            fs.apply_removexattr_bytes(b"/f\xff", NAME).unwrap();
                        }
                        let (after, raw) = fs.read_inode_verified(ino).unwrap();
                        assert_eq!(
                            crate::xattr::get_resolved(fs, &after, &raw, NAME).unwrap(),
                            None
                        );
                        assert_eq!(after.ctime, 1_700_000_100);
                        assert_eq!(after.ctime_nsec, 0);
                        assert_eq!(
                            (
                                after.mode,
                                after.uid,
                                after.gid,
                                after.flags,
                                after.generation,
                                after.size,
                                after.atime,
                                after.mtime,
                                after.crtime
                            ),
                            (
                                before.mode,
                                before.uid,
                                before.gid,
                                before.flags,
                                before.generation,
                                before.size,
                                before.atime,
                                before.mtime,
                                before.crtime
                            )
                        );
                        let refs = |nr| {
                            let b = fs.read_block(nr).unwrap();
                            assert!(fs.csum.verify_xattr_block(nr, &b));
                            u32::from_le_bytes(b[4..8].try_into().unwrap())
                        };
                        match branch {
                            0 | 1 => {
                                assert_ne!(after.file_acl, 0);
                                assert_eq!(after.blocks, before.blocks);
                                assert_eq!(
                                    crate::xattr::get_resolved(fs, &after, &raw, "user.keep")
                                        .unwrap(),
                                    Some(vec![b'k'; 200])
                                );
                                assert_eq!(refs(after.file_acl), 1);
                                if branch == 0 {
                                    assert_eq!(after.file_acl, before.file_acl);
                                    assert_eq!(free_blocks(&dev), free);
                                } else {
                                    assert_ne!(after.file_acl, before.file_acl);
                                    assert_eq!(refs(before.file_acl), 1);
                                    assert_eq!(free_blocks(&dev), free - 1);
                                }
                            }
                            _ => {
                                assert_eq!(after.file_acl, 0);
                                assert_eq!(after.blocks, before.blocks - u64::from(bs) / 512);
                                assert_eq!(free_blocks(&dev), free + u64::from(branch != 2));
                                if branch == 2 {
                                    assert_eq!(refs(before.file_acl), 1);
                                }
                                if branch == 5 {
                                    assert_eq!(&raw[128..130], &[0, 0]);
                                }
                            }
                        }
                        if let Some(p) = peer {
                            let (other, raw) = fs.read_inode_verified(p).unwrap();
                            assert_eq!(Some(raw.clone()), peer_raw);
                            assert_eq!(
                                crate::xattr::get_resolved(fs, &other, &raw, NAME).unwrap(),
                                Some(vec![b'v'; 600])
                            );
                        }
                        assert_eq!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
                        assert_eq!(fs.chmod_operation().unwrap().operation.status().epoch, 2);
                    });
                    drop(fs);
                    assert_ne!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
                    let fs = Filesystem::mount(dev.clone()).unwrap();
                    let (inode, raw) = fs.read_inode_verified(ino).unwrap();
                    assert_eq!(
                        crate::xattr::get_resolved(&fs, &inode, &raw, NAME).unwrap(),
                        None
                    );
                }
            }
        }
    }
}
