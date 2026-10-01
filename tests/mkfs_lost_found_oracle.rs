//! `/lost+found` on a volume this crate formats is the one `mke2fs` makes
//! (#443), as `debugfs` reads both, and `e2fsck -fn` passes the volume.
//!
//! For each flavour and block size the formatter lays out, a volume of the
//! same size is made both ways, and `debugfs -R 'stat /lost+found'` on each is
//! reduced to the fields that are the directory's shape rather than its
//! placement or its timestamps: inode number, type, mode, flags, link count,
//! size and block count. `ls -l` of the root and of `/lost+found` is compared
//! the same way, by inode, mode and name. The sizes are whatever `mke2fs` in
//! the harness VM chooses, so nothing here restates them.
//!
//! `tests/mkfs_lost_found.rs` reads the same volumes back through the driver.

use fs_ext4::block_io::{BlockDevice, FileDevice};
use fs_ext4::features::FsFlavor;
use fs_ext4::mkfs;
use fs_ext4_test_support::{assert_e2fsck_clean, oracle, temp_path};

/// The `stat` fields that describe the directory itself.
const STAT_FIELDS: [&str; 7] = [
    "Inode",
    "Type",
    "Mode",
    "Flags",
    "Links",
    "Size",
    "Blockcount",
];

/// `key: value` pairs `debugfs -R stat` prints, for the keys asked about.
fn stat_fields(report: &str) -> Vec<(String, String)> {
    let tokens: Vec<&str> = report.split_whitespace().collect();
    STAT_FIELDS
        .iter()
        .map(|key| {
            let label = format!("{key}:");
            let value = tokens
                .iter()
                .position(|t| *t == label)
                .and_then(|i| tokens.get(i + 1))
                .unwrap_or_else(|| panic!("no `{label}` in debugfs's stat:\n{report}"));
            ((*key).to_string(), (*value).to_string())
        })
        .collect()
}

fn debugfs(request: &str, image: &str) -> String {
    let out = oracle("debugfs").args(["-R", request, image]).output();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// `ls -l` as (inode, mode, name) rows, without sizes and dates, which
/// differ between two volumes made at different times.
fn listing(image: &str, dir: &str) -> Vec<(String, String, String)> {
    debugfs(&format!("ls -l {dir}"), image)
        .lines()
        .filter_map(|line| {
            let cols: Vec<&str> = line.split_whitespace().collect();
            // inode, mode, (links), uid, gid, size, date, time, name
            (cols.len() >= 9 && cols[0] != "0").then(|| {
                (
                    cols[0].to_string(),
                    cols[1].to_string(),
                    cols[cols.len() - 1].to_string(),
                )
            })
        })
        .collect()
}

fn scratch(tag: &str, who: &str, size: u64) -> String {
    let path = temp_path!("fs_ext4_lf_{who}_{tag}_{}.img", std::process::id());
    std::fs::File::create(&path)
        .and_then(|f| f.set_len(size))
        .unwrap_or_else(|e| panic!("create {path}: {e}"));
    path
}

fn matches_mke2fs(tag: &str, flavor: FsFlavor, block_size: u32, size: u64) {
    let ours = scratch(tag, "ours", size);
    {
        let dev = FileDevice::open_rw(&ours).expect("open_rw");
        mkfs::format_filesystem_with_flavor(&dev, None, None, size, block_size, flavor)
            .expect("format");
        dev.flush().expect("flush");
    }
    assert_e2fsck_clean(&ours, tag);

    let theirs = scratch(tag, "mke2fs", size);
    let kind = match flavor {
        FsFlavor::Ext2 => "ext2",
        FsFlavor::Ext3 => "ext3",
        FsFlavor::Ext4 => "ext4",
    };
    let made = oracle("mke2fs")
        .args(["-q", "-F", "-t", kind, "-b"])
        .arg(block_size.to_string())
        .arg(&theirs)
        .output();
    assert!(
        made.status.success(),
        "[{tag}] mke2fs: {}",
        String::from_utf8_lossy(&made.stderr)
    );

    assert_eq!(
        stat_fields(&debugfs("stat /lost+found", &ours)),
        stat_fields(&debugfs("stat /lost+found", &theirs)),
        "[{tag}] /lost+found: ours, then mke2fs's"
    );
    for dir in ["/", "/lost+found"] {
        let (a, b) = (listing(&ours, dir), listing(&theirs, dir));
        assert_eq!(a, b, "[{tag}] ls -l {dir}: ours, then mke2fs's");
    }

    let _ = std::fs::remove_file(&ours);
    let _ = std::fs::remove_file(&theirs);
}

#[test]
fn ext2_1k_blocks() {
    matches_mke2fs("ext2_1k", FsFlavor::Ext2, 1024, 8 << 20);
}

#[test]
fn ext3_1k_blocks() {
    matches_mke2fs("ext3_1k", FsFlavor::Ext3, 1024, 8 << 20);
}

#[test]
fn extents_1k_blocks() {
    matches_mke2fs("extents_1k", FsFlavor::Ext4, 1024, 8 << 20);
}

#[test]
fn extents_2k_blocks() {
    matches_mke2fs("extents_2k", FsFlavor::Ext4, 2048, 16 << 20);
}

#[test]
fn extents_4k_blocks() {
    matches_mke2fs("extents_4k", FsFlavor::Ext4, 4096, 32 << 20);
}

#[test]
fn extents_4k_blocks_multi_group() {
    matches_mke2fs("extents_4k_mg3", FsFlavor::Ext4, 4096, 320 << 20);
}

#[test]
fn extents_8k_blocks() {
    matches_mke2fs("extents_8k", FsFlavor::Ext4, 8192, 64 << 20);
}

#[test]
fn extents_16k_blocks() {
    matches_mke2fs("extents_16k", FsFlavor::Ext4, 16384, 64 << 20);
}

#[test]
fn extents_64k_blocks() {
    matches_mke2fs("extents_64k", FsFlavor::Ext4, 65536, 64 << 20);
}
