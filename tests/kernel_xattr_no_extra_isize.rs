//! An xattr set on an inode with `i_extra_isize = 0` is one the kernel
//! can read (#380).
//!
//! `i_extra_isize = 0` is what `ext2.ko`, older kernels and 256-byte-inode
//! `mke2fs -t ext2` volumes leave on an inode. The kernel parses no in-inode
//! xattr area there: it reads 0x80.. as `i_extra_isize`, `i_checksum_hi`
//! and the `i_*_extra` timestamp words. The driver used to put the area at
//! 0x80 anyway, where the kernel does not look -- and on `metadata_csum`
//! then stored a checksum high half of 0 over the area's magic.
//!
//! `debugfs` (e2fsprogs, not this crate) gives the inode
//! `i_extra_isize = 0`, the driver sets the attribute, and the real kernel,
//! loop-mounting the image in the harness VM, must read it back; `e2fsck
//! -fn` judges the volume.

use fs_ext4::block_io::FileDevice;
use fs_ext4::fs::Filesystem;
use fs_ext4_test_support::{guest_kernel_report, oracle, temp_path};
use std::sync::Arc;

#[test]
fn the_kernel_reads_an_xattr_set_on_an_inode_with_no_extra_isize() {
    let image = temp_path!("fs_ext4_kernel_no_extra_isize_{}.img", std::process::id());
    let _ = std::fs::remove_file(&image);
    std::fs::File::create(&image)
        .and_then(|f| f.set_len(64 * 1024 * 1024))
        .unwrap();
    let out = oracle("mkfs.ext4")
        .args(["-q", "-F", "-b", "4096", "-I", "256", "-O", "metadata_csum"])
        .arg(&image)
        .output();
    assert!(
        out.status.success(),
        "mkfs.ext4: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    oracle("debugfs")
        .args(["-w", "-f", "-"])
        .arg(&image)
        .stdin("write /dev/null f\nsif /f extra_isize 0\n")
        .judged()
        .clean("debugfs sif extra_isize 0");

    {
        let fs = Filesystem::mount(Arc::new(FileDevice::open_rw(&image).unwrap()))
            .expect("mount the inode debugfs left");
        fs.apply_setxattr("/f", "user.colour", b"amber")
            .expect("setxattr");
    }

    let report = guest_kernel_report(&image, "no extra_isize");
    assert_eq!(
        report
            .get(&("xattrs".to_string(), "f".to_string()))
            .map(String::as_str),
        Some("user.colour=amber"),
        "the kernel does not see the attribute the driver set"
    );
    fs_ext4_test_support::assert_e2fsck_clean(&image, "xattr on an inode with no extra_isize");
    let _ = std::fs::remove_file(&image);
}
