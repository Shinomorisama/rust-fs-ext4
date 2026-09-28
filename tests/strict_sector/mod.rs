//! The strict-sector device shared by the #373 tests:
//! `capi_callback_alignment.rs` (unit tier) and
//! `capi_callback_alignment_oracle.rs` (the same scenario, read back by the
//! independent checker in the harness VM).
//!
//! A host block resource that accepts only sector-aligned I/O refuses any
//! request whose offset or length is not a multiple of its sector size. The
//! callbacks here model that: they return -1 for an unaligned request while
//! `strict` is on, and record every request they were asked for, so a test
//! can prove the driver never sent one.
#![allow(dead_code)]

use fs_ext4::capi::*;
use std::ffi::CString;
use std::os::raw::{c_int, c_void};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

pub const SECTOR: u64 = 4096;
pub const IMAGE_BYTES: u64 = 32 * 1024 * 1024;
pub const FILE_BYTES: usize = 1024 * 1024;

pub struct StrictDev {
    pub bytes: Mutex<Vec<u8>>,
    /// Refuse unaligned requests (off while the byte-granular mkfs runs).
    pub strict: AtomicBool,
    pub requests: AtomicUsize,
    /// Every unaligned request seen: (is_write, offset, length).
    pub unaligned: Mutex<Vec<(bool, u64, u64)>>,
}

impl StrictDev {
    pub fn new() -> Box<Self> {
        Box::new(StrictDev {
            bytes: Mutex::new(vec![0u8; IMAGE_BYTES as usize]),
            strict: AtomicBool::new(false),
            requests: AtomicUsize::new(0),
            unaligned: Mutex::new(Vec::new()),
        })
    }

    /// Record the request; `false` when a strict device must refuse it.
    fn admit(&self, is_write: bool, offset: u64, length: u64) -> bool {
        self.requests.fetch_add(1, Ordering::Relaxed);
        if offset.is_multiple_of(SECTOR) && length.is_multiple_of(SECTOR) {
            return true;
        }
        self.unaligned
            .lock()
            .unwrap()
            .push((is_write, offset, length));
        !self.strict.load(Ordering::Relaxed)
    }

    pub fn cfg(&self, block_size: u32) -> fs_ext4_blockdev_cfg_t {
        fs_ext4_blockdev_cfg_t {
            read: Some(read_cb),
            context: self as *const StrictDev as *mut c_void,
            size_bytes: IMAGE_BYTES,
            block_size,
            write: Some(write_cb),
            flush: None,
        }
    }

    /// Format the image through the byte-granular path, then turn strict on.
    pub fn formatted() -> Box<Self> {
        let dev = Self::new();
        let cfg = dev.cfg(4096); // mkfs: the filesystem block size
        let rc = unsafe { fs_ext4_mkfs(&cfg, std::ptr::null(), std::ptr::null()) };
        assert_eq!(rc, 0, "mkfs failed: errno {}", fs_ext4_last_errno());
        dev.strict.store(true, Ordering::Relaxed);
        dev.requests.store(0, Ordering::Relaxed);
        dev.unaligned.lock().unwrap().clear();
        dev
    }

    pub fn assert_every_request_aligned(&self, tag: &str) {
        let seen = self.requests.load(Ordering::Relaxed);
        assert!(seen > 0, "[{tag}] the callbacks saw no request at all");
        let bad = self.unaligned.lock().unwrap();
        assert!(
            bad.is_empty(),
            "[{tag}] {} of {seen} requests were not {SECTOR}-aligned (is_write, offset, length), first: {:?}",
            bad.len(),
            &bad[..bad.len().min(8)]
        );
    }
}

extern "C" fn read_cb(ctx: *mut c_void, buf: *mut c_void, offset: u64, length: u64) -> c_int {
    let dev = unsafe { &*(ctx as *const StrictDev) };
    if !dev.admit(false, offset, length) {
        return -1;
    }
    let bytes = dev.bytes.lock().unwrap();
    let Some(end) = offset
        .checked_add(length)
        .filter(|&e| e <= bytes.len() as u64)
    else {
        return -1;
    };
    let src = &bytes[offset as usize..end as usize];
    unsafe { std::ptr::copy_nonoverlapping(src.as_ptr(), buf as *mut u8, src.len()) };
    0
}

