//! `set_flags` changes only the bits the kernel lets a caller change (#381),
//! judged by e2fsprogs rather than by this crate's own reader.
//!
//! A flag like `INDEX_FL` changes how the bytes an inode already holds are
//! read, so this crate reading its own write back would agree with itself.
//! `debugfs stat` reports the `i_flags` word the write left, and
//! `e2fsck -fn` judges the volume around it.

use fs_ext4::block_io::FileDevice;
use fs_ext4::inode::InodeFlags;
use fs_ext4::Filesystem;
use std::sync::Arc;

#[track_caller]
fn scratch() -> String {
    let src = fs_ext4_test_support::fixture(env!("CARGO_MANIFEST_DIR"), "ext4-basic.img");
    let dst = fs_ext4_test_support::temp_path!("fs_ext4_set_flags_{}.img", std::process::id());
    std::fs::copy(&src, &dst).unwrap_or_else(|e| panic!("copy {src} -> {dst}: {e}"));
    dst
}

fn flags(fs: &Filesystem, path: &str) -> u32 {
    let mut reader = |ino: u32| fs.read_inode_verified(ino).map(|(i, _)| i);
    let ino = fs_ext4::path::lookup(fs.dev.as_ref(), &fs.sb, &mut reader, path).unwrap();
    fs.read_inode_verified(ino).unwrap().0.flags
}

/// The `Flags: 0x...` word `debugfs stat` prints for `path`.
#[track_caller]
fn debugfs_flags(image: &str, path: &str) -> u32 {
    let out = fs_ext4_test_support::oracle("debugfs")
        .args(["-R", &format!("stat {path}"), image])
        .output();
    let text = String::from_utf8_lossy(&out.stdout);
    let word = text
        .split_whitespace()
        .skip_while(|w| *w != "Flags:")
        .nth(1)
        .unwrap_or_else(|| panic!("no Flags: in debugfs stat {path}:\n{text}"));
    u32::from_str_radix(word.trim_start_matches("0x"), 16)
        .unwrap_or_else(|e| panic!("debugfs Flags {word:?}: {e}"))
}

#[test]
fn user_modifiable_flags_land_and_index_on_a_linear_directory_is_refused() {
    let image = scratch();
    let fs = Filesystem::mount(Arc::new(FileDevice::open_rw(&image).unwrap())).unwrap();

    let file = flags(&fs, "/test.txt")
        | InodeFlags::IMMUTABLE.bits()
        | InodeFlags::NOATIME.bits()
        | InodeFlags::NODUMP.bits();
    fs.apply_set_flags("/test.txt", file).unwrap();

    let dir_before = flags(&fs, "/subdir");
    assert!(
        fs.apply_set_flags("/subdir", dir_before | InodeFlags::INDEX.bits())
            .is_err(),
        "INDEX_FL accepted on a linear directory"
    );
    let dir = dir_before | InodeFlags::NOATIME.bits() | 0x0001_0000; // DIRSYNC
    fs.apply_set_flags("/subdir", dir).unwrap();
    drop(fs);

    assert_eq!(debugfs_flags(&image, "/test.txt"), file, "/test.txt");
    assert_eq!(debugfs_flags(&image, "/subdir"), dir, "/subdir");
    fs_ext4_test_support::assert_e2fsck_clean(&image, "set_flags on a file and a directory");
    let _ = std::fs::remove_file(&image);
}
