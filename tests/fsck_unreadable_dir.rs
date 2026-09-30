//! A directory the audit cannot read is a finding, not a silence (#445).
//!
//! The walk used to note such a directory as incomplete and move on, which
//! only suppressed the link-count findings its missing entries could have
//! explained. Nothing was reported for the directory itself, so a volume whose
//! root could not be listed at all audited clean, and `fsck.ext4 -n` exited 0
//! on it. `e2fsck -fn` calls the same volume damaged
//! (`tests/fsck_unreadable_dir_oracle.rs`).
//!
//! The damage here is the one that reaches the entries rather than the inode:
//! the directory's 12-byte extent header is zeroed and the inode checksum
//! restamped, so the inode itself verifies and only its extent tree is wrong.

mod unreadable_dir;

use fs_ext4::block_io::FileDevice;
use fs_ext4::features::FsFlavor;
use fs_ext4::fs::Filesystem;
use fs_ext4::fsck;
use fs_ext4::fsck::Anomaly;
use std::sync::Arc;
use unreadable_dir::{
    audit, destroy_extent_header, formatted, formatted_flavor, mkdir_d, set_links, unreadable,
};

#[test]
fn an_unreadable_root_is_reported() {
    let img = formatted("root");
    assert!(audit(&img).is_clean(), "the fresh volume audits clean");
    destroy_extent_header(&img, 2);

    let report = audit(&img);
    assert!(!report.is_clean(), "an unlistable root is not clean");
    assert_eq!(unreadable(&report), vec![2], "{:?}", report.anomalies);
    assert_eq!(report.anomalies_count, report.anomalies.len() as u64);
    let _ = std::fs::remove_file(&img);
}

#[test]
fn an_unreadable_subdirectory_is_reported() {
    let img = formatted("subdir");
    let d = mkdir_d(&img);
    assert!(audit(&img).is_clean(), "the volume audits clean before");
    destroy_extent_header(&img, d);

    let report = audit(&img);
    assert!(!report.is_clean(), "an unlistable directory is not clean");
    assert_eq!(unreadable(&report), vec![d], "{:?}", report.anomalies);
    let _ = std::fs::remove_file(&img);
}

/// The repair pass has nothing it can safely do for a directory it cannot
/// read, so the finding survives it and the report stays unclean.
#[test]
fn a_repair_pass_leaves_it_reported() {
    let img = formatted("repair");
    destroy_extent_header(&img, 2);
    let fs =
        Filesystem::mount(Arc::new(FileDevice::open_rw(&img).expect("open rw"))).expect("mount");
    let report = fsck::audit_with_repair(&fs, u32::MAX, u32::MAX, |_, _, _| {}, |_| {}, true)
        .expect("repair pass");
    assert_eq!(report.repaired_count, 0, "nothing is repaired");
    assert!(!report.is_clean(), "still not clean after the repair pass");
    drop(fs);
    let _ = std::fs::remove_file(&img);
}

/// A block-mapped directory (ext2, ext3: no extent tree) is read, not called
/// unreadable. The audit used to refuse every such directory and note it as
/// incomplete, so on those volumes it examined nothing beneath the root and
/// suppressed every link-count finding; a wrong link count is found now.
#[test]
fn a_block_mapped_directory_is_read() {
    for (tag, flavor) in [("ext2", FsFlavor::Ext2), ("ext3", FsFlavor::Ext3)] {
        let img = formatted_flavor(&format!("block_mapped_{tag}"), flavor);
        let d = mkdir_d(&img);
        let report = audit(&img);
        assert!(report.is_clean(), "[{tag}] {:?}", report.anomalies);
        assert_eq!(
            report.directories_scanned, 3,
            "[{tag}] the root, lost+found and /d"
        );

        set_links(&img, d, 7);
        let report = audit(&img);
        assert!(
            report.anomalies.iter().any(|a| matches!(
                a,
                Anomaly::LinkCountTooHigh { ino, stored: 7, observed: 2 } if *ino == d
            )),
            "[{tag}] {:?}",
            report.anomalies
        );
        let _ = std::fs::remove_file(&img);
    }
}
