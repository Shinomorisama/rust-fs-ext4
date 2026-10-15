//! Private current allocation checks for intervals, not exclusive ownership.
//! Formats: https://docs.kernel.org/filesystems/ext4/bitmaps.html and
//! https://docs.kernel.org/filesystems/ext4/group_descr.html

use crate::bgd::{BgdFlags, BlockGroupDescriptor};
use crate::checksum::{group_desc_csum, linux_crc32c};
use crate::error::{Error, Result};
use crate::features::{Compat, Incompat, RoCompat};
use crate::fs::{BlockBuffer, Filesystem};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Range {
    pub start: u64,
    pub end: u64,
}

#[derive(Clone, Copy)]
pub(crate) struct Limits {
    pub ranges: usize,
    /// Distinct allocation groups touched by the ordered ranges.
    pub groups: u64,
    /// Logical fetches, including staged copies and failed attempts.
    pub reads: u64,
    /// Two full bitmap blocks per group bound checksum plus padding work;
    /// each range/group segment additionally charges its covered bitmap bytes.
    pub bitmap_bytes: u64,
    /// Two reusable blocks; no output, per-block list or growing bitmap cache.
    pub scratch_bytes: usize,
}

const LIMIT: Error = Error::Unsupported("block allocation resource limit");
const PROFILE: Error = Error::Unsupported("block allocation profile");

struct Reader<'a> {
    fs: &'a Filesystem,
    buf: &'a BlockBuffer,
    end: u64,
    left: u64,
}

impl Reader<'_> {
    fn block(&mut self, block: u64, out: &mut [u8]) -> Result<()> {
        if block >= self.end {
            return Err(Error::InvalidBlock(block));
        }
        self.left = self.left.checked_sub(1).ok_or(LIMIT)?;
        match self.buf.dirty.get(&block) {
            Some(bytes) if bytes.len() == out.len() => out.copy_from_slice(bytes),
            Some(_) => return Err(Error::Corrupt("block allocation staged block size")),
            None => self.fs.dev.read_at(
                block
                    .checked_mul(out.len() as u64)
                    .ok_or(Error::InvalidBlock(block))?,
                out,
            )?,
        }
        Ok(())
    }
}

/// Whether every bit in the half-open range is one. Callers validate bounds
/// and debit byte work before entry. Partial edge bytes ignore neighboring bits.
fn all_set(bitmap: &[u8], start: u64, end: u64) -> bool {
    for byte in start / 8..end.div_ceil(8) {
        let lo = start.saturating_sub(byte * 8).min(8) as u32;
        let hi = (end - byte * 8).min(8) as u32;
        let mask = ((0xffu16 << lo) & (0xffu16 >> (8 - hi))) as u8;
        if bitmap[byte as usize] & mask != mask {
            return false;
        }
    }
    true
}

