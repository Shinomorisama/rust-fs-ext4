//! Synthetic conversion contracts; independent Linux images are qualified separately.
use super::*;

struct Fixture {
    dev: Arc<Device>,
    fs: Filesystem,
    clock: Arc<Clock>,
    parent: u32,
    target: InodeRef,
    path: String,
}

fn fixture(bs: u32, continuation: bool, journal: bool, dirty: bool) -> Fixture {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, bs);
    // Find, and actually prove, adjacent source/parent placement in one table block.
    let (path, parent, target) = (0..4)
        .find_map(|n| {
            let path = format!("/inline-{n}");
            let parent = fs.apply_mkdir(&path, 0o750).unwrap();
            let target = fs.apply_create(&format!("{path}/source"), 0o640).unwrap();
            let a = crate::bgd::locate_inode(&fs.sb, &fs.groups, parent).unwrap();
            let b = crate::bgd::locate_inode(&fs.sb, &fs.groups, target).unwrap();
            (a.0 == b.0).then_some((path, parent, target))
        })
        .expect("same-table native specimen");
    fs.apply_replace_file_content(&format!("{path}/source"), b"inline link payload")
        .unwrap();
    fs.apply_setxattr(&format!("{path}/source"), NAME, &[b'v'; 600])
        .unwrap();
    fs.apply_setxattr(&path, "user.keep", b"k").unwrap();
    fs.apply_setxattr(&path, NAME, &[b'a'; 512]).unwrap();
    let mut entries = vec![(target, b"source".as_slice(), 1)];
    if continuation {
        for name in [b"a".as_slice(), b"b", b"c", b"tail"] {
            fs.apply_link(
                &format!("{path}/source"),
                &format!("{path}/{}", std::str::from_utf8(name).unwrap()),
            )
            .unwrap();
        }
        entries.extend([
            (target, b"a".as_slice(), 1),
            (target, b"b", 1),
            (target, b"c", 1),
        ]);
    }
    inline_directory(&fs, parent, 2, &entries);
    if continuation {
        let (inode, mut raw) = fs.read_inode_verified(parent).unwrap();
        let mut data = raw[40..100].to_vec();
        let mut tail = vec![0u8; 24];
        tail[..4].copy_from_slice(&target.to_le_bytes());
        tail[4..6].copy_from_slice(&24u16.to_le_bytes());
        tail[6] = 4;
        tail[7] = 1;
        tail[8..12].copy_from_slice(b"tail");
        data.extend(tail);
        crate::inline_mut::store(&mut raw, fs.sb.inode_size, &data).unwrap();
        Filesystem::patch_inode_size_and_blocks(&mut raw, 84, inode.blocks).unwrap();
        fs.finalize_inode_raw(parent, inode.generation, &mut raw)
            .unwrap();
        let mut buf = BlockBuffer::new(bs);
        fs.buffer_write_inode(&mut buf, parent, &raw).unwrap();
        fs.commit_block_buffer(buf).unwrap();
    }
    if journal {
        // A bounded declared plain journal, using the existing native fixture pattern.
        // This is a transaction-engine test, not an e2fsck-clean fixture claim.
        let temporary = fs.apply_create("/inline-journal", 0o600).unwrap();
        let mut data = vec![0u8; 64 * bs as usize];
        for (at, value) in [
            (0, crate::jbd2::JBD2_MAGIC_NUMBER),
            (4, crate::jbd2::JBD2_SUPERBLOCK_V2),
            (12, bs),
            (16, 64),
            (20, 1),
            (24, 1),
            (64, 1),
        ] {
            data[at..at + 4].copy_from_slice(&value.to_be_bytes());
        }
        data[48..64].copy_from_slice(&fs.sb.uuid);
        data[256..272].copy_from_slice(&fs.sb.uuid);
        fs.apply_pwrite("/inline-journal", 0, &data).unwrap();
        let (inode, mut raw) = fs.read_inode_verified(temporary).unwrap();
        fs.csum.patch_inode_checksum(8, inode.generation, &mut raw);
        let mut buf = BlockBuffer::new(bs);
        fs.buffer_write_inode(&mut buf, 8, &raw).unwrap();
        let root = fs.stat_ino(2).unwrap();
        fs.buffer_remove_dir_entry(&mut buf, 2, &root, b"inline-journal")
            .unwrap();
        fs.buffer_free_inode_slot(&mut buf, temporary).unwrap();
        fs.buffer_write_inode(&mut buf, temporary, &vec![0; fs.sb.inode_size as usize])
            .unwrap();
        fs.buffer_patch_sb_counters(&mut buf, 0, 1).unwrap();
        fs.commit_block_buffer(buf).unwrap();
    }
    drop(fs);
    if journal {
        let mut bytes = dev.bytes.lock().unwrap();
        let sb = &mut bytes[1024..2048];
        let compat = u32::from_le_bytes(sb[92..96].try_into().unwrap());
        sb[92..96].copy_from_slice(&(compat | features::Compat::HAS_JOURNAL.bits()).to_le_bytes());
        sb[224..228].copy_from_slice(&8u32.to_le_bytes());
        let sum = crate::checksum::linux_crc32c(!0, &sb[..1020]);
        sb[1020..].copy_from_slice(&sum.to_le_bytes());
    }
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
    assert_eq!(fs.journal.is_some(), journal);
    let inode = fs.stat_ino(parent).unwrap();
    assert!(inode.has_inline_data());
    assert_eq!(inode.size, if continuation { 84 } else { 60 });
    assert_eq!(
        fs.read_dir_ino(parent).unwrap().len(),
        if continuation { 7 } else { 3 }
    );
    if dirty {
        fs.apply_chmod("/peer", 0o600).unwrap();
    }
    let target = InodeRef::new(target, fs.stat_ino(target).unwrap().generation);
    dev.reset();
    clock.calls.store(0, Ordering::SeqCst);
    Fixture {
        dev,
        fs,
        clock,
        parent,
        target,
        path,
    }
}

