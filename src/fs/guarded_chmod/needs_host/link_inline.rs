//! Independent hard links that retain, convert, then reuse an inline parent.
use super::*;

const GUEST: &str = include_str!("link_inline_guest.py");

fn make_inline(image: &Path, bs: u32, journal: bool) {
    std::fs::File::create(image)
        .unwrap()
        .set_len(32 * 1024 * 1024)
        .unwrap();
    let mut features = "none,extent,64bit,filetype,metadata_csum,inline_data".to_owned();
    if journal {
        features.push_str(",has_journal");
    }
    let out = oracle("mke2fs")
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
            &features,
            "-E",
            "lazy_itable_init=0,lazy_journal_init=0",
        ])
        .arg(image)
        .output();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn paths(fs: &Filesystem) -> Vec<Vec<u8>> {
    let mut result = vec![b"/".to_vec()];
    let mut at = 0;
    let mut dirs = std::collections::BTreeSet::new();
    while at < result.len() {
        assert!(result.len() < 128, "bounded inline input inventory");
        let path = result[at].clone();
        at += 1;
        let ino = fs.resolve(&path).unwrap();
        if fs.stat_ino(ino).unwrap().is_dir() {
            assert!(dirs.insert(ino), "directory alias in controlled fixture");
            for entry in fs.read_dir_ino(ino).unwrap() {
                if entry.name == b"." || entry.name == b".." {
                    continue;
                }
                let mut child = path.clone();
                if child != b"/" {
                    child.push(b'/');
                }
                child.extend(entry.name);
                result.push(child);
            }
        }
    }
    result
}
fn pybytes(bytes: &[u8]) -> String {
    format!(
        "bytes.fromhex('{}')",
        bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
    )
}
fn attributes(fs: &Filesystem, ino: u32, name: &str) -> Option<Vec<u8>> {
    let (inode, raw) = fs.read_inode_verified(ino).unwrap();
    crate::xattr::get(
        fs.dev.as_ref(),
        &inode,
        &raw,
        fs.sb.inode_size,
        fs.sb.block_size(),
        name,
    )
    .unwrap()
}
fn inline_shape(fs: &Filesystem, ino: u32, continued: bool) {
    let (i, raw) = fs.read_inode_verified(ino).unwrap();
    let bs = fs.sb.block_size();
    assert!(i.has_inline_data());
    assert!(!i.has_extents());
    assert_eq!(i.flags & crate::inode::InodeFlags::INDEX.bits(), 0);
    assert_eq!(i.size > 60, continued);
    assert!(i.size < u64::from(bs));
    assert_ne!(
        fs.sb.feature_incompat & features::Incompat::INLINE_DATA.bits(),
        0
    );
    assert_eq!(fs.sb.inode_size, 256);
    assert!(u16::from_le_bytes(raw[128..130].try_into().unwrap()) > 0);
    let mut body = i.clone();
    body.file_acl = 0;
    assert_eq!(
        crate::xattr::get(fs.dev.as_ref(), &body, &raw, 256, bs, "user.keep").unwrap(),
        Some(b"k".to_vec())
    );
    let tail = crate::xattr::get(fs.dev.as_ref(), &body, &raw, 256, bs, "system.data")
        .unwrap()
        .expect("inline body system.data");
    assert_eq!(tail.len() as u64, i.size - 60);
    let mut dir = crate::inline_mut::InlineDir::load(fs.dev.as_ref(), &i, &raw, 256, bs).unwrap();
    assert!(!dir
        .add(123, &[b'n'; 255], crate::dir::DirEntryType::RegFile, true)
        .unwrap());
    if dir.expand(&raw, 256) {
        assert!(!dir
            .add(123, &[b'n'; 255], crate::dir::DirEntryType::RegFile, true)
            .unwrap());
    }
    let mut fit = crate::inline_mut::InlineDir::load(fs.dev.as_ref(), &i, &raw, 256, bs).unwrap();
    assert!(
        fit.add(123, b"fit", crate::dir::DirEntryType::RegFile, true)
            .unwrap(),
        "input must admit fit without expansion"
    );
    assert_eq!(u32::from_le_bytes(raw[40..44].try_into().unwrap()), 2);
}

