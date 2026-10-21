//! Public unlink controls; native evidence is not Linux qualification.
//! Constructed SB credits are ordering diagnostics, not valid deletions.
use super::*;
use crate::capi::*;
use std::ffi::CString;

const NAME: &str = "user.audit";
const STAMP: i64 = 1_700_000_100;

fn specimen(flavor: features::FsFlavor, bs: u32) -> (Arc<Device>, Filesystem, u32, Arc<Clock>) {
    let size = if bs == 1024 { 8 << 20 } else { 32 << 20 };
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
        Some([37; 16]),
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
    fs.apply_create("/ascii", 0o640).unwrap();
    fs.apply_setxattr("/ascii", NAME, b"old").unwrap();
    fs.apply_mkdir("/d", 0o750).unwrap();
    drop(fs);
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
    dev.reset();
    clock.calls.store(0, Ordering::SeqCst);
    assert_ne!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
    (dev, fs, ino, clock)
}
fn state(dev: &Device) -> u16 {
    u16::from_le_bytes(dev.bytes()[1082..1084].try_into().unwrap())
}
fn counters(dev: &Device) -> (u64, u32) {
    let b = dev.bytes();
    (
        u64::from(u32::from_le_bytes(b[1036..1040].try_into().unwrap()))
            | (u64::from(u32::from_le_bytes(b[1368..1372].try_into().unwrap())) << 32),
        u32::from_le_bytes(b[1040..1044].try_into().unwrap()),
    )
}
fn allocated(fs: &Filesystem, block: u64) -> bool {
    let (gi, bit, len) = fs.group_chunks(block, 1).unwrap()[0];
    assert_eq!(len, 1);
    let bitmap = fs.read_block(fs.groups[gi].block_bitmap).unwrap();
    bitmap[(bit / 8) as usize] & (1 << (bit % 8)) != 0
}
fn group_counters(fs: &Filesystem) -> (u64, u32, u32) {
    assert_eq!(fs.groups.len(), 1);
    let (block, off) = fs.sb.descriptor_location(0);
    let bytes = fs.read_block(block).unwrap();
    let word = |at| {
        u32::from(u16::from_le_bytes(
            bytes[off + at..off + at + 2].try_into().unwrap(),
        ))
    };
    let high = |at| {
        if fs.sb.desc_size >= 64 {
            word(at) << 16
        } else {
            0
        }
    };
    (
        u64::from(word(12) | high(44)),
        word(14) | high(46),
        word(16) | high(48),
    )
}
fn c_path(handle: *mut fs_ext4_fs_t, path: &[u8]) -> i32 {
    let path = CString::new(path).unwrap();
    unsafe { fs_ext4_unlink(handle, path.as_ptr()) }
}
fn c_at(handle: *mut fs_ext4_fs_t, dir: InodeRef, name: &[u8]) -> i32 {
    unsafe {
        fs_ext4_unlink_at(
            handle,
            dir.ino,
            dir.generation.unwrap_or(FS_EXT4_GEN_ANY),
            name.as_ptr().cast(),
            name.len(),
        )
    }
}

fn held_entry(which: u8) {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
    let _fs = with_test_mount(fs, |handle, fs| {
        let held = fs.chmod_operation().unwrap();
        let status = held.operation.status();
        let bytes = dev.bytes();
        let refused = match which {
            0 => matches!(fs.apply_unlink("/ascii"), Err(Error::Unsupported(_))),
            1 => matches!(fs.apply_unlink_bytes(b"/f\xff"), Err(Error::Unsupported(_))),
            2 => matches!(
                fs.apply_unlink_at(2u32, b"f\xff"),
                Err(Error::Unsupported(_))
            ),
            3 => {
                c_path(handle, b"/f\xff") == -1
                    && fs_ext4_last_errno() == crate::error::errno::ENOTSUP
            }
            4 => {
                c_at(handle, 2u32.into(), b"f\xff") == -1
                    && fs_ext4_last_errno() == crate::error::errno::ENOTSUP
            }
            _ => unreachable!(),
        };
        eprintln!(
            "held unlink entry={which} refused={refused} callbacks={:?} clock={}",
            dev.counts(),
            clock.calls.load(Ordering::SeqCst)
        );
        assert!(refused, "public unlink bypassed held shared context");
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert_eq!(held.operation.status(), status);
        assert!(dev.bytes() == bytes);
    });
}
macro_rules! held_test {
    ($name:ident, $which:expr) => {
        #[test]
        fn $name() {
            held_entry($which);
        }
    };
}
held_test!(
    rust_string_unlink_must_refuse_held_context_before_callbacks,
    0
);
held_test!(
    rust_bytes_unlink_must_refuse_held_context_before_callbacks,
    1
);
held_test!(
    rust_parent_unlink_must_refuse_held_context_before_callbacks,
    2
);
held_test!(c_path_unlink_must_refuse_held_context_before_callbacks, 3);
held_test!(c_parent_unlink_must_refuse_held_context_before_callbacks, 4);

