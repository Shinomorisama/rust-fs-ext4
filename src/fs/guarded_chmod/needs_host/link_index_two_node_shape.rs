//! Shared test-only geometry delta; callers verify checksums, names and metadata.
use crate::htree::{parse_node_entries, parse_root_entries, parse_root_info, DxEntry};
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub(super) struct View<'a> {
    pub blocks: &'a [Vec<u8>],
    pub mapping: &'a [u64],
}

#[derive(Clone, Copy)]
pub(super) struct Case<'a> {
    pub root_slot: usize,
    pub leaf: u32,
    pub major: u32,
    pub name: &'a [u8],
    pub split: bool,
}

type Result<T> = std::result::Result<T, &'static str>;

fn require(value: bool) -> Result<()> {
    if value {
        Ok(())
    } else {
        Err("two-node geometry delta")
    }
}

fn roots(view: View<'_>) -> Result<Vec<DxEntry>> {
    let root = view.blocks.first().ok_or("missing root")?;
    let bs = root.len();
    require([1024, 4096].contains(&bs) && (5..=1024).contains(&view.blocks.len()))?;
    require(view.blocks.len() == view.mapping.len() && view.blocks.iter().all(|b| b.len() == bs))?;
    require(view.mapping.iter().copied().collect::<HashSet<_>>().len() == view.mapping.len())?;
    let info = parse_root_info(root).map_err(|_| "bad two-node root")?;
    let (header, routes) = parse_root_entries(root).map_err(|_| "bad two-node routes")?;
    require(info.indirect_levels == 1 && info.info_length == 8 && info.unused_flags == 0)?;
    require(routes.len() == 2 && usize::from(header.limit) == (bs - 40) / 8)?;
    require(routes[0].hash == 0 && routes[1].hash > 0 && routes[1].hash & 1 == 0)?;
    let mut seen = HashSet::from([0]);
    for r in &routes {
        require(r.block != 0 && (r.block as usize) < view.blocks.len() && seen.insert(r.block))?;
    }
    for (slot, r) in routes.iter().enumerate() {
        let node = &view.blocks[r.block as usize];
        let (head, entries) = parse_node_entries(node).map_err(|_| "bad two-node child")?;
        require(usize::from(head.limit) == (bs - 16) / 8 && !entries.is_empty())?;
        require(entries.windows(2).all(|w| w[0].hash < w[1].hash))?;
        let top = routes
            .get(slot + 1)
            .map_or(1u64 << 32, |r| u64::from(r.hash));
        for (n, e) in entries.iter().enumerate() {
            let lower = if n == 0 { r.hash } else { e.hash };
            let upper = entries.get(n + 1).map_or(top, |e| u64::from(e.hash));
            require(lower & 1 == 0 && u64::from(lower) < upper && upper <= top)?;
            require((e.block as usize) < view.blocks.len() && seen.insert(e.block))?;
        }
    }
    require(seen.len() == view.blocks.len())?;
    Ok(routes)
}

pub(super) fn check(a: View<'_>, b: View<'_>, case: Case<'_>) -> Result<Option<u64>> {
    let before = roots(a)?;
    let after = roots(b)?;
    require(case.root_slot < 2 && case.name.len() == 255 && case.major & 1 == 0)?;
    require(a.blocks[0] == b.blocks[0])?;
    require(
        before
            .iter()
            .zip(&after)
            .all(|(a, b)| a.hash == b.hash && a.block == b.block),
    )?;
    require(b.mapping.len() == a.mapping.len() + usize::from(case.split))?;
    require(b.mapping[..a.mapping.len()] == *a.mapping)?;
    let selected = before[case.root_slot].block;
    require(crate::htree::find_entry_for_hash(&before, case.major).block == selected)?;
    let (old, old_routes) =
        parse_node_entries(&a.blocks[selected as usize]).map_err(|_| "bad old child")?;
    let (new, new_routes) =
        parse_node_entries(&b.blocks[selected as usize]).map_err(|_| "bad new child")?;
    require(crate::htree::find_entry_for_hash(&old_routes, case.major).block == case.leaf)?;
    require(case.leaf != selected && (case.leaf as usize) < a.blocks.len())?;
    require(new.limit == old.limit && new.count == old.count + u16::from(case.split))?;
    let mut trial = a.blocks[case.leaf as usize].clone();
    let added = crate::dir::add_entry_to_block(
        &mut trial,
        1,
        case.name,
        crate::dir::DirEntryType::RegFile,
        true,
        12,
    );
    require(if case.split {
        matches!(added, Err(crate::error::Error::OutOfBounds))
    } else {
        added.is_ok()
    })?;
    for (logical, block) in a.blocks.iter().enumerate() {
        if logical != case.leaf as usize && !(case.split && logical == selected as usize) {
            require(*block == b.blocks[logical])?;
        }
    }
    require(a.blocks[case.leaf as usize] != b.blocks[case.leaf as usize])?;
    if !case.split {
        require(a.blocks[selected as usize] == b.blocks[selected as usize])?;
        return Ok(None);
    }
    require(old.count < old.limit)?;
    require(a.blocks[selected as usize][..10] == b.blocks[selected as usize][..10])?;
    let logical = a.blocks.len() as u32;
    let additions: Vec<_> = new_routes.iter().filter(|r| r.block == logical).collect();
    require(additions.len() == 1)?;
    require(
        new_routes
            .iter()
            .filter(|r| r.block != logical)
            .map(|r| (r.hash, r.block))
            .eq(old_routes.iter().map(|r| (r.hash, r.block))),
    )?;
    let slot = old_routes
        .iter()
        .position(|r| r.block == case.leaf)
        .ok_or("selected leaf missing")?;
    let lower = if slot == 0 {
        before[case.root_slot].hash
    } else {
        old_routes[slot].hash
    };
    let top = before
        .get(case.root_slot + 1)
        .map_or(1u64 << 32, |r| u64::from(r.hash));
    let upper = old_routes.get(slot + 1).map_or(top, |r| u64::from(r.hash));
    require(
        additions[0].hash & 1 == 0
            && additions[0].hash > lower
            && u64::from(additions[0].hash) < upper,
    )?;
    Ok(b.mapping.last().copied())
}
