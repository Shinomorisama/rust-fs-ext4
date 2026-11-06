//! Linux-only first full-intermediate-node growth; no writable driver mount.
use super::*;

fn flattened(s: &Snapshot) -> Vec<(u32, u32)> {
    let (_, roots) = crate::htree::parse_root_entries(&s.view.directory[0]).unwrap();
    let mut routes = Vec::new();
    for root in roots {
        let (_, entries) =
            crate::htree::parse_node_entries(&s.view.directory[root.block as usize]).unwrap();
        for (n, entry) in entries.into_iter().enumerate() {
            routes.push((if n == 0 { root.hash } else { entry.hash }, entry.block));
        }
    }
    assert!(routes.windows(2).all(|w| w[0].0 < w[1].0));
    routes
}

pub(super) fn shape(before: &Snapshot, after: &Snapshot, image: &Path) {
    let (a, b) = (&before.view, &after.view);
    let fs = mount(image);
    let bs = fs.sb.block_size() as usize;
    assert_eq!((before.depth, after.depth), (1, 1));
    assert_eq!((a.roots, b.roots), (1, 2));
    assert_eq!(before.index_nodes.len(), 1);
    assert_eq!(after.index_nodes.len(), 2);
    assert_eq!(before.index_nodes[0], after.index_nodes[0]);
    let old_node = before.index_nodes[0] as usize;
    let (count, _) = crate::htree::parse_node_entries(&a.directory[old_node]).unwrap();
    assert_eq!(count.count, count.limit);
    assert_eq!(usize::from(count.limit), (bs - 16) / 8);
    for logical in &after.index_nodes {
        let (new, _) = crate::htree::parse_node_entries(&b.directory[*logical as usize]).unwrap();
        assert_eq!(new.limit, count.limit);
        assert!(new.count < new.limit);
    }
    let old = flattened(before);
    let new = flattened(after);
    assert_eq!(new.len(), old.len() + 1);
    assert!(old.iter().all(|r| new.binary_search(r).is_ok()));
    assert_eq!(
        after
            .index_nodes
            .iter()
            .filter(|n| !before.index_nodes.contains(n))
            .count(),
        1
    );
    stable(
        &a.directory[0],
        &b.directory[0],
        &[(34, 2), (40, 8), (bs - 4, 4)],
    );
    assert_eq!(&a.directory[old_node][..10], &b.directory[old_node][..10]);
    let name = format!("cap-{:05}-{}", a.entries.len(), "n".repeat(245)).into_bytes();
    let routed = crate::htree::lookup_leaf_with(
        &name,
        &a.directory[0],
        &fs.sb.hash_seed,
        fs.sb.unsigned_hash(),
        |n| Ok(a.directory[n as usize].clone()),
    )
    .unwrap()
    .unwrap();
    let changed = before
        .leaves
        .iter()
        .filter(|l| a.directory[**l as usize] != b.directory[**l as usize])
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(changed, vec![routed]);
    let mut leaf = a.directory[routed as usize].clone();
    assert!(matches!(
        crate::dir::add_entry_to_block(
            &mut leaf,
            a.records[1].0,
            &name,
            crate::dir::DirEntryType::RegFile,
            true,
            12
        ),
        Err(crate::error::Error::OutOfBounds)
    ));
    assert!(b.entries.binary_search(&(name, a.records[1].0)).is_ok());
}

fn profile(bs: u32, journal: bool) {
    intermediate::prepare_named(bs, journal, "link-node-full-source");
    let base =
        fs_ext4_test_support::temp_dir().join(format!("link-node-full-source-{bs}-{journal}"));
    let image = base.with_extension("node-working.img");
    let prior = base.with_extension("node-prior.img");
    std::fs::copy(base.with_extension("linux-split.img"), &image).unwrap();
    let initial = observe(&image);
    assert_eq!((initial.depth, initial.view.roots), (1, 1));
    assert_eq!(initial.leaves.len(), ((bs - 16) / 8) as usize);
    let mut total = initial.view.entries.len();
    let mut witness = None;
    for n in 0..128 {
        assert!(total < MAX_NAMES);
        std::fs::copy(&image, &prior).unwrap();
        guest(
            &image,
            &base,
            &format!("full-node-seek-{n}"),
            "fill",
            total,
            1,
            "",
        );
        total += 1;
        let state = probe(&image, &base, &format!("full-node-seek-{n}"));
        assert_eq!(state.depth, 1);
        assert_eq!(state.total, total);
        if state.count != 1 {
            assert_eq!(state.count, 2);
            let before = capture(&prior, &base, "node-full");
            let grown = capture(&image, &base, "node-grown");
            transition(&before, &grown, &image);
            assert_eq!(grown.view.entries.len(), total);
            witness = Some(grown);
            break;
        }
    }
    let grown = witness.expect("bounded Linux run did not grow the full intermediate node");
    let cold = base.with_extension("node-cold.img");
    std::fs::copy(&image, &cold).unwrap();
    let expected = format!(
        "EXPECTED = [{}]\nFLAGS = {:?}\nFREE = ({}, {})\n",
        TRACKED
            .iter()
            .zip(&grown.view.records)
            .map(|(p, (ino, i, _))| literal(p, *ino, i))
            .collect::<Vec<_>>()
            .join(","),
        grown
            .view
            .records
            .iter()
            .map(|r| r.1.flags)
            .collect::<Vec<_>>(),
        grown.view.free.0,
        grown.view.free.1,
    );
    std::fs::write(base.with_extension("node-cold.expected.py"), &expected).unwrap();
    guest(&cold, &base, "node-cold-guest", "cold", total, 0, &expected);
    let observed = capture(&cold, &base, "node-cold-readback");
    assert_eq!(grown.view.directory, observed.view.directory);
    assert_eq!(grown.view.tables, observed.view.tables);
    assert_eq!(grown.view.allocation, observed.view.allocation);
    assert_eq!(grown.view.eas, observed.view.eas);
    assert_eq!(grown.view.payloads, observed.view.payloads);
    assert_eq!(grown.view.root_directory, observed.view.root_directory);
    assert_eq!(grown.map_nodes, observed.map_nodes);
    assert_eq!(grown.view.entries, observed.view.entries);
    assert_eq!(grown.view.free, observed.view.free);
    let child_counts = grown
        .index_nodes
        .iter()
        .map(|n| {
            crate::htree::parse_node_entries(&grown.view.directory[*n as usize])
                .unwrap()
                .0
                .count
        })
        .collect::<Vec<_>>();
    std::fs::write(base.with_extension("node-observation.json"),format!("{{\"block_size\":{bs},\"journal\":{journal},\"before\":{},\"after\":{total},\"node_limit\":{},\"child_counts\":{:?},\"depth\":1,\"root_count\":2}}\n",total-1,(bs-16)/8,child_counts)).unwrap();
    eprintln!("Linux full-node growth bs={bs} journal={journal}: {total} aliases, two root routes, child counts {child_counts:?}; complete routes/allocation/protected bytes and cold metadata checked");
    for p in [image, prior, cold] {
        std::fs::remove_file(p).unwrap();
    }
}

#[test]
fn linux_records_first_full_node_growth() {
    for bs in [1024, 4096] {
        for journal in [false, true] {
            profile(bs, journal);
        }
    }
}
