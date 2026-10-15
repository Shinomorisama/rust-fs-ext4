//! Private fixed-layout protection ranges, not allocation or ownership proof.
//! Formats: https://docs.kernel.org/filesystems/ext4/overview.html and
//! https://docs.kernel.org/filesystems/ext4/group_descr.html

use crate::bgd::{BgdFlags, BlockGroupDescriptor};
use crate::checksum::group_desc_csum;
use crate::error::{Error, Result};
use crate::features::{Compat, Incompat, RoCompat};
use crate::fs::{BlockBuffer, Filesystem};

#[derive(Clone, Copy)]
pub(crate) struct Limits {
    pub groups: u64,
    /// Logical primary superblock/descriptor fetches, including staged copies.
    pub reads: u64,
    pub ranges: usize,
    pub scratch_bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    Boot,
    /// At 4 KiB, primary block zero includes the boot bytes too.
    Superblock,
    Descriptors,
    ReservedDescriptors,
    BlockBitmap,
    InodeBitmap,
    InodeTable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Range {
    pub start: u64,
    pub end: u64,
    pub role: Role,
    pub group: u32,
}

#[derive(Debug)]
pub(crate) struct Layout {
    /// Physically sorted, nonoverlapping. Retained capacity needs caller accounting.
    pub ranges: Vec<Range>,
}

const LIMIT: Error = Error::Unsupported("fixed metadata resource limit");
const PROFILE: Error = Error::Unsupported("fixed metadata layout profile");

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
            Some(_) => return Err(Error::Corrupt("fixed metadata staged block size")),
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

fn push(
    ranges: &mut Vec<Range>,
    cap: usize,
    end: u64,
    start: u64,
    length: u64,
    role: Role,
    group: u32,
) -> Result<()> {
    let stop = start
        .checked_add(length)
        .ok_or(Error::InvalidBlock(start))?;
    if length == 0 || stop > end {
        return Err(Error::InvalidBlock(start));
    }
    if ranges.len() == cap {
        return Err(LIMIT);
    }
    ranges.push(Range {
        start,
        end: stop,
        role,
        group,
    });
    Ok(())
}

/// Read current primary layout metadata on a trusted mount and stable device/
/// buffer view. Backups are reserved by the validated primary geometry, not
/// treated as authorities or required to contain identical current counters.
/// No bitmap contents, inode bodies or journal storage are read. Known uninit
/// flags do not remove physical reservations. No snapshot/epoch is published.
pub(crate) fn collect(fs: &Filesystem, buf: &BlockBuffer, limits: Limits) -> Result<Layout> {
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
        || (sb.reserved_gdt_blocks != 0 && sb.feature_compat & Compat::RESIZE_INODE.bits() == 0)
    {
        return Err(PROFILE);
    }
    let ipg = u64::from(sb.inodes_per_group);
    let bpg = u64::from(sb.blocks_per_group);
    if bpg == 0
        || bpg > (bs * 8) as u64
        || ipg == 0
        || ipg > (bs * 8) as u64
        || !ipg.is_multiple_of(8)
        || sb.inodes_count == 0
        || sb.inode_size < 128
        || usize::from(sb.inode_size) > bs
        || !sb.inode_size.is_power_of_two()
    {
        return Err(Error::Corrupt("fixed metadata geometry"));
    }
    let groups = sb.block_group_count();
    let end = fs.byte_ceiling()? / bs as u64;
    if groups == 0
        || groups > u64::from(u32::MAX)
        || groups != fs.groups.len() as u64
        || u64::from(sb.inodes_count).div_ceil(ipg) != groups
        || end != sb.blocks_count
    {
        return Err(Error::Corrupt("fixed metadata group coverage"));
    }
    if groups > limits.groups || 2 * bs > limits.scratch_bytes {
        return Err(LIMIT);
    }
    if sb.feature_compat & Compat::SPARSE_SUPER2.bits() != 0 {
        let [a, b] = sb.backup_bgs;
        if [a, b]
            .iter()
            .any(|g| *g != u32::MAX && (*g == 0 || u64::from(*g) >= groups))
            || (a != u32::MAX && a == b)
        {
            return Err(PROFILE);
        }
    }
    if buf
        .uninit_cleared
        .last_key_value()
        .is_some_and(|(g, _)| *g as u64 >= groups)
    {
        return Err(Error::Corrupt("fixed metadata pending group outside table"));
    }
    limits
        .ranges
        .checked_mul(std::mem::size_of::<Range>())
        .and_then(|n| n.checked_add(2 * bs))
        .ok_or(LIMIT)?;
    let mut ranges = Vec::new();
    ranges.try_reserve_exact(limits.ranges).map_err(|_| LIMIT)?;
    let mut scratch = Vec::new();
    scratch.try_reserve_exact(2 * bs).map_err(|_| LIMIT)?;
    scratch.resize(2 * bs, 0);
    let (superblock, descriptors) = scratch.split_at_mut(bs);
    let mut reader = Reader {
        fs,
        buf,
        end,
        left: limits.reads,
    };
    let primary = crate::superblock::SUPERBLOCK_OFFSET / bs as u64;
    reader.block(primary, superblock)?;
    let offset = (crate::superblock::SUPERBLOCK_OFFSET % bs as u64) as usize;
    let raw = &superblock[offset..offset + 1024];
    if !fs.csum.verify_superblock(raw) {
        return Err(Error::BadChecksum { what: "superblock" });
    }
    let mounted = sb
        .raw
        .get(..0x3fc)
        .ok_or(Error::Corrupt("fixed metadata superblock size"))?;
    if raw[..0x3fc]
        .iter()
        .zip(mounted)
        .enumerate()
        .any(|(at, (now, old))| {
            !(0x0c..0x14).contains(&at) && !(0x158..0x15c).contains(&at) && now != old
        })
    {
        return Err(Error::Unsupported("fixed metadata changed superblock"));
    }
    if bs == 1024 {
        push(&mut ranges, limits.ranges, end, 0, 1, Role::Boot, 0)?;
    }
    let descriptor_blocks = groups.div_ceil((bs / usize::from(sb.desc_size)) as u64);
    let table_blocks = (ipg * u64::from(sb.inode_size)).div_ceil(bs as u64);
    let mut cached = None;
    for group in 0..groups {
        let first = u64::from(sb.first_data_block) + group * bpg;
        let group_end = first
            .checked_add(bpg)
            .ok_or(Error::InvalidBlock(first))?
            .min(end);
        if sb.group_has_super(group) {
            let length = 1 + descriptor_blocks + u64::from(sb.reserved_gdt_blocks);
            if first
                .checked_add(length)
                .is_none_or(|stop| stop > group_end)
            {
                return Err(Error::Corrupt("fixed metadata group head bounds"));
            }
            push(
                &mut ranges,
                limits.ranges,
                end,
                first,
                1,
                Role::Superblock,
                group as u32,
            )?;
            push(
                &mut ranges,
                limits.ranges,
                end,
                first + 1,
                descriptor_blocks,
                Role::Descriptors,
                group as u32,
            )?;
            if sb.reserved_gdt_blocks != 0 {
                push(
                    &mut ranges,
                    limits.ranges,
                    end,
                    first + 1 + descriptor_blocks,
                    u64::from(sb.reserved_gdt_blocks),
                    Role::ReservedDescriptors,
                    group as u32,
                )?;
            }
        }
        let (block, at) = sb.descriptor_location(group);
        if cached != Some(block) {
            reader.block(block, descriptors)?;
            cached = Some(block);
        }
        let raw = descriptors
            .get(at..at + usize::from(sb.desc_size))
            .ok_or(Error::Corrupt("fixed metadata descriptor bounds"))?;
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
            return Err(Error::Unsupported("fixed metadata relocation"));
        }
        if current.flags & !BgdFlags::all().bits() != 0 {
            return Err(PROFILE);
        }
        if buf
            .uninit_cleared
            .get(&(group as usize))
            .is_some_and(|f| *f != current.flags)
        {
            return Err(Error::Corrupt("fixed metadata pending flags disagree"));
        }
        for (start, length, role) in [
            (current.block_bitmap, 1, Role::BlockBitmap),
            (current.inode_bitmap, 1, Role::InodeBitmap),
            (current.inode_table, table_blocks, Role::InodeTable),
        ] {
            if start < u64::from(sb.first_data_block).max(1) {
                return Err(Error::InvalidBlock(start));
            }
            if sb.feature_incompat & Incompat::FLEX_BG.bits() == 0
                && (start < first
                    || start
                        .checked_add(length)
                        .is_none_or(|stop| stop > group_end))
            {
                return Err(Error::Corrupt("fixed metadata outside own group"));
            }
            push(
                &mut ranges,
                limits.ranges,
                end,
                start,
                length,
                role,
                group as u32,
            )?;
        }
    }
    ranges.sort_unstable_by_key(|r| (r.start, r.end));
    if ranges.windows(2).any(|pair| pair[0].end > pair[1].start) {
        return Err(Error::Corrupt("fixed metadata role overlap"));
    }
    Ok(Layout { ranges })
}

