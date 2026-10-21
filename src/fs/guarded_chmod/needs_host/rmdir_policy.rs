//! Reference-only Linux rmdir policy; the driver never emits or repairs bytes.
use super::setxattr::make;
use super::*;
use crate::inode::Inode;
use std::path::Path;

#[test]
fn linux_records_rmdir_policy() {
    for bs in [1024, 4096] {
        for size in [128, 256] {
            for journal in [false, true] {
                reference(bs, size, journal);
            }
        }
    }
}

fn group_counters(fs: &Filesystem) -> (u64, u32, u32) {
    let mut counts = (0, 0, 0);
    for gi in 0..fs.groups.len() {
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
        counts.0 += u64::from(word(12) | high(44));
        counts.1 += word(14) | high(46);
        counts.2 += word(16) | high(48);
    }
    assert_eq!(counts.0, fs.sb.free_blocks_count);
    assert_eq!(counts.1, fs.sb.free_inodes_count);
    counts
}

fn attribute_refs(fs: &Filesystem, inode: &Inode) -> u32 {
    assert_ne!(inode.file_acl, 0);
    let b = fs.read_block(inode.file_acl).unwrap();
    assert!(fs.csum.verify_xattr_block(inode.file_acl, &b));
    u32::from_le_bytes(b[4..8].try_into().unwrap())
}

fn reference(bs: u32, size: u16, journal: bool) {
    let base = fs_ext4_test_support::temp_dir().join(format!("rmdir-policy-{bs}-{size}-{journal}"));
    let image = base.with_extension("img");
    make(&image, bs, size, journal);
    let boundary = format!("/names/{}", "n".repeat(255));
    let paths = [
        "/policy",
        "/names",
        "/policy/exclusive",
        "/policy/shared",
        "/policy/peer",
        &boundary,
    ];
    let mut originals: Vec<(u32, Inode, Vec<u8>)> = Vec::new();
    let mut initial_counts = (0, 0, 0);
    for (step, phase) in ["input", "names", "exclusive", "shared"]
        .into_iter()
        .enumerate()
    {
        let command = format!(
            "python3 - \"$MNT\" {phase} <<'RMDIR_POLICY'\n{}\nRMDIR_POLICY\n",
            include_str!("rmdir_policy_guest.py")
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
        let lines = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|line| line.starts_with('{'))
            .count();
        assert_eq!(
            lines,
            if phase == "names" { 24 } else { 1 },
            "missing executed policy observations"
        );
        assert_e2fsck_clean(image.to_str().unwrap(), &format!("Linux rmdir {phase}"));
        std::fs::copy(&image, base.with_extension(format!("{phase}.img"))).unwrap();
        let fs = Filesystem::mount(Arc::new(FileDevice::open_path(&image).unwrap())).unwrap();
        assert!(!fs.dev.is_writable());
        assert_eq!(
            (fs.sb.block_size(), fs.sb.inode_size, fs.csum.enabled),
            (bs, size, size == 256)
        );
        assert_eq!(
            fs.sb.feature_compat & features::Compat::HAS_JOURNAL.bits() != 0,
            journal
        );
        let reference_journal = crate::jbd2::read_superblock(&fs).unwrap();
        assert_eq!(reference_journal.is_some(), journal);
        if let Some(reference_journal) = reference_journal {
            assert!(reference_journal.is_clean());
        }
        let counts = group_counters(&fs);
        if step == 0 {
            for path in paths {
                let ino = fs.lookup_path_bytes(path.as_bytes()).unwrap();
                let (inode, raw) = fs.read_inode_verified(ino).unwrap();
                assert!(inode.is_dir() && !inode.has_inline_data());
                assert_eq!(inode.has_extents(), size == 256);
                originals.push((ino, inode, raw));
            }
            for index in [2, 3] {
                let target = &originals[index].1;
                assert_eq!(
                    (target.links_count, target.size, target.blocks),
                    (2, u64::from(bs), u64::from(bs) / 256)
                );
            }
            assert_ne!(originals[2].1.file_acl, originals[3].1.file_acl);
            assert_eq!(originals[3].1.file_acl, originals[4].1.file_acl);
            assert_eq!(attribute_refs(&fs, &originals[2].1), 1);
            assert_eq!(attribute_refs(&fs, &originals[3].1), 2);
            initial_counts = counts;
        }
        let mut freed = 0;
        let mut deleted = 0;
        for (index, gone) in [(5, step >= 1), (2, step >= 2), (3, step >= 3)] {
            let (ino, before, _) = &originals[index];
            assert_eq!(fs.inode_bit_is_set(*ino).unwrap(), !gone);
            if gone {
                assert!(matches!(
                    fs.lookup_path_bytes(paths[index].as_bytes()),
                    Err(Error::NotFound)
                ));
                assert!(matches!(fs.live_inode((*ino).into()), Err(Error::Stale)));
                freed += before.blocks / (u64::from(bs) / 512) - u64::from(index == 3);
                deleted += 1;
            } else {
                assert_eq!(fs.lookup_path_bytes(paths[index].as_bytes()).unwrap(), *ino);
                let (after, raw) = fs.read_inode_verified(*ino).unwrap();
                assert_eq!(
                    (
                        after.blocks,
                        after.file_acl,
                        after.links_count,
                        after.generation
                    ),
                    (
                        before.blocks,
                        before.file_acl,
                        before.links_count,
                        before.generation
                    )
                );
                assert_eq!(raw, originals[index].2);
            }
        }
        assert_eq!(
            counts,
            (
                initial_counts.0 + freed,
                initial_counts.1 + deleted,
                initial_counts.2 - deleted
            )
        );
        let parent = fs.read_inode_verified(originals[0].0).unwrap().0;
        let names = fs.read_inode_verified(originals[1].0).unwrap().0;
        assert_eq!(
            parent.links_count,
            originals[0].1.links_count - (step.saturating_sub(1) as u16)
        );
        assert_eq!(
            names.links_count,
            originals[1].1.links_count - u16::from(step >= 1)
        );
        assert_eq!(
            fs.read_inode_verified(originals[4].0).unwrap().1,
            originals[4].2
        );
        assert_eq!(
            attribute_refs(&fs, &originals[4].1),
            if step >= 3 { 1 } else { 2 }
        );
        if step >= 2 {
            assert!(
                (parent.mtime, parent.mtime_nsec)
                    > (originals[0].1.mtime, originals[0].1.mtime_nsec)
            );
            assert!(
                (parent.ctime, parent.ctime_nsec)
                    > (originals[0].1.ctime, originals[0].1.ctime_nsec)
            );
        }
        eprintln!("rmdir reference bs={bs} inode={size} journal={journal} phase={phase}: observations={lines}, freed_blocks={freed}, freed_inodes={deleted}, used_dirs={}", counts.2);
        let inodes: Vec<u32> = originals.iter().map(|r| r.0).collect();
        drop(fs);
        report(&image, &base, phase, &inodes);
    }
    std::fs::remove_file(image).unwrap();
}

fn report(image: &Path, base: &Path, phase: &str, inodes: &[u32]) {
    for (label, ino) in ["parent", "names", "exclusive", "shared", "peer", "boundary"]
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
