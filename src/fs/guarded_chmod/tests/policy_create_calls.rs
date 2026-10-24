//! Native expectations measured independently by create_policy's VM oracle.
//! Executed before repair; independent Linux emission qualification remains required.
//! Counted Into-before-name, rooted-path and low-twelve-mode-bit API contracts stay explicit.
use super::*;
use crate::capi::*;
use crate::inode::Inode;
use std::ffi::CString;

const INITIAL: i64 = 1_700_000_000;
const FIRST: i64 = INITIAL + 100;
const SECOND: i64 = INITIAL + 200;
const NAME: &str = "user.audit";
const DATA: &[u8] = b"target payload";

#[derive(Clone, Copy, Debug)]
enum Entry {
    RustString,
    RustBytes,
    RustAt,
    CPath,
    CAt,
}
const ENTRIES: [Entry; 5] = [
    Entry::RustString,
    Entry::RustBytes,
    Entry::RustAt,
    Entry::CPath,
    Entry::CAt,
];

fn specimen(
    flavor: features::FsFlavor,
    bs: u32,
) -> (Arc<Device>, Filesystem, u32, u32, Arc<Clock>) {
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
        Some([41; 16]),
        size as u64,
        bs,
        flavor,
    )
    .unwrap();
    let clock = Arc::new(Clock {
        calls: AtomicUsize::new(0),
        now: std::sync::atomic::AtomicI64::new(INITIAL),
        hook: Mutex::new(None),
    });
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
    let parent = fs.apply_mkdir("/p", 0o750).unwrap();
    let ino = fs.apply_mkdir("/p/target", 0o750).unwrap();
    fs.apply_mkdir("/p/second", 0o750).unwrap();
    fs.apply_mkdir("/p/nonempty", 0o750).unwrap();
    fs.apply_create("/p/nonempty/kept", 0o640).unwrap();
    fs.apply_create("/p/regular", 0o640).unwrap();
    fs.apply_replace_file_content("/p/regular", DATA).unwrap();
    fs.apply_setxattr("/p/regular", NAME, &[b'v'; 600]).unwrap();
    fs.apply_symlink("target", "/p/link").unwrap();
    fs.apply_symlink("regular", "/p/link-file").unwrap();
    fs.apply_symlink("absent", "/p/dangling").unwrap();
    fs.apply_symlink("p", "/parent-link").unwrap();
    fs.apply_setxattr("/p", NAME, &[b'a'; 512]).unwrap();
    for path in ["/p/target", "/p/second", "/p"] {
        fs.apply_utimens(path, 1_600_000_000, 0, 1_600_000_001, 0)
            .unwrap();
    }
    drop(fs);
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
    assert_eq!(fs.sb.block_size(), bs);
    assert_eq!(
        fs.sb.inode_size,
        if flavor == features::FsFlavor::Ext4 {
            256
        } else {
            128
        }
    );
    assert_eq!(fs.csum.enabled, flavor == features::FsFlavor::Ext4);
    assert_eq!(fs.journal.is_some(), flavor == features::FsFlavor::Ext3);
    assert_eq!(fs.read_inode_verified(ino).unwrap().0.links_count, 2);
    dev.reset();
    clock.calls.store(0, Ordering::SeqCst);
    (dev, fs, parent, ino, clock)
}

fn call(
    fs: &Filesystem,
    handle: *mut fs_ext4_fs_t,
    parent: InodeRef,
    entry: Entry,
    name: &[u8],
    mode: u16,
) -> std::result::Result<u32, i32> {
    call_path(fs, handle, parent, entry, name, mode, None)
}

