//! Where each group's descriptor lives, and how many blocks head each
//! group, against `dumpe2fs` on volumes `mkfs.ext4` made.
//!
//! The volumes cover the layouts the documentation describes: one
//! descriptor table after the superblock (sparse, not sparse, and
//! SPARSE_SUPER2's two named backups) and META_BG's per-meta-group blocks
//! (64- and 32-byte descriptors, and a short last meta group), at 1 KiB
//! and 4 KiB blocks, each with dozens to hundreds of groups.
//!
//! For every group:
//! - the descriptor [`Superblock::descriptor_location`] names, read from
//!   the image, must give the block bitmap `dumpe2fs` reports for it, and
//!   its block must be the primary copy `dumpe2fs` lists (the table after
//!   the superblock, or the meta group's own block);
//! - [`Superblock::group_head_metadata_blocks`] must equal the superblock,
//!   descriptor and reserved GDT blocks `dumpe2fs` lists for the group.
//!
//! The tools run in the harness VM; the test fails without it.

use fs_ext4::block_io::FileDevice;
use fs_ext4::superblock::Superblock;
use fs_ext4_test_support::oracle;
use std::collections::BTreeMap;

/// What `dumpe2fs` says about one group.
#[derive(Default, Debug)]
struct Group {
    block_bitmap: u64,
    /// Blocks listed as superblock, group descriptor(s) or reserved GDT.
    head: u64,
    /// The first block of each "Group descriptor(s) at A-B" range.
    descriptor_ranges: Vec<(u64, u64)>,
}

fn range(text: &str) -> (u64, u64) {
    let text = text.trim().trim_end_matches(',');
    match text.split_once('-') {
        Some((a, b)) => (a.parse().unwrap(), b.parse().unwrap()),
        None => {
            let a = text.parse().unwrap();
            (a, a)
        }
    }
}

fn dumpe2fs(image: &str) -> BTreeMap<u64, Group> {
    let out = oracle("dumpe2fs").arg(image).judged().clean("dumpe2fs");
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let mut groups = BTreeMap::new();
    let mut current: Option<u64> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("Group ") {
            current = rest.split(':').next().and_then(|g| g.parse().ok());
            if let Some(g) = current {
                groups.insert(g, Group::default());
            }
            continue;
        }
        let Some(g) = current else { continue };
        let group = groups.get_mut(&g).unwrap();
        for part in line.split(", ") {
            let part = part.trim();
            if part.contains("superblock at") {
                group.head += 1;
            } else if let Some(r) = part
                .strip_prefix("Group descriptors at ")
                .or_else(|| part.strip_prefix("Group descriptor at "))
            {
                let (a, b) = range(r);
                group.head += b - a + 1;
                group.descriptor_ranges.push((a, b));
            } else if let Some(r) = part.strip_prefix("Reserved GDT blocks at ") {
                let (a, b) = range(r);
                group.head += b - a + 1;
            } else if let Some(r) = part.strip_prefix("Block bitmap at ") {
                group.block_bitmap = r.split_whitespace().next().unwrap().parse().unwrap();
            }
        }
    }
    groups
}

fn check(tag: &str, block_size: u32, blocks_per_group: u32, mib: u64, features: &str) {
    let image = fs_ext4_test_support::temp_path!("fs_ext4_layout_{tag}_{}.img", std::process::id());
    std::fs::File::create(&image)
        .and_then(|f| f.set_len(mib << 20))
        .unwrap();
    let mut mkfs = oracle("mkfs.ext4").args([
        "-q",
        "-F",
        "-b",
        &block_size.to_string(),
        "-g",
        &blocks_per_group.to_string(),
    ]);
    if !features.is_empty() {
        mkfs = mkfs.args(["-O", features]);
    }
    let out = mkfs.arg(&image).output();
    assert!(
        out.status.success(),
        "[{tag}] mkfs.ext4: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let groups = dumpe2fs(&image);
    let bytes = std::fs::read(&image).unwrap();
    let sb = Superblock::read(&FileDevice::open(&image).unwrap()).unwrap();
    assert_eq!(groups.len() as u64, sb.block_group_count(), "[{tag}]");
    assert!(groups.len() >= 30, "[{tag}] only {} groups", groups.len());
    let bs = u64::from(sb.block_size());
    let per_block = sb.descs_per_block();

    for (&g, reported) in &groups {
        let (block, offset) = sb.descriptor_location(g);
        let at = (block * bs) as usize + offset;
        let lo = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let hi = if sb.is_64bit() && sb.desc_size >= 64 {
            u32::from_le_bytes(bytes[at + 0x20..at + 0x24].try_into().unwrap())
        } else {
            0
        };
        assert_eq!(
            u64::from(lo) | (u64::from(hi) << 32),
            reported.block_bitmap,
            "[{tag}] group {g}: the descriptor at block {block} + {offset}"
        );

        // The primary copy: in group 0's table, or in the head of the
        // first group of the meta group that lists this descriptor block.
        let holder = (g / per_block) * per_block;
        let in_group_zero = groups[&0]
            .descriptor_ranges
            .first()
            .is_some_and(|&(a, b)| (a..=b).contains(&block) && block - a == g / per_block);
        let in_holder = groups[&holder]
            .descriptor_ranges
            .first()
            .is_some_and(|&(a, b)| a == b && a == block);
        assert!(
            in_group_zero || in_holder,
            "[{tag}] group {g}: block {block} is not where dumpe2fs lists its primary \
             descriptor ({:?} in group 0, {:?} in group {holder})",
            groups[&0].descriptor_ranges,
            groups[&holder].descriptor_ranges
        );

        assert_eq!(
            sb.group_head_metadata_blocks(g),
            reported.head,
            "[{tag}] group {g}'s superblock, descriptor and reserved GDT blocks"
        );
    }
    let _ = std::fs::remove_file(&image);
}

#[test]
fn a_1k_sparse_volume_with_reserved_gdt_blocks() {
    check("1k_sparse", 1024, 1024, 64, "");
}

#[test]
fn a_1k_volume_without_sparse_super() {
    check("1k_nosparse", 1024, 1024, 32, "^sparse_super,^resize_inode");
}

#[test]
fn a_1k_volume_with_sparse_super2() {
    check("1k_sparse2", 1024, 1024, 32, "sparse_super2,^resize_inode");
}

#[test]
fn a_1k_meta_bg_volume() {
    check("1k_meta", 1024, 1024, 64, "meta_bg,^resize_inode");
}

#[test]
fn a_4k_volume_mkfs_makes_meta_bg() {
    // mkfs.ext4 turns META_BG on by itself here: 256 groups of 4 MiB.
    check("4k_flex", 4096, 1024, 1024, "");
}

#[test]
fn a_4k_meta_bg_volume_with_32_byte_descriptors_and_a_short_last_meta_group() {
    // 128 descriptors per block, 200 groups: the second meta group stops
    // at group 199, so its "last group" copy has nowhere to go.
    check("4k_meta32", 4096, 256, 200, "meta_bg,^resize_inode,^64bit");
}

#[test]
fn a_4k_volume_without_flex_bg() {
    check("4k_noflex", 4096, 4096, 512, "^flex_bg");
}
