//! Two-root-range native controls. Synthetic layouts are not Linux qualification.
use super::*;

use crate::fs::guarded_chmod::two_node_query as query_preflight;

struct Indexed {
    dev: Arc<Device>,
    fs: Filesystem,
    parent: InodeRef,
    target: InodeRef,
    clock: Arc<Clock>,
    names: Vec<Vec<u8>>,
    queries: [Vec<u8>; 2],
    selected: [u32; 2],
}

fn hash(fs: &Filesystem, name: &[u8]) -> u32 {
    crate::hash::name_hash(
        name,
        crate::hash::effective_version(crate::hash::HashVersion::HalfMd4, fs.sb.unsigned_hash()),
        &fs.sb.hash_seed,
    )
    .major
}

fn indexed(bs: u32, full_leaf: bool, full_node: Option<usize>) -> Indexed {
    let (dev, fs, parent, target, clock) = fixture(bs);
    let inode = fs.stat_ino(parent.ino).unwrap();
    let root_phys = fs.map_inode_logical(&inode, 0).unwrap().unwrap();
    let limit = ((bs - 16) / 8) as usize;
    let counts = std::array::from_fn::<_, 2, _>(|n| if full_node == Some(n) { limit } else { 2 });
    let capacity = ((bs - 12) / 264) as usize;
    let width = capacity + 1;
    let candidates = counts.iter().sum::<usize>() * width;
    assert!(candidates < 10000);
    // A bounded spare pool makes incidental equal hashes harmless to fixture
    // selection; exhaustion fails rather than changing the seed or operation.
    let budget = candidates + 64;
    // Keeping the four-digit names also gives a stricter bound than 16,384.
    assert!(budget < 10000);
    let mut pool: Vec<_> = (0..budget)
        .map(|n| {
            let name = long(n);
            (hash(&fs, &name), name)
        })
        .collect();
    pool.sort_by_key(|p| p.0);
    pool.dedup_by_key(|p| p.0);
    assert!(
        pool.len() >= candidates,
        "finite distinct-hash fixture pool exhausted"
    );
    pool.truncate(candidates);
    assert!(pool.windows(2).all(|w| w[0].0 < w[1].0));
    let queries = [
        pool[capacity].1.clone(),
        pool[counts[0] * width + capacity].1.clone(),
    ];
    let selected = [3, 3 + counts[0] as u32];
    let mut root = fs.read_block(root_phys).unwrap();
    root[30] = 1;
    root[32..].fill(0);
    root[32..34].copy_from_slice(&(((bs - 40) / 8) as u16).to_le_bytes());
    root[34..36].copy_from_slice(&2u16.to_le_bytes());
    root[36..40].copy_from_slice(&1u32.to_le_bytes());
    root[40..44].copy_from_slice(&pool[counts[0] * width].0.to_le_bytes());
    root[44..48].copy_from_slice(&2u32.to_le_bytes());
    let mut buf = BlockBuffer::new(bs);
    fs.buffer_append_dir_block(&mut buf, parent.ino, vec![0; bs as usize])
        .unwrap();
    let mut nodes = Vec::new();
    let mut names = Vec::new();
    let mut group = 0;
    for count in counts {
        let mut node = vec![0; bs as usize];
        node[4..6].copy_from_slice(&(bs as u16).to_le_bytes());
        node[8..10].copy_from_slice(&(limit as u16).to_le_bytes());
        node[10..12].copy_from_slice(&(count as u16).to_le_bytes());
        for slot in 0..count {
            let at = group * width;
            let take = if slot == 0 && full_leaf { capacity } else { 1 };
            let entries: Vec<_> = pool[at..at + take].iter().map(|p| p.1.clone()).collect();
            let mut bytes = leaf(bs, target.ino, &entries);
            assert!(fs
                .csum
                .patch_dir_entry_tail(parent.ino, inode.generation, &mut bytes));
            fs.buffer_append_dir_block(&mut buf, parent.ino, bytes)
                .unwrap();
            let offset = 8 + slot * 8;
            if slot != 0 {
                node[offset..offset + 4].copy_from_slice(&pool[at].0.to_le_bytes());
            }
            node[offset + 4..offset + 8].copy_from_slice(&((group + 3) as u32).to_le_bytes());
            names.extend(entries);
            group += 1;
        }
        assert!(fs
            .csum
            .patch_dx_tail(parent.ino, inode.generation, &mut node, 8));
        nodes.push(node);
    }
    let (current, _) = fs.buffered_inode_verified(&buf, parent.ino).unwrap();
    assert_eq!(
        crate::extent::ExtentHeader::parse(&current.block)
            .unwrap()
            .depth,
        0
    );
    for (n, node) in nodes.into_iter().enumerate() {
        buf.put(
            fs.map_inode_logical(&current, (n + 1) as u64)
                .unwrap()
                .unwrap(),
            node,
        );
    }
    assert!(fs
        .csum
        .patch_dx_tail(parent.ino, inode.generation, &mut root, 32));
    buf.put(root_phys, root);
    let (source, mut raw) = fs.read_inode_verified(target.ino).unwrap();
    fs.patch_inode_nlink(target.ino, &mut raw, &source, names.len() as i32)
        .unwrap();
    fs.buffer_write_inode(&mut buf, target.ino, &raw).unwrap();
    fs.commit_block_buffer(buf).unwrap();
    drop(fs);
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
    let actual = inventory(&fs, parent, target);
    names.sort();
    assert_eq!(actual, names);
    let current = fs.stat_ino(parent.ino).unwrap();
    let root = fs
        .read_block(fs.map_inode_logical(&current, 0).unwrap().unwrap())
        .unwrap();
    for range in 0..2 {
        assert_eq!(
            crate::htree::lookup_leaf_with(
                &queries[range],
                &root,
                &fs.sb.hash_seed,
                fs.sb.unsigned_hash(),
                |n| { fs.read_block(fs.map_inode_logical(&current, u64::from(n))?.unwrap()) }
            )
            .unwrap(),
            Some(selected[range])
        );
        let block = fs
            .read_block(
                fs.map_inode_logical(&current, u64::from(selected[range]))
                    .unwrap()
                    .unwrap(),
            )
            .unwrap();
        let mut trial = block.clone();
        let fits = crate::dir::add_entry_to_block(
            &mut trial,
            target.ino,
            &queries[range],
            crate::dir::DirEntryType::RegFile,
            true,
            12,
        )
        .is_ok();
        assert_eq!(fits, !full_leaf);
    }
    dev.reset();
    clock.calls.store(0, Ordering::SeqCst);
    Indexed {
        dev,
        fs,
        parent,
        target,
        clock,
        names,
        queries,
        selected,
    }
}

