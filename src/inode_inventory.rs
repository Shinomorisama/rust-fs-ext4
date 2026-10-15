//! Read-only allocation inventory, not inode interpretation or ownership proof.
//! Public formats: https://docs.kernel.org/filesystems/ext4/bitmaps.html and
//! https://docs.kernel.org/filesystems/ext4/group_descr.html

use crate::bgd::{BgdFlags, BlockGroupDescriptor};
use crate::checksum::{group_desc_csum, linux_crc32c};
use crate::error::{Error, Result};
use crate::fs::{BlockBuffer, Filesystem};

#[derive(Clone, Copy)]
pub(crate) struct Limits {
    pub groups: u64,
    /// Every bit in each bitmap block, including padding, is examined.
    pub bit_probes: u64,
    /// Logical block fetches, including staged copies and failed attempts.
    pub reads: u64,
    pub inodes: usize,
    pub scratch_bytes: usize,
}

#[derive(Debug)]
pub(crate) struct Inventory {
    /// Ascending allocated IDs, including reserved and unlinked slots. Bodies
    /// and link counts are not read. Retained capacity must be budgeted by users.
    pub inodes: Vec<u32>,
}

const LIMIT: Error = Error::Unsupported("inode inventory resource limit");

struct Reader<'a> {
    fs: &'a Filesystem,
    buf: &'a BlockBuffer,
    limit: u64,
    reads_left: u64,
}

impl Reader<'_> {
    fn block(&mut self, block: u64, out: &mut [u8]) -> Result<()> {
        if block >= self.limit {
            return Err(Error::InvalidBlock(block));
        }
        self.reads_left = self.reads_left.checked_sub(1).ok_or(LIMIT)?;
        match self.buf.dirty.get(&block) {
            Some(bytes) if bytes.len() == out.len() => out.copy_from_slice(bytes),
            Some(_) => return Err(Error::Corrupt("inode inventory staged block size")),
            None => {
                let at = block
                    .checked_mul(self.fs.sb.block_size() as u64)
                    .ok_or(Error::InvalidBlock(block))?;
                self.fs.dev.read_at(at, out)?;
            }
        }
        Ok(())
    }
}