/// Verify sorted, disjoint, nonempty intervals on a trusted mount and stable,
/// coherent device/staged view. Requires an independently qualified current
/// fixed layout on that same view; this does not prove fixed-role separation,
/// inode ownership, complete claim coverage or an epoch/lifecycle. No result is
/// cached/published and no caller input is mutated, even on failure.
///
/// Initial profile: initialized block bitmaps, metadata checksums, ordinary
/// allocation, 1/4 KiB blocks and 32/64-byte descriptors. CRC covers bpg/8 bytes;
/// every bit after the group's real end through the whole bitmap must be one.
/// The 1 KiB boot block zero is outside this domain; 4 KiB block zero has a bit.
pub(crate) fn verify(
    fs: &Filesystem,
    buf: &BlockBuffer,
    ranges: &[Range],
    limits: Limits,
) -> Result<()> {
    let sb = &fs.sb;
    let bs = sb.block_size() as usize;
    let compat = Compat::HAS_JOURNAL.bits()
        | Compat::EXT_ATTR.bits()
        | Compat::RESIZE_INODE.bits()
        | Compat::DIR_INDEX.bits()
        | Compat::SPARSE_SUPER2.bits();
    let incompat = Incompat::FILETYPE.bits()
        | Incompat::EXTENTS.bits()
        | Incompat::BIT64.bits()
        | Incompat::FLEX_BG.bits()
        | Incompat::CSUM_SEED.bits()
        | Incompat::CASEFOLD.bits();
    let ro = RoCompat::SPARSE_SUPER.bits()
        | RoCompat::LARGE_FILE.bits()
        | RoCompat::HUGE_FILE.bits()
        | RoCompat::DIR_NLINK.bits()
        | RoCompat::EXTRA_ISIZE.bits()
        | RoCompat::METADATA_CSUM.bits();
    if !fs.csum.enabled
        || sb.feature_ro_compat & RoCompat::METADATA_CSUM.bits() == 0
        || sb.feature_compat & !compat != 0
        || sb.feature_incompat & !incompat != 0
        || sb.feature_ro_compat & !ro != 0
        || sb.creator_os != 0
        || !matches!(bs, 1024 | 4096)
        || !matches!(sb.desc_size, 32 | 64)
        || sb.first_data_block != u32::from(bs == 1024)
    {
        return Err(PROFILE);
    }
    let first = u64::from(sb.first_data_block);
    let bpg = u64::from(sb.blocks_per_group);
    let ipg = u64::from(sb.inodes_per_group);
    if bpg == 0
        || bpg > (bs * 8) as u64
        || !bpg.is_multiple_of(8)
        || ipg == 0
        || ipg > (bs * 8) as u64
        || !ipg.is_multiple_of(8)
        || sb.inodes_count == 0
        || sb.inode_size < 128
        || usize::from(sb.inode_size) > bs
        || !sb.inode_size.is_power_of_two()
        || sb.blocks_count <= first
    {
        return Err(Error::Corrupt("block allocation geometry"));
    }
    let groups = sb.block_group_count();
    let end = fs.byte_ceiling()? / bs as u64;
    if groups > u64::from(u32::MAX)
        || groups != fs.groups.len() as u64
        || u64::from(sb.inodes_count).div_ceil(ipg) != groups
        || end != sb.blocks_count
    {
        return Err(Error::Corrupt("block allocation group coverage"));
    }
    if buf
        .uninit_cleared
        .last_key_value()
        .is_some_and(|(g, _)| *g as u64 >= groups)
    {
        return Err(Error::Corrupt(
            "block allocation pending group outside table",
        ));
    }
    if ranges.len() > limits.ranges || 2 * bs > limits.scratch_bytes {
        return Err(LIMIT);
    }
    let mut previous = first;
    for range in ranges {
        if range.start < previous || range.start >= range.end || range.end > end {
            return Err(Error::InvalidArgument(
                "block allocation interval ordering/bounds",
            ));
        }
        previous = range.end;
    }
    let mut scratch = Vec::new();
    scratch.try_reserve_exact(2 * bs).map_err(|_| LIMIT)?;
    scratch.resize(2 * bs, 0);
    let (descriptors, bitmap) = scratch.split_at_mut(bs);
    let mut reader = Reader {
        fs,
        buf,
        end,
        left: limits.reads,
    };
    let primary = crate::superblock::SUPERBLOCK_OFFSET / bs as u64;
    reader.block(primary, bitmap)?;
    let offset = (crate::superblock::SUPERBLOCK_OFFSET % bs as u64) as usize;
    let raw = &bitmap[offset..offset + 1024];
    if !fs.csum.verify_superblock(raw) {
        return Err(Error::BadChecksum { what: "superblock" });
    }
    let mounted = sb
        .raw
        .get(..0x3fc)
        .ok_or(Error::Corrupt("block allocation superblock size"))?;
    if raw[..0x3fc]
        .iter()
        .zip(mounted)
        .enumerate()
        .any(|(at, (now, old))| {
            !(0x0c..0x14).contains(&at) && !(0x158..0x15c).contains(&at) && now != old
        })
    {
        return Err(Error::Unsupported("block allocation changed superblock"));
    }
    let mut cached_descriptor = None;
    let mut cached_group = None;
    let mut groups_left = limits.groups;
    let mut work_left = limits.bitmap_bytes;
    for range in ranges {
        let mut cursor = range.start;
        while cursor < range.end {
            let group = (cursor - first) / bpg;
            // The bounded bpg and u32 group count make this product fit u64.
            let group_first = first + group * bpg;
            let group_end = group_first
                .checked_add(bpg)
                .ok_or(Error::InvalidBlock(cursor))?
                .min(end);
            if cached_group != Some(group) {
                groups_left = groups_left.checked_sub(1).ok_or(LIMIT)?;
                let (block, at) = sb.descriptor_location(group);
                if cached_descriptor != Some(block) {
                    reader.block(block, descriptors)?;
                    cached_descriptor = Some(block);
                }
                let raw = descriptors
                    .get(at..at + usize::from(sb.desc_size))
                    .ok_or(Error::Corrupt("block allocation descriptor bounds"))?;
                let current = BlockGroupDescriptor::parse(raw, sb.desc_size)?;
                if group_desc_csum(sb, &fs.csum, group as u32, raw) != Some(current.checksum) {
                    return Err(Error::BadChecksum {
                        what: "block group descriptor",
                    });
                }
                let mounted = &fs.groups[group as usize];
                if current.block_bitmap != mounted.block_bitmap
                    || current.inode_bitmap != mounted.inode_bitmap
                    || current.inode_table != mounted.inode_table
                {
                    return Err(Error::Unsupported("block allocation metadata relocation"));
                }
                if current.flags & !BgdFlags::all().bits() != 0 {
                    return Err(PROFILE);
                }
                if buf
                    .uninit_cleared
                    .get(&(group as usize))
                    .is_some_and(|f| *f != current.flags)
                {
                    return Err(Error::Corrupt("block allocation pending flags disagree"));
                }
                if current.flags().contains(BgdFlags::BLOCK_UNINIT) {
                    return Err(Error::Unsupported("block allocation uninitialized bitmap"));
                }
                let table_blocks = (ipg * u64::from(sb.inode_size)).div_ceil(bs as u64);
                for (address, length) in [
                    (current.block_bitmap, 1),
                    (current.inode_bitmap, 1),
                    (current.inode_table, table_blocks),
                ] {
                    if address < first.max(1)
                        || address.checked_add(length).is_none_or(|stop| stop > end)
                    {
                        return Err(Error::InvalidBlock(address));
                    }
                    if sb.feature_incompat & Incompat::FLEX_BG.bits() == 0
                        && (address < group_first || address + length > group_end)
                    {
                        return Err(Error::Corrupt(
                            "block allocation metadata outside own group",
                        ));
                    }
                }
                if current.block_bitmap == primary
                    || current.block_bitmap == block
                    || current.block_bitmap == current.inode_bitmap
                    || (current.inode_table..current.inode_table + table_blocks)
                        .contains(&current.block_bitmap)
                {
                    return Err(Error::Corrupt("block allocation bitmap metadata alias"));
                }
                work_left = work_left.checked_sub(2 * bs as u64).ok_or(LIMIT)?;
                reader.block(current.block_bitmap, bitmap)?;
                let crc = linux_crc32c(fs.csum.seed, &bitmap[..(bpg / 8) as usize]);
                let mask = if sb.desc_size == 64 { u32::MAX } else { 0xffff };
                if crc & mask != current.block_bitmap_csum {
                    return Err(Error::BadChecksum {
                        what: "block bitmap",
                    });
                }
                if !all_set(bitmap, group_end - group_first, (bs * 8) as u64) {
                    return Err(Error::Corrupt("block allocation clear padding bit"));
                }
                cached_group = Some(group);
            }
            let stop = range.end.min(group_end);
            let lo = cursor - group_first;
            let hi = stop - group_first;
            work_left = work_left
                .checked_sub(hi.div_ceil(8) - lo / 8)
                .ok_or(LIMIT)?;
            if !all_set(bitmap, lo, hi) {
                return Err(Error::Corrupt("block allocation clear proposed bit"));
            }
            cursor = stop;
        }
    }
    Ok(())
}

