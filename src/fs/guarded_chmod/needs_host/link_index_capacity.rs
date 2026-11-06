//! Linux-only first root-capacity transition; no writable driver mount.
use super::*;
use std::collections::HashSet;
use std::io::Write;

#[path = "link_index_capacity_driver.rs"]
mod driver_emission;

#[path = "link_index_intermediate_driver.rs"]
mod intermediate;

#[path = "link_index_node_full_reference.rs"]
mod node_full;

const MAX_NAMES: usize = 16_384;
const BATCH: usize = 256;
const MAX_DIR_BLOCKS: usize = 1024;
const GUEST: &str = include_str!("link_index_capacity_guest.py");

#[derive(Debug)]
struct Probe {
    depth: u8,
    count: u16,
    limit: u16,
    total: usize,
}

struct Snapshot {
    view: View,
    depth: u8,
    limit: u16,
    index_nodes: Vec<u32>,
    leaves: Vec<u32>,
    map_nodes: Vec<(u64, Vec<u8>)>,
}

fn guest(
    image: &Path,
    base: &Path,
    label: &str,
    mode: &str,
    start: usize,
    count: usize,
    expected: &str,
) {
    let out = guest_kernel_write(image.to_str().unwrap(), &format!(
        "python3 - \"$MNT\" {mode} {start} {count} <<'INDEX_CAPACITY'\n{expected}\n{GUEST}\nINDEX_CAPACITY\n"));
    std::fs::write(base.with_extension(format!("{label}.jsonl")), &out.stdout).unwrap();
    std::fs::write(
        base.with_extension(format!("{label}.stderr.txt")),
        &out.stderr,
    )
    .unwrap();
    assert!(
        out.status.success(),
        "{label}: {}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|s| s.starts_with('{'))
            .count(),
        1
    );
}

fn probe(image: &Path, base: &Path, label: &str) -> Probe {
    let fs = mount(image);
    let ino = fs.resolve(b"/policy").unwrap();
    let parent = fs.read_inode_verified(ino).unwrap().0;
    // Retain unexpected flags before requiring this bounded positive profile.
    let mut trace = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(base.with_extension("search.jsonl"))
        .unwrap();
    writeln!(
        trace,
        "{{\"phase\":\"{label}\",\"flags\":{},\"size\":{},\"links\":{}}}",
        parent.flags,
        parent.size,
        fs.stat_ino(fs.resolve(b"/source").unwrap())
            .unwrap()
            .links_count
    )
    .unwrap();
    assert_ne!(
        parent.flags & crate::inode::InodeFlags::INDEX.bits(),
        0,
        "Linux dropped INDEX; retained trace/image describe the unadmitted outcome"
    );
    let block = fs
        .read_block(fs.map_inode_logical(&parent, 0).unwrap().unwrap())
        .unwrap();
    fs.check_dx_block(ino, &parent, &block, true).unwrap();
    let info = crate::htree::parse_root_info(&block).unwrap();
    let (count, _) = crate::htree::parse_root_entries(&block).unwrap();
    let total = usize::from(
        fs.stat_ino(fs.resolve(b"/source").unwrap())
            .unwrap()
            .links_count,
    ) - 1;
    writeln!(
        trace,
        "{{\"phase\":\"{label}\",\"depth\":{},\"count\":{},\"limit\":{},\"total\":{total}}}",
        info.indirect_levels, count.count, count.limit
    )
    .unwrap();
    assert!(info.indirect_levels <= 1 && total <= MAX_NAMES);
    Probe {
        depth: info.indirect_levels,
        count: count.count,
        limit: count.limit,
        total,
    }
}

