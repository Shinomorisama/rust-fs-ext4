//! Native hard-link safety/positive audit; self-built fixtures are not Linux qualification.
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
    fs.apply_setxattr("/ascii", NAME, &[b'v'; 600]).unwrap();
    fs.apply_chown("/ascii", 1001, 1002).unwrap();
    fs.apply_create("/peer", 0o640).unwrap();
    fs.apply_setxattr("/peer", NAME, b"old").unwrap();
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
    target: InodeRef,
    entry: Entry,
    name: &[u8],
) -> std::result::Result<(), i32> {
    let dst = [b"/p/".as_slice(), name].concat();
    match entry {
        Entry::String => fs
            .apply_link("/ascii", std::str::from_utf8(&dst).unwrap())
            .map_err(|e| e.to_errno()),
        Entry::Bytes => fs
            .apply_link_bytes(b"/ascii", &dst)
            .map_err(|e| e.to_errno()),
        Entry::At => fs
            .apply_link_at(target, parent, name)
            .map_err(|e| e.to_errno()),
        Entry::CPath => {
            let src = CString::new("/ascii").unwrap();
            let dst = CString::new(dst).unwrap();
            let rc = unsafe { fs_ext4_link(h, src.as_ptr(), dst.as_ptr()) };
            if rc == 0 {
                assert_eq!(fs_ext4_last_errno(), 0);
                Ok(())
            } else {
                assert_eq!(rc, -1);
                Err(fs_ext4_last_errno())
            }
        }
        Entry::CAt => {
            let mut attr: fs_ext4_attr_t = unsafe { std::mem::zeroed() };
            let rc = unsafe {
                fs_ext4_link_at(
                    h,
                    target.ino,
                    target.generation.unwrap_or(FS_EXT4_GEN_ANY),
                    parent.ino,
                    parent.generation.unwrap_or(FS_EXT4_GEN_ANY),
                    name.as_ptr().cast(),
                    name.len(),
                    &mut attr,
                )
            };
            if rc == 0 {
                assert_eq!(fs_ext4_last_errno(), 0);
                assert_eq!(attr.inode, target.ino);
                Ok(())
            } else {
                assert_eq!(rc, -1);
                Err(fs_ext4_last_errno())
            }
        }
    }
}
fn target(fs: &Filesystem) -> InodeRef {
    let i = fs.resolve(b"/ascii").unwrap();
    InodeRef::new(i, fs.stat_ino(i).unwrap().generation)
}
fn held(entry: Entry) {
    let (dev, fs, parent, _) = specimen(features::FsFlavor::Ext4, 4096);
    let t = target(&fs);
    dev.reset();
    let _fs = with_test_mount(fs, |h, fs| {
        let held = fs.chmod_operation().unwrap();
        let status = held.operation.status();
        let before = dev.bytes();
        let got = call(fs, h, parent.into(), t, entry, b"new");
        eprintln!(
            "link held {entry:?}: got={got:?}, callbacks={:?}",
            dev.counts()
        );
        assert_eq!(got, Err(crate::error::errno::ENOTSUP));
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(dev.bytes(), before);
        assert_eq!(held.operation.status(), status);
        drop(held);
        call(fs, h, parent.into(), t, entry, b"released").unwrap();
        assert_eq!(fs.lookup_at(parent, b"released").unwrap(), t.ino);
    });
}
macro_rules! held_test {
    ($n:ident,$e:ident) => {
        #[test]
        fn $n() {
            held(Entry::$e);
        }
    };
}
held_test!(held_string_before_callbacks, String);
held_test!(held_bytes_before_callbacks, Bytes);
held_test!(held_counted_before_callbacks, At);
held_test!(held_c_path_before_callbacks, CPath);
held_test!(held_c_counted_before_callbacks, CAt);
struct Convert<'a> {
    calls: &'a AtomicUsize,
    value: InodeRef,
    hook: Option<&'a dyn Fn()>,
}
impl From<Convert<'_>> for InodeRef {
    fn from(v: Convert<'_>) -> Self {
        v.calls.fetch_add(1, Ordering::SeqCst);
        if let Some(h) = v.hook {
            h();
        }
        v.value
    }
}
fn held_into(which: bool) {
    let (dev, fs, p, _) = specimen(features::FsFlavor::Ext4, 4096);
    let t = target(&fs);
    let n = AtomicUsize::new(0);
    let held = fs.chmod_operation().unwrap();
    dev.reset();
    let v = Convert {
        calls: &n,
        value: if which { t } else { p.into() },
        hook: None,
    };
    let got = if which {
        fs.apply_link_at(v, p, b"new")
    } else {
        fs.apply_link_at(t, v, b"new")
    };
    assert!(matches!(got, Err(Error::Unsupported(_))));
    assert_eq!(n.load(Ordering::SeqCst), 0);
    assert_eq!(dev.counts(), [0; 4]);
    drop(held);
}
#[test]
fn held_precedes_source_into() {
    held_into(true);
}
#[test]
fn held_precedes_parent_into() {
    held_into(false);
}
fn patch_links(fs: &Filesystem, ino: u32, n: u16) {
    let (i, mut raw) = fs.read_inode_verified(ino).unwrap();
    raw[26..28].copy_from_slice(&n.to_le_bytes());
    fs.finalize_inode_raw(ino, i.generation, &mut raw).unwrap();
    let mut b = BlockBuffer::new(fs.sb.block_size());
    fs.buffer_write_inode(&mut b, ino, &raw).unwrap();
    fs.commit_block_buffer(b).unwrap();
}
#[test]
fn counted_names_source_and_dual_into_priorities() {
    let (dev, fs, p, _) = specimen(features::FsFlavor::Ext4, 4096);
    let t = target(&fs);
    let a = AtomicUsize::new(0);
    let b = AtomicUsize::new(0);
    for name in [b"".as_slice(), b"bad/name", b"bad\0name", &[b'x'; 256]] {
        let got = fs.apply_link_at(
            Convert {
                calls: &a,
                value: t,
                hook: None,
            },
            Convert {
                calls: &b,
                value: p.into(),
                hook: None,
            },
            name,
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
    assert_eq!((a.load(Ordering::SeqCst), b.load(Ordering::SeqCst)), (0, 0));
    for src in [InodeRef::new(t.ino, t.generation.unwrap() ^ 1), p.into()] {
        assert!(fs
            .apply_link_at(
                Convert {
                    calls: &a,
                    value: src,
                    hook: None
                },
                Convert {
                    calls: &b,
                    value: p.into(),
                    hook: None
                },
                b"new"
            )
            .is_err());
    }
    patch_links(&fs, t.ino, 65000);
    assert!(matches!(
        fs.apply_link_at(
            Convert {
                calls: &a,
                value: t,
                hook: None
            },
            Convert {
                calls: &b,
                value: p.into(),
                hook: None
            },
            b"new"
        ),
        Err(Error::TooManyLinks)
    ));
    assert_eq!((a.load(Ordering::SeqCst), b.load(Ordering::SeqCst)), (3, 0));
    dev.writable.store(false, Ordering::SeqCst);
    assert!(matches!(
        fs.apply_link_at(
            Convert {
                calls: &a,
                value: t,
                hook: None
            },
            Convert {
                calls: &b,
                value: p.into(),
                hook: None
            },
            b""
        ),
        Err(Error::ReadOnly)
    ));
    assert_eq!((a.load(Ordering::SeqCst), b.load(Ordering::SeqCst)), (3, 0));
}
fn metadata(fs: &Filesystem, family: u8) -> Result<()> {
    match family {
        0 => fs.apply_chmod("/peer", 0o600),
        1 => fs.apply_chown("/peer", 1001, 1002),
        2 => fs.apply_utimens("/peer", STAMP, 0, STAMP, 0),
        3 => fs.apply_set_flags(
            "/peer",
            crate::inode::InodeFlags::EXTENTS.bits() | crate::inode::InodeFlags::NODUMP.bits(),
        ),
        4 => fs.apply_removexattr("/peer", NAME),
        5 => fs.apply_setxattr("/peer", NAME, b"new"),
        6 => fs.apply_unlink("/peer"),
        7 => fs.apply_rmdir("/p/d"),
        8 => fs.apply_mkdir("/p/guarded-directory", 0o750).map(drop),
        9 => fs.apply_create("/p/guarded-file", 0o640).map(drop),
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

fn retired_observation(dev: &Device, fs: Filesystem) -> bool {
    let latched = fs.operation_failed.load(Ordering::SeqCst);
    dev.reset();
    let before = dev.bytes();
    let mut refused = true;
    // Catch legal Rust unwinds as a failed retirement observation, not a test-harness abort.
    for family in 0..10 {
        refused &= matches!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| metadata(&fs, family))),
            Ok(Err(_))
        );
    }
    refused &= matches!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || fs.apply_link("/ascii", "/p/after-fault")
        )),
        Ok(Err(_))
    );
    refused &= fs.flush().is_err();
    // A poisoned publication lock is only safe to drop after operation_failed has latched.
    let dropped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(fs))).is_ok();
    latched && refused && dropped && dev.counts() == [0; 4] && dev.bytes() == before
}
fn conversion_hook(parent: bool, unwind: bool) {
    let (dev, fs, p, _) = specimen(features::FsFlavor::Ext4, 4096);
    let t = target(&fs);
    let fs = Arc::new(fs);
    let n = AtomicUsize::new(0);
    let observed = Mutex::new(Vec::new());
    let before = dev.bytes();
    dev.reset();
    let hook = || {
        if unwind {
            panic!("link Into unwind");
        }
        for family in 0..10 {
            observed.lock().unwrap().push(worker(fs.clone(), family));
        }
    };
    let got = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if parent {
            fs.apply_link_at(
                t,
                Convert {
                    calls: &n,
                    value: p.into(),
                    hook: Some(&hook),
                },
                b"new",
            )
        } else {
            fs.apply_link_at(
                Convert {
                    calls: &n,
                    value: t,
                    hook: Some(&hook),
                },
                p,
                b"new",
            )
        }
    }));
    assert_eq!(n.load(Ordering::SeqCst), 1);
    if unwind {
        assert!(got.is_err());
        let unchanged = dev.bytes() == before && dev.counts()[1..3] == [0, 0];
        let retired = retired_observation(&dev, Arc::try_unwrap(fs).ok().unwrap());
        eprintln!("link Into unwind parent={parent}: unchanged={unchanged} retired={retired}");
        assert!(unchanged && retired);
    } else {
        got.unwrap().unwrap();
        let o = observed.lock().unwrap();
        eprintln!("link Into parent={parent} ten-family refusals={o:?}");
        assert_eq!(*o, vec![true; 10]);
    }
}
#[test]
fn source_into_excludes_ten_families() {
    conversion_hook(false, false);
}
#[test]
fn parent_into_excludes_ten_families() {
    conversion_hook(true, false);
}
#[test]
fn source_into_unwind_retires_without_marker() {
    conversion_hook(false, true);
}
#[test]
fn parent_into_unwind_retires_without_marker() {
    conversion_hook(true, true);
}
struct CallbackDevice {
    inner: Arc<Device>,
    hook: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
    phase: u8,
    after: AtomicUsize,
}
impl CallbackDevice {
    fn fire(&self, phase: u8) {
        if phase == self.phase {
            if self.after.load(Ordering::SeqCst) > 0 {
                self.after.fetch_sub(1, Ordering::SeqCst);
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
        self.fire(4);
        self.inner.size_bytes()
    }
    fn is_writable(&self) -> bool {
        self.fire(3);
        self.inner.is_writable()
    }
}

fn device_callback(phase: u8, after: usize, same: bool) {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
    drop(fs);
    let gate = Arc::new(CallbackDevice {
        inner: dev.clone(),
        hook: Mutex::new(None),
        phase,
        after: AtomicUsize::new(after),
    });
    let fs = Arc::new(Filesystem::mount_inner(gate.clone(), false, 0, clock).unwrap());
    let weak = Arc::downgrade(&fs);
    let out = Arc::new(Mutex::new(Vec::new()));
    let observed = out.clone();
    *gate.hook.lock().unwrap() = Some(Arc::new(move || {
        let fs = weak.upgrade().unwrap();
        for family in 0..=10 {
            let before = dev.counts();
            let refused = if same {
                matches!(
                    if family == 10 {
                        fs.apply_link("/ascii", "/p/nested")
                    } else {
                        metadata(&fs, family)
                    },
                    Err(Error::Unsupported(_))
                )
            } else {
                let fs = fs.clone();
                let (tx, rx) = mpsc::channel();
                let j = std::thread::spawn(move || {
                    tx.send(matches!(
                        if family == 10 {
                            fs.apply_link("/ascii", "/p/nested")
                        } else {
                            metadata(&fs, family)
                        },
                        Err(Error::Unsupported(_))
                    ))
                    .unwrap();
                });
                let r = rx
                    .recv_timeout(Duration::from_secs(5))
                    .expect("link callback worker blocked");
                j.join().unwrap();
                r
            };
            observed
                .lock()
                .unwrap()
                .push(refused && dev.counts() == before);
        }
    }));
    fs.apply_link("/ascii", "/p/new").unwrap();
    let o = out.lock().unwrap();
    eprintln!(
        "link device phase={phase} after={after} same={same}: eleven callback-free refusals={o:?}"
    );
    assert_eq!(
        *o,
        vec![true; 11],
        "hook must execute and refuse all nested forms without callbacks"
    );
}
macro_rules! callback_test {
    ($n:ident,$p:expr,$s:expr) => {
        #[test]
        fn $n() {
            device_callback($p, $s, false);
        }
    };
}
callback_test!(read_excludes_eleven_families, 0, 0);
callback_test!(marker_write_excludes_eleven_families, 1, 0);
callback_test!(commit_write_excludes_eleven_families, 1, 1);
callback_test!(marker_flush_excludes_eleven_families, 2, 0);
callback_test!(commit_flush_excludes_eleven_families, 2, 1);
callback_test!(writable_excludes_eleven_families, 3, 0);
callback_test!(size_excludes_eleven_families, 4, 0);
#[test]
fn same_thread_read_excludes_eleven_families() {
    device_callback(0, 0, true);
}
#[test]
fn ten_qualified_family_callbacks_exclude_link() {
    let mut observations = Vec::new();
    for family in 0..10 {
        let (_, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
        let fs = Arc::new(fs);
        let weak = Arc::downgrade(&fs);
        let out = Arc::new(Mutex::new(Vec::new()));
        let observed = out.clone();
        let once = AtomicBool::new(true);
        *clock.hook.lock().unwrap() = Some(Arc::new(move || {
            if once.swap(false, Ordering::SeqCst) {
                let fs = weak.upgrade().unwrap();
                let (tx, rx) = mpsc::channel();
                let j = std::thread::spawn(move || {
                    tx.send(matches!(
                        fs.apply_link("/ascii", "/p/nested"),
                        Err(Error::Unsupported(_))
                    ))
                    .unwrap()
                });
                observed.lock().unwrap().push(
                    rx.recv_timeout(Duration::from_secs(5))
                        .expect("reverse worker blocked"),
                );
                j.join().unwrap();
            }
        }));
        metadata(&fs, family).unwrap();
        *clock.hook.lock().unwrap() = None;
        let o = out.lock().unwrap();
        assert_eq!(o.len(), 1);
        observations.push(o[0]);
    }
    eprintln!("link reverse ten-family callback refusals={observations:?}");
    assert_eq!(observations, vec![true; 10]);
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
        fs.apply_link("/ascii", "/p/new"),
        Err(Error::ReadOnly)
    ));
    assert_eq!(gate.queries.load(Ordering::SeqCst), 2);
    eprintln!(
        "link second verdict writes={} flushes={}",
        dev.counts()[1],
        dev.counts()[2]
    );
    assert_eq!(&dev.counts()[1..3], &[0, 0]);
    assert!(dev.bytes() == before);
    assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
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
                return Err(std::io::Error::other("link post-commit readback failure").into());
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
#[derive(Clone, Copy, Debug)]
enum Refusal {
    Empty,
    Slash,
    Nul,
    Long,
    StaleSource,
    StaleParent,
    Directory,
    FileParent,
    Duplicate,
    Dot,
    Dotdot,
    MaxLinks,
    SourceChecksum,
    DirChecksum,
    NoBlocks,
    Unsupported,
}
const REFUSALS: [Refusal; 16] = [
    Refusal::Empty,
    Refusal::Slash,
    Refusal::Nul,
    Refusal::Long,
    Refusal::StaleSource,
    Refusal::StaleParent,
    Refusal::Directory,
    Refusal::FileParent,
    Refusal::Duplicate,
    Refusal::Dot,
    Refusal::Dotdot,
    Refusal::MaxLinks,
    Refusal::SourceChecksum,
    Refusal::DirChecksum,
    Refusal::NoBlocks,
    Refusal::Unsupported,
];
fn refusal(c: bool, kind: Refusal, dirty: bool) {
    let (dev, fs, p, clock) = specimen(features::FsFlavor::Ext4, 4096);
    let original = target(&fs);
    if matches!(kind, Refusal::NoBlocks) {
        full_parent(&fs, p, &[b'x'; 255]);
    }
    if matches!(kind, Refusal::MaxLinks) {
        patch_links(&fs, original.ino, 65000);
    }
    let mut buf = BlockBuffer::new(4096);
    match kind {
        Refusal::SourceChecksum => {
            let (block, off) = crate::bgd::locate_inode(&fs.sb, &fs.groups, original.ino).unwrap();
            buf.get_mut(&fs, block).unwrap()[off as usize + 8] ^= 1;
        }
        Refusal::DirChecksum => {
            let i = fs.stat_ino(p).unwrap();
            let b = fs.map_inode_logical(&i, 0).unwrap().unwrap();
            buf.get_mut(&fs, b).unwrap()[4095] ^= 1;
        }
        Refusal::NoBlocks => {
            for (gi, g) in fs.groups.iter().enumerate() {
                buf.put(g.block_bitmap, vec![0xff; 4096]);
                fs.buffer_refresh_bitmap_csum(&mut buf, gi, false).unwrap();
            }
        }
        Refusal::Unsupported => {
            let (i, mut raw) = fs.read_inode_verified(p).unwrap();
            raw[32..36].copy_from_slice(
                &(i.flags | crate::inode::InodeFlags::CASEFOLD.bits()).to_le_bytes(),
            );
            fs.finalize_inode_raw(p, i.generation, &mut raw).unwrap();
            fs.buffer_write_inode(&mut buf, p, &raw).unwrap();
        }
        _ => {}
    }
    fs.commit_block_buffer(buf).unwrap();
    drop(fs);
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock).unwrap();
    if dirty {
        fs.apply_chmod("/peer", 0o600).unwrap();
    }
    let long = [b'x'; 256];
    let mut t = original;
    let mut parent: InodeRef = p.into();
    let (name, expected): (&[u8], Option<i32>) = match kind {
        Refusal::Empty => (b"", Some(crate::error::errno::EINVAL)),
        Refusal::Slash => (b"bad/name", Some(crate::error::errno::EINVAL)),
        Refusal::Nul => (b"bad\0name", Some(crate::error::errno::EINVAL)),
        Refusal::Long => (&long, Some(crate::error::errno::ENAMETOOLONG)),
        Refusal::StaleSource => {
            t = InodeRef::new(t.ino, t.generation.unwrap() ^ 1);
            (b"new", Some(crate::error::errno::ESTALE))
        }
        Refusal::StaleParent => {
            parent = InodeRef::new(p, fs.stat_ino(p).unwrap().generation ^ 1);
            (b"new", Some(crate::error::errno::ESTALE))
        }
        Refusal::Directory => {
            t = p.into();
            (b"new", Some(crate::error::errno::EISDIR))
        }
        Refusal::FileParent => {
            parent = original;
            (b"new", Some(crate::error::errno::ENOTDIR))
        }
        Refusal::Duplicate => (b"d", Some(crate::error::errno::EEXIST)),
        Refusal::Dot => (b".", Some(crate::error::errno::EEXIST)),
        Refusal::Dotdot => (b"..", Some(crate::error::errno::EEXIST)),
        Refusal::MaxLinks => (b"new", Some(Error::TooManyLinks.to_errno())),
        Refusal::NoBlocks => (&long[..255], None),
        _ => (b"new", None),
    };
    let status = fs.chmod_operation().unwrap().operation.status();
    let flags = fs.uninit_cleared.lock().unwrap().cleared.clone();
    let before = dev.bytes();
    dev.reset();
    let _fs = with_test_mount(fs, |h, fs| {
        let got = call(
            fs,
            h,
            parent,
            t,
            if c { Entry::CAt } else { Entry::At },
            name,
        );
        if let Some(e) = expected {
            assert_eq!(got, Err(e));
        } else {
            assert!(got.is_err(), "constructed refusal was not reached");
        }
        let unchanged = dev.bytes() == before && dev.counts()[1..3] == [0, 0];
        eprintln!("link refusal {kind:?} c={c} dirty={dirty}: got={got:?} unchanged={unchanged}");
        assert_eq!(fs.uninit_cleared.lock().unwrap().cleared, flags);
        assert!(!fs.operation_failed.load(Ordering::SeqCst));
        assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
        assert!(
            unchanged,
            "ordinary refusal changed bytes/marker/counters/nlink"
        );
        assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), dirty);
    });
}
macro_rules! refusal_test {
    ($n:ident,$c:expr,$k:ident) => {
        #[test]
        fn $n() {
            refusal($c, Refusal::$k, false);
        }
    };
}
refusal_test!(rust_empty_refusal_must_be_untouched, false, Empty);
refusal_test!(c_empty_refusal_must_be_untouched, true, Empty);
refusal_test!(rust_slash_refusal_must_be_untouched, false, Slash);
refusal_test!(c_slash_refusal_must_be_untouched, true, Slash);
refusal_test!(rust_nul_refusal_must_be_untouched, false, Nul);
refusal_test!(c_nul_refusal_must_be_untouched, true, Nul);
refusal_test!(rust_long_refusal_must_be_untouched, false, Long);
refusal_test!(c_long_refusal_must_be_untouched, true, Long);
refusal_test!(
    rust_stalesource_refusal_must_be_untouched,
    false,
    StaleSource
);
refusal_test!(c_stalesource_refusal_must_be_untouched, true, StaleSource);
refusal_test!(
    rust_staleparent_refusal_must_be_untouched,
    false,
    StaleParent
);
refusal_test!(c_staleparent_refusal_must_be_untouched, true, StaleParent);
refusal_test!(rust_directory_refusal_must_be_untouched, false, Directory);
refusal_test!(c_directory_refusal_must_be_untouched, true, Directory);
refusal_test!(rust_fileparent_refusal_must_be_untouched, false, FileParent);
refusal_test!(c_fileparent_refusal_must_be_untouched, true, FileParent);
refusal_test!(rust_duplicate_refusal_must_be_untouched, false, Duplicate);
refusal_test!(c_duplicate_refusal_must_be_untouched, true, Duplicate);
refusal_test!(rust_dot_refusal_must_be_untouched, false, Dot);
refusal_test!(c_dot_refusal_must_be_untouched, true, Dot);
refusal_test!(rust_dotdot_refusal_must_be_untouched, false, Dotdot);
refusal_test!(c_dotdot_refusal_must_be_untouched, true, Dotdot);
refusal_test!(rust_maxlinks_refusal_must_be_untouched, false, MaxLinks);
refusal_test!(c_maxlinks_refusal_must_be_untouched, true, MaxLinks);
refusal_test!(
    rust_sourcechecksum_refusal_must_be_untouched,
    false,
    SourceChecksum
);
refusal_test!(
    c_sourcechecksum_refusal_must_be_untouched,
    true,
    SourceChecksum
);
refusal_test!(
    rust_dirchecksum_refusal_must_be_untouched,
    false,
    DirChecksum
);
refusal_test!(c_dirchecksum_refusal_must_be_untouched, true, DirChecksum);
refusal_test!(rust_noblocks_refusal_must_be_untouched, false, NoBlocks);
refusal_test!(c_noblocks_refusal_must_be_untouched, true, NoBlocks);
refusal_test!(
    rust_unsupported_refusal_must_be_untouched,
    false,
    Unsupported
);
refusal_test!(c_unsupported_refusal_must_be_untouched, true, Unsupported);
#[test]
fn dirty_refusals_preserve_all_bytes_health_and_pending_flags() {
    for c in [false, true] {
        for kind in REFUSALS {
            refusal(c, kind, true);
        }
    }
}
fn path_refusals(c: bool) {
    let mut violations = 0;
    for (src, dst, e) in [
        (
            b"/ascii".as_slice(),
            b"".as_slice(),
            crate::error::errno::EINVAL,
        ),
        (b"/ascii", b"bare", crate::error::errno::EINVAL),
        (b"/ascii", b"/", crate::error::errno::EINVAL),
        (b"/missing", b"/p/new", crate::error::errno::ENOENT),
        (b"/p", b"/missing/new", crate::error::errno::EISDIR),
        (b"/ascii", b"/missing/new", crate::error::errno::ENOENT),
        (b"/ascii", b"/ascii/new", crate::error::errno::ENOTDIR),
        (b"/ascii", b"/p/d", crate::error::errno::EEXIST),
        (b"/ascii/", b"/p/new", crate::error::errno::ENOTDIR),
    ] {
        let (dev, fs, _, _) = specimen(features::FsFlavor::Ext4, 4096);
        let before = dev.bytes();
        let _fs = with_test_mount(fs, |h, fs| {
            let got = if c {
                let s = CString::new(src).unwrap();
                let d = CString::new(dst).unwrap();
                assert_eq!(unsafe { fs_ext4_link(h, s.as_ptr(), d.as_ptr()) }, -1);
                Err(fs_ext4_last_errno())
            } else {
                fs.apply_link_bytes(src, dst).map_err(|e| e.to_errno())
            };
            assert_eq!(got, Err(e));
            let unchanged = dev.bytes() == before && dev.counts()[1..3] == [0, 0];
            eprintln!("link path refusal c={c} src={src:?} dst={dst:?} unchanged={unchanged}");
            violations += usize::from(!unchanged);
        });
    }
    assert_eq!(violations, 0);
}
#[test]
fn rust_path_refusals_must_be_untouched() {
    path_refusals(false);
}
#[test]
fn c_path_refusals_must_be_untouched() {
    path_refusals(true);
}
fn fault_specimen(
    flavor: features::FsFlavor,
    bs: u32,
    dirty: bool,
) -> (Arc<Device>, Filesystem, u32, InodeRef) {
    let (dev, fs, p, _) = specimen(flavor, bs);
    let t = target(&fs);
    if dirty {
        fs.apply_chmod("/peer", 0o600).unwrap();
    }
    dev.reset();
    (dev, fs, p, t)
}
fn failures(c: bool, flush: bool, unwind: bool) {
    let mut exercised = 0;
    let mut violations = 0;
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            for dirty in [false, true] {
                let (dev, fs, p, t) = fault_specimen(flavor, bs, dirty);
                let fs = with_test_mount(fs, |h, fs| {
                    call(
                        fs,
                        h,
                        p.into(),
                        t,
                        if c { Entry::CAt } else { Entry::At },
                        b"new",
                    )
                    .unwrap()
                });
                let index = if flush { 2 } else { 1 };
                let total = dev.counts()[index];
                assert!(total > 0);
                drop(fs);
                eprintln!("link measured c={c} flush={flush} unwind={unwind} {flavor:?}/{bs}/dirty={dirty}: positions={total}");
                for nth in 1..=total {
                    let (dev, fs, p, t) = fault_specimen(flavor, bs, dirty);
                    (if unwind {
                        &dev.panic_flush
                    } else if flush {
                        &dev.fail_flush
                    } else {
                        &dev.fail_write
                    })
                    .store(nth, Ordering::SeqCst);
                    let fs = if c {
                        with_test_mount(fs, |h, fs| {
                            assert_eq!(
                                call(fs, h, p.into(), t, Entry::CAt, b"new"),
                                Err(crate::error::errno::EIO)
                            )
                        })
                    } else {
                        if unwind {
                            assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                                || fs.apply_link_at(t, p, b"new")
                            ))
                            .is_err());
                        } else {
                            assert!(fs.apply_link_at(t, p, b"new").is_err());
                        }
                        fs
                    };
                    assert!(dev.counts()[index] >= nth, "injected position not reached");
                    violations += usize::from(!retired_observation(&dev, fs));
                    exercised += 1;
                }
            }
        }
    }
    eprintln!("link fault sweep c={c} flush={flush} unwind={unwind}: exercised={exercised} retirement_violations={violations}");
    assert_eq!(violations, 0);
}
#[test]
fn rust_every_write_failure_retires() {
    failures(false, false, false);
}
#[test]
fn c_every_write_failure_retires() {
    failures(true, false, false);
}
#[test]
fn rust_every_flush_failure_retires() {
    failures(false, true, false);
}
#[test]
fn c_every_flush_failure_retires() {
    failures(true, true, false);
}
#[test]
fn rust_every_flush_unwind_retires() {
    failures(false, true, true);
}
#[test]
fn c_every_flush_unwind_retires() {
    failures(true, true, true);
}
fn readback_fixture(
    flavor: features::FsFlavor,
    bs: u32,
) -> (Arc<ReadbackDevice>, Filesystem, u32, InodeRef) {
    let (dev, fs, p, clock) = specimen(flavor, bs);
    let t = target(&fs);
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
    (gate, fs, p, t)
}
fn c_at(h: *mut fs_ext4_fs_t, p: u32, t: InodeRef, name: &[u8], attr: *mut fs_ext4_attr_t) -> i32 {
    unsafe {
        fs_ext4_link_at(
            h,
            t.ino,
            t.generation.unwrap_or(FS_EXT4_GEN_ANY),
            p,
            FS_EXT4_GEN_ANY,
            name.as_ptr().cast(),
            name.len(),
            attr,
        )
    }
}
fn readback_count(flavor: features::FsFlavor, bs: u32) -> usize {
    let (g, fs, p, t) = readback_fixture(flavor, bs);
    let _fs = with_test_mount(fs, |h, _| {
        assert_eq!(c_at(h, p, t, b"new", std::ptr::null_mut()), 0)
    });
    let n = g.read.load(Ordering::SeqCst);
    assert!(n > 0);
    n
}
fn readback(mode: u8) {
    let mut violations = 0;
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            let n = readback_count(flavor, bs);
            let (g, fs, p, t) = readback_fixture(flavor, bs);
            let counts = counters(&g.inner);
            let old = fs.stat_ino(t.ino).unwrap();
            g.read.store(0, Ordering::SeqCst);
            let observed = Arc::new(Mutex::new(Vec::new()));
            let out = observed.clone();
            g.at.store(n + 1, Ordering::SeqCst);
            g.fail.store(mode == 1, Ordering::SeqCst);
            let fs = with_test_mount(fs, |h, fs| {
                // Synchronous shared Rust callback; no reentry through a mutable C handle.
                let addr = fs as *const Filesystem as usize;
                if mode != 1 {
                    *g.hook.lock().unwrap() = Some(Arc::new(move || {
                        if mode == 2 {
                            panic!("link Rust device readback unwind");
                        }
                        std::thread::scope(|scope| {
                            let j = scope.spawn(move || {
                                let fs = unsafe { &*(addr as *const Filesystem) };
                                matches!(metadata(fs, 0), Err(Error::Unsupported(_)))
                            });
                            out.lock().unwrap().push(j.join().unwrap());
                        });
                    }));
                }
                let mut attr: fs_ext4_attr_t = unsafe { std::mem::zeroed() };
                attr.inode = 0xfeed1234;
                attr.generation = 0x13572468;
                let rc = c_at(h, p, t, b"new", &mut attr);
                assert_eq!(rc, if mode == 0 { 0 } else { -1 });
                if mode == 0 {
                    assert_eq!(attr.inode, t.ino);
                    assert_eq!(attr.generation, old.generation);
                    assert_eq!(attr.link_count, old.links_count + 1);
                    assert_eq!(
                        (attr.mode, attr.uid, attr.gid, attr.size, attr.blocks_512),
                        (old.mode & 0o7777, old.uid, old.gid, old.size, old.blocks)
                    );
                    violations += usize::from(*observed.lock().unwrap() != vec![true]);
                } else {
                    assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
                    assert_eq!((attr.inode, attr.generation), (0xfeed1234, 0x13572468));
                }
                if mode != 0 {
                    assert_eq!(g.read.load(Ordering::SeqCst), n + 1);
                } else {
                    assert!(g.read.load(Ordering::SeqCst) > n);
                }
                g.fail.store(false, Ordering::SeqCst);
                g.at.store(0, Ordering::SeqCst);
                assert_eq!(fs.lookup_at(p, b"new").unwrap(), t.ino);
                assert_eq!(fs.stat_ino(t.ino).unwrap().links_count, old.links_count + 1);
                assert_eq!(counters(&g.inner), counts);
            });
            if mode == 2 {
                violations += usize::from(!retired_observation(&g.inner, fs));
            } else {
                drop(fs);
            }
            eprintln!("link C readback mode={mode} {flavor:?}/{bs}: existing target increment/entry/accounting retained");
        }
    }
    assert_eq!(violations, 0);
}
#[test]
fn c_readback_holds_context() {
    readback(0);
}
#[test]
fn c_failed_readback_leaves_exactly_one_link_and_sentinels() {
    readback(1);
}
#[test]
fn c_rust_readback_unwind_retires() {
    readback(2);
}
#[test]
fn actual_post_commit_publication_unwind_retires() {
    let mut violations = 0;
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            let (dev, fs, p, clock) = specimen(flavor, bs);
            drop(fs);
            let gate = Arc::new(CallbackDevice {
                inner: dev.clone(),
                hook: Mutex::new(None),
                phase: 2,
                after: AtomicUsize::new(1),
            });
            let fs = Arc::new(Filesystem::mount_inner(gate.clone(), false, 0, clock).unwrap());
            let t = target(&fs);
            let old = fs.read_inode_verified(t.ino).unwrap().0.links_count;
            let weak = Arc::downgrade(&fs);
            // First commit flush occurs after the last planning-state reads for this allocation-free link.
            *gate.hook.lock().unwrap() = Some(Arc::new(move || {
                let fs = weak.upgrade().unwrap();
                assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let _held = fs.uninit_cleared.lock().unwrap();
                    panic!("link publication poison after last planning reads");
                }))
                .is_err());
            }));
            let got = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                fs.apply_link("/ascii", "/p/new")
            }));
            assert!(got.is_err());
            assert_eq!(
                fs.find_entry_in_dir(p, &fs.read_inode_verified(p).unwrap().0, b"new")
                    .unwrap(),
                t.ino,
                "must reach post-commit publication"
            );
            assert_eq!(
                fs.read_inode_verified(t.ino).unwrap().0.links_count,
                old + 1
            );
            let retired = retired_observation(&dev, Arc::try_unwrap(fs).ok().unwrap());
            eprintln!(
                "link publication {flavor:?}/{bs}: namespace/count committed, retired={retired}"
            );
            violations += usize::from(!retired);
        }
    }
    assert_eq!(violations, 0);
}
#[derive(Clone, Copy, Debug)]
enum Shape {
    Plain,
    Full,
    Inline,
    Convert,
    Unknown,
}
fn source_preserved(fs: &Filesystem, t: u32, before: &[u8], links: u16) {
    let (i, raw) = fs.read_inode_verified(t).unwrap();
    assert_eq!(i.links_count, links + 1);
    let mut a = raw;
    let mut b = before.to_vec();
    for (off, len) in [(12, 4), (26, 2), (124, 2), (130, 2), (132, 4)] {
        if off + len > a.len() {
            continue;
        }
        a[off..off + len].fill(0);
        b[off..off + len].fill(0);
    }
    assert_eq!(
        a, b,
        "link changed source beyond nlink/ctime/checksum; time policy is measured separately"
    );
}
fn positive(flavor: features::FsFlavor, bs: u32, entry: Entry, shape: Shape) {
    let (dev, fs, p, clock) = specimen(flavor, bs);
    let t = target(&fs);
    let name = if matches!(shape, Shape::Full | Shape::Convert) {
        vec![b'n'; 255]
    } else if matches!(entry, Entry::String) {
        b"new".to_vec()
    } else {
        b"n\xff".to_vec()
    };
    if matches!(shape, Shape::Full) {
        full_parent(&fs, p, &name);
    }
    if matches!(shape, Shape::Inline | Shape::Convert) {
        let inode = fs.stat_ino(p).unwrap();
        let entries: Vec<_> = [b"d".as_slice(), b"d\xff", b"full", b"link"]
            .iter()
            .map(|n| {
                let e = fs.find_dir_entry(p, &inode, n).unwrap();
                (e.inode, e.name, e.file_type)
            })
            .collect();
        let entries: Vec<_> = entries
            .iter()
            .map(|(i, n, k)| (*i, n.as_slice(), *k as u8))
            .collect();
        inline_directory(&fs, p, 2, &entries);
    }
    if matches!(shape, Shape::Unknown) {
        unknown_links(&fs, p);
    }
    drop(fs);
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock).unwrap();
    let before = fs.stat_ino(t.ino).unwrap();
    let raw = fs.read_inode_verified(t.ino).unwrap().1;
    let parent = fs.stat_ino(p).unwrap();
    let peer = fs.resolve(b"/peer").unwrap();
    let peer_raw = fs.read_inode_verified(peer).unwrap().1;
    let counts = counters(&dev);
    let gc = group_counters(&fs);
    let inode_bitmap = fs.read_block(fs.groups[0].inode_bitmap).unwrap();
    let ea = fs.read_block(before.file_acl).unwrap();
    let prior_entries = if parent.has_inline_data() {
        fs.read_dir_ino(p).unwrap()
    } else {
        let mut entries = Vec::new();
        for logical in 0..parent.size.div_ceil(u64::from(bs)) {
            let phys = fs.map_inode_logical(&parent, logical).unwrap().unwrap();
            entries.extend(
                crate::dir::parse_block_verified(
                    &fs.read_block(phys).unwrap(),
                    true,
                    p,
                    parent.generation,
                    &fs.csum,
                )
                .unwrap(),
            );
        }
        entries
    };
    let fs = with_test_mount(fs, |h, fs| {
        call(fs, h, p.into(), t, entry, &name).unwrap();
        assert_eq!(fs.lookup_at(p, &name).unwrap(), t.ino);
        source_preserved(fs, t.ino, &raw, before.links_count);
        let now = fs.stat_ino(p).unwrap();
        let extra = now.blocks - parent.blocks;
        assert_eq!(extra % (u64::from(bs) / 512), 0);
        let blocks = extra / (u64::from(bs) / 512);
        assert_eq!(counters(&dev), (counts.0 - blocks, counts.1));
        let g = group_counters(fs);
        assert_eq!(g, (gc.0 - blocks, gc.1, gc.2));
        assert_eq!(now.links_count, parent.links_count);
        assert_eq!(
            fs.read_block(fs.groups[0].inode_bitmap).unwrap(),
            inode_bitmap
        );
        assert_eq!(fs.read_block(before.file_acl).unwrap(), ea);
        assert_eq!(fs.read_inode_verified(peer).unwrap().1, peer_raw);
        for e in &prior_entries {
            assert_eq!(fs.lookup_at(p, &e.name).unwrap(), e.inode);
        }
        if matches!(shape, Shape::Full | Shape::Convert) {
            assert!(now.size > parent.size || !now.has_inline_data());
            assert!(blocks > 0);
        } else {
            assert_eq!(blocks, 0);
        }
        let mut data = [0; 12];
        assert_eq!(fs.read_ino(t, 0, &mut data).unwrap(), 12);
        assert_eq!(&data, b"peer payload");
        eprintln!("link positive {flavor:?}/{bs}/{entry:?}/{shape:?}: existing identity/EA/data/peer/entries/CRC; allocated_blocks={blocks}, no inode/parent-link delta");
    });
    drop(fs);
    let fs = Filesystem::mount(dev).unwrap();
    assert_eq!(fs.lookup_at(p, &name).unwrap(), t.ino);
    assert_eq!(
        fs.stat_ino(t.ino).unwrap().links_count,
        before.links_count + 1
    );
}
#[test]
fn positive_all_five_forms_retain_existing_identity() {
    for f in [
        features::FsFlavor::Ext2,
        features::FsFlavor::Ext3,
        features::FsFlavor::Ext4,
    ] {
        for bs in [1024, 4096] {
            for e in ENTRIES {
                positive(f, bs, e, Shape::Plain);
            }
        }
    }
}
#[test]
fn positive_full_parent_growth() {
    for f in [
        features::FsFlavor::Ext2,
        features::FsFlavor::Ext3,
        features::FsFlavor::Ext4,
    ] {
        for bs in [1024, 4096] {
            for e in [Entry::At, Entry::CAt] {
                positive(f, bs, e, Shape::Full);
            }
        }
    }
}
#[test]
fn positive_inline_and_conversion() {
    for bs in [1024, 4096] {
        for e in [Entry::At, Entry::CAt] {
            for shape in [Shape::Inline, Shape::Convert] {
                positive(features::FsFlavor::Ext4, bs, e, shape);
            }
        }
    }
}
#[test]
fn positive_unknown_parent_links() {
    for bs in [1024, 4096] {
        for e in [Entry::At, Entry::CAt] {
            positive(features::FsFlavor::Ext4, bs, e, Shape::Unknown);
        }
    }
}
#[test]
fn positive_regular_symlinks_special_and_parent_aliases() {
    for bs in [1024, 4096] {
        let (_, fs, p, _) = specimen(features::FsFlavor::Ext4, bs);
        fs.apply_symlink("ascii", "/fast").unwrap();
        fs.apply_symlink(&"x".repeat(1000), "/slow").unwrap();
        fs.apply_symlink("missing", "/dangling").unwrap();
        for (n, path) in ["/ascii", "/fast", "/slow", "/dangling"].iter().enumerate() {
            let t = fs.resolve(path.as_bytes()).unwrap();
            let old = fs.stat_ino(t).unwrap();
            let raw = fs.read_inode_verified(t).unwrap().1;
            fs.apply_link(path, &format!("/p/a{n}")).unwrap();
            source_preserved(&fs, t, &raw, old.links_count);
            assert_eq!(fs.lookup_at(p, format!("a{n}").as_bytes()).unwrap(), t);
            fs.apply_link_at(t, 2, format!("b{n}").as_bytes()).unwrap();
            assert_eq!(fs.stat_ino(t).unwrap().links_count, old.links_count + 2);
        }
        for (n, kind) in [
            crate::inode::S_IFIFO,
            crate::inode::S_IFSOCK,
            crate::inode::S_IFCHR,
            crate::inode::S_IFBLK,
        ]
        .iter()
        .enumerate()
        {
            let t = fs.apply_create(&format!("/special{n}"), 0o640).unwrap();
            let (i, mut raw) = fs.read_inode_verified(t).unwrap();
            raw[..2].copy_from_slice(&(*kind | 0o640).to_le_bytes());
            fs.finalize_inode_raw(t, i.generation, &mut raw).unwrap();
            let mut b = BlockBuffer::new(bs);
            fs.buffer_write_inode(&mut b, t, &raw).unwrap();
            fs.commit_block_buffer(b).unwrap();
            fs.apply_link_at(
                InodeRef::new(t, i.generation),
                p,
                format!("s{n}").as_bytes(),
            )
            .unwrap();
            source_preserved(&fs, t, &raw, i.links_count);
        }
        eprintln!("link positive type/aliases bs={bs}: regular/fast/slow/dangling and four supported special inode identities");
    }
}
#[test]
fn positive_source_trailing_slash_and_destination_path_contracts() {
    let (_, fs, p, _) = specimen(features::FsFlavor::Ext4, 4096);
    fs.apply_symlink("p", "/parent-link").unwrap();
    fs.apply_symlink("ascii", "/source-link").unwrap();
    let t = target(&fs);
    let link = fs.resolve(b"/source-link").unwrap();
    assert_ne!(link, t.ino);
    fs.apply_link("/source-link", "/parent-link/Link///")
        .unwrap();
    assert_eq!(fs.lookup_at(p, b"Link").unwrap(), link);
    fs.apply_link("/ascii", "p/link-lower").unwrap();
    assert_eq!(fs.lookup_at(p, b"link-lower").unwrap(), t.ino);
    assert!(matches!(
        fs.apply_link("/source-link/", "/p/refused"),
        Err(Error::NotADirectory)
    ));
    assert!(matches!(
        fs.apply_link("/parent-link/", "/p/refused"),
        Err(Error::IsADirectory)
    ));
    assert!(matches!(
        fs.apply_link("/missing", "bare"),
        Err(Error::InvalidArgument(_))
    ));
    assert!(matches!(
        fs.apply_link_bytes(b"/missing", &[b"/p/".as_slice(), &[b'x'; 256]].concat()),
        Err(Error::NameTooLong)
    ));
}
#[test]
fn positive_real_same_table_source_parent_slices() {
    for bs in [1024, 4096] {
        let (_, fs, _, _) = specimen(features::FsFlavor::Ext4, bs);
        let mut pair = None;
        for n in 0..8 {
            let p = fs.apply_mkdir(&format!("/share{n}"), 0o750).unwrap();
            let t = fs
                .apply_create(&format!("/share{n}/source"), 0o640)
                .unwrap();
            if crate::bgd::locate_inode(&fs.sb, &fs.groups, p).unwrap().0
                == crate::bgd::locate_inode(&fs.sb, &fs.groups, t).unwrap().0
            {
                pair = Some((p, t));
                break;
            }
        }
        let (p, t) = pair.expect("actual shared-table fixture");
        full_parent(&fs, p, &[b'x'; 255]);
        let old = fs.stat_ino(t).unwrap();
        let raw = fs.read_inode_verified(t).unwrap().1;
        let parent = fs.stat_ino(p).unwrap();
        fs.apply_link_at(t, p, &[b'x'; 255]).unwrap();
        source_preserved(&fs, t, &raw, old.links_count);
        let now = fs.stat_ino(p).unwrap();
        assert!(now.size > parent.size);
        assert_eq!(now.links_count, parent.links_count);
        assert_eq!(fs.lookup_at(p, &[b'x'; 255]).unwrap(), t);
        eprintln!(
            "link positive shared-table bs={bs}: source count and growing parent slices survive"
        );
    }
}
#[test]
fn positive_legacy_single_indirect_and_extent_parent_growth() {
    for f in [features::FsFlavor::Ext2, features::FsFlavor::Ext4] {
        for bs in [1024, 4096] {
            let (dev, fs, p, _) = specimen(f, bs);
            let t = target(&fs);
            let counts = counters(&dev);
            let old = fs.stat_ino(p).unwrap();
            let mut names = Vec::new();
            for n in 0..260 {
                let name = format!("{n:04}-{}", "x".repeat(250));
                fs.apply_link_at(t, p, name.as_bytes()).unwrap();
                names.push(name);
                if fs.stat_ino(p).unwrap().size > 12 * u64::from(bs) {
                    break;
                }
            }
            let now = fs.stat_ino(p).unwrap();
            assert!(now.size > 12 * u64::from(bs));
            if f == features::FsFlavor::Ext2 {
                assert_ne!(u32::from_le_bytes(now.block[48..52].try_into().unwrap()), 0);
            } else {
                assert!(now.has_extents());
            }
            assert_eq!(now.links_count, old.links_count);
            assert_eq!(
                fs.stat_ino(t.ino).unwrap().links_count,
                1 + names.len() as u16
            );
            assert_eq!(counters(&dev).1, counts.1);
            assert_eq!(
                counts.0 - counters(&dev).0,
                (now.blocks - old.blocks) / (u64::from(bs) / 512)
            );
            for n in names {
                assert_eq!(fs.lookup_at(p, n.as_bytes()).unwrap(), t.ino);
            }
            eprintln!("link positive mapping {f:?}/{bs}: actual >12 data blocks and exact data/map block accounting");
        }
    }
}
#[test]
fn ffi_pointer_ceiling_and_volume_priorities() {
    for profile in 0..6 {
        let (dev, mut fs, p, _) = specimen(features::FsFlavor::Ext4, 4096);
        let t = target(&fs);
        match profile {
            0 => dev.writable.store(false, Ordering::SeqCst),
            1 => fs.replay_pending.store(true, Ordering::SeqCst),
            2 => fs.operation_failed.store(true, Ordering::SeqCst),
            3 => fs.direct_commit_failed.store(true, Ordering::SeqCst),
            4 => fs.sb.feature_incompat |= features::Incompat::CASEFOLD.bits(),
            _ => fs.sb.feature_ro_compat |= features::RoCompat::QUOTA.bits(),
        }
        assert!(match profile {
            1 => matches!(fs.apply_link_at(t, p, b""), Err(Error::JournalNotReplayed)),
            4 => matches!(
                fs.apply_link_at(t, p, b""),
                Err(Error::UnsupportedIncompat(_))
            ),
            5 => matches!(
                fs.apply_link_at(t, p, b""),
                Err(Error::UnsupportedRoCompat(_))
            ),
            _ => matches!(fs.apply_link_at(t, p, b""), Err(Error::ReadOnly)),
        });
        let _fs = with_test_mount(fs, |h, _| {
            dev.reset();
            for (name, len, e) in [
                (std::ptr::null(), 0, crate::error::errno::EINVAL),
                (
                    b"x".as_ptr().cast(),
                    usize::MAX,
                    crate::error::errno::ENAMETOOLONG,
                ),
            ] {
                assert_eq!(
                    unsafe {
                        fs_ext4_link_at(
                            h,
                            t.ino,
                            FS_EXT4_GEN_ANY,
                            p,
                            FS_EXT4_GEN_ANY,
                            name,
                            len,
                            std::ptr::null_mut(),
                        )
                    },
                    -1
                );
                assert_eq!(fs_ext4_last_errno(), e);
            }
            assert_eq!(
                unsafe { fs_ext4_link(h, std::ptr::null(), std::ptr::null()) },
                -1
            );
            assert_eq!(fs_ext4_last_errno(), crate::error::errno::EINVAL);
            assert_eq!(dev.counts(), [0; 4]);
        });
    }
}
#[test]
fn positive_refusal_reuse_and_max_source_count_boundary() {
    for dirty in [false, true] {
        let (dev, fs, p, _) = specimen(features::FsFlavor::Ext4, 4096);
        let t = target(&fs);
        if dirty {
            fs.apply_chmod("/peer", 0o600).unwrap();
        }
        for name in [b"".as_slice(), b"bad/name", b"bad\0name", &[b'x'; 256]] {
            assert!(fs.apply_link_at(t, p, name).is_err());
        }
        assert!(!fs.operation_failed.load(Ordering::SeqCst));
        patch_links(&fs, t.ino, 64999);
        fs.apply_link_at(t, p, b"last").unwrap();
        assert_eq!(fs.stat_ino(t.ino).unwrap().links_count, 65000);
        assert!(matches!(
            fs.apply_link_at(t, p, b"over"),
            Err(Error::TooManyLinks)
        ));
        drop(fs);
        assert_ne!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
    }
}