#[derive(Clone, Copy, Debug)]
enum Refusal {
    Missing,
    InvalidName,
    StaleParent,
    Directory,
    TrailingSlash,
}
fn refuse_call(
    fs: &Filesystem,
    handle: *mut fs_ext4_fs_t,
    capi: bool,
    kind: Refusal,
    stale: InodeRef,
) {
    let errno = match kind {
        Refusal::Missing => crate::error::errno::ENOENT,
        Refusal::InvalidName => crate::error::errno::EINVAL,
        Refusal::StaleParent => crate::error::errno::ESTALE,
        Refusal::Directory => crate::error::errno::EISDIR,
        Refusal::TrailingSlash => crate::error::errno::ENOTDIR,
    };
    if capi {
        let rc = match kind {
            Refusal::Missing => c_path(handle, b"/missing"),
            Refusal::InvalidName => c_at(handle, 2u32.into(), b"bad/name"),
            Refusal::StaleParent => c_at(handle, stale, b"f\xff"),
            Refusal::Directory => c_path(handle, b"/d"),
            Refusal::TrailingSlash => c_path(handle, b"/ascii/"),
        };
        assert_eq!(rc, -1);
        assert_eq!(fs_ext4_last_errno(), errno);
    } else {
        let result = match kind {
            Refusal::Missing => fs.apply_unlink_bytes(b"/missing"),
            Refusal::InvalidName => fs.apply_unlink_at(2u32, b"bad/name"),
            Refusal::StaleParent => fs.apply_unlink_at(stale, b"f\xff"),
            Refusal::Directory => fs.apply_unlink("/d"),
            Refusal::TrailingSlash => fs.apply_unlink("/ascii/"),
        };
        assert!(
            matches!(
                (kind, result),
                (Refusal::Missing, Err(Error::NotFound))
                    | (Refusal::InvalidName, Err(Error::InvalidArgument(_)))
                    | (Refusal::StaleParent, Err(Error::Stale))
                    | (Refusal::Directory, Err(Error::IsADirectory))
                    | (Refusal::TrailingSlash, Err(Error::NotADirectory))
            ),
            "wrong logical refusal for {kind:?}"
        );
    }
}
fn refusal(capi: bool, kind: Refusal, prior_dirty: bool) {
    let (dev, fs, ino, clock) = specimen(features::FsFlavor::Ext4, 4096);
    let generation = fs.read_inode_verified(2).unwrap().0.generation;
    let stale = InodeRef::new(2, generation ^ 1);
    if prior_dirty {
        fs.apply_chmod_ino(ino, 0o600).unwrap();
    }
    let initial_status = fs.chmod_operation().unwrap().operation.status();
    dev.reset();
    clock.calls.store(0, Ordering::SeqCst);
    let before = dev.bytes();
    let _fs = with_test_mount(fs, |handle, fs| {
        refuse_call(fs, handle, capi, kind, stale);
        eprintln!("unlink refusal capi={capi} kind={kind:?} prior_dirty={prior_dirty} writes={} flushes={}", dev.counts()[1], dev.counts()[2]);
        assert_eq!(
            &dev.counts()[1..3],
            &[0, 0],
            "refused unlink wrote/flushed marker"
        );
        assert!(dev.bytes() == before);
        assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), prior_dirty);
        assert!(!fs.operation_failed.load(Ordering::SeqCst));
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            fs.chmod_operation().unwrap().operation.status(),
            initial_status
        );
    });
}
macro_rules! refusal_test {
    ($name:ident, $capi:expr, $kind:ident) => {
        #[test]
        fn $name() {
            refusal($capi, Refusal::$kind, false);
        }
    };
}
refusal_test!(
    rust_missing_unlink_must_not_mark_clean_mount,
    false,
    Missing
);
refusal_test!(c_missing_unlink_must_not_mark_clean_mount, true, Missing);
refusal_test!(
    rust_invalid_name_unlink_must_not_mark_clean_mount,
    false,
    InvalidName
);
refusal_test!(
    c_invalid_name_unlink_must_not_mark_clean_mount,
    true,
    InvalidName
);
refusal_test!(
    rust_stale_parent_unlink_must_not_mark_clean_mount,
    false,
    StaleParent
);
refusal_test!(
    c_stale_parent_unlink_must_not_mark_clean_mount,
    true,
    StaleParent
);
refusal_test!(
    rust_directory_unlink_must_not_mark_clean_mount,
    false,
    Directory
);
refusal_test!(
    c_directory_unlink_must_not_mark_clean_mount,
    true,
    Directory
);
refusal_test!(
    rust_trailing_slash_unlink_must_not_mark_clean_mount,
    false,
    TrailingSlash
);
refusal_test!(
    c_trailing_slash_unlink_must_not_mark_clean_mount,
    true,
    TrailingSlash
);

