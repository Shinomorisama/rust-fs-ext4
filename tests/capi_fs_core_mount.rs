//! The two `fs_core` mount entry points: `fs_ext4_mount_with_fs_core_device`
//! and its `_lazy` twin share one body and differ only in whether the
//! journal is replayed before the mount returns (#333). These pin that each
//! entry point refuses a null handle, mounts a real volume, and reaches its
//! own mount variant -- the one thing the shared body could get backwards.
//!
//! The volume is formatted in memory, so this needs no fixture and no VM.

use fs_ext4::capi::*;
use std::ffi::CStr;
use std::sync::{Arc, Mutex};

const VOL: u64 = 16 * 1024 * 1024;
/// POSIX `EINVAL`, the same on every platform this builds for.
const EINVAL: i32 = 22;

/// One in-memory byte buffer, readable as both this crate's device (to
/// format it) and `fs_core`'s (to hand it over as an `FsCoreDevice`).
struct Mem(Mutex<Vec<u8>>);

impl Mem {
    fn copy(&self, offset: u64, buf: &mut [u8]) -> bool {
        let b = self.0.lock().unwrap();
        let start = offset as usize;
        match b.get(start..start + buf.len()) {
            Some(src) => {
                buf.copy_from_slice(src);
                true
            }
            None => false,
        }
    }

    fn store(&self, offset: u64, buf: &[u8]) -> bool {
        let mut b = self.0.lock().unwrap();
        let start = offset as usize;
        match b.get_mut(start..start + buf.len()) {
            Some(dst) => {
                dst.copy_from_slice(buf);
                true
            }
            None => false,
        }
    }
}

impl fs_ext4::block_io::BlockDevice for Mem {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> fs_ext4::Result<()> {
        if self.copy(offset, buf) {
            Ok(())
        } else {
            Err(fs_ext4::Error::Corrupt("Mem: read past end"))
        }
    }
    fn size_bytes(&self) -> u64 {
        self.0.lock().unwrap().len() as u64
    }
    fn write_at(&self, offset: u64, buf: &[u8]) -> fs_ext4::Result<()> {
        if self.store(offset, buf) {
            Ok(())
        } else {
            Err(fs_ext4::Error::Corrupt("Mem: write past end"))
        }
    }
    fn is_writable(&self) -> bool {
        true
    }
}

impl fs_core::BlockRead for Mem {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> fs_core::Result<()> {
        if self.copy(offset, buf) {
            Ok(())
        } else {
            Err(fs_core::Error::ShortRead {
                offset,
                want: buf.len(),
                got: 0,
            })
        }
    }
    fn size_bytes(&self) -> u64 {
        self.0.lock().unwrap().len() as u64
    }
}

impl fs_core::BlockDevice for Mem {
    fn write_at(&self, offset: u64, buf: &[u8]) -> fs_core::Result<()> {
        if self.store(offset, buf) {
            Ok(())
        } else {
            Err(fs_core::Error::ShortRead {
                offset,
                want: buf.len(),
                got: 0,
            })
        }
    }
    fn is_writable(&self) -> bool {
        true
    }
}

fn handle(bytes: Vec<u8>) -> *mut fs_core::ffi::FsCoreDevice {
    fs_core::ffi::FsCoreDevice::into_handle(Arc::new(Mem(Mutex::new(bytes))))
}

fn formatted() -> Vec<u8> {
    let mem = Mem(Mutex::new(vec![0u8; VOL as usize]));
    fs_ext4::mkfs::format_filesystem(&mem, Some("fscore"), None, VOL, 4096).expect("format");
    mem.0.into_inner().unwrap()
}

fn last_error() -> String {
    unsafe { CStr::from_ptr(fs_ext4_last_error()) }
        .to_string_lossy()
        .into_owned()
}

type MountFn = unsafe extern "C" fn(*mut fs_core::ffi::FsCoreDevice) -> *mut fs_ext4_fs_t;

const ENTRY_POINTS: [(&str, MountFn, &str); 2] = [
    (
        "fs_ext4_mount_with_fs_core_device",
        fs_ext4_mount_with_fs_core_device,
        "mount via fs_core handle",
    ),
    (
        "fs_ext4_mount_with_fs_core_device_lazy",
        fs_ext4_mount_with_fs_core_device_lazy,
        "mount_lazy via fs_core handle",
    ),
];

#[test]
fn a_null_handle_is_refused_with_einval() {
    for (name, mount, _) in ENTRY_POINTS {
        let fs = unsafe { mount(std::ptr::null_mut()) };
        assert!(fs.is_null(), "{name} mounted a null handle");
        assert_eq!(fs_ext4_last_errno(), EINVAL, "{name}");
        assert_eq!(last_error(), "null fs_core handle", "{name}");
    }
}

#[test]
fn a_formatted_volume_mounts_through_either_entry_point() {
    for (name, mount, _) in ENTRY_POINTS {
        let h = handle(formatted());
        let fs = unsafe { mount(h) };
        assert!(!fs.is_null(), "{name}: {}", last_error());
        assert_eq!(fs_ext4_last_errno(), 0, "{name}");
        let mut attr: fs_ext4_attr_t = unsafe { std::mem::zeroed() };
        let rc = unsafe { fs_ext4_stat(fs, c"/".as_ptr(), &mut attr) };
        assert_eq!(rc, 0, "{name}: stat /: {}", last_error());
        assert_eq!(attr.inode, 2, "{name}");
        unsafe {
            fs_ext4_umount(fs);
            fs_core::ffi::fs_core_device_close(h);
        }
    }
}

/// Each entry point reaches its own mount variant: the error a refused
/// mount reports names the variant that refused it.
#[test]
fn each_entry_point_reaches_its_own_mount_variant() {
    for (name, mount, context) in ENTRY_POINTS {
        let h = handle(vec![0u8; VOL as usize]);
        let fs = unsafe { mount(h) };
        assert!(fs.is_null(), "{name} mounted a zeroed device");
        let err = last_error();
        assert!(
            err.starts_with(&format!("{context}: ")),
            "{name} reported {err:?}, not an error from `{context}`"
        );
        unsafe { fs_core::ffi::fs_core_device_close(h) };
    }
}