fn inventory(fs: &Filesystem, parent: InodeRef, target: InodeRef) -> Vec<Vec<u8>> {
    let inode = fs.stat_ino(parent.ino).unwrap();
    let bs = fs.sb.block_size();
    let read = |n| {
        fs.read_block(fs.map_inode_logical(&inode, u64::from(n)).unwrap().unwrap())
            .unwrap()
    };
    let root = read(0);
    fs.check_dx_block(parent.ino, &inode, &root, true).unwrap();
    assert_eq!(
        crate::htree::parse_root_info(&root)
            .unwrap()
            .indirect_levels,
        1
    );
    let (_, roots) = crate::htree::parse_root_entries(&root).unwrap();
    assert_eq!(roots.len(), 2);
    assert_eq!((roots[0].block, roots[1].block), (1, 2));
    assert!(roots[1].hash > 0 && roots[1].hash & 1 == 0);
    let mut leaves = std::collections::HashSet::new();
    let mut names = Vec::new();
    for (range, route) in roots.iter().enumerate() {
        let node = read(route.block);
        fs.check_dx_block(parent.ino, &inode, &node, false).unwrap();
        let (_, entries) = crate::htree::parse_node_entries(&node).unwrap();
        let upper = roots
            .get(range + 1)
            .map(|r| u64::from(r.hash))
            .unwrap_or(1 << 32);
        assert!(entries.windows(2).all(|w| w[0].hash < w[1].hash));
        for (slot, entry) in entries.iter().enumerate() {
            assert!(entry.block >= 3 && leaves.insert(entry.block));
            let lower = if slot == 0 { route.hash } else { entry.hash };
            let high = entries
                .get(slot + 1)
                .map(|e| u64::from(e.hash))
                .unwrap_or(upper);
            assert!(lower & 1 == 0 && u64::from(lower) < high && high <= upper);
            let records = crate::dir::parse_block_verified(
                &read(entry.block),
                true,
                parent.ino,
                inode.generation,
                &fs.csum,
            )
            .unwrap();
            assert!(!records.is_empty());
            for record in records {
                let major = u64::from(hash(fs, &record.name));
                assert!(major >= u64::from(lower) && major < high);
                assert_eq!(record.inode, target.ino);
                names.push(record.name);
            }
        }
    }
    assert_eq!(inode.size / u64::from(bs), leaves.len() as u64 + 3);
    assert_eq!(leaves, (3..(leaves.len() as u32 + 3)).collect());
    names.sort();
    assert!(names.windows(2).all(|w| w[0] < w[1]));
    for name in &names {
        assert_eq!(fs.lookup_at(parent.ino, name).unwrap(), target.ino);
    }
    names
}

