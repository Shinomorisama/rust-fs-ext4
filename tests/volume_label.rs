//! The volume label can be changed after the volume is made (#447).
//!
//! `s_volume_name` is the 16 bytes at superblock offset 0x78, NUL-padded.
//! `Filesystem::set_volume_label` rewrites it in the primary superblock and
//! in every backup the volume's layout places, restamping each one's checksum
//! on a metadata_csum volume. This file reads the result back through the
//! driver and from the raw bytes; `tests/volume_label_oracle.rs` has
//! `dumpe2fs` read it back and `e2fsck -fn` judge the volume.

use fs_ext4::block_io::{BlockDevice, FileDevice};
use fs_ext4::features::FsFlavor;
use fs_ext4::fs::{Filesystem, VOLUME_LABEL_MAX};
use fs_ext4::{mkfs, Error};
use std::sync::Arc;

fn format(tag: &str, size: u64, block_size: u32, flavor: FsFlavor) -> String {
    let path =
        fs_ext4_test_support::temp_path!("fs_ext4_volume_label_{tag}_{}.img", std::process::id());
    std::fs::File::create(&path)
        .and_then(|f| f.set_len(size))
        .unwrap_or_else(|e| panic!("create {path}: {e}"));
    let dev = FileDevice::open_rw(&path).expect("open_rw");
    mkfs::format_filesystem_with_flavor(&dev, Some("before"), None, size, block_size, flavor)
        .expect("format");
    dev.flush().expect("flush");
    path
}

fn mount_rw(image: &str) -> Filesystem {
    Filesystem::mount(Arc::new(FileDevice::open_rw(image).expect("open rw"))).expect("mount")
}

fn mount(image: &str) -> Filesystem {
    Filesystem::mount(Arc::new(FileDevice::open(image).expect("open"))).expect("mount")
}

/// Every superblock on the volume, primary first, as (group, raw 1024 bytes).
/// Where the backups are is worked out here from the geometry and the
/// sparse_super rule (groups 0, 1 and the powers of 3, 5 and 7), not asked of
/// the driver.
fn superblocks(image: &str) -> Vec<(u64, Vec<u8>)> {
    let fs = mount(image);
    let bs = u64::from(fs.sb.block_size());
    let per_group = u64::from(fs.sb.blocks_per_group);
    let groups = fs.sb.blocks_count.div_ceil(per_group);
    let sparse = fs.sb.feature_ro_compat & 0x1 != 0;
    let powers = |g: u64, b: u64| {
        let mut p = b;
        while p < g {
            p *= b;
        }
        p == g
    };
    let dev = FileDevice::open(image).expect("open");
    let mut out = Vec::new();
    for g in 0..groups {
        let has = !sparse || g <= 1 || powers(g, 3) || powers(g, 5) || powers(g, 7);
        if !has {
            continue;
        }
        let at = if g == 0 {
            1024
        } else {
            (u64::from(fs.sb.first_data_block) + g * per_group) * bs
        };
        let mut raw = vec![0u8; 1024];
        dev.read_at(at, &mut raw).expect("read a superblock");
        assert_eq!(&raw[0x38..0x3A], &[0x53, 0xEF], "group {g}: no magic");
        out.push((g, raw));
    }
    out
}

fn padded(label: &[u8]) -> [u8; 16] {
    let mut field = [0u8; 16];
    field[..label.len()].copy_from_slice(label);
    field
}

fn check(tag: &str, size: u64, block_size: u32, flavor: FsFlavor, min_backups: usize) {
    let img = format(tag, size, block_size, flavor);
    assert_eq!(mount(&img).sb.volume_name, "before", "[{tag}] as formatted");

    let mut fs = mount_rw(&img);
    fs.set_volume_label(b"after").expect("set the label");
    assert_eq!(
        fs.sb.volume_name, "after",
        "[{tag}] the mount sees its own write"
    );
    fs.finish().expect("finish");

    let fs = mount(&img);
    assert_eq!(
        fs.sb.volume_name, "after",
        "[{tag}] read back after a remount"
    );
    let csum = fs.csum.enabled;
    let sbs = superblocks(&img);
    assert!(
        sbs.len() > min_backups,
        "[{tag}] {} superblocks, wanted a primary and at least {min_backups} backups",
        sbs.len()
    );
    for (g, raw) in &sbs {
        assert_eq!(
            raw[0x78..0x88],
            padded(b"after"),
            "[{tag}] group {g}'s label"
        );
        if csum {
            assert!(
                fs.csum.verify_superblock(raw),
                "[{tag}] group {g}'s superblock checksum"
            );
        }
    }

    // Sixteen bytes fill the field with no terminator; an empty label
    // clears it.
    let full = b"0123456789abcdef";
    assert_eq!(full.len(), VOLUME_LABEL_MAX);
    let mut fs = mount_rw(&img);
    fs.set_volume_label(full).expect("a 16-byte label");
    fs.finish().expect("finish");
    assert_eq!(
        mount(&img).sb.volume_name,
        "0123456789abcdef",
        "[{tag}] 16 bytes"
    );
    let mut fs = mount_rw(&img);
    fs.set_volume_label(b"").expect("an empty label");
    fs.finish().expect("finish");
    assert_eq!(mount(&img).sb.volume_name, "", "[{tag}] cleared");
    for (g, raw) in superblocks(&img) {
        assert_eq!(raw[0x78..0x88], [0u8; 16], "[{tag}] group {g} cleared");
    }

    let _ = std::fs::remove_file(&img);
}

#[test]
fn extents_4k_blocks_multi_group() {
    check("extents_4k_mg3", 320 << 20, 4096, FsFlavor::Ext4, 1);
}

#[test]
fn extents_1k_blocks() {
    check("extents_1k", 8 << 20, 1024, FsFlavor::Ext4, 0);
}

// The formatter lays out ext2 and ext3 in one group only; the oracle test
// has mke2fs make multi-group volumes of both, with backups but no checksums.
#[test]
fn ext3_1k_blocks() {
    check("ext3_1k", 8 << 20, 1024, FsFlavor::Ext3, 0);
}

#[test]
fn ext2_1k_blocks() {
    check("ext2_1k", 8 << 20, 1024, FsFlavor::Ext2, 0);
}

#[test]
fn a_label_that_does_not_fit_is_refused_and_nothing_is_written() {
    let img = format("refused", 320 << 20, 4096, FsFlavor::Ext4);
    let before = superblocks(&img);
    let mut fs = mount_rw(&img);
    for bad in [&b"0123456789abcdefg"[..], b"with\0nul"] {
        match fs.set_volume_label(bad) {
            Err(Error::InvalidArgument(_)) => {}
            other => panic!("{bad:?}: {other:?}"),
        }
    }
    assert_eq!(fs.sb.volume_name, "before");
    fs.finish().expect("finish");
    assert_eq!(superblocks(&img), before, "a refused label wrote something");
    let _ = std::fs::remove_file(&img);
}

#[test]
fn a_read_only_mount_refuses() {
    let img = format("read_only", 8 << 20, 1024, FsFlavor::Ext4);
    let mut fs = mount(&img);
    assert!(
        fs.set_volume_label(b"x").is_err(),
        "a read-only mount wrote"
    );
    assert_eq!(mount(&img).sb.volume_name, "before");
    let _ = std::fs::remove_file(&img);
}