#[test]
fn prior_dirty_refusals_retain_errors_bytes_and_status() {
    for capi in [false, true] {
        for kind in [
            Refusal::Missing,
            Refusal::InvalidName,
            Refusal::StaleParent,
            Refusal::Directory,
            Refusal::TrailingSlash,
        ] {
            refusal(capi, kind, true);
        }
    }
}

struct CountInto<'a>(&'a AtomicUsize);
impl From<CountInto<'_>> for InodeRef {
    fn from(probe: CountInto<'_>) -> Self {
        probe.0.fetch_add(1, Ordering::SeqCst);
        2u32.into()
    }
}
#[test]
fn invalid_counted_name_and_read_only_refusal_precede_generic_conversion() {
    let (dev, fs, _, _) = specimen(features::FsFlavor::Ext4, 4096);
    let conversions = AtomicUsize::new(0);
    assert!(matches!(
        fs.apply_unlink_at(CountInto(&conversions), b"bad/name"),
        Err(Error::InvalidArgument(_))
    ));
    assert_eq!(conversions.load(Ordering::SeqCst), 0);
    dev.writable.store(false, Ordering::SeqCst);
    assert!(matches!(
        fs.apply_unlink_at(CountInto(&conversions), b"f\xff"),
        Err(Error::ReadOnly)
    ));
    assert_eq!(conversions.load(Ordering::SeqCst), 0);
}

struct RevokeOnSecondVerdict {
    inner: Arc<Device>,
    armed: AtomicBool,
    queries: AtomicUsize,
}
impl BlockDevice for RevokeOnSecondVerdict {
    fn read_at(&self, at: u64, out: &mut [u8]) -> Result<()> {
        self.inner.read_at(at, out)
    }
    fn write_at(&self, at: u64, bytes: &[u8]) -> Result<()> {
        self.inner.write_at(at, bytes)
    }
    fn flush(&self) -> Result<()> {
        self.inner.flush()
    }
    fn size_bytes(&self) -> u64 {
        self.inner.size_bytes()
    }
    fn is_writable(&self) -> bool {
        let writable = self.inner.is_writable();
        writable
            && (!self.armed.load(Ordering::SeqCst)
                || self.queries.fetch_add(1, Ordering::SeqCst) == 0)
    }
}
#[test]
fn second_path_verdict_must_refuse_without_marker_effects() {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
    drop(fs);
    let gate = Arc::new(RevokeOnSecondVerdict {
        inner: dev.clone(),
        armed: AtomicBool::new(false),
        queries: AtomicUsize::new(0),
    });
    let fs = Filesystem::mount_with_runtime(gate.clone(), clock.clone()).unwrap();
    let before = dev.bytes();
    let status = fs.chmod_operation().unwrap().operation.status();
    dev.reset();
    gate.armed.store(true, Ordering::SeqCst);
    assert!(matches!(
        fs.apply_unlink_bytes(b"/f\xff"),
        Err(Error::ReadOnly)
    ));
    assert_eq!(
        gate.queries.load(Ordering::SeqCst),
        2,
        "path-to-parent verdict was not exercised"
    );
    eprintln!(
        "second unlink verdict: read-only; writes={} flushes={}",
        dev.counts()[1],
        dev.counts()[2]
    );
    assert_eq!(
        &dev.counts()[1..3],
        &[0, 0],
        "second policy refusal retained early marker effects"
    );
    assert!(dev.bytes() == before);
    assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
}

