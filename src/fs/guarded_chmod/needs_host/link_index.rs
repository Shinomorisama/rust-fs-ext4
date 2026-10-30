//! Linux-only root-index link reference. No writable driver mount or mutation.
use super::*;

const GUEST: &str = include_str!("link_index_guest.py");
const TRACKED: [&[u8]; 3] = [b"/policy", b"/source", b"/peer"];

struct View {
    records: Vec<(u32, Inode, Vec<u8>)>,
    tables: Vec<(u64, Vec<u8>)>,
    allocation: Vec<Vec<u8>>,
    eas: Vec<Vec<u8>>,
    payloads: Vec<Vec<u8>>,
    free: (u64, u32),
    mapping: Vec<u64>,
    entries: Vec<(Vec<u8>, u32)>,
    roots: usize,
    directory: Vec<Vec<u8>>,
    root_directory: Vec<Vec<u8>>,
}

fn mount(image: &Path) -> Filesystem {
    Filesystem::mount(Arc::new(FileDevice::open_path(image).unwrap())).unwrap()
}

fn guest(image: &Path, base: &Path, mode: &str, argument: usize, expected: &str) -> String {
    let out = guest_kernel_write(image.to_str().unwrap(), &format!(
        "python3 - \"$MNT\" {mode} {argument} <<'INDEX_REFERENCE'\n{expected}\n{GUEST}\nINDEX_REFERENCE\n"));
    std::fs::write(
        base.with_extension(format!("{mode}-{argument}.jsonl")),
        &out.stdout,
    )
    .unwrap();
    std::fs::write(
        base.with_extension(format!("{mode}-{argument}.stderr.txt")),
        &out.stderr,
    )
    .unwrap();
    assert!(
        out.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8(out.stdout).unwrap();
    assert_eq!(text.lines().filter(|s| s.starts_with('{')).count(), 1);
    text.trim().to_owned()
}

fn view(image: &Path) -> View {
    let fs = mount(image);
    assert!(
        fs.journal.is_none(),
        "reader must not acquire a writable journal"
    );
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
        .map(|path| {
            let ino = fs.resolve(path).unwrap();
            let (inode, raw) = fs.read_inode_verified(ino).unwrap();
            (ino, inode, raw)
        })
        .collect();
    let (parent_ino, parent, _) = &records[0];
    assert!(parent.has_extents() && !parent.has_inline_data());
    assert_ne!(parent.flags & crate::inode::InodeFlags::INDEX.bits(), 0);
    assert_eq!(parent.flags & (EXT4_CASEFOLD_FL | EXT4_ENCRYPT_FL), 0);
    let header = crate::extent::ExtentHeader::parse(&parent.block).unwrap();
    assert_eq!(header.depth, 0, "external extent maps are a later matrix");
    let bs = fs.sb.block_size();
    assert_eq!(fs.sb.inode_size, 256);
    assert!(fs.csum.enabled);
    assert_eq!(parent.size % u64::from(bs), 0);
    assert!((3..=8).contains(&(parent.size / u64::from(bs))));
    let mapping: Vec<_> = (0..parent.size / u64::from(bs))
        .map(|logical| fs.map_inode_logical(parent, logical).unwrap().unwrap())
        .collect();
    assert_eq!(
        mapping
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        mapping.len()
    );
    assert_eq!(
        fs.map_inode_logical(parent, mapping.len() as u64).unwrap(),
        None
    );
    let root = fs.read_block(mapping[0]).unwrap();
    fs.check_dx_block(*parent_ino, parent, &root, true).unwrap();
    let info = crate::htree::parse_root_info(&root).unwrap();
    assert_eq!(
        (info.indirect_levels, info.info_length, info.unused_flags),
        (0, 8, 0)
    );
    let (count, routes) = crate::htree::parse_root_entries(&root).unwrap();
    assert!(
        count.count < count.limit,
        "full root/fallback is outside this reference"
    );
    let mut routed: Vec<_> = routes.iter().map(|e| e.block).collect();
    routed.sort_unstable();
    assert_eq!(routed, (1..mapping.len() as u32).collect::<Vec<_>>());
    let mut entries = Vec::new();
    for (logical, physical) in mapping.iter().enumerate().skip(1) {
        let block = fs.read_block(*physical).unwrap();
        assert!(crate::dir::has_csum_tail(&block));
        for e in
            crate::dir::parse_block_verified(&block, true, *parent_ino, parent.generation, &fs.csum)
                .unwrap()
        {
            assert_eq!(e.inode, records[1].0);
            assert_eq!(e.file_type, crate::dir::DirEntryType::RegFile);
            let routed = crate::htree::lookup_leaf_with(
                &e.name,
                &root,
                &fs.sb.hash_seed,
                fs.sb.unsigned_hash(),
                |_| panic!("root-only input"),
            )
            .unwrap()
            .unwrap();
            assert_eq!(
                routed as usize, logical,
                "entry is not in its hash-routed leaf"
            );
            assert_eq!(fs.lookup_at(*parent_ino, &e.name).unwrap(), records[1].0);
            entries.push((e.name, e.inode));
        }
    }
    entries.sort();
    assert!(entries.windows(2).all(|w| w[0].0 != w[1].0));
    assert_eq!(usize::from(records[1].1.links_count), entries.len() + 1);
    let source_location = crate::bgd::locate_inode(&fs.sb, &fs.groups, records[1].0).unwrap();
    assert_eq!(
        crate::bgd::locate_inode(&fs.sb, &fs.groups, *parent_ino)
            .unwrap()
            .0,
        source_location.0,
        "same-table staging control must be real"
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
            assert!(!mapping.contains(&i.file_acl));
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
    let directory = mapping.iter().map(|p| fs.read_block(*p).unwrap()).collect();
    let root_inode = fs.read_inode_verified(2).unwrap().0;
    let root_directory = (0..root_inode.size.div_ceil(u64::from(bs)))
        .map(|logical| {
            fs.read_block(fs.map_inode_logical(&root_inode, logical).unwrap().unwrap())
                .unwrap()
        })
        .collect();
    View {
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
    }
}

fn saved(image: &Path, base: &Path, phase: &str) -> View {
    let state = view(image);
    let out = oracle("e2fsck")
        .arg("-fn")
        .arg(image)
        .judged()
        .clean("indexed Linux link reference");
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
    report(image, base, phase, &state.records);
    for (index, (_, _, raw)) in state.records.iter().enumerate() {
        std::fs::write(
            base.with_extension(format!("{phase}.{index}.inode.bin")),
            raw,
        )
        .unwrap();
    }
    let dx = oracle("debugfs")
        .args(["-R", "htree_dump /policy"])
        .arg(image)
        .output();
    assert!(dx.status.success());
    let text = String::from_utf8(dx.stdout).unwrap();
    assert!(
        text.contains("Root node dump") && text.contains("Number of entries"),
        "not an htree report: {text}"
    );
    std::fs::write(base.with_extension(format!("{phase}.htree.txt")), text).unwrap();
    std::fs::write(
        base.with_extension(format!("{phase}.htree.stderr.txt")),
        dx.stderr,
    )
    .unwrap();
    let rows: Vec<_> = state
        .entries
        .iter()
        .map(|(n, ino)| {
            format!(
                "[\"{}\",{ino}]",
                n.iter().map(|b| format!("{b:02x}")).collect::<String>()
            )
        })
        .collect();
    std::fs::write(base.with_extension(format!("{phase}.inventory.json")), format!("{{\"flags\":{},\"size\":{},\"blocks\":{},\"root_entries\":{},\"mapping\":{:?},\"entries\":[{}]}}\n", state.records[0].1.flags, state.records[0].1.size, state.records[0].1.blocks, state.roots, state.mapping, rows.join(","))).unwrap();
    std::fs::copy(image, base.with_extension(format!("{phase}.img"))).unwrap();
    state
}

fn transition(before: &View, after: &View, image: &Path, growth: bool) {
    let fs = mount(image);
    let bs = fs.sb.block_size();
    assert_eq!(before.free.1, after.free.1);
    assert_eq!(before.free.0 - u64::from(growth), after.free.0);
    assert_eq!(before.eas, after.eas);
    assert_eq!(before.payloads, after.payloads);
    assert_eq!(before.root_directory, after.root_directory);
    assert_eq!(before.records[2].2, after.records[2].2);
    assert_eq!(after.entries.len(), before.entries.len() + 1);
    assert!(before.entries.iter().all(|e| after.entries.contains(e)));
    assert_eq!(
        after.records[1].1.links_count,
        before.records[1].1.links_count + 1
    );
    stable(
        &before.records[1].2,
        &after.records[1].2,
        &[(12, 4), (26, 2), (124, 2), (130, 2), (132, 4)],
    );
    let (old, now) = (&before.records[0].1, &after.records[0].1);
    assert_eq!(now.flags, old.flags);
    assert_eq!(now.links_count, old.links_count);
    assert_eq!(now.size, old.size + u64::from(growth) * u64::from(bs));
    assert_eq!(
        now.blocks,
        old.blocks + u64::from(growth) * u64::from(bs / 512)
    );
    assert_eq!(after.roots, before.roots + usize::from(growth));
    assert_eq!(&after.mapping[..before.mapping.len()], before.mapping);
    stable(
        &before.records[0].2,
        &after.records[0].2,
        &[
            (4, 4),
            (12, 8),
            (28, 4),
            (36, 4),
            (40, 60),
            (108, 4),
            (124, 2),
            (130, 2),
            (132, 8),
            (152, 4),
        ],
    );
    let locations: Vec<_> = before.records[..2]
        .iter()
        .map(|(ino, _, _)| crate::bgd::locate_inode(&fs.sb, &fs.groups, *ino).unwrap())
        .collect();
    for ((block, a), (other, b)) in before.tables.iter().zip(&after.tables) {
        assert_eq!(block, other);
        let allowed: Vec<_> = locations
            .iter()
            .filter(|(nr, _)| nr == block)
            .map(|(_, at)| (*at as usize, usize::from(fs.sb.inode_size)))
            .collect();
        stable(a, b, &allowed);
    }
    if !growth {
        assert_eq!(before.allocation, after.allocation);
        return;
    }
    let added = *after.mapping.last().unwrap();
    let relative = added - u64::from(fs.sb.first_data_block);
    let changed = (relative / u64::from(fs.sb.blocks_per_group)) as usize;
    let bit = relative % u64::from(fs.sb.blocks_per_group);
    let (desc_block, offset) = fs.sb.descriptor_location(changed as u64);
    for index in 0..fs.groups.len() {
        let row = index * 3;
        let (block, at) = fs.sb.descriptor_location(index as u64);
        let desc_size = usize::from(fs.sb.desc_size);
        let old = crate::bgd::BlockGroupDescriptor::parse(
            &before.allocation[row][at..at + desc_size],
            fs.sb.desc_size,
        )
        .unwrap();
        let new = crate::bgd::BlockGroupDescriptor::parse(
            &after.allocation[row][at..at + desc_size],
            fs.sb.desc_size,
        )
        .unwrap();
        assert_eq!(
            old.free_blocks_count - u32::from(index == changed),
            new.free_blocks_count
        );
        let allowed = if block == desc_block {
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
        stable(&before.allocation[row], &after.allocation[row], &allowed);
        assert_eq!(before.allocation[row + 1], after.allocation[row + 1]);
        let mut bitmap = before.allocation[row + 2].clone();
        if index == changed {
            let byte = (bit / 8) as usize;
            let mask = 1u8 << (bit % 8);
            assert_eq!(bitmap[byte] & mask, 0);
            bitmap[byte] |= mask;
        }
        assert_eq!(bitmap, after.allocation[row + 2]);
    }
}

#[test]
fn linux_records_ordinary_indexed_link_growth() {
    for bs in [1024, 4096] {
        for journal in [false, true] {
            profile(bs, journal);
        }
    }
}

fn profile(bs: u32, journal: bool) {
    let base = fs_ext4_test_support::temp_dir().join(format!("link-index-{bs}-{journal}"));
    let image = base.with_extension("img");
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
    guest(&image, &base, "input", 0, "");
    let input = saved(&image, &base, "input");
    guest(&image, &base, "refuse", 0, "");
    let refused = view(&image);
    assert_eq!(
        input.records.iter().map(|r| &r.2).collect::<Vec<_>>(),
        refused.records.iter().map(|r| &r.2).collect::<Vec<_>>()
    );
    assert_eq!(input.tables, refused.tables);
    assert_eq!(input.allocation, refused.allocation);
    assert_eq!(input.entries, refused.entries);
    assert_eq!(input.directory, refused.directory);
    assert_eq!(input.root_directory, refused.root_directory);
    assert_eq!(input.eas, refused.eas);
    assert_eq!(input.payloads, refused.payloads);
    assert_eq!(input.free, refused.free);
    guest(&image, &base, "inplace", 0, "");
    let inplace = saved(&image, &base, "inplace");
    transition(&input, &inplace, &image, false);
    let mut before = inplace;
    let mut final_json = None;
    for n in 0..64 {
        std::fs::copy(&image, base.with_extension("before-growth.img")).unwrap();
        let json = guest(&image, &base, "step", n, "");
        let after = view(&image);
        let grew = after.records[0].1.size > before.records[0].1.size;
        transition(&before, &after, &image, grew);
        if grew {
            let _ = saved(
                &base.with_extension("before-growth.img"),
                &base,
                "growth-input",
            );
            let _ = saved(&image, &base, "grown");
            final_json = Some(json);
            break;
        }
        before = after;
    }
    let json = final_json.expect("bounded Linux reference never split a leaf");
    let cold = base.with_extension("cold.img");
    std::fs::copy(&image, &cold).unwrap();
    let expected = format!("import json\nEXPECTED = json.loads(r'''{json}''')['after']\n");
    guest(&cold, &base, "cold", 0, &expected);
    let cold_view = saved(&cold, &base, "cold-readback");
    let final_view = view(&image);
    assert_eq!(cold_view.directory, final_view.directory);
    assert_eq!(cold_view.tables, final_view.tables);
    assert_eq!(cold_view.allocation, final_view.allocation);
    assert_eq!(cold_view.eas, final_view.eas);
    assert_eq!(cold_view.payloads, final_view.payloads);
    assert_eq!(cold_view.root_directory, final_view.root_directory);
    eprintln!("Linux indexed link bs={bs} inode=256 journal={journal}: root-only in-place and one-leaf retained-index split, complete routing/inventory/allocation and cold readback; no driver namespace writes");
    std::fs::remove_file(image).unwrap();
}
