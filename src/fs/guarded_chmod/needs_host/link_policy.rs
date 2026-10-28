//! Linux-only hard-link reference policy. The driver reads closed images only.
use super::setxattr::make;
use super::*;
use crate::inode::Inode;
use std::path::Path;

const PHASES: [&str; 10] = [
    "input", "names", "first", "second", "boundary", "fast", "slow", "dangling", "special",
    "closed",
];
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

#[test]
fn linux_records_link_policy() {
    for bs in [1024, 4096] {
        for size in [128, 256] {
            for journal in [false, true] {
                reference(bs, size, journal);
            }
        }
    }
}

fn times(i: &Inode) -> [(i64, u32); 3] {
    [
        (i.atime, i.atime_nsec),
        (i.mtime, i.mtime_nsec),
        (i.ctime, i.ctime_nsec),
    ]
}
fn groups(fs: &Filesystem) -> Vec<(u64, u32, u32)> {
    let counts: Vec<_> = fs
        .groups
        .iter()
        .map(|g| {
            (
                u64::from(g.free_blocks_count),
                g.free_inodes_count,
                g.used_dirs_count,
            )
        })
        .collect();
    assert_eq!(
        counts.iter().map(|g| g.0).sum::<u64>(),
        fs.sb.free_blocks_count
    );
    assert_eq!(
        counts.iter().map(|g| g.1).sum::<u32>(),
        fs.sb.free_inodes_count
    );
    counts
}
// The published ext4 inode format places the Linux change counter at 0x24,
// with its covered high word at 0x98. It is distinct from i_generation.
fn directory_version(raw: &[u8]) -> u64 {
    let low = u32::from_le_bytes(raw[36..40].try_into().unwrap());
    let high = if raw.len() >= 156 && u16::from_le_bytes(raw[128..130].try_into().unwrap()) >= 28 {
        u32::from_le_bytes(raw[152..156].try_into().unwrap())
    } else {
        0
    };
    u64::from(low) | (u64::from(high) << 32)
}
fn stable_raw(before: &[u8], now: &[u8], allowed: &[(usize, usize)]) {
    let mut a = before.to_vec();
    let mut b = now.to_vec();
    for &(at, len) in allowed {
        if at + len <= a.len() {
            a[at..at + len].fill(0);
            b[at..at + len].fill(0);
        }
    }
    assert_eq!(a, b, "reference changed an unmeasured inode field");
}
fn reference(bs: u32, size: u16, journal: bool) {
    let base = fs_ext4_test_support::temp_dir().join(format!("link-policy-{bs}-{size}-{journal}"));
    let image = base.with_extension("img");
    make(&image, bs, size, journal);
    let mut originals: Vec<(u32, Inode, Vec<u8>)> = Vec::new();
    let mut previous: Vec<Inode> = Vec::new();
    let mut previous_versions = [0; 2];
    let mut initial_groups = Vec::new();
    let mut bitmaps: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    let mut ea_blocks: Vec<(u64, Vec<u8>)> = Vec::new();
    let boundary = [b"/names/".as_slice(), &[b'n'; 255]].concat();
    let aliases: [(usize, &[u8]); 7] = [
        (7, b"/policy/alias"),
        (7, b"/policy/raw-\xff"),
        (7, &boundary),
        (8, b"/policy/fast-alias"),
        (9, b"/policy/slow-alias"),
        (10, b"/policy/dangling-alias"),
        (11, b"/policy/fifo-alias"),
    ];
    for (step, phase) in PHASES.into_iter().enumerate() {
        let command = format!(
            "python3 - \"$MNT\" {phase} <<'LINK_POLICY'\n{}\nLINK_POLICY\n",
            include_str!("link_policy_guest.py")
        );
        let out = guest_kernel_write(image.to_str().unwrap(), &command);
        std::fs::write(base.with_extension(format!("{phase}.jsonl")), &out.stdout).unwrap();
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
        let observations = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| l.starts_with('{'))
            .count();
        assert_eq!(observations, if phase == "names" { 71 } else { 1 });
        assert_e2fsck_clean(image.to_str().unwrap(), &format!("Linux link {phase}"));
        std::fs::copy(&image, base.with_extension(format!("{phase}.img"))).unwrap();
        let before_read = std::fs::read(&image).unwrap();
        let fs = Filesystem::mount(Arc::new(FileDevice::open_path(&image).unwrap())).unwrap();
        assert!(!fs.dev.is_writable());
        assert_eq!(
            (fs.sb.block_size(), fs.sb.inode_size, fs.csum.enabled),
            (bs, size, size == 256)
        );
        assert_ne!(fs.sb.state & crate::superblock::EXT4_VALID_FS, 0);
        assert_eq!(
            fs.sb.feature_incompat & features::Incompat::RECOVER.bits(),
            0
        );
        assert_eq!(
            fs.sb.feature_compat & features::Compat::HAS_JOURNAL.bits() != 0,
            journal
        );
        let jsb = crate::jbd2::read_superblock(&fs).unwrap();
        assert_eq!(jsb.is_some(), journal);
        if let Some(jsb) = jsb {
            assert!(jsb.is_clean());
        }
        let actual_groups = groups(&fs);
        if step == 0 {
            for path in PATHS {
                let ino = fs.lookup_path_bytes(path).unwrap();
                let (inode, raw) = fs.read_inode_verified(ino).unwrap();
                assert!(!inode.has_inline_data());
                if inode.is_dir() {
                    assert_eq!(inode.size, u64::from(bs));
                    assert_eq!(inode.flags & crate::inode::InodeFlags::INDEX.bits(), 0);
                }
                originals.push((ino, inode, raw));
            }
            assert!(originals[7].1.is_file());
            assert_eq!(
                (
                    originals[7].1.uid,
                    originals[7].1.gid,
                    originals[7].1.mode & 0o7777
                ),
                (1001, 1002, 0o640)
            );
            for index in [8, 9, 10] {
                assert!(originals[index].1.is_symlink());
            }
            assert_eq!(originals[11].1.mode & 0xf000, crate::inode::S_IFIFO);
            assert_eq!(originals[8].1.size, 7);
            assert_eq!(originals[9].1.size, 300);
            assert_eq!(originals[10].1.size, 6);
            assert_eq!(originals[8].1.blocks, 0);
            assert!(originals[9].1.blocks > 0);
            for index in [0, 3, 7] {
                let nr = originals[index].1.file_acl;
                assert_ne!(nr, 0);
                let bytes = fs.read_block(nr).unwrap();
                assert!(fs.csum.verify_xattr_block(nr, &bytes));
                assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 1);
                ea_blocks.push((nr, bytes));
            }
            initial_groups = actual_groups.clone();
            bitmaps = fs
                .groups
                .iter()
                .map(|g| {
                    (
                        fs.read_block(g.block_bitmap).unwrap(),
                        fs.read_block(g.inode_bitmap).unwrap(),
                    )
                })
                .collect();
        }
        assert_eq!(
            actual_groups, initial_groups,
            "hard links must allocate no inode/block in these non-growing profiles"
        );
        for (gi, g) in fs.groups.iter().enumerate() {
            assert_eq!(fs.read_block(g.block_bitmap).unwrap(), bitmaps[gi].0);
            assert_eq!(fs.read_block(g.inode_bitmap).unwrap(), bitmaps[gi].1);
        }
        for (nr, bytes) in &ea_blocks {
            assert_eq!(fs.read_block(*nr).unwrap(), *bytes);
        }
        let created = step.saturating_sub(1).min(7);
        let mut current = Vec::new();
        for (index, (ino, original, raw)) in originals.iter().enumerate() {
            assert_eq!(fs.lookup_path_bytes(PATHS[index]).unwrap(), *ino);
            let (inode, now) = fs.read_inode_verified(*ino).unwrap();
            let increments = aliases[..created]
                .iter()
                .filter(|(source, _)| *source == index)
                .count() as u16;
            assert_eq!(inode.links_count, original.links_count + increments);
            if index >= 7 {
                stable_raw(
                    raw,
                    &now,
                    &[
                        (8, 4),
                        (12, 4),
                        (26, 2),
                        (124, 2),
                        (130, 2),
                        (132, 4),
                        (140, 4),
                    ],
                );
                assert_eq!(times(&inode)[1], times(original)[1]);
                if step > 1 {
                    assert_eq!(times(&inode)[0], times(&previous[index])[0]);
                }
                if (2..=8).contains(&step) && aliases[step - 2].0 == index {
                    assert!(times(&inode)[2] > times(&previous[index])[2]);
                } else if step > 0 {
                    assert_eq!(times(&inode)[2], times(&previous[index])[2]);
                }
            } else if index <= 1 {
                stable_raw(
                    raw,
                    &now,
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
                assert_eq!(times(&inode)[0], times(original)[0]);
                previous_versions[index] = if step == 0 {
                    directory_version(&now)
                } else {
                    previous_versions[index]
                };
                if step > 0 {
                    let changed = (index == 0 && matches!(step, 2 | 3 | 5 | 6 | 7 | 8))
                        || (index == 1 && step == 4);
                    assert_eq!(
                        directory_version(&now),
                        previous_versions[index] + u64::from(changed)
                    );
                    for t in [1, 2] {
                        if changed {
                            assert!(times(&inode)[t] > times(&previous[index])[t]);
                        } else {
                            assert_eq!(times(&inode)[t], times(&previous[index])[t]);
                        }
                    }
                }
            } else {
                assert_eq!(
                    &now, raw,
                    "protected old parent/peer/existing inode changed"
                );
            }
            if index <= 1 {
                previous_versions[index] = directory_version(&now);
            }
            current.push(inode);
        }
        for (index, (source, path)) in aliases.iter().enumerate() {
            if index < created {
                assert_eq!(fs.lookup_path_bytes(path).unwrap(), originals[*source].0);
            } else {
                assert!(matches!(fs.lookup_path_bytes(path), Err(Error::NotFound)));
            }
        }
        let mut data = vec![0; 4096 + 20];
        let expected = [b"link-reference-data:".as_slice(), &[b'p'; 4096]].concat();
        assert_eq!(expected.len(), data.len());
        for index in [4, 7] {
            assert_eq!(
                fs.read_ino(originals[index].0, 0, &mut data).unwrap(),
                expected.len()
            );
            assert_eq!(data, expected);
        }
        previous = current;
        eprintln!("link reference bs={bs} inode={size} journal={journal} phase={phase}: observations={observations}, aliases={created}, source identity/count/ctime, parent times, exact group/bitmap/EA preservation");
        let inodes: Vec<_> = originals.iter().map(|r| r.0).collect();
        drop(fs);
        assert!(
            std::fs::read(&image).unwrap() == before_read,
            "driver reference reads modified image"
        );
        report(&image, &base, phase, &inodes);
    }
    std::fs::remove_file(image).unwrap();
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
        assert!(text.contains("Inode:") && text.contains("ctime:"), "{text}");
    }
}