fn invoke(
    identity: (u32, InodeRef, &str),
    h: *mut fs_ext4_fs_t,
    fs: &Filesystem,
    entry: Entry,
    name: &[u8],
) -> std::result::Result<(), i32> {
    let (parent, target, path) = identity;
    let src = format!("{}/source", path);
    let dst = [path.as_bytes(), b"/", name].concat();
    match entry {
        Entry::String => fs
            .apply_link(&src, std::str::from_utf8(&dst).unwrap())
            .map_err(|e| e.to_errno()),
        Entry::Bytes => fs
            .apply_link_bytes(src.as_bytes(), &dst)
            .map_err(|e| e.to_errno()),
        Entry::At => fs
            .apply_link_at(target, parent, name)
            .map_err(|e| e.to_errno()),
        Entry::CPath => {
            let src = CString::new(src).unwrap();
            let dst = CString::new(dst).unwrap();
            let rc = unsafe { fs_ext4_link(h, src.as_ptr(), dst.as_ptr()) };
            if rc == 0 {
                assert_eq!(fs_ext4_last_errno(), 0);
                Ok(())
            } else {
                assert_eq!(rc, -1);
                Err(fs_ext4_last_errno())
            }
        }
        Entry::CAt => {
            let mut attr: fs_ext4_attr_t = unsafe { std::mem::zeroed() };
            let rc = c_at(h, parent, target, name, &mut attr);
            if rc == 0 {
                assert_eq!(attr.inode, target.ino);
                assert_eq!(fs_ext4_last_errno(), 0);
                Ok(())
            } else {
                assert_eq!(rc, -1);
                Err(fs_ext4_last_errno())
            }
        }
    }
}

fn stable_bytes(a: &[u8], b: &[u8], fields: &[(usize, usize)]) {
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    for &(at, len) in fields {
        a[at..at + len].fill(0);
        b[at..at + len].fill(0);
    }
    assert_eq!(a, b);
}

