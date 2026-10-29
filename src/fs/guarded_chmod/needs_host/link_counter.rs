//! Linux-only reference; a separate child qualifies driver-emitted boundary bytes.
use super::*;
use std::io::{Seek, SeekFrom, Write};

#[path = "link_counter_emission.rs"]
mod driver_emission;

const GUEST: &str = include_str!("link_counter_guest.py");
const TRACKED: [&[u8]; 3] = [b"/policy", b"/policy/source", b"/peer"];

struct Snapshot {
    records: Vec<(u32, Inode, Vec<u8>)>,
    tables: Vec<(u64, Vec<u8>)>,
    allocation: Vec<Vec<u8>>,
    free: (u64, u32),
    eas: Vec<(u64, Vec<u8>)>,
    entries: Vec<(Vec<u8>, u32, crate::dir::DirEntryType)>,
    root_entries: Vec<(Vec<u8>, u32, crate::dir::DirEntryType)>,
}

fn mount(image: &Path) -> Filesystem {
    Filesystem::mount(Arc::new(FileDevice::open_path(image).unwrap())).unwrap()
}

fn check_clean(fs: &Filesystem) {
    assert_ne!(fs.sb.state & crate::superblock::EXT4_VALID_FS, 0);
    assert_eq!(
        fs.sb.feature_incompat & features::Incompat::RECOVER.bits(),
        0
    );
    if let Some(journal) = crate::jbd2::read_superblock(fs).unwrap() {
        assert!(journal.is_clean());
    }
}

fn judged_reference(image: &Path, base: &Path, phase: &str) {
    let out = oracle("e2fsck")
        .arg("-fn")
        .arg(image)
        .judged()
        .clean("Linux counter reference");
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
}

// This reference deliberately includes legacy and extent-mapped one-block
// directories. Keep its inventory bounded rather than changing public APIs.
fn directory_inventory(fs: &Filesystem, ino: u32) -> Vec<(Vec<u8>, u32, crate::dir::DirEntryType)> {
    let (inode, _) = fs.read_inode_verified(ino).unwrap();
    assert!(inode.is_dir() && !inode.has_inline_data());
    assert_eq!(inode.flags & crate::inode::InodeFlags::INDEX.bits(), 0);
    assert_eq!(inode.size, u64::from(fs.sb.block_size()));
    if inode.has_extents() {
        let header = crate::extent::ExtentHeader::parse(&inode.block).unwrap();
        assert_eq!((header.depth, header.entries), (0, 1));
    } else {
        assert!(
            inode.block[4..].iter().all(|b| *b == 0),
            "only direct block zero belongs to this reference"
        );
    }
    let physical = fs
        .map_inode_logical(&inode, 0)
        .unwrap()
        .expect("reference directory hole");
    assert!(physical < fs.sb.blocks_count);
    assert_eq!(fs.map_inode_logical(&inode, 1).unwrap(), None);
    let block = fs.read_block(physical).unwrap();
    assert_ne!(
        fs.sb.feature_incompat & features::Incompat::FILETYPE.bits(),
        0
    );
    if fs.csum.enabled {
        assert!(
            crate::dir::has_csum_tail(&block),
            "reference checksum tail missing"
        );
    }
    crate::dir::parse_block_verified(&block, true, ino, inode.generation, &fs.csum)
        .unwrap()
        .into_iter()
        .map(|e| (e.name, e.inode, e.file_type))
        .collect()
}

fn snapshot(image: &Path, base: &Path, phase: &str) -> Snapshot {
    let fs = mount(image);
    check_clean(&fs);
    let records: Vec<_> = TRACKED
        .iter()
        .map(|path| {
            let ino = fs.resolve(path).unwrap();
            let (inode, raw) = fs.read_inode_verified(ino).unwrap();
            (ino, inode, raw)
        })
        .collect();
    let mut tables = Vec::new();
    for g in &fs.groups {
        let count = (u64::from(fs.sb.inodes_per_group) * u64::from(fs.sb.inode_size))
            .div_ceil(u64::from(fs.sb.block_size()));
        for block in g.inode_table..g.inode_table + count {
            tables.push((block, fs.read_block(block).unwrap()));
        }
    }
    let eas = records
        .iter()
        .map(|(_, i, _)| {
            assert_ne!(i.file_acl, 0);
            (i.file_acl, fs.read_block(i.file_acl).unwrap())
        })
        .collect();
    let entries = directory_inventory(&fs, records[0].0);
    let root_entries = directory_inventory(&fs, 2);
    let raw_rows = records
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
        .collect::<Vec<_>>();
    std::fs::write(
        base.with_extension(format!("{phase}.raw.json")),
        format!("[{}]\n", raw_rows.join(",")),
    )
    .unwrap();
    report(image, base, phase, &records);
    let result = Snapshot {
        records,
        tables,
        allocation: allocation(&fs),
        free: free(image),
        eas,
        entries,
        root_entries,
    };
    drop(fs);
    judged_reference(image, base, phase);
    result
}