fn metadata(fs: &Filesystem, family: u8) -> Result<()> {
    match family {
        0 => fs.apply_chmod("/ascii", 0o600),
        1 => fs.apply_chown("/ascii", 1001, 1002),
        2 => fs.apply_utimens("/ascii", STAMP, 0, STAMP, 0),
        3 => fs.apply_set_flags(
            "/ascii",
            crate::inode::InodeFlags::EXTENTS.bits() | crate::inode::InodeFlags::NODUMP.bits(),
        ),
        4 => fs.apply_removexattr("/ascii", NAME),
        5 => fs.apply_setxattr("/ascii", NAME, b"new"),
        _ => unreachable!(),
    }
}
struct WorkerInto {
    fs: Arc<Filesystem>,
    results: Arc<Mutex<Vec<bool>>>,
}
impl From<WorkerInto> for InodeRef {
    fn from(probe: WorkerInto) -> Self {
        for family in 0..6 {
            let fs = probe.fs.clone();
            let (tx, rx) = mpsc::channel();
            let worker = std::thread::spawn(move || {
                tx.send(matches!(metadata(&fs, family), Err(Error::Unsupported(_))))
                    .unwrap();
            });
            let refused = rx
                .recv_timeout(Duration::from_secs(5))
                .expect("metadata worker blocked");
            worker.join().unwrap();
            probe.results.lock().unwrap().push(refused);
        }
        2u32.into()
    }
}
#[test]
fn parent_conversion_must_hold_context_against_all_six_metadata_workers() {
    let (_, fs, _, _) = specimen(features::FsFlavor::Ext4, 4096);
    let fs = Arc::new(fs);
    let results = Arc::new(Mutex::new(Vec::new()));
    fs.apply_unlink_at(
        WorkerInto {
            fs: fs.clone(),
            results: results.clone(),
        },
        b"f\xff",
    )
    .unwrap();
    let results = results.lock().unwrap();
    eprintln!("unlink Into metadata worker refusals={results:?}");
    assert_eq!(
        &*results, &[true; 6],
        "parent conversion ran outside the shared context"
    );
}
#[test]
fn six_metadata_callbacks_must_exclude_unlink_workers() {
    let mut observations = Vec::new();
    for family in 0..6 {
        let (_, fs, ino, clock) = specimen(features::FsFlavor::Ext4, 4096);
        fs.apply_link_at(ino, 2u32, b"alias").unwrap(); // Non-final unlink must not call the clock recursively.
        let fs = Arc::new(fs);
        let weak = Arc::downgrade(&fs);
        let reports = Arc::new(Mutex::new(Vec::new()));
        let output = reports.clone();
        *clock.hook.lock().unwrap() = Some(Arc::new(move || {
            let fs = weak.upgrade().unwrap();
            let (tx, rx) = mpsc::channel();
            let worker = std::thread::spawn(move || {
                tx.send(matches!(
                    fs.apply_unlink_bytes(b"/f\xff"),
                    Err(Error::Unsupported(_))
                ))
                .unwrap();
            });
            let refused = rx
                .recv_timeout(Duration::from_secs(5))
                .expect("unlink worker blocked");
            worker.join().unwrap();
            output.lock().unwrap().push(refused);
        }));
        metadata(&fs, family).unwrap();
        *clock.hook.lock().unwrap() = None;
        let reports = reports.lock().unwrap();
        assert!(
            !reports.is_empty(),
            "family {family} did not reach callback"
        );
        observations.push(reports.iter().all(|v| *v));
    }
    eprintln!("metadata callback unlink worker refusals={observations:?}");
    assert_eq!(
        observations,
        vec![true; 6],
        "unlink worker bypassed metadata context"
    );
}

#[test]
fn c_final_unlink_planning_panic_must_retire_mount_and_close() {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
    let fs = with_test_mount(fs, |handle, fs| {
        *clock.hook.lock().unwrap() =
            Some(Arc::new(|| panic!("unlink deletion-time planning panic")));
        assert_eq!(c_path(handle, b"/f\xff"), -1);
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
        *clock.hook.lock().unwrap() = None;
        eprintln!(
            "unlink caught planning panic: operation_failed={} writes={} flushes={}",
            fs.operation_failed.load(Ordering::SeqCst),
            dev.counts()[1],
            dev.counts()[2]
        );
        assert!(
            fs.operation_failed.load(Ordering::SeqCst),
            "caught planning unwind did not retire mount"
        );
        dev.reset();
        assert!(fs.apply_unlink("/ascii").is_err());
        assert!(fs.flush().is_err());
        assert_eq!(dev.counts(), [0; 4]);
    });
    dev.reset();
    let bytes = dev.bytes();
    drop(fs);
    assert_eq!(dev.counts(), [0; 4]);
    assert!(dev.bytes() == bytes);
}

