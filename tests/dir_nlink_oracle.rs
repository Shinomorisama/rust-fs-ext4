//! A directory with more subdirectories than `i_links_count` can count keeps
//! the `DIR_NLINK` value e2fsck expects (#385).
//!
//! On a `dir_nlink` volume a directory past `EXT4_LINK_MAX` (65000) links is
//! written with a count of 1, "too many to count": the kernel pins it there
//! and e2fsck pass 4 expects exactly that. The driver treated the 1 as a
//! literal count, so a mkdir under such a directory wrote 2 and an rmdir
//! wrote 0 -- which Linux then refuses to load.
//!
//! The volume comes from the real toolchain: `mkfs.ext4` makes it and the
//! Linux kernel, through a read-write mount in the harness VM, gives `/many`
//! 65001 subdirectories -- so the parent's count of 1 is the kernel's own.
//! (Built instead by `debugfs`, one linear lookup per subdirectory, the
//! fixture took 25 minutes of CI.) `e2fsck -fn` accepts that, the driver
//! adds and removes subdirectories, and `e2fsck -fn` must still find
//! nothing. Fails without the VM, where the kernel and e2fsprogs run.

use fs_ext4::block_io::FileDevice;
use fs_ext4::fs::Filesystem;
use std::sync::Arc;

/// Past EXT4_LINK_MAX: `.`, the entry in `/`, and one `..` per subdirectory
/// come to 65003.
const SUBDIRS: usize = 65001;

fn resolve(fs: &Filesystem, path: &str) -> fs_ext4::Result<u32> {
    let mut reader = |ino: u32| fs.read_inode_verified(ino).map(|(inode, _)| inode);
    fs_ext4::path::lookup(fs.dev.as_ref(), &fs.sb, &mut reader, path)
}

fn links(fs: &Filesystem, path: &str) -> u16 {
    let ino = resolve(fs, path).expect("resolve");
    fs.read_inode_verified(ino).expect("inode").0.links_count
}

/// A fresh image whose `/many` holds [`SUBDIRS`] empty subdirectories.
fn uncountable_parent_volume() -> String {
    let image = fs_ext4_test_support::temp_path!("fs_ext4_dir_nlink_{}.img", std::process::id());
    std::fs::File::create(&image)
        .and_then(|f| f.set_len(160 * 1024 * 1024))
        .unwrap();
    // 1 KiB blocks: each subdirectory costs one.
    let out = fs_ext4_test_support::oracle("mkfs.ext4")
        .args([
            "-q",
            "-F",
            "-b",
            "1024",
            "-N",
            "70000",
            "-O",
            "dir_nlink",
            &image,
        ])
        .output();
    assert_eq!(
        out.status.code(),
        Some(0),
        "mkfs.ext4: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let out = fs_ext4_test_support::guest_kernel_write(
        &image,
        &format!(
            r#"
mkdir "$MNT/many"
cd "$MNT/many"
seq -f 'd%05g' 0 {last} | xargs mkdir
sync
"#,
            last = SUBDIRS - 1
        ),
    );
    assert!(
        out.status.success(),
        "the kernel could not make the subdirectories:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    image
}

#[test]
fn a_dir_nlink_count_of_one_survives_mkdir_and_rmdir_under_e2fsck() {
    let image = uncountable_parent_volume();
    fs_ext4_test_support::oracle("e2fsck")
        .args(["-fn", &image])
        .judged()
        .clean("the fixture");
    {
        let fs = Filesystem::mount(Arc::new(FileDevice::open_rw(&image).unwrap())).unwrap();
        assert_eq!(
            links(&fs, "/many"),
            1,
            "fixture: the kernel did not leave /many at the DIR_NLINK count"
        );
        fs.apply_mkdir("/many/added", 0o755).expect("mkdir");
    }
    fs_ext4_test_support::oracle("e2fsck")
        .args(["-fn", &image])
        .judged()
        .clean("after mkdir under a DIR_NLINK parent");
    {
        let fs = Filesystem::mount(Arc::new(FileDevice::open_rw(&image).unwrap())).unwrap();
        fs.apply_rmdir("/many/added").expect("rmdir");
        fs.apply_rmdir("/many/d00000").expect("rmdir");
    }
    fs_ext4_test_support::oracle("e2fsck")
        .args(["-fn", &image])
        .judged()
        .clean("after rmdir under a DIR_NLINK parent");
    let fs = Filesystem::mount(Arc::new(FileDevice::open(&image).unwrap())).unwrap();
    assert_eq!(links(&fs, "/many"), 1, "the count stayed uncountable");
    drop(fs);
    let _ = std::fs::remove_file(&image);
}
