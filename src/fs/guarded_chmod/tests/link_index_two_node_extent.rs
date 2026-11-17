//! Native controls for extent-map promotion during a two-node directory split.
use super::*;
use crate::extent::{Extent, ExtentHeader};
use crate::extent_mut::{plan_insert_extent, ExtentMutation};
use crate::fs::guarded_chmod::two_node_shape::{self as shape, Case, View};

fn mapping(fs: &Filesystem, inode: &Inode) -> Vec<u64> {
    (0..inode.size / u64::from(fs.sb.block_size()))
        .map(|logical| fs.map_inode_logical(inode, logical).unwrap().unwrap())
        .collect()
}

fn full_inline_map(mut indexed: Indexed) -> Indexed {
    let fs = &indexed.fs;
    let ino = indexed.parent.ino;
    let (parent, mut raw) = fs.read_inode_verified(ino).unwrap();
    let map = mapping(fs, &parent);
    assert_eq!(map.len(), 7);
    assert!(map.windows(2).all(|p| p[1] == p[0] + 1));

    // Hold the next physical block in a regular file so an append cannot
    // merge with the final directory extent.
    let spacer = fs.apply_create("/extent-spacer", 0o600).unwrap();
    fs.apply_replace_file_content("/extent-spacer", &vec![b's'; fs.sb.block_size() as usize])
        .unwrap();
    assert_eq!(mapping(fs, &fs.stat_ino(spacer).unwrap()), [map[6] + 1]);

    // Swap the physical locations of logical directory blocks 2 and 3.
    // All seven data blocks keep their owner and all logical bytes stay put,
    // while the physical runs are now [0,1], [3], [2], [4,5,6].
    let mut root = parent.block.to_vec();
    root[2..4].fill(0);
    root[12..].fill(0);
    for (logical, physical, length) in [(0, 0, 2), (2, 3, 1), (3, 2, 1), (4, 4, 3)] {
        let edits = plan_insert_extent(
            &root,
            Extent {
                logical_block: logical,
                physical_block: map[physical],
                length,
                uninitialized: false,
            },
        )
        .unwrap();
        let [ExtentMutation::WriteRoot { bytes }] = edits.as_slice() else {
            panic!("fixture extent must fit inline");
        };
        root.clone_from(bytes);
    }
    let header = ExtentHeader::parse(&root).unwrap();
    assert_eq!((header.depth, header.entries, header.max), (0, 4, 4));
    Filesystem::patch_inode_block_area(&mut raw, &root).unwrap();
    fs.finalize_inode_raw(ino, parent.generation, &mut raw)
        .unwrap();
    let mut buf = BlockBuffer::new(fs.sb.block_size());
    buf.put(map[2], fs.read_block(map[3]).unwrap());
    buf.put(map[3], fs.read_block(map[2]).unwrap());
    fs.buffer_write_inode(&mut buf, ino, &raw).unwrap();
    fs.commit_block_buffer(buf).unwrap();
    indexed.fs.finish().unwrap();
    indexed.fs =
        Filesystem::mount_with_runtime(indexed.dev.clone(), indexed.clock.clone()).unwrap();
    assert_eq!(
        inventory(&indexed.fs, indexed.parent, indexed.target),
        indexed.names
    );
    indexed.clock.calls.store(0, Ordering::SeqCst);
    indexed.clock.now.store(STAMP, Ordering::SeqCst);
    indexed.dev.reset();
    indexed
}