fn constructed(flavor: features::FsFlavor, bs: u32) {
    let (dev, fs, _, _) = specimen(flavor, bs);
    let mut context = fs.chmod_operation().unwrap();
    let before = counters(&dev);
    let mut buf = BlockBuffer::new(bs);
    // Synthetic deletion credits only: there are no bitmap/inode frees.
    fs.buffer_patch_sb_counters(&mut buf, 2, 1).unwrap();
    let offset = crate::superblock::SUPERBLOCK_OFFSET;
    let block = buf.get_mut(&fs, offset / u64::from(bs)).unwrap();
    let start = (offset % u64::from(bs)) as usize;
    let sb = &mut block[start..start + 1024];
    let expected_state = u16::from_le_bytes(sb[58..60].try_into().unwrap()) | 0x200;
    sb[58..60].copy_from_slice(&expected_state.to_le_bytes());
    if fs.csum.enabled {
        let checksum = crate::checksum::linux_crc32c(!0, &sb[..1020]);
        sb[1020..].copy_from_slice(&checksum.to_le_bytes());
    }
    if bs == 4096 {
        block[0] = 0xa5;
        block[2048] = 0x5a;
    }
    fs.commit_unlink_in(&mut context, buf).unwrap();
    assert_eq!(counters(&dev), (before.0 + 2, before.1 + 1));
    assert_eq!(
        state(&dev) & !crate::superblock::EXT4_VALID_FS,
        expected_state & !crate::superblock::EXT4_VALID_FS
    );
    let bytes = dev.bytes();
    if fs.csum.enabled {
        assert_eq!(
            u32::from_le_bytes(bytes[2044..2048].try_into().unwrap()),
            crate::checksum::linux_crc32c(!0, &bytes[1024..2044])
        );
    }
    if bs == 4096 {
        assert_eq!(bytes[0], 0xa5);
        assert_eq!(bytes[2048], 0x5a);
    }
    assert!(fs.marked_not_clean.load(Ordering::SeqCst));
    assert_eq!(context.operation.status().epoch, 2);
    eprintln!("constructed unlink credits bs={bs} journal={} counters/state/checksum/sentinels preserved; valid_fs={}", fs.journal.is_some(), state(&dev) & crate::superblock::EXT4_VALID_FS);
    assert_eq!(
        state(&dev) & crate::superblock::EXT4_VALID_FS,
        0,
        "constructed pre-marker SB restored clean state"
    );
}
macro_rules! constructed_test {
    ($name:ident, $flavor:ident, $bs:expr) => {
        #[test]
        fn $name() {
            constructed(features::FsFlavor::$flavor, $bs);
        }
    };
}
constructed_test!(
    constructed_direct_1k_credit_plan_must_preserve_dirty_marker,
    Ext4,
    1024
);
constructed_test!(
    constructed_direct_4k_credit_plan_must_preserve_dirty_marker,
    Ext4,
    4096
);
constructed_test!(
    constructed_journal_1k_credit_plan_must_preserve_dirty_marker,
    Ext3,
    1024
);
constructed_test!(
    constructed_journal_4k_credit_plan_must_preserve_dirty_marker,
    Ext3,
    4096
);

#[test]
fn volume_and_c_argument_priorities_precede_filesystem_planning() {
    for gate in 0..5 {
        let (dev, mut fs, _, _) = specimen(features::FsFlavor::Ext4, 4096);
        match gate {
            0 => dev.writable.store(false, Ordering::SeqCst),
            1 => fs.replay_pending.store(true, Ordering::SeqCst),
            2 => fs.operation_failed.store(true, Ordering::SeqCst),
            3 => fs.direct_commit_failed.store(true, Ordering::SeqCst),
            4 => fs.sb.feature_incompat |= features::Incompat::CASEFOLD.bits(),
            _ => unreachable!(),
        }
        let before = dev.bytes();
        let fs = with_test_mount(fs, |handle, fs| {
            let result = fs.apply_unlink_at(2u32, b"");
            assert!(match gate {
                1 => matches!(result, Err(Error::JournalNotReplayed)),
                4 => matches!(result, Err(Error::UnsupportedIncompat(_))),
                _ => matches!(result, Err(Error::ReadOnly)),
            });
            dev.reset();
            assert_eq!(unsafe { fs_ext4_unlink(handle, std::ptr::null()) }, -1);
            assert_eq!(fs_ext4_last_errno(), crate::error::errno::EINVAL);
            assert_eq!(
                unsafe {
                    fs_ext4_unlink_at(handle, 2, FS_EXT4_GEN_ANY, b"x".as_ptr().cast(), usize::MAX)
                },
                -1
            );
            assert_eq!(fs_ext4_last_errno(), crate::error::errno::ENAMETOOLONG);
            assert_eq!(dev.counts(), [0; 4]);
            assert!(dev.bytes() == before);
        });
        drop(fs);
    }
}

