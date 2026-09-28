//! The C ABI's inode-addressed entry points (#372).
//!
//! A handle-based host holds an inode number (and the generation it read
//! with it) for every item, and a `(directory inode, name)` pair for every
//! mutation. These tests drive the `_ino` / `_at` exports the way such a
//! host would, and hold each one to its path twin: the same attributes,
//! entries, bytes and errors. A freed inode answers ESTALE.
//!
//! Byte-for-byte agreement of the Rust entry points with their path twins is
//! in `tests/inode_api.rs`; e2fsck's verdict on an image written this way is
//! in `tests/inode_api_e2fsck.rs`.

use fs_ext4::capi::*;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::sync::atomic::{AtomicU32, Ordering};

const ROOT: u32 = 2;
const ANY: u32 = FS_EXT4_GEN_ANY;
const ESTALE: i32 = fs_ext4::error::errno::ESTALE;

fn last_err() -> String {
    unsafe {
        CStr::from_ptr(fs_ext4_last_error())
            .to_string_lossy()
            .into_owned()
    }
}

fn errno() -> i32 {
    fs_ext4_last_errno()
}

/// The errno of a call expected to fail.
fn refused<T>(r: Result<T, i32>) -> i32 {
    match r {
        Ok(_) => panic!("expected a refusal"),
        Err(e) => e,
    }
}

/// A freshly formatted scratch volume, mounted read-write.
struct Volume {
    fs: *mut fs_ext4_fs_t,
    path: String,
}

impl Volume {
    fn new() -> Self {
        static N: AtomicU32 = AtomicU32::new(0);
        let path = fs_ext4_test_support::temp_path!(
            "fs_ext4_capi_ino_{}_{}.img",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        );
        let size = 16 * 1024 * 1024;
        std::fs::File::create(&path)
            .and_then(|f| f.set_len(size))
            .unwrap();
        let dev = fs_ext4::block_io::FileDevice::open_rw(&path).unwrap();
        fs_ext4::mkfs::format_filesystem(&dev, None, Some([9; 16]), size, 4096).unwrap();
        drop(dev);
        let c = CString::new(path.clone()).unwrap();
        let fs = unsafe { fs_ext4_mount_rw(c.as_ptr()) };
        assert!(!fs.is_null(), "mount_rw: {}", last_err());
        Volume { fs, path }
    }
}

impl Drop for Volume {
    fn drop(&mut self) {
        unsafe { fs_ext4_umount(self.fs) };
        std::fs::remove_file(&self.path).ok();
    }
}

fn zeroed_attr() -> fs_ext4_attr_t {
    unsafe { std::mem::zeroed() }
}

/// Every field, so two attrs compare whole.
fn fields(a: &fs_ext4_attr_t) -> String {
    format!(
        "ino={} mode={:o} uid={} gid={} size={} a={}.{} m={}.{} c={}.{} cr={}.{} links={} type={:?} flags={:#x} gen={} blocks={}",
        a.inode, a.mode, a.uid, a.gid, a.size, a.atime, a.atime_nsec, a.mtime, a.mtime_nsec,
        a.ctime, a.ctime_nsec, a.crtime, a.crtime_nsec, a.link_count, a.file_type as u32,
        a.inode_flags, a.generation, a.blocks_512
    )
}

fn stat(v: &Volume, path: &str) -> fs_ext4_attr_t {
    let c = CString::new(path).unwrap();
    let mut a = zeroed_attr();
    let rc = unsafe { fs_ext4_stat(v.fs, c.as_ptr(), &mut a) };
    assert_eq!(rc, 0, "stat {path}: {}", last_err());
    a
}

fn stat_ino(v: &Volume, ino: u32, generation: u32) -> Result<fs_ext4_attr_t, i32> {
    let mut a = zeroed_attr();
    match unsafe { fs_ext4_stat_ino(v.fs, ino, generation, &mut a) } {
        0 => Ok(a),
        _ => Err(errno()),
    }
}

fn lookup(v: &Volume, dir: u32, name: &[u8]) -> Result<fs_ext4_attr_t, i32> {
    let mut a = zeroed_attr();
    let rc = unsafe { fs_ext4_lookup_at(v.fs, dir, ANY, name.as_ptr().cast(), name.len(), &mut a) };
    if rc == 0 {
        Ok(a)
    } else {
        Err(errno())
    }
}

fn create_at(v: &Volume, dir: u32, name: &[u8]) -> fs_ext4_attr_t {
    let mut a = zeroed_attr();
    let rc = unsafe {
        fs_ext4_create_at(
            v.fs,
            dir,
            ANY,
            name.as_ptr().cast(),
            name.len(),
            0o644,
            &mut a,
        )
    };
    assert_eq!(rc, 0, "create_at {name:?}: {}", last_err());
    a
}

