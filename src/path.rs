//! Path-to-inode resolution.
//!
//! Walk a slash-separated byte path from the root directory (inode 2) down to
//! a target inode number. Used by every public-facing C API function that
//! accepts a path (stat, dir_open, read_file, readlink).
//!
//! Algorithm: start at `EXT4_ROOT_INODE`, read its inode, for each non-empty
//! path component look up the entry by name in the current directory's data
//! blocks, then descend into the matching child inode.
//!
//! An htree-indexed directory is searched through its index first
//! (`htree::lookup_leaf_with`), and a name the index does not lead to is
//! then found by a linear scan, which is always valid because the index is
//! an acceleration over ordinary directory blocks, not a replacement for
//! them. Casefold lookups validate their index and scan all referenced leaves
//! to detect equivalent names across blocks. Other directories are scanned linearly.

use crate::block_io::BlockDevice;
use crate::dir::{self, DirEntry};
use crate::error::{Error, Result};
use crate::htree;
use crate::indirect;
use crate::inline_data;
use crate::inode::{Inode, InodeFlags};
use crate::superblock::Superblock;

/// Root directory inode number (always 2 on every ext[234] filesystem).
pub const EXT4_ROOT_INODE: u32 = 2;

/// Resolve a slash-separated path to an inode number.
///
/// `path` is expected in the form `/a/b/c` (leading slash accepted, trailing
/// slash accepted, empty path returns the root). Non-UTF-8 bytes in directory
/// entries are compared literally in ordinary directories. Casefold directories
/// use their frozen encoding and refuse unqualified malformed names.
///
/// Returns:
/// - `Ok(inode)` — inode number for the resolved path
/// - `Err(Error::NotFound)` — any component did not exist
/// - `Err(Error::NotADirectory)` — a non-dir component appeared mid-path
/// - `Err(Error::Io(..))` / other corruption errors — disk I/O or malformed data
pub fn lookup<F>(
    dev: &dyn BlockDevice,
    sb: &Superblock,
    read_inode: &mut F,
    path: &str,
) -> Result<u32>
where
    F: FnMut(u32) -> Result<Inode>,
{
    // Backwards-compatible shim — defers to `lookup_with_csum` with a
    // disabled `Checksummer` so callers that don't have one keep working
    // (verification is silently skipped).
    //
    // FOR CALLERS THAT GENUINELY HAVE NO `Checksummer`, AND NOTHING ELSE.
    // Sixteen sites in `fs.rs` used this from inside methods holding
    // `&self`, and therefore holding `self.csum` — so every mutating C
    // entry point resolved its path with verification off, while
    // `capi::resolve_path` resolved with it on. A corrupt directory block
    // was refused by `stat` and accepted by `unlink`, `rename`, `mkdir`,
    // `rmdir`, `link`, `chmod` and the rest, which then edited the block
    // and re-stamped a fresh valid checksum over it.
    //
    // THE SEED IS WRONG AS WELL AS THE FLAG, which is why the remedy is
    // to pass the real `Checksummer` and never to flip `enabled` here:
    // `seed: 0` would verify good blocks against the wrong seed and
    // reject them.
    let csum = crate::checksum::Checksummer {
        seed: 0,
        enabled: false,
    };
    lookup_with_csum(dev, sb, read_inode, path, &csum)
}

/// Path → inode lookup with directory-block checksum verification.
///
/// Identical to `lookup`, but passes the supplied `Checksummer` down to
/// `find_entry` so each directory block's CRC32C tail is verified
/// (when present and `csum.enabled`). Callers with a mounted `Filesystem`
/// should pass `&fs.csum`.
pub fn lookup_with_csum<F>(
    dev: &dyn BlockDevice,
    sb: &Superblock,
    read_inode: &mut F,
    path: &str,
    csum: &crate::checksum::Checksummer,
) -> Result<u32>
where
    F: FnMut(u32) -> Result<Inode>,
{
    lookup_bytes_with_csum(dev, sb, read_inode, path.as_bytes(), csum)
}

