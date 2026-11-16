//! Read-only query preparation shared by synthetic controls and guest support.
use crate::error::{Error, Result};
use crate::fs::Filesystem;
use std::collections::HashSet;

pub(super) const MAX_CANDIDATES: usize = 16_384;

#[derive(Debug)]
pub(super) struct Selection {
    pub name: Vec<u8>,
    pub major: u32,
    pub minor: u32,
    pub version: u8,
    pub root_slot: usize,
    pub node: u32,
    pub leaf: u32,
    pub node_physical: u64,
    pub leaf_physical: u64,
    pub split: bool,
    pub boundary: Option<u32>,
}

impl Selection {
    pub fn json(&self) -> String {
        let hex = self
            .name
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let boundary = self
            .boundary
            .map_or_else(|| "null".to_owned(), |h| h.to_string());
        format!(
            "{{\"name_hex\":\"{hex}\",\"major\":{},\"minor\":{},\"version\":{},\"root_slot\":{},\"node\":{},\"leaf\":{},\"node_physical\":{},\"leaf_physical\":{},\"split\":{},\"boundary\":{boundary}}}\n",
            self.major, self.minor, self.version, self.root_slot, self.node, self.leaf,
            self.node_physical, self.leaf_physical, self.split,
        )
    }
}

pub(super) struct Search {
    pub selected: Option<Selection>,
    pub trace: String,
}

struct Leaf {
    range: usize,
    node: u32,
    logical: u32,
    lower: u32,
    upper: u64,
    bytes: Vec<u8>,
}

fn require(condition: bool) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(Error::Unsupported("two-node query preflight shape"))
    }
}

pub(super) fn name(n: usize) -> Vec<u8> {
    assert!(n < MAX_CANDIDATES);
    format!("cap-{n:05}-{}", "n".repeat(245)).into_bytes()
}

