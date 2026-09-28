//! A volume written only through the inode-addressed entry points (#372)
//! is one e2fsck accepts.
//!
//! `tests/inode_api.rs` proves each inode entry point writes the same bytes
//! as its path twin; this proves those bytes are a valid filesystem by a
//! judge that is not this crate. The volume is made by `mkfs.ext4` (journal,
//! `metadata_csum`, htree) and then touched by nothing but `lookup_at`,
//! the `_at` namespace operations and the `_ino` data and attribute
//! operations: hard links, cross-directory renames of files and
//! directories, a replacing rename, a directory grown past one block,
//! fast and slow symlinks, a FIFO, positional writes, both truncate
//! directions, and removal of files and directories. e2fsck runs in the
//! harness VM and is read through its verdict helper.

use fs_ext4::block_io::FileDevice;
use fs_ext4::{Filesystem, InodeRef};
use std::sync::Arc;

const ROOT: u32 = 2;

fn mkfs(tag: &str, features: &str) -> String {
    let path =
        fs_ext4_test_support::temp_path!("fs_ext4_inode_api_{tag}_{}.img", std::process::id());
    std::fs::File::create(&path)
        .and_then(|f| f.set_len(64 * 1024 * 1024))
        .unwrap();
    let out = fs_ext4_test_support::oracle("mkfs.ext4")
        .args(["-q", "-F", "-b", "1024", "-O", features])
        .arg(&path)
        .output();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    path
}

/// Every write below names its target by inode, never by path.
fn write_through_inodes(fs: &Filesystem) {
    let a = fs.apply_mkdir_at(ROOT, b"a", 0o755).unwrap();
    let b = fs.apply_mkdir_at(ROOT, b"b", 0o755).unwrap();
    let sub = fs.apply_mkdir_at(a, b"sub", 0o700).unwrap();

    let f = fs.apply_create_at(a, b"f", 0o644).unwrap();
    let f = InodeRef::new(f, fs.stat_ino(f).unwrap().generation);
    fs.apply_pwrite_ino(f, 0, &vec![0xA5; 70_000]).unwrap();
    fs.apply_pwrite_ino(f, 200_000, b"sparse tail").unwrap();
    fs.apply_link_at(f, b, b"g").unwrap();

    fs.apply_symlink_at(a, b"fast", b"f").unwrap();
    fs.apply_symlink_at(a, b"slow", "t".repeat(300).as_bytes())
        .unwrap();
    fs.apply_mknod_at(sub, b"fifo", 0o010644, 0, 0).unwrap();
    fs.apply_create_at(sub, b"caf\xe9", 0o600).unwrap();

    // A directory moved across parents, then renamed in place.
    fs.apply_rename_at(a, b"sub", b, b"sub", false).unwrap();
    fs.apply_rename_at(ROOT, b"a", ROOT, b"c", false).unwrap();

    // Grow a directory well past one block, then thin it out.
    let big = fs.apply_mkdir_at(ROOT, b"big", 0o755).unwrap();
    for i in 0..300 {
        let name = format!("entry-with-a-reasonably-long-name-{i:04}");
        fs.apply_create_at(big, name.as_bytes(), 0o644).unwrap();
    }
    for i in (0..300).step_by(2) {
        let name = format!("entry-with-a-reasonably-long-name-{i:04}");
        fs.apply_unlink_at(big, name.as_bytes()).unwrap();
    }

    // Replace a file with another by rename; the victim is freed.
    let victim = fs.apply_create_at(b, b"victim", 0o644).unwrap();
    fs.apply_pwrite_ino(victim, 0, &[1u8; 5000]).unwrap();
    fs.apply_rename_at(
        big,
        b"entry-with-a-reasonably-long-name-0001",
        b,
        b"victim",
        true,
    )
    .unwrap();

    // Attributes and sizes.
    fs.apply_chmod_ino(f, 0o600).unwrap();
    fs.apply_chown_ino(f, 1000, 1000).unwrap();
    fs.apply_utimens_ino(f, 1_600_000_000, 1, 1_600_000_001, 2)
        .unwrap();
    fs.apply_truncate_ino(f, 1000).unwrap();
    fs.apply_truncate_ino(f, 50_000).unwrap();

    // Remove a name of the linked file, then a whole subtree.
    let c = fs.lookup_at(ROOT, b"c").unwrap();
    fs.apply_unlink_at(c, b"f").unwrap();
    assert_eq!(fs.stat_ino(f).unwrap().links_count, 1);
    let sub = fs.lookup_at(b, b"sub").unwrap();
    fs.apply_unlink_at(sub, b"fifo").unwrap();
    fs.apply_unlink_at(sub, b"caf\xe9").unwrap();
    fs.apply_rmdir_at(b, b"sub").unwrap();
}

#[test]
fn a_volume_written_through_inodes_is_clean() {
    for (tag, features) in [("csum", "metadata_csum"), ("nocsum", "^metadata_csum")] {
        let path = mkfs(tag, features);
        {
            let fs = Filesystem::mount(Arc::new(FileDevice::open_rw(&path).unwrap())).unwrap();
            write_through_inodes(&fs);
        }
        fs_ext4_test_support::assert_e2fsck_clean(&path, &format!("inode API, {tag}"));
        std::fs::remove_file(&path).ok();
    }
}
