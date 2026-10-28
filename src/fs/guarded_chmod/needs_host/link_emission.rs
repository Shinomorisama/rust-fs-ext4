//! Independent ordinary public-link emission; allocation-free layouts only.
use super::super::timestamp_report::debugfs_time;
use super::setxattr::make;
use super::*;
use crate::capi::*;
use crate::inode::Inode;
use std::path::Path;

const PATHS: [&[u8]; 12] = [
    b"/policy",
    b"/names",
    b"/old",
    b"/peer",
    b"/peer/kept",
    b"/policy/existingdir",
    b"/policy/existing",
    b"/old/regular",
    b"/old/fast",
    b"/old/slow",
    b"/old/dangling",
    b"/old/fifo",
];
struct LinkClock {
    stamp: i64,
    calls: std::sync::atomic::AtomicUsize,
}
impl crate::runtime::Runtime for LinkClock {
    fn now_unix_seconds(&self) -> i64 {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.stamp
    }
    fn next_inode_generation(&self) -> u32 {
        panic!("a hard link must retain the existing inode generation")
    }
}
struct Alias {
    phase: &'static str,
    source: usize,
    parent: usize,
    name: Vec<u8>,
    entry: usize,
}
impl Alias {
    fn path(&self) -> Vec<u8> {
        [PATHS[self.parent], b"/", &self.name].concat()
    }
}
fn aliases() -> Vec<Alias> {
    [
        ("first", 7, 0, b"alias".to_vec(), 0),
        ("second", 7, 0, b"raw-\xff".to_vec(), 1),
        ("boundary", 7, 1, vec![b'n'; 255], 2),
        ("fast", 8, 0, b"fast-alias".to_vec(), 0),
        ("slow", 9, 0, b"slow-alias".to_vec(), 1),
        ("dangling", 10, 0, b"dangling-alias".to_vec(), 2),
        ("special", 11, 0, b"fifo-alias".to_vec(), 1),
    ]
    .into_iter()
    .map(|(phase, source, parent, name, entry)| Alias {
        phase,
        source,
        parent,
        name,
        entry,
    })
    .collect()
}
fn version(raw: &[u8]) -> u64 {
    let lo = u32::from_le_bytes(raw[36..40].try_into().unwrap());
    let hi = if raw.len() >= 156 && u16::from_le_bytes(raw[128..130].try_into().unwrap()) >= 28 {
        u32::from_le_bytes(raw[152..156].try_into().unwrap())
    } else {
        0
    };
    u64::from(lo) | (u64::from(hi) << 32)
}
fn stable(before: &[u8], after: &[u8], allowed: &[(usize, usize)]) {
    let mut a = before.to_vec();
    let mut b = after.to_vec();
    for &(at, len) in allowed {
        if at + len <= a.len() {
            a[at..at + len].fill(0);
            b[at..at + len].fill(0);
        }
    }
    assert_eq!(a, b, "link changed an unqualified inode field");
}
fn free(image: &Path) -> (u64, u32) {
    let b = std::fs::read(image).unwrap();
    (
        u64::from(u32::from_le_bytes(b[1036..1040].try_into().unwrap()))
            | (u64::from(u32::from_le_bytes(b[1368..1372].try_into().unwrap())) << 32),
        u32::from_le_bytes(b[1040..1044].try_into().unwrap()),
    )
}
fn allocation(fs: &Filesystem) -> Vec<Vec<u8>> {
    // Full descriptor/bitmap bytes retain counters, checksums, unused-inode and
    // uninitialised-group flags, rather than only the mounted counter cache.
    let mut rows = Vec::new();
    for (index, group) in fs.groups.iter().enumerate() {
        rows.push(
            fs.read_block(fs.sb.descriptor_location(index as u64).0)
                .unwrap(),
        );
        rows.push(fs.read_block(group.inode_bitmap).unwrap());
        rows.push(fs.read_block(group.block_bitmap).unwrap());
    }
    rows
}
fn literal(path: &[u8], ino: u32, i: &Inode) -> String {
    let hex = path.iter().map(|b| format!("{b:02x}")).collect::<String>();
    format!("(bytes.fromhex('{hex}'), dict(ino={ino},kind={},mode={},uid={},gid={},links={},size={},blocks={},atime={},mtime={},ctime={}))",
        i.mode&crate::inode::S_IFMT,i.mode&0xfff,i.uid,i.gid,i.links_count,i.size,i.blocks,
        i128::from(i.atime)*1_000_000_000+i128::from(i.atime_nsec),
        i128::from(i.mtime)*1_000_000_000+i128::from(i.mtime_nsec),
        i128::from(i.ctime)*1_000_000_000+i128::from(i.ctime_nsec))
}
fn attr_literal(path: &[u8], kind: u16, a: &fs_ext4_attr_t) -> String {
    let hex = path.iter().map(|b| format!("{b:02x}")).collect::<String>();
    format!("(bytes.fromhex('{hex}'), dict(ino={},kind={kind},mode={},uid={},gid={},links={},size={},blocks={},atime={},mtime={},ctime={}))",
        a.inode,a.mode,a.uid,a.gid,a.link_count,a.size,a.blocks_512,
        i128::from(a.atime)*1_000_000_000+i128::from(a.atime_nsec),
        i128::from(a.mtime)*1_000_000_000+i128::from(a.mtime_nsec),
        i128::from(a.ctime)*1_000_000_000+i128::from(a.ctime_nsec))
}
fn call(
    fs: &Filesystem,
    h: *mut fs_ext4_fs_t,
    capi: bool,
    alias: &Alias,
    handles: (InodeRef, InodeRef),
    paths: (&[u8], &[u8], &[u8]),
) -> std::result::Result<Option<fs_ext4_attr_t>, i32> {
    let (target, parent) = handles;
    let (source, dst, name) = paths;
    if !capi {
        let result = match alias.entry {
            0 => fs.apply_link(
                std::str::from_utf8(source).unwrap(),
                std::str::from_utf8(dst).unwrap(),
            ),
            1 => fs.apply_link_at(target, parent, name),
            2 => fs.apply_link_bytes(source, dst),
            _ => unreachable!(),
        };
        return result.map(|()| None).map_err(|e| e.to_errno());
    }
    let mut attr: fs_ext4_attr_t = unsafe { std::mem::zeroed() };
    let read_attr = alias.entry == 1;
    let rc = if alias.entry == 1 || alias.phase == "boundary" {
        unsafe {
            fs_ext4_link_at(
                h,
                target.ino,
                target.generation.unwrap(),
                parent.ino,
                parent.generation.unwrap(),
                name.as_ptr().cast(),
                name.len(),
                if read_attr {
                    &mut attr
                } else {
                    std::ptr::null_mut()
                },
            )
        }
    } else {
        let source = std::ffi::CString::new(source).unwrap();
        let dst = std::ffi::CString::new(dst).unwrap();
        unsafe { fs_ext4_link(h, source.as_ptr(), dst.as_ptr()) }
    };
    if rc == -1 {
        return Err(fs_ext4_last_errno());
    }
    assert_eq!(rc, 0);
    assert_eq!(fs_ext4_last_errno(), 0);
    Ok(read_attr.then_some(attr))
}
fn check_attr(a: &fs_ext4_attr_t, ino: u32, i: &Inode) {
    assert_eq!(
        (
            a.inode,
            a.generation,
            a.mode,
            a.uid,
            a.gid,
            a.size,
            a.blocks_512,
            a.link_count,
            a.inode_flags
        ),
        (
            ino,
            i.generation,
            i.mode & 0xfff,
            i.uid,
            i.gid,
            i.size,
            i.blocks,
            i.links_count,
            i.flags
        )
    );
    assert_eq!(
        (
            a.atime,
            a.atime_nsec,
            a.mtime,
            a.mtime_nsec,
            a.ctime,
            a.ctime_nsec,
            a.crtime,
            a.crtime_nsec
        ),
        (
            i.atime,
            i.atime_nsec,
            i.mtime,
            i.mtime_nsec,
            i.ctime,
            i.ctime_nsec,
            i.crtime,
            i.crtime_nsec
        )
    );
    let kind = if i.is_file() {
        fs_ext4_file_type_t::RegFile
    } else if i.is_symlink() {
        fs_ext4_file_type_t::Symlink
    } else {
        assert_eq!(i.mode & crate::inode::S_IFMT, crate::inode::S_IFIFO);
        fs_ext4_file_type_t::Fifo
    };
    assert_eq!(a.file_type as u32, kind as u32);
}
fn judged(image: &Path, base: &Path, phase: &str) {
    let out = oracle("e2fsck")
        .arg("-fn")
        .arg(image)
        .judged()
        .clean("public hard-link emission");
    std::fs::write(
        base.with_extension(format!("{phase}.e2fsck.txt")),
        &out.stdout,
    )
    .unwrap();
    std::fs::write(
        base.with_extension(format!("{phase}.e2fsck.stderr.txt")),
        &out.stderr,
    )
    .unwrap();
}
fn report(image: &Path, base: &Path, phase: &str, records: &[(u32, Inode, Vec<u8>)]) {
    for (index, (ino, inode, raw)) in records.iter().enumerate() {
        let out = oracle("debugfs")
            .args(["-R", &format!("stat <{ino}>")])
            .arg(image)
            .env("TZ", "GMT")
            .output();
        std::fs::write(
            base.with_extension(format!("{phase}.{index}.debugfs.txt")),
            &out.stdout,
        )
        .unwrap();
        std::fs::write(
            base.with_extension(format!("{phase}.{index}.debugfs.stderr.txt")),
            &out.stderr,
        )
        .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let text = String::from_utf8(out.stdout).unwrap();
        let field = |label: &str| {
            text.split(label)
                .nth(1)
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap()
        };
        assert_eq!(field("Inode:").parse::<u32>().unwrap(), *ino);
        assert_eq!(
            field("Generation:").parse::<u32>().unwrap(),
            inode.generation
        );
        assert_eq!(
            u16::from_str_radix(field("Mode:"), 8).unwrap(),
            inode.mode & 0xfff
        );
        assert_eq!(field("Size:").parse::<u64>().unwrap(), inode.size);
        assert_eq!(field("Links:").parse::<u16>().unwrap(), inode.links_count);
        assert_eq!(field("Blockcount:").parse::<u64>().unwrap(), inode.blocks);
        assert_eq!(field("ACL:").parse::<u64>().unwrap(), inode.file_acl);
        assert_eq!(
            super::super::owner_report::owner_id(field("User:")).unwrap(),
            inode.uid
        );
        assert_eq!(
            super::super::owner_report::owner_id(field("Group:")).unwrap(),
            inode.gid
        );
        let hex = |s: &str| u32::from_str_radix(s.trim_start_matches("0x"), 16).unwrap();
        assert_eq!(hex(field("Flags:")), inode.flags);
        // Both display shapes were observed in the saved eight-layout Linux
        // reference: legacy one-word version and covered high:low version.
        let found = if let Some((hi, lo)) = field("Version:").split_once(':') {
            (u64::from(hex(hi)) << 32) | u64::from(hex(lo))
        } else {
            u64::from(hex(field("Version:")))
        };
        assert_eq!(found, version(raw), "{text}");
        assert_eq!(
            [
                debugfs_time(&text, "atime"),
                debugfs_time(&text, "mtime"),
                debugfs_time(&text, "ctime")
            ],
            [
                (inode.atime, inode.atime_nsec),
                (inode.mtime, inode.mtime_nsec),
                (inode.ctime, inode.ctime_nsec)
            ],
            "{text}"
        );
    }
}
#[test]
fn linux_checks_public_link_metadata() {
    for capi in [false, true] {
        for bs in [1024, 4096] {
            for size in [128, 256] {
                for journal in [false, true] {
                    profile(capi, bs, size, journal);
                }
            }
        }
    }
}
fn profile(capi: bool, bs: u32, size: u16, journal: bool) {
    let base = fs_ext4_test_support::temp_dir().join(format!(
        "link-emission-{}-{bs}-{size}-{journal}",
        if capi { "capi" } else { "rust" }
    ));
    let image = base.with_extension("img");
    make(&image, bs, size, journal);
    let input = guest_kernel_write(
        image.to_str().unwrap(),
        &format!(
            "python3 - \"$MNT\" input <<'LINK_INPUT'\n{}\nLINK_INPUT\n",
            include_str!("link_policy_guest.py")
        ),
    );
    std::fs::write(base.with_extension("input.jsonl"), &input.stdout).unwrap();
    std::fs::write(base.with_extension("input.stderr.txt"), &input.stderr).unwrap();
    assert!(
        input.status.success(),
        "{}",
        String::from_utf8_lossy(&input.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&input.stdout)
            .lines()
            .filter(|l| l.starts_with('{'))
            .count(),
        1
    );
    judged(&image, &base, "input");
    std::fs::copy(&image, base.with_extension("input.img")).unwrap();
    let fs = Filesystem::mount(Arc::new(FileDevice::open_path(&image).unwrap())).unwrap();
    assert_eq!(
        (fs.sb.block_size(), fs.sb.inode_size, fs.csum.enabled),
        (bs, size, size == 256)
    );
    assert_eq!(
        fs.sb.feature_compat & features::Compat::HAS_JOURNAL.bits() != 0,
        journal
    );
    let mut records: Vec<_> = PATHS
        .iter()
        .map(|p| {
            let ino = fs.resolve(p).unwrap();
            let (inode, raw) = fs.read_inode_verified(ino).unwrap();
            (ino, inode, raw)
        })
        .collect();
    for (_, i, _) in &records[..4] {
        assert!(i.is_dir() && !i.has_inline_data());
        assert_eq!(i.size, u64::from(bs));
        assert_eq!(i.flags & crate::inode::InodeFlags::INDEX.bits(), 0);
    }
    assert!(records[7].1.is_file());
    for n in [8, 9, 10] {
        assert!(records[n].1.is_symlink());
    }
    assert_eq!(
        records[11].1.mode & crate::inode::S_IFMT,
        crate::inode::S_IFIFO
    );
    let first = records.iter().map(|r| r.1.ctime).max().unwrap() + 100;
    assert!(first + 700 < i64::from(i32::MAX));
    let initial_free = free(&image);
    let initial_alloc = allocation(&fs);
    let eas: Vec<_> = [0, 3, 7]
        .iter()
        .map(|&n| {
            let nr = records[n].1.file_acl;
            assert_ne!(nr, 0);
            let raw = fs.read_block(nr).unwrap();
            assert_eq!(u32::from_le_bytes(raw[4..8].try_into().unwrap()), 1);
            (nr, raw)
        })
        .collect();
    drop(fs);
    report(&image, &base, "input", &records);
    let aliases = aliases();
    for (index, alias) in aliases.iter().enumerate() {
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
        let target = InodeRef::new(records[alias.source].0, records[alias.source].1.generation);
        let parent = InodeRef::new(records[alias.parent].0, records[alias.parent].1.generation);
        let source = [b"/old-link/".as_slice(), &PATHS[alias.source][5..]].concat();
        let dst = if index == 0 {
            [b"/parent-link/".as_slice(), &alias.name, b"///"].concat()
        } else {
            alias.path()
        };
        let mut attrs = Vec::new();
        let fs = crate::capi::with_test_mount(fs, |h, fs| {
            let before = std::fs::read(&image).unwrap();
            for (name, err) in [
                (b"existing".as_slice(), crate::error::errno::EEXIST),
                (b".", crate::error::errno::EEXIST),
                (&[b'n'; 256], crate::error::errno::ENAMETOOLONG),
            ] {
                let dst = [PATHS[alias.parent], b"/", name].concat();
                // Only /policy has the existing entry; dot and length exercise
                // every parent. The boundary phase uses /names instead.
                let name = if name == b"existing" && alias.parent == 1 {
                    b"..".as_slice()
                } else {
                    name
                };
                let dst = if name == b"." || name == b".." {
                    [PATHS[alias.parent], b"/", name].concat()
                } else {
                    dst
                };
                assert_eq!(
                    call(fs, h, capi, alias, (target, parent), (&source, &dst, name)).err(),
                    Some(err)
                );
                assert_eq!(std::fs::read(&image).unwrap(), before);
            }
            let dir = InodeRef::new(records[2].0, records[2].1.generation);
            assert_eq!(
                call(
                    fs,
                    h,
                    capi,
                    alias,
                    (dir, parent),
                    (b"/old", &dst, &alias.name)
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
                alias,
                (target, parent),
                (&source, &dst, &alias.name),
            )
            .unwrap();
            assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
            assert_eq!(fs.resolve(&alias.path()).unwrap(), target.ino);
            let mut current = Vec::new();
            for (n, (ino, old, raw)) in records.iter().enumerate() {
                let (inode, found) = fs.read_inode_verified(*ino).unwrap();
                if n == alias.source {
                    assert_eq!(inode.links_count, old.links_count + 1);
                    assert_eq!((inode.ctime, inode.ctime_nsec), (clock.stamp, 0));
                    stable(
                        raw,
                        &found,
                        &[(12, 4), (26, 2), (124, 2), (130, 2), (132, 4)],
                    );
                } else if n == alias.parent {
                    assert_eq!(
                        (inode.mtime, inode.mtime_nsec, inode.ctime, inode.ctime_nsec),
                        (clock.stamp, 0, clock.stamp, 0)
                    );
                    assert_eq!(version(&found), version(raw) + 1);
                    stable(
                        raw,
                        &found,
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
                } else {
                    assert_eq!(found, *raw, "protected peer/source changed");
                }
                current.push((*ino, inode, found));
            }
            for a in &aliases[..=index] {
                assert_eq!(fs.resolve(&a.path()).unwrap(), current[a.source].0);
            }
            assert_eq!(free(&image), initial_free);
            assert_eq!(allocation(fs), initial_alloc);
            for (nr, raw) in &eas {
                assert_eq!(fs.read_block(*nr).unwrap(), *raw);
            }
            if let Some(a) = attr {
                check_attr(&a, target.ino, &current[alias.source].1);
                attrs.push(attr_literal(
                    PATHS[alias.source],
                    current[alias.source].1.mode & crate::inode::S_IFMT,
                    &a,
                ));
            }
            records = current;
            let bytes = std::fs::read(&image).unwrap();
            assert_eq!(
                u16::from_le_bytes(bytes[1082..1084].try_into().unwrap())
                    & crate::superblock::EXT4_VALID_FS,
                0
            );
            if fs.csum.enabled {
                assert_eq!(
                    u32::from_le_bytes(bytes[2044..2048].try_into().unwrap()),
                    crate::checksum::linux_crc32c(!0, &bytes[1024..2044])
                );
            }
            std::fs::copy(
                &image,
                base.with_extension(format!("{}.dirty.img", alias.phase)),
            )
            .unwrap();
        });
        fs.finish().unwrap();
        std::fs::copy(&image, base.with_extension(format!("{}.img", alias.phase))).unwrap();
        let original = std::fs::read(&image).unwrap();
        let closed = Filesystem::mount(Arc::new(FileDevice::open_path(&image).unwrap())).unwrap();
        assert_ne!(closed.sb.state & crate::superblock::EXT4_VALID_FS, 0);
        assert_eq!(
            closed.sb.feature_incompat & features::Incompat::RECOVER.bits(),
            0
        );
        let jsb = crate::jbd2::read_superblock(&closed).unwrap();
        assert_eq!(jsb.is_some(), journal);
        if let Some(jsb) = jsb {
            assert!(jsb.is_clean());
        }
        assert_eq!(allocation(&closed), initial_alloc);
        for (ino, _, raw) in &records {
            assert_eq!(closed.read_inode_verified(*ino).unwrap().1, *raw);
        }
        drop(closed);
        judged(&image, &base, alias.phase);
        report(&image, &base, alias.phase, &records);
        assert_eq!(
            std::fs::read(&image).unwrap(),
            original,
            "read-only checks changed driver bytes"
        );
        let mut expected: Vec<_> = PATHS
            .iter()
            .zip(&records)
            .map(|(p, (ino, i, _))| literal(p, *ino, i))
            .collect();
        let mut pairs = Vec::new();
        for a in &aliases[..=index] {
            let (ino, i, _) = &records[a.source];
            expected.push(literal(&a.path(), *ino, i));
            let hex = |p: &[u8]| p.iter().map(|b| format!("{b:02x}")).collect::<String>();
            pairs.push(format!(
                "(bytes.fromhex('{}'),bytes.fromhex('{}'))",
                hex(PATHS[a.source]),
                hex(&a.path())
            ));
        }
        let expected=format!("EXPECTED = [{}]\nATTRS = [{}]\nALIASES = [{}]\nFREE = ({}, {})\nBS = {bs}\nCOUNT = {}\n",expected.join(","),attrs.join(","),pairs.join(","),initial_free.0,initial_free.1,index+1);
        std::fs::write(
            base.with_extension(format!("{}.expected.py", alias.phase)),
            &expected,
        )
        .unwrap();
        // The Linux copy can update symlink atime while reading targets. Keep
        // the actual driver bytes pristine for the next staged link/audit.
        let cold = base.with_extension(format!("{}.cold.img", alias.phase));
        std::fs::copy(&image, &cold).unwrap();
        let out = guest_kernel_write(
            cold.to_str().unwrap(),
            &format!(
                "python3 - \"$MNT\" <<'LINK_READBACK'\n{expected}\n{}\nLINK_READBACK\n",
                include_str!("link_emission_guest.py")
            ),
        );
        std::fs::write(
            base.with_extension(format!("{}.linux.jsonl", alias.phase)),
            &out.stdout,
        )
        .unwrap();
        std::fs::write(
            base.with_extension(format!("{}.stderr.txt", alias.phase)),
            &out.stderr,
        )
        .unwrap();
        assert!(
            out.status.success(),
            "{base:?} {}: {}\n{}",
            alias.phase,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|l| l.starts_with('{'))
                .count(),
            12 + index + 1 + attrs.len() + 1
        );
        judged(&cold, &base, &format!("{}.cold", alias.phase));
        assert_eq!(
            std::fs::read(&image).unwrap(),
            original,
            "cold-copy validation changed actual driver image"
        );
        eprintln!("link emission capi={capi} bs={bs} inode={size} journal={journal} phase={}: aliases={} source={} ctime={} no allocations; dirty/closed/cold bytes retained",alias.phase,index+1,target.ino,clock.stamp);
    }
    std::fs::remove_file(image).unwrap();
}