#[test]
fn real_surviving_and_final_links_retain_counters_shared_attributes_and_reopen() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            for shared in [false, true] {
                for capi in [false, true] {
                    let (dev, fs, ino, clock) = specimen(flavor, bs);
                    fs.apply_replace_file_content("/ascii", b"untouched")
                        .unwrap();
                    fs.apply_replace_file_content_bytes(b"/f\xff", b"target payload")
                        .unwrap();
                    fs.apply_setxattr_bytes(b"/f\xff", NAME, &[b'v'; 600])
                        .unwrap();
                    fs.apply_link_at(ino, 2u32, b"alias").unwrap();
                    let peer = fs.resolve(b"/ascii").unwrap();
                    if shared {
                        fs.apply_removexattr("/ascii", NAME).unwrap();
                        let target = fs.read_inode_verified(ino).unwrap().0;
                        let (inode, mut raw) = fs.read_inode_verified(peer).unwrap();
                        Filesystem::write_file_acl(&mut raw, target.file_acl).unwrap();
                        Filesystem::patch_inode_size_and_blocks(
                            &mut raw,
                            inode.size,
                            inode.blocks + u64::from(bs) / 512,
                        )
                        .unwrap();
                        fs.finalize_inode_raw(peer, inode.generation, &mut raw)
                            .unwrap();
                        let mut buf = BlockBuffer::new(bs);
                        fs.buffer_write_inode(&mut buf, peer, &raw).unwrap();
                        let block = buf.get_mut(&fs, target.file_acl).unwrap();
                        block[4..8].copy_from_slice(&2u32.to_le_bytes());
                        if fs.csum.enabled {
                            fs.csum.patch_xattr_block(target.file_acl, block);
                        }
                        fs.commit_block_buffer(buf).unwrap();
                    }
                    drop(fs);
                    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
                    clock.now.store(STAMP, Ordering::SeqCst);
                    let (before, raw) = fs.read_inode_verified(ino).unwrap();
                    let peer_raw = fs.read_inode_verified(peer).unwrap().1;
                    let counts = counters(&dev);
                    let group_before = group_counters(&fs);
                    assert_eq!((group_before.0, group_before.1), counts);
                    let data_block = fs.map_inode_logical(&before, 0).unwrap().unwrap();
                    assert!(allocated(&fs, data_block));
                    assert!(allocated(&fs, before.file_acl));
                    assert_eq!(before.links_count, 2);
                    assert_ne!(before.file_acl, 0);
                    let fs = with_test_mount(fs, |handle, fs| {
                        if capi {
                            assert_eq!(unsafe { fs_ext4_unlink(handle, std::ptr::null()) }, -1);
                            assert_eq!(fs_ext4_last_errno(), crate::error::errno::EINVAL);
                            assert_eq!(c_path(handle, b"/f\xff"), 0);
                            assert_eq!(fs_ext4_last_errno(), 0);
                        } else {
                            fs.apply_unlink_bytes(b"/f\xff").unwrap();
                        }
                        let (survivor, survivor_raw) = fs.read_inode_verified(ino).unwrap();
                        assert_eq!(survivor.links_count, 1);
                        assert_eq!(fs.resolve(b"/alias").unwrap(), ino);
                        assert!(matches!(fs.resolve(b"/f\xff"), Err(Error::NotFound)));
                        assert_eq!(counters(&dev), counts);
                        assert_eq!(group_counters(fs), group_before);
                        assert!(fs.inode_bit_is_set(ino).unwrap());
                        assert!(allocated(fs, data_block));
                        assert!(allocated(fs, before.file_acl));
                        let mut content = [0; 14];
                        assert_eq!(fs.read_ino(ino, 0, &mut content).unwrap(), content.len());
                        assert_eq!(&content, b"target payload");
                        assert_eq!(
                            (survivor.blocks, survivor.file_acl, survivor.generation),
                            (before.blocks, before.file_acl, before.generation)
                        );
                        assert_eq!(
                            crate::xattr::get_resolved(fs, &survivor, &survivor_raw, NAME).unwrap(),
                            Some(vec![b'v'; 600])
                        );
                        assert_eq!(fs.read_inode_verified(peer).unwrap().1, peer_raw);
                        if capi {
                            assert_eq!(c_at(handle, 2u32.into(), b"alias"), 0);
                            assert_eq!(fs_ext4_last_errno(), 0);
                        } else {
                            fs.apply_unlink_at(2u32, b"alias").unwrap();
                        }
                        assert!(matches!(
                            fs.live_inode(InodeRef::new(ino, before.generation)),
                            Err(Error::Stale)
                        ));
                        assert!(matches!(fs.resolve(b"/alias"), Err(Error::NotFound)));
                        let expected = before.blocks / (u64::from(bs) / 512) - u64::from(shared);
                        assert_eq!(counters(&dev), (counts.0 + expected, counts.1 + 1));
                        assert_eq!(
                            group_counters(fs),
                            (counts.0 + expected, counts.1 + 1, group_before.2)
                        );
                        assert!(!fs.inode_bit_is_set(ino).unwrap());
                        assert!(!allocated(fs, data_block));
                        assert_eq!(allocated(fs, before.file_acl), shared);
                        let freed = fs.read_inode_verified(ino).unwrap().1;
                        assert_eq!(u16::from_le_bytes(freed[..2].try_into().unwrap()), 0);
                        assert_eq!(
                            u32::from_le_bytes(freed[100..104].try_into().unwrap()),
                            before.generation
                        );
                        assert_eq!(
                            u32::from_le_bytes(freed[20..24].try_into().unwrap()),
                            STAMP as u32
                        );
                        assert_eq!(fs.read_inode_verified(peer).unwrap().1, peer_raw);
                        if shared {
                            let block = fs.read_block(before.file_acl).unwrap();
                            assert_eq!(u32::from_le_bytes(block[4..8].try_into().unwrap()), 1);
                            assert!(fs.csum.verify_xattr_block(before.file_acl, &block));
                            let (p, r) = fs.read_inode_verified(peer).unwrap();
                            assert_eq!(
                                crate::xattr::get_resolved(fs, &p, &r, NAME).unwrap(),
                                Some(vec![b'v'; 600])
                            );
                        }
                        assert_eq!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
                        assert!(raw != freed);
                    });
                    drop(fs);
                    assert_ne!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
                    let fs = Filesystem::mount(dev).unwrap();
                    assert!(matches!(fs.resolve(b"/alias"), Err(Error::NotFound)));
                    assert_eq!(fs.read_inode_verified(peer).unwrap().1, peer_raw);
                    let mut content = [0; 9];
                    assert_eq!(fs.read_ino(peer, 0, &mut content).unwrap(), content.len());
                    assert_eq!(&content, b"untouched");
                    eprintln!("positive unlink bs={bs} journal={} shared={shared} capi={capi}: non-final/final links, counts, attributes, freed generation/dtime and reopen retained", fs.journal.is_some());
                }
            }
        }
    }
}