#[test]
fn fit_convert_and_postfit_preserve_same_table_metadata_and_exact_allocation() {
    for bs in [1024, 4096] {
        for continuation in [false, true] {
            for journal in [false, true] {
                for entry in ENTRIES {
                    let f = fixture(bs, continuation, journal, false);
                    let fs = with_test_mount(f.fs, |h, fs| {
                        let initial = counters(&f.dev);
                        let initial_groups = group_counters(fs);
                        let peer = fs.resolve(b"/peer").unwrap();
                        let peer_raw = fs.read_inode_verified(peer).unwrap().1;
                        let mut prior_names = fs.read_dir_ino(f.parent).unwrap();
                        let original_source = fs.read_inode_verified(f.target.ino).unwrap().1;
                        let initial_parent = fs.stat_ino(f.parent).unwrap();
                        let parent_ea = fs.read_block(initial_parent.file_acl).unwrap();
                        let source_ea = fs
                            .read_block(fs.stat_ino(f.target.ino).unwrap().file_acl)
                            .unwrap();
                        let inode_bitmap = fs.read_block(fs.groups[0].inode_bitmap).unwrap();
                        let bitmap = fs.read_block(fs.groups[0].block_bitmap).unwrap();
                        for (phase, name) in [b"fit".to_vec(), vec![b'n'; 255], b"post".to_vec()]
                            .iter()
                            .enumerate()
                        {
                            let (old, raw) = fs.read_inode_verified(f.parent).unwrap();
                            let (table, off) =
                                crate::bgd::locate_inode(&fs.sb, &fs.groups, f.parent).unwrap();
                            let (target_table, target_off) =
                                crate::bgd::locate_inode(&fs.sb, &fs.groups, f.target.ino).unwrap();
                            assert_eq!(table, target_table);
                            let table_before = fs.read_block(table).unwrap();
                            let before_clock = f.clock.calls.load(Ordering::SeqCst);
                            invoke((f.parent, f.target, &f.path), h, fs, entry, name).unwrap();
                            assert_eq!(f.clock.calls.load(Ordering::SeqCst), before_clock + 1);
                            let (now, after) = fs.read_inode_verified(f.parent).unwrap();
                            assert_eq!(now.has_inline_data(), phase == 0);
                            assert_eq!(now.links_count, initial_parent.links_count);
                            assert_eq!(now.generation, initial_parent.generation);
                            assert_eq!(now.file_acl, initial_parent.file_acl);
                            assert_eq!(
                                now.blocks,
                                initial_parent.blocks + u64::from(phase > 0) * u64::from(bs / 512)
                            );
                            assert_eq!(
                                counters(&f.dev),
                                (initial.0 - u64::from(phase > 0), initial.1)
                            );
                            assert_eq!(now.mtime, fs.stat_ino(f.target.ino).unwrap().ctime);
                            assert_eq!(now.ctime, now.mtime);
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
                            if phase < 2 {
                                allowed.extend([(40, 60), (160, 96)]);
                            }
                            if phase == 1 {
                                allowed.extend([(4, 4), (28, 4), (32, 4), (108, 4), (116, 2)]);
                            }
                            stable_bytes(&raw, &after, &allowed);
                            stable_bytes(
                                &table_before,
                                &fs.read_block(table).unwrap(),
                                &[(off as usize, 256), (target_off as usize, 256)],
                            );
                            assert_eq!(
                                group_counters(fs),
                                (
                                    initial_groups.0 - u64::from(phase > 0),
                                    initial_groups.1,
                                    initial_groups.2
                                )
                            );
                            assert_eq!(
                                u32::from_le_bytes(after[36..40].try_into().unwrap()),
                                u32::from_le_bytes(raw[36..40].try_into().unwrap()) + 1
                            );
                            if phase > 0 {
                                assert_eq!(
                                    now.flags,
                                    (initial_parent.flags
                                        & !crate::inode::InodeFlags::INLINE_DATA.bits())
                                        | crate::inode::InodeFlags::EXTENTS.bits()
                                );
                                assert_eq!(now.size, u64::from(bs));
                                assert_eq!(fs.map_inode_logical(&now, 1).unwrap(), None);
                                let physical = fs.map_inode_logical(&now, 0).unwrap().unwrap();
                                let mut expected = bitmap.clone();
                                let (_, bit, _) = fs.group_chunks(physical, 1).unwrap()[0];
                                assert_eq!(expected[bit as usize / 8] & (1 << (bit % 8)), 0);
                                expected[bit as usize / 8] |= 1 << (bit % 8);
                                assert_eq!(
                                    fs.read_block(fs.groups[0].block_bitmap).unwrap(),
                                    expected
                                );
                                assert_eq!(
                                    crate::xattr::get(
                                        fs.dev.as_ref(),
                                        &now,
                                        &after,
                                        fs.sb.inode_size,
                                        bs,
                                        "system.data"
                                    )
                                    .unwrap(),
                                    None
                                );
                            } else {
                                assert_eq!(now.flags, old.flags);
                                assert_eq!(
                                    fs.read_block(fs.groups[0].block_bitmap).unwrap(),
                                    bitmap
                                );
                            }
                            assert_eq!(
                                crate::xattr::get(
                                    fs.dev.as_ref(),
                                    &now,
                                    &after,
                                    fs.sb.inode_size,
                                    bs,
                                    "user.keep"
                                )
                                .unwrap(),
                                Some(b"k".to_vec())
                            );
                            assert_eq!(fs.read_block(now.file_acl).unwrap(), parent_ea);
                            assert_eq!(
                                fs.read_block(fs.stat_ino(f.target.ino).unwrap().file_acl)
                                    .unwrap(),
                                source_ea
                            );
                            assert_eq!(fs.read_inode_verified(peer).unwrap().1, peer_raw);
                            assert_eq!(
                                fs.read_block(fs.groups[0].inode_bitmap).unwrap(),
                                inode_bitmap
                            );
                            source_preserved(
                                fs,
                                f.target.ino,
                                &original_source,
                                u16::from_le_bytes(original_source[26..28].try_into().unwrap())
                                    + phase as u16,
                            );
                            let mut payload = [0u8; 19];
                            assert_eq!(
                                fs.read_ino(f.target, 0, &mut payload).unwrap(),
                                payload.len()
                            );
                            assert_eq!(&payload, b"inline link payload");
                            for e in &prior_names {
                                assert_eq!(fs.lookup_at(f.parent, &e.name).unwrap(), e.inode);
                                let found = fs.find_dir_entry(f.parent, &now, &e.name).unwrap();
                                assert_eq!((found.inode, found.file_type), (e.inode, e.file_type));
                            }
                            assert_eq!(fs.lookup_at(f.parent, name).unwrap(), f.target.ino);
                            assert_eq!(
                                fs.find_dir_entry(f.parent, &now, name).unwrap().file_type,
                                crate::dir::DirEntryType::RegFile
                            );
                            prior_names = fs.read_dir_ino(f.parent).unwrap();
                        }
                    });
                    fs.finish().unwrap();
                    eprintln!("native inline {bs}/{continuation}/{journal}/{entry:?}: fit/convert/postfit, same-table and exact one-bit allocation");
                }
            }
        }
    }
}

