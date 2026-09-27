//! With eight inodes a group, `s_first_ino` 11 reaches into group 1, and
//! a file created there is one `e2fsck` accepts (#327).
//!
//! The allocator's reserved-inode floor applied to group 0 only, so on a
//! geometry where `s_first_ino > inodes_per_group + 1` group 1 could hand
//! out inodes 9 and 10. On an image `mke2fs` wrote those bits are set, so
//! the allocation that matters here is the one the fixed floor makes: the
//! first inode this crate gives out in group 1 must be at or past 11, and
//! the volume must check clean afterwards. Fails without e2fsprogs (they
//! run in the harness VM).

use fs_ext4::block_io::{BlockDevice, FileDevice};
use fs_ext4::fs::Filesystem;

use std::sync::Arc;

#[test]
fn a_file_created_in_group_1_past_the_reserved_inodes_checks_clean() {
    let dir =
        fs_ext4_test_support::temp_dir().join(format!("ext4-reserved-g1-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let img = dir.join("r.img");
    std::fs::File::create(&img)
        .unwrap()
        .set_len(4096 * 1024)
        .unwrap();
    // Four 1024-block groups of eight inodes each: group 0 is inodes
    // 1..=8, all reserved, and group 1 starts at 9.
    let made = fs_ext4_test_support::oracle("mkfs.ext4")
        .args([
            "-q", "-F", "-b", "1024", "-g", "1024", "-N", "32", "-I", "256",
        ])
        .arg(&img)
        .arg("4096")
        .output();
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
    let path = img.to_str().unwrap();

    {
        let dev = FileDevice::open_rw(path).expect("open_rw");
        let fs = Filesystem::mount(Arc::new(dev) as Arc<dyn BlockDevice>)
            .unwrap_or_else(|e| panic!("a volume mke2fs made must mount: {e:?}"));
        // THE FIXTURE'S OWN SHAPE FIRST: a test on a geometry where the
        // reserved range ends inside group 0 asserts nothing about #327.
        assert_eq!(
            (fs.sb.inodes_per_group, fs.sb.first_inode),
            (8, 11),
            "mke2fs did not make the geometry this test is about"
        );
        assert_eq!(fs.groups[0].free_inodes_count, 0, "group 0 is all reserved");

        // lost+found is inode 11 and, where mke2fs makes one, the orphan
        // file the next: which inode is free first depends on the
        // e2fsprogs version, but it is in group 1 and never 9 or 10.
        let ino = fs.apply_create("/f", 0o644).expect("create");
        assert!(
            ino >= fs.sb.first_inode,
            "inode {ino} is below s_first_ino {}, in group 1",
            fs.sb.first_inode
        );
        assert_eq!((ino - 1) / 8, 1, "inode {ino} is not in group 1");
        fs.apply_pwrite("/f", 0, b"past the reserved inodes")
            .expect("write");
        fs.finish().expect("unmount");
    }

    fs_ext4_test_support::assert_e2fsck_clean(path, "reserved-g1");
    let _ = std::fs::remove_dir_all(&dir);
}