fn call_path(
    fs: &Filesystem,
    handle: *mut fs_ext4_fs_t,
    parent: InodeRef,
    entry: Entry,
    name: &[u8],
    mode: u16,
    override_path: Option<&[u8]>,
) -> std::result::Result<u32, i32> {
    let default_path = [b"/p/".as_slice(), name].concat();
    let path = override_path.unwrap_or(&default_path);
    match entry {
        Entry::RustString => fs
            .apply_create(std::str::from_utf8(path).unwrap(), mode)
            .map_err(|e| e.to_errno()),
        Entry::RustBytes => fs.apply_create_bytes(path, mode).map_err(|e| e.to_errno()),
        Entry::RustAt => fs
            .apply_create_at(parent, name, mode)
            .map_err(|e| e.to_errno()),
        Entry::CPath => {
            let path = CString::new(path).unwrap();
            let ino = unsafe { fs_ext4_create(handle, path.as_ptr(), mode) };
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
                fs_ext4_create_at(
                    handle,
                    parent.ino,
                    parent.generation.unwrap_or(FS_EXT4_GEN_ANY),
                    name.as_ptr().cast(),
                    name.len(),
                    mode,
                    &mut attr,
                )
            };
            if rc == -1 {
                Err(fs_ext4_last_errno())
            } else {
                assert_eq!(rc, 0);
                assert_eq!(fs_ext4_last_errno(), 0);
                assert_eq!(attr.mode, mode & 0xfff);
                assert_eq!(attr.link_count, 1);
                assert_eq!((attr.size, attr.blocks_512), (0, 0));
                Ok(attr.inode)
            }
        }
    }
}

fn times(inode: &Inode) -> [(i64, u32); 3] {
    [
        (inode.atime, inode.atime_nsec),
        (inode.mtime, inode.mtime_nsec),
        (inode.ctime, inode.ctime_nsec),
    ]
}

fn counters(dev: &Device) -> (u64, u32) {
    let b = dev.bytes();
    (
        u64::from(u32::from_le_bytes(b[1036..1040].try_into().unwrap()))
            | (u64::from(u32::from_le_bytes(b[1368..1372].try_into().unwrap())) << 32),
        u32::from_le_bytes(b[1040..1044].try_into().unwrap()),
    )
}

// The separate safety patch diagnoses premature dirty-marker writes. These
// controls compare allocated inode/namespace bytes and allocation credits.
fn metadata(fs: &Filesystem) -> Vec<(u32, Vec<u8>)> {
    [
        b"/p".as_slice(),
        b"/p/target",
        b"/p/second",
        b"/p/nonempty",
        b"/p/nonempty/kept",
        b"/p/regular",
        b"/p/link",
        b"/p/link-file",
        b"/p/dangling",
        b"/parent-link",
    ]
    .into_iter()
    .map(|path| {
        let ino = fs.lookup_path_bytes(path).unwrap();
        (ino, fs.read_inode_verified(ino).unwrap().1)
    })
    .collect()
}

fn peer_preserved(fs: &Filesystem, raw: &[u8]) {
    let ino = fs.resolve(b"/p/regular").unwrap();
    let (inode, found) = fs.read_inode_verified(ino).unwrap();
    assert_eq!(found, raw);
    let mut data = vec![0; DATA.len()];
    assert_eq!(fs.read_ino(ino, 0, &mut data).unwrap(), DATA.len());
    assert_eq!(data, DATA);
    assert_eq!(
        crate::xattr::get_resolved(fs, &inode, &found, NAME).unwrap(),
        Some(vec![b'v'; 600])
    );
}