#[test]
fn real_inplace_staging_has_no_superblock_and_no_allocation() {
    for bs in [1024, 4096] {
        let (dev, fs, p, _) = specimen(features::FsFlavor::Ext4, bs);
        let t = target(&fs);
        let (src, mut raw) = fs.live_inode(t).unwrap();
        let parent = fs.live_dir(p.into()).unwrap().0;
        let mut buf = BlockBuffer::new(bs);
        fs.patch_inode_nlink(t.ino, &mut raw, &src, 1).unwrap();
        fs.buffer_write_inode(&mut buf, t.ino, &raw).unwrap();
        fs.buffer_add_dir_entry_inplace(
            &mut buf,
            p,
            &parent,
            b"new",
            t.ino,
            crate::dir::DirEntryType::RegFile,
        )
        .unwrap();
        assert!(!buf.dirty.contains_key(&(1024 / u64::from(bs))));
        assert!(buf.uninit_cleared.is_empty());
        let counts = counters(&dev);
        fs.apply_link_at(t, p, b"new").unwrap();
        assert_eq!(counters(&dev), counts);
        eprintln!(
            "link real allocation-free staging bs={bs}: no staged SB; public call retains counters"
        );
    }
}
#[test]
fn positive_ordinary_index_insertion_and_actual_leaf_growth() {
    for bs in [1024, 4096] {
        let (dev, fs, _, _) = specimen(features::FsFlavor::Ext4, bs);
        let t = target(&fs);
        let p = fs.apply_mkdir("/indexed", 0o750).unwrap();
        let parent = fs.stat_ino(p).unwrap();
        let phys = fs.map_inode_logical(&parent, 0).unwrap().unwrap();
        let mut leaf = vec![0; bs as usize];
        leaf[4..6].copy_from_slice(&((bs - 12) as u16).to_le_bytes());
        assert!(fs
            .csum
            .patch_dir_entry_tail(p, parent.generation, &mut leaf));
        let mut buf = BlockBuffer::new(bs);
        fs.buffer_append_dir_block(&mut buf, p, leaf).unwrap();
        let (_, mut raw) = fs.buffered_inode_verified(&buf, p).unwrap();
        raw[32..36].copy_from_slice(
            &(parent.flags | crate::inode::InodeFlags::INDEX.bits()).to_le_bytes(),
        );
        fs.finalize_inode_raw(p, parent.generation, &mut raw)
            .unwrap();
        fs.buffer_write_inode(&mut buf, p, &raw).unwrap();
        let mut root = fs.read_block(phys).unwrap();
        root[16..18].copy_from_slice(&((bs - 12) as u16).to_le_bytes());
        root[24..].fill(0);
        root[28] = 1;
        root[29] = 8;
        root[32..34].copy_from_slice(&(((bs - 32 - 8) / 8) as u16).to_le_bytes());
        root[34..36].copy_from_slice(&1u16.to_le_bytes());
        root[36..40].copy_from_slice(&1u32.to_le_bytes());
        assert!(fs.csum.patch_dx_tail(p, parent.generation, &mut root, 32));
        buf.put(phys, root);
        fs.commit_block_buffer(buf).unwrap();
        let old = fs.stat_ino(p).unwrap();
        let before = counters(&dev);
        let mut names = Vec::new();
        for n in 0..64 {
            let name = format!("{n:04}-{}", "i".repeat(250));
            fs.apply_link_at(t, p, name.as_bytes()).unwrap();
            names.push(name);
            let now = fs.stat_ino(p).unwrap();
            if now.size > old.size {
                break;
            }
        }
        let now = fs.stat_ino(p).unwrap();
        assert!(
            now.size > old.size,
            "must execute leaf split or supported index fallback"
        );
        for name in &names {
            assert_eq!(fs.lookup_at(p, name.as_bytes()).unwrap(), t.ino);
        }
        assert_eq!(
            fs.stat_ino(t.ino).unwrap().links_count,
            1 + names.len() as u16
        );
        assert_eq!(now.links_count, old.links_count);
        assert_eq!(
            counters(&dev),
            (
                before.0 - (now.blocks - old.blocks) / (u64::from(bs) / 512),
                before.1
            )
        );
        eprintln!(
            "link positive indexed bs={bs}: actual leaf growth; index_retained={}",
            now.flags & crate::inode::InodeFlags::INDEX.bits() != 0
        );
    }
}
#[test]
fn positive_uninit_group_growth_changes_only_block_allocation() {
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
    let t = fs.apply_create("/source", 0o640).unwrap();
    full_parent(&fs, 2, &[b'u'; 255]);
    let mut buf = BlockBuffer::new(bs);
    let (b, o) = fs.sb.descriptor_location(0);
    let bytes = fs.read_block(b).unwrap();
    let g0 = crate::bgd::BlockGroupDescriptor::parse(
        &bytes[o..o + fs.sb.desc_size as usize],
        fs.sb.desc_size,
    )
    .unwrap();
    buf.put(fs.groups[0].block_bitmap, vec![0xff; bs as usize]);
    fs.buffer_refresh_bitmap_csum(&mut buf, 0, false).unwrap();
    fs.buffer_patch_bgd_counters(&mut buf, 0, -(g0.free_blocks_count as i32), 0, 0)
        .unwrap();
    fs.buffer_patch_sb_counters(&mut buf, -(i64::from(g0.free_blocks_count)), 0)
        .unwrap();
    let (b, o) = fs.sb.descriptor_location(1);
    let bytes = buf.get_mut(&fs, b).unwrap();
    let flags = u16::from_le_bytes(bytes[o + 18..o + 20].try_into().unwrap())
        | crate::bgd::BgdFlags::BLOCK_UNINIT.bits()
        | crate::bgd::BgdFlags::INODE_UNINIT.bits();
    bytes[o + 18..o + 20].copy_from_slice(&flags.to_le_bytes());
    fs.restamp_group_desc_csum(bytes, o, 1);
    buf.put(fs.groups[1].block_bitmap, vec![0xa5; bs as usize]);
    fs.commit_block_buffer(buf).unwrap();
    drop(fs);
    let fs = Filesystem::mount(dev.clone()).unwrap();
    let before = counters(&dev);
    let inode_maps: Vec<_> = fs
        .groups
        .iter()
        .map(|g| fs.read_block(g.inode_bitmap).unwrap())
        .collect();
    let desc = fs.read_block(b).unwrap();
    fs.apply_link_at(t, 2, &[b'u'; 255]).unwrap();
    fs.apply_link_at(t, 2, &[b'v'; 255]).unwrap();
    let root = fs.stat_ino(2).unwrap();
    let phys = fs.map_inode_logical(&root, 1).unwrap().unwrap();
    assert_eq!(
        (phys - u64::from(fs.sb.first_data_block)) / u64::from(fs.sb.blocks_per_group),
        1
    );
    assert!(allocated(&fs, phys));
    assert_eq!(counters(&dev), (before.0 - 1, before.1));
    assert!(!fs.allocation_groups()[1]
        .flags()
        .contains(crate::bgd::BgdFlags::BLOCK_UNINIT));
    assert!(fs.allocation_groups()[1]
        .flags()
        .contains(crate::bgd::BgdFlags::INODE_UNINIT));
    for (gi, g) in fs.groups.iter().enumerate() {
        assert_eq!(fs.read_block(g.inode_bitmap).unwrap(), inode_maps[gi]);
    }
    let now = fs.read_block(b).unwrap();
    for at in [14, 16, 28, 46, 48, 50] {
        assert_eq!(&desc[o + at..o + at + 2], &now[o + at..o + at + 2]);
    }
    let descriptor = crate::bgd::BlockGroupDescriptor::parse(
        &now[o..o + fs.sb.desc_size as usize],
        fs.sb.desc_size,
    )
    .unwrap();
    let bm = fs.read_block(fs.groups[1].block_bitmap).unwrap();
    assert_eq!(
        descriptor.block_bitmap_csum,
        fs.csum.crc(&bm[..fs.sb.blocks_per_group as usize / 8])
    );
    assert_eq!(fs.stat_ino(t).unwrap().links_count, 3);
    drop(fs);
    let fs = Filesystem::mount(dev).unwrap();
    assert_eq!(fs.lookup_at(2, &[b'v'; 255]).unwrap(), t);
    eprintln!("link positive 2group/uninit: one actual parent block; no inode bitmap/count/itable-unused/used-dir changes, pending allocation publication and reopen retained");
}

