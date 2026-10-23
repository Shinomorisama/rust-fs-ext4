//! Executed native mkdir safety audit; self-built fixtures are not Linux qualification.
//! Constructed SB diagnostics now exercise the actual mkdir-only commit boundary.
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
) -> std::result::Result<u32, i32> {
    let path = [b"/p/".as_slice(), name].concat();
    match entry {
        Entry::String => fs
            .apply_mkdir(std::str::from_utf8(&path).unwrap(), 0o2750)
            .map_err(|e| e.to_errno()),
        Entry::Bytes => fs
            .apply_mkdir_bytes(&path, 0o2750)
            .map_err(|e| e.to_errno()),
        Entry::At => fs
            .apply_mkdir_at(parent, name, 0o2750)
            .map_err(|e| e.to_errno()),
        Entry::CPath => {
            let path = CString::new(path).unwrap();
            let ino = unsafe { fs_ext4_mkdir(h, path.as_ptr(), 0o2750) };
            if ino == 0 {
                Err(fs_ext4_last_errno())
            } else {
                assert_eq!(fs_ext4_last_errno(), 0);
                Ok(ino)
            }
        }
        Entry::CAt => {
            let mut attr: fs_ext4_attr_t = unsafe { std::mem::zeroed() };
            let rc = unsafe {
                fs_ext4_mkdir_at(
                    h,
                    parent.ino,
                    parent.generation.unwrap_or(FS_EXT4_GEN_ANY),
                    name.as_ptr().cast(),
                    name.len(),
                    0o2750,
                    &mut attr,
                )
            };
            if rc == 0 {
                assert_eq!(fs_ext4_last_errno(), 0);
                Ok(attr.inode)
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
        let got = call(fs, h, parent.into(), entry, b"new");
        eprintln!(
            "held mkdir {entry:?}: result={got:?} callbacks={:?}",
            dev.counts()
        );
        assert_eq!(
            got,
            Err(crate::error::errno::ENOTSUP),
            "mkdir bypassed shared context"
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
    Empty,
    Slash,
    Nul,
    Long,
    Stale,
    File,
    Duplicate,
    Dot,
    Dotdot,
    Checksum,
    Links,
    Inodes,
    Blocks,
}
const REFUSALS: [Refusal; 13] = [
    Refusal::Empty,
    Refusal::Slash,
    Refusal::Nul,
    Refusal::Long,
    Refusal::Stale,
    Refusal::File,
    Refusal::Duplicate,
    Refusal::Dot,
    Refusal::Dotdot,
    Refusal::Checksum,
    Refusal::Links,
    Refusal::Inodes,
    Refusal::Blocks,
];
fn refusal(capi: bool, kind: Refusal, dirty: bool) {
    let (dev, fs, parent, clock) = specimen(features::FsFlavor::Ext4, 4096);
    let mut buf = BlockBuffer::new(4096);
    match kind {
        Refusal::Checksum => {
            let inode = fs.read_inode_verified(parent).unwrap().0;
            let block = fs.map_inode_logical(&inode, 0).unwrap().unwrap();
            buf.get_mut(&fs, block).unwrap()[4095] ^= 1;
        }
        Refusal::Links => {
            let (inode, mut raw) = fs.read_inode_verified(parent).unwrap();
            raw[26..28].copy_from_slice(&65000u16.to_le_bytes());
            fs.finalize_inode_raw(parent, inode.generation, &mut raw)
                .unwrap();
            fs.buffer_write_inode(&mut buf, parent, &raw).unwrap();
        }
        Refusal::Inodes | Refusal::Blocks => {
            for (gi, g) in fs.groups.iter().enumerate() {
                let inode = matches!(kind, Refusal::Inodes);
                buf.put(
                    if inode {
                        g.inode_bitmap
                    } else {
                        g.block_bitmap
                    },
                    vec![0xff; 4096],
                );
                fs.buffer_refresh_bitmap_csum(&mut buf, gi, inode).unwrap();
            }
        }
        _ => {}
    }
    fs.commit_block_buffer(buf).unwrap();
    drop(fs);
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
    if dirty {
        fs.apply_chmod("/ascii", 0o600).unwrap();
    }
    let generation = fs.read_inode_verified(parent).unwrap().0.generation;
    let file = fs.resolve(b"/ascii").unwrap();
    let long = [b'x'; 256];
    let (dir, name, expected): (InodeRef, &[u8], Option<i32>) = match kind {
        Refusal::Empty => (parent.into(), b"", Some(crate::error::errno::EINVAL)),
        Refusal::Slash => (
            parent.into(),
            b"bad/name",
            Some(crate::error::errno::EINVAL),
        ),
        Refusal::Nul => (
            parent.into(),
            b"bad\0name",
            Some(crate::error::errno::EINVAL),
        ),
        Refusal::Long => (
            parent.into(),
            &long,
            Some(crate::error::errno::ENAMETOOLONG),
        ),
        Refusal::Stale => (
            InodeRef::new(parent, generation ^ 1),
            b"new",
            Some(crate::error::errno::ESTALE),
        ),
        Refusal::File => (file.into(), b"new", Some(crate::error::errno::ENOTDIR)),
        Refusal::Duplicate => (parent.into(), b"d", Some(crate::error::errno::EEXIST)),
        Refusal::Dot => (parent.into(), b".", Some(crate::error::errno::EEXIST)),
        Refusal::Dotdot => (parent.into(), b"..", Some(crate::error::errno::EEXIST)),
        Refusal::Checksum => (
            parent.into(),
            b"new",
            Some(
                Error::BadChecksum {
                    what: "directory block",
                }
                .to_errno(),
            ),
        ),
        Refusal::Links => (parent.into(), b"new", Some(crate::error::errno::EMLINK)),
        Refusal::Inodes | Refusal::Blocks => (parent.into(), b"new", None),
    };
    let status = fs.chmod_operation().unwrap().operation.status();
    dev.reset();
    clock.calls.store(0, Ordering::SeqCst);
    let before = dev.bytes();
    let _fs = with_test_mount(fs, |h, fs| {
        let got = call(fs, h, dir, if capi { Entry::CAt } else { Entry::At }, name);
        if let Some(e) = expected {
            assert_eq!(got, Err(e));
        } else {
            assert!(got.is_err(), "exhausted allocation unexpectedly succeeded");
        }
        eprintln!(
            "mkdir refusal {kind:?} capi={capi} dirty={dirty}: result={got:?} writes={} flushes={}",
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
    ($name:ident,$c:expr,$kind:ident) => {
        #[test]
        fn $name() {
            refusal($c, Refusal::$kind, false);
        }
    };
}
refusal_test!(rust_empty_must_not_mark_clean_mount, false, Empty);
refusal_test!(c_empty_must_not_mark_clean_mount, true, Empty);
refusal_test!(rust_slash_must_not_mark_clean_mount, false, Slash);
refusal_test!(c_slash_must_not_mark_clean_mount, true, Slash);
refusal_test!(rust_nul_must_not_mark_clean_mount, false, Nul);
refusal_test!(c_nul_must_not_mark_clean_mount, true, Nul);
refusal_test!(rust_long_must_not_mark_clean_mount, false, Long);
refusal_test!(c_long_must_not_mark_clean_mount, true, Long);
refusal_test!(rust_stale_must_not_mark_clean_mount, false, Stale);
refusal_test!(c_stale_must_not_mark_clean_mount, true, Stale);
refusal_test!(rust_file_must_not_mark_clean_mount, false, File);
refusal_test!(c_file_must_not_mark_clean_mount, true, File);
refusal_test!(rust_duplicate_must_not_mark_clean_mount, false, Duplicate);
refusal_test!(c_duplicate_must_not_mark_clean_mount, true, Duplicate);
refusal_test!(rust_dot_must_not_mark_clean_mount, false, Dot);
refusal_test!(c_dot_must_not_mark_clean_mount, true, Dot);
refusal_test!(rust_dotdot_must_not_mark_clean_mount, false, Dotdot);
refusal_test!(c_dotdot_must_not_mark_clean_mount, true, Dotdot);
refusal_test!(rust_checksum_must_not_mark_clean_mount, false, Checksum);
refusal_test!(c_checksum_must_not_mark_clean_mount, true, Checksum);
refusal_test!(rust_links_must_not_mark_clean_mount, false, Links);
refusal_test!(c_links_must_not_mark_clean_mount, true, Links);
refusal_test!(rust_inodes_must_not_mark_clean_mount, false, Inodes);
refusal_test!(c_inodes_must_not_mark_clean_mount, true, Inodes);
refusal_test!(rust_blocks_must_not_mark_clean_mount, false, Blocks);
refusal_test!(c_blocks_must_not_mark_clean_mount, true, Blocks);

#[test]
fn prior_dirty_refusals_preserve_errors_bytes_and_context() {
    for c in [false, true] {
        for kind in REFUSALS {
            refusal(c, kind, true);
        }
    }
}
fn path_refusals(capi: bool) {
    for (path, errno) in [
        (b"".as_slice(), crate::error::errno::EINVAL),
        (b"relative", crate::error::errno::EINVAL),
        (b"/", crate::error::errno::EINVAL),
        (b"/missing/new", crate::error::errno::ENOENT),
        (b"/ascii/new", crate::error::errno::ENOTDIR),
        (b"/p/d", crate::error::errno::EEXIST),
    ] {
        let (dev, fs, _, _) = specimen(features::FsFlavor::Ext4, 4096);
        let before = dev.bytes();
        let _fs = with_test_mount(fs, |h, fs| {
            let got = if capi {
                let p = CString::new(path).unwrap();
                let rc = unsafe { fs_ext4_mkdir(h, p.as_ptr(), 0o750) };
                assert_eq!(rc, 0);
                Err(fs_ext4_last_errno())
            } else {
                fs.apply_mkdir_bytes(path, 0o750).map_err(|e| e.to_errno())
            };
            assert_eq!(got, Err(errno));
            eprintln!(
                "mkdir path refusal capi={capi} path={path:?} writes={}",
                dev.counts()[1]
            );
            assert_eq!(&dev.counts()[1..3], &[0, 0]);
            assert!(dev.bytes() == before);
        });
    }
}
#[test]
fn rust_path_refusals_must_be_untouched() {
    path_refusals(false);
}
#[test]
fn c_path_refusals_must_be_untouched() {
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
fn held_context_precedes_parent_conversion() {
    let (dev, fs, parent, _) = specimen(features::FsFlavor::Ext4, 4096);
    let held = fs.chmod_operation().unwrap();
    let calls = AtomicUsize::new(0);
    let got = fs.apply_mkdir_at(
        Convert {
            calls: &calls,
            parent,
            hook: None,
        },
        b"new",
        0o750,
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(matches!(got, Err(Error::Unsupported(_))));
    assert_eq!(dev.counts(), [0; 4]);
    drop(held);
}
#[test]
fn invalid_long_and_volume_priorities_precede_conversion() {
    let (dev, fs, parent, _) = specimen(features::FsFlavor::Ext4, 4096);
    let calls = AtomicUsize::new(0);
    let long = [b'x'; 256];
    for name in [b"".as_slice(), b"bad/name", b"bad\0name", &long] {
        let got = fs.apply_mkdir_at(
            Convert {
                calls: &calls,
                parent,
                hook: None,
            },
            name,
            0o750,
        );
        assert_eq!(
            got.unwrap_err().to_errno(),
            if name.len() > 255 {
                crate::error::errno::ENAMETOOLONG
            } else {
                crate::error::errno::EINVAL
            }
        );
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    dev.writable.store(false, Ordering::SeqCst);
    assert!(matches!(
        fs.apply_mkdir_at(
            Convert {
                calls: &calls,
                parent,
                hook: None
            },
            b"new",
            0o750
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
        7 => fs.apply_rmdir("/p/d"),
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
fn conversion_must_hold_context_against_eight_guarded_workers() {
    let (_, fs, parent, _) = specimen(features::FsFlavor::Ext4, 4096);
    let fs = Arc::new(fs);
    let calls = AtomicUsize::new(0);
    let results = Mutex::new(Vec::new());
    let hook = || {
        for family in 0..8 {
            results.lock().unwrap().push(worker(fs.clone(), family));
        }
    };
    fs.apply_mkdir_at(
        Convert {
            calls: &calls,
            parent,
            hook: Some(&hook),
        },
        b"new",
        0o750,
    )
    .unwrap();
    let results = results.lock().unwrap();
    eprintln!("mkdir Into guarded-worker refusals={results:?}");
    assert_eq!(&*results, &[true; 8]);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

struct HookRuntime {
    inner: Arc<Clock>,
    generation: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}
impl crate::runtime::Runtime for HookRuntime {
    fn now_unix_seconds(&self) -> i64 {
        self.inner.now_unix_seconds()
    }
    fn next_inode_generation(&self) -> u32 {
        let h = self.generation.lock().unwrap().take();
        if let Some(h) = h {
            h();
        }
        0x12345678
    }
}
struct CallbackDevice {
    inner: Arc<Device>,
    hook: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
    phase: u8,
    skip: AtomicUsize,
}
impl CallbackDevice {
    fn fire(&self, phase: u8) {
        if phase == self.phase {
            if self.skip.load(Ordering::SeqCst) > 0 {
                self.skip.fetch_sub(1, Ordering::SeqCst);
                return;
            }
            let h = self.hook.lock().unwrap().take();
            if let Some(h) = h {
                h();
            }
        }
    }
}
impl BlockDevice for CallbackDevice {
    fn read_at(&self, at: u64, b: &mut [u8]) -> Result<()> {
        self.fire(0);
        self.inner.read_at(at, b)
    }
    fn write_at(&self, at: u64, b: &[u8]) -> Result<()> {
        self.fire(1);
        self.inner.write_at(at, b)
    }
    fn flush(&self) -> Result<()> {
        self.fire(2);
        self.inner.flush()
    }
    fn size_bytes(&self) -> u64 {
        self.inner.size_bytes()
    }
    fn is_writable(&self) -> bool {
        self.fire(3);
        self.inner.is_writable()
    }
}
fn device_callback(phase: u8, skip: usize) {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
    drop(fs);
    let gate = Arc::new(CallbackDevice {
        inner: dev,
        hook: Mutex::new(None),
        phase,
        skip: AtomicUsize::new(skip),
    });
    let fs = Arc::new(Filesystem::mount_inner(gate.clone(), false, 0, clock).unwrap());
    let weak = Arc::downgrade(&fs);
    let results = Arc::new(Mutex::new(Vec::new()));
    let out = results.clone();
    *gate.hook.lock().unwrap() = Some(Arc::new(move || {
        out.lock().unwrap().push(worker(weak.upgrade().unwrap(), 0));
    }));
    fs.apply_mkdir("/p/new", 0o750).unwrap();
    let result = results.lock().unwrap();
    eprintln!("mkdir device phase={phase} guarded-worker refusal={result:?}");
    assert_eq!(&*result, &[true]);
}
#[test]
fn read_callback_excludes_guarded_worker() {
    device_callback(0, 0);
}
#[test]
fn marker_write_callback_excludes_guarded_worker() {
    device_callback(1, 0);
}
#[test]
fn marker_flush_callback_excludes_guarded_worker() {
    device_callback(2, 0);
}
#[test]
fn writability_callback_excludes_guarded_worker() {
    device_callback(3, 0);
}
fn runtime_callback(generation: bool) {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
    drop(fs);
    let runtime = Arc::new(HookRuntime {
        inner: clock.clone(),
        generation: Mutex::new(None),
    });
    let fs = Arc::new(Filesystem::mount_with_runtime(dev, runtime.clone()).unwrap());
    let weak = Arc::downgrade(&fs);
    let results = Arc::new(Mutex::new(Vec::new()));
    let out = results.clone();
    let once = AtomicBool::new(true);
    let hook: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
        if once.swap(false, Ordering::SeqCst) {
            out.lock().unwrap().push(worker(weak.upgrade().unwrap(), 0));
        }
    });
    if generation {
        *runtime.generation.lock().unwrap() = Some(hook);
    } else {
        *clock.hook.lock().unwrap() = Some(hook);
    }
    fs.apply_mkdir("/p/new", 0o750).unwrap();
    *clock.hook.lock().unwrap() = None;
    let result = results.lock().unwrap();
    eprintln!("mkdir runtime generation={generation} worker refusal={result:?}");
    assert_eq!(&*result, &[true]);
}
#[test]
fn time_callback_excludes_guarded_worker() {
    runtime_callback(false);
}
#[test]
fn generation_callback_excludes_guarded_worker() {
    runtime_callback(true);
}
#[test]
fn eight_guarded_time_callbacks_exclude_mkdir_worker() {
    let mut observed = Vec::new();
    for family in 0..8 {
        let (_, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
        let fs = Arc::new(fs);
        let weak = Arc::downgrade(&fs);
        let once = AtomicBool::new(true);
        let result = Arc::new(Mutex::new(Vec::new()));
        let out = result.clone();
        *clock.hook.lock().unwrap() = Some(Arc::new(move || {
            if once.swap(false, Ordering::SeqCst) {
                let fs = weak.upgrade().unwrap();
                let (tx, rx) = mpsc::channel();
                let j = std::thread::spawn(move || {
                    tx.send(matches!(
                        fs.apply_mkdir("/p/nested", 0o750),
                        Err(Error::Unsupported(_))
                    ))
                    .unwrap();
                });
                let refused = rx
                    .recv_timeout(Duration::from_secs(5))
                    .expect("mkdir worker blocked");
                j.join().unwrap();
                out.lock().unwrap().push(refused);
            }
        }));
        metadata(&fs, family).unwrap();
        *clock.hook.lock().unwrap() = None;
        let r = result.lock().unwrap();
        assert_eq!(r.len(), 1);
        observed.push(r[0]);
    }
    eprintln!("eight guarded-family callbacks mkdir refusals={observed:?}");
    assert_eq!(observed, vec![true; 8]);
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
    assert!(matches!(
        fs.apply_mkdir("/p/new", 0o750),
        Err(Error::ReadOnly)
    ));
    assert_eq!(gate.queries.load(Ordering::SeqCst), 2);
    eprintln!(
        "mkdir second verdict writes={} flushes={}",
        dev.counts()[1],
        dev.counts()[2]
    );
    assert_eq!(&dev.counts()[1..3], &[0, 0]);
    assert!(dev.bytes() == before);
    assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
}

#[test]
fn volume_ffi_and_generation_priorities_are_retained() {
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
        let _fs = with_test_mount(fs, |h, fs| {
            let got = fs.apply_mkdir_at(parent, b"", 0o750);
            assert!(match gate {
                1 => matches!(got, Err(Error::JournalNotReplayed)),
                4 => matches!(got, Err(Error::UnsupportedIncompat(_))),
                5 => matches!(got, Err(Error::UnsupportedRoCompat(_))),
                _ => matches!(got, Err(Error::ReadOnly)),
            });
            dev.reset();
            assert_eq!(unsafe { fs_ext4_mkdir(h, std::ptr::null(), 0o750) }, 0);
            assert_eq!(fs_ext4_last_errno(), crate::error::errno::EINVAL);
            assert_eq!(
                unsafe {
                    fs_ext4_mkdir_at(
                        h,
                        parent,
                        FS_EXT4_GEN_ANY,
                        b"x".as_ptr().cast(),
                        usize::MAX,
                        0o750,
                        std::ptr::null_mut(),
                    )
                },
                -1
            );
            assert_eq!(fs_ext4_last_errno(), crate::error::errno::ENAMETOOLONG);
            assert_eq!(dev.counts(), [0; 4]);
            assert!(dev.bytes() == before);
        });
    }
    let (_, fs, parent, _) = specimen(features::FsFlavor::Ext4, 4096);
    let stale = InodeRef::new(parent, 0x87654321);
    assert!(matches!(
        fs.apply_mkdir_at(stale, b"d", 0o750),
        Err(Error::Stale)
    ));
    assert_eq!(
        fs.apply_mkdir_at(stale, &[b'x'; 256], 0o750)
            .unwrap_err()
            .to_errno(),
        crate::error::errno::ENAMETOOLONG
    );
}
fn retired_observation(dev: &Device, fs: Filesystem) -> bool {
    let latched = fs.operation_failed.load(Ordering::SeqCst);
    dev.reset();
    let before = dev.bytes();
    let mut refused = true;
    for f in 0..8 {
        refused &= metadata(&fs, f).is_err();
    }
    refused &= fs.apply_mkdir("/p/after-fault", 0o750).is_err();
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
    *clock.hook.lock().unwrap() = Some(Arc::new(|| panic!("mkdir creation-time planning unwind")));
    let fs = if capi {
        with_test_mount(fs, |h, fs| {
            assert_eq!(
                call(fs, h, parent.into(), Entry::CPath, b"new"),
                Err(crate::error::errno::EIO)
            );
        })
    } else {
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || fs.apply_mkdir("/p/new", 0o750)
        ))
        .is_err());
        fs
    };
    *clock.hook.lock().unwrap() = None;
    let unchanged = dev.bytes() == before && dev.counts()[1..3] == [0, 0];
    let latched = fs.operation_failed.load(Ordering::SeqCst);
    let retired = retired_observation(&dev, fs);
    eprintln!("mkdir planning unwind capi={capi} dirty={dirty}: unchanged={unchanged} latched={latched} nine families/close retired={retired}");
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
    let hook = || panic!("mkdir Into unwind");
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fs.apply_mkdir_at(
            Convert {
                calls: &calls,
                parent,
                hook: Some(&hook)
            },
            b"new",
            0o750
        )))
        .is_err()
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let unchanged = dev.bytes() == before && dev.counts()[1..3] == [0, 0];
    let retired = retired_observation(&dev, fs);
    eprintln!("mkdir Into unwind unchanged={unchanged} retired={retired}");
    assert!(unchanged && retired);
}

fn fault_specimen(
    flavor: features::FsFlavor,
    bs: u32,
    dirty: bool,
) -> (Arc<Device>, Filesystem, u32) {
    let (dev, fs, parent, _) = specimen(flavor, bs);
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
                fs.apply_mkdir("/p/new", 0o750).unwrap();
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
                                call(fs, h, parent.into(), Entry::CAt, b"new"),
                                Err(crate::error::errno::EIO)
                            );
                        })
                    } else {
                        if unwind {
                            assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                                || fs.apply_mkdir("/p/new", 0o750)
                            ))
                            .is_err());
                        } else {
                            assert!(fs.apply_mkdir("/p/new", 0o750).is_err());
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
    eprintln!("mkdir fault sweep capi={capi} flush={flush} unwind={unwind}: {exercised} observed faults; nine-family/flush/close retirement violations={violations}");
    assert_eq!(
        violations, 0,
        "mkdir uncertainty did not retire every guarded family and close"
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
    fs.buffer_patch_sb_counters(&mut buf, -1, -1).unwrap();
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
    // Constructed trusted staging isolates the actual mkdir boundary's SB rule.
    fs.commit_mkdir_in(&mut context, MkdirPlan { ino: 0, buf })
        .unwrap();
    assert_eq!(counters(&dev), (before.0 - 1, before.1 - 1));
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
    eprintln!("constructed mkdir boundary bs={bs} journal={}: credits/state/checksum/sentinels preserved; clean={}", fs.journal.is_some(), state(&dev) & crate::superblock::EXT4_VALID_FS);
    assert_eq!(
        state(&dev) & crate::superblock::EXT4_VALID_FS,
        0,
        "mkdir boundary restored clean state"
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

#[derive(Clone, Copy, Debug)]
enum Shape {
    Plain,
    Full,
    Inline,
    Convert,
    Unknown,
}
fn full_parent(fs: &Filesystem, parent: u32, name: &[u8]) {
    let bs = fs.sb.block_size();
    let has_ft = fs.sb.feature_incompat & features::Incompat::FILETYPE.bits() != 0;
    for i in 0..64 {
        let inode = fs.read_inode_verified(parent).unwrap().0;
        assert_eq!(
            inode.size,
            u64::from(bs),
            "fixture must still be exactly one full block"
        );
        let phys = fs.map_inode_logical(&inode, 0).unwrap().unwrap();
        let mut copy = fs.read_block(phys).unwrap();
        let tail = if fs.csum.enabled { 12 } else { 0 };
        if matches!(
            crate::dir::add_entry_to_block(
                &mut copy,
                4000,
                name,
                crate::dir::DirEntryType::Directory,
                has_ft,
                tail
            ),
            Err(Error::OutOfBounds)
        ) {
            return;
        }
        let fill = format!("fill-{i:02}-{}", "x".repeat(220));
        fs.apply_create_at(parent, fill.as_bytes(), 0o640).unwrap();
    }
    panic!("fixture never reached actual parent saturation");
}
fn unknown_links(fs: &Filesystem, parent: u32) {
    let (inode, mut raw) = fs.read_inode_verified(parent).unwrap();
    raw[26..28].copy_from_slice(&1u16.to_le_bytes());
    fs.finalize_inode_raw(parent, inode.generation, &mut raw)
        .unwrap();
    let mut buf = BlockBuffer::new(fs.sb.block_size());
    fs.buffer_write_inode(&mut buf, parent, &raw).unwrap();
    let bs = u64::from(fs.sb.block_size());
    let block = buf.get_mut(fs, 1024 / bs).unwrap();
    let at = (1024 % bs) as usize;
    let sb = &mut block[at..at + 1024];
    let ro =
        u32::from_le_bytes(sb[100..104].try_into().unwrap()) | features::RoCompat::DIR_NLINK.bits();
    sb[100..104].copy_from_slice(&ro.to_le_bytes());
    if fs.csum.enabled {
        let sum = crate::checksum::linux_crc32c(!0, &sb[..1020]);
        sb[1020..].copy_from_slice(&sum.to_le_bytes());
    }
    fs.commit_block_buffer(buf).unwrap();
}
fn positive(flavor: features::FsFlavor, bs: u32, entry: Entry, shape: Shape) {
    let (dev, fs, parent, clock) = specimen(flavor, bs);
    fs.apply_setxattr("/p", NAME, &[b'p'; 600]).unwrap();
    let name: Vec<u8> = if matches!(shape, Shape::Full | Shape::Convert) {
        vec![b'n'; 255]
    } else if matches!(entry, Entry::String) {
        b"new".to_vec()
    } else {
        b"n\xff".to_vec()
    };
    if matches!(shape, Shape::Full) {
        full_parent(&fs, parent, &name);
    }
    if matches!(shape, Shape::Inline | Shape::Convert) {
        let inode = fs.read_inode_verified(parent).unwrap().0;
        let entries: Vec<_> = [b"d".as_slice(), b"d\xff", b"full", b"link"]
            .iter()
            .map(|n| {
                let e = fs.find_dir_entry(parent, &inode, n).unwrap();
                (e.inode, e.name, e.file_type)
            })
            .collect();
        let entries: Vec<_> = entries
            .iter()
            .map(|(ino, name, kind)| (*ino, name.as_slice(), *kind as u8))
            .collect();
        inline_directory(&fs, parent, 2, &entries);
    }
    if matches!(shape, Shape::Unknown) {
        unknown_links(&fs, parent);
    }
    drop(fs);
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
    let before_parent = fs.read_inode_verified(parent).unwrap().0;
    assert_eq!(
        before_parent.has_inline_data(),
        matches!(shape, Shape::Inline | Shape::Convert)
    );
    let same_table_before = (parent - 1) * u32::from(fs.sb.inode_size) / bs;
    let peer = fs.resolve(b"/ascii").unwrap();
    let peer_raw = fs.read_inode_verified(peer).unwrap().1;
    let parent_ea = fs.read_block(before_parent.file_acl).unwrap();
    let counts = counters(&dev);
    let groups = group_counters(&fs);
    clock.now.store(STAMP, Ordering::SeqCst);
    let mut created = 0;
    let fs = with_test_mount(fs, |h, fs| {
        created = call(fs, h, parent.into(), entry, &name).unwrap();
        let (child, raw) = fs.read_inode_verified(created).unwrap();
        let after = fs.read_inode_verified(parent).unwrap().0;
        assert_eq!(
            (after.mtime, after.mtime_nsec, after.ctime, after.ctime_nsec),
            (STAMP, 0, STAMP, 0)
        );
        assert_eq!(
            (after.atime, after.atime_nsec),
            (before_parent.atime, before_parent.atime_nsec)
        );
        assert_eq!(child.mode, crate::inode::S_IFDIR | 0o2750);
        assert_eq!(child.links_count, 2);
        assert_eq!(child.generation, 0x12345678);
        assert_eq!(
            (child.atime, child.mtime, child.ctime),
            (STAMP, STAMP, STAMP)
        );
        assert_eq!(child.size, u64::from(bs));
        assert_eq!(child.blocks, u64::from(bs / 512));
        assert!(!child.has_inline_data());
        assert_eq!(child.flags & crate::inode::InodeFlags::INDEX.bits(), 0);
        assert_eq!(child.has_extents(), flavor == features::FsFlavor::Ext4);
        if fs.csum.enabled {
            assert!(fs.csum.verify_inode(created, child.generation, &raw));
        }
        assert!(fs.inode_bit_is_set(created).unwrap());
        let block = fs.map_inode_logical(&child, 0).unwrap().unwrap();
        assert!(allocated(fs, block));
        assert_eq!(
            fs.find_entry_in_dir(created, &child, b".").unwrap(),
            created
        );
        assert_eq!(
            fs.find_entry_in_dir(created, &child, b"..").unwrap(),
            parent
        );
        let path = [b"/p/".as_slice(), &name].concat();
        assert_eq!(fs.resolve(&path).unwrap(), created);
        let parent_extra = (after.blocks - before_parent.blocks) / u64::from(bs / 512);
        assert_eq!(counters(&dev), (counts.0 - 1 - parent_extra, counts.1 - 1));
        assert_eq!(
            group_counters(fs),
            (groups.0 - 1 - parent_extra, groups.1 - 1, groups.2 + 1)
        );
        assert_eq!(
            after.links_count,
            if matches!(shape, Shape::Unknown) {
                1
            } else {
                before_parent.links_count + 1
            }
        );
        assert_eq!(after.has_inline_data(), matches!(shape, Shape::Inline));
        assert_eq!(after.atime, before_parent.atime);
        assert_eq!(after.file_acl, before_parent.file_acl);
        assert_eq!(fs.read_block(after.file_acl).unwrap(), parent_ea);
        if matches!(shape, Shape::Full | Shape::Convert) {
            assert!(after.blocks > before_parent.blocks);
        }
        if !after.has_inline_data() {
            for logical in 0..after.size.div_ceil(u64::from(bs)) {
                assert_ne!(fs.map_inode_logical(&after, logical).unwrap(), Some(block));
            }
        }
        assert_eq!(fs.read_inode_verified(peer).unwrap().1, peer_raw);
        let mut bytes = [0; 12];
        assert_eq!(fs.read_ino(peer, 0, &mut bytes).unwrap(), 12);
        assert_eq!(&bytes, b"peer payload");
        assert_eq!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
        let next = fs.apply_mkdir_at(parent, b"next", 0o750).unwrap();
        assert_ne!(created, next);
        let next_inode = fs.read_inode_verified(next).unwrap().0;
        assert_ne!(fs.map_inode_logical(&next_inode, 0).unwrap(), Some(block));
        eprintln!("positive mkdir {flavor:?}/{bs}/{entry:?}/{shape:?}: inode={created} same-table={} parent-extra={parent_extra} links/entries/EA/peer/counters/CRC retained",same_table_before==(created-1)*u32::from(fs.sb.inode_size)/bs);
    });
    drop(fs);
    assert_ne!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
    let fs = Filesystem::mount(dev).unwrap();
    assert_eq!(
        fs.resolve(&[b"/p/".as_slice(), &name].concat()).unwrap(),
        created
    );
    assert_eq!(fs.read_inode_verified(peer).unwrap().1, peer_raw);
}
#[test]
fn real_all_entries_create_and_reopen_with_exact_accounting() {
    for flavor in [
        features::FsFlavor::Ext2,
        features::FsFlavor::Ext3,
        features::FsFlavor::Ext4,
    ] {
        for bs in [1024, 4096] {
            for entry in ENTRIES {
                positive(flavor, bs, entry, Shape::Plain);
            }
        }
    }
}
#[test]
fn real_full_parents_grow_without_losing_child_or_parent() {
    for flavor in [
        features::FsFlavor::Ext2,
        features::FsFlavor::Ext3,
        features::FsFlavor::Ext4,
    ] {
        for bs in [1024, 4096] {
            for entry in [Entry::At, Entry::CAt] {
                positive(flavor, bs, entry, Shape::Full);
            }
        }
    }
}
#[test]
fn real_inline_parent_edits_and_conversion_keep_other_entries() {
    for bs in [1024, 4096] {
        for entry in [Entry::At, Entry::CAt] {
            for shape in [Shape::Inline, Shape::Convert] {
                positive(features::FsFlavor::Ext4, bs, entry, shape);
            }
        }
    }
}
#[test]
fn real_dir_nlink_unknown_count_remains_unknown() {
    for bs in [1024, 4096] {
        for entry in [Entry::At, Entry::CAt] {
            positive(features::FsFlavor::Ext4, bs, entry, Shape::Unknown);
        }
    }
}
#[test]
fn real_parent_symlink_trailing_slash_and_case_distinctions() {
    let (_, fs, parent, _) = specimen(features::FsFlavor::Ext4, 4096);
    fs.apply_symlink("p", "/parent-link").unwrap();
    let a = fs.apply_mkdir("/parent-link/New///", 0o750).unwrap();
    let b = fs.apply_mkdir("/p/new", 0o750).unwrap();
    assert_ne!(a, b);
    assert_eq!(fs.lookup_at(parent, b"New").unwrap(), a);
    assert_eq!(fs.lookup_at(a, b"..").unwrap(), parent);
}

struct ReadbackDevice {
    inner: Arc<Device>,
    read: AtomicUsize,
    at: AtomicUsize,
    fail: AtomicBool,
    hook: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}
impl BlockDevice for ReadbackDevice {
    fn read_at(&self, at: u64, b: &mut [u8]) -> Result<()> {
        let n = self.read.fetch_add(1, Ordering::SeqCst) + 1;
        if n == self.at.load(Ordering::SeqCst) {
            let h = self.hook.lock().unwrap().take();
            if let Some(h) = h {
                h();
            }
            if self.fail.load(Ordering::SeqCst) {
                return Err(std::io::Error::other("mkdir post-commit readback failure").into());
            }
        }
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
    }
}
fn readback_fixture(flavor: features::FsFlavor, bs: u32) -> (Arc<ReadbackDevice>, Filesystem, u32) {
    let (dev, fs, parent, clock) = specimen(flavor, bs);
    drop(fs);
    let gate = Arc::new(ReadbackDevice {
        inner: dev,
        read: AtomicUsize::new(0),
        at: AtomicUsize::new(0),
        fail: AtomicBool::new(false),
        hook: Mutex::new(None),
    });
    let fs = Filesystem::mount_inner(gate.clone(), false, 0, clock).unwrap();
    gate.read.store(0, Ordering::SeqCst);
    (gate, fs, parent)
}
fn readback_count(flavor: features::FsFlavor, bs: u32) -> usize {
    let (gate, fs, parent) = readback_fixture(flavor, bs);
    let _fs = with_test_mount(fs, |h, _| {
        assert_eq!(
            unsafe {
                fs_ext4_mkdir_at(
                    h,
                    parent,
                    FS_EXT4_GEN_ANY,
                    b"new".as_ptr().cast(),
                    3,
                    0o750,
                    std::ptr::null_mut(),
                )
            },
            0
        )
    });
    let reads = gate.read.load(Ordering::SeqCst);
    assert!(reads > 0);
    reads
}
#[test]
fn c_optional_attribute_readback_must_remain_under_context() {
    let mut observed = Vec::new();
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            let reads = readback_count(flavor, bs);
            let (gate, fs, parent) = readback_fixture(flavor, bs);
            let out = Arc::new(Mutex::new(Vec::new()));
            let output = out.clone();
            let fs = with_test_mount(fs, |h, fs| {
                // The callback uses a scoped worker over a shared reference, never an aliased mutable C handle.
                let addr = fs as *const Filesystem as usize;
                *gate.hook.lock().unwrap() = Some(Arc::new(move || {
                    // SAFETY: this synchronous device callback joins before the enclosing shared borrow ends.
                    std::thread::scope(|scope| {
                        let t = scope.spawn(move || {
                            let fs = unsafe { &*(addr as *const Filesystem) };
                            matches!(metadata(fs, 0), Err(Error::Unsupported(_)))
                        });
                        output.lock().unwrap().push(t.join().unwrap());
                    });
                }));
                gate.at.store(reads + 1, Ordering::SeqCst);
                let mut attr: fs_ext4_attr_t = unsafe { std::mem::zeroed() };
                assert_eq!(
                    unsafe {
                        fs_ext4_mkdir_at(
                            h,
                            parent,
                            FS_EXT4_GEN_ANY,
                            b"new".as_ptr().cast(),
                            3,
                            0o750,
                            &mut attr,
                        )
                    },
                    0
                );
                assert_eq!(attr.generation, 0x12345678);
                assert_eq!(fs.lookup_at(parent, b"new").unwrap(), attr.inode);
            });
            drop(fs);
            let o = out.lock().unwrap();
            assert_eq!(o.len(), 1, "post-commit readback callback did not execute");
            observed.push(o[0]);
        }
    }
    eprintln!("mkdir C optional readback guarded refusals={observed:?}");
    assert_eq!(observed, vec![true; 4]);
}
#[test]
fn c_post_commit_attribute_read_failure_keeps_one_created_entry() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            let reads = readback_count(flavor, bs);
            let (gate, fs, parent) = readback_fixture(flavor, bs);
            let before = counters(&gate.inner);
            gate.at.store(reads + 1, Ordering::SeqCst);
            gate.fail.store(true, Ordering::SeqCst);
            let fs = with_test_mount(fs, |h, fs| {
                let mut attr: fs_ext4_attr_t = unsafe { std::mem::zeroed() };
                assert_eq!(
                    unsafe {
                        fs_ext4_mkdir_at(
                            h,
                            parent,
                            FS_EXT4_GEN_ANY,
                            b"new".as_ptr().cast(),
                            3,
                            0o750,
                            &mut attr,
                        )
                    },
                    -1
                );
                assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
                assert_eq!(gate.read.load(Ordering::SeqCst), reads + 1);
                gate.fail.store(false, Ordering::SeqCst);
                let ino = fs.lookup_at(parent, b"new").unwrap();
                assert!(fs.stat_ino(ino).unwrap().is_dir());
                assert_eq!(counters(&gate.inner), (before.0 - 1, before.1 - 1));
            });
            drop(fs);
            eprintln!("positive mkdir {flavor:?}/{bs}: null attr avoids readback, injected first readback failure returns EIO after exactly one inode/data allocation");
        }
    }
}

#[test]
fn real_same_table_child_and_parent_edits_are_both_retained() {
    for bs in [1024, 4096] {
        let (_, fs, _, _) = specimen(features::FsFlavor::Ext4, bs);
        let mut chosen = None;
        for i in 0..8 {
            let parent = fs.apply_mkdir(&format!("/same-{i}"), 0o750).unwrap();
            let plan = crate::alloc::plan_inode_allocation(
                &fs.sb,
                &fs.allocation_groups(),
                true,
                (parent - 1) / fs.sb.inodes_per_group,
                |b| fs.read_block(b),
            )
            .unwrap();
            let a = crate::bgd::locate_inode(&fs.sb, &fs.groups, parent).unwrap();
            let b = crate::bgd::locate_inode(&fs.sb, &fs.groups, plan.inode).unwrap();
            if a.0 == b.0 {
                chosen = Some((parent, plan.inode));
                break;
            }
        }
        let (parent, expected) = chosen.expect("fixture has no shared table block");
        let old = fs.read_inode_verified(parent).unwrap().0;
        let child = fs.apply_mkdir_at(parent, b"child", 0o2750).unwrap();
        assert_eq!(child, expected);
        let now = fs.read_inode_verified(parent).unwrap().0;
        assert_eq!(now.links_count, old.links_count + 1);
        assert_eq!(now.block, old.block);
        assert_eq!(now.mode, old.mode);
        assert_eq!(fs.lookup_at(parent, b"child").unwrap(), child);
        let inode = fs.stat_ino(child).unwrap();
        assert_eq!(inode.mode, crate::inode::S_IFDIR | 0o2750);
        assert_eq!(fs.lookup_at(child, b"..").unwrap(), parent);
        eprintln!("positive mkdir same-table bs={bs}: parent={parent} child={child} asserted same table block and both inode slices retained");
    }
}

#[test]
fn real_uninit_group_growth_allocates_distinct_child_parent_and_next_child() {
    let bs = 2048;
    let size = 40 * 1024 * 1024;
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
        Some([61; 16]),
        size as u64,
        bs,
        features::FsFlavor::Ext4,
    )
    .unwrap();
    let fs = Filesystem::mount(dev.clone()).unwrap();
    assert_eq!(fs.groups.len(), 2);
    full_parent(&fs, 2, &[b'u'; 255]);
    assert_eq!(fs.groups[1].free_inodes_count, fs.sb.inodes_per_group);
    assert_eq!(fs.groups[1].used_dirs_count, 0);
    let mut buf = BlockBuffer::new(bs);
    // Constructed bitmap-allocation profile, not ownership-qualified: reserve every
    // remaining group-0 block so both child and parent growth must use group 1.
    let free0 = fs.groups[0].free_blocks_count;
    buf.put(fs.groups[0].block_bitmap, vec![0xff; bs as usize]);
    fs.buffer_refresh_bitmap_csum(&mut buf, 0, false).unwrap();
    fs.buffer_patch_bgd_counters(&mut buf, 0, -(free0 as i32), 0, 0)
        .unwrap();
    fs.buffer_patch_sb_counters(&mut buf, -(free0 as i64), 0)
        .unwrap();
    let (b, o) = fs.sb.descriptor_location(1);
    let raw = buf.get_mut(&fs, b).unwrap();
    let flags = u16::from_le_bytes(raw[o + 18..o + 20].try_into().unwrap())
        | crate::bgd::BgdFlags::BLOCK_UNINIT.bits()
        | crate::bgd::BgdFlags::INODE_UNINIT.bits();
    raw[o + 18..o + 20].copy_from_slice(&flags.to_le_bytes());
    raw[o + 28..o + 30].copy_from_slice(&(fs.sb.inodes_per_group as u16).to_le_bytes());
    raw[o + 50..o + 52].copy_from_slice(&0u16.to_le_bytes());
    fs.restamp_group_desc_csum(raw, o, 1);
    buf.put(fs.groups[1].block_bitmap, vec![0xa5; bs as usize]);
    buf.put(fs.groups[1].inode_bitmap, vec![0x5a; bs as usize]);
    fs.commit_block_buffer(buf).unwrap();
    drop(fs);
    let fs = Filesystem::mount(dev.clone()).unwrap();
    assert!(fs.groups[1]
        .flags()
        .contains(crate::bgd::BgdFlags::BLOCK_UNINIT | crate::bgd::BgdFlags::INODE_UNINIT));
    let before = counters(&dev);
    let parent = fs.stat_ino(2).unwrap();
    assert_eq!(parent.size, u64::from(bs));
    let a = fs.apply_mkdir_at(2, &[b'u'; 255], 0o750).unwrap();
    let b = fs.apply_mkdir_at(2, &[b'v'; 255], 0o750).unwrap();
    assert_ne!(a, b);
    assert_eq!((a - 1) / fs.sb.inodes_per_group, 1);
    assert_eq!((b - 1) / fs.sb.inodes_per_group, 1);
    let ia = fs.stat_ino(a).unwrap();
    let ib = fs.stat_ino(b).unwrap();
    let root = fs.stat_ino(2).unwrap();
    let pa = fs.map_inode_logical(&ia, 0).unwrap().unwrap();
    let pb = fs.map_inode_logical(&ib, 0).unwrap().unwrap();
    let pp = fs.map_inode_logical(&root, 1).unwrap().unwrap();
    assert_eq!(std::collections::BTreeSet::from([pa, pb, pp]).len(), 3);
    assert!([pa, pb, pp]
        .iter()
        .all(
            |p| ((*p - u64::from(fs.sb.first_data_block)) / u64::from(fs.sb.blocks_per_group)) == 1
        ));
    assert!(!fs.allocation_groups()[1]
        .flags()
        .intersects(crate::bgd::BgdFlags::BLOCK_UNINIT | crate::bgd::BgdFlags::INODE_UNINIT));
    assert_eq!(counters(&dev), (before.0 - 3, before.1 - 2));
    assert_eq!(root.links_count, parent.links_count + 2);
    assert_eq!(fs.lookup_at(2, &[b'u'; 255]).unwrap(), a);
    assert_eq!(fs.lookup_at(2, &[b'v'; 255]).unwrap(), b);
    let (desc, off) = fs.sb.descriptor_location(1);
    let raw = fs.read_block(desc).unwrap();
    let descriptor = crate::bgd::BlockGroupDescriptor::parse(
        &raw[off..off + fs.sb.desc_size as usize],
        fs.sb.desc_size,
    )
    .unwrap();
    assert_eq!(u64::from(descriptor.free_blocks_count), before.0 - 3);
    assert_eq!(descriptor.free_inodes_count, fs.sb.inodes_per_group - 2);
    assert_eq!(descriptor.used_dirs_count, 2);
    for (block, inode, stored) in [
        (
            fs.groups[1].block_bitmap,
            false,
            descriptor.block_bitmap_csum,
        ),
        (
            fs.groups[1].inode_bitmap,
            true,
            descriptor.inode_bitmap_csum,
        ),
    ] {
        let bytes = fs.read_block(block).unwrap();
        let coverage = if inode {
            fs.sb.inodes_per_group as usize / 8
        } else {
            fs.sb.blocks_per_group as usize / 8
        };
        assert_eq!(stored, fs.csum.crc(&bytes[..coverage]));
    }
    assert!(fs.inode_bit_is_set(a).unwrap() && fs.inode_bit_is_set(b).unwrap());
    let (inode, raw_a) = fs.read_inode_verified(a).unwrap();
    assert!(fs.csum.verify_inode(a, inode.generation, &raw_a));
    let bm = fs.read_block(fs.groups[1].inode_bitmap).unwrap();
    assert!(bm[(fs.sb.inodes_per_group / 8) as usize..]
        .iter()
        .all(|b| *b == 0xff));
    let expected_floor = fs.sb.inodes_per_group - ((b - 1) % fs.sb.inodes_per_group) - 1;
    let unused = u32::from(u16::from_le_bytes(
        raw[off + 28..off + 30].try_into().unwrap(),
    )) | (u32::from(u16::from_le_bytes(
        raw[off + 50..off + 52].try_into().unwrap(),
    )) << 16);
    assert_eq!(unused, expected_floor);
    drop(fs);
    let fs = Filesystem::mount(dev).unwrap();
    assert_eq!(fs.lookup_at(2, &[b'v'; 255]).unwrap(), b);
    eprintln!("positive mkdir 2KiB/2group uninit: two new inodes, three distinct child/parent blocks, bitmap padding/itable-unused/publication and reopen retained");
}