fn mkdir_at(v: &Volume, dir: u32, name: &[u8]) -> fs_ext4_attr_t {
    let mut a = zeroed_attr();
    let rc = unsafe {
        fs_ext4_mkdir_at(
            v.fs,
            dir,
            ANY,
            name.as_ptr().cast(),
            name.len(),
            0o755,
            &mut a,
        )
    };
    assert_eq!(rc, 0, "mkdir_at {name:?}: {}", last_err());
    a
}

fn pwrite_ino(v: &Volume, ino: u32, generation: u32, data: &[u8], off: u64) -> i64 {
    unsafe {
        fs_ext4_pwrite_ino(
            v.fs,
            ino,
            generation,
            data.as_ptr().cast(),
            data.len() as u64,
            off,
        )
    }
}

fn read_path(v: &Volume, path: &str, len: usize) -> Vec<u8> {
    let c = CString::new(path).unwrap();
    let mut out = vec![0u8; len];
    let n = unsafe { fs_ext4_read_file(v.fs, c.as_ptr(), out.as_mut_ptr().cast(), 0, len as u64) };
    assert!(n >= 0, "read_file {path}: {}", last_err());
    out.truncate(n as usize);
    out
}

fn read_ino(v: &Volume, ino: u32, off: u64, len: usize) -> Result<Vec<u8>, i32> {
    let mut out = vec![0u8; len];
    let n = unsafe {
        fs_ext4_pread_ino(
            v.fs,
            ino,
            ANY,
            out.as_mut_ptr().cast::<c_void>(),
            off,
            len as u64,
        )
    };
    if n < 0 {
        return Err(errno());
    }
    out.truncate(n as usize);
    Ok(out)
}

unsafe fn drain(iter: *mut fs_ext4_dir_iter_t) -> Vec<(Vec<u8>, u32, u8)> {
    assert!(!iter.is_null(), "dir open: {}", last_err());
    let mut out = Vec::new();
    loop {
        let e = fs_ext4_dir_next(iter);
        if e.is_null() {
            break;
        }
        let e = &*e;
        let name: Vec<u8> = e.name[..e.name_len as usize]
            .iter()
            .map(|&c| c.to_ne_bytes()[0])
            .collect();
        out.push((name, e.inode, e.file_type));
    }
    fs_ext4_dir_close(iter);
    out.sort();
    out
}

fn list_path(v: &Volume, path: &str) -> Vec<(Vec<u8>, u32, u8)> {
    let c = CString::new(path).unwrap();
    unsafe { drain(fs_ext4_dir_open(v.fs, c.as_ptr())) }
}

fn list_ino(v: &Volume, ino: u32) -> Vec<(Vec<u8>, u32, u8)> {
    unsafe { drain(fs_ext4_dir_open_ino(v.fs, ino, ANY)) }
}

/// The scenario that motivated the API: a hard link and a directory rename
/// leave an inode handle valid, and the last unlink makes it stale.
#[test]
fn an_inode_handle_survives_links_and_renames_and_goes_stale_when_freed() {
    let v = Volume::new();
    let a = mkdir_at(&v, ROOT, b"a");
    let b = mkdir_at(&v, ROOT, b"b");
    let f = create_at(&v, a.inode, b"f");
    assert!(f.inode != 0 && f.link_count == 1, "{}", fields(&f));

    let g = b"g";
    let rc = unsafe {
        fs_ext4_link_at(
            v.fs,
            f.inode,
            f.generation,
            b.inode,
            b.generation,
            g.as_ptr().cast(),
            1,
            std::ptr::null_mut(),
        )
    };
    assert_eq!(rc, 0, "link_at: {}", last_err());
    let rc = unsafe {
        fs_ext4_rename_at(
            v.fs,
            ROOT,
            ANY,
            b"a".as_ptr().cast(),
            1,
            ROOT,
            ANY,
            b"c".as_ptr().cast(),
            1,
            0,
        )
    };
    assert_eq!(rc, 0, "rename_at: {}", last_err());

    let payload = b"written through the inode";
    assert_eq!(
        pwrite_ino(&v, f.inode, f.generation, payload, 0),
        payload.len() as i64,
        "{}",
        last_err()
    );
    assert_eq!(read_path(&v, "/c/f", 64), payload);
    assert_eq!(read_path(&v, "/b/g", 64), payload);

    let rc = unsafe { fs_ext4_unlink_at(v.fs, b.inode, b.generation, g.as_ptr().cast(), 1) };
    assert_eq!(rc, 0, "unlink_at g: {}", last_err());
    assert_eq!(stat(&v, "/c/f").link_count, 1);
    let rc = unsafe { fs_ext4_unlink_at(v.fs, a.inode, a.generation, b"f".as_ptr().cast(), 1) };
    assert_eq!(rc, 0, "unlink_at f: {}", last_err());

    assert_eq!(refused(stat_ino(&v, f.inode, f.generation)), ESTALE);
    assert_eq!(refused(stat_ino(&v, f.inode, ANY)), ESTALE);

    // Reused slot, new generation: the old handle stays stale and writes
    // through it are refused.
    let h = create_at(&v, ROOT, b"h");
    assert_eq!(
        h.inode, f.inode,
        "the slot must be reused for this check to mean anything"
    );
    assert_ne!(h.generation, f.generation);
    assert_eq!(refused(stat_ino(&v, f.inode, f.generation)), ESTALE);
    assert_eq!(pwrite_ino(&v, f.inode, f.generation, b"x", 0), -1);
    assert_eq!(errno(), ESTALE);
    assert_eq!(stat_ino(&v, h.inode, h.generation).unwrap().size, 0);
}

