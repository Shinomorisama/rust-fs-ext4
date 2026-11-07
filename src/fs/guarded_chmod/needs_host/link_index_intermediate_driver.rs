//! Exact Linux predecessor for the last free intermediate route slot.
use super::*;
use std::path::PathBuf;

fn node_size(image: &Path) -> (u64, u16, u16) {
    let fs = mount(image);
    let ino = fs.resolve(b"/policy").unwrap();
    let parent = fs.stat_ino(ino).unwrap();
    let root = fs
        .read_block(fs.map_inode_logical(&parent, 0).unwrap().unwrap())
        .unwrap();
    fs.check_dx_block(ino, &parent, &root, true).unwrap();
    assert_eq!(
        crate::htree::parse_root_info(&root)
            .unwrap()
            .indirect_levels,
        1
    );
    let (count, routes) = crate::htree::parse_root_entries(&root).unwrap();
    assert_eq!(count.count, 1);
    let node = fs
        .read_block(
            fs.map_inode_logical(&parent, u64::from(routes[0].block))
                .unwrap()
                .unwrap(),
        )
        .unwrap();
    fs.check_dx_block(ino, &parent, &node, false).unwrap();
    let (count, _) = crate::htree::parse_node_entries(&node).unwrap();
    (parent.size, count.count, count.limit)
}

pub(super) fn split(before: &Snapshot, after: &Snapshot, name: &[u8], fs: &Filesystem) {
    let (a, b) = (&before.view, &after.view);
    assert_eq!((before.depth, after.depth), (1, 1));
    assert_eq!((a.roots, b.roots), (1, 1));
    assert_eq!(a.records[0].1.flags, b.records[0].1.flags);
    assert_eq!(a.directory[0], b.directory[0]);
    assert_eq!(before.index_nodes, after.index_nodes);
    assert_eq!(before.index_nodes.len(), 1);
    let n = before.index_nodes[0] as usize;
    let (old, routes) = crate::htree::parse_node_entries(&a.directory[n]).unwrap();
    let (new, grown) = crate::htree::parse_node_entries(&b.directory[n]).unwrap();
    assert_eq!(old.count + 1, old.limit);
    assert_eq!((new.count, new.limit), (old.limit, old.limit));
    assert_eq!(&a.directory[n][..10], &b.directory[n][..10]);
    assert!(routes
        .iter()
        .all(|r| grown.iter().any(|g| g.hash == r.hash && g.block == r.block)));
    assert_eq!(
        grown
            .iter()
            .filter(|r| r.block as usize == a.mapping.len())
            .count(),
        1
    );
    assert_eq!(after.leaves.len(), before.leaves.len() + 1);
    assert!(before.leaves.iter().all(|n| after.leaves.contains(n)));
    // The common observer already checks every old/new name's actual hash route.
    // Require that only the selected old leaf changed, and that insertion really
    // needed a split rather than merely a free record in that leaf.
    let routed = crate::htree::lookup_leaf_with(
        name,
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
    let mut selected = a.directory[changed[0] as usize].clone();
    assert!(matches!(
        crate::dir::add_entry_to_block(
            &mut selected,
            a.records[1].0,
            name,
            crate::dir::DirEntryType::RegFile,
            true,
            12
        ),
        Err(crate::error::Error::OutOfBounds)
    ));
}

fn prepare(bs: u32, journal: bool) -> (PathBuf, Snapshot) {
    prepare_named(bs, journal, "link-intermediate-source")
}

pub(super) fn prepare_named(bs: u32, journal: bool, prefix: &str) -> (PathBuf, Snapshot) {
    profile_named(bs, journal, prefix);
    let base = fs_ext4_test_support::temp_dir().join(format!("{prefix}-{bs}-{journal}"));
    let image = base.with_extension("seek.img");
    let prior = base.with_extension("prior.img");
    std::fs::copy(base.with_extension("grown.img"), &image).unwrap();
    let mut total = observe(&image).view.entries.len();
    let mut state = node_size(&image);
    assert_eq!(state.1 + 2, state.2);
    // Every alias has a 255-byte name and needs a 264-byte aligned record.
    // Before the node fills, at most limit - 1 leaves can hold those records.
    // One more alias than their combined capacity must reach the full node,
    // regardless of the Linux-created hash seed or distribution among leaves.
    // A fixed 64-call search failed on a valid 4 KiB journaled predecessor.
    let capacity = (usize::from(state.2) - 1) * ((bs as usize - 12) / 264);
    assert!(total <= capacity && capacity < MAX_NAMES);
    let seeks = capacity + 1 - total;
    for n in 0..seeks {
        assert!(total < MAX_NAMES);
        std::fs::copy(&image, &prior).unwrap();
        guest(
            &image,
            &base,
            &format!("node-seek-{n}"),
            "fill",
            total,
            1,
            "",
        );
        total += 1;
        let next = node_size(&image);
        assert_eq!(next.2, state.2);
        if next.0 != state.0 {
            assert_eq!(next.0, state.0 + u64::from(bs));
            assert_eq!(next.1, state.1 + 1);
            if next.1 == next.2 {
                let before = capture(&prior, &base, "split-input");
                let linux = capture(&image, &base, "linux-split");
                assert_eq!(before.view.entries.len() + 1, total);
                let name = format!("cap-{:05}-{}", total - 1, "n".repeat(245));
                let fs = mount(&image);
                split(&before, &linux, name.as_bytes(), &fs);
                drop(fs);
                for p in [image, prior] {
                    std::fs::remove_file(p).unwrap();
                }
                return (base.with_extension("split-input.img"), linux);
            }
        } else {
            assert_eq!(next.1, state.1);
        }
        state = next;
    }
    panic!(
        "capacity-bounded Linux search did not fill the last intermediate slot: \
         bs={bs} journal={journal} total={total} state={state:?} seeks={seeks}"
    );
}

#[test]
fn linux_checks_public_intermediate_split() {
    for bs in [1024, 4096] {
        for journal in [false, true] {
            let (source, linux) = prepare(bs, journal);
            for capi in [false, true] {
                driver_emission::emit(
                    &source,
                    &linux,
                    capi,
                    bs,
                    journal,
                    driver_emission::Mode::RetainedSplit,
                );
            }
        }
    }
}
