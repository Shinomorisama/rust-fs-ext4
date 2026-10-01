//! A freshly formatted volume has `/lost+found`, laid out the way `mke2fs`
//! lays it out (#443).
//!
//! `e2fsck` reconnects an orphaned inode into `/lost+found`, and it looks for
//! the directory rather than making one: when there is none it has to allocate
//! a directory on the volume it is in the middle of repairing. `mke2fs`
//! therefore always creates it, as inode 11 (the first inode past the reserved
//! ones), mode 0700, and pre-sized so that reconnecting does not need to grow
//! it. This formatter made a root holding only `.` and `..`, which `e2fsck -fn`
//! accepts, so no oracle test had noticed.
//!
//! This file reads the result back through the driver, for every flavour and
//! both layout paths (the single-group one below 2 KiB blocks, and the
//! multi-group one). `tests/mkfs_lost_found_oracle.rs` holds the same volumes
//! to what `debugfs` reports for a volume `mke2fs` made, which is where the
//! sizes below come from.

use fs_ext4::block_io::{BlockDevice, FileDevice};
use fs_ext4::dir::DirEntryType;
use fs_ext4::features::FsFlavor;
use fs_ext4::fs::Filesystem;
use fs_ext4::mkfs;
use std::sync::Arc;

const LOST_FOUND_INO: u32 = 11;

/// `i_size` of `/lost+found` on a volume `mke2fs` 1.47 made, by block size:
/// 16 KiB, but never fewer than two blocks, and at 1 KiB blocks twelve (the
/// directory needs no indirect block). Measured with `debugfs -R 'stat
/// /lost+found'`; the oracle test compares against `mke2fs` directly.
fn mke2fs_lost_found_size(block_size: u32) -> u64 {
    match block_size {
        1024 => 12 * 1024,
        2048 | 4096 | 8192 => 16 * 1024,
        bs => 2 * u64::from(bs),
    }
}

fn format(tag: &str, size: u64, block_size: u32, flavor: FsFlavor) -> String {
    let path =
        fs_ext4_test_support::temp_path!("fs_ext4_lost_found_{tag}_{}.img", std::process::id());
    std::fs::File::create(&path)
        .and_then(|f| f.set_len(size))
        .unwrap_or_else(|e| panic!("create {path}: {e}"));
    let dev = FileDevice::open_rw(&path).expect("open_rw");
    mkfs::format_filesystem_with_flavor(&dev, Some("LF"), None, size, block_size, flavor)
        .expect("format");
    dev.flush().expect("flush");
    path
}

fn check(tag: &str, size: u64, block_size: u32, flavor: FsFlavor) {
    let path = format(tag, size, block_size, flavor);
    {
        let fs = Filesystem::mount(Arc::new(FileDevice::open(&path).expect("ro"))).expect("mount");

        let ino = fs
            .lookup_path_bytes(b"/lost+found")
            .unwrap_or_else(|e| panic!("[{tag}] /lost+found: {e}"));
        assert_eq!(ino, LOST_FOUND_INO, "[{tag}] /lost+found's inode");

        let (lf, _) = fs.read_inode_verified(ino).expect("lost+found verifies");
        assert!(lf.is_dir(), "[{tag}] /lost+found is a directory");
        assert_eq!(lf.mode & 0o7777, 0o700, "[{tag}] /lost+found's mode");
        assert_eq!(lf.links_count, 2, "[{tag}] /lost+found's links");
        assert_eq!(
            lf.size,
            mke2fs_lost_found_size(block_size),
            "[{tag}] /lost+found's size"
        );
        assert_eq!(
            lf.blocks * 512,
            lf.size,
            "[{tag}] /lost+found's i_blocks covers its size and nothing else"
        );

        let (root, _) = fs.read_inode_verified(2).expect("root verifies");
        assert_eq!(
            root.links_count, 3,
            "[{tag}] root links: `.`, `..` and lost+found's `..`"
        );
        for (dir, name, want) in [
            (LOST_FOUND_INO, &b"."[..], LOST_FOUND_INO),
            (LOST_FOUND_INO, b"..", 2),
            (2, b"lost+found", LOST_FOUND_INO),
        ] {
            let got = fs.lookup_at(dir, name).unwrap_or_else(|e| {
                panic!("[{tag}] {} in {dir}: {e}", String::from_utf8_lossy(name))
            });
            assert_eq!(got, want, "[{tag}] {}", String::from_utf8_lossy(name));
        }

        // Nothing else in either directory. Listed only where the volume
        // uses extents: listing a directory mapped by `i_block` pointers is
        // refused by the driver, for any directory (the oracle test lists
        // the ext2 and ext3 volumes through debugfs instead).
        if flavor == FsFlavor::Ext4 {
            let live = |ino: u32| -> Vec<(Vec<u8>, u32, DirEntryType)> {
                fs.read_dir_ino(ino)
                    .expect("list")
                    .into_iter()
                    .filter(|e| e.inode != 0)
                    .map(|e| (e.name, e.inode, e.file_type))
                    .collect()
            };
            let dir = DirEntryType::Directory;
            assert_eq!(
                live(LOST_FOUND_INO),
                vec![
                    (b".".to_vec(), LOST_FOUND_INO, dir),
                    (b"..".to_vec(), 2, dir)
                ],
                "[{tag}] /lost+found holds `.` and `..` and nothing else"
            );
            assert_eq!(
                live(2),
                vec![
                    (b".".to_vec(), 2, dir),
                    (b"..".to_vec(), 2, dir),
                    (b"lost+found".to_vec(), LOST_FOUND_INO, dir),
                ],
                "[{tag}] the root's entries"
            );
        }

        let report = fs_ext4::fsck::audit(&fs, u32::MAX, u32::MAX).expect("audit");
        assert!(
            report.is_clean(),
            "[{tag}] structural anomalies: {:?}",
            report.anomalies
        );
    }
    let _ = std::fs::remove_file(&path);
}

#[test]
fn extents_1k_blocks_single_group() {
    check("extents_1k", 8 << 20, 1024, FsFlavor::Ext4);
}

#[test]
fn ext2_1k_blocks() {
    check("ext2_1k", 8 << 20, 1024, FsFlavor::Ext2);
}

#[test]
fn ext3_1k_blocks() {
    check("ext3_1k", 8 << 20, 1024, FsFlavor::Ext3);
}

#[test]
fn extents_2k_blocks() {
    check("extents_2k", 16 << 20, 2048, FsFlavor::Ext4);
}

#[test]
fn extents_4k_blocks() {
    check("extents_4k", 32 << 20, 4096, FsFlavor::Ext4);
}

#[test]
fn extents_4k_blocks_multi_group() {
    check("extents_4k_mg3", 320 << 20, 4096, FsFlavor::Ext4);
}

#[test]
fn extents_64k_blocks() {
    check("extents_64k", 64 << 20, 65_536, FsFlavor::Ext4);
}