fn invoke(
    fs: &Filesystem,
    h: *mut fs_ext4_fs_t,
    source: InodeRef,
    parent: InodeRef,
    name: &[u8],
    capi: bool,
    attr: &mut fs_ext4_attr_t,
) -> std::result::Result<(), i32> {
    if !capi {
        return fs
            .apply_link_at(source, parent, name)
            .map_err(|e| e.to_errno());
    }
    let rc = unsafe {
        fs_ext4_link_at(
            h,
            source.ino,
            source.generation.unwrap(),
            parent.ino,
            parent.generation.unwrap(),
            name.as_ptr().cast(),
            name.len(),
            attr,
        )
    };
    if rc == 0 {
        assert_eq!(fs_ext4_last_errno(), 0);
        Ok(())
    } else {
        assert_eq!(rc, -1);
        Err(fs_ext4_last_errno())
    }
}

fn positive(bs: u32) {
    query_controls(bs);
    for range in 0..2 {
        for split in [false, true] {
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
                } = indexed(bs, split, None);
                let query = &queries[range];
                let before = fs.stat_ino(parent.ino).unwrap();
                let source = fs.read_inode_verified(target.ino).unwrap();
                let parent_raw = fs.read_inode_verified(parent.ino).unwrap().1;
                let map: Vec<_> = (0..before.size / u64::from(bs))
                    .map(|n| fs.map_inode_logical(&before, n).unwrap().unwrap())
                    .collect();
                let blocks: Vec<_> = map.iter().map(|p| fs.read_block(*p).unwrap()).collect();
                let free = counters(&dev);
                let groups = group_counters(&fs);
                let bitmap = fs.read_block(fs.groups[0].block_bitmap).unwrap();
                let inode_bitmap = fs.read_block(fs.groups[0].inode_bitmap).unwrap();
                let peer = fs
                    .read_inode_verified(fs.resolve(b"/peer").unwrap())
                    .unwrap()
                    .1;
                let disk_before = dev.bytes();
                clock.now.store(STAMP, Ordering::SeqCst);
                let fs = with_test_mount(fs, |h, fs| {
                    let mut attr = unsafe { std::mem::zeroed() };
                    invoke(fs, h, target, parent, query, capi, &mut attr).unwrap();
                    let now = fs.stat_ino(parent.ino).unwrap();
                    assert_eq!(now.flags, before.flags);
                    assert_eq!(now.size, before.size + u64::from(bs) * u64::from(split));
                    assert_eq!(
                        now.blocks,
                        before.blocks + u64::from(bs / 512) * u64::from(split)
                    );
                    assert_eq!(now.links_count, before.links_count);
                    assert_eq!(
                        (now.mtime, now.mtime_nsec, now.ctime, now.ctime_nsec),
                        (STAMP, 0, STAMP, 0)
                    );
                    for (n, p) in map.iter().enumerate() {
                        assert_eq!(fs.map_inode_logical(&now, n as u64).unwrap(), Some(*p));
                    }
                    for (n, p) in map.iter().enumerate() {
                        if n != selected[range] as usize && !(split && n == range + 1) {
                            assert_eq!(fs.read_block(*p).unwrap(), blocks[n]);
                        }
                    }
                    let old_routes = crate::htree::parse_node_entries(&blocks[range + 1]).unwrap();
                    let node = fs.read_block(map[range + 1]).unwrap();
                    let new_routes = crate::htree::parse_node_entries(&node).unwrap();
                    assert_eq!(new_routes.0.count, old_routes.0.count + u16::from(split));
                    if split {
                        let inserted: Vec<_> = new_routes
                            .1
                            .iter()
                            .filter(|e| e.block == map.len() as u32)
                            .collect();
                        assert_eq!(inserted.len(), 1);
                        assert_eq!(
                            new_routes
                                .1
                                .iter()
                                .filter(|e| e.block != map.len() as u32)
                                .map(|e| (e.hash, e.block))
                                .collect::<Vec<_>>(),
                            old_routes
                                .1
                                .iter()
                                .map(|e| (e.hash, e.block))
                                .collect::<Vec<_>>()
                        );
                    }
                    assert_eq!(counters(&dev), (free.0 - u64::from(split), free.1));
                    assert_eq!(
                        group_counters(fs),
                        (groups.0 - u64::from(split), groups.1, groups.2)
                    );
                    let mut expected_bitmap = bitmap.clone();
                    let appended = if split {
                        let p = fs
                            .map_inode_logical(&now, map.len() as u64)
                            .unwrap()
                            .unwrap();
                        let (group, bit, _) = fs.group_chunks(p, 1).unwrap()[0];
                        assert_eq!(group, 0);
                        assert!(!crate::alloc::bit_is_set(&expected_bitmap, bit as u32));
                        expected_bitmap[(bit / 8) as usize] |= 1 << (bit % 8);
                        Some(p)
                    } else {
                        None
                    };
                    assert_eq!(
                        fs.read_block(fs.groups[0].block_bitmap).unwrap(),
                        expected_bitmap
                    );
                    assert_eq!(
                        fs.read_block(fs.groups[0].inode_bitmap).unwrap(),
                        inode_bitmap
                    );
                    assert_eq!(
                        fs.read_inode_verified(fs.resolve(b"/peer").unwrap())
                            .unwrap()
                            .1,
                        peer
                    );
                    source_preserved(fs, target.ino, &source.1, source.0.links_count);
                    let current_source = fs.stat_ino(target.ino).unwrap();
                    assert_eq!(
                        (current_source.ctime, current_source.ctime_nsec),
                        (STAMP, 0)
                    );
                    if capi {
                        assert_eq!(
                            (
                                attr.inode,
                                attr.generation,
                                attr.link_count,
                                attr.inode_flags
                            ),
                            (
                                target.ino,
                                current_source.generation,
                                current_source.links_count,
                                current_source.flags
                            )
                        );
                        assert_eq!(
                            (attr.mode, attr.uid, attr.gid, attr.size, attr.blocks_512),
                            (
                                current_source.mode & 0o7777,
                                current_source.uid,
                                current_source.gid,
                                current_source.size,
                                current_source.blocks
                            )
                        );
                        assert_eq!(
                            (
                                attr.atime,
                                attr.atime_nsec,
                                attr.mtime,
                                attr.mtime_nsec,
                                attr.ctime,
                                attr.ctime_nsec
                            ),
                            (
                                current_source.atime,
                                current_source.atime_nsec,
                                current_source.mtime,
                                current_source.mtime_nsec,
                                current_source.ctime,
                                current_source.ctime_nsec
                            )
                        );
                    }
                    let now_raw = fs.read_inode_verified(parent.ino).unwrap().1;
                    let version = |b: &[u8]| {
                        u64::from(u32::from_le_bytes(b[36..40].try_into().unwrap()))
                            | (u64::from(u32::from_le_bytes(b[152..156].try_into().unwrap())) << 32)
                    };
                    assert_eq!(version(&now_raw), version(&parent_raw) + 1);
                    let mut allowed =
                        vec![(12, 8), (36, 4), (124, 2), (130, 2), (132, 8), (152, 4)];
                    if split {
                        allowed.extend([(4, 4), (28, 4), (40, 60), (108, 4), (116, 2)]);
                    }
                    protected_disk(
                        fs,
                        &disk_before,
                        &dev.bytes(),
                        parent,
                        target,
                        &allowed,
                        selected[range],
                        split.then_some(range as u32 + 1),
                        appended,
                    );
                    names.push(query.clone());
                    names.sort();
                    assert_eq!(inventory(fs, parent, target), names);
                    assert_eq!(usize::from(current_source.links_count), names.len() + 1);
                    assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
                    assert_eq!(
                        fs.chmod_operation().unwrap().operation.status().health,
                        crate::operation::Health::Healthy
                    );
                });
                fs.finish().unwrap();
                let reopened = Filesystem::mount(dev).unwrap();
                assert_eq!(inventory(&reopened, parent, target), names);
            }
        }
    }
}