/// Test-only reader of complete dumpe2fs group free lists. Explicit counts and
/// coverage distinguish an allocated block zero from a missing/truncated list.
#[cfg(test)]
fn oracle_free_blocks(
    report: &str,
    first: u64,
    end: u64,
    bpg: u64,
) -> Result<std::collections::BTreeSet<u64>> {
    const BAD: Error = Error::Corrupt("incomplete or inconsistent oracle free-block report");
    if bpg == 0 || end <= first || end - first > 1_000_000 {
        return Err(BAD);
    }
    let number = |text: &str| text.trim().parse::<u64>().map_err(|_| BAD);
    let expected_groups = (end - first).div_ceil(bpg);
    let mut global = None;
    let mut groups = 0;
    let mut current = None;
    let mut group_count = None;
    let mut listed = false;
    let mut free = std::collections::BTreeSet::new();
    for line in report.lines() {
        if let Some(value) = line.strip_prefix("Free blocks:") {
            if global.replace(number(value)?).is_some() {
                return Err(BAD);
            }
        } else if let Some((group, bounds)) = line
            .strip_prefix("Group ")
            .and_then(|r| r.split_once(": (Blocks "))
        {
            if (current.is_some() && !listed)
                || groups >= expected_groups
                || number(group)? != groups
            {
                return Err(BAD);
            }
            let bounds = bounds.split_once(')').ok_or(BAD)?.0;
            let (lo, hi) = bounds.split_once('-').ok_or(BAD)?;
            let lo = number(lo)?;
            let hi = number(hi)?;
            let start = groups
                .checked_mul(bpg)
                .and_then(|n| first.checked_add(n))
                .ok_or(BAD)?;
            let stop = start.checked_add(bpg).ok_or(BAD)?.min(end);
            if lo != start || hi != stop - 1 {
                return Err(BAD);
            }
            current = Some((start, stop));
            groups += 1;
            group_count = None;
            listed = false;
        } else if let Some(value) = line.strip_prefix("  Free blocks:") {
            let (start, stop) = current.ok_or(BAD)?;
            if listed {
                return Err(BAD);
            }
            let before = free.len();
            if !value.trim().is_empty() {
                for item in value.trim().split(',') {
                    let (lo, hi) = match item.split_once('-') {
                        Some((a, b)) => (number(a)?, number(b)?),
                        None => {
                            let n = number(item)?;
                            (n, n)
                        }
                    };
                    if lo < start || hi >= stop || lo > hi {
                        return Err(BAD);
                    }
                    for block in lo..=hi {
                        if !free.insert(block) {
                            return Err(BAD);
                        }
                    }
                }
            }
            if Some((free.len() - before) as u64) != group_count {
                return Err(BAD);
            }
            listed = true;
        } else if current.is_some() {
            if let Some((count, _)) = line.trim_start().split_once(" free blocks,") {
                if group_count.replace(number(count)?).is_some() {
                    return Err(BAD);
                }
            }
        }
    }
    if groups != expected_groups || !listed || global != Some(free.len() as u64) {
        return Err(BAD);
    }
    Ok(free)
}