#[test]
fn commit_write_callback_excludes_guarded_worker() {
    device_callback(1, 1);
}
#[test]
fn commit_flush_callback_excludes_guarded_worker() {
    device_callback(2, 1);
}

#[test]
fn public_allocation_publication_unwind_must_retire_mount() {
    let mut violations = 0;
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            let (dev, fs, parent, clock) = specimen(flavor, bs);
            let fs = Arc::new(fs);
            let weak = Arc::downgrade(&fs);
            *clock.hook.lock().unwrap() = Some(Arc::new(move || {
                let fs = weak.upgrade().unwrap();
                assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let _held = fs.uninit_cleared.lock().unwrap();
                    panic!("mkdir publication poison after allocation planning");
                }))
                .is_err());
            }));
            assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || fs.apply_mkdir("/p/new", 0o750)
            ))
            .is_err());
            *clock.hook.lock().unwrap() = None;
            assert!(
                dev.counts()[1] > 1,
                "must reach committed data, not just marker"
            );
            assert!(
                fs.find_entry_in_dir(parent, &fs.read_inode_verified(parent).unwrap().0, b"new")
                    .is_ok(),
                "publication unwind must occur after committed namespace"
            );
            let latched = fs.operation_failed.load(Ordering::SeqCst);
            dev.reset();
            let before = dev.bytes();
            let mut all_refused = true;
            for family in 0..8 {
                all_refused &= matches!(
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| metadata(
                        &fs, family
                    ))),
                    Ok(Err(_))
                );
            }
            all_refused &= matches!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                    || fs.apply_mkdir("/p/after-pub", 0o750)
                )),
                Ok(Err(_))
            );
            all_refused &= fs.flush().is_err();
            drop(fs);
            let retired = latched && all_refused && dev.counts() == [0; 4] && dev.bytes() == before;
            eprintln!("mkdir {flavor:?}/{bs} actual public post-commit publication unwind: latched={latched}, nine-family/flush/close retirement={retired}");
            violations += usize::from(!retired);
        }
    }
    assert_eq!(violations, 0, "publication unwind did not retire mount");
}