fn query_controls(bs: u32) {
    for full in [false, true] {
        let Indexed {
            dev,
            fs,
            parent,
            target,
            clock,
            ..
        } = indexed(bs, full, None);
        let before = dev.bytes();
        let marker = fs.marked_not_clean.load(Ordering::SeqCst);
        let status = fs.chmod_operation().unwrap().operation.status();
        let inode = fs.stat_ino(parent.ino).unwrap();
        let root = fs
            .read_block(fs.map_inode_logical(&inode, 0).unwrap().unwrap())
            .unwrap();
        let (_, roots) = crate::htree::parse_root_entries(&root).unwrap();
        for range in 0..2 {
            dev.reset();
            let result = query_preflight::select(
                &fs,
                parent.ino,
                target.ino,
                range,
                full,
                0..query_preflight::MAX_CANDIDATES,
            )
            .unwrap();
            let selected = result.selected.unwrap();
            assert_eq!(selected.name.len(), 255);
            assert!(!result.trace.contains("\"exhausted\""));
            assert_eq!(selected.root_slot, range);
            assert_eq!(selected.node, roots[range].block);
            assert_eq!(
                selected.node_physical,
                fs.map_inode_logical(&inode, u64::from(selected.node))
                    .unwrap()
                    .unwrap()
            );
            assert_eq!(
                selected.leaf_physical,
                fs.map_inode_logical(&inode, u64::from(selected.leaf))
                    .unwrap()
                    .unwrap()
            );
            assert_eq!(selected.major, hash(&fs, &selected.name));
            assert!(selected.major >= roots[range].hash);
            if range == 0 {
                assert!(selected.major < roots[1].hash);
            }
            assert_eq!(selected.boundary.is_some(), full);
            assert_eq!(selected.split, full);
            assert_eq!(
                crate::htree::lookup_leaf_with(
                    &selected.name,
                    &root,
                    &fs.sb.hash_seed,
                    fs.sb.unsigned_hash(),
                    |n| { fs.read_block(fs.map_inode_logical(&inode, u64::from(n))?.unwrap()) }
                )
                .unwrap(),
                Some(selected.leaf)
            );
            let again = query_preflight::select(
                &fs,
                parent.ino,
                target.ino,
                range,
                full,
                0..query_preflight::MAX_CANDIDATES,
            )
            .unwrap();
            assert_eq!(again.selected.unwrap().json(), selected.json());
            assert_eq!(again.trace, result.trace);
            assert_eq!(dev.bytes(), before);
            assert_eq!(dev.counts()[1..3], [0, 0]);
        }
        if !full {
            let exhausted =
                query_preflight::select(&fs, parent.ino, target.ino, 0, true, 0..64).unwrap();
            assert!(exhausted.selected.is_none());
            assert!(exhausted.trace.ends_with("{\"event\":\"exhausted\"}\n"));
            assert_eq!(exhausted.trace.lines().count(), 66);
        }
        for (range, candidates) in [(2, 0..1), (0, 0..0), (0, 0..16_385)] {
            assert!(matches!(
                query_preflight::select(&fs, parent.ino, target.ino, range, false, candidates),
                Err(Error::InvalidArgument(_))
            ));
        }
        assert!(matches!(
            query_preflight::select(&fs, target.ino, target.ino, 0, false, 0..1),
            Err(Error::Unsupported(_))
        ));
        assert_eq!(dev.bytes(), before);
        assert_eq!(dev.counts()[1..3], [0, 0]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), marker);
        assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
    }
}

