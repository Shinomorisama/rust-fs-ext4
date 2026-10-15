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
                let commands = base.with_extension("commands.txt");
                let text: String = (first..end)
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
                        block >= first && block < end && answers.insert(block, allocated).is_none(),
                        "duplicate/outside block answer"
                    );
                }
                assert_eq!(
                    answers.len() as u64,
                    end - first,
                    "incomplete debugfs allocation report"
                );
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