#[test]
fn positive_all_entry_forms_succeed_after_context_release() {
    for entry in ENTRIES {
        let (_, fs, p, _) = specimen(features::FsFlavor::Ext4, 4096);
        let t = target(&fs);
        let _fs = with_test_mount(fs, |h, fs| {
            let held = fs.chmod_operation().unwrap();
            drop(held);
            call(fs, h, p.into(), t, entry, b"released").unwrap();
            assert_eq!(fs.lookup_at(p, b"released").unwrap(), t.ino);
        });
    }
}
#[test]
fn ffi_null_ceiling_and_bounded_paths_precede_held_context() {
    let (dev, fs, p, _) = specimen(features::FsFlavor::Ext4, 4096);
    let t = target(&fs);
    let _fs = with_test_mount(fs, |h, fs| {
        let held = fs.chmod_operation().unwrap();
        dev.reset();
        for (name, len, e) in [
            (std::ptr::null(), 0, crate::error::errno::EINVAL),
            (
                b"x".as_ptr().cast(),
                usize::MAX,
                crate::error::errno::ENAMETOOLONG,
            ),
        ] {
            assert_eq!(
                unsafe {
                    fs_ext4_link_at(
                        h,
                        t.ino,
                        FS_EXT4_GEN_ANY,
                        p,
                        FS_EXT4_GEN_ANY,
                        name,
                        len,
                        std::ptr::null_mut(),
                    )
                },
                -1
            );
            assert_eq!(fs_ext4_last_errno(), e);
        }
        let long = [b'x'; 4097];
        let valid = CString::new("/ascii").unwrap();
        assert_eq!(
            unsafe { fs_ext4_link(h, long.as_ptr().cast(), valid.as_ptr()) },
            -1
        );
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::ENAMETOOLONG);
        assert_eq!(
            unsafe { fs_ext4_link(h, valid.as_ptr(), long.as_ptr().cast()) },
            -1
        );
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::ENAMETOOLONG);
        assert_eq!(dev.counts(), [0; 4]);
        drop(held);
    });
}