extern "C" fn write_cb(ctx: *mut c_void, buf: *const c_void, offset: u64, length: u64) -> c_int {
    let dev = unsafe { &*(ctx as *const StrictDev) };
    if !dev.admit(true, offset, length) {
        return -1;
    }
    let mut bytes = dev.bytes.lock().unwrap();
    let Some(end) = offset
        .checked_add(length)
        .filter(|&e| e <= bytes.len() as u64)
    else {
        return -1;
    };
    let dst = &mut bytes[offset as usize..end as usize];
    unsafe { std::ptr::copy_nonoverlapping(buf as *const u8, dst.as_mut_ptr(), dst.len()) };
    0
}

/// A payload whose every byte depends on its position, so a misplaced
/// sector shows up as a mismatch rather than as equal bytes.
pub fn payload() -> Vec<u8> {
    (0..FILE_BYTES)
        .map(|i| (i as u32).wrapping_mul(2_654_435_761).rotate_right(13) as u8)
        .collect()
}

pub fn cstr(s: &str) -> CString {
    CString::new(s).unwrap()
}

/// Mount RW through `dev` with `block_size = SECTOR`, stat the root,
/// create a file, pwrite 1 MiB at an unaligned offset, read it back, make a
/// directory, unmount; then mount again read-only and read the file.
pub fn mount_write_read_round_trip(dev: &StrictDev) {
    let cfg = dev.cfg(SECTOR as u32);
    let fs = unsafe { fs_ext4_mount_rw_with_callbacks(&cfg) };
    assert!(
        !fs.is_null(),
        "RW mount through a {SECTOR}-byte-sector device failed: errno {}; unaligned requests: {:?}",
        fs_ext4_last_errno(),
        dev.unaligned.lock().unwrap()
    );

    let mut attr = unsafe { std::mem::zeroed::<fs_ext4_attr_t>() };
    assert_eq!(
        unsafe { fs_ext4_stat(fs, cstr("/").as_ptr(), &mut attr) },
        0,
        "stat /"
    );

    let path = cstr("/aligned.bin");
    assert_ne!(
        unsafe { fs_ext4_create(fs, path.as_ptr(), 0o644) },
        0,
        "create"
    );
    let data = payload();
    // 1 MiB at offset 1000: neither end on a sector boundary.
    let off = 1000u64;
    let n = unsafe {
        fs_ext4_pwrite(
            fs,
            path.as_ptr(),
            data.as_ptr() as *const c_void,
            data.len() as u64,
            off,
        )
    };
    // pwrite returns the new file size.
    assert_eq!(
        n,
        (off + data.len() as u64) as i64,
        "pwrite: errno {}",
        fs_ext4_last_errno()
    );
    assert_ne!(
        unsafe { fs_ext4_mkdir(fs, cstr("/d").as_ptr(), 0o755) },
        0,
        "mkdir"
    );

    let mut back = vec![0u8; data.len()];
    let r = unsafe {
        fs_ext4_read_file(
            fs,
            path.as_ptr(),
            back.as_mut_ptr() as *mut c_void,
            off,
            back.len() as u64,
        )
    };
    assert_eq!(r, data.len() as i64, "read_file");
    assert!(back == data, "read-back differs from what was written");
    unsafe { fs_ext4_umount(fs) };

    let ro = unsafe { fs_ext4_mount_with_callbacks(&cfg) };
    assert!(
        !ro.is_null(),
        "RO remount failed: errno {}",
        fs_ext4_last_errno()
    );
    let mut attr = unsafe { std::mem::zeroed::<fs_ext4_attr_t>() };
    assert_eq!(
        unsafe { fs_ext4_stat(ro, path.as_ptr(), &mut attr) },
        0,
        "stat file"
    );
    assert_eq!(attr.size, off + data.len() as u64, "file size");
    let mut back = vec![0u8; data.len()];
    let r = unsafe {
        fs_ext4_read_file(
            ro,
            path.as_ptr(),
            back.as_mut_ptr() as *mut c_void,
            off,
            back.len() as u64,
        )
    };
    assert_eq!(r, data.len() as i64, "read_file after remount");
    assert!(back == data, "read-back after remount differs");
    unsafe { fs_ext4_umount(ro) };
}