fn parent_times(entry: Entry, twice: bool) {
    let mut mismatches = Vec::new();
    let mut creations = 0;
    for flavor in [
        features::FsFlavor::Ext2,
        features::FsFlavor::Ext3,
        features::FsFlavor::Ext4,
    ] {
        for bs in [1024, 4096] {
            let (dev, fs, parent, _, clock) = specimen(flavor, bs);
            let mut saved = Vec::new();
            let fs = with_test_mount(fs, |handle, fs| {
                let before = fs.read_inode_verified(parent).unwrap().0;
                let peer = fs
                    .read_inode_verified(fs.resolve(b"/p/regular").unwrap())
                    .unwrap()
                    .1;
                let counts = counters(&dev);
                assert_eq!(before.ctime, INITIAL);
                let names: &[(&[u8], i64)] = if twice {
                    &[(b"new", FIRST), (b"next", SECOND)]
                } else {
                    &[(b"new", FIRST)]
                };
                for (index, &(name, stamp)) in names.iter().enumerate() {
                    clock.now.store(stamp, Ordering::SeqCst);
                    let ino = call(fs, handle, parent.into(), entry, name, 0o750).unwrap();
                    let path = [b"/p/".as_slice(), name].concat();
                    assert_eq!(fs.resolve(&path).unwrap(), ino);
                    let child = fs.read_inode_verified(ino).unwrap().0;
                    assert!(child.is_file());
                    assert_eq!(child.mode & 0xfff, 0o750);
                    assert_eq!(child.links_count, 1);
                    assert_eq!(child.size, 0);
                    assert_eq!(child.blocks, 0);
                    assert_eq!(times(&child), [(stamp, 0); 3]);
                    assert_eq!(fs.map_inode_logical(&child, 0).unwrap(), None);
                    let mut empty = [0; 1];
                    assert_eq!(fs.read_ino(ino, 0, &mut empty).unwrap(), 0);
                    assert_eq!(counters(&dev), (counts.0, counts.1 - index as u32 - 1));
                    let (after, raw) = fs.read_inode_verified(parent).unwrap();
                    assert_eq!(after.links_count, before.links_count);
                    assert_eq!(
                        (
                            after.size,
                            after.blocks,
                            after.mode,
                            after.uid,
                            after.gid,
                            after.flags
                        ),
                        (
                            before.size,
                            before.blocks,
                            before.mode,
                            before.uid,
                            before.gid,
                            before.flags
                        )
                    );
                    assert_eq!(times(&after)[0], times(&before)[0]);
                    assert_eq!(
                        crate::xattr::get_resolved(fs, &after, &raw, NAME).unwrap(),
                        Some(vec![b'a'; 512])
                    );
                    if times(&after)[1..] != [(stamp, 0); 2] {
                        mismatches.push(format!("{flavor:?}/{bs}: creation={} parent mtime/ctime={:?}, expected={stamp}", index+1, &times(&after)[1..]));
                    }
                    peer_preserved(fs, &peer);
                    saved.push((path, ino));
                    creations += 1;
                }
            });
            let parent_saved = fs.read_inode_verified(parent).unwrap().1;
            drop(fs);
            let reopened = Filesystem::mount(dev).unwrap();
            assert_eq!(
                reopened.read_inode_verified(parent).unwrap().1,
                parent_saved
            );
            for (path, ino) in saved {
                assert_eq!(reopened.resolve(&path).unwrap(), ino);
            }
        }
    }
    assert_eq!(creations, if twice { 12 } else { 6 });
    eprintln!("create policy times entry={entry:?} twice={twice}: all 6 native profiles, {creations} creations; mismatches={mismatches:?}");
    assert!(
        mismatches.is_empty(),
        "Linux-measured create parent timestamp mismatch"
    );
}

fn refusals(entry: Entry) {
    let mut observations = 0;
    for flavor in [
        features::FsFlavor::Ext2,
        features::FsFlavor::Ext3,
        features::FsFlavor::Ext4,
    ] {
        for bs in [1024, 4096] {
            let (dev, fs, parent, _, clock) = specimen(flavor, bs);
            let _fs = with_test_mount(fs, |handle, fs| {
                let before = metadata(fs);
                let counts = counters(&dev);
                clock.now.store(SECOND, Ordering::SeqCst);
                for name in [
                    b"target".as_slice(),
                    b"regular",
                    b"link",
                    b"link-file",
                    b"dangling",
                    b".",
                    b"..",
                ] {
                    assert_eq!(
                        call(fs, handle, parent.into(), entry, name, 0o750),
                        Err(crate::error::errno::EEXIST)
                    );
                    assert_eq!(metadata(fs), before);
                    assert_eq!(counters(&dev), counts);
                    observations += 1;
                }
                assert_eq!(
                    call(fs, handle, parent.into(), entry, &[b'n'; 256], 0o750),
                    Err(crate::error::errno::ENAMETOOLONG)
                );
                assert_eq!(metadata(fs), before);
                assert_eq!(counters(&dev), counts);
                observations += 1;
            });
        }
    }
    assert_eq!(observations, 48);
    eprintln!("create refusal entry={entry:?}: {observations} unchanged-metadata observations");
}

