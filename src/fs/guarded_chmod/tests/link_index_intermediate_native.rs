//! Distinct intermediate routes; synthetic controls, not Linux qualification.
use super::*;

struct Indexed {
    dev: Arc<Device>,
    fs: Filesystem,
    parent: InodeRef,
    target: InodeRef,
    clock: Arc<Clock>,
    names: Vec<Vec<u8>>,
    query: Vec<u8>,
}

fn indexed(bs: u32, full: bool) -> Indexed {
    let (dev, fs, parent, target, clock) = fixture(bs);
    let inode = fs.stat_ino(parent.ino).unwrap();
    let root_phys = fs.map_inode_logical(&inode, 0).unwrap().unwrap();
    let limit = ((bs - 16) / 8) as usize;
    let count = limit - usize::from(!full);
    let per_leaf = ((bs - 12) / 264) as usize;
    let seed = fs.sb.hash_seed;
    let hash = |n: &[u8]| crate::hash::name_hash(n, crate::hash::HashVersion::HalfMd4, &seed).major;
    let mut pool: Vec<_> = (0..count * per_leaf + 1).map(long).collect();
    pool.sort_by_key(|n| hash(n));
    assert!(pool.windows(2).all(|w| hash(&w[0]) < hash(&w[1])));
    let query = pool[per_leaf].clone();
    let mut root = fs.read_block(root_phys).unwrap();
    root[30] = 1;
    let mut node = vec![0; bs as usize];
    node[4..6].copy_from_slice(&(bs as u16).to_le_bytes());
    node[8..10].copy_from_slice(&(limit as u16).to_le_bytes());
    node[10..12].copy_from_slice(&(count as u16).to_le_bytes());
    let mut buf = BlockBuffer::new(bs);
    let mut names = Vec::new();
    for n in 0..count {
        let entries = if n == 0 {
            pool[..per_leaf].to_vec()
        } else {
            vec![pool[n * per_leaf + 1].clone()]
        };
        let mut bytes = leaf(bs, target.ino, &entries);
        assert!(fs
            .csum
            .patch_dir_entry_tail(parent.ino, inode.generation, &mut bytes));
        fs.buffer_append_dir_block(&mut buf, parent.ino, bytes)
            .unwrap();
        let at = 8 + n * 8;
        if n != 0 {
            node[at..at + 4].copy_from_slice(&hash(&entries[0]).to_le_bytes());
        }
        node[at + 4..at + 8].copy_from_slice(&((n + 2) as u32).to_le_bytes());
        names.extend(entries);
    }
    assert!(fs
        .csum
        .patch_dx_tail(parent.ino, inode.generation, &mut root, 32));
    buf.put(root_phys, root);
    assert!(fs
        .csum
        .patch_dx_tail(parent.ino, inode.generation, &mut node, 8));
    buf.put(fs.map_inode_logical(&inode, 1).unwrap().unwrap(), node);
    let (source, mut raw) = fs.read_inode_verified(target.ino).unwrap();
    fs.patch_inode_nlink(target.ino, &mut raw, &source, names.len() as i32)
        .unwrap();
    fs.buffer_write_inode(&mut buf, target.ino, &raw).unwrap();
    fs.commit_block_buffer(buf).unwrap();
    drop(fs);
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
    let inode = fs.stat_ino(parent.ino).unwrap();
    assert_eq!(inode.size, (count + 2) as u64 * u64::from(bs));
    let node = fs
        .read_block(fs.map_inode_logical(&inode, 1).unwrap().unwrap())
        .unwrap();
    let (capacity, routes) = crate::htree::parse_node_entries(&node).unwrap();
    assert_eq!(usize::from(capacity.count), count);
    assert_eq!(usize::from(capacity.limit), limit);
    assert_eq!(
        routes
            .iter()
            .map(|r| r.block)
            .collect::<std::collections::HashSet<_>>()
            .len(),
        count
    );
    assert_eq!(
        crate::htree::find_entry_for_hash(&routes, hash(&query)).block,
        2
    );
    for name in &names {
        assert_eq!(fs.lookup_at(parent.ino, name).unwrap(), target.ino);
    }
    assert_eq!(
        fs.stat_ino(target.ino).unwrap().links_count as usize,
        names.len() + 1
    );
    dev.reset();
    clock.calls.store(0, Ordering::SeqCst);
    Indexed {
        dev,
        fs,
        parent,
        target,
        clock,
        names,
        query,
    }
}

fn decline(bs: u32) {
    let Indexed {
        dev,
        fs,
        parent,
        target,
        clock,
        query,
        ..
    } = indexed(bs, true);
    let mut buf = BlockBuffer::new(bs);
    buf.uninit_cleared.insert(0, 0);
    let dirty = buf.dirty.clone();
    let pending = buf.uninit_cleared.clone();
    let bytes = dev.bytes();
    let status = fs.chmod_operation().unwrap().operation.status();
    let marker = fs.marked_not_clean.load(Ordering::SeqCst);
    assert!(!fs
        .buffer_split_htree_leaf_and_add_entry(
            &mut buf,
            parent.ino,
            &query,
            target.ino,
            crate::dir::DirEntryType::RegFile,
            true
        )
        .unwrap());
    assert_eq!(buf.dirty, dirty);
    assert_eq!(buf.uninit_cleared, pending);
    assert_eq!(dev.bytes(), bytes);
    assert_eq!(dev.counts()[1..3], [0, 0]);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
    assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), marker);
    assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
}