#[cfg(test)]
mod needs_host {
    use super::*;

    fn observed_range(text: &str, role: Role, group: u32) -> Range {
        let token = text
            .split_whitespace()
            .next()
            .unwrap()
            .trim_end_matches(',');
        let (first, last) = token.split_once('-').unwrap_or((token, token));
        let start = first.parse::<u64>().unwrap();
        let end = last.parse::<u64>().unwrap().checked_add(1).unwrap();
        Range {
            start,
            end,
            role,
            group,
        }
    }

    #[test]
    fn linux_locations_match_complete_fixed_metadata_ranges() {
        use crate::block_io::FileDevice;
        use fs_ext4_test_support::{assert_e2fsck_clean, oracle};
        use std::collections::BTreeSet;
        use std::sync::Arc;
        for bs in [1024u32, 4096] {
            for descriptor in [32, 64] {
                for profile in 0..if descriptor == 64 { 4 } else { 3 } {
                    let many = profile == 3;
                    let base = fs_ext4_test_support::temp_dir()
                        .join(format!("fixed-metadata-{bs}-{descriptor}-{profile}"));
                    let image = base.with_extension("img");
                    let mib = if many && bs == 4096 { 68 } else { 16 };
                    std::fs::File::create(&image)
                        .unwrap()
                        .set_len(mib * 1024 * 1024)
                        .unwrap();
                    let mut features = String::from("none,extent,filetype,metadata_csum");
                    if descriptor == 64 {
                        features.push_str(",64bit");
                    }
                    match profile {
                        1 => features.push_str(",sparse_super,resize_inode,flex_bg"),
                        2 => features.push_str(",sparse_super2,flex_bg"),
                        _ => {}
                    }
                    let bpg = match (bs, many) {
                        (1024, false) => 2048,
                        (4096, false) => 512,
                        (1024, true) => 512,
                        _ => 256,
                    };
                    let inodes = if many {
                        if bs == 1024 {
                            256
                        } else {
                            1088
                        }
                    } else {
                        128
                    };
                    let mut command = oracle("mke2fs");
                    command = command.args([
                        "-q",
                        "-F",
                        "-t",
                        "ext4",
                        "-b",
                        &bs.to_string(),
                        "-I",
                        "256",
                        "-N",
                        &inodes.to_string(),
                        "-g",
                        &bpg.to_string(),
                        "-m",
                        "0",
                        "-O",
                        &features,
                        "-E",
                        "lazy_itable_init=0",
                    ]);
                    if profile == 1 || profile == 2 {
                        command = command.args(["-G", "4"]);
                    }
                    let made = command.arg(&image).output();
                    assert!(
                        made.status.success(),
                        "{}",
                        String::from_utf8_lossy(&made.stderr)
                    );
                    let image = image.to_str().unwrap();
                    assert_e2fsck_clean(image, "fixed metadata location control");
                    let report = oracle("dumpe2fs").arg(image).output();
                    assert!(
                        report.status.success(),
                        "{}",
                        String::from_utf8_lossy(&report.stderr)
                    );
                    let report = String::from_utf8(report.stdout).unwrap();
                    std::fs::write(base.with_extension("locations.txt"), &report).unwrap();
                    let fs = Filesystem::mount(Arc::new(FileDevice::open(image).unwrap())).unwrap();
                    assert_eq!(fs.sb.desc_size, descriptor);
                    assert_eq!(
                        fs.groups.len(),
                        if many {
                            if bs == 1024 {
                                32
                            } else {
                                68
                            }
                        } else {
                            8
                        }
                    );
                    let mut expected = Vec::new();
                    if bs == 1024 {
                        expected.push(Range {
                            start: 0,
                            end: 1,
                            role: Role::Boot,
                            group: 0,
                        });
                    }
                    let mut group = None;
                    let mut groups = BTreeSet::new();
                    for line in report.lines().map(str::trim) {
                        if let Some((number, _)) =
                            line.strip_prefix("Group ").and_then(|s| s.split_once(':'))
                        {
                            if let Ok(number) = number.parse::<u32>() {
                                assert!(groups.insert(number));
                                group = Some(number);
                            }
                        }
                        if let Some(text) = line
                            .strip_prefix("Primary superblock at ")
                            .or_else(|| line.strip_prefix("Backup superblock at "))
                        {
                            let (sb, gdt) = text
                                .split_once(", Group descriptors at ")
                                .expect("missing descriptor report");
                            expected.push(observed_range(sb, Role::Superblock, group.unwrap()));
                            expected.push(observed_range(gdt, Role::Descriptors, group.unwrap()));
                        }
                        for (prefix, role) in [
                            ("Reserved GDT blocks at ", Role::ReservedDescriptors),
                            ("Block bitmap at ", Role::BlockBitmap),
                            ("Inode bitmap at ", Role::InodeBitmap),
                            ("Inode table at ", Role::InodeTable),
                        ] {
                            if let Some(text) = line.strip_prefix(prefix) {
                                expected.push(observed_range(text, role, group.unwrap()));
                            }
                        }
                    }
                    assert_eq!(
                        groups.into_iter().collect::<Vec<_>>(),
                        (0..fs.groups.len() as u32).collect::<Vec<_>>()
                    );
                    expected.sort_unstable_by_key(|r| (r.start, r.end));
                    let got = collect(
                        &fs,
                        &BlockBuffer::new(bs),
                        Limits {
                            groups: fs.groups.len() as u64,
                            reads: 1 + fs.groups.len() as u64,
                            ranges: 512,
                            scratch_bytes: 2 * bs as usize,
                        },
                    )
                    .unwrap();
                    assert_eq!(
                        got.ranges, expected,
                        "metadata locations {bs}/{descriptor}/{profile}"
                    );
                    if profile == 1 {
                        assert!(got
                            .ranges
                            .iter()
                            .any(|r| r.role == Role::ReservedDescriptors));
                    }
                    if profile == 1 || profile == 2 {
                        assert!(got.ranges.iter().any(|r| r.role == Role::InodeTable
                            && r.start
                                < u64::from(fs.sb.first_data_block)
                                    + u64::from(r.group) * u64::from(fs.sb.blocks_per_group)));
                    }
                    if many {
                        assert!(got
                            .ranges
                            .iter()
                            .any(|r| r.role == Role::Descriptors && r.end - r.start > 1));
                    }
                }
            }
        }
    }
}
