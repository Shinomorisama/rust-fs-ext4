//! `fs_ext4_set_volume_label` (#447): the label it sets is the one
//! `fs_ext4_get_volume_info` reports, on the same handle and after a
//! remount; a label longer than 16 bytes is EINVAL and a read-only mount is
//! EROFS, and neither changes the label.

use fs_ext4::block_io::{BlockDevice, FileDevice};
use fs_ext4::capi::*;
use fs_ext4::mkfs;
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;

const EINVAL: i32 = 22;
const EROFS: i32 = 30;

fn formatted(tag: &str) -> CString {
    let path =
        fs_ext4_test_support::temp_path!("fs_ext4_capi_label_{tag}_{}.img", std::process::id());
    let size = 32 << 20;
    std::fs::File::create(&path)
        .and_then(|f| f.set_len(size))
        .unwrap_or_else(|e| panic!("create {path}: {e}"));
    let dev = FileDevice::open_rw(&path).expect("open_rw");
    mkfs::format_filesystem(&dev, Some("before"), None, size, 4096).expect("format");
    dev.flush().expect("flush");
    CString::new(path).unwrap()
}

fn label_of(fs: *mut fs_ext4_fs_t) -> String {
    let mut info = MaybeUninit::<fs_ext4_volume_info_t>::uninit();
    assert_eq!(unsafe { fs_ext4_get_volume_info(fs, info.as_mut_ptr()) }, 0);
    let info = unsafe { info.assume_init() };
    unsafe { CStr::from_ptr(info.volume_name.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

fn label_after_remount(img: &CString) -> String {
    let fs = unsafe { fs_ext4_mount(img.as_ptr()) };
    assert!(!fs.is_null());
    let label = label_of(fs);
    unsafe { fs_ext4_umount(fs) };
    label
}

#[test]
fn the_label_set_is_the_label_reported() {
    let img = formatted("set");
    let fs = unsafe { fs_ext4_mount_rw(img.as_ptr()) };
    assert!(!fs.is_null());
    assert_eq!(label_of(fs), "before");
    let after = CString::new("after").unwrap();
    assert_eq!(unsafe { fs_ext4_set_volume_label(fs, after.as_ptr()) }, 0);
    assert_eq!(label_of(fs), "after", "the same handle");

    let long = CString::new("0123456789abcdefg").unwrap();
    assert_eq!(unsafe { fs_ext4_set_volume_label(fs, long.as_ptr()) }, -1);
    assert_eq!(fs_ext4_last_errno(), EINVAL);
    assert_eq!(
        unsafe { fs_ext4_set_volume_label(fs, std::ptr::null()) },
        -1
    );
    assert_eq!(fs_ext4_last_errno(), EINVAL);
    assert_eq!(label_of(fs), "after", "a refused label changed it");
    unsafe { fs_ext4_umount(fs) };

    assert_eq!(label_after_remount(&img), "after");
    let _ = std::fs::remove_file(img.to_str().unwrap());
}

#[test]
fn a_read_only_mount_is_erofs() {
    let img = formatted("ro");
    let fs = unsafe { fs_ext4_mount(img.as_ptr()) };
    assert!(!fs.is_null());
    let x = CString::new("x").unwrap();
    assert_eq!(unsafe { fs_ext4_set_volume_label(fs, x.as_ptr()) }, -1);
    assert_eq!(fs_ext4_last_errno(), EROFS);
    unsafe { fs_ext4_umount(fs) };
    assert_eq!(label_after_remount(&img), "before");
    let _ = std::fs::remove_file(img.to_str().unwrap());
}