fn promotion(bs: u32) {
    for range in 0..2 {
        for capi in [false, true] {
            let Indexed {
                dev,
                fs,
                parent,
                target,
                clock,
                mut names,
                queries,
                selected,
            } = full_inline_map(indexed(bs, true, None));
            let query = &queries[range];
            let (before, parent_raw) = fs.read_inode_verified(parent.ino).unwrap();
            let source = fs.read_inode_verified(target.ino).unwrap();
            let old_map = mapping(&fs, &before);
            let old_blocks: Vec<_> = old_map.iter().map(|p| fs.read_block(*p).unwrap()).collect();
            let free = counters(&dev);
            let group = group_counters(&fs);
            let mut bitmap = fs.read_block(fs.groups[0].block_bitmap).unwrap();
            let disk = dev.bytes();

            let fs = with_test_mount(fs, |h, fs| {
                let mut attr = unsafe { std::mem::zeroed() };
                invoke(fs, h, target, parent, query, capi, &mut attr).unwrap();
                let (after, raw) = fs.read_inode_verified(parent.ino).unwrap();
                let header = ExtentHeader::parse(&after.block).unwrap();
                assert_eq!((header.depth, header.entries), (1, 1));
                let child = crate::extent::ExtentIdx::parse(&after.block[12..]).unwrap();
                assert_eq!(child.logical_block, 0);
                let node = fs.read_block(child.leaf_block).unwrap();
                assert!(fs
                    .csum
                    .verify_extent_tail(parent.ino, after.generation, &node));
                let leaf_header = ExtentHeader::parse(&node).unwrap();
                assert_eq!((leaf_header.depth, leaf_header.entries), (0, 5));

                let new_map = mapping(fs, &after);
                let new_blocks: Vec<_> =
                    new_map.iter().map(|p| fs.read_block(*p).unwrap()).collect();
                let new_data = shape::check(
                    View {
                        blocks: &old_blocks,
                        mapping: &old_map,
                    },
                    View {
                        blocks: &new_blocks,
                        mapping: &new_map,
                    },
                    Case {
                        root_slot: range,
                        leaf: selected[range],
                        major: hash(fs, query),
                        name: query,
                        split: true,
                    },
                )
                .unwrap()
                .unwrap();
                assert_ne!(new_data, child.leaf_block);
                assert_eq!(after.size, before.size + u64::from(bs));
                assert_eq!(after.blocks, before.blocks + 2 * u64::from(bs / 512));
                assert_eq!(counters(&dev), (free.0 - 2, free.1));
                assert_eq!(group_counters(fs), (group.0 - 2, group.1, group.2));
                let added = [new_data, child.leaf_block];
                for p in added {
                    let (group, bit, count) = fs.group_chunks(p, 1).unwrap()[0];
                    assert_eq!((group, count), (0, 1));
                    assert!(!crate::alloc::bit_is_set(&bitmap, bit as u32));
                    bitmap[(bit / 8) as usize] |= 1 << (bit % 8);
                }
                assert_eq!(fs.read_block(fs.groups[0].block_bitmap).unwrap(), bitmap);
                let version = |bytes: &[u8]| {
                    u64::from(u32::from_le_bytes(bytes[36..40].try_into().unwrap()))
                        | (u64::from(u32::from_le_bytes(bytes[152..156].try_into().unwrap())) << 32)
                };
                assert_eq!(version(&raw), version(&parent_raw) + 1);
                assert_eq!(
                    (after.mtime, after.mtime_nsec, after.ctime, after.ctime_nsec),
                    (STAMP, 0, STAMP, 0)
                );
                source_preserved(fs, target.ino, &source.1, source.0.links_count);
                let current = fs.stat_ino(target.ino).unwrap();
                assert_eq!((current.ctime, current.ctime_nsec), (STAMP, 0));
                if capi {
                    assert_eq!(
                        (
                            attr.inode,
                            attr.generation,
                            attr.link_count,
                            attr.ctime,
                            attr.ctime_nsec
                        ),
                        (
                            target.ino,
                            current.generation,
                            current.links_count,
                            STAMP,
                            0
                        )
                    );
                }
                assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
                protected_disk(
                    fs,
                    &disk,
                    &dev.bytes(),
                    parent,
                    target,
                    &[
                        (4, 4),
                        (12, 8),
                        (28, 4),
                        (36, 4),
                        (40, 60),
                        (108, 4),
                        (116, 2),
                        (124, 2),
                        (130, 2),
                        (132, 8),
                        (152, 4),
                    ],
                    selected[range],
                    Some(range as u32 + 1),
                    &added,
                );
                names.push(query.clone());
                names.sort();
                assert_eq!(inventory(fs, parent, target), names);
            });
            fs.finish().unwrap();
            let reopened = Filesystem::mount(dev).unwrap();
            assert_eq!(inventory(&reopened, parent, target), names);
        }
    }
}

#[test]
fn promotes_inline_extent_map_1k() {
    promotion(1024);
}

#[test]
fn promotes_inline_extent_map_4k() {
    promotion(4096);
}
