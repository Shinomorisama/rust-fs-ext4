//! Independent Linux/e2fsck evidence for ordinary public Rust/C create emission.
use super::setxattr::make;
use super::*;
use crate::inode::Inode;
use std::path::Path;

struct CreateClock(i64);
impl crate::runtime::Runtime for CreateClock {
    fn now_unix_seconds(&self) -> i64 {
        self.0
    }
    fn next_inode_generation(&self) -> u32 {
        17
    }
}

#[test]
fn linux_checks_public_create_metadata() {
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

struct Target<'a> {
    dir: u32,
    name: &'a [u8],
    path: &'a [u8],
}
fn call(
    fs: &Filesystem,
    h: *mut crate::capi::fs_ext4_fs_t,
    capi: bool,
    entry: usize,
    target: Target<'_>,
    mode: u16,
) -> std::result::Result<u32, i32> {
    if !capi {
        return match entry {
            0 => fs.apply_create(std::str::from_utf8(target.path).unwrap(), mode),
            1 => fs.apply_create_at(target.dir, target.name, mode),
            2 => fs.apply_create_bytes(target.path, mode),
            _ => unreachable!(),
        }
        .map_err(|e| e.to_errno());
    }
    use crate::capi::*;
    assert_eq!(unsafe { fs_ext4_create(h, std::ptr::null(), mode) }, 0);
    assert_eq!(fs_ext4_last_errno(), crate::error::errno::EINVAL);
    if entry == 1 {
        let mut attr: fs_ext4_attr_t = unsafe { std::mem::zeroed() };
        let rc = unsafe {
            fs_ext4_create_at(
                h,
                target.dir,
                FS_EXT4_GEN_ANY,
                target.name.as_ptr().cast(),
                target.name.len(),
                mode,
                &mut attr,
            )
        };
        if rc == -1 {
            return Err(fs_ext4_last_errno());
        }
        assert_eq!(rc, 0);
        assert_eq!(fs_ext4_last_errno(), 0);
        assert_eq!(attr.mode, mode);
        assert_eq!(attr.generation, 17);
        assert_eq!(attr.link_count, 1);
        assert_eq!((attr.size, attr.blocks_512), (0, 0));
        Ok(attr.inode)
    } else {
        let path = std::ffi::CString::new(target.path).unwrap();
        let ino = unsafe { fs_ext4_create(h, path.as_ptr(), mode) };
        if ino == 0 {
            Err(fs_ext4_last_errno())
        } else {
            assert_eq!(fs_ext4_last_errno(), 0);
            Ok(ino)
        }
    }
}

fn free(image: &Path) -> (u64, u32) {
    let b = std::fs::read(image).unwrap();
    (
        u64::from(u32::from_le_bytes(b[1036..1040].try_into().unwrap()))
            | (u64::from(u32::from_le_bytes(b[1368..1372].try_into().unwrap())) << 32),
        u32::from_le_bytes(b[1040..1044].try_into().unwrap()),
    )
}

fn groups(fs: &Filesystem) -> Vec<(u64, u32, u32)> {
    (0..fs.groups.len())
        .map(|gi| {
            let (block, off) = fs.sb.descriptor_location(gi as u64);
            let b = fs.read_block(block).unwrap();
            let word = |at| {
                u32::from(u16::from_le_bytes(
                    b[off + at..off + at + 2].try_into().unwrap(),
                ))
            };
            let high = |at| {
                if fs.sb.desc_size >= 64 {
                    word(at) << 16
                } else {
                    0
                }
            };
            (
                u64::from(word(12) | high(44)),
                word(14) | high(46),
                word(16) | high(48),
            )
        })
        .collect()
}