fn accounting(
    fs: &Filesystem,
    initial: &[Vec<u8>],
    old_free: (u64, u32),
    image: &Path,
    added: Option<u64>,
) {
    let after = allocation(fs);
    assert_eq!(
        free(image),
        (old_free.0 - u64::from(added.is_some()), old_free.1)
    );
    let group = added.map(|p| {
        ((p - u64::from(fs.sb.first_data_block)) / u64::from(fs.sb.blocks_per_group)) as usize
    });
    let selected = group.map(|g| fs.sb.descriptor_location(g as u64));
    for (g, desc) in fs.groups.iter().enumerate() {
        let row = g * 3;
        let (block, at) = fs.sb.descriptor_location(g as u64);
        let mut allowed = Vec::new();
        if let Some((changed, offset)) = selected {
            if changed == block {
                allowed.extend([
                    (offset + 12, 2),
                    (offset + 24, 2),
                    (offset + 30, 2),
                    (offset + 44, 2),
                    (offset + 56, 2),
                ]);
            }
        }
        stable(&initial[row], &after[row], &allowed);
        let old = crate::bgd::BlockGroupDescriptor::parse(&initial[row][at..at + 64], 64).unwrap();
        let now = crate::bgd::BlockGroupDescriptor::parse(&after[row][at..at + 64], 64).unwrap();
        assert_eq!(
            now.free_blocks_count,
            old.free_blocks_count - u32::from(group == Some(g))
        );
        assert_eq!(initial[row + 1], after[row + 1]);
        let mut wanted = initial[row + 2].clone();
        if group == Some(g) {
            let bit = (added.unwrap() - u64::from(fs.sb.first_data_block))
                % u64::from(fs.sb.blocks_per_group);
            let mask = 1u8 << (bit % 8);
            assert_eq!(wanted[bit as usize / 8] & mask, 0);
            wanted[bit as usize / 8] |= mask;
        }
        assert_eq!(wanted, after[row + 2], "unrelated block bitmap bit");
        // Retain the identity used to interpret the complete rows.
        assert!(desc.inode_bitmap != desc.block_bitmap);
    }
}

#[test]
fn linux_checks_public_link_inline_conversion() {
    for capi in [false, true] {
        for bs in [1024, 4096] {
            for journal in [false, true] {
                for continued in [false, true] {
                    profile(capi, bs, journal, continued);
                }
            }
        }
    }
}

