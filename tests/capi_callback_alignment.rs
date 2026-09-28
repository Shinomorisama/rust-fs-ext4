//! `cfg.block_size` is the device's sector size on the callback mounts
//! (#373): with it set, the callbacks receive only requests aligned to it.
//!
//! A host whose block resource accepts only sector-aligned I/O refused the
//! engine's first read -- the superblock, 1024 bytes at offset 1024 -- so the
//! mount returned NULL. The field was documented as the physical block size
//! and no mount path read it.
//!
//! The same scenario, read back by e2fsck, is
//! `capi_callback_alignment_oracle.rs`.

mod strict_sector;

use fs_ext4::capi::*;
use std::sync::atomic::Ordering;
use strict_sector::*;

#[test]
fn a_sector_aligned_device_mounts_writes_and_reads_through_the_callbacks() {
    let dev = StrictDev::formatted();
    mount_write_read_round_trip(&dev);
    dev.assert_every_request_aligned("round trip");
}

#[test]
fn the_lazy_rw_mount_honours_the_sector_size_too() {
    let dev = StrictDev::formatted();
    let cfg = dev.cfg(SECTOR as u32);
    let fs = unsafe { fs_ext4_mount_rw_with_callbacks_lazy(&cfg) };
    assert!(
        !fs.is_null(),
        "lazy RW mount failed: errno {}",
        fs_ext4_last_errno()
    );
    assert_eq!(unsafe { fs_ext4_replay_journal_if_dirty(fs) }, 0);
    assert_ne!(
        unsafe { fs_ext4_create(fs, cstr("/lazy").as_ptr(), 0o644) },
        0
    );
    unsafe { fs_ext4_umount(fs) };
    dev.assert_every_request_aligned("lazy");
}

#[test]
fn a_block_size_that_is_not_a_power_of_two_is_einval() {
    let dev = StrictDev::formatted();
    for bs in [3u32, 1536, 4097] {
        let cfg = dev.cfg(bs);
        let fs = unsafe { fs_ext4_mount_rw_with_callbacks(&cfg) };
        assert!(fs.is_null(), "block_size {bs} mounted");
        assert_eq!(fs_ext4_last_errno(), libc_einval(), "block_size {bs}");
        let fs = unsafe { fs_ext4_mount_with_callbacks(&cfg) };
        assert!(fs.is_null(), "block_size {bs} mounted read-only");
        assert_eq!(fs_ext4_last_errno(), libc_einval(), "block_size {bs} (RO)");
    }
    assert_eq!(
        dev.requests.load(Ordering::Relaxed),
        0,
        "a refused cfg touched the device"
    );
}

#[test]
fn a_sector_larger_than_the_filesystem_block_is_einval() {
    // 4 KiB filesystem blocks; a 8 KiB sector cannot hold one block alone.
    let dev = StrictDev::formatted();
    let cfg = dev.cfg(8192);
    let fs = unsafe { fs_ext4_mount_with_callbacks(&cfg) };
    assert!(fs.is_null(), "an 8 KiB sector under 4 KiB blocks mounted");
    assert_eq!(fs_ext4_last_errno(), libc_einval());
}

#[test]
fn a_block_size_that_does_not_divide_the_device_is_einval() {
    let dev = StrictDev::formatted();
    let mut cfg = dev.cfg(SECTOR as u32);
    cfg.size_bytes -= 512;
    let fs = unsafe { fs_ext4_mount_with_callbacks(&cfg) };
    assert!(fs.is_null(), "a device of a partial sector mounted");
    assert_eq!(fs_ext4_last_errno(), libc_einval());
}

#[test]
fn block_size_zero_and_one_keep_byte_granular_requests() {
    for bs in [0u32, 1] {
        let dev = StrictDev::formatted();
        dev.strict.store(false, Ordering::Relaxed);
        let cfg = dev.cfg(bs);
        let fs = unsafe { fs_ext4_mount_with_callbacks(&cfg) };
        assert!(
            !fs.is_null(),
            "block_size {bs}: errno {}",
            fs_ext4_last_errno()
        );
        unsafe { fs_ext4_umount(fs) };
        let unaligned = dev.unaligned.lock().unwrap();
        assert!(
            unaligned.contains(&(false, 1024, 1024)),
            "block_size {bs}: the superblock read was not passed through as 1024@1024: {:?}",
            &unaligned[..unaligned.len().min(8)]
        );
    }
}

fn libc_einval() -> i32 {
    22
}