fn gated(f: &Fixture) -> Arc<ReadbackDevice> {
    Arc::new(ReadbackDevice {
        inner: f.dev.clone(),
        read: AtomicUsize::new(0),
        at: AtomicUsize::new(0),
        fail: AtomicBool::new(false),
        hook: Mutex::new(None),
    })
}

#[test]
fn every_conversion_planning_read_error_leaves_state_and_clock_untouched() {
    let mut total = 0;
    for bs in [1024, 4096] {
        for journal in [false, true] {
            let f = fixture(bs, true, journal, false);
            let gate = gated(&f);
            drop(f.fs);
            let fs = Filesystem::mount_inner(gate.clone(), false, 0, f.clock.clone()).unwrap();
            gate.read.store(0, Ordering::SeqCst);
            let before = f.dev.bytes();
            let plan = fs.plan_link_at(f.target, f.parent, &[b'n'; 255]).unwrap();
            let count = gate.read.load(Ordering::SeqCst);
            assert!(count > 0);
            drop(plan);
            drop(fs);
            assert_eq!(f.dev.bytes(), before);
            for position in 1..=count {
                let fs = Filesystem::mount_inner(gate.clone(), false, 0, f.clock.clone()).unwrap();
                gate.read.store(0, Ordering::SeqCst);
                gate.at.store(position, Ordering::SeqCst);
                gate.fail.store(true, Ordering::SeqCst);
                f.dev.reset();
                f.clock.calls.store(0, Ordering::SeqCst);
                let status = fs.chmod_operation().unwrap().operation.status();
                assert!(fs.plan_link_at(f.target, f.parent, &[b'n'; 255]).is_err());
                assert_eq!(gate.read.load(Ordering::SeqCst), position);
                assert_eq!(&f.dev.counts()[1..3], &[0, 0]);
                assert_eq!(f.clock.calls.load(Ordering::SeqCst), 0);
                assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
                assert_eq!(f.dev.bytes(), before);
                gate.at.store(0, Ordering::SeqCst);
                gate.fail.store(false, Ordering::SeqCst);
                drop(fs);
                total += 1;
            }
            eprintln!("inline planning reads {bs}/{journal}: {count} injected failures, no device/clock/health delta");
        }
    }
    assert!(total > 0);
}