fn positive(bs: u32) {
    for capi in [false, true] {
        let Indexed {
            dev,
            fs,
            parent,
            target,
            clock,
            mut names,
            query,
        } = indexed(bs, false);
        let before = fs.stat_ino(parent.ino).unwrap();
        let mapping: Vec<_> = (0..before.size / u64::from(bs))
            .map(|n| fs.map_inode_logical(&before, n).unwrap().unwrap())
            .collect();
        let free = counters(&dev);
        let root = fs.read_block(mapping[0]).unwrap();
        let peer = fs
            .read_inode_verified(fs.resolve(b"/peer").unwrap())
            .unwrap()
            .1;
        let fs = with_test_mount(fs, |h, fs| {
            if capi {
                assert_eq!(
                    unsafe {
                        fs_ext4_link_at(
                            h,
                            target.ino,
                            target.generation.unwrap(),
                            parent.ino,
                            parent.generation.unwrap(),
                            query.as_ptr().cast(),
                            query.len(),
                            std::ptr::null_mut(),
                        )
                    },
                    0
                );
            } else {
                fs.apply_link_at(target, parent, &query).unwrap();
            }
            let now = fs.stat_ino(parent.ino).unwrap();
            assert_eq!(now.flags, before.flags);
            assert_eq!(fs.read_block(mapping[0]).unwrap(), root);
            let node = fs.read_block(mapping[1]).unwrap();
            let (count, _) = crate::htree::parse_node_entries(&node).unwrap();
            assert_eq!(count.count, count.limit);
            assert_eq!(now.size, before.size + u64::from(bs));
            assert_eq!(now.blocks, before.blocks + u64::from(bs / 512));
            assert_eq!(now.links_count, before.links_count);
            for (n, p) in mapping.iter().enumerate() {
                assert_eq!(fs.map_inode_logical(&now, n as u64).unwrap(), Some(*p));
            }
            assert_eq!(counters(&dev), (free.0 - 1, free.1));
            assert_eq!(
                fs.read_inode_verified(fs.resolve(b"/peer").unwrap())
                    .unwrap()
                    .1,
                peer
            );
            assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
            names.push(query.clone());
            names.sort();
            let mut actual = Vec::new();
            for n in 0..now.size / u64::from(bs) {
                if n < 2 {
                    continue;
                }
                let b = fs
                    .read_block(fs.map_inode_logical(&now, n).unwrap().unwrap())
                    .unwrap();
                for e in
                    crate::dir::parse_block_verified(&b, true, parent.ino, now.generation, &fs.csum)
                        .unwrap()
                {
                    if e.name != b"." && e.name != b".." {
                        assert_eq!(e.inode, target.ino);
                        actual.push(e.name);
                    }
                }
            }
            actual.sort();
            assert_eq!(actual, names);
            for name in &names {
                assert_eq!(fs.lookup_at(parent.ino, name).unwrap(), target.ino);
            }
            assert_eq!(
                usize::from(fs.stat_ino(target.ino).unwrap().links_count),
                names.len() + 1
            );
            assert_eq!(
                fs.chmod_operation().unwrap().operation.status().health,
                crate::operation::Health::Healthy
            );
        });
        fs.finish().unwrap();
        let fs = Filesystem::mount(dev).unwrap();
        assert_eq!(fs.lookup_at(parent.ino, &query).unwrap(), target.ino);
    }
}

fn corrupt(bs: u32) {
    for checksum in [false, true] {
        for capi in [false, true] {
            let Indexed {
                dev,
                fs,
                parent,
                target,
                clock,
                query,
                ..
            } = indexed(bs, false);
            let inode = fs.stat_ino(parent.ino).unwrap();
            let physical = fs.map_inode_logical(&inode, 2).unwrap().unwrap();
            let mut block = fs.read_block(physical).unwrap();
            if checksum {
                block[8] ^= 1;
            } else {
                block[4..6].copy_from_slice(&2u16.to_le_bytes());
                assert!(fs
                    .csum
                    .patch_dir_entry_tail(parent.ino, inode.generation, &mut block));
            }
            let mut buf = BlockBuffer::new(bs);
            buf.put(physical, block);
            fs.commit_block_buffer(buf).unwrap();
            let bytes = dev.bytes();
            let status = fs.chmod_operation().unwrap().operation.status();
            let marker = fs.marked_not_clean.load(Ordering::SeqCst);
            dev.reset();
            let _fs = with_test_mount(fs, |h, fs| {
                let result = if capi {
                    assert_eq!(
                        unsafe {
                            fs_ext4_link_at(
                                h,
                                target.ino,
                                target.generation.unwrap(),
                                parent.ino,
                                parent.generation.unwrap(),
                                query.as_ptr().cast(),
                                query.len(),
                                std::ptr::null_mut(),
                            )
                        },
                        -1
                    );
                    fs_ext4_last_errno()
                } else {
                    fs.apply_link_at(target, parent, &query)
                        .unwrap_err()
                        .to_errno()
                };
                assert_eq!(result, crate::error::errno::EIO);
                assert_eq!(dev.bytes(), bytes);
                assert_eq!(dev.counts()[1..3], [0, 0]);
                assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
                assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), marker);
                assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
            });
        }
    }
}

#[test]
fn full_node_1k() {
    decline(1024);
}
#[test]
fn full_node_4k() {
    decline(4096);
}
#[test]
fn last_slot_1k() {
    positive(1024);
}
#[test]
fn last_slot_4k() {
    positive(4096);
}
#[test]
fn corrupt_leaf_1k() {
    corrupt(1024);
}
#[test]
fn corrupt_leaf_4k() {
    corrupt(4096);
}