fn profile(capi: bool, bs: u32, journal: bool, continued: bool) {
    let base = fs_ext4_test_support::temp_dir().join(format!(
        "link-inline-{}-{bs}-{journal}-{continued}",
        if capi { "capi" } else { "rust" }
    ));
    let image = base.with_extension("img");
    make_inline(&image, bs, journal);
    let fs = Filesystem::mount(Arc::new(FileDevice::open_path(&image).unwrap())).unwrap();
    let ipg = fs.sb.inodes_per_group;
    drop(fs);
    let out = guest_kernel_write(
        image.to_str().unwrap(),
        &format!(
            "python3 - \"$MNT\" input {bs} {ipg} {} <<'INLINE_INPUT'\n{GUEST}\nINLINE_INPUT\n",
            u8::from(continued)
        ),
    );
    std::fs::write(base.with_extension("input.jsonl"), &out.stdout).unwrap();
    std::fs::write(base.with_extension("input.stderr.txt"), &out.stderr).unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    judged(&image, &base, "input");
    std::fs::copy(&image, base.with_extension("input.img")).unwrap();
    let fs = Filesystem::mount(Arc::new(FileDevice::open_path(&image).unwrap())).unwrap();
    assert!(fs.csum.enabled);
    assert_eq!(fs.sb.desc_size, 64);
    let parent = fs.resolve(b"/policy").unwrap();
    let target = fs.resolve(b"/policy/source").unwrap();
    inline_shape(&fs, parent, continued);
    let parent_location = crate::bgd::locate_inode(&fs.sb, &fs.groups, parent).unwrap();
    let target_location = crate::bgd::locate_inode(&fs.sb, &fs.groups, target).unwrap();
    assert_eq!(
        parent_location.0, target_location.0,
        "actual same-table input"
    );
    let mut names = paths(&fs);
    let mut records: Vec<_> = names
        .iter()
        .map(|p| {
            let n = fs.resolve(p).unwrap();
            let (i, r) = fs.read_inode_verified(n).unwrap();
            (n, i, r)
        })
        .collect();
    let directories: Vec<_> = names
        .iter()
        .filter(|p| fs.stat_ino(fs.resolve(p).unwrap()).unwrap().is_dir())
        .map(|p| {
            let entries = fs
                .read_dir_ino(fs.resolve(p).unwrap())
                .unwrap()
                .into_iter()
                .filter(|e| e.name != b"." && e.name != b"..")
                .map(|e| e.name)
                .collect::<Vec<_>>();
            (p.clone(), entries)
        })
        .collect();
    let initial_parent = fs.stat_ino(parent).unwrap();
    let initial_alloc = allocation(&fs);
    let initial_free = free(&image);
    let eas: Vec<_> = [b"/policy".as_slice(), b"/policy/source", b"/peer"]
        .into_iter()
        .map(|p| {
            let i = fs.stat_ino(fs.resolve(p).unwrap()).unwrap();
            assert_ne!(i.file_acl, 0);
            (i.file_acl, fs.read_block(i.file_acl).unwrap())
        })
        .collect();
    let first = records.iter().map(|r| r.1.ctime).max().unwrap() + 100;
    report(&image, &base, "input", &records);
    drop(fs);
    let mut added = None;
    for (index, (phase, name)) in [
        ("fit", b"fit".to_vec()),
        ("convert", vec![b'n'; 255]),
        ("post", b"post".to_vec()),
    ]
    .into_iter()
    .enumerate()
    {
        let clock = Arc::new(LinkClock {
            stamp: first + index as i64 * 100,
            calls: std::sync::atomic::AtomicUsize::new(0),
        });
        let fs = Filesystem::mount_with_runtime(
            Arc::new(FileDevice::open_path_rw(&image).unwrap()),
            clock.clone(),
        )
        .unwrap();
        assert_eq!(fs.journal.is_some(), journal);
        let alias = Alias {
            phase,
            source: 0,
            parent: 0,
            name: name.clone(),
            entry: (usize::from(bs == 4096) + 2 * usize::from(journal) + usize::from(continued))
                % if capi { 2 } else { 3 },
        };
        let t = InodeRef::new(target, fs.stat_ino(target).unwrap().generation);
        let p = InodeRef::new(parent, fs.stat_ino(parent).unwrap().generation);
        let destination = [b"/policy/".as_slice(), &name].concat();
        let mut attrs = Vec::new();
        let fs = crate::capi::with_test_mount(fs, |h, fs| {
            let before = std::fs::read(&image).unwrap();
            for (bad, errno) in [
                (b"source".as_slice(), crate::error::errno::EEXIST),
                (b".", crate::error::errno::EEXIST),
                (&[b'x'; 256], crate::error::errno::ENAMETOOLONG),
            ] {
                let dst = [b"/policy/".as_slice(), bad].concat();
                assert_eq!(
                    call(fs, h, capi, &alias, (t, p), (b"/policy/source", &dst, bad)).err(),
                    Some(errno)
                );
                assert_eq!(std::fs::read(&image).unwrap(), before);
            }
            assert_eq!(
                call(
                    fs,
                    h,
                    capi,
                    &alias,
                    (p, p),
                    (b"/policy", &destination, &name)
                )
                .err(),
                Some(crate::error::errno::EISDIR)
            );
            assert_eq!(std::fs::read(&image).unwrap(), before);
            assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
            let attr = call(
                fs,
                h,
                capi,
                &alias,
                (t, p),
                (b"/policy/source", &destination, &name),
            )
            .unwrap();
            assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
            let mut current = Vec::new();
            for (ino, old, raw) in &records {
                let (now, found) = fs.read_inode_verified(*ino).unwrap();
                if *ino == target {
                    assert_eq!(now.links_count, old.links_count + 1);
                    assert_eq!((now.ctime, now.ctime_nsec), (clock.stamp, 0));
                    stable(
                        raw,
                        &found,
                        &[(12, 4), (26, 2), (124, 2), (130, 2), (132, 4)],
                    );
                } else if *ino == parent {
                    assert_eq!(
                        (now.ctime, now.ctime_nsec, now.mtime, now.mtime_nsec),
                        (clock.stamp, 0, clock.stamp, 0)
                    );
                    assert_eq!(version(&found), version(raw) + 1);
                    let mut allowed = vec![
                        (12, 4),
                        (16, 4),
                        (36, 4),
                        (124, 2),
                        (130, 2),
                        (132, 4),
                        (136, 4),
                        (152, 4),
                    ];
                    if index < 2 {
                        allowed.extend([(40, 60), (160, 96)]);
                    }
                    if index == 1 {
                        allowed.extend([(4, 4), (28, 4), (32, 4), (108, 4), (116, 2)]);
                    }
                    stable(raw, &found, &allowed);
                    assert_eq!(now.has_inline_data(), index == 0);
                    assert_eq!(
                        now.blocks,
                        initial_parent.blocks + u64::from(index > 0) * u64::from(bs / 512)
                    );
                    if index > 0 {
                        assert_eq!(
                            now.flags,
                            (initial_parent.flags & !crate::inode::InodeFlags::INLINE_DATA.bits())
                                | crate::inode::InodeFlags::EXTENTS.bits()
                        );
                        assert_eq!(now.size, u64::from(bs));
                        assert_eq!(fs.map_inode_logical(&now, 1).unwrap(), None);
                        let block = fs.map_inode_logical(&now, 0).unwrap().unwrap();
                        if let Some(prior) = added {
                            assert_eq!(block, prior);
                        } else {
                            added = Some(block);
                        }
                    } else if continued {
                        assert_eq!(
                            &raw[40..100],
                            &found[40..100],
                            "fit belongs in continuation"
                        );
                    }
                } else {
                    assert_eq!(raw, &found, "protected inode changed");
                }
                current.push((*ino, now, found));
            }
            records = current;
            assert_eq!(attributes(fs, parent, "user.keep"), Some(b"k".to_vec()));
            if index > 0 {
                assert_eq!(attributes(fs, parent, "system.data"), None);
            }
            for (nr, raw) in &eas {
                assert_eq!(fs.read_block(*nr).unwrap(), *raw);
            }
            accounting(fs, &initial_alloc, initial_free, &image, added);
            // The old source/parent table may contain unrelated adjacent inodes.
            let input = std::fs::read(base.with_extension("input.img")).unwrap();
            let start = parent_location.0 as usize * bs as usize;
            stable(
                &input[start..start + bs as usize],
                &fs.read_block(parent_location.0).unwrap(),
                &[
                    (parent_location.1 as usize, 256),
                    (target_location.1 as usize, 256),
                ],
            );
            if let Some(attr) = attr {
                let inode = fs.stat_ino(target).unwrap();
                check_attr(&attr, target, &inode);
                attrs.push(attr_literal(
                    b"/policy/source",
                    inode.mode & crate::inode::S_IFMT,
                    &attr,
                ));
            }
            let bytes = std::fs::read(&image).unwrap();
            assert_eq!(
                u16::from_le_bytes(bytes[1082..1084].try_into().unwrap())
                    & crate::superblock::EXT4_VALID_FS,
                0
            );
            assert_eq!(
                u32::from_le_bytes(bytes[2044..2048].try_into().unwrap()),
                crate::checksum::linux_crc32c(!0, &bytes[1024..2044])
            );
            std::fs::copy(&image, base.with_extension(format!("{phase}.dirty.img"))).unwrap();
        });
        fs.finish().unwrap();
        std::fs::copy(&image, base.with_extension(format!("{phase}.img"))).unwrap();
        let bytes = std::fs::read(&image).unwrap();
        let fs = Filesystem::mount(Arc::new(FileDevice::open_path(&image).unwrap())).unwrap();
        assert_ne!(fs.sb.state & crate::superblock::EXT4_VALID_FS, 0);
        assert_eq!(
            fs.sb.feature_incompat & features::Incompat::RECOVER.bits(),
            0
        );
        if let Some(j) = crate::jbd2::read_superblock(&fs).unwrap() {
            assert!(j.is_clean());
        }
        for (ino, _, raw) in &records {
            assert_eq!(&fs.read_inode_verified(*ino).unwrap().1, raw);
        }
        accounting(&fs, &initial_alloc, initial_free, &image, added);
        let inode = fs.stat_ino(target).unwrap();
        let raw = fs.read_inode_verified(target).unwrap().1;
        names.push(destination.clone());
        records.push((target, inode, raw));
        let expected = names
            .iter()
            .zip(&records)
            .map(|(p, (n, i, _))| literal(p, *n, i))
            .collect::<Vec<_>>();
        let mut lists = Vec::new();
        for (path, entries) in &directories {
            let mut entries = entries.clone();
            if path == b"/policy" {
                for (_, alias) in [
                    ("fit", b"fit".to_vec()),
                    ("convert", vec![b'n'; 255]),
                    ("post", b"post".to_vec()),
                ]
                .iter()
                .take(index + 1)
                {
                    entries.push(alias.clone());
                }
            }
            lists.push(format!(
                "({},[{}])",
                pybytes(path),
                entries
                    .iter()
                    .map(|p| pybytes(p))
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
        let files = names
            .iter()
            .zip(&records)
            .filter(|(_, (_, i, _))| i.is_file())
            .map(|(p, _)| pybytes(p))
            .collect::<Vec<_>>();
        let expected=format!("EXPECTED = [{}]\nATTRS = [{}]\nDIRECTORIES = [{}]\nFILES = [{}]\nFREE = ({},{})\nBS = {bs}\nCOUNT = {}\n",expected.join(","),attrs.join(","),lists.join(","),files.join(","),initial_free.0-u64::from(index>0),initial_free.1,index+1);
        std::fs::write(
            base.with_extension(format!("{phase}.expected.py")),
            &expected,
        )
        .unwrap();
        std::fs::write(base.with_extension(format!("{phase}.allocation.json")),format!("{{\"parent\":{parent},\"source\":{target},\"added_block\":{},\"free_blocks\":{},\"free_inodes\":{}}}\n",added.map_or("null".to_owned(),|n|n.to_string()),initial_free.0-u64::from(index>0),initial_free.1)).unwrap();
        drop(fs);
        judged(&image, &base, phase);
        report(&image, &base, phase, &records);
        assert_eq!(std::fs::read(&image).unwrap(), bytes);
        let cold = base.with_extension(format!("{phase}.cold.img"));
        std::fs::copy(&image, &cold).unwrap();
        let out = guest_kernel_write(
            cold.to_str().unwrap(),
            &format!("python3 - \"$MNT\" cold <<'INLINE_COLD'\n{expected}\n{GUEST}\nINLINE_COLD\n"),
        );
        std::fs::write(
            base.with_extension(format!("{phase}.linux.jsonl")),
            &out.stdout,
        )
        .unwrap();
        std::fs::write(
            base.with_extension(format!("{phase}.stderr.txt")),
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
                .filter(|l| l.starts_with('{'))
                .count(),
            names.len() + attrs.len() + 1
        );
        judged(&cold, &base, &format!("{phase}.cold"));
        assert_eq!(std::fs::read(&image).unwrap(), bytes);
        eprintln!("inline link capi={capi} bs={bs} journal={journal} continued={continued} phase={phase}: added={added:?}, same table, original entries/EA/accounting/cold metadata retained");
    }
    std::fs::remove_file(image).unwrap();
}