#[test]
fn positive_raw_source_and_destination_bytes_keep_identity() {
    let (_, fs, p, _) = specimen(features::FsFlavor::Ext4, 4096);
    let ino = fs.apply_create_bytes(b"/source\xff", 0o640).unwrap();
    fs.apply_link_bytes(b"/source\xff", b"/p/dest\xfe").unwrap();
    assert_eq!(fs.lookup_at(p, b"dest\xfe").unwrap(), ino);
    let _fs = with_test_mount(fs, |h, fs| {
        let src = CString::new(b"/source\xff".as_slice()).unwrap();
        let dst = CString::new(b"/p/dest\xfd".as_slice()).unwrap();
        assert_eq!(unsafe { fs_ext4_link(h, src.as_ptr(), dst.as_ptr()) }, 0);
        assert_eq!(fs.lookup_at(p, b"dest\xfd").unwrap(), ino);
        assert_eq!(fs.stat_ino(ino).unwrap().links_count, 3);
    });
}

#[test]
fn positive_fragmented_extent_parent_promotes_and_appends_at_depth_one() {
    for bs in [1024, 4096] {
        let (dev, fs, p, _) = specimen(features::FsFlavor::Ext4, bs);
        let t = target(&fs);
        let original = fs.read_inode_verified(t.ino).unwrap().1;
        let mut growths = 0;
        let mut promoted = false;
        let mut names = Vec::new();
        // Real held files separate parent allocations; no invented block owners.
        fs.apply_create("/spacer-first", 0o640).unwrap();
        fs.apply_replace_file_content("/spacer-first", &vec![1; bs as usize])
            .unwrap();
        for n in 0..320 {
            let name = format!("{n:04}-{}", "f".repeat(250));
            let old = fs.stat_ino(p).unwrap();
            let counts = counters(&dev);
            fs.apply_link_at(t, p, name.as_bytes()).unwrap();
            names.push(name);
            let now = fs.stat_ino(p).unwrap();
            assert_eq!(
                counters(&dev),
                (
                    counts.0 - (now.blocks - old.blocks) / (u64::from(bs) / 512),
                    counts.1
                )
            );
            if now.size > old.size {
                growths += 1;
                let depth = crate::extent::ExtentHeader::parse(&now.block)
                    .unwrap()
                    .depth;
                if depth == 1 && promoted {
                    break;
                }
                promoted |= depth == 1;
                let spacer = format!("/spacer-{growths}");
                fs.apply_create(&spacer, 0o640).unwrap();
                fs.apply_replace_file_content(&spacer, &vec![1; bs as usize])
                    .unwrap();
            }
        }
        assert!(promoted, "must reach actual fragmented depth-one mapping");
        assert!(growths >= 5);
        let now = fs.stat_ino(p).unwrap();
        assert_eq!(
            crate::extent::ExtentHeader::parse(&now.block)
                .unwrap()
                .depth,
            1
        );
        source_preserved(&fs, t.ino, &original, names.len() as u16);
        for name in &names {
            assert_eq!(fs.lookup_at(p, name.as_bytes()).unwrap(), t.ino);
        }
        eprintln!("link positive fragmented extents bs={bs}: {growths} actual growths; depth-one promotion and later append, inode/data/EA and per-link accounting retained");
    }
}

