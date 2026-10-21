//! Guarded public rmdir controls; native fixtures are not Linux qualification.
//! Constructed pre-marker SB credits exercise the rmdir-only boundary.
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
        Some([43; 16]),
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
    let parent = fs.apply_mkdir("/p", 0o750).unwrap();
    fs.apply_mkdir("/p/d", 0o750).unwrap();
    fs.apply_mkdir_bytes(b"/p/d\xff", 0o750).unwrap();
    fs.apply_mkdir("/p/full", 0o750).unwrap();
    fs.apply_create("/p/full/child", 0o640).unwrap();
    fs.apply_symlink("d", "/p/link").unwrap();
    fs.apply_create("/ascii", 0o640).unwrap();
    fs.apply_replace_file_content("/ascii", b"peer payload")
        .unwrap();
    fs.apply_setxattr("/ascii", NAME, b"old").unwrap();
    drop(fs);
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
    assert_eq!(fs.sb.block_size(), bs);
    assert_eq!(fs.csum.enabled, flavor == features::FsFlavor::Ext4);
    assert_eq!(fs.journal.is_some(), flavor == features::FsFlavor::Ext3);
    assert_ne!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
    dev.reset();
    clock.calls.store(0, Ordering::SeqCst);
    (dev, fs, parent, clock)
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
    fs.read_block(fs.groups[gi].block_bitmap).unwrap()[(bit / 8) as usize] & (1 << (bit % 8)) != 0
}
fn group_counters(fs: &Filesystem) -> (u64, u32, u32) {
    assert_eq!(fs.groups.len(), 1);
    let (block, off) = fs.sb.descriptor_location(0);
    let b = fs.read_block(block).unwrap();
    let word = |at| {
        u32::from(u16::from_le_bytes(
            b[off + at..off + at + 2].try_into().unwrap(),
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

#[derive(Clone, Copy, Debug)]
enum Entry {
    String,
    Bytes,
    At,
    CPath,
    CAt,
}
const ENTRIES: [Entry; 5] = [
    Entry::String,
    Entry::Bytes,
    Entry::At,
    Entry::CPath,
    Entry::CAt,
];
fn call(
    fs: &Filesystem,
    h: *mut fs_ext4_fs_t,
    parent: InodeRef,
    entry: Entry,
    name: &[u8],
) -> std::result::Result<(), i32> {
    let path = [b"/p/".as_slice(), name].concat();
    match entry {
        Entry::String => fs
            .apply_rmdir(std::str::from_utf8(&path).unwrap())
            .map_err(|e| e.to_errno()),
        Entry::Bytes => fs.apply_rmdir_bytes(&path).map_err(|e| e.to_errno()),
        Entry::At => fs.apply_rmdir_at(parent, name).map_err(|e| e.to_errno()),
        Entry::CPath | Entry::CAt => {
            let rc = if matches!(entry, Entry::CPath) {
                let path = CString::new(path).unwrap();
                unsafe { fs_ext4_rmdir(h, path.as_ptr()) }
            } else {
                unsafe {
                    fs_ext4_rmdir_at(
                        h,
                        parent.ino,
                        parent.generation.unwrap_or(FS_EXT4_GEN_ANY),
                        name.as_ptr().cast(),
                        name.len(),
                    )
                }
            };
            if rc == 0 {
                assert_eq!(fs_ext4_last_errno(), 0);
                Ok(())
            } else {
                assert_eq!(rc, -1);
                Err(fs_ext4_last_errno())
            }
        }
    }
}

fn held(entry: Entry) {
    let (dev, fs, parent, clock) = specimen(features::FsFlavor::Ext4, 4096);
    let _fs = with_test_mount(fs, |h, fs| {
        let held = fs.chmod_operation().unwrap();
        let status = held.operation.status();
        let before = dev.bytes();
        let got = call(fs, h, parent.into(), entry, b"d");
        eprintln!(
            "held rmdir {entry:?}: result={got:?} callbacks={:?}",
            dev.counts()
        );
        assert_eq!(
            got,
            Err(crate::error::errno::ENOTSUP),
            "rmdir bypassed shared context"
        );
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert_eq!(held.operation.status(), status);
        assert!(dev.bytes() == before);
    });
}
macro_rules! held_test {
    ($name:ident, $entry:ident) => {
        #[test]
        fn $name() {
            held(Entry::$entry);
        }
    };
}
held_test!(
    rust_string_must_refuse_held_context_before_callbacks,
    String
);
held_test!(rust_bytes_must_refuse_held_context_before_callbacks, Bytes);
held_test!(rust_parent_must_refuse_held_context_before_callbacks, At);
held_test!(c_path_must_refuse_held_context_before_callbacks, CPath);
held_test!(c_parent_must_refuse_held_context_before_callbacks, CAt);

#[derive(Clone, Copy, Debug)]
enum Refusal {
    Missing,
    Empty,
    Slash,
    Nul,
    Stale,
    File,
    Symlink,
    Nonempty,
    Dot,
    Dotdot,
}
const REFUSALS: [Refusal; 10] = [
    Refusal::Missing,
    Refusal::Empty,
    Refusal::Slash,
    Refusal::Nul,
    Refusal::Stale,
    Refusal::File,
    Refusal::Symlink,
    Refusal::Nonempty,
    Refusal::Dot,
    Refusal::Dotdot,
];
fn refusal(capi: bool, kind: Refusal, dirty: bool) {
    let (dev, fs, parent, clock) = specimen(features::FsFlavor::Ext4, 4096);
    if dirty {
        fs.apply_chmod("/ascii", 0o600).unwrap();
    }
    let generation = fs.read_inode_verified(parent).unwrap().0.generation;
    let (dir, name, expected): (InodeRef, &[u8], i32) = match kind {
        Refusal::Missing => (parent.into(), b"missing", crate::error::errno::ENOENT),
        Refusal::Empty => (parent.into(), b"", crate::error::errno::EINVAL),
        Refusal::Slash => (parent.into(), b"bad/name", crate::error::errno::EINVAL),
        Refusal::Nul => (parent.into(), b"bad\0name", crate::error::errno::EINVAL),
        Refusal::Stale => (
            InodeRef::new(parent, generation ^ 1),
            b"d",
            crate::error::errno::ESTALE,
        ),
        Refusal::File => (2u32.into(), b"ascii", crate::error::errno::ENOTDIR),
        Refusal::Symlink => (parent.into(), b"link", crate::error::errno::ENOTDIR),
        Refusal::Nonempty => (parent.into(), b"full", crate::error::errno::ENOTEMPTY),
        Refusal::Dot => (parent.into(), b".", crate::error::errno::EINVAL),
        Refusal::Dotdot => (parent.into(), b"..", crate::error::errno::ENOTEMPTY),
    };
    let status = fs.chmod_operation().unwrap().operation.status();
    dev.reset();
    clock.calls.store(0, Ordering::SeqCst);
    let before = dev.bytes();
    let _fs = with_test_mount(fs, |h, fs| {
        let got = call(fs, h, dir, if capi { Entry::CAt } else { Entry::At }, name);
        assert_eq!(got, Err(expected));
        eprintln!(
            "rmdir refusal {kind:?} capi={capi} dirty={dirty}: writes={} flushes={}",
            dev.counts()[1],
            dev.counts()[2]
        );
        assert_eq!(&dev.counts()[1..3], &[0, 0], "logical refusal wrote marker");
        assert!(dev.bytes() == before);
        assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), dirty);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert!(!fs.operation_failed.load(Ordering::SeqCst));
        assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
    });
}
macro_rules! refusal_test {
    ($name:ident, $c:expr, $kind:ident) => {
        #[test]
        fn $name() {
            refusal($c, Refusal::$kind, false);
        }
    };
}
refusal_test!(rust_missing_must_not_mark_clean_mount, false, Missing);
refusal_test!(c_missing_must_not_mark_clean_mount, true, Missing);
refusal_test!(rust_empty_name_must_not_mark_clean_mount, false, Empty);
refusal_test!(c_empty_name_must_not_mark_clean_mount, true, Empty);
refusal_test!(rust_slash_name_must_not_mark_clean_mount, false, Slash);
refusal_test!(c_slash_name_must_not_mark_clean_mount, true, Slash);
refusal_test!(rust_nul_name_must_not_mark_clean_mount, false, Nul);
refusal_test!(c_nul_name_must_not_mark_clean_mount, true, Nul);
refusal_test!(rust_stale_parent_must_not_mark_clean_mount, false, Stale);
refusal_test!(c_stale_parent_must_not_mark_clean_mount, true, Stale);
refusal_test!(rust_file_target_must_not_mark_clean_mount, false, File);
refusal_test!(c_file_target_must_not_mark_clean_mount, true, File);
refusal_test!(
    rust_symlink_target_must_not_mark_clean_mount,
    false,
    Symlink
);
refusal_test!(c_symlink_target_must_not_mark_clean_mount, true, Symlink);
refusal_test!(
    rust_nonempty_target_must_not_mark_clean_mount,
    false,
    Nonempty
);
refusal_test!(c_nonempty_target_must_not_mark_clean_mount, true, Nonempty);
refusal_test!(rust_dot_must_not_mark_clean_mount, false, Dot);
refusal_test!(c_dot_must_not_mark_clean_mount, true, Dot);
refusal_test!(rust_dotdot_must_not_mark_clean_mount, false, Dotdot);
refusal_test!(c_dotdot_must_not_mark_clean_mount, true, Dotdot);

#[test]
fn prior_dirty_refusals_preserve_errors_bytes_and_context() {
    for capi in [false, true] {
        for kind in REFUSALS {
            refusal(capi, kind, true);
        }
    }
}

fn path_refusals(capi: bool) {
    let mut changed = Vec::new();
    for (path, expected) in [
        (b"/missing".as_slice(), crate::error::errno::ENOENT),
        (b"/", crate::error::errno::EINVAL),
        (b"", crate::error::errno::EINVAL),
        (b"relative", crate::error::errno::EINVAL),
        (b"/ascii/", crate::error::errno::ENOTDIR),
        (b"/p/link/", crate::error::errno::ENOTDIR),
        (b"/p/full/", crate::error::errno::ENOTEMPTY),
        (b"/p/./", crate::error::errno::EINVAL),
        (b"/p/../", crate::error::errno::ENOTEMPTY),
    ] {
        let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
        let before = dev.bytes();
        let status = fs.chmod_operation().unwrap().operation.status();
        let _fs = with_test_mount(fs, |h, fs| {
            if capi {
                let path = CString::new(path).unwrap();
                assert_eq!(unsafe { fs_ext4_rmdir(h, path.as_ptr()) }, -1);
                assert_eq!(fs_ext4_last_errno(), expected);
            } else {
                assert_eq!(fs.apply_rmdir_bytes(path).unwrap_err().to_errno(), expected);
            }
            assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
            assert!(!fs.operation_failed.load(Ordering::SeqCst));
            assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
            changed.push(dev.bytes() != before || dev.counts()[1..3] != [0, 0]);
        });
    }
    eprintln!("rmdir path refusals capi={capi}: changed={changed:?}");
    assert_eq!(changed, vec![false; 9], "path refusal wrote a marker");
}
#[test]
fn rust_path_edge_refusals_must_not_mark_clean_mount() {
    path_refusals(false);
}
#[test]
fn c_path_edge_refusals_must_not_mark_clean_mount() {
    path_refusals(true);
}

struct Convert<'a> {
    calls: &'a AtomicUsize,
    parent: u32,
    hook: Option<&'a dyn Fn()>,
}
impl From<Convert<'_>> for InodeRef {
    fn from(v: Convert<'_>) -> Self {
        v.calls.fetch_add(1, Ordering::SeqCst);
        if let Some(hook) = v.hook {
            hook();
        }
        v.parent.into()
    }
}
#[test]
fn held_context_must_precede_generic_parent_conversion() {
    let (dev, fs, parent, _) = specimen(features::FsFlavor::Ext4, 4096);
    let held = fs.chmod_operation().unwrap();
    let calls = AtomicUsize::new(0);
    let got = fs.apply_rmdir_at(
        Convert {
            calls: &calls,
            parent,
            hook: None,
        },
        b"d",
    );
    eprintln!(
        "held rmdir Into: conversions={}",
        calls.load(Ordering::SeqCst)
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(matches!(got, Err(Error::Unsupported(_))));
    assert_eq!(dev.counts(), [0; 4]);
    drop(held);
}
#[test]
fn invalid_dot_and_volume_priorities_precede_conversion() {
    let (dev, fs, parent, _) = specimen(features::FsFlavor::Ext4, 4096);
    let calls = AtomicUsize::new(0);
    for name in [b"".as_slice(), b"bad/name", b"bad\0name", b".", b".."] {
        let result = fs.apply_rmdir_at(
            Convert {
                calls: &calls,
                parent,
                hook: None,
            },
            name,
        );
        assert_eq!(
            result.unwrap_err().to_errno(),
            if name == b".." {
                crate::error::errno::ENOTEMPTY
            } else {
                crate::error::errno::EINVAL
            }
        );
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    dev.writable.store(false, Ordering::SeqCst);
    assert!(matches!(
        fs.apply_rmdir_at(
            Convert {
                calls: &calls,
                parent,
                hook: None
            },
            b"d"
        ),
        Err(Error::ReadOnly)
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
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
        6 => fs.apply_unlink("/ascii"),
        _ => unreachable!(),
    }
}
fn worker(fs: Arc<Filesystem>, family: u8) -> bool {
    let (tx, rx) = mpsc::channel();
    let join = std::thread::spawn(move || {
        tx.send(matches!(metadata(&fs, family), Err(Error::Unsupported(_))))
            .unwrap();
    });
    let result = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("metadata worker blocked");
    join.join().unwrap();
    result
}
#[test]
fn conversion_must_hold_context_against_seven_guarded_workers() {
    let (_, fs, parent, _) = specimen(features::FsFlavor::Ext4, 4096);
    let fs = Arc::new(fs);
    let calls = AtomicUsize::new(0);
    let results = Mutex::new(Vec::new());
    let hook = || {
        for family in 0..7 {
            results.lock().unwrap().push(worker(fs.clone(), family));
        }
    };
    fs.apply_rmdir_at(
        Convert {
            calls: &calls,
            parent,
            hook: Some(&hook),
        },
        b"d",
    )
    .unwrap();
    let results = results.lock().unwrap();
    eprintln!("rmdir Into guarded-worker refusals={results:?}");
    assert_eq!(&*results, &[true; 7]);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

struct CallbackDevice {
    inner: Arc<Device>,
    hook: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
    on_read: bool,
}
impl CallbackDevice {
    fn fire(&self) {
        let hook = self.hook.lock().unwrap().take();
        if let Some(hook) = hook {
            hook();
        }
    }
}
impl BlockDevice for CallbackDevice {
    fn read_at(&self, at: u64, out: &mut [u8]) -> Result<()> {
        if self.on_read {
            self.fire();
        }
        self.inner.read_at(at, out)
    }
    fn write_at(&self, at: u64, b: &[u8]) -> Result<()> {
        self.inner.write_at(at, b)
    }
    fn flush(&self) -> Result<()> {
        self.inner.flush()
    }
    fn size_bytes(&self) -> u64 {
        self.inner.size_bytes()
    }
    fn is_writable(&self) -> bool {
        if !self.on_read {
            self.fire();
        }
        self.inner.is_writable()
    }
}
fn device_callback(on_read: bool) {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
    drop(fs);
    let gate = Arc::new(CallbackDevice {
        inner: dev,
        hook: Mutex::new(None),
        on_read,
    });
    let fs = Arc::new(Filesystem::mount_with_runtime(gate.clone(), clock).unwrap());
    let weak = Arc::downgrade(&fs);
    let results = Arc::new(Mutex::new(Vec::new()));
    let output = results.clone();
    *gate.hook.lock().unwrap() = Some(Arc::new(move || {
        output
            .lock()
            .unwrap()
            .push(worker(weak.upgrade().unwrap(), 0));
    }));
    fs.apply_rmdir("/p/d").unwrap();
    let results = results.lock().unwrap();
    eprintln!("rmdir device callback on_read={on_read} worker refusals={results:?}");
    assert_eq!(&*results, &[true], "callback must execute under context");
}
#[test]
fn writability_callback_must_exclude_metadata_worker() {
    device_callback(false);
}
#[test]
fn read_callback_must_exclude_metadata_worker() {
    device_callback(true);
}

#[test]
fn runtime_callback_must_exclude_metadata_worker() {
    let (_, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
    let fs = Arc::new(fs);
    let weak = Arc::downgrade(&fs);
    let armed = AtomicBool::new(true);
    let results = Arc::new(Mutex::new(Vec::new()));
    let output = results.clone();
    *clock.hook.lock().unwrap() = Some(Arc::new(move || {
        if armed.swap(false, Ordering::SeqCst) {
            output
                .lock()
                .unwrap()
                .push(worker(weak.upgrade().unwrap(), 0));
        }
    }));
    fs.apply_rmdir("/p/d").unwrap();
    *clock.hook.lock().unwrap() = None;
    let results = results.lock().unwrap();
    eprintln!("rmdir runtime worker refusals={results:?}");
    assert_eq!(&*results, &[true]);
}
#[test]
fn seven_guarded_runtime_callbacks_must_exclude_rmdir_worker() {
    let mut observations = Vec::new();
    for family in 0..7 {
        let (_, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
        let fs = Arc::new(fs);
        let weak = Arc::downgrade(&fs);
        let armed = AtomicBool::new(true);
        let results = Arc::new(Mutex::new(Vec::new()));
        let output = results.clone();
        *clock.hook.lock().unwrap() = Some(Arc::new(move || {
            if armed.swap(false, Ordering::SeqCst) {
                let fs = weak.upgrade().unwrap();
                let (tx, rx) = mpsc::channel();
                let join = std::thread::spawn(move || {
                    tx.send(matches!(fs.apply_rmdir("/p/d"), Err(Error::Unsupported(_))))
                        .unwrap();
                });
                let refused = rx
                    .recv_timeout(Duration::from_secs(5))
                    .expect("rmdir worker blocked");
                join.join().unwrap();
                output.lock().unwrap().push(refused);
            }
        }));
        metadata(&fs, family).unwrap();
        *clock.hook.lock().unwrap() = None;
        let results = results.lock().unwrap();
        assert_eq!(results.len(), 1);
        observations.push(results[0]);
    }
    eprintln!("guarded-family runtime rmdir refusals={observations:?}");
    assert_eq!(observations, vec![true; 7]);
}

struct Revoke {
    inner: Arc<Device>,
    armed: AtomicBool,
    queries: AtomicUsize,
}
impl BlockDevice for Revoke {
    fn read_at(&self, at: u64, b: &mut [u8]) -> Result<()> {
        self.inner.read_at(at, b)
    }
    fn write_at(&self, at: u64, b: &[u8]) -> Result<()> {
        self.inner.write_at(at, b)
    }
    fn flush(&self) -> Result<()> {
        self.inner.flush()
    }
    fn size_bytes(&self) -> u64 {
        self.inner.size_bytes()
    }
    fn is_writable(&self) -> bool {
        self.inner.is_writable()
            && (!self.armed.load(Ordering::SeqCst)
                || self.queries.fetch_add(1, Ordering::SeqCst) == 0)
    }
}
#[test]
fn second_path_verdict_must_refuse_without_marker_effects() {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
    drop(fs);
    let gate = Arc::new(Revoke {
        inner: dev.clone(),
        armed: AtomicBool::new(false),
        queries: AtomicUsize::new(0),
    });
    let fs = Filesystem::mount_with_runtime(gate.clone(), clock).unwrap();
    let before = dev.bytes();
    let status = fs.chmod_operation().unwrap().operation.status();
    dev.reset();
    gate.armed.store(true, Ordering::SeqCst);
    assert!(matches!(fs.apply_rmdir("/p/d"), Err(Error::ReadOnly)));
    assert_eq!(gate.queries.load(Ordering::SeqCst), 2);
    eprintln!(
        "rmdir second verdict writes={} flushes={}",
        dev.counts()[1],
        dev.counts()[2]
    );
    assert_eq!(&dev.counts()[1..3], &[0, 0]);
    assert!(dev.bytes() == before);
    assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
}

#[test]
fn volume_and_c_argument_priorities_are_retained() {
    for gate in 0..6 {
        let (dev, mut fs, parent, _) = specimen(features::FsFlavor::Ext4, 4096);
        match gate {
            0 => dev.writable.store(false, Ordering::SeqCst),
            1 => fs.replay_pending.store(true, Ordering::SeqCst),
            2 => fs.operation_failed.store(true, Ordering::SeqCst),
            3 => fs.direct_commit_failed.store(true, Ordering::SeqCst),
            4 => fs.sb.feature_incompat |= features::Incompat::CASEFOLD.bits(),
            5 => fs.sb.feature_ro_compat |= features::RoCompat::QUOTA.bits(),
            _ => unreachable!(),
        }
        let before = dev.bytes();
        let fs = with_test_mount(fs, |h, fs| {
            let got = fs.apply_rmdir_at(parent, b".");
            assert!(match gate {
                1 => matches!(got, Err(Error::JournalNotReplayed)),
                4 => matches!(got, Err(Error::UnsupportedIncompat(_))),
                5 => matches!(got, Err(Error::UnsupportedRoCompat(_))),
                _ => matches!(got, Err(Error::ReadOnly)),
            });
            dev.reset();
            assert_eq!(unsafe { fs_ext4_rmdir(h, std::ptr::null()) }, -1);
            assert_eq!(fs_ext4_last_errno(), crate::error::errno::EINVAL);
            assert_eq!(
                unsafe {
                    fs_ext4_rmdir_at(h, parent, FS_EXT4_GEN_ANY, b"x".as_ptr().cast(), usize::MAX)
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

fn retired_observation(dev: &Device, fs: Filesystem) -> bool {
    let latched = fs.operation_failed.load(Ordering::SeqCst);
    dev.reset();
    let before = dev.bytes();
    let mut refused = true;
    for family in 0..7 {
        refused &= metadata(&fs, family).is_err();
    }
    refused &= fs.apply_rmdir_bytes(b"/p/d\xff").is_err();
    refused &= fs.flush().is_err();
    drop(fs);
    latched && refused && dev.counts() == [0; 4] && dev.bytes() == before
}
fn planning_unwind(capi: bool, dirty: bool) {
    let (dev, fs, parent, clock) = specimen(features::FsFlavor::Ext4, 4096);
    if dirty {
        fs.apply_chmod("/ascii", 0o600).unwrap();
    }
    dev.reset();
    let before = dev.bytes();
    *clock.hook.lock().unwrap() = Some(Arc::new(|| panic!("rmdir deletion-time planning unwind")));
    let fs = if capi {
        with_test_mount(fs, |h, fs| {
            assert_eq!(
                call(fs, h, parent.into(), Entry::CPath, b"d"),
                Err(crate::error::errno::EIO)
            );
        })
    } else {
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fs.apply_rmdir("/p/d")))
                .is_err()
        );
        fs
    };
    *clock.hook.lock().unwrap() = None;
    let unchanged = dev.bytes() == before && dev.counts()[1..3] == [0, 0];
    let latched = fs.operation_failed.load(Ordering::SeqCst);
    let retired = retired_observation(&dev, fs);
    eprintln!("rmdir planning unwind capi={capi} dirty={dirty}: unchanged={unchanged} latched={latched} eight families/close retired={retired}");
    assert!(
        unchanged && retired,
        "planning unwind failed untouched retirement contract"
    );
}
#[test]
fn rust_clean_planning_unwind_must_retire_without_marker() {
    planning_unwind(false, false);
}
#[test]
fn c_clean_planning_unwind_must_retire_without_marker() {
    planning_unwind(true, false);
}
#[test]
fn rust_dirty_planning_unwind_must_retire_without_writes() {
    planning_unwind(false, true);
}
#[test]
fn c_dirty_planning_unwind_must_retire_without_writes() {
    planning_unwind(true, true);
}
#[test]
fn conversion_unwind_must_retire_without_marker() {
    let (dev, fs, parent, _) = specimen(features::FsFlavor::Ext4, 4096);
    let calls = AtomicUsize::new(0);
    let before = dev.bytes();
    let hook = || panic!("rmdir Into unwind");
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fs.apply_rmdir_at(
            Convert {
                calls: &calls,
                parent,
                hook: Some(&hook)
            },
            b"d"
        )))
        .is_err()
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let unchanged = dev.bytes() == before && dev.counts()[1..3] == [0, 0];
    let retired = retired_observation(&dev, fs);
    eprintln!("rmdir Into unwind unchanged={unchanged} retired={retired}");
    assert!(unchanged && retired);
}

fn fault_specimen(
    flavor: features::FsFlavor,
    bs: u32,
    dirty: bool,
) -> (Arc<Device>, Filesystem, u32) {
    let (dev, fs, parent, clock) = specimen(flavor, bs);
    fs.apply_setxattr("/p/d", NAME, &[b'v'; 600]).unwrap();
    drop(fs);
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock).unwrap();
    if dirty {
        fs.apply_chmod("/ascii", 0o600).unwrap();
    }
    dev.reset();
    (dev, fs, parent)
}
fn failures(capi: bool, flush: bool, unwind: bool) {
    let mut exercised = 0;
    let mut violations = 0;
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            for dirty in [false, true] {
                let (dev, fs, _) = fault_specimen(flavor, bs, dirty);
                fs.apply_rmdir("/p/d").unwrap();
                let index = if flush { 2 } else { 1 };
                let total = dev.counts()[index];
                assert!(total > 0);
                drop(fs);
                for nth in 1..=total {
                    let (dev, fs, parent) = fault_specimen(flavor, bs, dirty);
                    (if unwind {
                        &dev.panic_flush
                    } else if flush {
                        &dev.fail_flush
                    } else {
                        &dev.fail_write
                    })
                    .store(nth, Ordering::SeqCst);
                    let fs = if capi {
                        with_test_mount(fs, |h, fs| {
                            assert_eq!(
                                call(fs, h, parent.into(), Entry::CAt, b"d"),
                                Err(crate::error::errno::EIO)
                            );
                        })
                    } else {
                        if unwind {
                            assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                                || fs.apply_rmdir("/p/d")
                            ))
                            .is_err());
                        } else {
                            assert!(fs.apply_rmdir("/p/d").is_err());
                        }
                        fs
                    };
                    assert!(dev.counts()[index] >= nth);
                    violations += usize::from(!retired_observation(&dev, fs));
                    exercised += 1;
                }
            }
        }
    }
    eprintln!("rmdir fault sweep capi={capi} flush={flush} unwind={unwind}: {exercised} observed faults; eight-family/flush/close retirement violations={violations}");
    assert_eq!(
        violations, 0,
        "rmdir uncertainty did not retire every guarded family and close"
    );
}
#[test]
fn rust_each_observed_write_fault_must_retire_mount() {
    failures(false, false, false);
}
#[test]
fn c_each_observed_write_fault_must_retire_mount() {
    failures(true, false, false);
}
#[test]
fn rust_each_observed_flush_fault_must_retire_mount() {
    failures(false, true, false);
}
#[test]
fn c_each_observed_flush_fault_must_retire_mount() {
    failures(true, true, false);
}
#[test]
fn rust_each_observed_flush_unwind_must_retire_mount() {
    failures(false, true, true);
}
#[test]
fn c_each_observed_flush_unwind_must_retire_mount() {
    failures(true, true, true);
}

fn constructed(flavor: features::FsFlavor, bs: u32) {
    let (dev, fs, _, _) = specimen(flavor, bs);
    let mut context = fs.chmod_operation().unwrap();
    let before = counters(&dev);
    let mut buf = BlockBuffer::new(bs);
    fs.buffer_patch_sb_counters(&mut buf, 2, 1).unwrap();
    let offset = crate::superblock::SUPERBLOCK_OFFSET;
    let block = buf.get_mut(&fs, offset / u64::from(bs)).unwrap();
    let start = (offset % u64::from(bs)) as usize;
    let sb = &mut block[start..start + 1024];
    let expected = u16::from_le_bytes(sb[58..60].try_into().unwrap()) | 0x200;
    sb[58..60].copy_from_slice(&expected.to_le_bytes());
    if fs.csum.enabled {
        let checksum = crate::checksum::linux_crc32c(!0, &sb[..1020]);
        sb[1020..].copy_from_slice(&checksum.to_le_bytes());
    }
    if bs == 4096 {
        block[0] = 0xa5;
        block[2048] = 0x5a;
    }
    // Constructed credits, not a valid deletion: exercise the actual boundary.
    fs.commit_rmdir_in(&mut context, buf).unwrap();
    assert_eq!(counters(&dev), (before.0 + 2, before.1 + 1));
    assert_eq!(
        state(&dev) & !crate::superblock::EXT4_VALID_FS,
        expected & !crate::superblock::EXT4_VALID_FS
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
    assert_eq!(context.operation.status().epoch, 2);
    eprintln!("constructed future rmdir bs={bs} journal={}: credits/state/checksum/sentinels preserved; clean={}", fs.journal.is_some(), state(&dev) & crate::superblock::EXT4_VALID_FS);
    assert_eq!(
        state(&dev) & crate::superblock::EXT4_VALID_FS,
        0,
        "future pre-marker buffer restored clean state"
    );
}
#[test]
fn constructed_future_direct_1k_plan_must_keep_dirty_state() {
    constructed(features::FsFlavor::Ext4, 1024);
}
#[test]
fn constructed_future_direct_4k_plan_must_keep_dirty_state() {
    constructed(features::FsFlavor::Ext4, 4096);
}
#[test]
fn constructed_future_journal_1k_plan_must_keep_dirty_state() {
    constructed(features::FsFlavor::Ext3, 1024);
}
#[test]
fn constructed_future_journal_4k_plan_must_keep_dirty_state() {
    constructed(features::FsFlavor::Ext3, 4096);
}

fn share_attributes(fs: &Filesystem, target: u32, peer: u32) {
    fs.apply_removexattr("/ascii", NAME).unwrap();
    let target = fs.read_inode_verified(target).unwrap().0;
    let (inode, mut raw) = fs.read_inode_verified(peer).unwrap();
    Filesystem::write_file_acl(&mut raw, target.file_acl).unwrap();
    Filesystem::patch_inode_size_and_blocks(
        &mut raw,
        inode.size,
        inode.blocks + u64::from(fs.sb.block_size()) / 512,
    )
    .unwrap();
    fs.finalize_inode_raw(peer, inode.generation, &mut raw)
        .unwrap();
    let mut buf = BlockBuffer::new(fs.sb.block_size());
    fs.buffer_write_inode(&mut buf, peer, &raw).unwrap();
    let block = buf.get_mut(fs, target.file_acl).unwrap();
    block[4..8].copy_from_slice(&2u32.to_le_bytes());
    if fs.csum.enabled {
        fs.csum.patch_xattr_block(target.file_acl, block);
    }
    fs.commit_block_buffer(buf).unwrap();
}

// Coherent native inline fixture: release the old map, add system.data and
// persist the feature bit, rather than relabeling pointers and leaking blocks.
fn inline_directory(fs: &Filesystem, ino: u32, parent: u32, entries: &[(u32, &[u8], u8)]) {
    let (inode, mut raw) = fs.read_inode_verified(ino).unwrap();
    assert!(inode.has_extents());
    let bs = fs.sb.block_size();
    let mut buf = BlockBuffer::new(bs);
    let freed = fs
        .buffer_free_runs(&mut buf, &fs.extent_tree_runs(ino, &inode).unwrap())
        .unwrap();
    fs.buffer_patch_sb_counters(&mut buf, freed as i64, 0)
        .unwrap();
    let mut data = [0u8; 60];
    data[..4].copy_from_slice(&parent.to_le_bytes());
    let mut at = 4;
    for (i, (entry, name, kind)) in entries.iter().enumerate() {
        let len = if i + 1 == entries.len() {
            60 - at
        } else {
            crate::dir::entry_rec_len(name.len())
        };
        assert!(len >= crate::dir::entry_rec_len(name.len()));
        data[at..at + 4].copy_from_slice(&entry.to_le_bytes());
        data[at + 4..at + 6].copy_from_slice(&(len as u16).to_le_bytes());
        data[at + 6] = name.len() as u8;
        data[at + 7] = *kind;
        data[at + 8..at + 8 + name.len()].copy_from_slice(name);
        at += len;
    }
    if entries.is_empty() {
        data[8..10].copy_from_slice(&56u16.to_le_bytes());
    }
    crate::inline_mut::store(&mut raw, fs.sb.inode_size, &data).unwrap();
    let flags = (inode.flags & !crate::inode::InodeFlags::EXTENTS.bits())
        | crate::inode::InodeFlags::INLINE_DATA.bits();
    raw[32..36].copy_from_slice(&flags.to_le_bytes());
    Filesystem::patch_inode_size_and_blocks(
        &mut raw,
        60,
        Filesystem::xattr_block_sectors(&inode, bs),
    )
    .unwrap();
    fs.finalize_inode_raw(ino, inode.generation, &mut raw)
        .unwrap();
    fs.buffer_write_inode(&mut buf, ino, &raw).unwrap();
    let offset = crate::superblock::SUPERBLOCK_OFFSET;
    let block = buf.get_mut(fs, offset / u64::from(bs)).unwrap();
    let start = (offset % u64::from(bs)) as usize;
    let sb = &mut block[start..start + 1024];
    let incompat = u32::from_le_bytes(sb[96..100].try_into().unwrap())
        | features::Incompat::INLINE_DATA.bits();
    sb[96..100].copy_from_slice(&incompat.to_le_bytes());
    if fs.csum.enabled {
        let sum = crate::checksum::linux_crc32c(!0, &sb[..1020]);
        sb[1020..].copy_from_slice(&sum.to_le_bytes());
    }
    fs.commit_block_buffer(buf).unwrap();
}

#[allow(clippy::too_many_arguments)]
fn positive(
    flavor: features::FsFlavor,
    bs: u32,
    entry: Entry,
    shared: bool,
    parent_inline: bool,
    target_inline: bool,
    unknown_links: bool,
) {
    let (dev, fs, parent, clock) = specimen(flavor, bs);
    let name: &[u8] = if matches!(entry, Entry::String) {
        b"d"
    } else {
        b"d\xff"
    };
    let path = [b"/p/".as_slice(), name].concat();
    let target = fs.resolve(&path).unwrap();
    let peer = fs.resolve(b"/ascii").unwrap();
    fs.apply_setxattr_bytes(&path, NAME, &[b'v'; 600]).unwrap();
    if shared {
        share_attributes(&fs, target, peer);
    }
    if target_inline {
        inline_directory(&fs, target, parent, &[]);
    }
    if parent_inline {
        let d = fs.resolve(b"/p/d").unwrap();
        let raw = fs.resolve(b"/p/d\xff").unwrap();
        let full = fs.resolve(b"/p/full").unwrap();
        let link = fs
            .find_entry_in_dir(parent, &fs.read_inode_verified(parent).unwrap().0, b"link")
            .unwrap();
        inline_directory(
            &fs,
            parent,
            2,
            &[
                (d, b"d", 2),
                (raw, b"d\xff", 2),
                (full, b"full", 2),
                (link, b"link", 7),
            ],
        );
    }
    if unknown_links {
        let (inode, mut raw) = fs.read_inode_verified(parent).unwrap();
        raw[26..28].copy_from_slice(&1u16.to_le_bytes());
        fs.finalize_inode_raw(parent, inode.generation, &mut raw)
            .unwrap();
        let mut buf = BlockBuffer::new(bs);
        fs.buffer_write_inode(&mut buf, parent, &raw).unwrap();
        let offset = crate::superblock::SUPERBLOCK_OFFSET;
        let block = buf.get_mut(&fs, offset / u64::from(bs)).unwrap();
        let at = (offset % u64::from(bs)) as usize;
        let sb = &mut block[at..at + 1024];
        let ro = u32::from_le_bytes(sb[100..104].try_into().unwrap())
            | features::RoCompat::DIR_NLINK.bits();
        sb[100..104].copy_from_slice(&ro.to_le_bytes());
        if fs.csum.enabled {
            let sum = crate::checksum::linux_crc32c(!0, &sb[..1020]);
            sb[1020..].copy_from_slice(&sum.to_le_bytes());
        }
        fs.commit_block_buffer(buf).unwrap();
    }
    drop(fs);
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
    let (before, raw) = fs.read_inode_verified(target).unwrap();
    let parent_before = fs.read_inode_verified(parent).unwrap().0;
    let peer_raw = fs.read_inode_verified(peer).unwrap().1;
    assert_eq!(before.has_inline_data(), target_inline);
    assert_eq!(parent_before.has_inline_data(), parent_inline);
    assert_eq!(parent_before.links_count == 1, unknown_links);
    let attr = before.file_acl;
    assert_ne!(attr, 0);
    assert!(allocated(&fs, attr));
    let storage = if target_inline {
        Vec::new()
    } else {
        vec![fs.map_inode_logical(&before, 0).unwrap().unwrap()]
    };
    assert!(storage.iter().all(|b| allocated(&fs, *b)));
    let counts = counters(&dev);
    let groups = group_counters(&fs);
    assert_eq!((groups.0, groups.1), counts);
    clock.now.store(STAMP, Ordering::SeqCst);
    let fs = with_test_mount(fs, |h, fs| {
        if matches!(entry, Entry::CPath | Entry::CAt) {
            assert_eq!(unsafe { fs_ext4_rmdir(h, std::ptr::null()) }, -1);
            assert_eq!(fs_ext4_last_errno(), crate::error::errno::EINVAL);
        }
        call(fs, h, parent.into(), entry, name).unwrap();
        assert!(matches!(fs.resolve(&path), Err(Error::NotFound)));
        assert!(!fs.inode_bit_is_set(target).unwrap());
        assert!(matches!(
            fs.live_inode(InodeRef::new(target, before.generation)),
            Err(Error::Stale)
        ));
        let freed = before.blocks / (u64::from(bs) / 512) - u64::from(shared);
        assert_eq!(counters(&dev), (counts.0 + freed, counts.1 + 1));
        assert_eq!(
            group_counters(fs),
            (groups.0 + freed, groups.1 + 1, groups.2 - 1)
        );
        assert!(storage.iter().all(|b| !allocated(fs, *b)));
        assert_eq!(allocated(fs, attr), shared);
        let parent_after = fs.read_inode_verified(parent).unwrap().0;
        assert_eq!(
            parent_after.links_count,
            if unknown_links {
                1
            } else {
                parent_before.links_count - 1
            }
        );
        assert_eq!(parent_after.has_inline_data(), parent_inline);
        for retained in [b"full".as_slice(), b"link"] {
            assert!(fs
                .find_entry_in_dir(parent, &parent_after, retained)
                .is_ok());
        }
        let dead = fs.read_inode_verified(target).unwrap().1;
        assert_eq!(u16::from_le_bytes(dead[..2].try_into().unwrap()), 0);
        assert_eq!(u16::from_le_bytes(dead[26..28].try_into().unwrap()), 0);
        assert_eq!(
            u32::from_le_bytes(dead[100..104].try_into().unwrap()),
            before.generation
        );
        assert_eq!(
            u32::from_le_bytes(dead[20..24].try_into().unwrap()),
            STAMP as u32
        );
        assert!(raw != dead);
        assert_eq!(fs.read_inode_verified(peer).unwrap().1, peer_raw);
        let mut payload = [0; 12];
        assert_eq!(fs.read_ino(peer, 0, &mut payload).unwrap(), 12);
        assert_eq!(&payload, b"peer payload");
        if shared {
            let b = fs.read_block(attr).unwrap();
            assert_eq!(u32::from_le_bytes(b[4..8].try_into().unwrap()), 1);
            assert!(fs.csum.verify_xattr_block(attr, &b));
            let (p, r) = fs.read_inode_verified(peer).unwrap();
            assert_eq!(
                crate::xattr::get_resolved(fs, &p, &r, NAME).unwrap(),
                Some(vec![b'v'; 600])
            );
        }
        assert_eq!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
    });
    drop(fs);
    assert_ne!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
    let fs = Filesystem::mount(dev).unwrap();
    assert!(matches!(fs.resolve(&path), Err(Error::NotFound)));
    assert_eq!(fs.read_inode_verified(peer).unwrap().1, peer_raw);
    eprintln!("positive rmdir {flavor:?} bs={bs} entry={entry:?} shared={shared} parent_inline={parent_inline} target_inline={target_inline} unknown_links={unknown_links}: counters/used_dirs/links/attributes/peer/dirty-close/reopen retained");
}
#[test]
fn real_all_entry_removals_preserve_accounting_attributes_and_reopen() {
    for flavor in [
        features::FsFlavor::Ext2,
        features::FsFlavor::Ext3,
        features::FsFlavor::Ext4,
    ] {
        for bs in [1024, 4096] {
            for entry in ENTRIES {
                for shared in [false, true] {
                    positive(flavor, bs, entry, shared, false, false, false);
                }
            }
        }
    }
}
#[test]
fn real_inline_and_dir_nlink_removals_preserve_buffered_parent_and_peer() {
    for bs in [1024, 4096] {
        for parent in [false, true] {
            for target in [false, true] {
                for shared in [false, true] {
                    for unknown in [false, true] {
                        for entry in [Entry::Bytes, Entry::CAt] {
                            positive(
                                features::FsFlavor::Ext4,
                                bs,
                                entry,
                                shared,
                                parent,
                                target,
                                unknown,
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn real_parent_symlink_and_trailing_slashes_preserve_final_entry_semantics() {
    for entry in [Entry::String, Entry::Bytes, Entry::CPath] {
        let (dev, fs, parent, clock) = specimen(features::FsFlavor::Ext4, 4096);
        fs.apply_symlink("p", "/parent-link").unwrap();
        drop(fs);
        let fs = Filesystem::mount_with_runtime(dev.clone(), clock).unwrap();
        let link = fs
            .find_entry_in_dir(2, &fs.read_inode_verified(2).unwrap().0, b"parent-link")
            .unwrap();
        let (path, name): (&[u8], &[u8]) = if matches!(entry, Entry::String) {
            (b"/parent-link/d///", b"d")
        } else {
            (b"/parent-link/d\xff///", b"d\xff")
        };
        let before = fs.read_inode_verified(parent).unwrap().0.links_count;
        let target = fs
            .find_entry_in_dir(parent, &fs.read_inode_verified(parent).unwrap().0, name)
            .unwrap();
        let fs = with_test_mount(fs, |h, fs| {
            match entry {
                Entry::String => fs.apply_rmdir(std::str::from_utf8(path).unwrap()).unwrap(),
                Entry::Bytes => fs.apply_rmdir_bytes(path).unwrap(),
                Entry::CPath => {
                    let c = CString::new(path).unwrap();
                    assert_eq!(unsafe { fs_ext4_rmdir(h, c.as_ptr()) }, 0);
                    assert_eq!(fs_ext4_last_errno(), 0);
                }
                _ => unreachable!(),
            }
            assert_eq!(
                fs.read_inode_verified(parent).unwrap().0.links_count,
                before - 1
            );
            assert!(!fs.inode_bit_is_set(target).unwrap());
            assert_eq!(fs.read_link(link).unwrap(), b"p");
            assert_eq!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
        });
        drop(fs);
        assert_ne!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
        let fs = Filesystem::mount(dev).unwrap();
        assert!(matches!(
            fs.find_entry_in_dir(parent, &fs.read_inode_verified(parent).unwrap().0, name),
            Err(Error::NotFound)
        ));
        assert_eq!(fs.read_link(link).unwrap(), b"p");
        eprintln!("positive rmdir parent-symlink/trailing-slashes entry={entry:?}: target removed, parent link count and symlink retained across reopen");
    }
}

#[test]
fn rmdir_foreign_context_and_malformed_superblocks_refuse_before_callbacks() {
    for bs in [1024, 4096] {
        let (_, first, _, _) = specimen(features::FsFlavor::Ext4, bs);
        let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, bs);
        let mut foreign = first.chmod_operation().unwrap();
        assert!(matches!(
            fs.rmdir_bytes_in(&mut foreign, b"/p/d"),
            Err(Error::Stale)
        ));
        assert!(matches!(
            fs.rmdir_path_in(&mut foreign, 2, b"d"),
            Err(Error::Stale)
        ));
        assert!(matches!(
            fs.rmdir_from_in(&mut foreign, 2u32, b"d"),
            Err(Error::Stale)
        ));
        assert!(matches!(
            fs.commit_rmdir_in(&mut foreign, BlockBuffer::new(bs)),
            Err(Error::Stale)
        ));
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        let mut context = fs.chmod_operation().unwrap();
        let status = context.operation.status();
        let bytes = dev.bytes();
        assert!(matches!(
            fs.commit_rmdir_in(&mut context, BlockBuffer::new(bs)),
            Err(Error::Corrupt(_))
        ));
        for bytes in [
            vec![],
            vec![0; 1023],
            vec![0; 1025],
            vec![0; bs as usize],
            vec![0; bs as usize + 1],
        ] {
            let mut buf = BlockBuffer::new(bs);
            buf.put(crate::superblock::SUPERBLOCK_OFFSET / u64::from(bs), bytes);
            assert!(matches!(
                fs.commit_rmdir_in(&mut context, buf),
                Err(Error::Corrupt(_))
            ));
            assert_eq!(context.operation.status(), status);
        }
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert_eq!(dev.bytes(), bytes);
        assert!(!fs.operation_failed.load(Ordering::SeqCst));
    }
}

#[test]
fn rmdir_marker_and_commit_errors_latch_while_context_is_retained() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for prior_dirty in [false, true] {
            let (dev, fs, parent) = fault_specimen(flavor, 4096, prior_dirty);
            let mut context = fs.chmod_operation().unwrap();
            let buf = fs.plan_rmdir_at(parent.into(), b"d").unwrap();
            dev.reset();
            dev.fail_write.store(1, Ordering::SeqCst);
            assert!(fs.commit_rmdir_in(&mut context, buf).is_err());
            assert!(dev.counts()[1] > 0);
            assert_eq!(context.operation.status().health, Health::Faulted);
            assert!(fs.operation_failed.load(Ordering::SeqCst));
            dev.reset();
            for family in 0..7 {
                assert!(matches!(metadata(&fs, family), Err(Error::ReadOnly)));
            }
            assert!(matches!(fs.apply_rmdir("/p/d"), Err(Error::ReadOnly)));
            assert_eq!(dev.counts(), [0; 4]);
            drop(context);
            assert!(retired_observation(&dev, fs));
        }
    }
}

#[test]
fn rmdir_publication_unwind_retires_mount_before_retained_context_drops() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        let (dev, fs, parent, _) = specimen(flavor, 4096);
        let mut context = fs.chmod_operation().unwrap();
        let buf = fs.plan_rmdir_at(parent.into(), b"d").unwrap();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _held = fs.uninit_cleared.lock().unwrap();
            panic!("rmdir allocation publication poison");
        }))
        .is_err());
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            fs.commit_rmdir_in(&mut context, buf)
        }))
        .is_err());
        assert_eq!(context.operation.status().health, Health::Uncertain);
        assert!(fs.operation_failed.load(Ordering::SeqCst));
        assert!(dev.counts()[1] > 0);
        dev.reset();
        assert!(matches!(fs.apply_rmdir("/p/d"), Err(Error::ReadOnly)));
        assert_eq!(dev.counts(), [0; 4]);
        drop(context);
        assert!(retired_observation(&dev, fs));
    }
}

#[test]
fn second_path_verdict_precedes_resolved_regular_parent_dot_policy() {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
    drop(fs);
    let gate = Arc::new(Revoke {
        inner: dev.clone(),
        armed: AtomicBool::new(false),
        queries: AtomicUsize::new(0),
    });
    let fs = Filesystem::mount_with_runtime(gate.clone(), clock).unwrap();
    let before = dev.bytes();
    let status = fs.chmod_operation().unwrap().operation.status();
    dev.reset();
    gate.armed.store(true, Ordering::SeqCst);
    assert!(matches!(fs.apply_rmdir("/ascii/."), Err(Error::ReadOnly)));
    assert_eq!(gate.queries.load(Ordering::SeqCst), 2);
    assert_eq!(&dev.counts()[1..3], &[0, 0]);
    assert_eq!(dev.bytes(), before);
    assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
}
