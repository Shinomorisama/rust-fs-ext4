//! Private complete current-view ownership, disconnected from write admission.
//! Format evidence and lifecycle requirements: docs/CASEFOLD-INDEX-REVIEW.md.

use crate::block_allocation;
use crate::block_io::BlockDevice;
use crate::error::{Error, Result};
use crate::extent::{ownership as tree, ExtentVerifyCtx};
use crate::fixed_metadata;
use crate::fs::{BlockBuffer, Filesystem};
use crate::inode_inventory;
use crate::inode_roles::{self, Role, Storage};

#[derive(Clone, Copy)]
pub(crate) struct Limits {
    /// Conservative shared reservation: fixed, inventory and allocation groups.
    pub groups: u64,
    pub inode_bits: u64,
    pub inodes: usize,
    pub claims: usize,
    /// Explicit per-inode transient ceiling, also bounded by remaining claims.
    pub tree_claims: usize,
    /// Conservative phase read reservations plus actual external tree fetches.
    pub reads: u64,
    /// Shared across all extent-backed allocated inodes, including empty files.
    pub tree_records: usize,
    pub bitmap_bytes: u64,
    /// Requested capacity bound, not allocator/cache overhead or measured RSS.
    pub memory_bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct InodeOwner {
    pub ino: u32,
    pub generation: u32,
    pub role: Role,
    pub storage: Storage,
    pub size: u64,
    pub sectors: u64,
    pub links: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Owner {
    Fixed {
        role: fixed_metadata::Role,
        group: u32,
    },
    Inode {
        ino: u32,
        generation: u32,
        role: Role,
        storage: tree::Role,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Claim {
    pub start: u64,
    pub end: u64,
    pub owner: Owner,
}

#[derive(Debug)]
pub(crate) struct Ownership {
    /// Every allocated ID once, including no-storage/reserved/zero-link records.
    pub inodes: Vec<InodeOwner>,
    /// Complete sorted disjoint fixed and inode storage claims; no epoch token.
    pub claims: Vec<Claim>,
    pub charged_reads: u64,
    pub tree_records: usize,
}

const LIMIT: Error = Error::Unsupported("ownership build resource limit");

impl Limits {
    /// A conservative sum of all requested capacities, even phases that do not
    /// coexist. Includes explicit checksum stack copy; excludes other scalar
    /// stack/allocator overhead and the mount/device cache, including replay pins.
    pub fn requested_bytes(self, bs: usize) -> Result<usize> {
        let mut bytes = bs
            .checked_mul(8)
            .and_then(|n| n.checked_add(256))
            .ok_or(LIMIT)?;
        for (count, width) in [
            (self.inodes, std::mem::size_of::<u32>()),
            (self.inodes, std::mem::size_of::<InodeOwner>()),
            (self.claims, std::mem::size_of::<Claim>()),
            (self.claims, std::mem::size_of::<fixed_metadata::Range>()),
            (self.claims, std::mem::size_of::<block_allocation::Range>()),
            (
                self.tree_claims.min(self.claims),
                std::mem::size_of::<tree::Claim>(),
            ),
        ] {
            bytes = count
                .checked_mul(width)
                .and_then(|n| bytes.checked_add(n))
                .ok_or(LIMIT)?;
        }
        Ok(bytes)
    }
}

fn reserved<T>(count: usize) -> Result<Vec<T>> {
    let mut out = Vec::new();
    out.try_reserve_exact(count).map_err(|_| LIMIT)?;
    Ok(out)
}

/// Aligned full-node staged precedence, with exact size and filesystem bounds.
/// The tree walker debits its fetch before reaching this adapter, including errors.
struct View<'a> {
    fs: &'a Filesystem,
    buf: &'a BlockBuffer,
    end: u64,
}
impl BlockDevice for View<'_> {
    fn read_at(&self, at: u64, out: &mut [u8]) -> Result<()> {
        let bs = u64::from(self.fs.sb.block_size());
        if out.len() as u64 != bs
            || !at.is_multiple_of(bs)
            || at.checked_add(bs).is_none_or(|end| end > self.end)
        {
            return Err(Error::OutOfBounds);
        }
        match self.buf.dirty.get(&(at / bs)) {
            Some(bytes) if bytes.len() == out.len() => out.copy_from_slice(bytes),
            Some(_) => return Err(Error::Corrupt("ownership staged map block size")),
            None => self.fs.dev.read_at(at, out)?,
        }
        Ok(())
    }
    fn size_bytes(&self) -> u64 {
        self.end
    }
}

/// Requires exclusive/coherent stable mount and staged bytes throughout this
/// invocation. Rust borrows alone do not exclude retained external device handles.
/// Returns a local complete result or nothing; no cache, epoch, dependencies,
/// transaction overlay or admission is published. No caller/device bytes change.
pub(crate) fn build(fs: &Filesystem, buf: &BlockBuffer, limits: Limits) -> Result<Ownership> {
    let bs = fs.sb.block_size() as usize;
    let groups = fs.groups.len() as u64;
    if !matches!(bs, 1024 | 4096)
        || !matches!(fs.sb.desc_size, 32 | 64)
        || groups == 0
        || groups > u64::from(u32::MAX)
    {
        return Err(Error::Unsupported("ownership build geometry profile"));
    }
    if groups.checked_mul(3).ok_or(LIMIT)? > limits.groups
        || limits.requested_bytes(bs)? > limits.memory_bytes
    {
        return Err(LIMIT);
    }
    let descriptors = groups.div_ceil((bs / usize::from(fs.sb.desc_size)) as u64);
    let fixed_reads = 1 + descriptors;
    let bitmap_reads = fixed_reads + groups;
    let mut reads_left = limits
        .reads
        .checked_sub(fixed_reads + 2 * bitmap_reads)
        .ok_or(LIMIT)?;
    let mut records_left = limits.tree_records;
    // Establish the strict common inode/feature profile before layout traversal.
    let mut roles = inode_roles::Reader::new(
        fs,
        buf,
        inode_roles::Limits {
            records: limits.inodes as u64,
            reads: limits.inodes as u64,
            scratch_bytes: bs + 256,
        },
    )?;
    let fixed = fixed_metadata::collect(
        fs,
        buf,
        fixed_metadata::Limits {
            groups,
            reads: fixed_reads,
            ranges: limits.claims,
            scratch_bytes: 2 * bs,
        },
    )?;
    let inventory = inode_inventory::collect(
        fs,
        buf,
        inode_inventory::Limits {
            groups,
            bit_probes: limits.inode_bits,
            reads: bitmap_reads,
            inodes: limits.inodes,
            scratch_bytes: 2 * bs,
        },
    )?;
    reads_left = reads_left
        .checked_sub(inventory.inodes.len() as u64)
        .ok_or(LIMIT)?;
    let journal = fs.sb.feature_compat & crate::features::Compat::HAS_JOURNAL.bits() != 0;
    if !inventory.inodes.contains(&2) || (journal && !inventory.inodes.contains(&8)) {
        return Err(Error::Corrupt("ownership missing root/journal allocation"));
    }
    let mut inodes = reserved(limits.inodes)?;
    let mut claims = reserved(limits.claims)?;
    for range in fixed.ranges {
        claims.push(Claim {
            start: range.start,
            end: range.end,
            owner: Owner::Fixed {
                role: range.role,
                group: range.group,
            },
        });
    }
    let view = View {
        fs,
        buf,
        end: fs.sb.blocks_count.checked_mul(bs as u64).ok_or(LIMIT)?,
    };
    let mut root_seen = false;
    let mut journal_seen = false;
    for ino in inventory.inodes {
        let record = roles.read(ino)?;
        root_seen |= record.role == Role::RootDirectory;
        journal_seen |= record.role == Role::Journal;
        inodes.push(InodeOwner {
            ino,
            generation: record.generation,
            role: record.role,
            storage: record.storage,
            size: record.size,
            sectors: record.sectors,
            links: record.links,
        });
        if matches!(record.storage, Storage::Extents { .. }) {
            let tree = tree::collect(
                &record.block,
                &view,
                &ExtentVerifyCtx {
                    csum: &fs.csum,
                    ino,
                    generation: record.generation,
                },
                tree::Geometry {
                    block_size: bs as u32,
                    first_block: u64::from(fs.sb.first_data_block).max(1),
                    end_block: fs.sb.blocks_count,
                },
                tree::Limits {
                    node_reads: usize::try_from(reads_left).unwrap_or(usize::MAX),
                    records: records_left,
                    claims: limits.tree_claims.min(limits.claims - claims.len()),
                    scratch_bytes: bs,
                },
            )?;
            reads_left = reads_left
                .checked_sub(tree.usage.node_reads as u64)
                .ok_or(LIMIT)?;
            records_left = records_left.checked_sub(tree.usage.records).ok_or(LIMIT)?;
            let blocks = tree.claims.iter().try_fold(0u64, |n, claim| {
                n.checked_add(claim.end - claim.start).ok_or(LIMIT)
            })?;
            if blocks.checked_mul(bs as u64 / 512).ok_or(LIMIT)? != record.sectors {
                return Err(Error::Corrupt("ownership inode sector count"));
            }
            for claim in tree.claims {
                if claims.len() == limits.claims {
                    return Err(LIMIT);
                }
                claims.push(Claim {
                    start: claim.start,
                    end: claim.end,
                    owner: Owner::Inode {
                        ino,
                        generation: record.generation,
                        role: record.role,
                        storage: claim.role,
                    },
                });
            }
            // Each transient vector is consumed/dropped before the next inode.
        }
    }
    if !root_seen || journal_seen != journal {
        return Err(Error::Corrupt("ownership missing root/journal role"));
    }
    claims.sort_unstable_by_key(|claim| (claim.start, claim.end));
    if claims.windows(2).any(|p| p[0].end > p[1].start) {
        return Err(Error::Corrupt("ownership physical conflict"));
    }
    let mut allocation = reserved(limits.claims)?;
    for claim in &claims {
        if matches!(
            claim.owner,
            Owner::Fixed {
                role: fixed_metadata::Role::Boot,
                ..
            }
        ) {
            continue;
        }
        allocation.push(block_allocation::Range {
            start: claim.start,
            end: claim.end,
        });
    }
    block_allocation::verify(
        fs,
        buf,
        &allocation,
        block_allocation::Limits {
            ranges: limits.claims,
            groups,
            reads: bitmap_reads,
            bitmap_bytes: limits.bitmap_bytes,
            scratch_bytes: 2 * bs,
        },
    )?;
    Ok(Ownership {
        inodes,
        claims,
        charged_reads: limits.reads - reads_left,
        tree_records: limits.tree_records - records_left,
    })
}

#[cfg(test)]
mod needs_host {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    fn interval(text: &str) -> (u64, u64) {
        let first = text.split_whitespace().next().unwrap();
        let (lo, hi) = first.split_once('-').unwrap_or((first, first));
        (
            lo.parse().unwrap(),
            hi.parse::<u64>().unwrap().checked_add(1).unwrap(),
        )
    }

    #[test]
    fn linux_complete_ownership_matches_all_inode_and_fixed_storage_reports() {
        use crate::block_io::FileDevice;
        use fs_ext4_test_support::{assert_e2fsck_clean, guest_kernel_write, oracle};
        use std::sync::Arc;
        for bs in [1024u32, 4096] {
            for (descriptor, journal) in [(32, false), (64, false), (64, true)] {
                let base = fs_ext4_test_support::temp_dir()
                    .join(format!("ownership-{bs}-{descriptor}-{journal}"));
                let image = base.with_extension("img");
                std::fs::File::create(&image)
                    .unwrap()
                    .set_len(16 * 1024 * 1024)
                    .unwrap();
                let mut features = String::from("none,extent,filetype,metadata_csum");
                if descriptor == 64 {
                    features.push_str(",64bit");
                }
                if journal {
                    features.push_str(",has_journal");
                }
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
                        if bs == 1024 { "8192" } else { "2048" },
                        "-m",
                        "0",
                        "-O",
                        &features,
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
                let populated = guest_kernel_write(
                    image.to_str().unwrap(),
                    &format!(
                        "python3 - <<'OWNERSHIP_PY'\n{}\nOWNERSHIP_PY\n",
                        include_str!("../tests/ownership_guest.py")
                    ),
                );
                assert!(
                    populated.status.success(),
                    "{}",
                    String::from_utf8_lossy(&populated.stderr)
                );
                assert_e2fsck_clean(image.to_str().unwrap(), "Linux complete ownership");
                let fs =
                    Filesystem::mount(Arc::new(FileDevice::open(image.to_str().unwrap()).unwrap()))
                        .unwrap();
                assert_eq!(
                    (fs.sb.desc_size, fs.groups.len(), fs.sb.inodes_count),
                    (descriptor, 2, 64)
                );
                let commands = base.with_extension("inventory.commands.txt");
                std::fs::write(
                    &commands,
                    (1..=64)
                        .map(|ino| format!("testi <{ino}>\n"))
                        .collect::<String>(),
                )
                .unwrap();
                let observed = oracle("debugfs")
                    .arg("-f")
                    .arg(&commands)
                    .arg(&image)
                    .output();
                assert!(observed.status.success());
                let report = String::from_utf8(observed.stdout).unwrap();
                std::fs::write(base.with_extension("inventory.txt"), &report).unwrap();
                let mut ids = Vec::new();
                for ino in 1..=64 {
                    let used = format!("Inode {ino} is marked in use");
                    let free = format!("Inode {ino} is not in use");
                    let yes = report.lines().filter(|s| s.trim() == used).count();
                    let no = report.lines().filter(|s| s.trim() == free).count();
                    assert_eq!(yes + no, 1, "missing/duplicate inode answer {ino}");
                    if yes == 1 {
                        ids.push(ino);
                    }
                }
                let commands = base.with_extension("owners.commands.txt");
                std::fs::write(
                    &commands,
                    ids.iter()
                        .map(|ino| format!("stat <{ino}>\n"))
                        .collect::<String>(),
                )
                .unwrap();
                let observed = oracle("debugfs")
                    .arg("-f")
                    .arg(&commands)
                    .arg(&image)
                    .output();
                assert!(observed.status.success());
                let report = String::from_utf8(observed.stdout).unwrap();
                std::fs::write(base.with_extension("owners.txt"), &report).unwrap();
                let mut kinds = BTreeMap::new();
                let mut generations = BTreeMap::new();
                let mut sectors = BTreeMap::new();
                let mut sizes = BTreeMap::new();
                let mut links = BTreeMap::new();
                let mut extent_inodes = BTreeSet::new();
                let mut expected = Vec::new();
                let mut current = None;
                let mut extents = false;
                for line in report.lines() {
                    if let Some(header) = line.strip_prefix("Inode:") {
                        let (ino, rest) = header.split_once("Type:").unwrap();
                        let ino: u32 = ino.trim().parse().unwrap();
                        let (kind, _) = rest.split_once("Mode:").unwrap();
                        assert!(kinds.insert(ino, kind.trim().to_lowercase()).is_none());
                        current = Some(ino);
                        extents = false;
                    } else if let Some(value) = line.strip_prefix("Generation:") {
                        assert!(generations
                            .insert(
                                current.unwrap(),
                                value
                                    .split_whitespace()
                                    .next()
                                    .unwrap()
                                    .parse::<u32>()
                                    .unwrap()
                            )
                            .is_none());
                    } else if line.starts_with("User:") {
                        assert!(sizes
                            .insert(
                                current.unwrap(),
                                line.rsplit_once("Size:")
                                    .unwrap()
                                    .1
                                    .trim()
                                    .parse::<u64>()
                                    .unwrap()
                            )
                            .is_none());
                    } else if let Some(value) = line.strip_prefix("Links:") {
                        let (count, blocks) = value.split_once("Blockcount:").unwrap();
                        assert!(links
                            .insert(current.unwrap(), count.trim().parse::<u16>().unwrap())
                            .is_none());
                        assert!(sectors
                            .insert(current.unwrap(), blocks.trim().parse::<u64>().unwrap())
                            .is_none());
                    } else if line == "EXTENTS:" {
                        extents = true;
                        extent_inodes.insert(current.unwrap());
                    } else if line.starts_with("debugfs:") {
                        extents = false;
                    } else if extents && line.starts_with('(') {
                        let ino = current.unwrap();
                        let role = if ino == 2 {
                            Role::RootDirectory
                        } else if journal && ino == 8 {
                            Role::Journal
                        } else {
                            match kinds[&ino].as_str() {
                                "directory" => Role::Directory,
                                "regular" => Role::Regular,
                                "symlink" => Role::Symlink,
                                other => panic!("unexpected extent owner type {other}"),
                            }
                        };
                        for token in line.split(',').map(str::trim) {
                            let (label, physical) = token.split_once(':').unwrap();
                            let label = label.strip_prefix('(').unwrap().strip_suffix(')').unwrap();
                            let (start, end) = interval(physical);
                            let storage = if label == "ETB0" {
                                assert_eq!(end - start, 1);
                                tree::Role::Mapping
                            } else {
                                let unwritten = label.contains("[u]");
                                let label = label.replace("[u]", "");
                                let (logical, logical_end) = interval(label.trim());
                                assert_eq!(logical_end - logical, end - start);
                                tree::Role::Data { logical, unwritten }
                            };
                            expected.push(Claim {
                                start,
                                end,
                                owner: Owner::Inode {
                                    ino,
                                    generation: generations[&ino],
                                    role,
                                    storage,
                                },
                            });
                        }
                    }
                }
                assert_eq!(kinds.keys().copied().collect::<Vec<_>>(), ids);
                assert_eq!(generations.len(), ids.len());
                assert_eq!(
                    (sectors.len(), sizes.len(), links.len()),
                    (ids.len(), ids.len(), ids.len())
                );
                let observed = oracle("dumpe2fs").arg(&image).output();
                assert!(observed.status.success());
                let report = String::from_utf8(observed.stdout).unwrap();
                std::fs::write(base.with_extension("fixed.txt"), &report).unwrap();
                let mut group = None;
                let mut groups = BTreeSet::new();
                let mut free_count = None;
                if bs == 1024 {
                    expected.push(Claim {
                        start: 0,
                        end: 1,
                        owner: Owner::Fixed {
                            role: fixed_metadata::Role::Boot,
                            group: 0,
                        },
                    });
                }
                for line in report.lines() {
                    if let Some(value) = line.strip_prefix("Free blocks:") {
                        assert!(free_count
                            .replace(value.trim().parse::<u64>().unwrap())
                            .is_none());
                    }
                    let text = line.trim();
                    if let Some((number, _)) =
                        text.strip_prefix("Group ").and_then(|s| s.split_once(':'))
                    {
                        if let Ok(number) = number.parse::<u32>() {
                            assert!(groups.insert(number));
                            group = Some(number);
                        }
                    }
                    let mut push = |text: &str, role| {
                        let (start, end) = interval(text);
                        expected.push(Claim {
                            start,
                            end,
                            owner: Owner::Fixed {
                                role,
                                group: group.unwrap(),
                            },
                        });
                    };
                    if let Some(text) = text
                        .strip_prefix("Primary superblock at ")
                        .or_else(|| text.strip_prefix("Backup superblock at "))
                    {
                        let (sb, gd) = text.split_once(", Group descriptors at ").unwrap();
                        push(sb, fixed_metadata::Role::Superblock);
                        push(gd, fixed_metadata::Role::Descriptors);
                    }
                    for (prefix, role) in [
                        (
                            "Reserved GDT blocks at ",
                            fixed_metadata::Role::ReservedDescriptors,
                        ),
                        ("Block bitmap at ", fixed_metadata::Role::BlockBitmap),
                        ("Inode bitmap at ", fixed_metadata::Role::InodeBitmap),
                        ("Inode table at ", fixed_metadata::Role::InodeTable),
                    ] {
                        if let Some(text) = text.strip_prefix(prefix) {
                            push(text, role);
                        }
                    }
                }
                assert_eq!(groups, [0, 1].into());
                expected.sort_unstable_by_key(|c| (c.start, c.end));
                let got = build(
                    &fs,
                    &BlockBuffer::new(bs),
                    Limits {
                        groups: 6,
                        inode_bits: 2 * u64::from(bs) * 8,
                        inodes: 64,
                        claims: 512,
                        tree_claims: 128,
                        reads: 4096,
                        tree_records: 4096,
                        bitmap_bytes: 128 * u64::from(bs),
                        memory_bytes: 256 * 1024,
                    },
                )
                .unwrap();
                assert_eq!(got.inodes.iter().map(|r| r.ino).collect::<Vec<_>>(), ids);
                assert_eq!(
                    got.claims, expected,
                    "complete owner identity/range/role mismatch {bs}/{descriptor}/{journal}"
                );
                for record in &got.inodes {
                    let ino = record.ino;
                    assert_eq!(
                        (record.generation, record.sectors, record.size, record.links),
                        (generations[&ino], sectors[&ino], sizes[&ino], links[&ino])
                    );
                    let kind = match record.role {
                        Role::InactiveReserved => "bad type",
                        Role::RootDirectory | Role::Directory => "directory",
                        Role::Journal | Role::Regular => "regular",
                        Role::Symlink => "symlink",
                        Role::Fifo => "fifo",
                    };
                    assert_eq!(kinds[&ino], kind);
                    let mapped = expected.iter().any(|c| matches!(c.owner, Owner::Inode { ino: n, storage: tree::Role::Mapping, .. } if n == ino));
                    if extent_inodes.contains(&ino) {
                        assert_eq!(
                            record.storage,
                            Storage::Extents {
                                depth: u16::from(mapped)
                            }
                        );
                    }
                }
                assert!(got.claims.iter().any(|c| matches!(
                    c.owner,
                    Owner::Inode {
                        storage: tree::Role::Mapping,
                        ..
                    }
                )));
                assert!(got.claims.iter().any(|c| matches!(c.owner, Owner::Inode { storage: tree::Role::Data { logical, unwritten: true }, .. } if logical >= 1024)));
                let used: u64 = expected
                    .iter()
                    .filter(|c| {
                        !matches!(
                            c.owner,
                            Owner::Fixed {
                                role: fixed_metadata::Role::Boot,
                                ..
                            }
                        )
                    })
                    .map(|c| c.end - c.start)
                    .sum();
                assert_eq!(
                    used,
                    fs.sb.blocks_count - u64::from(fs.sb.first_data_block) - free_count.unwrap()
                );
                std::fs::write(base.with_extension("summary.txt"), format!("block_size={bs} descriptor={descriptor} journal={journal} inodes={} claims={} used={used} charged_reads={} tree_records={}\n", got.inodes.len(), got.claims.len(), got.charged_reads, got.tree_records)).unwrap();
            }
        }
    }
}
