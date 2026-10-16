//! Complete extent-ownership collection, separate from write admission.
//! Format: https://docs.kernel.org/filesystems/ext4/blockmap.html#extent-tree

use super::{Extent, ExtentHeader, ExtentIdx, ExtentVerifyCtx, EXT4_EXT_MAX_DEPTH};
use crate::block_io::BlockDevice;
use crate::error::{Error, Result};

/// Explicit caller limits, not defaults qualified for a real card. Output
/// allocation requests are bounded by `claims * size_of::<Claim>()`; scratch
/// holds at most `depth * block_size` bytes. Five external levels use at most
/// six walk frames, including the inline root.
#[derive(Clone, Copy)]
pub(crate) struct Limits {
    pub node_reads: usize,
    pub records: usize,
    pub claims: usize,
    pub scratch_bytes: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct Geometry {
    pub block_size: u32,
    pub first_block: u64,
    pub end_block: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    Data { logical: u64, unwritten: bool },
    Mapping,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Claim {
    pub start: u64,
    pub end: u64,
    pub role: Role,
}

#[derive(Debug)]
pub(crate) struct Tree {
    pub ino: u32,
    pub generation: u32,
    /// Sorted by physical address; no two claims overlap.
    pub claims: Vec<Claim>,
    /// Successful traversal consumption, for a caller's aggregate budget.
    pub usage: Usage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Usage {
    pub node_reads: usize,
    pub records: usize,
}

const LIMIT: Error = Error::Unsupported("extent ownership resource limit");

fn reserve_zeroed(size: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(size).map_err(|_| LIMIT)?;
    bytes.resize(size, 0);
    Ok(bytes)
}

/// Read every record and child, not only the path of a logical lookup.
///
/// The caller supplies a verified inode root and its checksum identity, and a
/// stable device/staged view with filesystem bounds. This only proves the
/// extent tree's bounded shape and internal role separation: it does not check
/// allocation, fixed metadata, attributes, other inodes or transaction stability.
/// The initial profile requires metadata checksums and 1/4 KiB blocks with full
/// external-node capacity, matching the existing extent-tail verifier's layout.
pub(crate) fn collect(
    root: &[u8],
    dev: &dyn BlockDevice,
    ctx: &ExtentVerifyCtx<'_>,
    geometry: Geometry,
    limits: Limits,
) -> Result<Tree> {
    if !ctx.csum.enabled || !matches!(geometry.block_size, 1024 | 4096) {
        return Err(Error::Unsupported(
            "extent ownership checksum/block profile",
        ));
    }
    if ctx.ino == 0 || root.len() != 60 {
        return Err(Error::CorruptExtentTree("extent ownership inode root"));
    }
    let bs = geometry.block_size as usize;
    let geometry = Geometry {
        first_block: geometry.first_block.max(1),
        end_block: geometry.end_block.min(dev.size_bytes() / bs as u64),
        ..geometry
    };
    if geometry.first_block >= geometry.end_block {
        return Err(Error::InvalidArgument("extent ownership geometry"));
    }
    let header = ExtentHeader::parse(root)?;
    if header.depth > EXT4_EXT_MAX_DEPTH {
        return Err(Error::CorruptExtentTree("extent ownership depth"));
    }
    let scratch_size = bs.checked_mul(header.depth as usize).ok_or(LIMIT)?;
    if scratch_size > limits.scratch_bytes {
        return Err(LIMIT);
    }
    // Check byte arithmetic before allocation. These capacities remain fixed
    // throughout the walk; neither recursive frames nor pushes grow a Vec.
    limits
        .claims
        .checked_mul(std::mem::size_of::<Claim>())
        .and_then(|bytes| bytes.checked_add(scratch_size))
        .ok_or(LIMIT)?;
    let mut claims = Vec::new();
    claims.try_reserve_exact(limits.claims).map_err(|_| LIMIT)?;
    let mut scratch = reserve_zeroed(scratch_size)?;
    let mut walker = Walker {
        dev,
        ctx,
        geometry,
        limits,
        claims,
        reads: 0,
        records: 0,
    };
    walker.walk(root, &mut scratch, header.depth, 0, 1u64 << 32, true)?;
    // In-place sorting allocates no scratch and has bounded O(C log C) work.
    // This also detects repeated children and data/map aliases across branches.
    walker
        .claims
        .sort_unstable_by_key(|claim| (claim.start, claim.end));
    if walker
        .claims
        .windows(2)
        .any(|pair| pair[0].end > pair[1].start)
    {
        return Err(Error::CorruptExtentTree(
            "extent ownership physical overlap",
        ));
    }
    Ok(Tree {
        ino: ctx.ino,
        generation: ctx.generation,
        usage: Usage {
            node_reads: walker.reads,
            records: walker.records,
        },
        claims: walker.claims,
    })
}

struct Walker<'a, 'c> {
    dev: &'a dyn BlockDevice,
    ctx: &'a ExtentVerifyCtx<'c>,
    geometry: Geometry,
    limits: Limits,
    claims: Vec<Claim>,
    reads: usize,
    records: usize,
}

impl Walker<'_, '_> {
    fn claim(&mut self, start: u64, length: u64, role: Role) -> Result<()> {
        let end = start
            .checked_add(length)
            .ok_or(Error::InvalidBlock(start))?;
        if length == 0 || start < self.geometry.first_block || end > self.geometry.end_block {
            return Err(Error::InvalidBlock(start));
        }
        if self.claims.len() == self.limits.claims {
            return Err(LIMIT);
        }
        self.claims.push(Claim { start, end, role });
        Ok(())
    }

    fn walk(
        &mut self,
        bytes: &[u8],
        scratch: &mut [u8],
        depth: u16,
        lower: u64,
        upper: u64,
        root: bool,
    ) -> Result<()> {
        let header = ExtentHeader::parse(bytes)?;
        if header.depth != depth {
            return Err(Error::CorruptExtentTree("extent ownership child depth"));
        }
        let capacity = if root { 4 } else { (bytes.len() - 16) / 12 };
        if header.max == 0 || header.max as usize > capacity {
            return Err(Error::CorruptExtentTree("extent ownership capacity"));
        }
        if !root && header.max as usize != capacity {
            return Err(Error::Unsupported(
                "extent ownership external capacity profile",
            ));
        }
        if header.entries == 0 && (!root || depth != 0) {
            return Err(Error::CorruptExtentTree(
                "extent ownership empty child/index",
            ));
        }
        self.records = self
            .records
            .checked_add(header.entries as usize)
            .ok_or(LIMIT)?;
        if self.records > self.limits.records {
            return Err(LIMIT);
        }
        let mut previous_end = lower;
        for slot in 0..usize::from(header.entries) {
            let at = 12 + slot * 12;
            if depth == 0 {
                let extent = Extent::parse(&bytes[at..at + 12])?;
                let logical = u64::from(extent.logical_block);
                let end = logical + u64::from(extent.length);
                if extent.length == 0
                    || logical < previous_end
                    || end > upper
                    || (!root && slot == 0 && logical != lower)
                {
                    return Err(Error::CorruptExtentTree(
                        "extent ownership logical coverage",
                    ));
                }
                previous_end = end;
                self.claim(
                    extent.physical_block,
                    u64::from(extent.length),
                    Role::Data {
                        logical,
                        unwritten: extent.uninitialized,
                    },
                )?;
            } else {
                let index = ExtentIdx::parse(&bytes[at..at + 12])?;
                let logical = u64::from(index.logical_block);
                let next = if slot + 1 < usize::from(header.entries) {
                    u64::from(u32::from_le_bytes(
                        bytes[at + 12..at + 16].try_into().unwrap(),
                    ))
                } else {
                    upper
                };
                if logical < lower
                    || logical >= next
                    || next > upper
                    || (!root && slot == 0 && logical != lower)
                {
                    return Err(Error::CorruptExtentTree("extent ownership index keys"));
                }
                self.claim(index.leaf_block, 1, Role::Mapping)?;
                if self.reads == self.limits.node_reads {
                    return Err(LIMIT);
                }
                self.reads += 1;
                let (child, rest) = scratch.split_at_mut(self.geometry.block_size as usize);
                let offset = index
                    .leaf_block
                    .checked_mul(u64::from(self.geometry.block_size))
                    .ok_or(Error::InvalidBlock(index.leaf_block))?;
                self.dev.read_at(offset, child)?;
                if !self
                    .ctx
                    .csum
                    .verify_extent_tail(self.ctx.ino, self.ctx.generation, child)
                {
                    return Err(Error::BadChecksum {
                        what: "extent block",
                    });
                }
                self.walk(child, rest, depth - 1, logical, next, false)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::block_io::BlockDevice;
    use crate::checksum::Checksummer;
    use crate::error::Result;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const BS: usize = 1024;
    const INO: u32 = 12;
    const GENERATION: u32 = 7;

    struct Fixture {
        root: Vec<u8>,
        bytes: Vec<u8>,
        reads: AtomicUsize,
        csum: Checksummer,
    }

    impl BlockDevice for Fixture {
        fn read_at(&self, offset: u64, out: &mut [u8]) -> Result<()> {
            self.reads.fetch_add(1, Ordering::Relaxed);
            let start = offset as usize;
            let src = self
                .bytes
                .get(start..start + out.len())
                .ok_or(crate::Error::OutOfBounds)?;
            out.copy_from_slice(src);
            Ok(())
        }
        fn size_bytes(&self) -> u64 {
            self.bytes.len() as u64
        }
    }

    fn header(bytes: &mut [u8], depth: u16, entries: u16, max: u16) {
        bytes[..2].copy_from_slice(&crate::extent::EXT4_EXT_MAGIC.to_le_bytes());
        bytes[2..4].copy_from_slice(&entries.to_le_bytes());
        bytes[4..6].copy_from_slice(&max.to_le_bytes());
        bytes[6..8].copy_from_slice(&depth.to_le_bytes());
    }
    fn data(bytes: &mut [u8], index: usize, logical: u32, length: u16, physical: u32) {
        let at = 12 + index * 12;
        bytes[at..at + 4].copy_from_slice(&logical.to_le_bytes());
        bytes[at + 4..at + 6].copy_from_slice(&length.to_le_bytes());
        bytes[at + 8..at + 12].copy_from_slice(&physical.to_le_bytes());
    }
    fn index(bytes: &mut [u8], slot: usize, logical: u32, physical: u32) {
        let at = 12 + slot * 12;
        bytes[at..at + 4].copy_from_slice(&logical.to_le_bytes());
        bytes[at + 4..at + 8].copy_from_slice(&physical.to_le_bytes());
    }
    fn fixture() -> Fixture {
        let mut f = Fixture {
            root: vec![0; 60],
            bytes: vec![0; 32 * BS],
            reads: AtomicUsize::new(0),
            csum: Checksummer {
                seed: 0x12345678,
                enabled: true,
            },
        };
        header(&mut f.root, 1, 2, 4);
        index(&mut f.root, 0, 0, 2);
        index(&mut f.root, 1, 10, 3);
        header(&mut f.bytes[2 * BS..3 * BS], 0, 1, 84);
        data(&mut f.bytes[2 * BS..3 * BS], 0, 0, 1, 20);
        header(&mut f.bytes[3 * BS..4 * BS], 0, 1, 84);
        data(&mut f.bytes[3 * BS..4 * BS], 0, 10, 1, 21);
        stamp(&mut f);
        f
    }
    fn stamp(f: &mut Fixture) {
        for block in [2, 3] {
            assert!(f.csum.patch_extent_tail(
                INO,
                GENERATION,
                &mut f.bytes[block * BS..(block + 1) * BS]
            ));
        }
    }
    use super::*;

    fn limits() -> Limits {
        Limits {
            node_reads: 16,
            records: 64,
            claims: 32,
            scratch_bytes: 5 * BS,
        }
    }
    fn tree(f: &Fixture, limits: Limits) -> Result<Tree> {
        collect(
            &f.root,
            f,
            &ExtentVerifyCtx {
                ino: INO,
                generation: GENERATION,
                csum: &f.csum,
            },
            Geometry {
                block_size: BS as u32,
                first_block: 1,
                end_block: 32,
            },
            limits,
        )
    }
    fn collect_fixture(f: &Fixture) -> Result<()> {
        tree(f, limits()).map(|_| ())
    }
    fn refused(f: &Fixture) {
        let before = f.bytes.clone();
        let root = f.root.clone();
        let result = collect_fixture(f);
        assert!(result.is_err(), "ownership tree accepted: {result:?}");
        assert_eq!(f.bytes, before);
        assert_eq!(f.root, root);
    }
    #[test]
    fn whole_tree_checks_a_checksum_in_an_unselected_child() {
        let mut f = fixture();
        f.bytes[3 * BS + 100] ^= 1;
        refused(&f);
    }
    #[test]
    fn whole_tree_checks_exact_child_depth() {
        let mut f = fixture();
        f.root[6..8].copy_from_slice(&2u16.to_le_bytes());
        refused(&f);
    }
    #[test]
    fn whole_tree_checks_parent_key_and_shadowed_records() {
        let mut f = fixture();
        data(&mut f.bytes[2 * BS..3 * BS], 0, 10, 1, 20);
        stamp(&mut f);
        refused(&f);
    }
    #[test]
    fn whole_tree_checks_external_capacity() {
        let mut f = fixture();
        f.bytes[2 * BS + 4..2 * BS + 6].copy_from_slice(&85u16.to_le_bytes());
        stamp(&mut f);
        refused(&f);
    }
    #[test]
    fn whole_tree_checks_duplicate_child_storage() {
        let mut f = fixture();
        index(&mut f.root, 1, 10, 2);
        refused(&f);
    }
    #[test]
    fn whole_tree_checks_data_and_mapping_role_overlap() {
        let mut f = fixture();
        data(&mut f.bytes[3 * BS..4 * BS], 0, 10, 1, 2);
        stamp(&mut f);
        refused(&f);
    }
    #[test]
    fn whole_tree_checks_zero_length_and_physical_bounds() {
        let mut f = fixture();
        data(&mut f.bytes[2 * BS..3 * BS], 0, 0, 0, 20);
        stamp(&mut f);
        refused(&f);
    }
    #[test]
    fn whole_tree_healthy_disjoint_control() {
        let f = fixture();
        let result = tree(&f, limits()).unwrap();
        assert_eq!((result.ino, result.generation), (INO, GENERATION));
        assert_eq!(
            result.claims,
            vec![
                Claim {
                    start: 2,
                    end: 3,
                    role: Role::Mapping
                },
                Claim {
                    start: 3,
                    end: 4,
                    role: Role::Mapping
                },
                Claim {
                    start: 20,
                    end: 21,
                    role: Role::Data {
                        logical: 0,
                        unwritten: false
                    }
                },
                Claim {
                    start: 21,
                    end: 22,
                    role: Role::Data {
                        logical: 10,
                        unwritten: false
                    }
                },
            ]
        );
        assert_eq!(f.reads.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn preserves_inline_unwritten_and_beyond_eof_claims_without_payload_reads() {
        let mut f = fixture();
        f.root.fill(0);
        header(&mut f.root, 0, 2, 4);
        data(&mut f.root, 0, 2, 3, 20);
        data(&mut f.root, 1, 100, 32770, 25);
        let result = tree(&f, limits()).unwrap();
        assert_eq!(
            result.claims,
            vec![
                Claim {
                    start: 20,
                    end: 23,
                    role: Role::Data {
                        logical: 2,
                        unwritten: false
                    }
                },
                Claim {
                    start: 25,
                    end: 27,
                    role: Role::Data {
                        logical: 100,
                        unwritten: true
                    }
                },
            ]
        );
        assert_eq!(f.reads.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn enforces_each_resource_limit_before_excess_reads_or_partial_output() {
        let exact = Limits {
            node_reads: 2,
            records: 4,
            claims: 4,
            scratch_bytes: BS,
        };
        assert_eq!(tree(&fixture(), exact).unwrap().claims.len(), 4);
        for small in [
            Limits {
                node_reads: 1,
                ..exact
            },
            Limits {
                records: 3,
                ..exact
            },
            Limits { claims: 3, ..exact },
            Limits {
                scratch_bytes: BS - 1,
                ..exact
            },
            Limits {
                claims: usize::MAX,
                ..exact
            },
        ] {
            let f = fixture();
            let before = f.bytes.clone();
            assert!(matches!(
                tree(&f, small),
                Err(Error::Unsupported("extent ownership resource limit"))
            ));
            assert!(f.reads.load(Ordering::Relaxed) <= small.node_reads);
            assert_eq!(f.bytes, before);
        }
        let f = fixture();
        assert!(tree(
            &f,
            Limits {
                node_reads: 0,
                ..exact
            }
        )
        .is_err());
        assert_eq!(f.reads.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn accepts_empty_inline_tree_with_zero_budgets() {
        let mut f = fixture();
        f.root.fill(0);
        header(&mut f.root, 0, 0, 4);
        let empty = Limits {
            node_reads: 0,
            records: 0,
            claims: 0,
            scratch_bytes: 0,
        };
        assert!(tree(&f, empty).unwrap().claims.is_empty());
        assert_eq!(f.reads.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn refuses_invalid_root_empty_nodes_and_unqualified_profiles() {
        for mode in 0..9 {
            let mut f = fixture();
            match mode {
                0 => {
                    f.root.pop();
                }
                1 => f.root[4..6].copy_from_slice(&5u16.to_le_bytes()),
                2 => f.root[6..8].copy_from_slice(&6u16.to_le_bytes()),
                3 => f.root[4..6].fill(0),
                4 => f.root[2..4].copy_from_slice(&5u16.to_le_bytes()),
                5 => f.root[2..4].fill(0),
                6 => f.bytes[2 * BS + 2..2 * BS + 4].fill(0),
                7 => f.csum.enabled = false,
                8 => f.bytes[2 * BS + 4..2 * BS + 6].copy_from_slice(&83u16.to_le_bytes()),
                _ => unreachable!(),
            }
            if f.csum.enabled {
                stamp(&mut f);
            }
            refused(&f);
        }
        let f = fixture();
        assert!(matches!(
            collect(
                &f.root,
                &f,
                &ExtentVerifyCtx {
                    ino: INO,
                    generation: GENERATION,
                    csum: &f.csum
                },
                Geometry {
                    block_size: 2048,
                    first_block: 1,
                    end_block: 16
                },
                limits()
            ),
            Err(Error::Unsupported(
                "extent ownership checksum/block profile"
            ))
        ));
    }

    #[test]
    fn refuses_unsorted_overlapping_and_mismatched_logical_ranges() {
        for mode in 0..5 {
            let mut f = fixture();
            match mode {
                0 => index(&mut f.root, 1, 0, 3),
                1 => index(&mut f.root, 0, 11, 2),
                2 => data(&mut f.bytes[2 * BS..3 * BS], 0, 0, 11, 20),
                3 => data(&mut f.bytes[3 * BS..4 * BS], 0, 11, 1, 21),
                4 => {
                    header(&mut f.bytes[2 * BS..3 * BS], 0, 2, 84);
                    data(&mut f.bytes[2 * BS..3 * BS], 1, 0, 1, 24);
                }
                _ => unreachable!(),
            }
            stamp(&mut f);
            refused(&f);
        }
    }

    #[test]
    fn refuses_out_of_bounds_data_and_nodes_before_device_reads() {
        for (start, length) in [(0, 1), (31, 2), (32, 1)] {
            let mut f = fixture();
            data(&mut f.bytes[3 * BS..4 * BS], 0, 10, length, start);
            stamp(&mut f);
            refused(&f);
        }
        for block in [0, 32, u32::MAX] {
            let mut f = fixture();
            index(&mut f.root, 0, 0, block);
            refused(&f);
            assert_eq!(f.reads.load(Ordering::Relaxed), 0);
        }
        let f = fixture();
        let ctx = ExtentVerifyCtx {
            ino: INO,
            generation: GENERATION,
            csum: &f.csum,
        };
        assert!(collect(
            &f.root,
            &f,
            &ctx,
            Geometry {
                block_size: BS as u32,
                first_block: 1,
                end_block: 2
            },
            limits()
        )
        .is_err());
        assert_eq!(f.reads.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn refuses_cross_branch_data_overlap_and_cycles() {
        let mut f = fixture();
        data(&mut f.bytes[3 * BS..4 * BS], 0, 10, 1, 20);
        stamp(&mut f);
        refused(&f);
        let mut f = fixture();
        f.root[6..8].copy_from_slice(&2u16.to_le_bytes());
        f.bytes[2 * BS..3 * BS].fill(0);
        header(&mut f.bytes[2 * BS..3 * BS], 1, 1, 84);
        index(&mut f.bytes[2 * BS..3 * BS], 0, 0, 2);
        stamp(&mut f);
        refused(&f);
        assert_eq!(f.reads.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn walks_maximum_depth_with_one_reused_buffer_per_level() {
        let mut f = fixture();
        f.root.fill(0);
        header(&mut f.root, 5, 1, 4);
        index(&mut f.root, 0, 16, 2);
        for block in 2..=6 {
            let node = &mut f.bytes[block * BS..(block + 1) * BS];
            node.fill(0);
            header(node, (6 - block) as u16, 1, 84);
            if block == 6 {
                data(node, 0, 16, 1, 20);
            } else {
                index(node, 0, 16, (block + 1) as u32);
            }
            assert!(f.csum.patch_extent_tail(INO, GENERATION, node));
        }
        let result = tree(
            &f,
            Limits {
                node_reads: 5,
                records: 6,
                claims: 6,
                scratch_bytes: 5 * BS,
            },
        )
        .unwrap();
        assert_eq!(
            result
                .claims
                .iter()
                .filter(|c| c.role == Role::Mapping)
                .count(),
            5
        );
        assert_eq!(f.reads.load(Ordering::Relaxed), 5);
    }

    #[test]
    fn propagates_device_errors_without_results_or_input_edits() {
        struct ReadError;
        impl BlockDevice for ReadError {
            fn read_at(&self, _: u64, _: &mut [u8]) -> Result<()> {
                Err(std::io::Error::other("injected ownership read error").into())
            }
            fn size_bytes(&self) -> u64 {
                (32 * BS) as u64
            }
        }
        let f = fixture();
        let before = f.root.clone();
        let ctx = ExtentVerifyCtx {
            ino: INO,
            generation: GENERATION,
            csum: &f.csum,
        };
        assert!(matches!(
            collect(
                &f.root,
                &ReadError,
                &ctx,
                Geometry {
                    block_size: BS as u32,
                    first_block: 1,
                    end_block: 32
                },
                limits()
            ),
            Err(Error::Io(_))
        ));
        assert_eq!(f.root, before);
    }

    #[test]
    fn verifies_checksum_identity_before_following_child_pointers() {
        let mut f = fixture();
        f.root[6..8].copy_from_slice(&2u16.to_le_bytes());
        header(&mut f.bytes[2 * BS..3 * BS], 1, 1, 84);
        index(&mut f.bytes[2 * BS..3 * BS], 0, 0, u32::MAX);
        assert!(matches!(
            tree(&f, limits()),
            Err(Error::BadChecksum {
                what: "extent block"
            })
        ));
        assert_eq!(f.reads.load(Ordering::Relaxed), 1);
        let f = fixture();
        let ctx = ExtentVerifyCtx {
            ino: INO + 1,
            generation: GENERATION,
            csum: &f.csum,
        };
        assert!(matches!(
            collect(
                &f.root,
                &f,
                &ctx,
                Geometry {
                    block_size: BS as u32,
                    first_block: 1,
                    end_block: 32
                },
                limits()
            ),
            Err(Error::BadChecksum {
                what: "extent block"
            })
        ));
    }

    #[test]
    fn retains_large_inline_ranges_and_logical_end_boundary() {
        struct NoReads;
        impl BlockDevice for NoReads {
            fn read_at(&self, _: u64, _: &mut [u8]) -> Result<()> {
                panic!("inline tree must not read")
            }
            fn size_bytes(&self) -> u64 {
                1u64 << 60
            }
        }
        let f = fixture();
        let mut root = vec![0; 60];
        header(&mut root, 0, 1, 4);
        data(&mut root, 0, 0, 32768, 20);
        root[18..20].copy_from_slice(&1u16.to_le_bytes());
        let ctx = ExtentVerifyCtx {
            ino: INO,
            generation: GENERATION,
            csum: &f.csum,
        };
        let geometry = Geometry {
            block_size: BS as u32,
            first_block: 1,
            end_block: 1u64 << 48,
        };
        let result = collect(&root, &NoReads, &ctx, geometry, limits()).unwrap();
        assert_eq!(
            result.claims,
            vec![Claim {
                start: (1u64 << 32) + 20,
                end: (1u64 << 32) + 32788,
                role: Role::Data {
                    logical: 0,
                    unwritten: false
                }
            }]
        );
        data(&mut root, 0, u32::MAX, 1, 20);
        assert!(collect(&root, &NoReads, &ctx, geometry, limits()).is_ok());
        data(&mut root, 0, u32::MAX, 2, 20);
        assert!(collect(&root, &NoReads, &ctx, geometry, limits()).is_err());
    }

    mod needs_host {
        use super::*;

        #[test]
        fn linux_fiemap_matches_complete_inline_and_external_ownership() {
            use crate::{block_io::FileDevice, fs::Filesystem};
            use fs_ext4_test_support::{assert_e2fsck_clean, guest_kernel_write, oracle};
            use std::sync::Arc;
            for bs in [1024u32, 4096] {
                let image = fs_ext4_test_support::temp_dir().join(format!("extent-owner-{bs}.img"));
                std::fs::File::create(&image)
                    .unwrap()
                    .set_len(64 * 1024 * 1024)
                    .unwrap();
                let made = oracle("mke2fs").args([
                    "-q", "-F", "-t", "ext4", "-b", &bs.to_string(), "-I", "256", "-m", "0", "-O",
                    "none,extent,64bit,filetype,sparse_super,large_file,huge_file,metadata_csum",
                    "-E", "lazy_itable_init=0",
                ]).arg(&image).output();
                assert!(
                    made.status.success(),
                    "{}",
                    String::from_utf8_lossy(&made.stderr)
                );
                let image = image.to_str().unwrap();
                let script = format!(
                    "python3 - {bs} <<'EXTENT_OWNER_PY'\n{}\nEXTENT_OWNER_PY\n",
                    include_str!("../../tests/extent_ownership_guest.py")
                );
                let observed = guest_kernel_write(image, &script);
                assert!(
                    observed.status.success(),
                    "{}",
                    String::from_utf8_lossy(&observed.stderr)
                );
                assert_e2fsck_clean(image, "Linux extent ownership control");
                let report = String::from_utf8(observed.stdout).unwrap();
                std::fs::write(
                    fs_ext4_test_support::temp_dir().join(format!("extent-owner-{bs}.fiemap.txt")),
                    &report,
                )
                .unwrap();
                let fs = Filesystem::mount(Arc::new(FileDevice::open(image).unwrap())).unwrap();
                for name in ["inline", "external"] {
                    let ino = fs.lookup_at(2, name.as_bytes()).unwrap();
                    let (inode, _) = fs.read_inode_verified(ino).unwrap();
                    let header = ExtentHeader::parse(&inode.block).unwrap();
                    if name == "inline" {
                        assert_eq!(header.depth, 0);
                    } else {
                        assert!(header.depth > 0);
                    }
                    let result = collect(
                        &inode.block,
                        fs.dev.as_ref(),
                        &ExtentVerifyCtx {
                            ino,
                            generation: inode.generation,
                            csum: &fs.csum,
                        },
                        Geometry {
                            block_size: bs,
                            first_block: u64::from(fs.sb.first_data_block),
                            end_block: fs.byte_ceiling().unwrap() / u64::from(bs),
                        },
                        Limits {
                            node_reads: 64,
                            records: 1024,
                            claims: 1024,
                            scratch_bytes: 5 * bs as usize,
                        },
                    )
                    .unwrap();
                    let mut expected = Vec::new();
                    let mut identities = 0;
                    for line in report.lines() {
                        let fields: Vec<_> = line.split_whitespace().collect();
                        assert!(
                            matches!(fields.first(), Some(&"INODE") | Some(&"EXTENT")),
                            "{line}"
                        );
                        if fields[1] != name {
                            continue;
                        }
                        let numbers: Vec<u64> =
                            fields[2..].iter().map(|n| n.parse().unwrap()).collect();
                        if fields[0] == "INODE" {
                            assert_eq!(numbers, [u64::from(ino), inode.blocks, inode.size]);
                            identities += 1;
                        } else {
                            assert_eq!(numbers.len(), 4);
                            expected.push((numbers[0], numbers[1], numbers[2], numbers[3]));
                        }
                    }
                    assert_eq!(identities, 1);
                    let mut actual: Vec<_> = result
                        .claims
                        .iter()
                        .filter_map(|claim| match claim.role {
                            Role::Data { logical, unwritten } => Some((
                                logical,
                                claim.start,
                                claim.end - claim.start,
                                u64::from(unwritten),
                            )),
                            Role::Mapping => None,
                        })
                        .collect();
                    actual.sort_unstable();
                    expected.sort_unstable();
                    assert!(!expected.is_empty());
                    assert_eq!(actual, expected, "Linux FIEMAP {bs}/{name}");
                    let blocks: u64 = result
                        .claims
                        .iter()
                        .map(|claim| claim.end - claim.start)
                        .sum();
                    assert_eq!(
                        blocks * u64::from(bs) / 512,
                        inode.blocks,
                        "including all map nodes"
                    );
                }
            }
        }
    }
}
