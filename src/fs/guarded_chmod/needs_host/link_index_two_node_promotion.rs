//! Linux, Rust and C splits from one fixture with four occupied inline extents.
use super::*;
use crate::extent::{Extent, ExtentHeader, ExtentIdx};
use crate::extent_mut::{plan_insert_extent, ExtentMutation};

const BLOCK_SIZE: u32 = 1024;
const EVIDENCE: &str = "tmp/link-index-two-node-promotion";

fn prepare_fixture(source: &Path, output: &Path) {
    let original = observe(source);
    let a = &original.view;
    let (ino, parent, raw) = &a.records[0];
    let fs = mount(source);
    assert_eq!(fs.sb.block_size(), BLOCK_SIZE);
    assert_eq!(
        fs.sb.feature_compat & features::Compat::HAS_JOURNAL.bits(),
        0
    );
    assert_eq!((original.depth, a.roots), (1, 2));
    let eh = ExtentHeader::parse(&parent.block).unwrap();
    assert_eq!((eh.depth, eh.entries, eh.max), (0, 1, 4));
    assert!(original.map_nodes.is_empty());
    assert!(a.mapping.len() >= 5);
    assert!(a.mapping.windows(2).all(|p| p[0] + 1 == p[1]));

    // Reverse the final three physical positions, moving their contents
    // along with the mapping. The logical directory bytes and all physical
    // owners stay fixed. The predecessor of the last logical block now
    // occupies its physical successor, so the next append cannot merge.
    let mut mapping = a.mapping.clone();
    let tail = mapping.len() - 3;
    mapping[tail..].reverse();
    let successor = mapping[tail + 2] + 1;
    assert_eq!(successor, mapping[tail + 1]);
    let (group, bit, _) = fs.group_chunks(successor, 1).unwrap()[0];
    assert!(crate::alloc::bit_is_set(
        &fs.read_block(fs.groups[group].block_bitmap).unwrap(),
        bit as u32,
    ));

    let mut root = parent.block.to_vec();
    root[2..4].fill(0);
    root[12..].fill(0);
    for (logical, length) in [(0, tail), (tail, 1), (tail + 1, 1), (tail + 2, 1)] {
        let edits = plan_insert_extent(
            &root,
            Extent {
                logical_block: u32::try_from(logical).unwrap(),
                length: u16::try_from(length).unwrap(),
                physical_block: mapping[logical],
                uninitialized: false,
            },
        )
        .unwrap();
        let [ExtentMutation::WriteRoot { bytes }] = edits.as_slice() else {
            panic!("fixture must fit exactly four inline extents");
        };
        root.clone_from(bytes);
    }
    let header = ExtentHeader::parse(&root).unwrap();
    assert_eq!((header.depth, header.entries, header.max), (0, 4, 4));
    let mut transformed_inode = raw.clone();
    Filesystem::patch_inode_block_area(&mut transformed_inode, &root).unwrap();
    fs.finalize_inode_raw(*ino, parent.generation, &mut transformed_inode)
        .unwrap();

    let original_bytes = std::fs::read(source).unwrap();
    let mut transformed = original_bytes.clone();
    let bs = BLOCK_SIZE as usize;
    let (first, last) = (
        a.mapping[tail] as usize * bs,
        a.mapping[tail + 2] as usize * bs,
    );
    transformed[first..first + bs].copy_from_slice(&original_bytes[last..last + bs]);
    transformed[last..last + bs].copy_from_slice(&original_bytes[first..first + bs]);
    let (table, offset) = crate::bgd::locate_inode(&fs.sb, &fs.groups, *ino).unwrap();
    let inode_at = table as usize * bs + offset as usize;
    transformed[inode_at..inode_at + transformed_inode.len()].copy_from_slice(&transformed_inode);
    std::fs::write(output, transformed).unwrap();
    let after = observe(output);
    let b = &after.view;
    assert_eq!(b.mapping, mapping);
    assert_eq!(b.directory, a.directory);
    assert_eq!(b.entries, a.entries);
    assert_eq!(b.allocation, a.allocation);
    assert_eq!(b.free, a.free);
    assert_eq!(b.records[0].2, transformed_inode);
    assert!(after.map_nodes.is_empty());
    stable(raw, &transformed_inode, &[(40, 60), (124, 2), (130, 2)]);
    std::fs::write(
        output.with_extension("layout.json"),
        format!(
            "{{\"source_sha256\":\"{}\",\"input_sha256\":\"{}\",\"directory_inode\":{ino},\
             \"logical_blocks\":{},\"inline_extents\":4,\"occupied_successor\":{successor},\
             \"swapped_physical\":[{},{}]}}\n",
            fs_ext4_test_support::sha256_hex(&original_bytes),
            sha(output),
            mapping.len(),
            a.mapping[tail],
            a.mapping[tail + 2],
        ),
    )
    .unwrap();
}