fn observe(image: &Path) -> Snapshot {
    let fs = mount(image);
    assert!(fs.journal.is_none());
    assert_ne!(fs.sb.state & crate::superblock::EXT4_VALID_FS, 0);
    assert_eq!(
        fs.sb.feature_incompat & features::Incompat::RECOVER.bits(),
        0
    );
    if let Some(journal) = crate::jbd2::read_superblock(&fs).unwrap() {
        assert!(journal.is_clean());
    }
    let records: Vec<_> = TRACKED
        .iter()
        .map(|p| {
            let ino = fs.resolve(p).unwrap();
            let (inode, raw) = fs.read_inode_verified(ino).unwrap();
            (ino, inode, raw)
        })
        .collect();
    let (ino, parent, _) = &records[0];
    let bs = fs.sb.block_size();
    assert!(parent.is_dir() && parent.has_extents() && !parent.has_inline_data());
    assert_ne!(parent.flags & crate::inode::InodeFlags::INDEX.bits(), 0);
    assert_eq!(parent.flags & (EXT4_CASEFOLD_FL | EXT4_ENCRYPT_FL), 0);
    assert_eq!(fs.sb.inode_size, 256);
    assert!(fs.csum.enabled);
    assert_eq!(parent.size % u64::from(bs), 0);
    let len = (parent.size / u64::from(bs)) as usize;
    assert!((3..=MAX_DIR_BLOCKS).contains(&len));
    let header = crate::extent::ExtentHeader::parse(&parent.block).unwrap();
    assert!(
        header.depth <= 1,
        "deeper extent maps are outside this reference"
    );
    let ctx = crate::extent::ExtentVerifyCtx {
        ino: *ino,
        generation: parent.generation,
        csum: &fs.csum,
    };
    let mapping: Vec<_> = (0..len)
        .map(|logical| {
            crate::extent::map_logical_verified(
                &parent.block,
                fs.dev.as_ref(),
                bs,
                logical as u64,
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
    assert!(nodes.len() <= 4);
    assert_eq!(
        extents.iter().map(|e| usize::from(e.length)).sum::<usize>(),
        len
    );
    let mut physical = HashSet::new();
    for p in mapping.iter().chain(&nodes) {
        assert!(*p < fs.sb.blocks_count && physical.insert(*p));
    }
    let map_nodes: Vec<_> = nodes
        .into_iter()
        .map(|p| {
            let b = fs.read_block(p).unwrap();
            assert!(fs.csum.verify_extent_tail(*ino, parent.generation, &b));
            (p, b)
        })
        .collect();
    assert_ne!(parent.file_acl, 0);
    assert!(!physical.contains(&parent.file_acl));
    assert_eq!(
        parent.blocks,
        (len + map_nodes.len() + 1) as u64 * u64::from(bs / 512)
    );
    let directory: Vec<_> = mapping.iter().map(|p| fs.read_block(*p).unwrap()).collect();
    let root = &directory[0];
    fs.check_dx_block(*ino, parent, root, true).unwrap();
    let info = crate::htree::parse_root_info(root).unwrap();
    assert!(info.indirect_levels <= 1 && info.info_length == 8 && info.unused_flags == 0);
    let (count, routes) = crate::htree::parse_root_entries(root).unwrap();
    assert_eq!(usize::from(count.limit), (bs as usize - 40) / 8);
    let mut seen = HashSet::from([0u32]);
    let mut index_nodes = Vec::new();
    let mut leaves = Vec::new();
    let check_routes = |entries: &[crate::htree::DxEntry]| {
        assert!(entries.windows(2).all(|w| w[0].hash < w[1].hash));
        assert!(
            entries.iter().all(|e| e.hash & 1 == 0),
            "collision boundaries are a separate reference"
        );
    };
    check_routes(&routes);
    for route in &routes {
        if info.indirect_levels == 0 {
            assert!(route.block < len as u32 && seen.insert(route.block));
            leaves.push(route.block);
        } else {
            assert!(route.block < len as u32 && seen.insert(route.block));
            index_nodes.push(route.block);
            let block = &directory[route.block as usize];
            fs.check_dx_block(*ino, parent, block, false).unwrap();
            let (node_count, entries) = crate::htree::parse_node_entries(block).unwrap();
            assert_eq!(usize::from(node_count.limit), (bs as usize - 16) / 8);
            check_routes(&entries);
            for entry in entries {
                assert!(entry.block < len as u32 && seen.insert(entry.block));
                leaves.push(entry.block);
            }
        }
    }
    assert_eq!(
        seen.len(),
        len,
        "every directory block must have exactly one role"
    );
    let mut entries = Vec::new();
    for logical in &leaves {
        let block = &directory[*logical as usize];
        assert!(crate::dir::has_csum_tail(block));
        for e in crate::dir::parse_block_verified(block, true, *ino, parent.generation, &fs.csum)
            .unwrap()
        {
            assert_eq!(e.inode, records[1].0);
            assert_eq!(e.file_type, crate::dir::DirEntryType::RegFile);
            let routed = crate::htree::lookup_leaf_with(
                &e.name,
                root,
                &fs.sb.hash_seed,
                fs.sb.unsigned_hash(),
                |n| Ok(directory[n as usize].clone()),
            )
            .unwrap()
            .unwrap();
            assert_eq!(routed, *logical, "name is not in its hash-routed leaf");
            assert_eq!(fs.lookup_at(*ino, &e.name).unwrap(), e.inode);
            entries.push((e.name, e.inode));
        }
    }
    entries.sort();
    assert!(entries.windows(2).all(|w| w[0].0 != w[1].0));
    assert!(entries.len() <= MAX_NAMES);
    assert_eq!(usize::from(records[1].1.links_count), entries.len() + 1);
    assert_eq!(
        crate::bgd::locate_inode(&fs.sb, &fs.groups, *ino)
            .unwrap()
            .0,
        crate::bgd::locate_inode(&fs.sb, &fs.groups, records[1].0)
            .unwrap()
            .0
    );
    let mut tables = Vec::new();
    for group in &fs.groups {
        let count = (u64::from(fs.sb.inodes_per_group) * u64::from(fs.sb.inode_size))
            .div_ceil(u64::from(bs));
        for block in group.inode_table..group.inode_table + count {
            tables.push((block, fs.read_block(block).unwrap()));
        }
    }
    let eas = records
        .iter()
        .map(|(_, i, _)| {
            assert_ne!(i.file_acl, 0);
            fs.read_block(i.file_acl).unwrap()
        })
        .collect();
    let payloads = records[1..]
        .iter()
        .map(|(_, i, _)| {
            assert_eq!(i.size, 280);
            fs.read_block(fs.map_inode_logical(i, 0).unwrap().unwrap())
                .unwrap()
        })
        .collect();
    let root_inode = fs.read_inode_verified(2).unwrap().0;
    let root_directory = (0..root_inode.size.div_ceil(u64::from(bs)))
        .map(|n| {
            fs.read_block(fs.map_inode_logical(&root_inode, n).unwrap().unwrap())
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
            roots: routes.len(),
            directory,
            root_directory,
        },
        depth: info.indirect_levels,
        limit: count.limit,
        index_nodes,
        leaves,
        map_nodes,
    }
}

fn capture(image: &Path, base: &Path, phase: &str) -> Snapshot {
    // Keep the image before any assertion/report can fail.
    std::fs::copy(image, base.with_extension(format!("{phase}.img"))).unwrap();
    let state = observe(image);
    let out = oracle("e2fsck")
        .arg("-fn")
        .arg(image)
        .judged()
        .clean("Linux root capacity reference");
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
    report(image, base, phase, &state.view.records);
    for (n, (_, _, raw)) in state.view.records.iter().enumerate() {
        std::fs::write(base.with_extension(format!("{phase}.{n}.inode.bin")), raw).unwrap();
    }
    let out = oracle("debugfs")
        .args(["-R", "htree_dump /policy"])
        .arg(image)
        .output();
    std::fs::write(
        base.with_extension(format!("{phase}.htree.stderr.txt")),
        &out.stderr,
    )
    .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("Root node dump") && text.contains("Number of entries"));
    std::fs::write(base.with_extension(format!("{phase}.htree.txt")), text).unwrap();
    let rows: Vec<_> = state
        .view
        .entries
        .iter()
        .map(|(name, ino)| {
            format!(
                "[\"{}\",{ino}]",
                name.iter().map(|b| format!("{b:02x}")).collect::<String>()
            )
        })
        .collect();
    std::fs::write(base.with_extension(format!("{phase}.inventory.json")),format!("{{\"depth\":{},\"root_count\":{},\"root_limit\":{},\"index_nodes\":{:?},\"leaves\":{:?},\"mapping\":{:?},\"extent_nodes\":{:?},\"entries\":[{}]}}\n",state.depth,state.view.roots,state.limit,state.index_nodes,state.leaves,state.view.mapping,state.map_nodes.iter().map(|n|n.0).collect::<Vec<_>>(),rows.join(","))).unwrap();
    state
}

fn transition(before: &Snapshot, after: &Snapshot, image: &Path) {
    let (a, b) = (&before.view, &after.view);
    let fs = mount(image);
    let bs = fs.sb.block_size();
    if before.depth == 0 {
        assert_eq!((before.depth, after.depth), (0, 1));
        assert_eq!(a.roots, usize::from(before.limit));
        assert_eq!(b.roots, 1);
        assert_eq!(after.index_nodes.len(), 1);
    } else {
        node_full::shape(before, after, image);
    }
    assert_eq!(after.leaves.len(), before.leaves.len() + 1);
    assert!(before.leaves.iter().all(|n| after.leaves.contains(n)));
    assert_eq!(b.entries.len(), a.entries.len() + 1);
    assert!(a.entries.iter().all(|e| b.entries.binary_search(e).is_ok()));
    assert_eq!(&b.mapping[..a.mapping.len()], a.mapping);
    assert_eq!(b.mapping.len(), a.mapping.len() + 2);
    let mut added = b.mapping[a.mapping.len()..].to_vec();
    for (p, _) in &before.map_nodes {
        assert!(after.map_nodes.iter().any(|n| n.0 == *p));
    }
    added.extend(
        after
            .map_nodes
            .iter()
            .filter(|n| !before.map_nodes.iter().any(|old| old.0 == n.0))
            .map(|n| n.0),
    );
    assert!((2..=3).contains(&added.len()));
    assert_eq!(b.free, (a.free.0 - added.len() as u64, a.free.1));
    assert_eq!(b.records[0].1.size, a.records[0].1.size + 2 * u64::from(bs));
    assert_eq!(
        b.records[0].1.blocks,
        a.records[0].1.blocks + added.len() as u64 * u64::from(bs / 512)
    );
    assert_eq!(b.records[0].1.flags, a.records[0].1.flags);
    assert_eq!(b.records[0].1.links_count, a.records[0].1.links_count);
    assert_eq!(b.records[1].1.links_count, a.records[1].1.links_count + 1);
    stable(
        &a.records[1].2,
        &b.records[1].2,
        &[(12, 4), (26, 2), (124, 2), (130, 2), (132, 4)],
    );
    stable(
        &a.records[0].2,
        &b.records[0].2,
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
    );
    assert_eq!(a.records[2].2, b.records[2].2);
    assert_eq!(a.eas, b.eas);
    assert_eq!(a.payloads, b.payloads);
    assert_eq!(a.root_directory, b.root_directory);
    let locations: Vec<_> = a.records[..2]
        .iter()
        .map(|(ino, _, _)| crate::bgd::locate_inode(&fs.sb, &fs.groups, *ino).unwrap())
        .collect();
    for ((block, x), (other, y)) in a.tables.iter().zip(&b.tables) {
        assert_eq!(block, other);
        let allowed: Vec<_> = locations
            .iter()
            .filter(|(p, _)| p == block)
            .map(|(_, off)| (*off as usize, usize::from(fs.sb.inode_size)))
            .collect();
        stable(x, y, &allowed);
    }
    for group in 0..fs.groups.len() {
        let row = group * 3;
        let mut bitmap = a.allocation[row + 2].clone();
        let mut debit = 0;
        for p in &added {
            let rel = *p - u64::from(fs.sb.first_data_block);
            if rel / u64::from(fs.sb.blocks_per_group) == group as u64 {
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
        let (block, offset) = fs.sb.descriptor_location(group as u64);
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

fn profile(bs: u32, journal: bool) {
    profile_named(bs, journal, "link-index-capacity");
}

fn profile_named(bs: u32, journal: bool, prefix: &str) {
    let base = fs_ext4_test_support::temp_dir().join(format!("{prefix}-{bs}-{journal}"));
    let image = base.with_extension("working.img");
    let batch = base.with_extension("batch-start.img");
    let replay = base.with_extension("replay.img");
    std::fs::File::create(&image)
        .unwrap()
        .set_len(32 * 1024 * 1024)
        .unwrap();
    let features = if journal {
        "none,extent,64bit,filetype,metadata_csum,dir_index,has_journal"
    } else {
        "none,extent,64bit,filetype,metadata_csum,dir_index"
    };
    let made = oracle("mke2fs")
        .args([
            "-q",
            "-F",
            "-t",
            "ext4",
            "-b",
            &bs.to_string(),
            "-I",
            "256",
            "-m",
            "0",
            "-O",
            features,
            "-E",
            "lazy_itable_init=0,lazy_journal_init=0",
        ])
        .arg(&image)
        .output();
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
    guest(&image, &base, "input-guest", "input", 0, 0, "");
    let input = capture(&image, &base, "input");
    assert_eq!(input.depth, 0);
    let mut total = input.view.entries.len();
    let mut witness = None;
    for round in 0..MAX_NAMES.div_ceil(BATCH) {
        if total == MAX_NAMES {
            break;
        }
        let count = BATCH.min(MAX_NAMES - total);
        std::fs::copy(&image, &batch).unwrap();
        guest(
            &image,
            &base,
            &format!("batch-{round}"),
            "fill",
            total,
            count,
            "",
        );
        let after = probe(&image, &base, &format!("batch-{round}"));
        assert_eq!(after.total, total + count);
        if after.depth == 1 {
            let (mut low, mut high) = (0, count);
            for search in 0..8 {
                if high - low == 1 {
                    break;
                }
                let mid = (low + high) / 2;
                std::fs::copy(&batch, &replay).unwrap();
                guest(
                    &replay,
                    &base,
                    &format!("search-{search}"),
                    "fill",
                    total,
                    mid,
                    "",
                );
                let state = probe(&replay, &base, &format!("search-{search}"));
                assert_eq!(state.total, total + mid);
                if state.depth == 0 {
                    low = mid;
                } else {
                    high = mid;
                }
            }
            assert_eq!(high - low, 1);
            std::fs::copy(&batch, &replay).unwrap();
            if low > 0 {
                guest(&replay, &base, "exact-before-fill", "fill", total, low, "");
            }
            let last = probe(&replay, &base, "exact-before");
            assert_eq!(last.total, total + low);
            assert_eq!((last.depth, last.count), (0, last.limit));
            guest(&replay, &base, "capacity-guest", "audit", last.total, 0, "");
            let before = capture(&replay, &base, "capacity");
            std::fs::copy(&replay, &image).unwrap();
            guest(&image, &base, "trigger", "fill", last.total, 1, "");
            guest(&image, &base, "grown-guest", "audit", last.total + 1, 0, "");
            let grown = capture(&image, &base, "grown");
            transition(&before, &grown, &image);
            witness = Some(grown);
            break;
        }
        total = after.total;
    }
    let grown =
        witness.expect("bounded Linux run did not reach the first root-capacity transition");
    let cold = base.with_extension("cold.img");
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
        grown.view.free.1
    );
    std::fs::write(base.with_extension("cold.expected.py"), &expected).unwrap();
    guest(
        &cold,
        &base,
        "cold-guest",
        "cold",
        grown.view.entries.len(),
        0,
        &expected,
    );
    let observed = capture(&cold, &base, "cold-readback");
    assert_eq!(grown.view.directory, observed.view.directory);
    assert_eq!(grown.view.tables, observed.view.tables);
    assert_eq!(grown.view.allocation, observed.view.allocation);
    assert_eq!(grown.view.eas, observed.view.eas);
    assert_eq!(grown.view.payloads, observed.view.payloads);
    assert_eq!(grown.view.root_directory, observed.view.root_directory);
    assert_eq!(grown.map_nodes, observed.map_nodes);
    eprintln!("Linux root capacity bs={bs} journal={journal}: {} aliases; first root-only to one-intermediate transition, all routes/allocation and cold observer checked; no driver namespace writes",grown.view.entries.len());
    for path in [image, batch, replay, cold] {
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn linux_records_first_root_capacity_transition() {
    for bs in [1024, 4096] {
        for journal in [false, true] {
            profile(bs, journal);
        }
    }
}
