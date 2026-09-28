//! An attribute has one copy, wherever it lives (#377).
//!
//! A value that crosses the in-inode capacity between two sets moves:
//! from the inode to the external block when it grows, and back when it
//! shrinks. The move used to leave the old copy where it was, so the name
//! was listed twice and the first copy -- the stale one -- was the one
//! read. The kernel (`ext4_xattr_set_handle`) removes the other copy in
//! both directions.
//!
//! The driver writes; `debugfs ea_list`, which walks the in-inode area
//! and the block independently, lists each copy it finds, and `e2fsck -fn`
//! judges the volume. Both run in the harness VM.

use fs_ext4::block_io::FileDevice;
use fs_ext4::fs::Filesystem;
use std::sync::Arc;

fn mkfs(tag: &str) -> String {
    let path =
        fs_ext4_test_support::temp_path!("fs_ext4_xattr_one_copy_{tag}_{}.img", std::process::id());
    std::fs::File::create(&path)
        .and_then(|f| f.set_len(64 * 1024 * 1024))
        .unwrap();
    let out = fs_ext4_test_support::oracle("mkfs.ext4")
        .args(["-q", "-F", "-b", "4096", "-I", "256"])
        .arg(&path)
        .output();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    path
}

/// Every line of `debugfs -R 'ea_list <file>'` that lists `name`, as the
/// tool prints it: `  <name> (<length>)`, then the value when it is short.
fn listed(image: &str, file: &str, name: &str) -> Vec<String> {
    let out = fs_ext4_test_support::oracle("debugfs")
        .arg("-R")
        .arg(format!("ea_list {file}"))
        .arg(image)
        .output();
    assert!(
        out.status.success(),
        "debugfs ea_list {file}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let prefix = format!("{name} (");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with(&prefix))
        .map(str::to_string)
        .collect()
}

#[test]
fn an_attribute_that_moves_between_the_inode_and_the_block_has_one_copy() {
    let image = mkfs("move");
    {
        let fs = Filesystem::mount(Arc::new(FileDevice::open_rw(&image).unwrap())).unwrap();
        // Grows past the inode: in-inode, then external.
        fs.apply_create("/grown", 0o644).unwrap();
        fs.apply_setxattr("/grown", "user.a", &[b'a'; 40]).unwrap();
        fs.apply_setxattr("/grown", "user.a", &[b'b'; 200]).unwrap();
        // Shrinks into the inode: external, then in-inode.
        fs.apply_create("/shrunk", 0o644).unwrap();
        fs.apply_setxattr("/shrunk", "user.a", &[b'c'; 200])
            .unwrap();
        fs.apply_setxattr("/shrunk", "user.a", &[b'd'; 8]).unwrap();
        // Shrinks into the inode, then is removed: nothing may come back.
        fs.apply_create("/removed", 0o644).unwrap();
        fs.apply_setxattr("/removed", "user.a", &[b'e'; 200])
            .unwrap();
        fs.apply_setxattr("/removed", "user.a", &[b'f'; 8]).unwrap();
        fs.apply_removexattr("/removed", "user.a").unwrap();
    }

    let grown = listed(&image, "/grown", "user.a");
    assert!(
        grown.len() == 1 && grown[0].starts_with("user.a (200)"),
        "/grown: debugfs lists {grown:?}; want one copy of 200 bytes"
    );
    let shrunk = listed(&image, "/shrunk", "user.a");
    assert!(
        shrunk.len() == 1 && shrunk[0].starts_with("user.a (8)"),
        "/shrunk: debugfs lists {shrunk:?}; want one copy of 8 bytes"
    );
    let removed = listed(&image, "/removed", "user.a");
    assert!(
        removed.is_empty(),
        "/removed: debugfs still lists {removed:?}"
    );

    fs_ext4_test_support::assert_e2fsck_clean(&image, "xattr moved between inode and block");
    let _ = std::fs::remove_file(&image);
}