// These exercise the real link-only staging/commit boundary after promotion.
#[test]
fn real_link_plan_omits_superblock_without_growth() {
    for bs in [1024, 4096] {
        let (dev, fs, parent, _) = specimen(features::FsFlavor::Ext4, bs);
        let t = target(&fs);
        let before = counters(&dev);
        let mut context = fs.chmod_operation().unwrap();
        dev.reset();
        let plan = fs.plan_link_at(t, parent, b"planned").unwrap();
        assert!(!plan.buf.dirty.contains_key(&(1024 / u64::from(bs))));
        assert_eq!(&dev.counts()[1..3], &[0, 0]);
        assert_eq!(fs.commit_link_in(&mut context, plan).unwrap(), t.ino);
        assert_eq!(counters(&dev), before);
        assert_eq!(fs.lookup_at(parent, b"planned").unwrap(), t.ino);
        assert_eq!(context.operation.status().epoch, 2);
        assert_eq!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
    }
}

#[test]
fn real_link_growth_plan_keeps_superblock_debit() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            let (dev, fs, parent, _) = specimen(flavor, bs);
            let name = vec![b'n'; 255];
            full_parent(&fs, parent, &name);
            let t = target(&fs);
            let before = counters(&dev);
            let mut context = fs.chmod_operation().unwrap();
            dev.reset();
            let plan = fs.plan_link_at(t, parent, &name).unwrap();
            assert!(plan.buf.dirty.contains_key(&(1024 / u64::from(bs))));
            assert_eq!(&dev.counts()[1..3], &[0, 0]);
            fs.commit_link_in(&mut context, plan).unwrap();
            assert_eq!(counters(&dev), (before.0 - 1, before.1));
            assert_eq!(fs.lookup_at(parent, &name).unwrap(), t.ino);
            assert_eq!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
        }
    }
}

