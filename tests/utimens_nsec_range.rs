//! `apply_utimens` and the nanoseconds field (#326).
//!
//! A nanosecond count is below one billion. ext4 packs it into the top
//! 30 bits of the matching `*_extra` field, so a value from 1e9 to
//! 2^30-1 was stored verbatim and read back as an impossible `tv_nsec`,
//! and anything from 2^30 up was masked into an unrelated number.
//!
//! The two values in that range that `utimensat(2)` gives a meaning to
//! -- `UTIME_NOW` and `UTIME_OMIT` -- are honoured per field; every
//! other one is refused with `InvalidArgument` before the inode is
//! touched.
//!
//! In-memory volume, deterministic clock: no fixture, no VM.

use fs_ext4::{block_io::BlockDevice, error::Error, error::Result, runtime::Runtime, Filesystem};
use std::sync::{Arc, Mutex};

/// `utimensat(2)`'s sentinels as Linux defines them.
const UTIME_NOW: u32 = (1 << 30) - 1;
const UTIME_OMIT: u32 = (1 << 30) - 2;

const NOW: i64 = 1_700_000_123;

struct MemDev {
    bytes: Mutex<Vec<u8>>,
    size: u64,
}

impl BlockDevice for MemDev {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<()> {
        let b = self.bytes.lock().unwrap();
        let start = offset as usize;
        buf.copy_from_slice(&b[start..start + buf.len()]);
        Ok(())
    }
    fn size_bytes(&self) -> u64 {
        self.size
    }
    fn write_at(&self, offset: u64, buf: &[u8]) -> Result<()> {
        let mut b = self.bytes.lock().unwrap();
        let start = offset as usize;
        b[start..start + buf.len()].copy_from_slice(buf);
        Ok(())
    }
    fn flush(&self) -> Result<()> {
        Ok(())
    }
    fn is_writable(&self) -> bool {
        true
    }
}

struct Fixed;
impl Runtime for Fixed {
    fn now_unix_seconds(&self) -> i64 {
        NOW
    }
    fn next_inode_generation(&self) -> u32 {
        1
    }
}

/// A fresh volume holding `/f`, with atime and mtime set to known values
/// that differ from each other and from the clock.
fn volume() -> (Filesystem, u32) {
    let size = 32 * 1024 * 1024;
    let dev = Arc::new(MemDev {
        bytes: Mutex::new(vec![0u8; size as usize]),
        size,
    });
    fs_ext4::mkfs::format_filesystem(dev.as_ref(), None, Some([7; 16]), size, 4096).unwrap();
    let fs = Filesystem::mount_with_runtime(dev, Arc::new(Fixed)).unwrap();
    let ino = fs.apply_create("/f", 0o644).unwrap();
    fs.apply_utimens("/f", 1_000_000_000, 111, 1_100_000_000, 222)
        .unwrap();
    (fs, ino)
}

fn raw(fs: &Filesystem, ino: u32) -> Vec<u8> {
    fs.read_inode_verified(ino).unwrap().1
}

#[test]
fn a_nanosecond_count_of_one_billion_or_more_is_refused_and_writes_nothing() {
    let (fs, ino) = volume();
    let before = raw(&fs, ino);
    for nsec in [1_000_000_000u32, 1_073_741_821, 1 << 30, u32::MAX] {
        for (label, a, m) in [("atime", nsec, 0), ("mtime", 0, nsec)] {
            let got = fs.apply_utimens("/f", 0, a, 0, m);
            assert!(
                matches!(got, Err(Error::InvalidArgument(_))),
                "{label} nsec {nsec:#x}: expected InvalidArgument, got {got:?}"
            );
            assert_eq!(
                raw(&fs, ino),
                before,
                "{label} nsec {nsec:#x}: a refused call changed the inode"
            );
        }
    }
}

#[test]
fn the_largest_valid_nanosecond_count_round_trips() {
    let (fs, ino) = volume();
    fs.apply_utimens("/f", 5, 999_999_999, 6, 999_999_999)
        .unwrap();
    let (inode, _) = fs.read_inode_verified(ino).unwrap();
    assert_eq!((inode.atime, inode.atime_nsec), (5, 999_999_999));
    assert_eq!((inode.mtime, inode.mtime_nsec), (6, 999_999_999));
}

#[test]
fn utime_now_sets_that_field_to_the_mounts_clock() {
    let (fs, ino) = volume();
    // The seconds beside UTIME_NOW are ignored, as utimensat(2) says --
    // even one ext4 could not store.
    fs.apply_utimens("/f", i64::MAX, UTIME_NOW, 7, 8).unwrap();
    let (inode, _) = fs.read_inode_verified(ino).unwrap();
    assert_eq!((inode.atime, inode.atime_nsec), (NOW, 0), "atime");
    assert_eq!((inode.mtime, inode.mtime_nsec), (7, 8), "mtime");

    fs.apply_utimens("/f", 9, 10, 0, UTIME_NOW).unwrap();
    let (inode, _) = fs.read_inode_verified(ino).unwrap();
    assert_eq!((inode.atime, inode.atime_nsec), (9, 10), "atime");
    assert_eq!((inode.mtime, inode.mtime_nsec), (NOW, 0), "mtime");
}

#[test]
fn utime_omit_leaves_that_field_unchanged() {
    let (fs, ino) = volume();
    fs.apply_utimens("/f", i64::MAX, UTIME_OMIT, 7, 8).unwrap();
    let (inode, _) = fs.read_inode_verified(ino).unwrap();
    assert_eq!(
        (inode.atime, inode.atime_nsec),
        (1_000_000_000, 111),
        "atime"
    );
    assert_eq!((inode.mtime, inode.mtime_nsec), (7, 8), "mtime");

    fs.apply_utimens("/f", 9, 10, 0, UTIME_OMIT).unwrap();
    let (inode, _) = fs.read_inode_verified(ino).unwrap();
    assert_eq!((inode.atime, inode.atime_nsec), (9, 10), "atime");
    assert_eq!((inode.mtime, inode.mtime_nsec), (7, 8), "mtime");
}

/// Both fields omitted is a no-op, ctime included -- what Linux does.
#[test]
fn utime_omit_on_both_fields_writes_nothing() {
    let (fs, ino) = volume();
    let before = raw(&fs, ino);
    fs.apply_utimens("/f", 0, UTIME_OMIT, 0, UTIME_OMIT)
        .unwrap();
    assert_eq!(raw(&fs, ino), before);
}