#[test]
fn a_non_utf8_name_round_trips_byte_exact() {
    let v = Volume::new();
    let name: &[u8] = b"\xff\xfe-not-utf8-\xe9";
    let made = create_at(&v, ROOT, name);
    let listed = list_ino(&v, ROOT);
    assert!(
        listed
            .iter()
            .any(|(n, ino, _)| n == name && *ino == made.inode),
        "{listed:?}"
    );
    assert_eq!(lookup(&v, ROOT, name).unwrap().inode, made.inode);
}

#[test]
fn every_read_entry_point_matches_its_path_twin() {
    let v = Volume::new();
    let d = mkdir_at(&v, ROOT, b"d");
    let f = create_at(&v, d.inode, b"f");
    assert_eq!(pwrite_ino(&v, f.inode, ANY, &[3u8; 3000], 100), 3100);
    let target = CString::new("t".repeat(150)).unwrap();
    let mut s = zeroed_attr();
    let rc = unsafe {
        fs_ext4_symlink_at(
            v.fs,
            d.inode,
            ANY,
            b"s".as_ptr().cast(),
            1,
            target.as_ptr(),
            &mut s,
        )
    };
    assert_eq!(rc, 0, "symlink_at: {}", last_err());

    for (path, ino) in [
        ("/", ROOT),
        ("/d", d.inode),
        ("/d/f", f.inode),
        ("/d/s", s.inode),
    ] {
        assert_eq!(
            fields(&stat(&v, path)),
            fields(&stat_ino(&v, ino, ANY).unwrap()),
            "{path}"
        );
    }
    let (dir_name, name) = (d.inode, b"f");
    assert_eq!(
        fields(&lookup(&v, dir_name, name).unwrap()),
        fields(&stat(&v, "/d/f"))
    );
    assert_eq!(list_path(&v, "/d"), list_ino(&v, d.inode));
    assert_eq!(list_path(&v, "/"), list_ino(&v, ROOT));
    assert_eq!(
        read_path(&v, "/d/f", 5000),
        read_ino(&v, f.inode, 0, 5000).unwrap()
    );

    let mut by_path = [0 as c_char; 256];
    let mut by_ino = [0 as c_char; 256];
    let p = CString::new("/d/s").unwrap();
    let n1 = unsafe { fs_ext4_readlink(v.fs, p.as_ptr(), by_path.as_mut_ptr(), 256) };
    let n2 = unsafe { fs_ext4_readlink_ino(v.fs, s.inode, s.generation, by_ino.as_mut_ptr(), 256) };
    assert_eq!((n1, n2), (150, 150), "{}", last_err());
    assert_eq!(by_path[..151], by_ino[..151]);
    // The #313 contract: too small a buffer is ERANGE and nothing written.
    let mut small = [0x55 as c_char; 150];
    let rc = unsafe { fs_ext4_readlink_ino(v.fs, s.inode, ANY, small.as_mut_ptr(), 150) };
    assert_eq!((rc, errno()), (-1, fs_ext4::error::errno::ERANGE));
    assert!(small.iter().all(|&c| c == 0x55));

    // Refusals match too.
    assert_eq!(
        read_ino(&v, d.inode, 0, 8).unwrap_err(),
        fs_ext4::error::errno::EINVAL
    );
    assert!(unsafe { fs_ext4_dir_open_ino(v.fs, f.inode, ANY) }.is_null());
    assert_eq!(errno(), fs_ext4::error::errno::ENOTDIR);
    assert_eq!(
        refused(lookup(&v, ROOT, b"missing")),
        fs_ext4::error::errno::ENOENT
    );
}

