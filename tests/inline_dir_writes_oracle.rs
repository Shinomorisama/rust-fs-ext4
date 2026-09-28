//! A mutation of an inline-data directory is refused and leaves the volume
//! as e2fsprogs made it (#382).
//!
//! An inline directory's `i_block` holds its parent's inode number and then
//! entries. Every directory writer read it as a block map, so renaming one
//! across parents wrote the new parent's number into the block its old
//! parent's inode number named. The directories here are made by `debugfs
//! mkdir` on a `mkfs.ext4 -O inline_data` volume — the independent layout,
//! not one this crate patched in — and `e2fsck -fn` judges what is left.
//! The e2fsprogs tools run in the harness VM; a test fails when it cannot
//! reach them.

use fs_ext4::block_io::FileDevice;
use fs_ext4::fs::Filesystem;
use fs_ext4::Error;
use std::sync::Arc;

fn mount(path: &str) -> Filesystem {
    Filesystem::mount(Arc::new(FileDevice::open_rw(path).unwrap())).unwrap()
}

fn inode_of(fs: &Filesystem, path: &str) -> fs_ext4::inode::Inode {
    let mut reader = |ino: u32| fs.read_inode_verified(ino).map(|(i, _)| i);
    let ino = fs_ext4::path::lookup(fs.dev.as_ref(), &fs.sb, &mut reader, path).unwrap();
    fs.read_inode_verified(ino).unwrap().0
}

#[test]
fn mutations_of_inline_directories_made_by_debugfs_are_refused() {
    let image = fs_ext4_test_support::temp_path!("fs_ext4_inline_dirs_{}.img", std::process::id());
    std::fs::File::create(&image)
        .and_then(|f| f.set_len(32 * 1024 * 1024))
        .unwrap();
    let made = fs_ext4_test_support::oracle("mkfs.ext4")
        .args(["-q", "-F", "-b", "4096", "-I", "256"])
        .args(["-O", "inline_data,metadata_csum,^has_journal"])
        .arg(&image)
        .output();
    assert!(
        made.status.success(),
        "mkfs.ext4: {}",
        String::from_utf8_lossy(&made.stderr)
    );
    fs_ext4_test_support::oracle("debugfs")
        .args(["-w", "-f", "-"])
        .arg(&image)
        .stdin("mkdir /a\nmkdir /b\n")
        .judged()
        .clean("debugfs mkdir");

    let fs = mount(&image);
    for dir in ["/a", "/b"] {
        assert!(
            inode_of(&fs, dir).has_inline_data(),
            "{dir}: debugfs made a block directory, so this test proves nothing"
        );
    }
    // A block directory of our own to move one into.
    fs.apply_mkdir("/c", 0o755).unwrap();
    drop(fs);
    fs_ext4_test_support::assert_e2fsck_clean(&image, "inline dirs: made");

    type Op = fn(&Filesystem) -> fs_ext4::Result<()>;
    let refused: [(&str, Op); 5] = [
        ("create inside", |fs| {
            fs.apply_create("/a/g", 0o644).map(drop)
        }),
        ("mkdir inside", |fs| fs.apply_mkdir("/a/h", 0o755).map(drop)),
        ("rename across parents", |fs| {
            fs.apply_rename("/a", "/c/a", false)
        }),
        ("rename into", |fs| fs.apply_rename("/c", "/b/c", false)),
        ("rmdir", |fs| fs.apply_rmdir("/b")),
    ];
    for (name, op) in refused {
        let fs = mount(&image);
        let r = op(&fs);
        drop(fs);
        assert!(
            matches!(r, Err(Error::Unsupported(_))),
            "{name} an inline directory: {r:?}"
        );
        fs_ext4_test_support::assert_e2fsck_clean(&image, &format!("inline dirs: {name}"));
    }

    // Renaming within one parent touches only the parent's entries, which
    // is a block directory here: it still goes through.
    let fs = mount(&image);
    fs.apply_rename("/a", "/a2", false).unwrap();
    assert!(inode_of(&fs, "/a2").has_inline_data());
    drop(fs);
    fs_ext4_test_support::assert_e2fsck_clean(&image, "inline dirs: renamed in place");
    let _ = std::fs::remove_file(&image);
}