#[test]
fn real_inline_link_plan_distinguishes_fit_and_conversion() {
    for bs in [1024, 4096] {
        for grow in [false, true] {
            let (dev, fs, parent, clock) = specimen(features::FsFlavor::Ext4, bs);
            inline_directory(&fs, parent, 2, &[]);
            drop(fs);
            let fs = Filesystem::mount_with_runtime(dev.clone(), clock).unwrap();
            let t = target(&fs);
            let name = vec![b'n'; if grow { 255 } else { 3 }];
            let before = counters(&dev);
            let mut context = fs.chmod_operation().unwrap();
            dev.reset();
            let plan = fs.plan_link_at(t, parent, &name).unwrap();
            assert_eq!(plan.buf.dirty.contains_key(&(1024 / u64::from(bs))), grow);
            assert_eq!(&dev.counts()[1..3], &[0, 0]);
            fs.commit_link_in(&mut context, plan).unwrap();
            assert_eq!(counters(&dev), (before.0 - u64::from(grow), before.1));
            assert_eq!(fs.lookup_at(parent, &name).unwrap(), t.ino);
            assert_eq!(fs.stat_ino(parent).unwrap().has_inline_data(), !grow);
        }
    }
}

fn constructed_link(flavor: features::FsFlavor, bs: u32) {
    let (dev, fs, _, _) = specimen(flavor, bs);
    let mut context = fs.chmod_operation().unwrap();
    let before = counters(&dev);
    let mut buf = BlockBuffer::new(bs);
    fs.buffer_patch_sb_counters(&mut buf, -1, 0).unwrap();
    let block = buf.get_mut(&fs, 1024 / u64::from(bs)).unwrap();
    let start = (1024 % u64::from(bs)) as usize;
    let sb = &mut block[start..start + 1024];
    let expected = u16::from_le_bytes(sb[58..60].try_into().unwrap()) | 0x200;
    sb[58..60].copy_from_slice(&expected.to_le_bytes());
    if fs.csum.enabled {
        let sum = crate::checksum::linux_crc32c(!0, &sb[..1020]);
        sb[1020..].copy_from_slice(&sum.to_le_bytes());
    }
    if bs == 4096 {
        block[0] = 0xa5;
        block[2048] = 0x5a;
    }
    fs.commit_link_in(&mut context, LinkPlan { target: 0, buf })
        .unwrap();
    assert_eq!(counters(&dev), (before.0 - 1, before.1));
    assert_eq!(state(&dev), expected & !crate::superblock::EXT4_VALID_FS);
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
}
#[test]
fn link_boundary_direct_1k_keeps_dirty_state() {
    constructed_link(features::FsFlavor::Ext4, 1024);
}
#[test]
fn link_boundary_direct_4k_keeps_dirty_state() {
    constructed_link(features::FsFlavor::Ext4, 4096);
}
#[test]
fn link_boundary_journal_1k_keeps_dirty_state() {
    constructed_link(features::FsFlavor::Ext3, 1024);
}
#[test]
fn link_boundary_journal_4k_keeps_dirty_state() {
    constructed_link(features::FsFlavor::Ext3, 4096);
}