fn effect_faults(flush: bool, unwind: bool) {
    let mut total = 0;
    for bs in [1024, 4096] {
        for journal in [false, true] {
            for dirty in [false, true] {
                for c in [false, true] {
                    let f = fixture(bs, true, journal, dirty);
                    let entry = if c { Entry::CAt } else { Entry::At };
                    let fs = with_test_mount(f.fs, |h, fs| {
                        invoke((f.parent, f.target, &f.path), h, fs, entry, &[b'n'; 255]).unwrap()
                    });
                    let index = if flush { 2 } else { 1 };
                    let count = f.dev.counts()[index];
                    assert!(count > 0);
                    drop(fs);
                    for position in 1..=count {
                        let f = fixture(bs, true, journal, dirty);
                        if unwind {
                            f.dev.panic_flush.store(position, Ordering::SeqCst);
                        } else if flush {
                            f.dev.fail_flush.store(position, Ordering::SeqCst);
                        } else {
                            f.dev.fail_write.store(position, Ordering::SeqCst);
                        }
                        let fs = if c {
                            with_test_mount(f.fs, |h, fs| {
                                assert_eq!(
                                    invoke(
                                        (f.parent, f.target, &f.path),
                                        h,
                                        fs,
                                        entry,
                                        &[b'n'; 255]
                                    ),
                                    Err(crate::error::errno::EIO)
                                )
                            })
                        } else {
                            if unwind {
                                assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                                    || f.fs.apply_link_at(f.target, f.parent, &[b'n'; 255])
                                ))
                                .is_err());
                            } else {
                                assert!(f
                                    .fs
                                    .apply_link_at(f.target, f.parent, &[b'n'; 255])
                                    .is_err());
                            }
                            f.fs
                        };
                        assert!(f.dev.counts()[index] >= position);
                        assert!(retired_observation(&f.dev, fs));
                        total += 1;
                    }
                    eprintln!("inline effect faults bs={bs} journal={journal} dirty={dirty} c={c} flush={flush} unwind={unwind}: {count} reached and retired");
                }
            }
        }
    }
    assert!(total > 0);
    eprintln!("inline effect faults total={total}");
}

#[test]
fn every_conversion_write_error_retires_rust_and_c_mounts() {
    effect_faults(false, false);
}
#[test]
fn every_conversion_flush_error_retires_rust_and_c_mounts() {
    effect_faults(true, false);
}
#[test]
fn every_conversion_flush_unwind_retires_rust_and_c_mounts() {
    effect_faults(true, true);
}