fn assert_promotion(before: &Snapshot, after: &Snapshot, base: &Path) {
    assert!(before.map_nodes.is_empty());
    let [(node, bytes)] = after.map_nodes.as_slice() else {
        panic!("split must allocate exactly one external extent-map leaf");
    };
    let parent = &after.view.records[0].1;
    let root = ExtentHeader::parse(&parent.block).unwrap();
    assert_eq!((root.depth, root.entries), (1, 1));
    let child = ExtentIdx::parse(&parent.block[12..]).unwrap();
    assert_eq!((child.logical_block, child.leaf_block), (0, *node));
    let leaf = ExtentHeader::parse(bytes).unwrap();
    assert_eq!((leaf.depth, leaf.entries), (0, 5));
    let data = after.view.mapping[before.view.mapping.len()];
    assert_ne!(data, *node);
    std::fs::write(base.with_extension("promoted-extent.bin"), bytes).unwrap();
    std::fs::write(
        base.with_extension("promotion.json"),
        format!("{{\"directory_leaf\":{data},\"extent_leaf\":{node},\"extent_records\":5}}\n"),
    )
    .unwrap();
}

#[test]
fn linux_compares_two_node_extent_promotion() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(EVIDENCE);
    let run = fs_ext4_test_support::create_unique_temp_dir(&root).unwrap();
    let mut references = Vec::new();
    let mut emissions = Vec::new();
    let manifest = |references: &[String], emissions: &[String]| {
        std::fs::write(
            run.join("matrix.json"),
            format!(
                "{{\"expected_references\":2,\"expected_emissions\":4,\"references\":[{}],\"emissions\":[{}]}}\n",
                references.join(","),
                emissions.join(","),
            ),
        )
        .unwrap();
    };
    manifest(&references, &emissions);
    let prefix = format!("two-node-promotion-source-{}", std::process::id());
    let result = std::panic::catch_unwind(|| node_full::profile_named(BLOCK_SIZE, false, &prefix));
    retain_bootstrap(&prefix, &run);
    if let Err(error) = result {
        std::panic::resume_unwind(error);
    }

    let source = run.join(format!("{prefix}-1024-false.node-grown.img"));
    let source_hash = sha(&source);
    let input = run.join("promotion-input.img");
    prepare_fixture(&source, &input);
    let input_hash = sha(&input);

    for range in 0..2 {
        let reference_base = run.join(format!("reference-1024-false-{range}-true"));
        std::fs::write(
            reference_base.with_extension("source.json"),
            format!(
                "{{\"file\":\"promotion-input.img\",\"sha256\":\"{input_hash}\",\
                 \"block_size\":1024,\"journal\":false,\"root_slot\":{range},\"split\":true}}\n"
            ),
        )
        .unwrap();
        let reference = run_case(&input, &reference_base, range, true);
        assert_eq!(reference.prepared.input_hash, input_hash);
        assert_promotion(
            &reference.prepared.before,
            &reference.linux,
            &reference_base,
        );
        references.push(reference.row.clone());
        manifest(&references, &emissions);
        for capi in [false, true] {
            let api = if capi { "capi" } else { "rust" };
            let base = run.join(format!("{api}-1024-false-{range}-true"));
            let (row, closed) = emissions::emit(&reference, &reference_base, &base, capi, false);
            assert_promotion(&reference.prepared.before, &closed, &base);
            emissions.push(row);
            manifest(&references, &emissions);
        }
        assert_eq!(sha(&input), input_hash);
        assert_eq!(sha(&source), source_hash);
    }
    assert_eq!((references.len(), emissions.len()), (2, 4));
}