#[test]
fn link_foreign_context_and_malformed_present_superblocks_refuse_without_callbacks() {
    for bs in [1024, 4096] {
        let (_, first, _, _) = specimen(features::FsFlavor::Ext4, bs);
        let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, bs);
        let mut foreign = first.chmod_operation().unwrap();
        assert!(matches!(
            fs.link_bytes_in(&mut foreign, b"/ascii", b"/p/new"),
            Err(Error::Stale)
        ));
        assert!(matches!(
            fs.link_from_in(&mut foreign, 2u32, 2u32, b"new"),
            Err(Error::Stale)
        ));
        assert!(matches!(
            fs.commit_link_in(
                &mut foreign,
                LinkPlan {
                    target: 0,
                    buf: BlockBuffer::new(bs)
                }
            ),
            Err(Error::Stale)
        ));
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        let mut context = fs.chmod_operation().unwrap();
        let status = context.operation.status();
        let before = dev.bytes();
        for bytes in [
            vec![],
            vec![0; 1023],
            vec![0; 1025],
            vec![0; bs as usize],
            vec![0; bs as usize + 1],
        ] {
            let mut buf = BlockBuffer::new(bs);
            buf.put(1024 / u64::from(bs), bytes);
            assert!(matches!(
                fs.commit_link_in(&mut context, LinkPlan { target: 0, buf }),
                Err(Error::Corrupt(_))
            ));
            assert_eq!(context.operation.status(), status);
        }
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert_eq!(dev.bytes(), before);
        assert!(!fs.operation_failed.load(Ordering::SeqCst));
    }
}

