//! Ordinary public-link boundaries compared with the retained Linux reference.
use super::*;

struct Boundary {
    name: &'static str,
    low: u32,
    high: u32,
    after: u64,
}

fn boundaries(size: u16) -> Vec<Boundary> {
    let mut cases = vec![
        Boundary {
            name: "last-low",
            low: u32::MAX - 1,
            high: 0,
            after: u64::from(u32::MAX),
        },
        Boundary {
            name: "carry-low",
            low: u32::MAX,
            high: 0,
            after: if size == 128 { 0 } else { 1u64 << 32 },
        },
    ];
    if size == 256 {
        cases.extend([
            Boundary {
                name: "carry-high",
                low: u32::MAX,
                high: 0x13572468,
                after: 0x1357246900000000,
            },
            Boundary {
                name: "wrap-wide",
                low: u32::MAX,
                high: u32::MAX,
                after: 0,
            },
        ]);
    }
    cases
}

// Read bytes only. This also permits local replay of the actual saved Linux
// inputs without invoking an oracle or mounting an image on the host.
fn observe(fs: &Filesystem, image: &Path) -> Snapshot {
    let records: Vec<_> = TRACKED
        .iter()
        .map(|path| {
            let ino = fs.resolve(path).unwrap();
            let (inode, raw) = fs.read_inode_verified(ino).unwrap();
            (ino, inode, raw)
        })
        .collect();
    let mut tables = Vec::new();
    for group in &fs.groups {
        let count = (u64::from(fs.sb.inodes_per_group) * u64::from(fs.sb.inode_size))
            .div_ceil(u64::from(fs.sb.block_size()));
        for block in group.inode_table..group.inode_table + count {
            tables.push((block, fs.read_block(block).unwrap()));
        }
    }
    let eas = records
        .iter()
        .map(|(_, inode, _)| {
            assert_ne!(inode.file_acl, 0);
            (inode.file_acl, fs.read_block(inode.file_acl).unwrap())
        })
        .collect();
    let entries = directory_inventory(fs, records[0].0);
    let root_entries = directory_inventory(fs, 2);
    Snapshot {
        records,
        tables,
        allocation: allocation(fs),
        free: free(image),
        eas,
        entries,
        root_entries,
    }
}

fn capture(image: &Path, base: &Path, phase: &str, clean: bool) -> Snapshot {
    let fs = mount(image);
    if clean {
        check_clean(&fs);
    } else {
        assert_eq!(fs.sb.state & crate::superblock::EXT4_VALID_FS, 0);
    }
    let result = observe(&fs, image);
    let rows: Vec<_> = result
        .records
        .iter()
        .map(|(ino, _, raw)| {
            let high = if raw.len() >= 156
                && u16::from_le_bytes(raw[128..130].try_into().unwrap()) >= 28
            {
                u32::from_le_bytes(raw[152..156].try_into().unwrap()).to_string()
            } else {
                "null".to_owned()
            };
            format!(
                "{{\"ino\":{ino},\"low\":{},\"high\":{high},\"raw\":\"{}\"}}",
                u32::from_le_bytes(raw[36..40].try_into().unwrap()),
                raw.iter().map(|b| format!("{b:02x}")).collect::<String>()
            )
        })
        .collect();
    std::fs::write(
        base.with_extension(format!("{phase}.raw.json")),
        format!("[{}]\n", rows.join(",")),
    )
    .unwrap();
    report(image, base, phase, &result.records);
    drop(fs);
    if clean {
        judged_reference(image, base, phase);
    }
    result
}

