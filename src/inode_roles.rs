//! Private current inode-role interpretation, not allocation/ownership proof.
//! Formats: https://docs.kernel.org/filesystems/ext4/inodes.html and
//! https://docs.kernel.org/filesystems/ext4/checksums.html

use crate::checksum::linux_crc32c;
use crate::error::{Error, Result};
use crate::features::{Compat, Incompat, RoCompat};
use crate::fs::{BlockBuffer, Filesystem};
use crate::inode::{Inode, InodeFlags, S_IFDIR, S_IFIFO, S_IFLNK, S_IFMT, S_IFREG};

#[derive(Clone, Copy)]
pub(crate) struct Limits {
    /// All attempted records, shared across calls on this reader.
    pub records: u64,
    /// Logical inode-table fetches, including staged copies and failed reads.
    pub reads: u64,
    /// One table block plus a fixed 256-byte checksum copy on the stack.
    pub scratch_bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    InactiveReserved,
    RootDirectory,
    Journal,
    Regular,
    Directory,
    Symlink,
    Fifo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Storage {
    None,
    FastSymlink,
    /// Only the inline header is checked here. The complete extent collector
    /// must verify every declared record/child before these become claims.
    Extents {
        depth: u16,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Record {
    pub ino: u32,
    pub generation: u32,
    pub role: Role,
    pub storage: Storage,
    pub size: u64,
    pub sectors: u64,
    pub links: u16,
    /// Verified inode bytes: extent root, fast symlink, or zero storage.
    pub block: [u8; 60],
}

const LIMIT: Error = Error::Unsupported("inode role resource limit");
const PROFILE: Error = Error::Unsupported("inode role profile");

/// Prerequisite: a successful current inode inventory on this same trusted
/// mounted geometry and stable device/buffer view. Pass only its allocated IDs.
/// Neither allocation nor descriptors are revalidated here. No epoch, snapshot,
/// cache, claims or write admission is published. Allocation/body/map edits end
/// that stable-view precondition; a later builder must enforce the lifecycle.
pub(crate) struct Reader<'a> {
    fs: &'a Filesystem,
    buf: &'a BlockBuffer,
    scratch: Vec<u8>,
    records_left: u64,
    reads_left: u64,
    journal: bool,
}

impl<'a> Reader<'a> {
    pub fn new(fs: &'a Filesystem, buf: &'a BlockBuffer, limits: Limits) -> Result<Self> {
        let sb = &fs.sb;
        let compat = Compat::HAS_JOURNAL.bits()
            | Compat::EXT_ATTR.bits()
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
        let bs = sb.block_size() as usize;
        if !fs.csum.enabled
            || sb.feature_ro_compat & RoCompat::METADATA_CSUM.bits() == 0
            || sb.feature_incompat & Incompat::EXTENTS.bits() == 0
            || sb.feature_compat & !compat != 0
            || sb.feature_incompat & !incompat != 0
            || sb.feature_ro_compat & !ro != 0
            || sb.creator_os != 0
            || sb.first_inode != 11
            || sb.last_orphan != 0
            || sb.inode_size != 256
            || !matches!(bs, 1024 | 4096)
        {
            return Err(PROFILE);
        }
        sb.casefold_encoding()?;
        let raw = sb.raw.get(..1024).ok_or(PROFILE)?;
        let journal = sb.feature_compat & Compat::HAS_JOURNAL.bits() != 0;
        let inum = u32::from_le_bytes(raw[0xe0..0xe4].try_into().unwrap());
        if inum != if journal { 8 } else { 0 }
            || raw[0xd0..0xe0].iter().any(|b| *b != 0)
            || raw[0xe4..0xe8].iter().any(|b| *b != 0)
        {
            return Err(PROFILE);
        }
        if limits.scratch_bytes < bs + 256 {
            return Err(LIMIT);
        }
        let mut scratch = Vec::new();
        scratch.try_reserve_exact(bs).map_err(|_| LIMIT)?;
        scratch.resize(bs, 0);
        Ok(Self {
            fs,
            buf,
            scratch,
            records_left: limits.records,
            reads_left: limits.reads,
            journal,
        })
    }

    pub fn read(&mut self, ino: u32) -> Result<Record> {
        self.records_left = self.records_left.checked_sub(1).ok_or(LIMIT)?;
        let (block, offset) = crate::bgd::locate_inode(&self.fs.sb, &self.fs.groups, ino)?;
        let bs = self.scratch.len();
        if block < u64::from(self.fs.sb.first_data_block).max(1)
            || block >= self.fs.byte_ceiling()? / bs as u64
        {
            return Err(Error::InvalidBlock(block));
        }
        self.reads_left = self.reads_left.checked_sub(1).ok_or(LIMIT)?;
        match self.buf.dirty.get(&block) {
            Some(bytes) if bytes.len() == bs => self.scratch.copy_from_slice(bytes),
            Some(_) => return Err(Error::Corrupt("inode role staged block size")),
            None => self.fs.dev.read_at(
                block
                    .checked_mul(bs as u64)
                    .ok_or(Error::InvalidBlock(block))?,
                &mut self.scratch,
            )?,
        }
        let raw = self
            .scratch
            .get(offset as usize..offset as usize + 256)
            .ok_or(Error::Corrupt("inode role body bounds"))?;
        classify(self.fs, self.journal, ino, raw)
    }
}

fn classify(fs: &Filesystem, journal: bool, ino: u32, raw: &[u8]) -> Result<Record> {
    let inode = Inode::parse(raw)?;
    let extra = u16::from_le_bytes(raw[128..130].try_into().unwrap()) as usize;
    // This initial profile qualifies the observed legacy-sized extra section
    // and the common 32-byte section, not arbitrary future fields or xattrs.
    if !matches!(extra, 0 | 32) {
        return Err(PROFILE);
    }
    let mut checksum_bytes: [u8; 256] = raw.try_into().map_err(|_| PROFILE)?;
    checksum_bytes[124..126].fill(0);
    if extra >= 4 {
        checksum_bytes[130..132].fill(0);
    }
    let crc = linux_crc32c(fs.csum.seed, &ino.to_le_bytes());
    let crc = linux_crc32c(crc, &inode.generation.to_le_bytes());
    let crc = linux_crc32c(crc, &checksum_bytes);
    let stored = u32::from(u16::from_le_bytes(raw[124..126].try_into().unwrap()))
        | if extra >= 4 {
            u32::from(u16::from_le_bytes(raw[130..132].try_into().unwrap())) << 16
        } else {
            0
        };
    if crc & if extra >= 4 { u32::MAX } else { 0xffff } != stored {
        return Err(Error::BadChecksum { what: "inode" });
    }
    if inode.file_acl != 0
        || raw[128 + extra..].iter().any(|b| *b != 0)
        || raw[0x70..0x74].iter().any(|b| *b != 0)
    {
        return Err(Error::Unsupported("inode role attributes/fragment"));
    }
    let kind = inode.mode & S_IFMT;
    let role = if ino == 2 {
        if kind != S_IFDIR {
            return Err(Error::Corrupt("inode role root type"));
        }
        Role::RootDirectory
    } else if journal && ino == 8 {
        if kind != S_IFREG || inode.size == 0 {
            return Err(Error::Corrupt("inode role journal type/size"));
        }
        Role::Journal
    } else if ino < fs.sb.first_inode {
        if inode.mode != 0 || inode.links_count != 0 {
            return Err(PROFILE);
        }
        Role::InactiveReserved
    } else {
        match kind {
            S_IFREG => Role::Regular,
            S_IFDIR => Role::Directory,
            S_IFLNK => Role::Symlink,
            S_IFIFO => Role::Fifo,
            _ => return Err(PROFILE),
        }
    };
    let extent_flag = InodeFlags::EXTENTS.bits();
    let storage = match role {
        Role::InactiveReserved | Role::Fifo => {
            if inode.flags != 0 || inode.size != 0 || inode.blocks != 0 || inode.block != [0; 60] {
                return Err(PROFILE);
            }
            Storage::None
        }
        Role::Symlink if inode.blocks == 0 => {
            let length = usize::try_from(inode.size).map_err(|_| PROFILE)?;
            if inode.flags != 0
                || length == 0
                || length > 60
                || inode.block[..length].contains(&0)
                || inode.block[length..].iter().any(|b| *b != 0)
            {
                return Err(PROFILE);
            }
            Storage::FastSymlink
        }
        _ => {
            let allowed = match role {
                Role::RootDirectory | Role::Directory => {
                    inode.directory_casefold_encoding(&fs.sb)?;
                    extent_flag | InodeFlags::INDEX.bits() | InodeFlags::CASEFOLD.bits()
                }
                _ => extent_flag,
            };
            if inode.flags & extent_flag == 0 || inode.flags & !allowed != 0 {
                return Err(PROFILE);
            }
            let header = crate::extent::ExtentHeader::parse(&inode.block)?;
            if header.max == 0
                || header.max > 4
                || header.depth > 1
                || (header.depth != 0 && header.entries == 0)
                || (role == Role::Journal && header.depth != 0)
            {
                return Err(PROFILE);
            }
            Storage::Extents {
                depth: header.depth,
            }
        }
    };
    Ok(Record {
        ino,
        generation: inode.generation,
        role,
        storage,
        size: inode.size,
        sectors: inode.blocks,
        links: inode.links_count,
        block: inode.block,
    })
}

#[cfg(test)]
mod needs_host {
    use super::*;

    #[test]
    fn linux_roles_match_every_allocated_debugfs_record() {
        use crate::block_io::FileDevice;
        use fs_ext4_test_support::{assert_e2fsck_clean, guest_kernel_write, oracle};
        use std::collections::BTreeMap;
        use std::sync::Arc;
        for bs in [1024u32, 4096] {
            for (descriptor, journal) in [(32, false), (64, false), (64, true)] {
                let base = fs_ext4_test_support::temp_dir()
                    .join(format!("inode-roles-{bs}-{descriptor}-{journal}"));
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
                let image = image.to_str().unwrap();
                let script = format!(
                    "python3 - <<'INODE_ROLES_PY'\n{}\nINODE_ROLES_PY\n",
                    include_str!("../tests/inode_roles_guest.py")
                );
                let populated = guest_kernel_write(image, &script);
                assert!(
                    populated.status.success(),
                    "{}",
                    String::from_utf8_lossy(&populated.stderr)
                );
                assert_e2fsck_clean(image, "Linux inode-role control");
                let fs = Filesystem::mount(Arc::new(FileDevice::open(image).unwrap())).unwrap();
                assert_eq!(fs.sb.desc_size, descriptor);
                assert_eq!((fs.groups.len(), fs.sb.inodes_per_group), (2, 32));
                let buf = BlockBuffer::new(bs);
                let inventory = crate::inode_inventory::collect(
                    &fs,
                    &buf,
                    crate::inode_inventory::Limits {
                        groups: 2,
                        bit_probes: 2 * u64::from(bs) * 8,
                        reads: 4,
                        inodes: 64,
                        scratch_bytes: 2 * bs as usize,
                    },
                )
                .unwrap();
                let commands = base.with_extension("commands.txt");
                std::fs::write(
                    &commands,
                    inventory
                        .inodes
                        .iter()
                        .map(|ino| format!("stat <{ino}>\n"))
                        .collect::<String>(),
                )
                .unwrap();
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
                std::fs::write(base.with_extension("roles.txt"), &report).unwrap();
                let mut types = BTreeMap::new();
                let mut blockcounts = BTreeMap::new();
                let mut current = None;
                for line in report.lines() {
                    if let Some(header) = line.strip_prefix("Inode:") {
                        let (ino, rest) = header.split_once("Type:").expect("missing oracle type");
                        let ino: u32 = ino.trim().parse().unwrap();
                        let (kind, _) = rest.split_once("Mode:").expect("missing oracle mode");
                        assert!(types.insert(ino, kind.trim().to_lowercase()).is_none());
                        current = Some(ino);
                    }
                    if let Some((_, sectors)) = line.split_once("Blockcount:") {
                        assert!(blockcounts
                            .insert(current.unwrap(), sectors.trim().parse::<u64>().unwrap())
                            .is_none());
                    }
                }
                assert_eq!(types.len(), inventory.inodes.len());
                assert_eq!(blockcounts.len(), inventory.inodes.len());
                let mut reader = Reader::new(
                    &fs,
                    &buf,
                    Limits {
                        records: inventory.inodes.len() as u64,
                        reads: inventory.inodes.len() as u64,
                        scratch_bytes: bs as usize + 256,
                    },
                )
                .unwrap();
                let mut fast = false;
                let mut long = false;
                let mut external = false;
                let mut fifo = false;
                let mut journal_seen = false;
                for ino in inventory.inodes {
                    let record = reader.read(ino).unwrap();
                    let expected = match record.role {
                        Role::InactiveReserved => "bad type",
                        Role::RootDirectory | Role::Directory => "directory",
                        Role::Regular | Role::Journal => "regular",
                        Role::Symlink => "symlink",
                        Role::Fifo => "fifo",
                    };
                    assert_eq!(types.get(&ino).unwrap(), expected, "inode {ino}");
                    assert_eq!(blockcounts[&ino], record.sectors);
                    fast |= record.role == Role::Symlink && record.storage == Storage::FastSymlink;
                    long |= record.role == Role::Symlink
                        && matches!(record.storage, Storage::Extents { depth: 0 });
                    external |= record.role == Role::Regular
                        && record.storage == Storage::Extents { depth: 1 };
                    fifo |= record.role == Role::Fifo;
                    journal_seen |= record.role == Role::Journal && ino == 8;
                }
                assert!(fast && long && external && fifo);
                assert_eq!(journal_seen, journal);
            }
        }
    }
}
