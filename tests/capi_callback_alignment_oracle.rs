//! A volume written only through sector-aligned callback requests (#373)
//! is one e2fsck finds nothing wrong with.
//!
//! `capi_callback_alignment.rs` proves the callbacks saw only aligned
//! requests and that the driver reads back what it wrote; the driver's own
//! reader shares its reading of the format, so the read-modify-write that
//! assembles each unaligned write is checked here by an independent one.

mod strict_sector;

use strict_sector::*;

#[test]
fn a_volume_written_through_a_sector_aligned_device_is_e2fsck_clean() {
    let dev = StrictDev::formatted();
    mount_write_read_round_trip(&dev);
    dev.assert_every_request_aligned("oracle round trip");

    let path = fs_ext4_test_support::temp_path!("fs_ext4_aligned_{}.img", std::process::id());
    std::fs::write(&path, &*dev.bytes.lock().unwrap())
        .unwrap_or_else(|e| panic!("write {path}: {e}"));
    fs_ext4_test_support::assert_e2fsck_clean(&path, "sector-aligned callbacks");
    let _ = std::fs::remove_file(&path);
}