/// [`lookup_with_csum`] of a path given as bytes, which is what a path is:
/// ordinary directories compare components byte for byte, including non-UTF-8.
/// Casefold directories compare valid UTF-8 using the volume's frozen Unicode
/// encoding; malformed bytes and normalized dot aliases are refused. A missing
/// supported name is `NotFound` — never the root by accident (#418).
///
/// The `&str` functions above are this with `str::as_bytes`, so a caller
/// passing UTF-8 sees no difference.
pub fn lookup_bytes_with_csum<F>(
    dev: &dyn BlockDevice,
    sb: &Superblock,
    read_inode: &mut F,
    path: &[u8],
    csum: &crate::checksum::Checksummer,
) -> Result<u32>
where
    F: FnMut(u32) -> Result<Inode>,
{
    let mut current_ino: u32 = EXT4_ROOT_INODE;

    for name in split_path(path) {
        let inode = read_inode(current_ino)?;
        if !inode.is_dir() {
            return Err(Error::NotADirectory);
        }
        current_ino = find_entry(dev, sb, current_ino, &inode, name, csum)?;
    }

    Ok(current_ino)
}

/// Read one directory's entries and return the inode number matching `name`.
///
/// Ordinary indexed directories use the htree fast path with a linear fallback.
/// Casefold directories scan their validated index leaves or their linear blocks.
pub(crate) fn find_entry(
    dev: &dyn BlockDevice,
    sb: &Superblock,
    dir_ino: u32,
    dir_inode: &Inode,
    name: &[u8],
    csum: &crate::checksum::Checksummer,
) -> Result<u32> {
    crate::file_io::refuse_encrypted_names(dir_inode)?;
    // Validate the actual parent at every path component, including direct
    // path API calls that did not pass through Filesystem::mount. Unsupported
    // encoding or inconsistent inode flags must not become a false absence.
    let encoding = dir_inode.directory_casefold_encoding(sb)?;
    if encoding.is_some() && dir_inode.has_inline_data() {
        return Err(Error::Unsupported("inline casefold directory lookup"));
    }
    let lookup_name = crate::casefold::LookupName::new(name, encoding)?;
    // Both extent-backed and legacy direct/indirect-backed directories are
    // supported here — `find_entry_linear` and `find_entry_htree` use
    // `indirect::map_logical_any` for flavor-aware logical→physical mapping.
    if dir_inode.has_inline_data() {
        return find_inline(dev, sb, dir_ino, dir_inode, name, csum);
    }

    let has_filetype = sb.feature_incompat & crate::features::Incompat::FILETYPE.bits() != 0;
    let block_size = sb.block_size();

    // Casefold scans every referenced leaf after validating the index. This
    // covers collision continuations and refuses equivalent entries in separate
    // leaves. Hash-directed casefold acceleration can follow separate qualification.
    // Ordinary indexed directories retain their existing fast path.
    if !lookup_name.is_folded() && (dir_inode.flags & InodeFlags::INDEX.bits()) != 0 {
        if let Some(found) = find_entry_htree(
            dev,
            sb,
            dir_ino,
            dir_inode,
            &lookup_name,
            has_filetype,
            csum,
        )? {
            return Ok(found);
        }
        // htree said "not in expected leaf" — fall through to a full linear
        // scan as a safety net (covers edge cases / corruption).
    }

    find_entry_linear(
        dev,
        sb,
        dir_ino,
        dir_inode,
        &lookup_name,
        has_filetype,
        block_size,
        csum,
    )
}