fn fault_specimen(
    flavor: features::FsFlavor,
    bs: u32,
    final_link: bool,
    prior: bool,
) -> (Arc<Device>, Filesystem) {
    let (dev, fs, ino, clock) = specimen(flavor, bs);
    fs.apply_replace_file_content_bytes(b"/f\xff", b"target payload")
        .unwrap();
    fs.apply_setxattr_bytes(b"/f\xff", NAME, &[b'v'; 600])
        .unwrap();
    if !final_link {
        fs.apply_link_at(ino, 2u32, b"alias").unwrap();
    }
    drop(fs);
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock).unwrap();
    if prior {
        fs.apply_chmod("/ascii", 0o600).unwrap();
    }
    dev.reset();
    (dev, fs)
}
fn retired_seven(dev: &Device, fs: Filesystem) {
    assert!(
        fs.operation_failed.load(Ordering::SeqCst),
        "unlink failure did not latch shared mount retirement"
    );
    dev.reset();
    for family in 0..6 {
        assert!(metadata(&fs, family).is_err());
    }
    assert!(fs.apply_unlink("/ascii").is_err());
    assert!(fs.flush().is_err());
    assert_eq!(dev.counts(), [0; 4]);
    let bytes = dev.bytes();
    drop(fs);
    assert_eq!(dev.counts(), [0; 4]);
    assert!(dev.bytes() == bytes);
}
fn every_unlink_fault(capi: bool, flush: bool) {
    let mut injected = 0;
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            for final_link in [false, true] {
                for prior in [false, true] {
                    let (dev, fs) = fault_specimen(flavor, bs, final_link, prior);
                    fs.apply_unlink_bytes(b"/f\xff").unwrap();
                    let index = if flush { 2 } else { 1 };
                    let total = dev.counts()[index];
                    assert!(total > 0);
                    drop(fs);
                    for nth in 1..=total {
                        let (dev, fs) = fault_specimen(flavor, bs, final_link, prior);
                        (if flush {
                            &dev.fail_flush
                        } else {
                            &dev.fail_write
                        })
                        .store(nth, Ordering::SeqCst);
                        let fs = if capi {
                            with_test_mount(fs, |handle, _| {
                                assert_eq!(c_path(handle, b"/f\xff"), -1);
                                assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
                            })
                        } else {
                            assert!(fs.apply_unlink_bytes(b"/f\xff").is_err());
                            fs
                        };
                        assert!(dev.counts()[index] >= nth);
                        retired_seven(&dev, fs);
                        injected += 1;
                    }
                }
            }
        }
    }
    eprintln!("unlink capi={capi} flush={flush}: {injected} observed callbacks failed after effects; all seven families and clean close retired");
}
#[test]
fn rust_each_unlink_write_fault_retires_seven_families() {
    every_unlink_fault(false, false);
}
#[test]
fn rust_each_unlink_flush_fault_retires_seven_families() {
    every_unlink_fault(false, true);
}
#[test]
fn c_each_unlink_write_fault_retires_seven_families() {
    every_unlink_fault(true, false);
}
#[test]
fn c_each_unlink_flush_fault_retires_seven_families() {
    every_unlink_fault(true, true);
}