#[test]
fn mkdir_foreign_context_and_malformed_superblocks_refuse_before_callbacks() {
    for bs in [1024, 4096] {
        let (_, first, _, _) = specimen(features::FsFlavor::Ext4, bs);
        let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, bs);
        let mut foreign = first.chmod_operation().unwrap();
        assert!(matches!(
            fs.mkdir_bytes_in(&mut foreign, b"/p/new", 0o750),
            Err(Error::Stale)
        ));
        assert!(matches!(
            fs.mkdir_path_in(&mut foreign, 2, b"new", 0o750),
            Err(Error::Stale)
        ));
        assert!(matches!(
            fs.mkdir_from_in(&mut foreign, 2u32, b"new", 0o750),
            Err(Error::Stale)
        ));
        assert!(matches!(
            fs.commit_mkdir_in(
                &mut foreign,
                MkdirPlan {
                    ino: 0,
                    buf: BlockBuffer::new(bs)
                }
            ),
            Err(Error::Stale)
        ));
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        let mut context = fs.chmod_operation().unwrap();
        let status = context.operation.status();
        let original = dev.bytes();
        assert!(matches!(
            fs.commit_mkdir_in(
                &mut context,
                MkdirPlan {
                    ino: 0,
                    buf: BlockBuffer::new(bs)
                }
            ),
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
                fs.commit_mkdir_in(&mut context, MkdirPlan { ino: 0, buf }),
                Err(Error::Corrupt(_))
            ));
            assert_eq!(context.operation.status(), status);
        }
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert_eq!(dev.bytes(), original);
        assert!(!fs.operation_failed.load(Ordering::SeqCst));
    }
}