#[test]
fn c_conversion_readback_error_preserves_committed_alias_and_output_sentinels() {
    for bs in [1024, 4096] {
        for journal in [false, true] {
            let f = fixture(bs, true, journal, false);
            let gate = gated(&f);
            drop(f.fs);
            let fs = Filesystem::mount_inner(gate.clone(), false, 0, f.clock.clone()).unwrap();
            gate.read.store(0, Ordering::SeqCst);
            let fs = with_test_mount(fs, |h, _| {
                assert_eq!(
                    c_at(h, f.parent, f.target, &[b'n'; 255], std::ptr::null_mut()),
                    0
                )
            });
            let count = gate.read.load(Ordering::SeqCst);
            assert!(count > 0);
            drop(fs);
            let f = fixture(bs, true, journal, false);
            let gate = gated(&f);
            drop(f.fs);
            let fs = Filesystem::mount_inner(gate.clone(), false, 0, f.clock.clone()).unwrap();
            let before = fs.stat_ino(f.target).unwrap();
            let free = counters(&f.dev);
            gate.read.store(0, Ordering::SeqCst);
            gate.at.store(count + 1, Ordering::SeqCst);
            gate.fail.store(true, Ordering::SeqCst);
            let fs = with_test_mount(fs, |h, fs| {
                let mut attr: fs_ext4_attr_t = unsafe { std::mem::zeroed() };
                attr.inode = 0xfeed1234;
                attr.generation = 0x13572468;
                assert_eq!(c_at(h, f.parent, f.target, &[b'n'; 255], &mut attr), -1);
                assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
                assert_eq!((attr.inode, attr.generation), (0xfeed1234, 0x13572468));
                assert_eq!(gate.read.load(Ordering::SeqCst), count + 1);
                gate.at.store(0, Ordering::SeqCst);
                gate.fail.store(false, Ordering::SeqCst);
                assert_eq!(fs.lookup_at(f.parent, &[b'n'; 255]).unwrap(), f.target.ino);
                assert!(!fs.stat_ino(f.parent).unwrap().has_inline_data());
                assert_eq!(
                    fs.stat_ino(f.target).unwrap().links_count,
                    before.links_count + 1
                );
                assert_eq!(counters(&f.dev), (free.0 - 1, free.1));
            });
            fs.finish().unwrap();
        }
    }
}

#[test]
fn malformed_inline_and_no_space_refuse_before_clock_marker_or_device_effect() {
    for bs in [1024, 4096] {
        for case in 0..4 {
            let mut f = fixture(bs, true, false, false);
            let (inode, mut raw) = f.fs.read_inode_verified(f.parent).unwrap();
            let mut buf = BlockBuffer::new(bs);
            match case {
                0 => raw[48..50].copy_from_slice(&0u16.to_le_bytes()),
                1 => {
                    let head = raw[40..100].to_vec();
                    crate::inline_mut::strip(&mut raw, f.fs.sb.inode_size).unwrap();
                    raw[40..100].copy_from_slice(&head);
                }
                2 => {
                    let mut data = raw[40..100].to_vec();
                    data.extend([0; 4]);
                    crate::inline_mut::store(&mut raw, f.fs.sb.inode_size, &data).unwrap();
                }
                _ => {
                    for (g, desc) in f.fs.groups.iter().enumerate() {
                        buf.put(desc.block_bitmap, vec![255; bs as usize]);
                        f.fs.buffer_refresh_bitmap_csum(&mut buf, g, false).unwrap();
                        f.fs.buffer_patch_bgd_counters(
                            &mut buf,
                            g,
                            -i32::try_from(desc.free_blocks_count).unwrap(),
                            0,
                            0,
                        )
                        .unwrap();
                    }
                    f.fs.buffer_patch_sb_counters(&mut buf, -(f.fs.sb.free_blocks_count as i64), 0)
                        .unwrap();
                }
            }
            // Missing/short system.data is tested while retaining the declared inline size/flag.
            if case == 1 {
                raw[32..36].copy_from_slice(&inode.flags.to_le_bytes());
            }
            f.fs.finalize_inode_raw(f.parent, inode.generation, &mut raw)
                .unwrap();
            f.fs.buffer_write_inode(&mut buf, f.parent, &raw).unwrap();
            f.fs.commit_block_buffer(buf).unwrap();
            drop(f.fs);
            f.fs = Filesystem::mount_with_runtime(f.dev.clone(), f.clock.clone()).unwrap();
            f.dev.reset();
            f.clock.calls.store(0, Ordering::SeqCst);
            let before = f.dev.bytes();
            let status = f.fs.chmod_operation().unwrap().operation.status();
            let result = f.fs.apply_link_at(f.target, f.parent, &[b'n'; 255]);
            if case == 3 {
                // The existing shared allocator conservatively classifies an
                // exhausted search as Corrupt. Preserve that contract here;
                // Linux-compatible ENOSPC classification is not established.
                assert!(
                    matches!(
                        result,
                        Err(Error::Corrupt(
                            "no group has a contiguous free run of this size"
                        ))
                    ),
                    "{result:?}"
                );
            } else {
                assert!(
                    matches!(
                        result,
                        Err(Error::Corrupt(_)) | Err(Error::CorruptDirEntry(_))
                    ),
                    "{result:?}"
                );
            }
            assert_eq!(&f.dev.counts()[1..3], &[0, 0]);
            assert_eq!(f.clock.calls.load(Ordering::SeqCst), 0);
            assert_eq!(f.dev.bytes(), before);
            assert_eq!(f.fs.chmod_operation().unwrap().operation.status(), status);
        }
    }
}