fn seed(image: &Path, boundary: &Boundary, size: u16) {
    let original = std::fs::read(image).unwrap();
    let fs = mount(image);
    check_clean(&fs);
    let ino = fs.resolve(b"/policy").unwrap();
    let (inode, mut raw) = fs.read_inode_verified(ino).unwrap();
    let before = raw.clone();
    assert_eq!(inode.size, u64::from(fs.sb.block_size()));
    raw[36..40].copy_from_slice(&boundary.low.to_le_bytes());
    if size == 256 {
        assert!(u16::from_le_bytes(raw[128..130].try_into().unwrap()) >= 28);
        raw[152..156].copy_from_slice(&boundary.high.to_le_bytes());
    }
    fs.csum
        .patch_inode_checksum(ino, inode.generation, &mut raw);
    let mut allowed = vec![(36, 4)];
    if size == 256 {
        allowed.extend([(152, 4), (124, 2), (130, 2)]);
    }
    stable(&before, &raw, &allowed);
    let (block, at) = crate::bgd::locate_inode(&fs.sb, &fs.groups, ino).unwrap();
    let offset = block * u64::from(fs.sb.block_size()) + u64::from(at);
    drop(fs);
    let mut file = std::fs::OpenOptions::new().write(true).open(image).unwrap();
    file.seek(SeekFrom::Start(offset)).unwrap();
    file.write_all(&raw).unwrap();
    file.sync_all().unwrap();
    drop(file);
    let mut wanted = original;
    wanted[offset as usize..offset as usize + raw.len()].copy_from_slice(&raw);
    assert_eq!(
        std::fs::read(image).unwrap(),
        wanted,
        "seed changed unrelated bytes"
    );
}

fn protected_blocks(fs: &Filesystem, input: &Snapshot) -> Vec<(u64, Vec<u8>)> {
    let mut inos = vec![2];
    inos.extend(input.records[1..].iter().map(|r| r.0));
    inos.into_iter()
        .map(|ino| {
            let inode = fs.read_inode_verified(ino).unwrap().0;
            assert!(!inode.has_inline_data());
            let block = fs.map_inode_logical(&inode, 0).unwrap().unwrap();
            (block, fs.read_block(block).unwrap())
        })
        .collect()
}

