//! The volume and the damage shared by the #445 tests:
//! `fsck_unreadable_dir.rs` (unit tier) and `fsck_unreadable_dir_oracle.rs`
//! (the same damage, judged by e2fsck in the harness VM).
#![allow(dead_code)]

use fs_ext4::block_io::{BlockDevice, FileDevice};
use fs_ext4::fs::Filesystem;
use fs_ext4::fsck::{self, Anomaly};
use fs_ext4::mkfs;
use std::sync::Arc;

/// A 32 MiB volume from this crate's formatter, at 4 KiB blocks.
pub fn formatted(tag: &str) -> String {
    let path =
        fs_ext4_test_support::temp_path!("fs_ext4_unreadable_dir_{tag}_{}.img", std::process::id());
    let size = 32 << 20;
    std::fs::File::create(&path)
        .and_then(|f| f.set_len(size))
        .unwrap_or_else(|e| panic!("create {path}: {e}"));
    let dev = FileDevice::open_rw(&path).expect("open_rw");
    mkfs::format_filesystem(&dev, None, None, size, 4096).expect("format");
    dev.flush().expect("flush");
    path
}

/// An 8 MiB volume in `flavor` at 1 KiB blocks: for ext2 and ext3 its
/// directories are block-mapped, with no extent tree.
pub fn formatted_flavor(tag: &str, flavor: fs_ext4::features::FsFlavor) -> String {
    let path =
        fs_ext4_test_support::temp_path!("fs_ext4_unreadable_dir_{tag}_{}.img", std::process::id());
    let size = 8 << 20;
    std::fs::File::create(&path)
        .and_then(|f| f.set_len(size))
        .unwrap_or_else(|e| panic!("create {path}: {e}"));
    let dev = FileDevice::open_rw(&path).expect("open_rw");
    mkfs::format_filesystem_with_flavor(&dev, None, None, size, 1024, flavor).expect("format");
    dev.flush().expect("flush");
    path
}

/// Set inode `ino`'s `i_links_count` on a volume without metadata_csum.
pub fn set_links(image: &str, ino: u32, links: u16) {
    let fs = Filesystem::mount(Arc::new(FileDevice::open(image).expect("open"))).expect("mount");
    assert!(!fs.csum.enabled, "set_links does not restamp a checksum");
    let (block, offset) = fs_ext4::bgd::locate_inode(&fs.sb, &fs.groups, ino).expect("locate");
    let at = block * u64::from(fs.sb.block_size()) + u64::from(offset) + 0x1A;
    let dev = FileDevice::open_rw(image).expect("open rw");
    dev.write_at(at, &links.to_le_bytes())
        .expect("write i_links_count");
    dev.flush().expect("flush");
}

/// Zero inode `ino`'s extent header and restamp the inode's checksum.
pub fn destroy_extent_header(image: &str, ino: u32) {
    let fs = Filesystem::mount(Arc::new(FileDevice::open(image).expect("open"))).expect("mount");
    let (block, offset) = fs_ext4::bgd::locate_inode(&fs.sb, &fs.groups, ino).expect("locate");
    let at = block * u64::from(fs.sb.block_size()) + u64::from(offset);
    let dev = FileDevice::open_rw(image).expect("open rw");
    let mut raw = vec![0u8; usize::from(fs.sb.inode_size)];
    dev.read_at(at, &mut raw).expect("read the inode");
    raw[0x28..0x28 + 12].fill(0);
    let generation = u32::from_le_bytes(raw[0x64..0x68].try_into().unwrap());
    let (lo, hi) = fs
        .csum
        .compute_inode_checksum(ino, generation, &raw)
        .expect("a metadata_csum volume");
    raw[0x7C..0x7E].copy_from_slice(&lo.to_le_bytes());
    raw[0x82..0x84].copy_from_slice(&hi.to_le_bytes());
    dev.write_at(at, &raw).expect("write the inode");
    dev.flush().expect("flush");
}

/// `/d`, made on `image`; its inode number.
pub fn mkdir_d(image: &str) -> u32 {
    let fs =
        Filesystem::mount(Arc::new(FileDevice::open_rw(image).expect("open rw"))).expect("mount");
    fs.apply_mkdir("/d", 0o755).expect("mkdir")
}

/// The driver's audit of `image`.
pub fn audit(image: &str) -> fsck::AuditReport {
    let fs = Filesystem::mount(Arc::new(FileDevice::open(image).expect("open"))).expect("mount");
    fsck::audit(&fs, u32::MAX, u32::MAX).expect("audit")
}

/// The directories the audit reported as unreadable.
pub fn unreadable(report: &fsck::AuditReport) -> Vec<u32> {
    report
        .anomalies
        .iter()
        .filter_map(|a| match a {
            Anomaly::UnreadableDirectory { ino, .. } => Some(*ino),
            _ => None,
        })
        .collect()
}
