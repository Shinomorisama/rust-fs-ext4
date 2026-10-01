//! The damage `tests/fsck_unreadable_dir.rs` gives a directory, judged by
//! e2fsck as well as by the driver's audit (#445): both must call the
//! volume damaged, where the audit used to call it clean.
//!
//! e2fsck reads a zeroed extent header on a directory as a corrupt extent
//! header and offers to clear the inode; with `-n` it answers no and exits 4.
//! The audit's finding is an `UnreadableDirectory` naming the same inode.

mod unreadable_dir;

use fs_ext4::features::FsFlavor;
use fs_ext4::fsck::Anomaly;
use unreadable_dir::{
    audit, destroy_extent_header, formatted, formatted_flavor, mkdir_d, set_links, unreadable,
};

fn both_call_it_damaged(tag: &str, img: &str, ino: u32) {
    let said = fs_ext4_test_support::oracle("e2fsck")
        .args(["-fn", img])
        .judged()
        .findings(tag);
    assert!(
        said.contains(&format!("Inode {ino} ")),
        "[{tag}] e2fsck's findings name inode {ino}:\n{said}"
    );
    let report = audit(img);
    assert!(!report.is_clean(), "[{tag}] the audit calls it clean");
    assert_eq!(
        unreadable(&report),
        vec![ino],
        "[{tag}] {:?}",
        report.anomalies
    );
}

#[test]
fn an_unreadable_root() {
    let img = formatted("oracle_root");
    fs_ext4_test_support::assert_e2fsck_clean(&img, "oracle_root before");
    destroy_extent_header(&img, 2);
    both_call_it_damaged("oracle_root", &img, 2);
    let _ = std::fs::remove_file(&img);
}

#[test]
fn an_unreadable_subdirectory() {
    let img = formatted("oracle_subdir");
    let d = mkdir_d(&img);
    fs_ext4_test_support::assert_e2fsck_clean(&img, "oracle_subdir before");
    destroy_extent_header(&img, d);
    both_call_it_damaged("oracle_subdir", &img, d);
    let _ = std::fs::remove_file(&img);
}

/// Under a block-mapped root (ext2, ext3) the audit reads the directories,
/// so a wrong link count on `/d` is found by both: e2fsck reports the
/// inode's ref count, the audit a `LinkCountTooHigh` naming it.
#[test]
fn a_wrong_link_count_under_a_block_mapped_root() {
    for (tag, flavor) in [
        ("oracle_ext2", FsFlavor::Ext2),
        ("oracle_ext3", FsFlavor::Ext3),
    ] {
        let img = formatted_flavor(tag, flavor);
        let d = mkdir_d(&img);
        fs_ext4_test_support::assert_e2fsck_clean(&img, &format!("{tag} before"));
        set_links(&img, d, 7);
        let said = fs_ext4_test_support::oracle("e2fsck")
            .args(["-fn", &img])
            .judged()
            .findings(tag);
        assert!(
            said.contains(&format!("Inode {d} ref count is 7")),
            "[{tag}] e2fsck's findings name inode {d}'s count:\n{said}"
        );
        let report = audit(&img);
        assert!(
            report.anomalies.iter().any(|a| matches!(
                a,
                Anomaly::LinkCountTooHigh { ino, stored: 7, .. } if *ino == d
            )),
            "[{tag}] {:?}",
            report.anomalies
        );
        let _ = std::fs::remove_file(&img);
    }
}