/// A tree as `(path, type, mode, uid, gid, size, links, content)` rows, for
/// comparing two volumes that were written the same way at different times.
fn tree(v: &Volume) -> Vec<String> {
    let mut rows = Vec::new();
    let mut stack = vec![String::new()];
    while let Some(dir) = stack.pop() {
        let listing = list_path(v, if dir.is_empty() { "/" } else { &dir });
        for (name, _, _) in listing {
            if name == b"." || name == b".." || name == b"lost+found" {
                continue;
            }
            let path = format!("{dir}/{}", String::from_utf8(name).unwrap());
            let a = stat(v, &path);
            let body = match a.file_type {
                fs_ext4_file_type_t::RegFile => {
                    format!("{:?}", read_path(v, &path, a.size as usize))
                }
                fs_ext4_file_type_t::Dir => {
                    stack.push(path.clone());
                    String::new()
                }
                _ => String::new(),
            };
            rows.push(format!(
                "{path} {:?} {:o} {} {} {} {} {body}",
                a.file_type as u32, a.mode, a.uid, a.gid, a.size, a.link_count
            ));
        }
    }
    rows.sort();
    rows
}

#[test]
fn every_write_entry_point_matches_its_path_twin() {
    let by_path = Volume::new();
    let by_ino = Volume::new();
    let c = |s: &str| CString::new(s).unwrap();
    let rc_path: Vec<i32> = unsafe {
        let fs = by_path.fs;
        vec![
            (fs_ext4_mkdir(fs, c("/d").as_ptr(), 0o750) != 0) as i32,
            (fs_ext4_mkdir(fs, c("/d/e").as_ptr(), 0o755) != 0) as i32,
            (fs_ext4_create(fs, c("/d/f").as_ptr(), 0o640) != 0) as i32,
            fs_ext4_pwrite(
                fs,
                c("/d/f").as_ptr(),
                [9u8; 2000].as_ptr().cast(),
                2000,
                10,
            ) as i32,
            fs_ext4_truncate(fs, c("/d/f").as_ptr(), 1500),
            fs_ext4_chmod(fs, c("/d/f").as_ptr(), 0o604),
            fs_ext4_chown(fs, c("/d/f").as_ptr(), 7, 8),
            fs_ext4_utimens(fs, c("/d/f").as_ptr(), 100, 0, 200, 0),
            fs_ext4_link(fs, c("/d/f").as_ptr(), c("/d/e/g").as_ptr()),
            (fs_ext4_symlink(fs, c("f").as_ptr(), c("/d/s").as_ptr()) != 0) as i32,
            (fs_ext4_mknod(fs, c("/d/p").as_ptr(), 0o010600, 0, 0) != 0) as i32,
            fs_ext4_rename2(fs, c("/d/p").as_ptr(), c("/d/e/p").as_ptr(), 0),
            fs_ext4_rename2(
                fs,
                c("/d/s").as_ptr(),
                c("/d/e/g").as_ptr(),
                FS_EXT4_RENAME_REPLACE,
            ),
            (fs_ext4_mkdir(fs, c("/d/gone").as_ptr(), 0o755) != 0) as i32,
            fs_ext4_rmdir(fs, c("/d/gone").as_ptr()),
            fs_ext4_unlink(fs, c("/d/e/p").as_ptr()),
            // Refusals: the errno is what is compared.
            fs_ext4_rmdir(fs, c("/d").as_ptr()),
            fs_ext4_unlink(fs, c("/d/e").as_ptr()),
            fs_ext4_rename2(fs, c("/d").as_ptr(), c("/d/e/x").as_ptr(), 0),
            fs_ext4_truncate(fs, c("/d/e").as_ptr(), 0),
        ]
        .into_iter()
        .map(|rc| if rc < 0 { -errno() } else { rc })
        .collect()
    };
    let rc_ino: Vec<i32> = unsafe {
        let fs = by_ino.fs;
        let n = |s: &'static str| (s.as_ptr().cast::<c_char>(), s.len());
        let mut d = zeroed_attr();
        let mut e = zeroed_attr();
        let mut f = zeroed_attr();
        let mut gone = zeroed_attr();
        let null = std::ptr::null_mut();
        let mut out =
            vec![(fs_ext4_mkdir_at(fs, ROOT, ANY, n("d").0, n("d").1, 0o750, &mut d) == 0) as i32];
        out.push(
            (fs_ext4_mkdir_at(fs, d.inode, d.generation, n("e").0, 1, 0o755, &mut e) == 0) as i32,
        );
        out.push(
            (fs_ext4_create_at(fs, d.inode, d.generation, n("f").0, 1, 0o640, &mut f) == 0) as i32,
        );
        out.extend([
            fs_ext4_pwrite_ino(
                fs,
                f.inode,
                f.generation,
                [9u8; 2000].as_ptr().cast(),
                2000,
                10,
            ) as i32,
            fs_ext4_truncate_ino(fs, f.inode, f.generation, 1500),
            fs_ext4_chmod_ino(fs, f.inode, f.generation, 0o604),
            fs_ext4_chown_ino(fs, f.inode, f.generation, 7, 8),
            fs_ext4_utimens_ino(fs, f.inode, f.generation, 100, 0, 200, 0),
            fs_ext4_link_at(
                fs,
                f.inode,
                f.generation,
                e.inode,
                e.generation,
                n("g").0,
                1,
                null,
            ),
            (fs_ext4_symlink_at(fs, d.inode, ANY, n("s").0, 1, c("f").as_ptr(), null) == 0) as i32,
            (fs_ext4_mknod_at(fs, d.inode, ANY, n("p").0, 1, 0o010600, 0, 0, null) == 0) as i32,
            fs_ext4_rename_at(fs, d.inode, ANY, n("p").0, 1, e.inode, ANY, n("p").0, 1, 0),
            fs_ext4_rename_at(
                fs,
                d.inode,
                ANY,
                n("s").0,
                1,
                e.inode,
                ANY,
                n("g").0,
                1,
                FS_EXT4_RENAME_REPLACE,
            ),
            (fs_ext4_mkdir_at(fs, d.inode, ANY, n("gone").0, 4, 0o755, &mut gone) == 0) as i32,
            fs_ext4_rmdir_at(fs, d.inode, ANY, n("gone").0, 4),
            fs_ext4_unlink_at(fs, e.inode, ANY, n("p").0, 1),
            fs_ext4_rmdir_at(fs, ROOT, ANY, n("d").0, 1),
            fs_ext4_unlink_at(fs, d.inode, ANY, n("e").0, 1),
            fs_ext4_rename_at(fs, ROOT, ANY, n("d").0, 1, e.inode, ANY, n("x").0, 1, 0),
            fs_ext4_truncate_ino(fs, e.inode, ANY, 0),
        ]);
        out.into_iter()
            .map(|rc| if rc < 0 { -errno() } else { rc })
            .collect()
    };
    assert_eq!(rc_path, rc_ino);
    assert!(
        rc_path[16..].iter().all(|&rc| rc < 0),
        "the refusals must refuse: {rc_path:?}"
    );
    assert_eq!(tree(&by_path), tree(&by_ino));
}