fn literal(path: &[u8], ino: u32, i: &Inode) -> String {
    let hex = path.iter().map(|b| format!("{b:02x}")).collect::<String>();
    format!("(bytes.fromhex('{hex}'), dict(ino={ino},kind={},mode={},uid={},gid={},links={},size={},blocks={},atime={},mtime={},ctime={}))",
        i.mode&crate::inode::S_IFMT,i.mode&0xfff,i.uid,i.gid,i.links_count,i.size,i.blocks,
        i128::from(i.atime)*1_000_000_000+i128::from(i.atime_nsec),
        i128::from(i.mtime)*1_000_000_000+i128::from(i.mtime_nsec),
        i128::from(i.ctime)*1_000_000_000+i128::from(i.ctime_nsec))
}

fn report(image: &Path, base: &Path, phase: &str, inodes: &[u32]) {
    for (index, ino) in inodes.iter().enumerate() {
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
        assert!(
            text.contains(&format!("Inode: {ino}"))
                && text.contains("ctime:")
                && text.contains("Generation:"),
            "{text}"
        );
    }
}

fn profile(capi: bool, bs: u32, size: u16, journal: bool) {
    let base = fs_ext4_test_support::temp_dir().join(format!(
        "create-emission-{}-{bs}-{size}-{journal}",
        if capi { "capi" } else { "rust" }
    ));
    let image = base.with_extension("img");
    make(&image, bs, size, journal);
    let input = guest_kernel_write(
        image.to_str().unwrap(),
        &format!(
            "python3 - \"$MNT\" input <<'CREATE_INPUT'\n{}\nCREATE_INPUT\n",
            include_str!("create_policy_guest.py")
        ),
    );
    std::fs::write(base.with_extension("input.jsonl"), &input.stdout).unwrap();
    std::fs::write(base.with_extension("input.stderr.txt"), &input.stderr).unwrap();
    assert!(
        input.status.success(),
        "{}",
        String::from_utf8_lossy(&input.stderr)
    );
    assert_e2fsck_clean(image.to_str().unwrap(), "Linux create emission input");
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
    let boundary = [b"/names/".as_slice(), &[b'n'; 255]].concat();
    let existing = [b"/policy/".as_slice(), &[b'e'; 255]].concat();
    let paths: Vec<Vec<u8>> = [
        b"/policy".as_slice(),
        b"/names",
        b"/peer",
        b"/peer/kept",
        b"/policy/existingdir",
        b"/policy/regular",
        &existing,
    ]
    .into_iter()
    .map(|p| p.to_vec())
    .collect();
    let originals: Vec<_> = paths
        .iter()
        .map(|p| {
            let ino = fs.resolve(p).unwrap();
            let (i, raw) = fs.read_inode_verified(ino).unwrap();
            (ino, i, raw)
        })
        .collect();
    for (_, i, _) in &originals[..3] {
        assert!(i.is_dir() && !i.has_inline_data());
        assert_eq!(i.flags & crate::inode::InodeFlags::INDEX.bits(), 0);
    }
    let parent = originals[0].0;
    let names = originals[1].0;
    let ea = fs.read_block(originals[0].1.file_acl).unwrap();
    assert_ne!(originals[0].1.file_acl, 0);
    let first = originals.iter().map(|(_, i, _)| i.ctime).max().unwrap() + 100;
    assert!(first + 200 < i64::from(i32::MAX));
    let initial = free(&image);
    let mut expected_groups = groups(&fs);
    let mut ids: Vec<_> = originals.iter().map(|(ino, _, _)| *ino).collect();
    drop(fs);
    report(&image, &base, "input", &ids);
    let mut children: Vec<(Vec<u8>, u32, Vec<u8>)> = Vec::new();
    for (index, (phase, name, mode)) in [
        ("first", b"child".as_slice(), 0o2750),
        ("second", b"raw-\xff", 0o1770),
        ("boundary", &[b'n'; 255], 0o750),
    ]
    .into_iter()
    .enumerate()
    {
        let stamp = first + index as i64 * 100;
        let fs = Filesystem::mount_with_runtime(
            Arc::new(FileDevice::open_path_rw(&image).unwrap()),
            Arc::new(CreateClock(stamp)),
        )
        .unwrap();
        assert_eq!(fs.journal.is_some(), journal);
        let mut expected = String::new();
        let fs = crate::capi::with_test_mount(fs, |h, fs| {
            let before = std::fs::read(&image).unwrap();
            for (bad, err) in [
                (b"existingdir".as_slice(), crate::error::errno::EEXIST),
                (b"regular", crate::error::errno::EEXIST),
                (b"link-dir", crate::error::errno::EEXIST),
                (b"link-file", crate::error::errno::EEXIST),
                (b"dangling", crate::error::errno::EEXIST),
                (b".", crate::error::errno::EEXIST),
                (b"..", crate::error::errno::EEXIST),
                (&[b'n'; 256], crate::error::errno::ENAMETOOLONG),
                (&[b'e'; 255], crate::error::errno::EEXIST),
            ] {
                let path = [b"/policy/".as_slice(), bad].concat();
                assert_eq!(
                    call(
                        fs,
                        h,
                        capi,
                        index,
                        Target {
                            dir: parent,
                            name: bad,
                            path: &path
                        },
                        mode
                    ),
                    Err(err)
                );
                assert_eq!(std::fs::read(&image).unwrap(), before);
            }
            if index != 1 {
                for (prefix, err) in [
                    (b"regular/".as_slice(), crate::error::errno::ENOTDIR),
                    (b"missing/", crate::error::errno::ENOENT),
                ] {
                    let bad = [prefix, &[b'n'; 256]].concat();
                    let path = [b"/policy/".as_slice(), &bad].concat();
                    assert_eq!(
                        call(
                            fs,
                            h,
                            capi,
                            index,
                            Target {
                                dir: parent,
                                name: &bad,
                                path: &path
                            },
                            mode
                        ),
                        Err(err)
                    );
                    assert_eq!(std::fs::read(&image).unwrap(), before);
                }
            }
            let (dir, path) = match index {
                0 => (parent, b"/parent-link/child///".to_vec()),
                1 => (parent, b"/policy/raw-\xff".to_vec()),
                _ => (names, boundary.clone()),
            };
            let ino = call(
                fs,
                h,
                capi,
                index,
                Target {
                    dir,
                    name,
                    path: &path,
                },
                mode,
            )
            .unwrap();
            let canonical = if index == 2 {
                boundary.clone()
            } else {
                [b"/policy/".as_slice(), name].concat()
            };
            assert_eq!(fs.resolve(&canonical).unwrap(), ino);
            let (child, raw) = fs.read_inode_verified(ino).unwrap();
            assert!(child.is_file() && !child.has_inline_data());
            assert_eq!(child.has_extents(), size == 256);
            assert_eq!(child.flags & crate::inode::InodeFlags::INDEX.bits(), 0);
            assert_eq!(
                (
                    child.mode & 0xfff,
                    child.links_count,
                    child.size,
                    child.blocks,
                    child.generation
                ),
                (mode, 1, 0, 0, 17)
            );
            assert_eq!((child.uid, child.gid), (0, 0));
            assert_eq!(
                (child.atime, child.mtime, child.ctime),
                (stamp, stamp, stamp)
            );
            assert_eq!(
                (child.atime_nsec, child.mtime_nsec, child.ctime_nsec),
                (0, 0, 0)
            );
            assert_eq!(child.file_acl, 0);
            assert_eq!(fs.map_inode_logical(&child, 0).unwrap(), None);
            assert!(fs.inode_bit_is_set(ino).unwrap());
            let gi = ((ino - 1) / fs.sb.inodes_per_group) as usize;
            expected_groups[gi].1 -= 1;
            assert_eq!(groups(fs), expected_groups);
            assert_eq!(free(&image), (initial.0, initial.1 - index as u32 - 1));
            children.push((canonical, ino, raw));
            ids.push(ino);
            let mut records = Vec::new();
            for (i, (p, (ino, original, raw))) in paths.iter().zip(&originals).enumerate() {
                let (after, found) = fs.read_inode_verified(*ino).unwrap();
                if i < 2 {
                    assert_eq!(
                        (after.size, after.blocks, after.atime, after.atime_nsec),
                        (
                            original.size,
                            original.blocks,
                            original.atime,
                            original.atime_nsec
                        )
                    );
                    let creations = if i == 0 {
                        (index + 1).min(2)
                    } else {
                        usize::from(index == 2)
                    };
                    assert_eq!(after.links_count, original.links_count);
                    assert_eq!(
                        (
                            after.mode,
                            after.uid,
                            after.gid,
                            after.flags,
                            after.generation,
                            after.file_acl
                        ),
                        (
                            original.mode,
                            original.uid,
                            original.gid,
                            original.flags,
                            original.generation,
                            original.file_acl
                        )
                    );
                    assert_eq!(&found[0x28..0x64], &raw[0x28..0x64]);
                    if creations > 0 {
                        let time = if i == 0 {
                            first + index.min(1) as i64 * 100
                        } else {
                            stamp
                        };
                        assert_eq!(
                            (after.mtime, after.mtime_nsec, after.ctime, after.ctime_nsec),
                            (time, 0, time, 0)
                        );
                    } else {
                        assert_eq!(found, *raw);
                    }
                } else {
                    assert_eq!(found, *raw);
                }
                records.push(literal(p, *ino, &after));
            }
            assert_eq!(fs.read_block(originals[0].1.file_acl).unwrap(), ea);
            for (p, ino, raw) in &children {
                let (i, found) = fs.read_inode_verified(*ino).unwrap();
                assert_eq!(found, *raw);
                records.push(literal(p, *ino, &i));
            }
            expected = format!(
                "EXPECTED = [{}]\nFREE = ({}, {})\nBS = {bs}\nCOUNT = {}\n",
                records.join(","),
                initial.0,
                initial.1 - index as u32 - 1,
                index + 1
            );
            let bytes = std::fs::read(&image).unwrap();
            assert_eq!(
                u16::from_le_bytes(bytes[1082..1084].try_into().unwrap())
                    & crate::superblock::EXT4_VALID_FS,
                0
            );
            std::fs::copy(&image, base.with_extension(format!("{phase}.dirty.img"))).unwrap();
        });
        fs.finish().unwrap();
        std::fs::copy(&image, base.with_extension(format!("{phase}.img"))).unwrap();
        let closed = Filesystem::mount(Arc::new(FileDevice::open_path(&image).unwrap())).unwrap();
        assert!(!closed.dev.is_writable());
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
        for (path, ino, raw) in &children {
            assert_eq!(closed.resolve(path).unwrap(), *ino);
            assert_eq!(closed.read_inode_verified(*ino).unwrap().1, *raw);
        }
        drop(closed);
        assert_e2fsck_clean(
            image.to_str().unwrap(),
            &format!("public create {phase} emission"),
        );
        report(&image, &base, phase, &ids);
        let command = format!(
            "python3 - \"$MNT\" <<'CREATE_READBACK'\n{expected}\n{}\nCREATE_READBACK\n",
            include_str!("create_guest.py")
        );
        let out = guest_kernel_write(image.to_str().unwrap(), &command);
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
            "{base:?} {phase}: {}\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert_e2fsck_clean(
            image.to_str().unwrap(),
            &format!("Linux create {phase} readback"),
        );
        eprintln!("create emission capi={capi} bs={bs} inode={size} journal={journal} phase={phase}: blocks=0 inodes={} used_dirs={}",index+1,expected_groups.iter().map(|g|g.2).sum::<u32>());
    }
    std::fs::remove_file(image).unwrap();
}
