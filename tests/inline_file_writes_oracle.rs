//! A content write to an inline-data file is refused and leaves the file and
//! the volume as the kernel made them (#383).
//!
//! An inline file's `i_block` holds its first 60 bytes. Replacing its
//! content read those bytes as block pointers and freed the blocks they
//! named; growing it patched only `i_size`. The files here are the kernel's
//! own, in `ext4-inline.img`; `debugfs cat` reads them back and
//! `e2fsck -fn` judges the volume. The e2fsprogs tools run in the harness
//! VM; a test fails when it cannot reach them.

use fs_ext4::block_io::FileDevice;
use fs_ext4::fs::Filesystem;
use fs_ext4::Error;
use std::sync::Arc;

fn inode_of(fs: &Filesystem, path: &str) -> (u32, fs_ext4::inode::Inode) {
    let mut reader = |ino: u32| fs.read_inode_verified(ino).map(|(i, _)| i);
    let ino = fs_ext4::path::lookup(fs.dev.as_ref(), &fs.sb, &mut reader, path).unwrap();
    (ino, fs.read_inode_verified(ino).unwrap().0)
}

#[track_caller]
fn debugfs_cat(image: &str, path: &str) -> Vec<u8> {
    let out = fs_ext4_test_support::oracle("debugfs")
        .args(["-R", &format!("cat {path}"), image])
        .output();
    assert!(out.status.success(), "debugfs cat {path}");
    out.stdout
}

#[test]
fn writes_to_the_kernels_inline_files_are_refused_and_leave_them_whole() {
    let src = fs_ext4_test_support::fixture(env!("CARGO_MANIFEST_DIR"), "ext4-inline.img");
    let image =
        fs_ext4_test_support::temp_path!("fs_ext4_inline_writes_{}.img", std::process::id());
    std::fs::copy(&src, &image).unwrap_or_else(|e| panic!("copy {src} -> {image}: {e}"));

    let files: [(&str, Vec<u8>); 2] = [
        ("/tiny.txt", b"tiny inline\n".to_vec()),
        ("/medium.txt", vec![b'A'; 100]),
    ];
    for (path, content) in &files {
        let fs = Filesystem::mount(Arc::new(FileDevice::open_rw(&image).unwrap())).unwrap();
        let (ino, inode) = inode_of(&fs, path);
        assert!(
            inode.has_inline_data(),
            "{path}: not inline, so this test proves nothing"
        );
        let results = [
            (
                "replace",
                fs.apply_replace_file_content(path, b"new content")
                    .map(drop),
            ),
            ("grow", fs.apply_truncate_grow(ino, 8192)),
            ("shrink", fs.apply_truncate_shrink(ino, 1)),
            ("pwrite", fs.apply_pwrite(path, 1, b"x").map(drop)),
        ];
        drop(fs);
        for (name, r) in results {
            assert!(
                matches!(r, Err(Error::Unsupported(_))),
                "{name} of {path}: {r:?}"
            );
        }
        assert_eq!(&debugfs_cat(&image, path), content, "{path}: debugfs cat");
    }
    fs_ext4_test_support::assert_e2fsck_clean(&image, "refused writes to inline files");
    let _ = std::fs::remove_file(&image);
}