// Only driver calls and byte observations; no external tool or guest call.
fn emit(
    image: &Path,
    base: &Path,
    input: &Snapshot,
    boundary: &Boundary,
    capi: bool,
    entry: usize,
    journal: bool,
) -> (Snapshot, Option<fs_ext4_attr_t>, i64) {
    let seeded = std::fs::read(image).unwrap();
    let clock = Arc::new(LinkClock {
        stamp: input.records.iter().map(|r| r.1.ctime).max().unwrap() + 100,
        calls: std::sync::atomic::AtomicUsize::new(0),
    });
    assert!(clock.stamp < i64::from(i32::MAX));
    let fs = Filesystem::mount_with_runtime(
        Arc::new(FileDevice::open_path_rw(image).unwrap()),
        clock.clone(),
    )
    .unwrap();
    assert_eq!(fs.journal.is_some(), journal);
    let parent = InodeRef::new(input.records[0].0, input.records[0].1.generation);
    let target = InodeRef::new(input.records[1].0, input.records[1].1.generation);
    let parent_location = crate::bgd::locate_inode(&fs.sb, &fs.groups, parent.ino).unwrap();
    let target_location = crate::bgd::locate_inode(&fs.sb, &fs.groups, target.ino).unwrap();
    assert_eq!(
        parent_location.0, target_location.0,
        "same-table profile required"
    );
    assert_ne!(parent_location.1, target_location.1);
    let protected = protected_blocks(&fs, input);
    let alias = Alias {
        phase: "counter",
        source: 1,
        parent: 0,
        name: b"alias".to_vec(),
        entry,
    };
    let mut output = None;
    let mut changed = None;
    let fs = crate::capi::with_test_mount(fs, |handle, fs| {
        let status = fs.chmod_operation().unwrap().operation.status();
        for (name, expected) in [
            (b"source".as_slice(), crate::error::errno::EEXIST),
            (b".".as_slice(), crate::error::errno::EEXIST),
            (&[b'x'; 256], crate::error::errno::ENAMETOOLONG),
        ] {
            let path = [b"/policy/".as_slice(), name].concat();
            assert_eq!(
                call(
                    fs,
                    handle,
                    capi,
                    &alias,
                    (target, parent),
                    (b"/policy/source", &path, name)
                )
                .err(),
                Some(expected)
            );
            assert_eq!(std::fs::read(image).unwrap(), seeded);
            assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
            assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        }
        assert_eq!(
            call(
                fs,
                handle,
                capi,
                &alias,
                (parent, parent),
                (b"/policy", b"/policy/alias", b"alias")
            )
            .err(),
            Some(crate::error::errno::EISDIR)
        );
        assert_eq!(std::fs::read(image).unwrap(), seeded);
        assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        let refused = observe(fs, image);
        unchanged(input, &refused, fs, false);
        std::fs::copy(image, base.with_extension("refused.img")).unwrap();

        let attr = call(
            fs,
            handle,
            capi,
            &alias,
            (target, parent),
            (b"/policy/source", b"/policy/alias", b"alias"),
        )
        .unwrap();
        assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            fs.chmod_operation().unwrap().operation.status().health,
            crate::operation::Health::Healthy
        );
        assert_eq!(fs.resolve(b"/policy/alias").unwrap(), target.ino);
        let dirty = observe(fs, image);
        unchanged(input, &dirty, fs, true);
        assert_eq!(version(&dirty.records[0].2), boundary.after);
        assert_eq!(
            (
                dirty.records[0].1.ctime,
                dirty.records[0].1.ctime_nsec,
                dirty.records[0].1.mtime,
                dirty.records[0].1.mtime_nsec
            ),
            (clock.stamp, 0, clock.stamp, 0)
        );
        assert_eq!(
            (dirty.records[1].1.ctime, dirty.records[1].1.ctime_nsec),
            (clock.stamp, 0)
        );
        for (block, bytes) in &protected {
            assert_eq!(fs.read_block(*block).unwrap(), *bytes);
        }
        if let Some(ref a) = attr {
            check_attr(a, target.ino, &dirty.records[1].1);
        }
        output = attr;
        std::fs::copy(image, base.with_extension("dirty.img")).unwrap();
        changed = Some(dirty);
    });
    fs.finish().unwrap();
    assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
    let dirty = changed.unwrap();
    let closed_fs = mount(image);
    check_clean(&closed_fs);
    let closed = observe(&closed_fs, image);
    unchanged(&dirty, &closed, &closed_fs, false);
    for (block, bytes) in protected {
        assert_eq!(closed_fs.read_block(block).unwrap(), bytes);
    }
    std::fs::write(base.with_extension("operation.json"), format!("{{\"source\":{},\"parent\":{},\"shared_table_block\":{},\"refusals\":4,\"clock_calls\":1,\"stamp\":{}}}\n", target.ino, parent.ino, parent_location.0, clock.stamp)).unwrap();
    (closed, output, clock.stamp)
}

#[test]
fn linux_checks_public_link_counter_boundaries() {
    for bs in [1024, 4096] {
        for size in [128, 256] {
            for journal in [false, true] {
                profile(bs, size, journal);
            }
        }
    }
}

