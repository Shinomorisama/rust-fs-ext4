//! Public ordinary full-root fallback against exact Linux predecessors.
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

struct DriverClock {
    stamp: i64,
    calls: AtomicUsize,
}
impl crate::runtime::Runtime for DriverClock {
    fn now_unix_seconds(&self) -> i64 {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.stamp
    }
    fn next_inode_generation(&self) -> u32 {
        panic!("hard link must retain generation")
    }
}
fn dispatch(
    fs: &Filesystem,
    h: *mut fs_ext4_fs_t,
    capi: bool,
    target: InodeRef,
    parent: InodeRef,
    name: &[u8],
) -> std::result::Result<Option<fs_ext4_attr_t>, i32> {
    let alias = Alias {
        phase: "capacity",
        source: 1,
        parent: 0,
        name: name.to_vec(),
        entry: 1,
    };
    call(
        fs,
        h,
        capi,
        &alias,
        (target, parent),
        (b"/source", &[b"/policy/".as_slice(), name].concat(), name),
    )
}

fn linear(image: &Path, prior: &Snapshot) -> Snapshot {
    let fs = mount(image);
    assert!(fs.journal.is_none());
    assert_ne!(fs.sb.state & crate::superblock::EXT4_VALID_FS, 0);
    assert_eq!(
        fs.sb.feature_incompat & features::Incompat::RECOVER.bits(),
        0
    );
    if let Some(j) = crate::jbd2::read_superblock(&fs).unwrap() {
        assert!(j.is_clean());
    }
    let records: Vec<_> = TRACKED
        .iter()
        .map(|p| {
            let ino = fs.resolve(p).unwrap();
            let (i, r) = fs.read_inode_verified(ino).unwrap();
            (ino, i, r)
        })
        .collect();
    let (ino, parent, _) = &records[0];
    let bs = fs.sb.block_size();
    assert_eq!(
        parent.flags,
        prior.view.records[0].1.flags & !crate::inode::InodeFlags::INDEX.bits()
    );
    assert_eq!(parent.size, prior.view.records[0].1.size + u64::from(bs));
    let len = (parent.size / u64::from(bs)) as usize;
    assert!(len <= MAX_DIR_BLOCKS);
    let ctx = crate::extent::ExtentVerifyCtx {
        ino: *ino,
        generation: parent.generation,
        csum: &fs.csum,
    };
    assert!(
        crate::extent::ExtentHeader::parse(&parent.block)
            .unwrap()
            .depth
            <= 1
    );
    let mapping: Vec<_> = (0..len)
        .map(|n| {
            crate::extent::map_logical_verified(
                &parent.block,
                fs.dev.as_ref(),
                bs,
                n as u64,
                Some(&ctx),
            )
            .unwrap()
            .unwrap()
        })
        .collect();
    assert_eq!(
        crate::extent::map_logical_verified(
            &parent.block,
            fs.dev.as_ref(),
            bs,
            len as u64,
            Some(&ctx)
        )
        .unwrap(),
        None
    );
    let (extents, nodes) =
        crate::extent::collect_all_with_nodes(&parent.block, fs.dev.as_ref(), bs).unwrap();
    assert_eq!(
        extents.iter().map(|e| usize::from(e.length)).sum::<usize>(),
        len
    );
    assert!(nodes.len() <= 4);
    let mut physical = HashSet::new();
    for p in mapping.iter().chain(&nodes) {
        assert!(*p < fs.sb.blocks_count && physical.insert(*p));
    }
    let map_nodes = nodes
        .into_iter()
        .map(|p| {
            let b = fs.read_block(p).unwrap();
            assert!(fs.csum.verify_extent_tail(*ino, parent.generation, &b));
            (p, b)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        parent.blocks,
        (len + map_nodes.len() + 1) as u64 * u64::from(bs / 512)
    );
    let directory = mapping
        .iter()
        .map(|p| fs.read_block(*p).unwrap())
        .collect::<Vec<_>>();
    let mut entries = Vec::new();
    for b in &directory {
        for e in
            crate::dir::parse_block_verified(b, true, *ino, parent.generation, &fs.csum).unwrap()
        {
            if e.name == b"." {
                assert_eq!(e.inode, *ino);
                continue;
            }
            if e.name == b".." {
                assert_eq!(e.inode, 2);
                continue;
            }
            assert_eq!(e.inode, records[1].0);
            assert_eq!(e.file_type, crate::dir::DirEntryType::RegFile);
            entries.push((e.name, e.inode));
        }
    }
    entries.sort();
    assert!(entries.windows(2).all(|w| w[0].0 != w[1].0));
    assert_eq!(usize::from(records[1].1.links_count), entries.len() + 1);
    let tables = prior
        .view
        .tables
        .iter()
        .map(|(p, _)| (*p, fs.read_block(*p).unwrap()))
        .collect();
    let eas = records
        .iter()
        .map(|(_, i, _)| {
            assert_ne!(i.file_acl, 0);
            let b = fs.read_block(i.file_acl).unwrap();
            assert!(fs.csum.verify_xattr_block(i.file_acl, &b));
            b
        })
        .collect();
    let payloads = records[1..]
        .iter()
        .map(|(_, i, _)| {
            fs.read_block(fs.map_inode_logical(i, 0).unwrap().unwrap())
                .unwrap()
        })
        .collect();
    let root = fs.read_inode_verified(2).unwrap().0;
    let root_directory = (0..root.size.div_ceil(u64::from(bs)))
        .map(|n| {
            fs.read_block(fs.map_inode_logical(&root, n).unwrap().unwrap())
                .unwrap()
        })
        .collect();
    Snapshot {
        view: View {
            records,
            tables,
            allocation: allocation(&fs),
            eas,
            payloads,
            free: free(image),
            mapping,
            entries,
            roots: 0,
            directory,
            root_directory,
        },
        depth: 0,
        limit: 0,
        index_nodes: vec![],
        leaves: (0..len as u32).collect(),
        map_nodes,
    }
}

fn save(image: &Path, base: &Path, phase: &str, prior: &Snapshot) -> Snapshot {
    std::fs::copy(image, base.with_extension(format!("{phase}.img"))).unwrap();
    let s = linear(image, prior);
    let out = oracle("e2fsck")
        .arg("-fn")
        .arg(image)
        .judged()
        .clean("ordinary full-root driver fallback");
    std::fs::write(
        base.with_extension(format!("{phase}.e2fsck.txt")),
        out.stdout,
    )
    .unwrap();
    std::fs::write(
        base.with_extension(format!("{phase}.e2fsck.stderr.txt")),
        out.stderr,
    )
    .unwrap();
    report(image, base, phase, &s.view.records);
    for (n, (_, _, raw)) in s.view.records.iter().enumerate() {
        std::fs::write(base.with_extension(format!("{phase}.{n}.inode.bin")), raw).unwrap();
    }
    let rows = s
        .view
        .entries
        .iter()
        .map(|(name, ino)| {
            format!(
                "[\"{}\",{ino}]",
                name.iter().map(|b| format!("{b:02x}")).collect::<String>()
            )
        })
        .collect::<Vec<_>>();
    std::fs::write(base.with_extension(format!("{phase}.inventory.json")),format!("{{\"flags\":{},\"size\":{},\"blocks\":{},\"mapping\":{:?},\"extent_nodes\":{:?},\"entries\":[{}]}}\n",s.view.records[0].1.flags,s.view.records[0].1.size,s.view.records[0].1.blocks,s.view.mapping,s.map_nodes.iter().map(|n|n.0).collect::<Vec<_>>(),rows.join(","))).unwrap();
    s
}

fn transition(before: &Snapshot, after: &Snapshot, image: &Path, name: &[u8], stamp: i64) {
    let (a, b) = (&before.view, &after.view);
    let fs = mount(image);
    let bs = fs.sb.block_size();
    let mut expected = a.entries.clone();
    expected.push((name.to_vec(), a.records[1].0));
    expected.sort();
    assert_eq!(b.entries, expected);
    assert_eq!(&b.mapping[..a.mapping.len()], a.mapping);
    assert_eq!(b.mapping.len(), a.mapping.len() + 1);
    let mut added = b.mapping[a.mapping.len()..].to_vec();
    for (p, _) in &before.map_nodes {
        assert!(after.map_nodes.iter().any(|n| n.0 == *p));
    }
    added.extend(
        after
            .map_nodes
            .iter()
            .filter(|n| !before.map_nodes.iter().any(|p| p.0 == n.0))
            .map(|n| n.0),
    );
    assert!((1..=2).contains(&added.len()));
    assert_eq!(b.free, (a.free.0 - added.len() as u64, a.free.1));
    assert_eq!(
        b.records[0].1.blocks,
        a.records[0].1.blocks + added.len() as u64 * u64::from(bs / 512)
    );
    assert_eq!(b.records[0].1.links_count, a.records[0].1.links_count);
    assert_eq!(b.records[1].1.links_count, a.records[1].1.links_count + 1);
    assert_eq!(
        (
            b.records[0].1.mtime,
            b.records[0].1.mtime_nsec,
            b.records[0].1.ctime,
            b.records[0].1.ctime_nsec
        ),
        (stamp, 0, stamp, 0)
    );
    assert_eq!(
        (b.records[1].1.ctime, b.records[1].1.ctime_nsec),
        (stamp, 0)
    );
    assert_eq!(version(&b.records[0].2), version(&a.records[0].2) + 1);
    stable(
        &a.records[0].2,
        &b.records[0].2,
        &[
            (4, 4),
            (12, 8),
            (28, 4),
            (32, 8),
            (40, 60),
            (108, 4),
            (116, 2),
            (124, 2),
            (130, 2),
            (132, 8),
            (152, 4),
        ],
    );
    stable(
        &a.records[1].2,
        &b.records[1].2,
        &[(12, 4), (26, 2), (124, 2), (130, 2), (132, 4)],
    );
    assert_eq!(a.records[2].2, b.records[2].2);
    assert_eq!(a.eas, b.eas);
    assert_eq!(a.payloads, b.payloads);
    assert_eq!(a.root_directory, b.root_directory);
    stable(
        &a.directory[0],
        &b.directory[0],
        &[(16, 2), (bs as usize - 12, 12)],
    );
    assert_eq!(&b.directory[1..a.directory.len()], &a.directory[1..]);
    let locations = a.records[..2]
        .iter()
        .map(|r| crate::bgd::locate_inode(&fs.sb, &fs.groups, r.0).unwrap())
        .collect::<Vec<_>>();
    for ((p, x), (q, y)) in a.tables.iter().zip(&b.tables) {
        assert_eq!(p, q);
        stable(
            x,
            y,
            &locations
                .iter()
                .filter(|(n, _)| n == p)
                .map(|(_, o)| (*o as usize, usize::from(fs.sb.inode_size)))
                .collect::<Vec<_>>(),
        );
    }
    for g in 0..fs.groups.len() {
        let row = g * 3;
        let mut bitmap = a.allocation[row + 2].clone();
        let mut debit = 0;
        for p in &added {
            let rel = *p - u64::from(fs.sb.first_data_block);
            if rel / u64::from(fs.sb.blocks_per_group) == g as u64 {
                let bit = rel % u64::from(fs.sb.blocks_per_group);
                let byte = (bit / 8) as usize;
                let mask = 1u8 << (bit % 8);
                assert_eq!(bitmap[byte] & mask, 0);
                bitmap[byte] |= mask;
                debit += 1;
            }
        }
        assert_eq!(bitmap, b.allocation[row + 2]);
        assert_eq!(a.allocation[row + 1], b.allocation[row + 1]);
        let (block, offset) = fs.sb.descriptor_location(g as u64);
        let old = crate::bgd::BlockGroupDescriptor::parse(
            &a.allocation[row][offset..offset + usize::from(fs.sb.desc_size)],
            fs.sb.desc_size,
        )
        .unwrap();
        let new = crate::bgd::BlockGroupDescriptor::parse(
            &b.allocation[row][offset..offset + usize::from(fs.sb.desc_size)],
            fs.sb.desc_size,
        )
        .unwrap();
        assert_eq!(new.free_blocks_count, old.free_blocks_count - debit);
        let mut allowed = Vec::new();
        for n in 0..fs.groups.len() {
            let (nr, at) = fs.sb.descriptor_location(n as u64);
            if nr == block
                && added.iter().any(|p| {
                    (*p - u64::from(fs.sb.first_data_block)) / u64::from(fs.sb.blocks_per_group)
                        == n as u64
                })
            {
                allowed.extend([
                    (at + 12, 2),
                    (at + 24, 2),
                    (at + 30, 2),
                    (at + 44, 2),
                    (at + 56, 2),
                ]);
            }
        }
        stable(&a.allocation[row], &b.allocation[row], &allowed);
    }
}

fn emit(source: &Path, linux: &Snapshot, capi: bool, bs: u32, journal: bool) {
    let base = fs_ext4_test_support::temp_dir().join(format!(
        "link-capacity-driver-{}-{bs}-{journal}",
        if capi { "capi" } else { "rust" }
    ));
    let image = base.with_extension("working.img");
    std::fs::copy(source, &image).unwrap();
    let input = capture(&image, &base, "input");
    assert_eq!(input.depth, 0);
    assert_eq!(input.view.roots, usize::from(input.limit));
    let total = input.view.entries.len();
    let name = format!("cap-{total:05}-{}", "n".repeat(245)).into_bytes();
    assert_eq!(name.len(), 255);
    let clock = Arc::new(DriverClock {
        stamp: input.view.records.iter().map(|r| r.1.ctime).max().unwrap() + 100,
        calls: AtomicUsize::new(0),
    });
    assert!(clock.stamp < i64::from(i32::MAX));
    let fs = Filesystem::mount_with_runtime(
        Arc::new(FileDevice::open_path_rw(&image).unwrap()),
        clock.clone(),
    )
    .unwrap();
    assert_eq!(fs.journal.is_some(), journal);
    let parent = InodeRef::new(input.view.records[0].0, input.view.records[0].1.generation);
    let target = InodeRef::new(input.view.records[1].0, input.view.records[1].1.generation);
    let fs = crate::capi::with_test_mount(fs, |h, fs| {
        let mounted = std::fs::read(&image).unwrap();
        for (candidate, errno) in [
            (
                input.view.entries[0].0.as_slice(),
                crate::error::errno::EEXIST,
            ),
            (&[b'x'; 256], crate::error::errno::ENAMETOOLONG),
        ] {
            assert_eq!(
                dispatch(fs, h, capi, target, parent, candidate).err(),
                Some(errno)
            );
            assert_eq!(std::fs::read(&image).unwrap(), mounted);
            assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        }
        assert_eq!(
            dispatch(fs, h, capi, parent, parent, b"directory-link").err(),
            Some(crate::error::errno::EISDIR)
        );
        assert_eq!(std::fs::read(&image).unwrap(), mounted);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        std::fs::copy(&image, base.with_extension("refused.img")).unwrap();
        let attr = dispatch(fs, h, capi, target, parent, &name).unwrap();
        assert_eq!(attr.is_some(), capi);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
        assert_eq!(fs.lookup_at(parent.ino, &name).unwrap(), target.ino);
        assert_eq!(
            fs.stat_ino(parent.ino).unwrap().flags & crate::inode::InodeFlags::INDEX.bits(),
            0
        );
        if let Some(a) = attr {
            check_attr(&a, target.ino, &fs.stat_ino(target.ino).unwrap());
            std::fs::write(
                base.with_extension("capi-attr.json"),
                format!(
                    "{{\"identity\":{:?},\"times\":{:?}}}\n",
                    [
                        u64::from(a.inode),
                        u64::from(a.generation),
                        u64::from(a.mode),
                        u64::from(a.uid),
                        u64::from(a.gid),
                        a.size,
                        a.blocks_512,
                        u64::from(a.link_count),
                        u64::from(a.inode_flags)
                    ],
                    [
                        a.atime,
                        i64::from(a.atime_nsec),
                        a.mtime,
                        i64::from(a.mtime_nsec),
                        a.ctime,
                        i64::from(a.ctime_nsec)
                    ]
                ),
            )
            .unwrap();
        }
        std::fs::copy(&image, base.with_extension("dirty.img")).unwrap();
    });
    fs.finish().unwrap();
    assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
    let closed = save(&image, &base, "closed", &input);
    transition(&input, &closed, &image, &name, clock.stamp);
    assert_eq!(closed.view.entries, linux.view.entries);
    std::fs::write(base.with_extension("observation.json"),format!("{{\"capi\":{capi},\"block_size\":{bs},\"journal\":{journal},\"clock_calls\":1,\"refusals\":3,\"before\":{total},\"after\":{},\"stamp\":{}}}\n",closed.view.entries.len(),clock.stamp)).unwrap();
    let cold = base.with_extension("cold.img");
    std::fs::copy(&image, &cold).unwrap();
    let expected = format!(
        "EXPECTED_INDEX = False\nEXPECTED = [{}]\nFLAGS = {:?}\nFREE = ({}, {})\n",
        TRACKED
            .iter()
            .zip(&closed.view.records)
            .map(|(p, (ino, i, _))| literal(p, *ino, i))
            .collect::<Vec<_>>()
            .join(","),
        closed
            .view
            .records
            .iter()
            .map(|r| r.1.flags)
            .collect::<Vec<_>>(),
        closed.view.free.0,
        closed.view.free.1
    );
    std::fs::write(base.with_extension("cold.expected.py"), &expected).unwrap();
    guest(
        &cold,
        &base,
        "cold-guest",
        "cold",
        closed.view.entries.len(),
        0,
        &expected,
    );
    let observed = save(&cold, &base, "cold-readback", &input);
    assert_eq!(closed.view.directory, observed.view.directory);
    assert_eq!(closed.view.tables, observed.view.tables);
    assert_eq!(closed.view.allocation, observed.view.allocation);
    assert_eq!(closed.view.eas, observed.view.eas);
    assert_eq!(closed.view.payloads, observed.view.payloads);
    assert_eq!(closed.view.root_directory, observed.view.root_directory);
    assert_eq!(closed.map_nodes, observed.map_nodes);
    eprintln!("full-root fallback capi={capi} bs={bs} journal={journal}: complete namespace, INDEX cleared, one linear block, exact allocator/protected bytes and cold Linux metadata checked");
    std::fs::remove_file(image).unwrap();
    std::fs::remove_file(cold).unwrap();
}

#[test]
fn linux_checks_public_full_root_fallback() {
    for bs in [1024, 4096] {
        for journal in [false, true] {
            profile_named(bs, journal, "link-capacity-driver-source");
            let base = fs_ext4_test_support::temp_dir()
                .join(format!("link-capacity-driver-source-{bs}-{journal}"));
            let source = base.with_extension("capacity.img");
            let linux = observe(&base.with_extension("grown.img"));
            for capi in [false, true] {
                emit(&source, &linux, capi, bs, journal);
            }
        }
    }
}