fn guest(image: &Path, base: &Path, mode: &str, expected: &str, rows: usize) -> Vec<u8> {
    let out = guest_kernel_write(image.to_str().unwrap(), &format!("python3 - \"$MNT\" {mode} <<'COUNTER_REFERENCE'\n{expected}\n{GUEST}\nCOUNTER_REFERENCE\n"));
    std::fs::write(base.with_extension(format!("{mode}.jsonl")), &out.stdout).unwrap();
    std::fs::write(
        base.with_extension(format!("{mode}.stderr.txt")),
        &out.stderr,
    )
    .unwrap();
    assert!(
        out.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|line| line.starts_with('{'))
            .count(),
        rows,
        "incomplete {mode} reference output"
    );
    out.stdout
}

fn unchanged(before: &Snapshot, after: &Snapshot, fs: &Filesystem, changed: bool) {
    assert_eq!(before.allocation, after.allocation);
    assert_eq!(before.free, after.free);
    assert_eq!(before.eas, after.eas);
    assert_eq!(before.root_entries, after.root_entries);
    assert_eq!(before.tables.len(), after.tables.len());
    let locations: Vec<_> = before.records[..2]
        .iter()
        .map(|(ino, _, _)| crate::bgd::locate_inode(&fs.sb, &fs.groups, *ino).unwrap())
        .collect();
    for ((block, a), (other, b)) in before.tables.iter().zip(&after.tables) {
        assert_eq!(block, other);
        let allowed: Vec<_> = locations
            .iter()
            .filter(|(nr, _)| changed && nr == block)
            .map(|(_, at)| (*at as usize, usize::from(fs.sb.inode_size)))
            .collect();
        stable(a, b, &allowed);
    }
    for (index, ((ino, old, raw), (other, now, found))) in
        before.records.iter().zip(&after.records).enumerate()
    {
        assert_eq!(ino, other);
        if changed && index == 0 {
            stable(
                raw,
                found,
                &[
                    (12, 4),
                    (16, 4),
                    (36, 4),
                    (124, 2),
                    (130, 2),
                    (132, 4),
                    (136, 4),
                    (152, 4),
                ],
            );
            assert!(now.ctime >= old.ctime && now.mtime >= old.mtime);
        } else if changed && index == 1 {
            assert_eq!(now.links_count, old.links_count + 1);
            assert!(now.ctime >= old.ctime);
            stable(
                raw,
                found,
                &[(12, 4), (26, 2), (124, 2), (130, 2), (132, 4)],
            );
        } else {
            assert_eq!(raw, found);
        }
    }
    let mut wanted = before.entries.clone();
    if changed {
        wanted.push((
            b"alias".to_vec(),
            before.records[1].0,
            crate::dir::DirEntryType::RegFile,
        ));
    }
    wanted.sort_by(|a, b| a.0.cmp(&b.0));
    let mut found = after.entries.clone();
    found.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(wanted, found);
}

#[test]
fn linux_records_link_counter_boundaries() {
    for bs in [1024, 4096] {
        for size in [128, 256] {
            for journal in [false, true] {
                profile(bs, size, journal);
            }
        }
    }
}