#[cfg(test)]
mod oracle_report_tests {
    use super::*;

    // Black-box dumpe2fs output from focused run 38787960237,
    // fixed-metadata-4096-32-0.locations.txt, with trailing whitespace trimmed.
    // This is a tool report, not tool implementation source.
    const CAPTURED: &str = include_str!("../tests/data/block-allocation-dumpe2fs.txt");

    #[test]
    fn captured_free_lists_include_the_block_zero_domain_and_fail_closed() {
        let free = oracle_free_blocks(CAPTURED, 0, 4096, 512).unwrap();
        assert_eq!(free.len(), 4051);
        assert!(!free.contains(&0));
        assert!(free.contains(&10) && free.contains(&4095));
        // A synthetic report alteration keeps the counts unchanged, but makes
        // zero explicitly free. Never silently classify it as always allocated.
        let changed = CAPTURED.replace("  Free blocks: 10-511", "  Free blocks: 0, 11-511");
        assert!(oracle_free_blocks(&changed, 0, 4096, 512)
            .unwrap()
            .contains(&0));
        for damaged in [
            CAPTURED.replace("  Free blocks: 10-511", ""),
            CAPTURED.replace("  Free blocks: 10-511", "  Free blocks: 10-511, 10"),
            CAPTURED.replace("  Free blocks: 10-511", "  Free blocks: 10-512"),
            CAPTURED.replace("  Free blocks: 10-511", "  Free blocks: 11-511"),
            CAPTURED.replace("Group 1: (Blocks 512-1023)", "Group 2: (Blocks 512-1023)"),
            CAPTURED.replace(
                "Free blocks:              4051",
                "Free blocks:              4050",
            ),
            CAPTURED.lines().take(60).collect::<Vec<_>>().join("\n"),
        ] {
            assert!(oracle_free_blocks(&damaged, 0, 4096, 512).is_err());
        }
    }