/// The caller captures the input first. Exhaustion returns the complete finite
/// trace and never mutates the input or substitutes a seed/operation class.
pub(super) fn select(
    fs: &Filesystem,
    parent: u32,
    source: u32,
    range: usize,
    split: bool,
    candidates: std::ops::Range<usize>,
) -> Result<Search> {
    if range >= 2 || candidates.start >= candidates.end || candidates.end > MAX_CANDIDATES {
        return Err(Error::InvalidArgument("two-node candidate bounds"));
    }
    let inode = fs.read_inode_verified(parent)?.0;
    let bs = fs.sb.block_size();
    require(
        inode.is_dir()
            && inode.has_extents()
            && inode.flags & crate::inode::InodeFlags::INDEX.bits() != 0
            && inode.flags & (0x4000_0000 | 0x800) == 0
            && fs.csum.enabled
            && fs.sb.feature_incompat & crate::features::Incompat::FILETYPE.bits() != 0
            && [1024, 4096].contains(&bs)
            && inode.size.is_multiple_of(u64::from(bs)),
    )?;
    let len = (inode.size / u64::from(bs)) as usize;
    require((5..=1024).contains(&len))?;
    let mut physical = HashSet::new();
    let mut mapping = Vec::new();
    for n in 0..len {
        let p = fs
            .map_inode_logical(&inode, n as u64)?
            .ok_or(Error::OutOfBounds)?;
        require(physical.insert(p))?;
        mapping.push(p);
    }
    let read = |logical: u32| -> Result<Vec<u8>> {
        fs.read_block(*mapping.get(logical as usize).ok_or(Error::OutOfBounds)?)
    };
    let root = read(0)?;
    fs.check_dx_block(parent, &inode, &root, true)?;
    let info = crate::htree::parse_root_info(&root)?;
    let (_, roots) = crate::htree::parse_root_entries(&root)?;
    require(info.indirect_levels == 1 && roots.len() == 2)?;
    require(roots[0].hash == 0 && roots[1].hash > 0 && roots[1].hash & 1 == 0)?;
    let version = crate::hash::effective_version(info.hash_version, fs.sb.unsigned_hash());
    let hash = |bytes: &[u8]| crate::hash::name_hash(bytes, version, &fs.sb.hash_seed);
    let mut roles = HashSet::from([0]);
    let mut names = HashSet::new();
    let mut majors = HashSet::new();
    let mut leaves = Vec::new();
    // Mark both nodes before admitting leaves, so cross-role aliases fail.
    for route in &roots {
        require(roles.insert(route.block))?;
    }
    for (slot, route) in roots.iter().enumerate() {
        let node = read(route.block)?;
        fs.check_dx_block(parent, &inode, &node, false)?;
        let (count, entries) = crate::htree::parse_node_entries(&node)?;
        require(usize::from(count.limit) == (bs as usize - 16) / 8 && !entries.is_empty())?;
        require(entries[0].hash == 0 && entries.windows(2).all(|w| w[0].hash < w[1].hash))?;
        let top = roots.get(slot + 1).map_or(1 << 32, |r| u64::from(r.hash));
        for (n, entry) in entries.iter().enumerate() {
            let lower = if n == 0 { route.hash } else { entry.hash };
            let upper = entries.get(n + 1).map_or(top, |r| u64::from(r.hash));
            require(lower & 1 == 0 && u64::from(lower) < upper && upper <= top)?;
            require(roles.insert(entry.block))?;
            let bytes = read(entry.block)?;
            let records =
                crate::dir::parse_block_verified(&bytes, true, parent, inode.generation, &fs.csum)?;
            require(!records.is_empty())?;
            for record in records {
                let major = hash(&record.name).major;
                require(
                    record.inode == source
                        && record.file_type == crate::dir::DirEntryType::RegFile
                        && major >= lower
                        && u64::from(major) < upper
                        && names.insert(record.name),
                )?;
                majors.insert(major);
            }
            leaves.push(Leaf {
                range: slot,
                node: route.block,
                logical: entry.block,
                lower,
                upper,
                bytes,
            });
        }
    }
    require(roles.len() == len && names.len() < MAX_CANDIDATES)?;
    let mut trace = format!(
        "{{\"event\":\"preflight\",\"start\":{},\"end\":{},\"root_slot\":{range},\"split\":{split},\"version\":{},\"seed\":{:?},\"names\":{},\"blocks\":{len}}}\n",
        candidates.start, candidates.end, version as u8, fs.sb.hash_seed, names.len()
    );
    for n in candidates {
        let query = name(n);
        let value = hash(&query);
        let leaf = leaves
            .iter()
            .find(|l| value.major >= l.lower && u64::from(value.major) < l.upper)
            .ok_or(Error::Corrupt("two-node hash interval gap"))?;
        let mut reason = "wrong-root";
        let mut boundary = None;
        if names.contains(&query) {
            reason = "existing-name";
        } else if majors.contains(&value.major) {
            reason = "equal-major";
        } else if leaf.range == range {
            let mut trial = leaf.bytes.clone();
            let add = |block: &mut [u8]| {
                crate::dir::add_entry_to_block(
                    block,
                    source,
                    &query,
                    crate::dir::DirEntryType::RegFile,
                    true,
                    12,
                )
            };
            match add(&mut trial) {
                Ok(()) if !split => reason = "selected",
                Ok(()) => reason = "has-room",
                Err(Error::OutOfBounds) if !split => reason = "full-leaf",
                Err(Error::OutOfBounds) => {
                    reason = "unsplittable";
                    if let Ok(mut plan) = crate::htree_mut::plan_leaf_split_with_encoding(
                        &leaf.bytes,
                        version,
                        &fs.sb.hash_seed,
                        true,
                        bs as usize,
                        12,
                        None,
                    ) {
                        let bound = plan.split_out_hash;
                        let node = read(leaf.node)?;
                        if bound & 1 == 0
                            && bound > leaf.lower
                            && u64::from(bound) < leaf.upper
                            && crate::htree_mut::plan_insert_dx_entry_node(&node, bound, len as u32)
                                .is_ok()
                        {
                            let into = if value.major >= bound {
                                &mut plan.right_bytes
                            } else {
                                &mut plan.left_bytes
                            };
                            if add(into).is_ok() {
                                boundary = Some(bound);
                                reason = "selected";
                            }
                        }
                    }
                }
                Err(error) => return Err(error),
            }
        }
        trace.push_str(&format!(
            "{{\"candidate\":{n},\"major\":{},\"minor\":{},\"root_slot\":{},\"node\":{},\"leaf\":{},\"reason\":\"{reason}\"}}\n",
            value.major, value.minor, leaf.range, leaf.node, leaf.logical
        ));
        if reason == "selected" {
            return Ok(Search {
                selected: Some(Selection {
                    name: query,
                    major: value.major,
                    minor: value.minor,
                    version: version as u8,
                    root_slot: range,
                    node: leaf.node,
                    leaf: leaf.logical,
                    node_physical: mapping[leaf.node as usize],
                    leaf_physical: mapping[leaf.logical as usize],
                    split,
                    boundary,
                }),
                trace,
            });
        }
    }
    trace.push_str("{\"event\":\"exhausted\"}\n");
    Ok(Search {
        selected: None,
        trace,
    })
}