macro_rules! entry_tests {
    ($entry:ident, $first:ident, $second:ident, $refusals:ident) => {
        #[test]
        fn $first() {
            parent_times(Entry::$entry, false);
        }
        #[test]
        fn $second() {
            parent_times(Entry::$entry, true);
        }
        #[test]
        fn $refusals() {
            refusals(Entry::$entry);
        }
    };
}
entry_tests!(
    RustString,
    rust_string_first_parent_times_follow_linux,
    rust_string_second_parent_times_follow_linux,
    rust_string_refusals_preserve_metadata
);
entry_tests!(
    RustBytes,
    rust_bytes_first_parent_times_follow_linux,
    rust_bytes_second_parent_times_follow_linux,
    rust_bytes_refusals_preserve_metadata
);
entry_tests!(
    RustAt,
    rust_at_first_parent_times_follow_linux,
    rust_at_second_parent_times_follow_linux,
    rust_at_refusals_preserve_metadata
);
entry_tests!(
    CPath,
    c_path_first_parent_times_follow_linux,
    c_path_second_parent_times_follow_linux,
    c_path_refusals_preserve_metadata
);
entry_tests!(
    CAt,
    c_at_first_parent_times_follow_linux,
    c_at_second_parent_times_follow_linux,
    c_at_refusals_preserve_metadata
);

fn mixed_paths(entry: Entry) {
    let mut observations = 0;
    let mut mismatches = Vec::new();
    for flavor in [
        features::FsFlavor::Ext2,
        features::FsFlavor::Ext3,
        features::FsFlavor::Ext4,
    ] {
        for bs in [1024, 4096] {
            let (dev, fs, parent, _, clock) = specimen(flavor, bs);
            let _fs = with_test_mount(fs, |handle, fs| {
                let before = metadata(fs);
                let counts = counters(&dev);
                clock.now.store(SECOND, Ordering::SeqCst);
                for (prefix, expected) in [
                    (b"regular/".as_slice(), crate::error::errno::ENOTDIR),
                    (b"missing/", crate::error::errno::ENOENT),
                    (b"link/", crate::error::errno::ENAMETOOLONG),
                    (b"target/", crate::error::errno::ENAMETOOLONG),
                    (b"./", crate::error::errno::ENAMETOOLONG),
                    (b"../", crate::error::errno::ENAMETOOLONG),
                ] {
                    let name = [prefix, &[b'n'; 256]].concat();
                    let found = call(fs, handle, parent.into(), entry, &name, 0o750);
                    if found != Err(expected) {
                        mismatches.push(format!("{flavor:?}/{bs}: prefix={prefix:?} found={found:?} expected={expected}"));
                    }
                    assert_eq!(metadata(fs), before);
                    assert_eq!(counters(&dev), counts);
                    observations += 1;
                }
            });
        }
    }
    assert_eq!(observations, 36);
    eprintln!("create mixed paths entry={entry:?}: all 6 native profiles, {observations} observations; mismatches={mismatches:?}");
    assert!(
        mismatches.is_empty(),
        "Linux-measured create mixed path policy mismatch"
    );
}
#[test]
fn rust_string_mixed_path_priorities_follow_linux() {
    mixed_paths(Entry::RustString);
}
#[test]
fn rust_bytes_mixed_path_priorities_follow_linux() {
    mixed_paths(Entry::RustBytes);
}
#[test]
fn c_path_mixed_path_priorities_follow_linux() {
    mixed_paths(Entry::CPath);
}