    #[test]
    fn empty_group_free_lists_require_an_explicit_zero_count() {
        // Synthetic report syntax control, not a filesystem qualification.
        let report = "Free blocks: 2\nGroup 0: (Blocks 0-7)\n  0 free blocks, 0 free inodes\n  Free blocks: \nGroup 1: (Blocks 8-15)\n  2 free blocks, 0 free inodes\n  Free blocks: 8, 15\n";
        assert_eq!(
            oracle_free_blocks(report, 0, 16, 8).unwrap(),
            [8, 15].into()
        );
        assert!(oracle_free_blocks(
            &report.replace("  0 free blocks", "  1 free blocks"),
            0,
            16,
            8
        )
        .is_err());
        assert!(oracle_free_blocks(&report.replace("  Free blocks: \n", ""), 0, 16, 8).is_err());
    }
}

#[cfg(test)]
mod needs_host {
    use super::*;

    #[test]
    fn linux_allocation_intervals_match_every_debugfs_block_bit() {
        use crate::block_io::FileDevice;
        use fs_ext4_test_support::{assert_e2fsck_clean, guest_kernel_write, oracle};
        use std::collections::BTreeMap;
        use std::sync::Arc;
        for bs in [1024u32, 4096] {
            for descriptor in [32, 64] {
                let base = fs_ext4_test_support::temp_dir()
                    .join(format!("block-allocation-{bs}-{descriptor}"));
                let image = base.with_extension("img");
                std::fs::File::create(&image)
                    .unwrap()
                    .set_len(2 * 1024 * 1024 - u64::from(bs))
                    .unwrap();
                let features = if descriptor == 64 {
                    "none,extent,filetype,metadata_csum,64bit"
                } else {
                    "none,extent,filetype,metadata_csum"
                };
                let made = oracle("mke2fs")
                    .args([
                        "-q",
                        "-F",
                        "-t",
                        "ext4",
                        "-b",
                        &bs.to_string(),
                        "-I",
                        "256",
                        "-N",
                        "64",
                        "-g",
                        if bs == 1024 { "1024" } else { "256" },
                        "-m",
                        "0",
                        "-O",
                        features,
                        "-E",
                        "lazy_itable_init=0",
                    ])
                    .arg(&image)
                    .output();
                assert!(
                    made.status.success(),
                    "{}",
                    String::from_utf8_lossy(&made.stderr)
                );
                let script = format!(
                    "python3 - <<'BLOCK_ALLOCATION_PY'\n{}\nBLOCK_ALLOCATION_PY\n",
                    include_str!("../tests/block_allocation_guest.py")
                );
                let populated = guest_kernel_write(image.to_str().unwrap(), &script);
                assert!(
                    populated.status.success(),
                    "{}",
                    String::from_utf8_lossy(&populated.stderr)
                );
                assert_e2fsck_clean(image.to_str().unwrap(), "Linux block allocation intervals");
                let fs =
                    Filesystem::mount(Arc::new(FileDevice::open(image.to_str().unwrap()).unwrap()))
                        .unwrap();
                assert_eq!(fs.sb.desc_size, descriptor);
                assert_eq!(fs.groups.len(), 2);
                assert!(fs
                    .groups
                    .iter()
                    .all(|g| !g.flags().contains(BgdFlags::BLOCK_UNINIT)));
                assert!(!(fs.sb.blocks_count - u64::from(fs.sb.first_data_block))
                    .is_multiple_of(u64::from(fs.sb.blocks_per_group)));
                let buf = BlockBuffer::new(bs);
                crate::fixed_metadata::collect(
                    &fs,
                    &buf,
                    crate::fixed_metadata::Limits {
                        groups: 2,
                        reads: 2,
                        ranges: 32,
                        scratch_bytes: 2 * bs as usize,
                    },
                )
                .unwrap();
                let first = u64::from(fs.sb.first_data_block);
                let end = fs.sb.blocks_count;
                let dumped = oracle("dumpe2fs").arg(&image).output();
                assert!(
                    dumped.status.success(),
                    "{}",
                    String::from_utf8_lossy(&dumped.stderr)
                );
                let free_report = String::from_utf8(dumped.stdout).unwrap();
                std::fs::write(base.with_extension("free-blocks.txt"), &free_report).unwrap();
                let free =
                    oracle_free_blocks(&free_report, first, end, u64::from(fs.sb.blocks_per_group))
                        .unwrap();
                let commands = base.with_extension("commands.txt");
                // Measured in run 38787960237: testb refuses zero, including
                // when the filesystem's allocation domain starts at zero.
                let queried_first = first.max(1);
                let text: String = (queried_first..end)
                    .map(|block| format!("testb {block}\n"))
                    .collect();
                std::fs::write(&commands, text).unwrap();
                let observed = oracle("debugfs")
                    .arg("-f")
                    .arg(&commands)
                    .arg(&image)
                    .output();
                assert!(
                    observed.status.success(),
                    "{}",
                    String::from_utf8_lossy(&observed.stderr)
                );
                let report = String::from_utf8(observed.stdout).unwrap();
                std::fs::write(base.with_extension("allocation.txt"), &report).unwrap();
                let mut answers = BTreeMap::new();
                for line in report.lines() {
                    let Some(rest) = line.trim().strip_prefix("Block ") else {
                        continue;
                    };
                    let (number, answer) = rest.split_once(' ').expect("block answer syntax");
                    let block: u64 = number.parse().unwrap();
                    let allocated = match answer {
                        "marked in use" | "is marked in use" => true,
                        "not in use" | "is not in use" => false,
                        _ => panic!("unrecognized block allocation answer: {line}"),
                    };
                    assert!(
                        block >= queried_first
                            && block < end
                            && answers.insert(block, allocated).is_none(),
                        "duplicate/outside block answer"
                    );
                }
                assert_eq!(
                    answers.len() as u64,
                    end - queried_first,
                    "incomplete debugfs allocation report"
                );
                for (&block, &allocated) in &answers {
                    assert_eq!(
                        allocated,
                        !free.contains(&block),
                        "tools disagree about block {block}"
                    );
                }
                if first == 0 {
                    // This is an observed allocation answer from the complete,
                    // count-checked free list, not a generic zero-block exception.
                    assert!(answers.insert(0, !free.contains(&0)).is_none());
                }
                assert_eq!(answers.len() as u64, end - first);
                let used = answers.values().filter(|allocated| **allocated).count();
                assert!(used > 0 && used < answers.len());
                let limits = Limits {
                    ranges: answers.len(),
                    groups: 2,
                    reads: 4,
                    bitmap_bytes: 8 * u64::from(bs),
                    scratch_bytes: 2 * bs as usize,
                };
                let mut runs: Vec<Range> = Vec::new();
                for block in first..end {
                    let range = Range {
                        start: block,
                        end: block + 1,
                    };
                    let actual = verify(&fs, &buf, &[range], limits);
                    if answers[&block] {
                        actual.unwrap();
                        match runs.last_mut() {
                            Some(previous) if previous.end == block => previous.end += 1,
                            _ => runs.push(range),
                        }
                    } else {
                        assert!(
                            matches!(
                                actual,
                                Err(Error::Corrupt("block allocation clear proposed bit"))
                            ),
                            "unexpected free-block result: {actual:?}"
                        );
                    }
                }
                verify(&fs, &buf, &runs, limits).unwrap();
                assert!(matches!(
                    verify(&fs, &buf, &[Range { start: first, end }], limits),
                    Err(Error::Corrupt("block allocation clear proposed bit"))
                ));
                if bs == 1024 {
                    assert!(matches!(
                        verify(&fs, &buf, &[Range { start: 0, end: 1 }], limits),
                        Err(Error::InvalidArgument(_))
                    ));
                } else {
                    assert!(answers[&0]);
                }
                std::fs::write(base.with_extension("summary.txt"), format!(
                    "block_size={bs} descriptor={descriptor} groups=2 answers={} used={used} runs={} first={first} end={end}\n",
                    answers.len(), runs.len(),
                )).unwrap();
            }
        }
    }
}