#[test]
fn bad_arguments_are_refused_cleanly() {
    let v = Volume::new();
    let mut a = zeroed_attr();
    unsafe {
        assert_eq!(
            fs_ext4_stat_ino(std::ptr::null_mut(), ROOT, ANY, &mut a),
            -1
        );
        assert_eq!(errno(), fs_ext4::error::errno::EINVAL);
        assert_eq!(fs_ext4_stat_ino(v.fs, ROOT, ANY, std::ptr::null_mut()), -1);
        assert_eq!(errno(), fs_ext4::error::errno::EINVAL);
        assert_eq!(
            fs_ext4_lookup_at(v.fs, ROOT, ANY, std::ptr::null(), 1, &mut a),
            -1
        );
        assert_eq!(errno(), fs_ext4::error::errno::EINVAL);
        assert_eq!(
            fs_ext4_create_at(v.fs, ROOT, ANY, b"x".as_ptr().cast(), 0, 0o644, &mut a),
            -1
        );
        assert_eq!(errno(), fs_ext4::error::errno::EINVAL);
        assert_eq!(
            fs_ext4_rename_at(
                v.fs,
                ROOT,
                ANY,
                b"a".as_ptr().cast(),
                1,
                ROOT,
                ANY,
                b"b".as_ptr().cast(),
                1,
                0x80
            ),
            -1
        );
        assert_eq!(errno(), fs_ext4::error::errno::EINVAL);
        // A generation that does not match is stale, even for the root.
        let root = stat_ino(&v, ROOT, ANY).unwrap();
        assert_eq!(
            refused(stat_ino(&v, ROOT, root.generation.wrapping_add(1))),
            ESTALE
        );
        assert_eq!(refused(stat_ino(&v, 0, ANY)), ESTALE);
    }
}