#[test]
fn conversion_publication_unwind_latches_after_the_alias_and_debit_commit() {
    for bs in [1024, 4096] {
        for journal in [false, true] {
            let f = fixture(bs, true, journal, false);
            drop(f.fs);
            let gate = Arc::new(CallbackDevice {
                inner: f.dev.clone(),
                hook: Mutex::new(None),
                phase: 2,
                after: AtomicUsize::new(1),
            });
            let fs =
                Arc::new(Filesystem::mount_inner(gate.clone(), false, 0, f.clock.clone()).unwrap());
            let prior = fs.stat_ino(f.target).unwrap().links_count;
            let parent_location = crate::bgd::locate_inode(&fs.sb, &fs.groups, f.parent).unwrap();
            let target_location =
                crate::bgd::locate_inode(&fs.sb, &fs.groups, f.target.ino).unwrap();
            let free = counters(&f.dev);
            let weak = Arc::downgrade(&fs);
            *gate.hook.lock().unwrap() = Some(Arc::new(move || {
                let fs = weak.upgrade().unwrap();
                assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let _held = fs.uninit_cleared.lock().unwrap();
                    panic!("conversion publication poison after last planning reads");
                }))
                .is_err());
            }));
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fs.apply_link_at(
                    f.target,
                    f.parent,
                    &[b'n'; 255]
                )))
                .is_err()
            );
            // Inspect committed bytes, not APIs on the deliberately poisoned,
            // retired mount. A post-fault lookup can itself consult that lock.
            let bytes = f.dev.bytes();
            let at = parent_location.0 as usize * bs as usize + parent_location.1 as usize;
            let raw = &bytes[at..at + 256];
            let parent = crate::inode::Inode::parse(raw).unwrap();
            assert!(fs.csum.verify_inode(f.parent, parent.generation, raw));
            assert!(!parent.has_inline_data());
            let header = crate::extent::ExtentHeader::parse(&parent.block).unwrap();
            assert_eq!((header.depth, header.entries), (0, 1));
            let physical = u64::from(u32::from_le_bytes(parent.block[20..24].try_into().unwrap()))
                | (u64::from(u16::from_le_bytes(parent.block[18..20].try_into().unwrap())) << 32);
            let block =
                &bytes[physical as usize * bs as usize..(physical as usize + 1) * bs as usize];
            let entries = crate::dir::parse_block_verified(
                block,
                true,
                f.parent,
                parent.generation,
                &fs.csum,
            )
            .unwrap();
            assert_eq!(
                entries
                    .iter()
                    .find(|e| e.name == vec![b'n'; 255])
                    .unwrap()
                    .inode,
                f.target.ino
            );
            let at = target_location.0 as usize * bs as usize + target_location.1 as usize;
            let raw = &bytes[at..at + 256];
            let target = crate::inode::Inode::parse(raw).unwrap();
            assert!(fs.csum.verify_inode(f.target.ino, target.generation, raw));
            assert_eq!(target.links_count, prior + 1);
            assert_eq!(counters(&f.dev), (free.0 - 1, free.1));
            assert!(retired_observation(
                &f.dev,
                Arc::try_unwrap(fs).ok().unwrap()
            ));
        }
    }
}
