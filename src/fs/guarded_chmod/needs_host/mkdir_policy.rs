//! Reference-only Linux mkdir policy; no driver mutation or repair is invoked.
use super::setxattr::make;
use super::*;
use crate::inode::Inode;
use std::path::Path;

#[test]
fn linux_records_mkdir_policy() {
    for bs in [1024, 4096] {
        for size in [128, 256] {
            for journal in [false, true] {
                reference(bs, size, journal);
            }
        }
    }
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

fn times(inode: &Inode) -> [(i64, u32); 3] {
    [
        (inode.atime, inode.atime_nsec),
        (inode.mtime, inode.mtime_nsec),
        (inode.ctime, inode.ctime_nsec),
    ]
}

fn reference(bs: u32, size: u16, journal: bool) {
    let base = fs_ext4_test_support::temp_dir().join(format!("mkdir-policy-{bs}-{size}-{journal}"));
    let image = base.with_extension("img");
    make(&image, bs, size, journal);
    let boundary = [b"/names/".as_slice(), &[b'n'; 255]].concat();
    let existing = [b"/policy/".as_slice(), &[b'e'; 255]].concat();
    let paths: [&[u8]; 7] = [
        b"/policy",
        b"/names",
        b"/peer",
        b"/peer/kept",
        b"/policy/existingdir",
        b"/policy/regular",
        &existing,
    ];
    let children: [&[u8]; 3] = [b"/policy/child", b"/policy/raw-\xff", &boundary];
    let mut originals: Vec<(u32, Inode, Vec<u8>)> = Vec::new();
    let mut initial_groups = Vec::new();
    let mut child_raw: Vec<(u32, Vec<u8>)> = Vec::new();
    let mut parent_ea = Vec::new();
    let mut previous_parent_times = [(0, 0); 3];
    for (step, phase) in ["input", "names", "first", "second", "boundary"]
        .into_iter()
        .enumerate()
    {
        let command = format!(
            "python3 - \"$MNT\" {phase} <<'MKDIR_POLICY'\n{}\nMKDIR_POLICY\n",
            include_str!("mkdir_policy_guest.py")
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
            .filter(|line| line.starts_with('{'))
            .count();
        assert_eq!(observations, if phase == "names" { 28 } else { 1 });
        assert_e2fsck_clean(image.to_str().unwrap(), &format!("Linux mkdir {phase}"));
        std::fs::copy(&image, base.with_extension(format!("{phase}.img"))).unwrap();
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
            for (index, path) in paths.into_iter().enumerate() {
                let ino = fs.lookup_path_bytes(path).unwrap();
                let (inode, raw) = fs.read_inode_verified(ino).unwrap();
                assert!(!inode.has_inline_data());
                assert_eq!(inode.has_extents(), size == 256);
                if index != 3 && index != 5 {
                    assert!(inode.is_dir());
                    assert_eq!(inode.size, u64::from(bs));
                    assert_eq!(inode.flags & crate::inode::InodeFlags::INDEX.bits(), 0);
                }
                originals.push((ino, inode, raw));
            }
            assert_ne!(originals[0].1.file_acl, 0);
            parent_ea = fs.read_block(originals[0].1.file_acl).unwrap();
            assert!(fs
                .csum
                .verify_xattr_block(originals[0].1.file_acl, &parent_ea));
            initial_groups = actual_groups.clone();
        }
        // Failed path walks can update symlink atime. Protected peers here are
        // regular files/directories, read by the guest with O_NOATIME.
        for index in 2..originals.len() {
            let (ino, _, raw) = &originals[index];
            assert_eq!(fs.lookup_path_bytes(paths[index]).unwrap(), *ino);
            assert_eq!(fs.read_inode_verified(*ino).unwrap().1, *raw);
        }
        let created = step.saturating_sub(1);
        let mut expected_groups = initial_groups.clone();
        let mut report_inodes: Vec<_> = originals.iter().map(|r| r.0).collect();
        for (index, path) in children.into_iter().enumerate() {
            if index >= created {
                assert!(matches!(fs.lookup_path_bytes(path), Err(Error::NotFound)));
                continue;
            }
            let ino = fs.lookup_path_bytes(path).unwrap();
            let (inode, raw) = fs.read_inode_verified(ino).unwrap();
            assert!(inode.is_dir() && !inode.has_inline_data());
            assert_eq!(inode.has_extents(), size == 256);
            assert_eq!(inode.flags & crate::inode::InodeFlags::INDEX.bits(), 0);
            assert_eq!(inode.links_count, 2);
            assert_eq!(inode.size, u64::from(bs));
            assert_eq!(inode.blocks, u64::from(bs / 512));
            assert_eq!(inode.file_acl, 0);
            assert_eq!((inode.uid, inode.gid), (0, 0));
            assert_eq!(inode.mode & 0o777, if index == 1 { 0o770 } else { 0o750 });
            assert!(fs.inode_bit_is_set(ino).unwrap());
            let data = fs.map_inode_logical(&inode, 0).unwrap().unwrap();
            let ig = ((ino - 1) / fs.sb.inodes_per_group) as usize;
            let bg = ((data - u64::from(fs.sb.first_data_block))
                / u64::from(fs.sb.blocks_per_group)) as usize;
            let bit =
                (data - u64::from(fs.sb.first_data_block)) % u64::from(fs.sb.blocks_per_group);
            assert_eq!(
                fs.groups[bg].flags & crate::bgd::BgdFlags::BLOCK_UNINIT.bits(),
                0
            );
            let bitmap = fs.read_block(fs.groups[bg].block_bitmap).unwrap();
            assert_ne!(bitmap[(bit / 8) as usize] & (1 << (bit % 8)), 0);
            expected_groups[ig].1 -= 1;
            expected_groups[ig].2 += 1;
            expected_groups[bg].0 -= 1;
            assert_eq!(fs.lookup_at(ino, b".").unwrap(), ino);
            assert_eq!(
                fs.lookup_at(ino, b"..").unwrap(),
                originals[usize::from(index == 2)].0
            );
            if child_raw.len() == index {
                child_raw.push((ino, raw.clone()));
            } else {
                assert_eq!(child_raw[index], (ino, raw));
            }
            report_inodes.push(ino);
        }
        assert_eq!(actual_groups, expected_groups);
        let parent = fs.read_inode_verified(originals[0].0).unwrap().0;
        let names = fs.read_inode_verified(originals[1].0).unwrap().0;
        assert_eq!(
            parent.links_count,
            originals[0].1.links_count + created.min(2) as u16
        );
        assert_eq!(
            names.links_count,
            originals[1].1.links_count + u16::from(created == 3)
        );
        assert_eq!(
            (parent.size, parent.blocks),
            (originals[0].1.size, originals[0].1.blocks)
        );
        assert_eq!(
            (names.size, names.blocks),
            (originals[1].1.size, originals[1].1.blocks)
        );
        assert_eq!(parent.file_acl, originals[0].1.file_acl);
        assert_eq!(fs.read_block(parent.file_acl).unwrap(), parent_ea);
        assert_eq!(times(&parent)[0], times(&originals[0].1)[0]);
        assert_eq!(times(&names)[0], times(&originals[1].1)[0]);
        if step == 2 || step == 3 {
            assert!(times(&parent)[1] > previous_parent_times[1]);
            assert!(times(&parent)[2] > previous_parent_times[2]);
        }
        if step <= 1 {
            assert_eq!(times(&parent), times(&originals[0].1));
        }
        if step == 4 {
            assert_eq!(times(&parent), previous_parent_times);
            assert!(times(&names)[1] > times(&originals[1].1)[1]);
            assert!(times(&names)[2] > times(&originals[1].1)[2]);
        }
        previous_parent_times = times(&parent);
        eprintln!("mkdir reference bs={bs} inode={size} journal={journal} phase={phase}: observations={observations}, created={created}, per-group allocations/used_dirs, fields/peers/EA/checksums retained");
        drop(fs);
        report(&image, &base, phase, &report_inodes);
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
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let text = String::from_utf8(out.stdout).unwrap();
        assert!(text.contains("Inode:") && text.contains("ctime:"), "{text}");
    }
}