#[test]
fn link_boundary_effect_errors_and_flush_unwinds_latch_with_retained_context() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for dirty in [false, true] {
            for unwind in [false, true] {
                let (dev, fs, p, t) = fault_specimen(flavor, 4096, dirty);
                let mut context = fs.chmod_operation().unwrap();
                let plan = fs.plan_link_at(t, p, b"new").unwrap();
                dev.reset();
                if unwind {
                    dev.panic_flush.store(1, Ordering::SeqCst);
                } else {
                    dev.fail_write.store(1, Ordering::SeqCst);
                }
                let got = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    fs.commit_link_in(&mut context, plan)
                }));
                assert!(if unwind {
                    got.is_err()
                } else {
                    got.unwrap().is_err()
                });
                assert_eq!(
                    context.operation.status().health,
                    if unwind {
                        Health::Uncertain
                    } else {
                        Health::Faulted
                    }
                );
                assert!(fs.operation_failed.load(Ordering::SeqCst));
                dev.reset();
                for family in 0..10 {
                    assert!(matches!(metadata(&fs, family), Err(Error::ReadOnly)));
                }
                assert!(matches!(
                    fs.apply_link("/ascii", "/p/again"),
                    Err(Error::ReadOnly)
                ));
                assert!(fs.flush().is_err());
                assert_eq!(dev.counts(), [0; 4]);
                drop(context);
                assert!(retired_observation(&dev, fs));
            }
        }
    }
}

#[test]
fn real_link_publication_unwind_latches_before_retained_context_drops() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        let (dev, fs, p, _) = specimen(flavor, 4096);
        let t = target(&fs);
        let mut context = fs.chmod_operation().unwrap();
        let plan = fs.plan_link_at(t, p, b"new").unwrap();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _held = fs.uninit_cleared.lock().unwrap();
            panic!("link publication poison after complete planning");
        }))
        .is_err());
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || fs.commit_link_in(&mut context, plan)
        ))
        .is_err());
        assert_eq!(
            fs.find_entry_in_dir(p, &fs.read_inode_verified(p).unwrap().0, b"new")
                .unwrap(),
            t.ino
        );
        assert_eq!(context.operation.status().health, Health::Uncertain);
        assert!(fs.operation_failed.load(Ordering::SeqCst));
        dev.reset();
        assert!(matches!(
            fs.apply_link("/ascii", "/p/again"),
            Err(Error::ReadOnly)
        ));
        assert_eq!(dev.counts(), [0; 4]);
        drop(context);
        assert!(retired_observation(&dev, fs));
    }
}

#[test]
fn actual_link_runtime_excludes_eleven_families_same_thread_and_worker() {
    for same in [false, true] {
        let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
        let fs = Arc::new(fs);
        let weak = Arc::downgrade(&fs);
        let out = Arc::new(Mutex::new(Vec::new()));
        let observed = out.clone();
        *clock.hook.lock().unwrap() = Some(Arc::new(move || {
            let fs = weak.upgrade().unwrap();
            for family in 0..=10 {
                let before = dev.counts();
                let run = || {
                    if family == 10 {
                        fs.apply_link("/ascii", "/p/nested")
                    } else {
                        metadata(&fs, family)
                    }
                };
                let got = if same {
                    run()
                } else {
                    std::thread::scope(|scope| scope.spawn(run).join().unwrap())
                };
                observed
                    .lock()
                    .unwrap()
                    .push(matches!(got, Err(Error::Unsupported(_))) && dev.counts() == before);
            }
        }));
        fs.apply_link("/ascii", "/p/new").unwrap();
        *clock.hook.lock().unwrap() = None;
        assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
        assert_eq!(*out.lock().unwrap(), vec![true; 11]);
    }
}

#[test]
fn actual_link_runtime_unwind_retires_rust_and_c_before_marker() {
    for c in [false, true] {
        let (dev, fs, p, clock) = specimen(features::FsFlavor::Ext4, 4096);
        let t = target(&fs);
        let before = dev.bytes();
        dev.reset();
        *clock.hook.lock().unwrap() = Some(Arc::new(|| panic!("link planning Runtime unwind")));
        let fs = if c {
            with_test_mount(fs, |h, fs| {
                assert_eq!(
                    call(fs, h, p.into(), t, Entry::CAt, b"new"),
                    Err(crate::error::errno::EIO)
                )
            })
        } else {
            assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || fs.apply_link_at(t, p, b"new")
            ))
            .is_err());
            fs
        };
        *clock.hook.lock().unwrap() = None;
        assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
        assert_eq!(dev.bytes(), before);
        assert_eq!(&dev.counts()[1..3], &[0, 0]);
        assert!(retired_observation(&dev, fs));
    }
}

#[test]
fn all_eleven_runtime_callbacks_exclude_link_on_same_thread() {
    for family in 0..=10 {
        let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
        let fs = Arc::new(fs);
        let weak = Arc::downgrade(&fs);
        let out = Arc::new(AtomicUsize::new(0));
        let observed = out.clone();
        *clock.hook.lock().unwrap() = Some(Arc::new(move || {
            let fs = weak.upgrade().unwrap();
            let before = dev.counts();
            assert!(matches!(
                fs.apply_link("/ascii", "/p/nested"),
                Err(Error::Unsupported(_))
            ));
            assert_eq!(dev.counts(), before);
            observed.fetch_add(1, Ordering::SeqCst);
        }));
        if family == 10 {
            fs.apply_link("/ascii", "/p/new").unwrap();
        } else {
            metadata(&fs, family).unwrap();
        }
        *clock.hook.lock().unwrap() = None;
        let calls = clock.calls.load(Ordering::SeqCst);
        assert!(calls > 0, "family {family} must actually reach Runtime");
        assert_eq!(out.load(Ordering::SeqCst), calls);
    }
}