fn counted_priorities(entry: Entry) {
    let mut observations = 0;
    for flavor in [
        features::FsFlavor::Ext2,
        features::FsFlavor::Ext3,
        features::FsFlavor::Ext4,
    ] {
        for bs in [1024, 4096] {
            let (dev, fs, parent, _, _) = specimen(flavor, bs);
            let _fs = with_test_mount(fs, |handle, fs| {
                let before = metadata(fs);
                let counts = counters(&dev);
                let stale = InodeRef::new(
                    parent,
                    fs.read_inode_verified(parent).unwrap().0.generation ^ 1,
                );
                let regular = fs.resolve(b"/p/regular").unwrap();
                for (dir, name, expected) in [
                    (stale, b"".as_slice(), crate::error::errno::EINVAL),
                    (stale, b"bad/name", crate::error::errno::EINVAL),
                    (stale, b"bad\0name", crate::error::errno::EINVAL),
                    (stale, &[b'n'; 256], crate::error::errno::ENAMETOOLONG),
                    (
                        regular.into(),
                        &[b'n'; 256],
                        crate::error::errno::ENAMETOOLONG,
                    ),
                    (0u32.into(), &[b'n'; 256], crate::error::errno::ENAMETOOLONG),
                    (stale, b"new", crate::error::errno::ESTALE),
                    (regular.into(), b"new", crate::error::errno::ENOTDIR),
                ] {
                    assert_eq!(call(fs, handle, dir, entry, name, 0o750), Err(expected));
                    assert_eq!(metadata(fs), before);
                    assert_eq!(counters(&dev), counts);
                    observations += 1;
                }
                if matches!(entry, Entry::CAt) {
                    for (name, len, expected) in [
                        (std::ptr::null(), 1, crate::error::errno::EINVAL),
                        (
                            b"x".as_ptr().cast(),
                            usize::MAX,
                            crate::error::errno::ENAMETOOLONG,
                        ),
                    ] {
                        assert_eq!(
                            unsafe {
                                fs_ext4_create_at(
                                    handle,
                                    parent,
                                    FS_EXT4_GEN_ANY,
                                    name,
                                    len,
                                    0o750,
                                    std::ptr::null_mut(),
                                )
                            },
                            -1
                        );
                        assert_eq!(fs_ext4_last_errno(), expected);
                        assert_eq!(metadata(fs), before);
                        assert_eq!(counters(&dev), counts);
                        observations += 1;
                    }
                }
            });
        }
    }
    assert_eq!(
        observations,
        if matches!(entry, Entry::CAt) { 60 } else { 48 }
    );
    eprintln!(
        "create counted priorities entry={entry:?}: {observations} explicit-API observations"
    );
}
#[test]
fn rust_at_preserves_counted_name_priorities() {
    counted_priorities(Entry::RustAt);
}
#[test]
fn c_at_preserves_counted_name_and_ffi_priorities() {
    counted_priorities(Entry::CAt);
}

