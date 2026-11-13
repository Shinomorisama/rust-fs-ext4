//! Private ordinary-directory oracle contract, not wired into the live oracle yet.
//!
//! The caller must supply independently checked, checksum-clean directory bytes
//! and name-bound independent hash answers. The admitted profile is ordinary,
//! 1 KiB, FILETYPE, metadata_csum, root-only. This validates routing,
//! continuation and record structure; it is not checksum, ownership or write
//! admission. Format: docs.kernel.org/filesystems/ext4/directory.html#hash-tree-directories.

#[derive(Clone, Copy, Debug)]
pub struct Profile {
    pub block_size: usize,
    pub filetype: bool,
    pub metadata_csum: bool,
    pub unsigned_hash: bool,
    pub casefold: bool,
    pub encrypted: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Query<'a> {
    pub name: &'a [u8],
    pub computed: (u32, u32),
    pub independent: (u32, u32),
    pub independent_name: &'a [u8],
    pub effective_version: u8,
    pub selected: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Unsupported,
    Bounds,
    Root,
    Routes,
    Leaf,
    Hash,
    Selection,
    Missing,
    Duplicate,
    Misrouted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Admission {
    pub inode: u32,
    pub actual_leaf: u32,
    pub candidate: u32,
    pub permitted_leaves: Vec<u32>,
}

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap())
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

pub fn validate(bytes: &[u8], profile: Profile, query: Query<'_>) -> Result<Admission, Refusal> {
    let bs = profile.block_size;
    if bs != 1024
        || !profile.filetype
        || !profile.metadata_csum
        || profile.casefold
        || profile.encrypted
    {
        return Err(Refusal::Unsupported);
    }
    // The largest admitted root and all its leaves fit this explicit bound.
    if bytes.len() < 2 * bs || bytes.len() > 2 * 1024 * 1024 || !bytes.len().is_multiple_of(bs) {
        return Err(Refusal::Bounds);
    }
    if query.name.is_empty()
        || query.name.len() > 255
        || query.name.contains(&0)
        || query.name.contains(&b'/')
        || matches!(query.name, b"." | b"..")
    {
        return Err(Refusal::Unsupported);
    }
    if query.name != query.independent_name
        || query.computed != query.independent
        || query.independent.0 & 1 != 0
    {
        return Err(Refusal::Hash);
    }
    let root = &bytes[..bs];
    if root[30] != 0 || root[28] > 5 {
        return Err(Refusal::Unsupported);
    }
    let version = if profile.unsigned_hash && root[28] < 3 {
        root[28] + 3
    } else {
        root[28]
    };
    if query.effective_version != version {
        return Err(Refusal::Hash);
    }
    if u32_at(root, 0) == 0
        || u16_at(root, 4) != 12
        || root[6..9] != [1, 2, b'.']
        || u32_at(root, 12) == 0
        || u16_at(root, 16) as usize != bs - 12
        || root[18..22] != [2, 2, b'.', b'.']
        || u32_at(root, 24) != 0
        || root[29] != 8
        || root[31] != 0
    {
        return Err(Refusal::Root);
    }
    let capacity = (bs - 40) / 8;
    let limit = u16_at(root, 32) as usize;
    let count = u16_at(root, 34) as usize;
    if limit != capacity || count == 0 || count > limit || count != bytes.len() / bs - 1 {
        return Err(Refusal::Routes);
    }
    if u32_at(root, 32 + limit * 8) != 0 {
        return Err(Refusal::Root);
    }
    let mut routes = Vec::with_capacity(count);
    let mut seen = std::collections::HashSet::new();
    for slot in 0..count {
        let hash = if slot == 0 {
            0
        } else {
            u32_at(root, 32 + slot * 8)
        };
        let leaf = u32_at(root, 36 + slot * 8);
        if leaf == 0 || leaf as usize >= bytes.len() / bs || !seen.insert(leaf) {
            return Err(Refusal::Routes);
        }
        if let Some(&(previous, _)) = routes.last() {
            if hash < previous || (hash == previous && hash & 1 == 0) {
                return Err(Refusal::Routes);
            }
        }
        routes.push((hash, leaf));
    }
    let major = query.independent.0;
    let start = routes.iter().rposition(|&(hash, _)| hash <= major).unwrap();
    let candidate = routes[start].1;
    if query.selected != candidate {
        return Err(Refusal::Selection);
    }
    let mut permitted = vec![candidate];
    for &(hash, leaf) in &routes[start + 1..] {
        if hash & 1 != 0 && hash & !1 == major {
            permitted.push(leaf);
        } else {
            break;
        }
    }
    let mut found = None;
    for &(_, leaf) in &routes {
        let block = &bytes[leaf as usize * bs..(leaf as usize + 1) * bs];
        if block[bs - 12..bs - 4] != [0, 0, 0, 0, 12, 0, 0, 0xde] {
            return Err(Refusal::Leaf);
        }
        let end = bs - 12;
        let mut offset = 0;
        while offset < end {
            if end - offset < 8 {
                return Err(Refusal::Leaf);
            }
            let inode = u32_at(block, offset);
            let length = u16_at(block, offset + 4) as usize;
            let name_length = block[offset + 6] as usize;
            if length < 8
                || !length.is_multiple_of(4)
                || length > end - offset
                || name_length > length - 8
            {
                return Err(Refusal::Leaf);
            }
            let name = &block[offset + 8..offset + 8 + name_length];
            if inode != 0 {
                if name.is_empty()
                    || name.contains(&0)
                    || name.contains(&b'/')
                    || matches!(name, b"." | b"..")
                    || block[offset + 7] > 7
                {
                    return Err(Refusal::Leaf);
                }
                if name == query.name && found.replace((leaf, inode)).is_some() {
                    return Err(Refusal::Duplicate);
                }
            }
            offset += length;
        }
    }
    let (actual_leaf, inode) = found.ok_or(Refusal::Missing)?;
    if !permitted.contains(&actual_leaf) {
        return Err(Refusal::Misrouted);
    }
    Ok(Admission {
        inode,
        actual_leaf,
        candidate,
        permitted_leaves: permitted,
    })
}