#[test]
fn mixed_long_names_keep_invalid_name_parent_and_volume_priorities() {
    let (dev, fs, _, _) = specimen(features::FsFlavor::Ext4, 4096);
    let parent = fs.read_inode_verified(2).unwrap().0;
    let stale = InodeRef::new(2, parent.generation.wrapping_add(1));
    assert!(matches!(
        fs.apply_unlink_at(stale, &[b'n'; 256]),
        Err(Error::Stale)
    ));
    for invalid in [b'/', 0] {
        let mut name = [b'n'; 256];
        name[255] = invalid;
        assert!(matches!(
            fs.apply_unlink_at(stale, &name),
            Err(Error::InvalidArgument(_))
        ));
    }
    dev.writable.store(false, Ordering::SeqCst);
    assert!(matches!(
        fs.apply_unlink_at(stale, &[b'n'; 256]),
        Err(Error::ReadOnly)
    ));
}

#[test]
fn unlink_foreign_context_and_malformed_superblocks_refuse_before_effects() {
    let (_, first, _, _) = specimen(features::FsFlavor::Ext4, 4096);
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
    let mut foreign = first.chmod_operation().unwrap();
    assert!(matches!(
        fs.unlink_bytes_in(&mut foreign, b"/ascii"),
        Err(Error::Stale)
    ));
    assert!(matches!(
        fs.unlink_from_in(&mut foreign, 2u32, b"ascii"),
        Err(Error::Stale)
    ));
    assert!(matches!(
        fs.commit_unlink_in(&mut foreign, BlockBuffer::new(4096)),
        Err(Error::Stale)
    ));
    let mut context = fs.chmod_operation().unwrap();
    let status = context.operation.status();
    for bytes in [vec![], vec![0; 1024], vec![0; 4096], vec![0; 4097]] {
        let mut buf = BlockBuffer::new(4096);
        buf.put(0, bytes);
        assert!(matches!(
            fs.commit_unlink_in(&mut context, buf),
            Err(Error::Corrupt(_))
        ));
        assert_eq!(context.operation.status(), status);
    }
    assert_eq!(dev.counts(), [0; 4]);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn unlink_publication_unwind_latches_with_retained_context() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        let (dev, fs, _, _) = specimen(flavor, 4096);
        let buf = fs.plan_unlink_at(2u32.into(), b"f\xff").unwrap();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _held = fs.uninit_cleared.lock().unwrap();
            panic!("unlink allocation publication poison");
        }))
        .is_err());
        let mut context = fs.chmod_operation().unwrap();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || fs.commit_unlink_in(&mut context, buf)
        ))
        .is_err());
        assert_eq!(context.operation.status().health, Health::Uncertain);
        assert!(fs.operation_failed.load(Ordering::SeqCst));
        drop(context);
        retired_seven(&dev, fs);
    }
}

#[test]
fn unlink_planning_and_each_flush_unwind_retire_seven_families_and_close() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for capi in [false, true] {
            for final_link in [false, true] {
                for prior in [false, true] {
                    let (dev, fs) = fault_specimen(flavor, 4096, final_link, prior);
                    fs.apply_unlink_bytes(b"/f\xff").unwrap();
                    let total = dev.counts()[2];
                    drop(fs);
                    for phase in 0..=total {
                        let (dev, fs) = fault_specimen(flavor, 4096, final_link, prior);
                        // This helper uses the same Clock type, but install a new
                        // clock mount before effects so the planning callback is controlled.
                        drop(fs);
                        let clock = Arc::new(Clock {
                            calls: AtomicUsize::new(0),
                            now: std::sync::atomic::AtomicI64::new(STAMP),
                            hook: Mutex::new(None),
                        });
                        let fs =
                            Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
                        if prior {
                            fs.apply_chmod("/ascii", 0o600).unwrap();
                        }
                        dev.reset();
                        let before = dev.bytes();
                        if phase == 0 {
                            *clock.hook.lock().unwrap() =
                                Some(Arc::new(|| panic!("unlink planning panic")));
                        } else {
                            dev.panic_flush.store(phase, Ordering::SeqCst);
                        }
                        let fs = if capi {
                            with_test_mount(fs, |handle, _| {
                                assert_eq!(c_path(handle, b"/f\xff"), -1);
                                assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
                            })
                        } else {
                            assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                                || fs.apply_unlink_bytes(b"/f\xff")
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
                        retired_seven(&dev, fs);
                    }
                }
            }
        }
    }
}