/// Linear scan of every directory data block.
/// For casefold indexes, only scan leaves reachable through the validated index.
#[allow(clippy::too_many_arguments)]
fn find_entry_linear(
    dev: &dyn BlockDevice,
    _sb: &Superblock,
    dir_ino: u32,
    dir_inode: &Inode,
    name: &crate::casefold::LookupName<'_>,
    has_filetype: bool,
    block_size: u32,
    csum: &crate::checksum::Checksummer,
) -> Result<u32> {
    let dir_size = dir_inode.size;
    let total_blocks = dir_size.div_ceil(block_size as u64);
    let gen = dir_inode.generation;
    let mut found = None;

    let mut block = vec![0u8; block_size as usize];
    let indexed_leaves = if name.is_folded() && dir_inode.flag_set().contains(InodeFlags::INDEX) {
        let root =
            indirect::map_logical_any(&dir_inode.block, dir_inode.flags, dev, block_size, 0)?
                .ok_or(Error::CorruptDirEntry("casefold index root is sparse"))?;
        dev.read_at(root * block_size as u64, &mut block)?;
        Some(casefold_index_leaves(
            &block,
            dir_ino,
            dir_inode,
            csum,
            total_blocks,
            |logical| {
                let physical = indirect::map_logical_any(
                    &dir_inode.block,
                    dir_inode.flags,
                    dev,
                    block_size,
                    u64::from(logical),
                )?
                .ok_or(Error::CorruptDirEntry("casefold index node is sparse"))?;
                let mut node = vec![0; block_size as usize];
                dev.read_at(physical * u64::from(block_size), &mut node)?;
                Ok(node)
            },
        )?)
    } else {
        None
    };
    let scan_count = indexed_leaves
        .as_ref()
        .map_or(total_blocks, |leaves| leaves.len() as u64);
    for index in 0..scan_count {
        let logical = indexed_leaves
            .as_ref()
            .map_or(index, |leaves| u64::from(leaves[index as usize]));
        let phys = match indirect::map_logical_any(
            &dir_inode.block,
            dir_inode.flags,
            dev,
            block_size,
            logical,
        )? {
            Some(p) => p,
            None if indexed_leaves.is_some() => {
                return Err(Error::CorruptDirEntry("casefold index leaf is sparse"));
            }
            None => continue,
        };
        dev.read_at(phys * block_size as u64, &mut block)?;

        if indexed_leaves.is_some() && csum.enabled && !dir::has_csum_tail(&block) {
            return Err(Error::BadChecksum {
                what: "directory block",
            });
        }

        // The first block of an indexed dir is the dx_root and *cannot* be
        // parsed as linear entries (its contents after "." and ".." are
        // dx_entry records, not dir entries). Skip parse errors there.
        match dir::parse_block_verified(&block, has_filetype, dir_ino, gen, csum) {
            Ok(entries) => {
                for entry in entries {
                    if name.matches(&entry.name)? {
                        if !name.is_folded() {
                            return Ok(entry.inode);
                        }
                        name.record_match(&mut found, entry.inode)?;
                    }
                }
            }
            Err(_)
                if !name.is_folded()
                    && logical == 0
                    && (dir_inode.flags & InodeFlags::INDEX.bits()) != 0 =>
            {
                // dx_root in an indexed dir — only "." and ".." matter here,
                // and find_entry_htree already handled the indexed path.
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    found.ok_or(Error::NotFound)
}

/// Enumerate a root and at most one intermediate level, validating every
/// index block before reading leaves. The visited set is shared across roles:
/// a node cannot also be a leaf, and cycles/repeated pointers are corruption.
/// Layout: https://docs.kernel.org/filesystems/ext4/directory.html#hash-tree-directories
fn casefold_index_leaves<F>(
    block: &[u8],
    ino: u32,
    inode: &Inode,
    csum: &crate::checksum::Checksummer,
    total_blocks: u64,
    mut read_node: F,
) -> Result<Vec<u32>>
where
    F: FnMut(u32) -> Result<Vec<u8>>,
{
    let info = htree::parse_root_info(block)?;
    if info.indirect_levels > 1 || info.unused_flags != 0 {
        return Err(Error::Unsupported("casefold index depth or flags"));
    }
    if info.info_length != 8 {
        return Err(Error::Corrupt("casefold index info_length is not 8"));
    }
    let mut seen = std::collections::HashSet::from([0]);
    let mut validate = |bytes: &[u8], root: bool| -> Result<Vec<u32>> {
        let offset = if root { 32 } else { 8 };
        if csum.enabled && csum.verify_dx_tail(ino, inode.generation, bytes, offset) != Some(true) {
            return Err(Error::BadChecksum {
                what: "htree index block",
            });
        }
        let (limit, entries) = if root {
            htree::parse_root_entries(bytes)?
        } else {
            // Larger-block rec_len encodings require separate qualification.
            if bytes.len() > u16::MAX as usize {
                return Err(Error::Unsupported("large casefold index node"));
            }
            if bytes.len() < 8
                || bytes[..4] != [0; 4]
                || bytes[6..8] != [0; 2]
                || usize::from(u16::from_le_bytes([bytes[4], bytes[5]])) != bytes.len()
            {
                return Err(Error::CorruptDirEntry("invalid casefold index node header"));
            }
            htree::parse_node_entries(bytes)?
        };
        let tail = if csum.enabled { 8 } else { 0 };
        if usize::from(limit.limit) != (bytes.len() - offset - tail) / 8 {
            return Err(Error::CorruptDirEntry("invalid casefold index capacity"));
        }
        if entries.windows(2).any(|pair| pair[0].hash > pair[1].hash) {
            return Err(Error::CorruptDirEntry(
                "casefold index hashes are not ordered",
            ));
        }
        for entry in &entries {
            if u64::from(entry.block) >= total_blocks || !seen.insert(entry.block) {
                return Err(Error::CorruptDirEntry(
                    "invalid or repeated casefold index block",
                ));
            }
        }
        Ok(entries.iter().map(|entry| entry.block).collect())
    };
    let children = validate(block, true)?;
    if info.indirect_levels == 0 {
        return Ok(children);
    }
    let mut leaves = Vec::new();
    for child in children {
        leaves.extend(validate(&read_node(child)?, false)?);
    }
    Ok(leaves)
}

/// HTree-indexed lookup. Returns:
///   Ok(Some(ino)) — found
///   Ok(None)      — htree said not in any leaf (caller may fall back)
///   Err(..)       — corruption or I/O error
fn find_entry_htree(
    dev: &dyn BlockDevice,
    sb: &Superblock,
    dir_ino: u32,
    dir_inode: &Inode,
    name: &crate::casefold::LookupName<'_>,
    has_filetype: bool,
    csum: &crate::checksum::Checksummer,
) -> Result<Option<u32>> {
    let block_size = sb.block_size();

    // Read logical block 0 of the directory: the dx_root. Flavor-aware
    // dispatch — htree directories on ext2/3 (rare but legal) use the
    // legacy block-map scheme, not extents.
    let phys0 =
        match indirect::map_logical_any(&dir_inode.block, dir_inode.flags, dev, block_size, 0)? {
            Some(p) => p,
            None => return Ok(None),
        };
    let mut root_block = vec![0u8; block_size as usize];
    dev.read_at(phys0 * block_size as u64, &mut root_block)?;

    // Walk the htree. lookup_leaf needs a closure for reading further dx
    // blocks (intermediate nodes); we map logical→physical via whichever
    // block-map scheme the dir's inode uses.
    let read_dx_block = |logical: u32| -> Result<Vec<u8>> {
        let phys = indirect::map_logical_any(
            &dir_inode.block,
            dir_inode.flags,
            dev,
            block_size,
            logical as u64,
        )?
        .ok_or(Error::CorruptDirEntry("htree pointed at sparse block"))?;
        let mut buf = vec![0u8; block_size as usize];
        dev.read_at(phys * block_size as u64, &mut buf)?;
        Ok(buf)
    };

    let leaf_logical = match htree::lookup_leaf_with(
        name.hash_bytes(),
        &root_block,
        &sb.hash_seed,
        sb.unsigned_hash(),
        read_dx_block,
    )? {
        Some(b) => b,
        None => return Ok(None),
    };

    // Read the leaf block and linear-scan it for the name.
    let phys = indirect::map_logical_any(
        &dir_inode.block,
        dir_inode.flags,
        dev,
        block_size,
        leaf_logical as u64,
    )?
    .ok_or(Error::CorruptDirEntry("htree leaf at sparse block"))?;
    let mut leaf = vec![0u8; block_size as usize];
    dev.read_at(phys * block_size as u64, &mut leaf)?;

    // Verify the leaf block's csum tail (if present) before scanning.
    if csum.enabled
        && dir::has_csum_tail(&leaf)
        && !csum.verify_dir_entry_tail(dir_ino, dir_inode.generation, &leaf)
    {
        return Err(Error::BadChecksum {
            what: "directory block",
        });
    }

    let mut found = None;
    for entry in dir::DirBlockIter::new(&leaf, has_filetype) {
        let entry: DirEntry = entry?;
        if name.matches(&entry.name)? {
            if !name.is_folded() {
                return Ok(Some(entry.inode));
            }
            name.record_match(&mut found, entry.inode)?;
        }
    }

    // Name not in the htree-selected leaf. Could be hash collision spilling
    // to neighbouring leaf — caller will fall back to linear scan.
    Ok(found)
}

/// Look `name` up in an inline-data directory: `.`, `..` from bytes 0..4 of
/// `i_block`, the entries from byte 4, then -- only when the name is not
/// among those and `i_size` says there is more -- the entries continued in
/// the `system.data` xattr (#427).
fn find_inline(
    dev: &dyn BlockDevice,
    sb: &Superblock,
    dir_ino: u32,
    dir_inode: &Inode,
    name: &[u8],
    csum: &crate::checksum::Checksummer,
) -> Result<u32> {
    let has_filetype = sb.feature_incompat & crate::features::Incompat::FILETYPE.bits() != 0;
    let in_block = inline_data::dir_entries(dir_ino, dir_inode, &[], has_filetype)?;
    if let Some(entry) = in_block.iter().find(|e| e.name == name) {
        return Ok(entry.inode);
    }
    if dir_inode.size as usize <= inline_data::INLINE_BLOCK_SIZE {
        return Err(Error::NotFound);
    }
    let raw = read_raw_inode(dev, sb, dir_ino, dir_inode, csum)?;
    let continuation =
        inline_data::dir_continuation(dev, dir_inode, &raw, sb.inode_size, sb.block_size())?;
    for entry in dir::DirBlockIter::new(&continuation, has_filetype) {
        let entry = entry?;
        if entry.name == name {
            return Ok(entry.inode);
        }
    }
    Err(Error::NotFound)
}

/// The on-disk bytes of `ino`, for the in-inode xattr an inline directory
/// continues into. The lookup's `read_inode` hands back a parsed [`Inode`]
/// only, so the inode table is found through the one group descriptor it
/// needs. The bytes must be the inode the lookup was given: checksum-valid
/// when checksums are on, and with the same `i_block`, size and generation.
fn read_raw_inode(
    dev: &dyn BlockDevice,
    sb: &Superblock,
    ino: u32,
    expected: &Inode,
    csum: &crate::checksum::Checksummer,
) -> Result<Vec<u8>> {
    if ino == 0 || ino > sb.inodes_count || sb.inodes_per_group == 0 {
        return Err(Error::InvalidInode(ino));
    }
    let group = u64::from((ino - 1) / sb.inodes_per_group);
    if group >= sb.block_group_count() {
        return Err(Error::InvalidInode(ino));
    }
    let block_size = u64::from(sb.block_size());
    let desc_size = sb.desc_size as usize;
    let (desc_block, desc_off) = sb.descriptor_location(group);
    let mut table = vec![0u8; block_size as usize];
    dev.read_at(
        desc_block
            .checked_mul(block_size)
            .ok_or(Error::Corrupt("group descriptor block number overflow"))?,
        &mut table,
    )?;
    let desc = table
        .get(desc_off..desc_off + desc_size)
        .ok_or(Error::Corrupt("group descriptor outside its block"))?;
    let bgd = crate::bgd::BlockGroupDescriptor::parse(desc, sb.desc_size)?;
    let inode_size = u64::from(sb.inode_size);
    let local = u64::from((ino - 1) % sb.inodes_per_group);
    let offset = bgd
        .inode_table
        .checked_mul(block_size)
        .and_then(|b| b.checked_add(local * inode_size))
        .ok_or(Error::Corrupt("inode table offset overflow"))?;
    let mut raw = vec![0u8; inode_size as usize];
    dev.read_at(offset, &mut raw)?;
    let parsed = Inode::parse(&raw)?;
    if csum.enabled && !csum.verify_inode(ino, parsed.generation, &raw) {
        return Err(Error::BadChecksum { what: "inode" });
    }
    if parsed.block != expected.block
        || parsed.size != expected.size
        || parsed.generation != expected.generation
    {
        return Err(Error::Corrupt(
            "inline directory: the inode table disagrees with the inode being looked up",
        ));
    }
    Ok(raw)
}

/// Split "/foo/bar/baz" into ["foo", "bar", "baz"]. Empty components (from
/// doubled slashes or leading/trailing slashes) are dropped.
fn split_path(path: &[u8]) -> impl Iterator<Item = &[u8]> {
    path.split(|&b| b == b'/').filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_path_basic() {
        let split = |p: &'static [u8]| split_path(p).collect::<Vec<_>>();
        let none: Vec<&[u8]> = Vec::new();
        assert_eq!(split(b""), none);
        assert_eq!(split(b"/"), none);
        assert_eq!(split(b"/foo"), vec![&b"foo"[..]]);
        assert_eq!(split(b"/foo/bar"), vec![&b"foo"[..], b"bar"]);
        assert_eq!(split(b"foo/bar"), vec![&b"foo"[..], b"bar"]);
        assert_eq!(split(b"/foo//bar/"), vec![&b"foo"[..], b"bar"]);
        assert_eq!(split(b"///"), none);
    }

    /// A component is bytes: one that is not UTF-8 survives the split
    /// exactly, rather than being decoded or dropped (#418).
    #[test]
    fn split_path_keeps_bytes_that_are_not_utf8() {
        let parts: Vec<&[u8]> = split_path(b"/d\xff/caf\xe9.txt").collect();
        assert_eq!(parts, vec![&b"d\xff"[..], b"caf\xe9.txt"]);
    }

    /// Tests that read a fixture from `test-disks/` (`chore fixtures`).
    mod needs_host {
        use super::*;
        use crate::bgd;
        use crate::block_io::FileDevice;
        use crate::extent;
        use crate::fs::Filesystem;
        use std::sync::Arc;

        /// Build a read_inode closure that reads raw bytes via Filesystem and
        /// parses them through Inode::parse.
        fn read_inode_fn(fs: &Filesystem) -> impl FnMut(u32) -> Result<Inode> + '_ {
            move |ino: u32| {
                let (block, offset) = bgd::locate_inode(&fs.sb, &fs.groups, ino)?;
                let block_data = fs.read_block(block)?;
                let inode_size = fs.sb.inode_size as usize;
                let off = offset as usize;
                Inode::parse(&block_data[off..off + inode_size])
            }
        }

        #[test]
        fn root_resolves_to_inode_2() {
            let path = fs_ext4_test_support::fixture(env!("CARGO_MANIFEST_DIR"), "ext4-basic.img");
            let file = FileDevice::open(&path).unwrap_or_else(|e| panic!("open {path}: {e}"));
            let dev: Arc<dyn BlockDevice> = Arc::new(file);
            let fs = Filesystem::mount(dev.clone()).expect("mount");
            let mut reader = read_inode_fn(&fs);

            for root_path in ["/", "", "///"] {
                let ino = lookup(dev.as_ref(), &fs.sb, &mut reader, root_path)
                    .unwrap_or_else(|e| panic!("lookup({root_path:?}) failed: {e}"));
                assert_eq!(ino, EXT4_ROOT_INODE, "path {root_path:?}");
            }
        }

        #[test]
        fn missing_path_returns_not_found() {
            let path = fs_ext4_test_support::fixture(env!("CARGO_MANIFEST_DIR"), "ext4-basic.img");
            let file = FileDevice::open(&path).unwrap_or_else(|e| panic!("open {path}: {e}"));
            let dev: Arc<dyn BlockDevice> = Arc::new(file);
            let fs = Filesystem::mount(dev.clone()).expect("mount");
            let mut reader = read_inode_fn(&fs);

            let result = lookup(
                dev.as_ref(),
                &fs.sb,
                &mut reader,
                "/this-does-not-exist-xyz",
            );
            assert!(matches!(result, Err(Error::NotFound)), "got {result:?}");
        }

        #[test]
        fn non_dir_component_returns_not_a_directory() {
            let path = fs_ext4_test_support::fixture(env!("CARGO_MANIFEST_DIR"), "ext4-basic.img");
            let file = FileDevice::open(&path).unwrap_or_else(|e| panic!("open {path}: {e}"));
            let dev: Arc<dyn BlockDevice> = Arc::new(file);
            let fs = Filesystem::mount(dev.clone()).expect("mount");
            let mut reader = read_inode_fn(&fs);

            // Find any regular file in root so we can stack a component after it.
            let root = reader(EXT4_ROOT_INODE).expect("root inode");
            let block_size = fs.sb.block_size();
            let total_blocks = root.size.div_ceil(block_size as u64);
            let has_filetype =
                fs.sb.feature_incompat & crate::features::Incompat::FILETYPE.bits() != 0;

            let mut reg_file_name: Option<Vec<u8>> = None;
            'outer: for logical in 0..total_blocks {
                if let Some(phys) =
                    extent::map_logical(&root.block, dev.as_ref(), block_size, logical)
                        .expect("map logical")
                {
                    let mut blk = vec![0u8; block_size as usize];
                    dev.read_at(phys * block_size as u64, &mut blk).unwrap();
                    for entry in dir::DirBlockIter::new(&blk, has_filetype) {
                        let e = entry.expect("entry");
                        if e.file_type == dir::DirEntryType::RegFile {
                            reg_file_name = Some(e.name);
                            break 'outer;
                        }
                    }
                }
            }

            let name = reg_file_name.expect(
                "ext4-basic.img has a regular file (test.txt) in its root \
             (test-disks/guest-build-images.sh)",
            );
            let name_str = std::str::from_utf8(&name).expect("name utf8");
            let bad_path = format!("/{name_str}/child");

            let result = lookup(dev.as_ref(), &fs.sb, &mut reader, &bad_path);
            assert!(
                matches!(result, Err(Error::NotADirectory)),
                "got {result:?} for path {bad_path}"
            );
        }
    }
}