/// Enumerate current initialized inode bitmaps under one set of scan limits.
/// Requires a trusted mounted filesystem and a stable device/staged view.
/// No snapshot is cached or published; failures discard the entire local list.
/// Initial profile: metadata checksums, 1/4 KiB blocks, 32/64-byte descriptors,
/// no META_BG/bigalloc, and no INODE_UNINIT. Unknown/unqualified states refuse.
pub(crate) fn collect(fs: &Filesystem, buf: &BlockBuffer, limits: Limits) -> Result<Inventory> {
    use crate::features::{Incompat, RoCompat};
    let sb = &fs.sb;
    let bs = sb.block_size() as usize;
    if !fs.csum.enabled
        || sb.feature_ro_compat & RoCompat::METADATA_CSUM.bits() == 0
        || !matches!(bs, 1024 | 4096)
        || !matches!(sb.desc_size, 32 | 64)
        || sb.feature_incompat & Incompat::META_BG.bits() != 0
        || sb.feature_ro_compat & RoCompat::BIGALLOC.bits() != 0
    {
        return Err(Error::Unsupported(
            "inode inventory geometry/checksum profile",
        ));
    }
    let ipg = u64::from(sb.inodes_per_group);
    if sb.blocks_per_group == 0
        || ipg == 0
        || ipg > (bs * 8) as u64
        || !ipg.is_multiple_of(8)
        || sb.inodes_count == 0
        || sb.inode_size < 128
        || usize::from(sb.inode_size) > bs
        || !sb.inode_size.is_power_of_two()
    {
        return Err(Error::Corrupt("inode inventory group geometry"));
    }
    let groups = sb.block_group_count();
    let count = u64::from(sb.inodes_count);
    let block_limit = fs.byte_ceiling()? / bs as u64;
    if groups == 0
        || groups != fs.groups.len() as u64
        || count.div_ceil(ipg) != groups
        || block_limit != sb.blocks_count
    {
        return Err(Error::Corrupt("inode inventory group coverage"));
    }
    if buf
        .uninit_cleared
        .last_key_value()
        .is_some_and(|(group, _)| *group as u64 >= groups)
    {
        return Err(Error::Corrupt(
            "inode inventory pending group outside table",
        ));
    }
    let probes = groups.checked_mul((bs * 8) as u64).ok_or(LIMIT)?;
    let scratch_size = bs.checked_mul(2).ok_or(LIMIT)?;
    if groups > limits.groups || probes > limits.bit_probes || scratch_size > limits.scratch_bytes {
        return Err(LIMIT);
    }
    limits
        .inodes
        .checked_mul(std::mem::size_of::<u32>())
        .and_then(|bytes| bytes.checked_add(scratch_size))
        .ok_or(LIMIT)?;
    let mut inodes = Vec::new();
    inodes.try_reserve_exact(limits.inodes).map_err(|_| LIMIT)?;
    let mut scratch = Vec::new();
    scratch.try_reserve_exact(scratch_size).map_err(|_| LIMIT)?;
    scratch.resize(scratch_size, 0);
    let (descriptor_bytes, bitmap) = scratch.split_at_mut(bs);
    let mut reader = Reader {
        fs,
        buf,
        limit: block_limit,
        reads_left: limits.reads,
    };
    let sb_block = crate::superblock::SUPERBLOCK_OFFSET / bs as u64;
    reader.block(sb_block, bitmap)?;
    let offset = (crate::superblock::SUPERBLOCK_OFFSET % bs as u64) as usize;
    let raw = &bitmap[offset..offset + 1024];
    if !fs.csum.verify_superblock(raw) {
        return Err(Error::BadChecksum { what: "superblock" });
    }
    let mounted = sb
        .raw
        .get(..0x3fc)
        .ok_or(Error::Corrupt("inode inventory superblock size"))?;
    if raw[..0x3fc]
        .iter()
        .zip(mounted)
        .enumerate()
        .any(|(at, (now, old))| {
            !(0x0c..0x14).contains(&at) && !(0x158..0x15c).contains(&at) && now != old
        })
    {
        return Err(Error::Unsupported("inode inventory changed superblock"));
    }

    let mut cached_descriptor = None;
    for group in 0..groups {
        let (block, at) = sb.descriptor_location(group);
        if cached_descriptor != Some(block) {
            reader.block(block, descriptor_bytes)?;
            cached_descriptor = Some(block);
        }
        let raw = descriptor_bytes
            .get(at..at + usize::from(sb.desc_size))
            .ok_or(Error::Corrupt("inode inventory descriptor bounds"))?;
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
            return Err(Error::Unsupported("inode inventory metadata relocation"));
        }
        if current.flags & !BgdFlags::all().bits() != 0 {
            return Err(Error::Unsupported(
                "inode inventory unknown allocation flags",
            ));
        }
        if buf
            .uninit_cleared
            .get(&(group as usize))
            .is_some_and(|flags| *flags != current.flags)
        {
            return Err(Error::Corrupt("inode inventory pending flags disagree"));
        }
        if current.flags().contains(BgdFlags::INODE_UNINIT) {
            return Err(Error::Unsupported("inode inventory uninitialized bitmap"));
        }
        let table_blocks = (ipg * u64::from(sb.inode_size)).div_ceil(bs as u64);
        for address in [
            current.block_bitmap,
            current.inode_bitmap,
            current.inode_table,
        ] {
            if address < u64::from(sb.first_data_block).max(1) || address >= block_limit {
                return Err(Error::InvalidBlock(address));
            }
        }
        if current
            .inode_table
            .checked_add(table_blocks)
            .is_none_or(|end| end > block_limit)
        {
            return Err(Error::InvalidBlock(current.inode_table));
        }
        if current.inode_bitmap == block
            || current.inode_bitmap == sb_block
            || current.inode_bitmap == current.block_bitmap
            || (current.inode_table..current.inode_table + table_blocks)
                .contains(&current.inode_bitmap)
        {
            return Err(Error::Corrupt("inode inventory bitmap metadata alias"));
        }
        reader.block(current.inode_bitmap, bitmap)?;
        let crc = linux_crc32c(fs.csum.seed, &bitmap[..(ipg / 8) as usize]);
        let mask = if sb.desc_size == 64 { u32::MAX } else { 0xffff };
        if crc & mask != current.inode_bitmap_csum {
            return Err(Error::BadChecksum {
                what: "inode bitmap",
            });
        }
        let valid = (count - group * ipg).min(ipg);
        for bit in 0..(bs * 8) as u64 {
            let set = bitmap[bit as usize / 8] & (1 << (bit % 8)) != 0;
            if bit >= valid {
                if !set {
                    return Err(Error::Corrupt("inode inventory clear padding bit"));
                }
            } else if set {
                if inodes.len() == limits.inodes {
                    return Err(LIMIT);
                }
                inodes.push((group * ipg + bit + 1) as u32);
            }
        }
    }
    Ok(Inventory { inodes })
}