fn profile(bs: u32, size: u16, journal: bool) {
    let root = fs_ext4_test_support::temp_dir().join(format!("link-counter-{bs}-{size}-{journal}"));
    let pristine = root.with_extension("pristine.img");
    make(&pristine, bs, size, journal);
    guest(&pristine, &root, "input", "", 1);
    let mut cases = vec![("last-low", u32::MAX - 1, 0), ("carry-low", u32::MAX, 0)];
    if size == 256 {
        cases.extend([
            ("carry-high", u32::MAX, 0x13572468),
            ("wrap-wide", u32::MAX, u32::MAX),
        ]);
    }
    for (case, low, high) in cases {
        let base = fs_ext4_test_support::temp_dir()
            .join(format!("link-counter-{bs}-{size}-{journal}-{case}"));
        let image = base.with_extension("img");
        std::fs::copy(&pristine, &image).unwrap();
        let fs = mount(&image);
        check_clean(&fs);
        assert_eq!(
            (
                fs.sb.block_size(),
                fs.sb.inode_size,
                fs.sb.feature_compat & features::Compat::HAS_JOURNAL.bits() != 0
            ),
            (bs, size, journal)
        );
        assert!(
            fs.journal.is_none(),
            "reference driver reader must be read-only"
        );
        assert_eq!(fs.csum.enabled, size == 256);
        assert_eq!(fs.sb.desc_size, if size == 256 { 64 } else { 32 });
        let parent = fs.resolve(b"/policy").unwrap();
        let (inode, mut raw) = fs.read_inode_verified(parent).unwrap();
        let before_seed = raw.clone();
        assert_eq!(inode.size, u64::from(bs));
        assert!(!inode.has_inline_data());
        assert_eq!(inode.flags & crate::inode::InodeFlags::INDEX.bits(), 0);
        raw[36..40].copy_from_slice(&low.to_le_bytes());
        if size == 256 {
            assert!(u16::from_le_bytes(raw[128..130].try_into().unwrap()) >= 28);
            raw[152..156].copy_from_slice(&high.to_le_bytes());
        }
        // Raw seed setup only. No writable driver mount or driver namespace call.
        fs.csum
            .patch_inode_checksum(parent, inode.generation, &mut raw);
        let mut allowed = vec![(36, 4)];
        if size == 256 {
            allowed.extend([(152, 4), (124, 2), (130, 2)]);
        }
        stable(&before_seed, &raw, &allowed);
        let (block, at) = crate::bgd::locate_inode(&fs.sb, &fs.groups, parent).unwrap();
        let offset = block * u64::from(bs) + u64::from(at);
        drop(fs);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .open(&image)
            .unwrap();
        file.seek(SeekFrom::Start(offset)).unwrap();
        file.write_all(&raw).unwrap();
        file.sync_all().unwrap();
        drop(file);
        let input = snapshot(&image, &base, "input");
        assert_eq!(input.records[0].2, raw);
        assert_eq!(
            version(&input.records[0].2),
            u64::from(low) | (u64::from(high) << 32)
        );
        std::fs::copy(&image, base.with_extension("input.img")).unwrap();
        guest(&image, &base, "refuse", "", 4);
        let refused = snapshot(&image, &base, "refused");
        let fs = mount(&image);
        unchanged(&input, &refused, &fs, false);
        drop(fs);
        std::fs::copy(&image, base.with_extension("refused.img")).unwrap();
        let output = guest(&image, &base, "link", "", 1);
        let text = String::from_utf8(output).unwrap();
        let ok = text.lines().filter(|s| *s == "COUNTER_LINK_OK").count();
        let errors: Vec<_> = text
            .lines()
            .filter_map(|s| s.strip_prefix("COUNTER_LINK_ERR "))
            .collect();
        assert_eq!(ok + errors.len(), 1);
        for error in errors {
            assert!(error.parse::<i32>().unwrap() > 0);
        }
        let created = ok == 1;
        let linked = snapshot(&image, &base, "linked");
        let fs = mount(&image);
        unchanged(&input, &linked, &fs, created);
        drop(fs);
        std::fs::copy(&image, base.with_extension("linked.img")).unwrap();
        let mut rows = TRACKED
            .iter()
            .zip(&linked.records)
            .map(|(p, (ino, inode, _))| literal(p, *ino, inode))
            .collect::<Vec<_>>();
        if created {
            rows.push(literal(
                b"/policy/alias",
                linked.records[1].0,
                &linked.records[1].1,
            ));
        }
        let expected = format!(
            "EXPECTED = [{}]\nFREE = ({},{})\nBS = {bs}\nCREATED = {}\n",
            rows.join(","),
            linked.free.0,
            linked.free.1,
            if created { "True" } else { "False" }
        );
        std::fs::write(base.with_extension("expected.py"), &expected).unwrap();
        std::fs::write(base.with_extension("observation.json"), format!("{{\"block_size\":{bs},\"inode_size\":{size},\"journal\":{journal},\"case\":\"{case}\",\"before\":{},\"after\":{},\"created\":{created}}}\n", version(&input.records[0].2), version(&linked.records[0].2))).unwrap();
        let original = std::fs::read(&image).unwrap();
        let cold = base.with_extension("cold.img");
        std::fs::copy(&image, &cold).unwrap();
        guest(&cold, &base, "cold", &expected, if created { 5 } else { 4 });
        let cold_view = snapshot(&cold, &base, "cold");
        let fs = mount(&cold);
        unchanged(&linked, &cold_view, &fs, false);
        drop(fs);
        assert_eq!(std::fs::read(&image).unwrap(), original);
        eprintln!("Linux counter reference bs={bs} inode={size} journal={journal} case={case}: raw version {} -> {}, created={created}; no driver namespace writes, wrap policy not assumed", version(&input.records[0].2), version(&linked.records[0].2));
        std::fs::remove_file(image).unwrap();
    }
    std::fs::remove_file(pristine).unwrap();
}
