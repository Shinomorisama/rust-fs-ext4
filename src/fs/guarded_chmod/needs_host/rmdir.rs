//! Independent Rust/C rmdir emission for ordinary, closed-target directories.
use super::setxattr::make;
use super::*;
use crate::inode::Inode;
use std::path::Path;

struct RmdirClock(i64);
impl crate::runtime::Runtime for RmdirClock {
    fn now_unix_seconds(&self) -> i64 {
        self.0
    }
    fn next_inode_generation(&self) -> u32 {
        17
    }
}

#[test]
fn linux_checks_public_rmdir_metadata() {
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
fn free(image: &Path) -> (u64, u32) {
    let b = std::fs::read(image).unwrap();
    (
        u64::from(u32::from_le_bytes(b[1036..1040].try_into().unwrap()))
            | (u64::from(u32::from_le_bytes(b[1368..1372].try_into().unwrap())) << 32),
        u32::from_le_bytes(b[1040..1044].try_into().unwrap()),
    )
}
fn call(
    fs: &Filesystem,
    h: *mut crate::capi::fs_ext4_fs_t,
    capi: bool,
    parent: u32,
    entry: usize,
    name: &[u8],
) -> std::result::Result<(), i32> {
    let path = [b"/p/".as_slice(), name].concat();
    if !capi {
        return match entry {
            0 => fs.apply_rmdir(std::str::from_utf8(&path).unwrap()),
            1 => fs.apply_rmdir_at(parent, name),
            2 => fs.apply_rmdir_bytes(&path),
            _ => unreachable!(),
        }
        .map_err(|e| e.to_errno());
    }
    // Seed an error so successful dispatch must clear it.
    assert_eq!(
        unsafe { crate::capi::fs_ext4_rmdir(h, std::ptr::null()) },
        -1
    );
    let rc = if entry == 1 {
        unsafe {
            crate::capi::fs_ext4_rmdir_at(
                h,
                parent,
                crate::capi::FS_EXT4_GEN_ANY,
                name.as_ptr().cast(),
                name.len(),
            )
        }
    } else {
        let path = std::ffi::CString::new(path).unwrap();
        unsafe { crate::capi::fs_ext4_rmdir(h, path.as_ptr()) }
    };
    if rc == 0 {
        assert_eq!(crate::capi::fs_ext4_last_errno(), 0);
        Ok(())
    } else {
        assert_eq!(rc, -1);
        Err(crate::capi::fs_ext4_last_errno())
    }
}
fn refs(fs: &Filesystem, inode: &Inode) -> u32 {
    assert_ne!(inode.file_acl, 0);
    let b = fs.read_block(inode.file_acl).unwrap();
    assert!(fs.csum.verify_xattr_block(inode.file_acl, &b));
    u32::from_le_bytes(b[4..8].try_into().unwrap())
}
fn report(image: &Path, base: &Path, phase: &str, inodes: &[u32]) {
    for (label, ino) in ["parent", "exclusive", "shared", "boundary", "peer"]
        .into_iter()
        .zip(inodes)
    {
        let out = oracle("debugfs")
            .args(["-R", &format!("stat <{ino}>")])
            .arg(image)
            .env("TZ", "GMT")
            .output();
        std::fs::write(
            base.with_extension(format!("{phase}.{label}.debugfs.txt")),
            &out.stdout,
        )
        .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let text = String::from_utf8(out.stdout).unwrap();
        assert!(text.contains("Inode:") && text.contains("ctime:"), "{text}");
    }
}

fn profile(capi: bool, bs: u32, size: u16, journal: bool) {
    let base = fs_ext4_test_support::temp_dir().join(format!(
        "rmdir-emission-{}-{bs}-{size}-{journal}",
        if capi { "capi" } else { "rust" }
    ));
    let image = base.with_extension("img");
    make(&image, bs, size, journal);
    let input = guest_kernel_write(
        image.to_str().unwrap(),
        &format!(
            "python3 - \"$MNT\" input <<'RMDIR_INPUT'\n{}\nRMDIR_INPUT\n",
            include_str!("rmdir_guest.py")
        ),
    );
    std::fs::write(base.with_extension("input.jsonl"), &input.stdout).unwrap();
    std::fs::write(base.with_extension("input.stderr.txt"), &input.stderr).unwrap();
    assert!(
        input.status.success(),
        "{}",
        String::from_utf8_lossy(&input.stderr)
    );
    assert_e2fsck_clean(image.to_str().unwrap(), "Linux rmdir emission input");
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
    let parent = fs.resolve(b"/p").unwrap();
    let parent_before = fs.read_inode_verified(parent).unwrap().0;
    assert!(parent_before.is_dir() && !parent_before.has_inline_data());
    assert_eq!(
        parent_before.flags & crate::inode::InodeFlags::INDEX.bits(),
        0
    );
    let boundary = "n".repeat(255);
    let targets: Vec<(String, u32, Inode, u64)> = ["exclusive", "shared", &boundary]
        .into_iter()
        .map(|name| {
            let ino = fs.resolve(format!("/p/{name}").as_bytes()).unwrap();
            let inode = fs.read_inode_verified(ino).unwrap().0;
            assert!(inode.is_dir() && !inode.has_inline_data());
            assert_eq!(inode.has_extents(), size == 256);
            assert_eq!(inode.flags & crate::inode::InodeFlags::INDEX.bits(), 0);
            assert_eq!((inode.links_count, inode.size), (2, u64::from(bs)));
            assert_eq!(
                inode.blocks,
                u64::from(bs) / 512 * if name == boundary { 1 } else { 2 }
            );
            assert_eq!(inode.file_acl == 0, name == boundary);
            let data = fs.map_inode_logical(&inode, 0).unwrap().unwrap();
            (name.to_owned(), ino, inode, data)
        })
        .collect();
    let peer = fs.resolve(b"/p/peer").unwrap();
    let (peer_inode, peer_raw) = fs.read_inode_verified(peer).unwrap();
    assert_eq!(peer_inode.file_acl, targets[1].2.file_acl);
    assert_ne!(targets[0].2.file_acl, targets[1].2.file_acl);
    assert_eq!(refs(&fs, &targets[0].2), 1);
    assert_eq!(refs(&fs, &targets[1].2), 2);
    let first = targets
        .iter()
        .map(|t| t.2.ctime)
        .chain([parent_before.ctime, peer_inode.ctime])
        .max()
        .unwrap()
        + 100;
    assert!(first + 200 < i64::from(i32::MAX));
    let mut expected_groups = groups(&fs);
    let initial_free = free(&image);
    let input_text = String::from_utf8(input.stdout).unwrap();
    let linux_ledger: Vec<u64> = input_text
        .lines()
        .find_map(|s| s.strip_prefix("LEDGER "))
        .unwrap()
        .split_whitespace()
        .map(|s| s.parse().unwrap())
        .collect();
    assert_eq!(
        linux_ledger,
        vec![initial_free.0, u64::from(initial_free.1)]
    );
    let inodes: Vec<u32> = std::iter::once(parent)
        .chain(targets.iter().map(|t| t.1))
        .chain([peer])
        .collect();
    drop(fs);
    report(&image, &base, "input", &inodes);
    let mut freed = 0;
    for (index, phase) in ["exclusive", "shared", "boundary"].into_iter().enumerate() {
        let stamp = first + index as i64 * 100;
        let fs = Filesystem::mount_with_runtime(
            Arc::new(FileDevice::open_path_rw(&image).unwrap()),
            Arc::new(RmdirClock(stamp)),
        )
        .unwrap();
        assert_eq!(fs.journal.is_some(), journal);
        let fs = crate::capi::with_test_mount(fs, |h, fs| {
            let before = std::fs::read(&image).unwrap();
            for (name, expected) in [
                (b"missing".as_slice(), crate::error::errno::ENOENT),
                (b"regular", crate::error::errno::ENOTDIR),
                (b"link", crate::error::errno::ENOTDIR),
                (b"nonempty", crate::error::errno::ENOTEMPTY),
                (b".", crate::error::errno::EINVAL),
                (b"..", crate::error::errno::ENOTEMPTY),
                (&[b'n'; 256], crate::error::errno::ENAMETOOLONG),
            ] {
                assert_eq!(call(fs, h, capi, parent, 1, name), Err(expected));
                assert_eq!(std::fs::read(&image).unwrap(), before);
            }
            let (name, ino, original, data) = &targets[index];
            call(fs, h, capi, parent, index, name.as_bytes()).unwrap();
            assert!(matches!(
                fs.resolve(format!("/p/{name}").as_bytes()),
                Err(Error::NotFound)
            ));
            assert!(!fs.inode_bit_is_set(*ino).unwrap());
            let (dead, raw) = fs.read_inode_verified(*ino).unwrap();
            assert_eq!(
                (dead.mode, dead.links_count, dead.generation),
                (0, 0, original.generation)
            );
            assert_eq!(
                u32::from_le_bytes(raw[20..24].try_into().unwrap()),
                stamp as u32
            );
            assert!(matches!(fs.live_inode((*ino).into()), Err(Error::Stale)));
            let mut released = vec![*data];
            if index == 0 {
                released.push(original.file_acl);
            }
            assert_eq!(
                released.len() as u64,
                original.blocks / (u64::from(bs) / 512) - u64::from(index == 1)
            );
            for block in released {
                assert!(!super::removexattr::allocated(fs, block));
                let gi = fs.group_chunks(block, 1).unwrap()[0].0;
                expected_groups[gi].0 += 1;
                freed += 1;
            }
            let gi = ((*ino - 1) / fs.sb.inodes_per_group) as usize;
            expected_groups[gi].1 += 1;
            expected_groups[gi].2 -= 1;
            assert_eq!(groups(fs), expected_groups);
            assert_eq!(fs.read_inode_verified(peer).unwrap().1, peer_raw);
            assert!(super::removexattr::allocated(fs, peer_inode.file_acl));
            assert_eq!(refs(fs, &peer_inode), if index == 0 { 2 } else { 1 });
            let after = fs.read_inode_verified(parent).unwrap().0;
            assert_eq!(
                (after.mtime, after.mtime_nsec, after.ctime, after.ctime_nsec),
                (stamp, 0, stamp, 0)
            );
            assert_eq!(
                (after.atime, after.atime_nsec),
                (parent_before.atime, parent_before.atime_nsec)
            );
            assert_eq!(
                after.links_count,
                parent_before.links_count - index as u16 - 1
            );
            assert_eq!(
                free(&image),
                (initial_free.0 + freed, initial_free.1 + index as u32 + 1)
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
        assert_e2fsck_clean(
            image.to_str().unwrap(),
            &format!("public rmdir {phase} emission"),
        );
        report(&image, &base, phase, &inodes);
        let expected = free(&image);
        let command = format!("python3 - \"$MNT\" {phase} {} {stamp} {parent} {peer} {} {} {} {bs} {} <<'RMDIR_READBACK'\n{}\nRMDIR_READBACK\n",
            index + 1, parent_before.links_count, expected.0, expected.1,
            peer_inode.ctime * 1_000_000_000 + i64::from(peer_inode.ctime_nsec), include_str!("rmdir_guest.py"));
        let checked = guest_kernel_write(image.to_str().unwrap(), &command);
        std::fs::write(
            base.with_extension(format!("{phase}.linux.jsonl")),
            &checked.stdout,
        )
        .unwrap();
        std::fs::write(
            base.with_extension(format!("{phase}.stderr.txt")),
            &checked.stderr,
        )
        .unwrap();
        assert!(
            checked.status.success(),
            "{base:?} {phase}: {}\n{}",
            String::from_utf8_lossy(&checked.stdout),
            String::from_utf8_lossy(&checked.stderr)
        );
        assert_e2fsck_clean(
            image.to_str().unwrap(),
            &format!("Linux rmdir {phase} readback"),
        );
        eprintln!("rmdir emission capi={capi} bs={bs} inode={size} journal={journal} phase={phase}: freed_blocks={freed} freed_inodes={} used_dirs={}", index + 1, expected_groups.iter().map(|g| g.2).sum::<u32>());
    }
    assert_eq!(freed, 4);
    std::fs::remove_file(image).unwrap();
}