#[cfg(test)]
mod needs_host {
    use super::*;

    #[test]
    fn linux_inventory_matches_every_debugfs_allocation_bit() {
        use crate::block_io::FileDevice;
        use fs_ext4_test_support::{assert_e2fsck_clean, guest_kernel_write, oracle};
        use std::sync::Arc;
        for bs in [1024u32, 4096] {
            for descriptor in [32, 64] {
                let base = fs_ext4_test_support::temp_dir()
                    .join(format!("inode-inventory-{bs}-{descriptor}"));
                let image = base.with_extension("img");
                std::fs::File::create(&image)
                    .unwrap()
                    .set_len(8 * 1024 * 1024)
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
                        if bs == 1024 { "4096" } else { "1024" },
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
                let image = image.to_str().unwrap();
                let script = format!(
                    "python3 - <<'INODE_INVENTORY_PY'\n{}\nINODE_INVENTORY_PY\n",
                    include_str!("../tests/inode_inventory_guest.py")
                );
                let populated = guest_kernel_write(image, &script);
                assert!(
                    populated.status.success(),
                    "{}",
                    String::from_utf8_lossy(&populated.stderr)
                );
                assert_e2fsck_clean(image, "Linux inode inventory control");
                let fs = Filesystem::mount(Arc::new(FileDevice::open(image).unwrap())).unwrap();
                assert_eq!(fs.sb.desc_size, descriptor);
                assert_eq!(fs.groups.len(), 2);
                assert_eq!(fs.sb.inodes_per_group, 32);
                assert!(fs
                    .groups
                    .iter()
                    .all(|group| !group.flags().contains(BgdFlags::INODE_UNINIT)));
                let count = fs.sb.inodes_count;
                let commands = base.with_extension("commands.txt");
                let text: String = (1..=count).map(|ino| format!("testi <{ino}>\n")).collect();
                std::fs::write(&commands, text).unwrap();
                let observed = oracle("debugfs")
                    .arg("-f")
                    .arg(&commands)
                    .arg(image)
                    .output();
                assert!(
                    observed.status.success(),
                    "{}",
                    String::from_utf8_lossy(&observed.stderr)
                );
                let report = String::from_utf8(observed.stdout).unwrap();
                std::fs::write(base.with_extension("allocation.txt"), &report).unwrap();
                let mut expected = Vec::new();
                for ino in 1..=count {
                    let used = format!("Inode {ino} is marked in use");
                    let free = format!("Inode {ino} is not in use");
                    let used_count = report.lines().filter(|line| line.trim() == used).count();
                    let free_count = report.lines().filter(|line| line.trim() == free).count();
                    assert_eq!(
                        used_count + free_count,
                        1,
                        "missing/duplicate answer for inode {ino}: {report}"
                    );
                    if used_count == 1 {
                        expected.push(ino);
                    }
                }
                assert!((1..fs.sb.first_inode).all(|ino| expected.contains(&ino)));
                assert!(expected.len() > 11 && expected.len() < count as usize);
                let result = collect(
                    &fs,
                    &BlockBuffer::new(bs),
                    Limits {
                        groups: 2,
                        bit_probes: 2 * u64::from(bs) * 8,
                        reads: 4,
                        inodes: count as usize,
                        scratch_bytes: 2 * bs as usize,
                    },
                )
                .unwrap();
                assert_eq!(
                    result.inodes, expected,
                    "Linux bitmap inventory {bs}/{descriptor}"
                );
            }
        }
    }
}