#[test]
fn mkdir_marker_and_commit_errors_latch_while_context_is_retained() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for prior_dirty in [false, true] {
            let (dev, fs, parent) = fault_specimen(flavor, 4096, prior_dirty);
            let mut context = fs.chmod_operation().unwrap();
            let plan = fs.plan_mkdir_at(parent.into(), b"new", 0o750).unwrap();
            dev.reset();
            dev.fail_write.store(1, Ordering::SeqCst);
            assert!(fs.commit_mkdir_in(&mut context, plan).is_err());
            assert!(dev.counts()[1] > 0);
            assert_eq!(context.operation.status().health, Health::Faulted);
            assert!(fs.operation_failed.load(Ordering::SeqCst));
            dev.reset();
            for family in 0..8 {
                assert!(matches!(metadata(&fs, family), Err(Error::ReadOnly)));
            }
            assert!(matches!(
                fs.apply_mkdir("/p/new", 0o750),
                Err(Error::ReadOnly)
            ));
            assert_eq!(dev.counts(), [0; 4]);
            drop(context);
            assert!(retired_observation(&dev, fs));
        }
    }
}

#[test]
fn mkdir_publication_unwind_retires_mount_before_retained_context_drops() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        let (dev, fs, parent, _) = specimen(flavor, 4096);
        let mut context = fs.chmod_operation().unwrap();
        let plan = fs.plan_mkdir_at(parent.into(), b"new", 0o750).unwrap();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _held = fs.uninit_cleared.lock().unwrap();
            panic!("mkdir allocation publication poison");
        }))
        .is_err());
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || fs.commit_mkdir_in(&mut context, plan)
        ))
        .is_err());
        assert_eq!(context.operation.status().health, Health::Uncertain);
        assert!(fs.operation_failed.load(Ordering::SeqCst));
        assert!(dev.counts()[1] > 0);
        dev.reset();
        assert!(matches!(
            fs.apply_mkdir("/p/new", 0o750),
            Err(Error::ReadOnly)
        ));
        assert_eq!(dev.counts(), [0; 4]);
        drop(context);
        assert!(retired_observation(&dev, fs));
    }
}

#[test]
fn second_mkdir_path_verdict_precedes_regular_parent_and_long_name_policy() {
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
    let path = format!("/ascii/{}", "n".repeat(256));
    assert!(matches!(fs.apply_mkdir(&path, 0o750), Err(Error::ReadOnly)));
    assert_eq!(gate.queries.load(Ordering::SeqCst), 2);
    assert_eq!(&dev.counts()[1..3], &[0, 0]);
    assert_eq!(dev.bytes(), before);
    assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
}