struct Convert<'a> {
    calls: &'a AtomicUsize,
    parent: u32,
}
impl From<Convert<'_>> for InodeRef {
    fn from(v: Convert<'_>) -> Self {
        v.calls.fetch_add(1, Ordering::SeqCst);
        v.parent.into()
    }
}
#[test]
fn caller_conversion_precedes_invalid_counted_names() {
    let mut observations = 0;
    for flavor in [
        features::FsFlavor::Ext2,
        features::FsFlavor::Ext3,
        features::FsFlavor::Ext4,
    ] {
        for bs in [1024, 4096] {
            let (_, fs, parent, _, _) = specimen(flavor, bs);
            let calls = AtomicUsize::new(0);
            for (index, name) in [b"".as_slice(), b"bad/name", b"bad\0name", &[b'n'; 256]]
                .into_iter()
                .enumerate()
            {
                let got = fs.apply_create_at(
                    Convert {
                        calls: &calls,
                        parent,
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
                assert_eq!(calls.load(Ordering::SeqCst), index + 1);
                observations += 1;
            }
        }
    }
    assert_eq!(observations, 24);
    eprintln!("create conversion: {observations} invalid names refused after Into");
}

#[test]
fn all_entries_create_255_byte_names_and_preserve_exact_identity() {
    let mut creations = 0;
    for flavor in [
        features::FsFlavor::Ext2,
        features::FsFlavor::Ext3,
        features::FsFlavor::Ext4,
    ] {
        for bs in [1024, 4096] {
            for entry in ENTRIES {
                // A fresh parent keeps this at the independently measured
                // non-growing boundary; five simultaneous long names can grow it.
                let (dev, fs, parent, _, _) = specimen(flavor, bs);
                let name = [b'n'; 255];
                let mut saved = 0;
                let fs = with_test_mount(fs, |handle, fs| {
                    let counts = counters(&dev);
                    let before = fs.read_inode_verified(parent).unwrap().0;
                    let ino = call(fs, handle, parent.into(), entry, &name, 0o750).unwrap();
                    assert_eq!(
                        fs.resolve(&[b"/p/".as_slice(), &name].concat()).unwrap(),
                        ino
                    );
                    let after = fs.read_inode_verified(parent).unwrap().0;
                    assert_eq!((after.size, after.blocks), (before.size, before.blocks));
                    assert_eq!(counters(&dev), (counts.0, counts.1 - 1));
                    saved = ino;
                    creations += 1;
                });
                drop(fs);
                let reopened = Filesystem::mount(dev).unwrap();
                assert_eq!(
                    reopened
                        .resolve(&[b"/p/".as_slice(), &name].concat())
                        .unwrap(),
                    saved
                );
            }
        }
    }
    assert_eq!(creations, 30);
    eprintln!("create 255-byte boundary: all {creations} non-growing creations and cold identities retained");
}

#[test]
fn byte_and_c_entries_preserve_raw_non_utf8_names() {
    let mut creations = 0;
    for flavor in [
        features::FsFlavor::Ext2,
        features::FsFlavor::Ext3,
        features::FsFlavor::Ext4,
    ] {
        for bs in [1024, 4096] {
            let (dev, fs, parent, _, _) = specimen(flavor, bs);
            let fs = with_test_mount(fs, |handle, fs| {
                for (i, entry) in [Entry::RustBytes, Entry::RustAt, Entry::CPath, Entry::CAt]
                    .into_iter()
                    .enumerate()
                {
                    let name = [b'r', b'a' + i as u8, 0xff];
                    let ino = call(fs, handle, parent.into(), entry, &name, 0o750).unwrap();
                    assert_eq!(
                        fs.resolve(&[b"/p/".as_slice(), &name].concat()).unwrap(),
                        ino
                    );
                    creations += 1;
                }
            });
            drop(fs);
            let reopened = Filesystem::mount(dev).unwrap();
            for i in 0..4 {
                assert!(reopened
                    .resolve(&[b"/p/".as_slice(), &[b'r', b'a' + i, 0xff]].concat())
                    .is_ok());
            }
        }
    }
    assert_eq!(creations, 24);
    eprintln!("create raw names: all {creations} exact byte identities retained cold");
}

#[test]
fn all_entries_preserve_explicit_low_twelve_mode_bits() {
    let mut creations = 0;
    for flavor in [
        features::FsFlavor::Ext2,
        features::FsFlavor::Ext3,
        features::FsFlavor::Ext4,
    ] {
        for bs in [1024, 4096] {
            let (_, fs, parent, _, _) = specimen(flavor, bs);
            let _fs = with_test_mount(fs, |handle, fs| {
                for (i, entry) in ENTRIES.into_iter().enumerate() {
                    for mode in [0o2750, 0o1770, 0o750] {
                        let name = format!("mode-{i}-{mode:o}");
                        let ino =
                            call(fs, handle, parent.into(), entry, name.as_bytes(), mode).unwrap();
                        assert_eq!(fs.read_inode_verified(ino).unwrap().0.mode & 0xfff, mode);
                        creations += 1;
                    }
                }
            });
        }
    }
    assert_eq!(creations, 90);
    eprintln!("create explicit mode contract: all {creations} low-twelve-bit modes retained; no syscall credential/umask emulation claimed");
}

fn path_forms(entry: Entry) {
    let mut observations = 0;
    let mut creations = 0;
    for flavor in [
        features::FsFlavor::Ext2,
        features::FsFlavor::Ext3,
        features::FsFlavor::Ext4,
    ] {
        for bs in [1024, 4096] {
            let (dev, fs, parent, _, _) = specimen(flavor, bs);
            let _fs = with_test_mount(fs, |handle, fs| {
                let before = metadata(fs);
                let counts = counters(&dev);
                for (name, expected) in [
                    (b"target/".as_slice(), crate::error::errno::EEXIST),
                    (b"regular/", crate::error::errno::EEXIST),
                    (b"link/", crate::error::errno::EEXIST),
                    (b"link-file/", crate::error::errno::EEXIST),
                    (b"./", crate::error::errno::EEXIST),
                    (b"../", crate::error::errno::EEXIST),
                    (b"regular/new", crate::error::errno::ENOTDIR),
                    (b"missing/new", crate::error::errno::ENOENT),
                    (b"regular/.", crate::error::errno::ENOTDIR),
                    (b"missing/.", crate::error::errno::ENOENT),
                    (b"link/.", crate::error::errno::EEXIST),
                ] {
                    assert_eq!(
                        call(fs, handle, parent.into(), entry, name, 0o750),
                        Err(expected)
                    );
                    assert_eq!(metadata(fs), before);
                    assert_eq!(counters(&dev), counts);
                    observations += 1;
                }
                // Separator and rooted-parent syntax is an explicit API boundary,
                // separate from Linux's cwd-relative empty-name syscall error.
                for path in [b"".as_slice(), b"relative"] {
                    assert_eq!(
                        call_path(
                            fs,
                            handle,
                            parent.into(),
                            entry,
                            b"unused",
                            0o750,
                            Some(path)
                        ),
                        Err(crate::error::errno::EINVAL)
                    );
                    assert_eq!(metadata(fs), before);
                    assert_eq!(counters(&dev), counts);
                    observations += 1;
                }
                let ino = call_path(
                    fs,
                    handle,
                    parent.into(),
                    entry,
                    b"new",
                    0o750,
                    Some(b"/parent-link/new///"),
                )
                .unwrap();
                assert_eq!(fs.resolve(b"/p/new").unwrap(), ino);
                assert_eq!(counters(&dev), (counts.0, counts.1 - 1));
                let rooted = call_path(
                    fs,
                    handle,
                    parent.into(),
                    entry,
                    b"rooted",
                    0o750,
                    Some(b"p/rooted"),
                )
                .unwrap();
                assert_ne!(ino, rooted);
                assert_eq!(fs.resolve(b"/p/rooted").unwrap(), rooted);
                assert_eq!(counters(&dev), (counts.0, counts.1 - 2));
                creations += 2;
            });
        }
    }
    assert_eq!(observations, 78);
    assert_eq!(creations, 12);
    eprintln!("create path forms entry={entry:?}: {observations} refusal/syntax observations and {creations} symlink/trailing and rooted-parent creations");
}
#[test]
fn rust_string_preserves_path_forms_and_syntax_contract() {
    path_forms(Entry::RustString);
}
#[test]
fn rust_bytes_preserves_path_forms_and_syntax_contract() {
    path_forms(Entry::RustBytes);
}
#[test]
fn c_path_preserves_path_forms_and_syntax_contract() {
    path_forms(Entry::CPath);
}
