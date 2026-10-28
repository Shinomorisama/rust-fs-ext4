//! First full-linear-parent allocation, not inline/index or deeper map growth.
use super::*;

#[test]
fn linux_checks_public_link_growth() {
    for capi in [false, true] {
        for bs in [1024, 4096] {
            for size in [128, 256] {
                for journal in [false, true] {
                    run_profile(capi, bs, size, journal, true);
                }
            }
        }
    }
}

pub(super) fn fill(image: &Path, base: &Path, bs: u32) {
    let out = guest_kernel_write(
        image.to_str().unwrap(),
        &format!(
            "python3 - \"$MNT\" {bs} <<'LINK_GROWTH'\n{}\nLINK_GROWTH\n",
            include_str!("link_growth_guest.py")
        ),
    );
    std::fs::write(base.with_extension("fill.jsonl"), &out.stdout).unwrap();
    std::fs::write(base.with_extension("fill.stderr.txt"), &out.stderr).unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| l.starts_with('{'))
            .count(),
        1
    );
}

pub(super) fn full_parent(fs: &Filesystem, parent: &Inode, block: u64) {
    assert_eq!(parent.size, u64::from(fs.sb.block_size()));
    // This controlled input has one directory-data block and one external EA
    // block. Linux i_blocks/stat.st_blocks account for both in 512-byte units;
    // the saved 3fe3f92 input reports four sectors for this 1 KiB profile.
    assert_ne!(parent.file_acl, 0);
    assert_ne!(parent.file_acl, block);
    assert_eq!(parent.blocks, 2 * u64::from(fs.sb.block_size() / 512));
    assert!(!parent.has_inline_data());
    assert_eq!(parent.flags & crate::inode::InodeFlags::INDEX.bits(), 0);
    assert_eq!(fs.map_inode_logical(parent, 1).unwrap(), None);
    // Decode every real record, including its usable slack. The first 255-byte
    // alias needs 264 bytes; no old record or checksum tail may accommodate it.
    let raw = fs.read_block(block).unwrap();
    let end = raw.len() - if fs.csum.enabled { 12 } else { 0 };
    let mut at = 0;
    let mut long = 0;
    while at < end {
        let ino = u32::from_le_bytes(raw[at..at + 4].try_into().unwrap());
        let span = usize::from(u16::from_le_bytes(raw[at + 4..at + 6].try_into().unwrap()));
        let name = usize::from(raw[at + 6]);
        assert!(span >= 8 && span % 4 == 0 && at + span <= end);
        let used = if ino == 0 {
            0
        } else {
            (8 + name).div_ceil(4) * 4
        };
        assert!(
            used <= span && span - used < 264,
            "input still admits a 255-byte name"
        );
        if name == 255 {
            long += 1;
        }
        at += span;
    }
    assert_eq!(at, end);
    assert_eq!(long, if fs.sb.block_size() == 1024 { 3 } else { 15 });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn account(
    fs: &Filesystem,
    image: &Path,
    base: &Path,
    before_parent: &Inode,
    parent: &Inode,
    original: u64,
    before: &[Vec<u8>],
    initial_free: (u64, u32),
) -> Vec<Vec<u8>> {
    let bs = fs.sb.block_size();
    assert_eq!(parent.size, 2 * u64::from(bs));
    assert_eq!(parent.blocks, before_parent.blocks + u64::from(bs / 512));
    assert_eq!(parent.flags, before_parent.flags);
    assert_eq!(fs.map_inode_logical(parent, 0).unwrap(), Some(original));
    assert_eq!(fs.map_inode_logical(parent, 2).unwrap(), None);
    let added = fs.map_inode_logical(parent, 1).unwrap().unwrap();
    assert_ne!(original, added);
    assert!(added < fs.sb.blocks_count);
    assert_eq!(free(image), (initial_free.0 - 1, initial_free.1));
    let relative = added - u64::from(fs.sb.first_data_block);
    let changed = (relative / u64::from(fs.sb.blocks_per_group)) as usize;
    let bit = relative % u64::from(fs.sb.blocks_per_group);
    let after = allocation(fs);
    let (desc_block, offset) = fs.sb.descriptor_location(changed as u64);
    let desc_size = usize::from(fs.sb.desc_size);
    for index in 0..fs.groups.len() {
        let row = index * 3;
        let (block, at) = fs.sb.descriptor_location(index as u64);
        let old = crate::bgd::BlockGroupDescriptor::parse(
            &before[row][at..at + desc_size],
            fs.sb.desc_size,
        )
        .unwrap();
        let new = crate::bgd::BlockGroupDescriptor::parse(
            &after[row][at..at + desc_size],
            fs.sb.desc_size,
        )
        .unwrap();
        assert_eq!(
            old.free_blocks_count - u32::from(index == changed),
            new.free_blocks_count
        );
        let mut allowed = Vec::new();
        if block == desc_block {
            allowed.push((offset + 12, 2));
            if desc_size >= 64 {
                allowed.push((offset + 44, 2));
            }
            if fs.csum.enabled {
                allowed.extend([(offset + 24, 2), (offset + 30, 2)]);
                if desc_size >= 64 {
                    allowed.push((offset + 56, 2));
                }
            }
        }
        stable(&before[row], &after[row], &allowed);
        assert_eq!(
            before[row + 1],
            after[row + 1],
            "hard link allocated an inode"
        );
        let mut expected = before[row + 2].clone();
        if index == changed {
            let byte = (bit / 8) as usize;
            let mask = 1u8 << (bit % 8);
            assert_eq!(expected[byte] & mask, 0, "new block was already allocated");
            expected[byte] |= mask;
        }
        assert_eq!(
            expected,
            after[row + 2],
            "allocation changed an unrelated bitmap bit"
        );
    }
    std::fs::write(base.with_extension("allocation.json"), format!(
        "{{\"original_block\":{original},\"added_block\":{added},\"group\":{changed},\"bit\":{bit},\"free_blocks_before\":{},\"free_blocks_after\":{},\"free_inodes\":{}}}\n",
        initial_free.0, initial_free.0 - 1, initial_free.1)).unwrap();
    after
}