fn profile(bs: u32, size: u16, journal: bool) {
    let root =
        fs_ext4_test_support::temp_dir().join(format!("link-counter-driver-{bs}-{size}-{journal}"));
    let pristine = root.with_extension("pristine.img");
    make(&pristine, bs, size, journal);
    guest(&pristine, &root, "input", "", 1);
    let layout = usize::from(bs == 4096) + 2 * usize::from(size == 256) + 4 * usize::from(journal);
    for capi in [false, true] {
        for (index, boundary) in boundaries(size).iter().enumerate() {
            let entry = (layout + index) % if capi { 2 } else { 3 };
            let base = fs_ext4_test_support::temp_dir().join(format!(
                "link-counter-driver-{}-{bs}-{size}-{journal}-{}",
                if capi { "capi" } else { "rust" },
                boundary.name
            ));
            let image = base.with_extension("img");
            std::fs::copy(&pristine, &image).unwrap();
            seed(&image, boundary, size);
            let input = capture(&image, &base, "input", true);
            let fs = mount(&image);
            assert_eq!(
                (
                    fs.sb.block_size(),
                    fs.sb.inode_size,
                    fs.csum.enabled,
                    fs.sb.desc_size
                ),
                (bs, size, size == 256, if size == 256 { 64 } else { 32 })
            );
            assert_eq!(
                fs.sb.feature_compat & features::Compat::HAS_JOURNAL.bits() != 0,
                journal
            );
            assert!(fs.journal.is_none());
            drop(fs);
            assert_eq!(
                version(&input.records[0].2),
                u64::from(boundary.low) | u64::from(boundary.high) << 32
            );
            std::fs::copy(&image, base.with_extension("input.img")).unwrap();
            let (closed, attr, stamp) = emit(&image, &base, &input, boundary, capi, entry, journal);
            let refused = capture(&base.with_extension("refused.img"), &base, "refused", true);
            let dirty = capture(&base.with_extension("dirty.img"), &base, "dirty", false);
            std::fs::copy(&image, base.with_extension("closed.img")).unwrap();
            let reported = capture(&image, &base, "closed", true);
            let fs = mount(&image);
            unchanged(&input, &refused, &fs, false);
            unchanged(&dirty, &closed, &fs, false);
            unchanged(&closed, &reported, &fs, false);
            drop(fs);
            let mut rows = TRACKED
                .iter()
                .zip(&closed.records)
                .map(|(p, (ino, inode, _))| literal(p, *ino, inode))
                .collect::<Vec<_>>();
            rows.push(literal(
                b"/policy/alias",
                closed.records[1].0,
                &closed.records[1].1,
            ));
            let expected = format!(
                "EXPECTED = [{}]\nFREE = ({},{})\nBS = {bs}\nCREATED = True\n",
                rows.join(","),
                closed.free.0,
                closed.free.1
            );
            std::fs::write(base.with_extension("expected.py"), &expected).unwrap();
            if let Some(a) = attr {
                let source = &closed.records[1];
                std::fs::write(base.with_extension("c-attrs.py"), format!("ATTRS = [{}]\nGENERATION = {}\nFLAGS = {}\nCRTIME = ({},{})\nFILE_TYPE = {}\n", attr_literal(b"/policy/source", source.1.mode & crate::inode::S_IFMT, &a), a.generation, a.inode_flags, a.crtime, a.crtime_nsec, a.file_type as u32)).unwrap();
            }
            std::fs::write(base.with_extension("observation.json"), format!("{{\"block_size\":{bs},\"inode_size\":{size},\"journal\":{journal},\"capi\":{capi},\"entry\":{entry},\"case\":\"{}\",\"before\":{},\"after\":{},\"stamp\":{stamp}}}\n", boundary.name, version(&input.records[0].2), version(&closed.records[0].2))).unwrap();
            let original = std::fs::read(&image).unwrap();
            let cold = base.with_extension("cold.img");
            std::fs::copy(&image, &cold).unwrap();
            guest(&cold, &base, "cold", &expected, 5);
            let cold_view = capture(&cold, &base, "cold", true);
            let fs = mount(&cold);
            unchanged(&closed, &cold_view, &fs, false);
            drop(fs);
            assert_eq!(std::fs::read(&image).unwrap(), original);
            eprintln!("driver counter capi={capi} entry={entry} bs={bs} inode={size} journal={journal} case={}: {} -> {}; one link, four zero-effect refusals", boundary.name, version(&input.records[0].2), version(&closed.records[0].2));
            std::fs::remove_file(image).unwrap();
        }
    }
    std::fs::remove_file(pristine).unwrap();
}