#[allow(clippy::too_many_arguments)]
fn protected_disk(
    fs: &Filesystem,
    before: &[u8],
    after: &[u8],
    parent: InodeRef,
    target: InodeRef,
    parent_fields: &[(usize, usize)],
    leaf: u32,
    node: Option<u32>,
    appended: Option<u64>,
) {
    let bs = fs.sb.block_size() as usize;
    let preserve = |block: u64, fields: &[(usize, usize)]| {
        let start = block as usize * bs;
        let mut expected = before[start..start + bs].to_vec();
        for &(offset, length) in fields {
            expected[offset..offset + length]
                .copy_from_slice(&after[start + offset..start + offset + length]);
        }
        assert_eq!(
            expected,
            after[start..start + bs],
            "protected metadata block {block}"
        );
    };
    let sb_block = if bs == 1024 { 1 } else { 0 };
    let sb_offset = 1024 % bs;
    let mut sb_fields = vec![(sb_offset + 58, 2), (sb_offset + 1020, 4)];
    if appended.is_some() {
        sb_fields.extend([(sb_offset + 12, 4), (sb_offset + 344, 4)]);
    }
    preserve(sb_block, &sb_fields);
    let (descriptor, offset) = fs.sb.descriptor_location(0);
    let descriptor_fields = if appended.is_some() {
        vec![
            (offset + 12, 2),
            (offset + 24, 2),
            (offset + 30, 2),
            (offset + 44, 2),
            (offset + 56, 2),
        ]
    } else {
        vec![]
    };
    preserve(descriptor, &descriptor_fields);
    let inode = fs.stat_ino(parent.ino).unwrap();
    let mut allowed = std::collections::HashSet::new();
    allowed.insert(
        fs.map_inode_logical(&inode, u64::from(leaf))
            .unwrap()
            .unwrap(),
    );
    if let Some(n) = node {
        allowed.insert(fs.map_inode_logical(&inode, u64::from(n)).unwrap().unwrap());
    }
    if let Some(p) = appended {
        allowed.insert(p);
        allowed.insert(fs.groups[0].block_bitmap);
        allowed.insert(fs.sb.descriptor_location(0).0);
    }
    allowed.insert(if bs == 1024 { 1 } else { 0 });
    let mut tables = std::collections::BTreeMap::new();
    for (ino, fields) in [
        (parent.ino, parent_fields),
        (
            target.ino,
            &[(12, 4), (26, 2), (124, 2), (130, 2), (132, 4)][..],
        ),
    ] {
        let (block, offset) = crate::bgd::locate_inode(&fs.sb, &fs.groups, ino).unwrap();
        let offset = offset as usize;
        let expected = tables
            .entry(block)
            .or_insert_with(|| before[block as usize * bs..(block as usize + 1) * bs].to_vec());
        for &(at, len) in fields {
            expected[offset + at..offset + at + len].copy_from_slice(
                &after[block as usize * bs + offset + at..block as usize * bs + offset + at + len],
            );
        }
        allowed.insert(block);
    }
    for (block, expected) in tables {
        assert_eq!(
            expected,
            &after[block as usize * bs..(block as usize + 1) * bs]
        );
    }
    for (block, (a, b)) in before.chunks(bs).zip(after.chunks(bs)).enumerate() {
        if !allowed.contains(&(block as u64)) {
            assert_eq!(a, b, "protected block {block}");
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn no_effect(
    fs: &Filesystem,
    dev: &Device,
    clock: &Clock,
    h: *mut fs_ext4_fs_t,
    parent: InodeRef,
    target: InodeRef,
    name: &[u8],
    expected: i32,
) {
    let before = dev.bytes();
    let status = fs.chmod_operation().unwrap().operation.status();
    let marker = fs.marked_not_clean.load(Ordering::SeqCst);
    for capi in [false, true] {
        dev.reset();
        let mut attr = unsafe { std::mem::zeroed() };
        assert_eq!(
            invoke(fs, h, target, parent, name, capi, &mut attr),
            Err(expected)
        );
        assert_eq!(dev.bytes(), before);
        assert_eq!(dev.counts()[1..3], [0, 0]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), marker);
        assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
    }
}

fn corruption(bs: u32) {
    for range in 0..2 {
        let Indexed {
            dev,
            fs,
            parent,
            target,
            clock,
            queries,
            selected,
            ..
        } = indexed(bs, false, None);
        let inode = fs.stat_ino(parent.ino).unwrap();
        let block = fs
            .read_block(
                fs.map_inode_logical(&inode, u64::from(selected[range]))
                    .unwrap()
                    .unwrap(),
            )
            .unwrap();
        let duplicate =
            crate::dir::parse_block_verified(&block, true, parent.ino, inode.generation, &fs.csum)
                .unwrap()[0]
                .name
                .clone();
        let _fs = with_test_mount(fs, |h, fs| {
            no_effect(
                fs,
                &dev,
                &clock,
                h,
                parent,
                target,
                &duplicate,
                crate::error::errno::EEXIST,
            );
            no_effect(
                fs,
                &dev,
                &clock,
                h,
                parent,
                target,
                &[b'x'; 256],
                crate::error::errno::ENAMETOOLONG,
            );
            no_effect(
                fs,
                &dev,
                &clock,
                h,
                parent,
                parent,
                &queries[range],
                crate::error::errno::EISDIR,
            );
        });
        for full in [false, true] {
            for kind in 0..6 {
                for dirty in [false, true] {
                    let Indexed {
                        dev,
                        fs,
                        parent,
                        target,
                        clock,
                        queries,
                        selected,
                        ..
                    } = indexed(bs, full, None);
                    let inode = fs.stat_ino(parent.ino).unwrap();
                    let logical = match kind / 2 {
                        0 => 0,
                        1 => range as u32 + 1,
                        _ => selected[range],
                    };
                    let physical = fs
                        .map_inode_logical(&inode, u64::from(logical))
                        .unwrap()
                        .unwrap();
                    let mut block = fs.read_block(physical).unwrap();
                    if kind % 2 == 0 {
                        block[bs as usize - 1] ^= 1;
                    } else {
                        match kind / 2 {
                            0 => {
                                block[29] = 9;
                                assert!(fs.csum.patch_dx_tail(
                                    parent.ino,
                                    inode.generation,
                                    &mut block,
                                    32
                                ));
                            }
                            1 => {
                                block[10..12].copy_from_slice(&0u16.to_le_bytes());
                                assert!(fs.csum.patch_dx_tail(
                                    parent.ino,
                                    inode.generation,
                                    &mut block,
                                    8
                                ));
                            }
                            _ => {
                                block[4..6].copy_from_slice(&2u16.to_le_bytes());
                                assert!(fs.csum.patch_dir_entry_tail(
                                    parent.ino,
                                    inode.generation,
                                    &mut block
                                ));
                            }
                        }
                    }
                    let mut buf = BlockBuffer::new(bs);
                    buf.put(physical, block);
                    fs.commit_block_buffer(buf).unwrap();
                    if dirty {
                        fs.mark_not_clean_once().unwrap();
                    }
                    let bytes = dev.bytes();
                    dev.reset();
                    assert!(query_preflight::select(
                        &fs,
                        parent.ino,
                        target.ino,
                        range,
                        full,
                        0..64
                    )
                    .is_err());
                    assert_eq!(dev.bytes(), bytes);
                    assert_eq!(dev.counts()[1..3], [0, 0]);
                    let _fs = with_test_mount(fs, |h, fs| {
                        no_effect(
                            fs,
                            &dev,
                            &clock,
                            h,
                            parent,
                            target,
                            &queries[range],
                            crate::error::errno::EIO,
                        );
                    });
                }
            }
        }
        decline(bs, range);
    }
}

fn decline(bs: u32, range: usize) {
    let Indexed {
        dev,
        fs,
        parent,
        target,
        clock,
        queries,
        ..
    } = indexed(bs, true, Some(range));
    let inode = fs.stat_ino(parent.ino).unwrap();
    let node = fs
        .read_block(
            fs.map_inode_logical(&inode, range as u64 + 1)
                .unwrap()
                .unwrap(),
        )
        .unwrap();
    let (count, _) = crate::htree::parse_node_entries(&node).unwrap();
    assert_eq!(count.count, count.limit);
    let search = query_preflight::select(&fs, parent.ino, target.ino, range, true, 0..64).unwrap();
    assert!(search.selected.is_none());
    assert!(search.trace.ends_with("{\"event\":\"exhausted\"}\n"));
    let mut buf = BlockBuffer::new(bs);
    buf.uninit_cleared.insert(0, 0);
    let pending = buf.uninit_cleared.clone();
    let staged = buf.dirty.clone();
    let bytes = dev.bytes();
    let marker = fs.marked_not_clean.load(Ordering::SeqCst);
    let status = fs.chmod_operation().unwrap().operation.status();
    dev.reset();
    assert!(!fs
        .buffer_split_htree_leaf_and_add_entry(
            &mut buf,
            parent.ino,
            &queries[range],
            target.ino,
            crate::dir::DirEntryType::RegFile,
            true
        )
        .unwrap());
    assert_eq!(buf.dirty, staged);
    assert_eq!(buf.uninit_cleared, pending);
    assert_eq!(dev.bytes(), bytes);
    assert_eq!(dev.counts()[1..3], [0, 0]);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
    assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), marker);
    assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
}

#[test]
fn positive_1k() {
    positive(1024);
}
#[test]
fn positive_4k() {
    positive(4096);
}
#[test]
fn corruption_1k() {
    corruption(1024);
}
#[test]
fn corruption_4k() {
    corruption(4096);
}
