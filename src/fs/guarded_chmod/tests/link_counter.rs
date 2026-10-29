//! Current counter arithmetic and preservation contracts, not Linux wrap qualification.
use super::*;

const DATA: &[u8] = b"counter native source";

struct Fixture {
    dev: Arc<Device>,
    fs: Filesystem,
    clock: Arc<Clock>,
    parent: u32,
    target: InodeRef,
    path: String,
}

fn fixture(
    flavor: features::FsFlavor,
    bs: u32,
    extra: Option<u16>,
    low: u32,
    high: u32,
) -> Fixture {
    let (dev, fs, _, clock) = specimen(flavor, bs);
    let (path, parent, target) = (0..4)
        .find_map(|n| {
            let path = format!("/counter-{n}");
            let parent = fs.apply_mkdir(&path, 0o750).unwrap();
            let target = fs.apply_create(&format!("{path}/source"), 0o640).unwrap();
            let a = crate::bgd::locate_inode(&fs.sb, &fs.groups, parent).unwrap();
            let b = crate::bgd::locate_inode(&fs.sb, &fs.groups, target).unwrap();
            (a.0 == b.0).then_some((path, parent, target))
        })
        .expect("bounded same-table counter fixture");
    fs.apply_replace_file_content(&format!("{path}/source"), DATA)
        .unwrap();
    fs.apply_setxattr(&format!("{path}/source"), NAME, &[b'v'; 600])
        .unwrap();
    let (inode, mut raw) = fs.read_inode_verified(parent).unwrap();
    assert_eq!(inode.file_acl, 0);
    assert!(raw[160.min(raw.len())..].iter().all(|b| *b == 0));
    if let Some(extra) = extra {
        assert_eq!(raw.len(), 256);
        raw[128..130].copy_from_slice(&extra.to_le_bytes());
        raw[152..156].copy_from_slice(&high.to_le_bytes());
    } else {
        assert_eq!(raw.len(), 128);
    }
    raw[36..40].copy_from_slice(&low.to_le_bytes());
    fs.finalize_inode_raw(parent, inode.generation, &mut raw)
        .unwrap();
    let mut buf = BlockBuffer::new(bs);
    fs.buffer_write_inode(&mut buf, parent, &raw).unwrap();
    fs.commit_block_buffer(buf).unwrap();
    fs.finish().unwrap();
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
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
    fs: &Filesystem,
    h: *mut fs_ext4_fs_t,
    entry: Entry,
    name: &[u8],
) -> std::result::Result<(), i32> {
    let (parent, target, path) = identity;
    let source = format!("{}/source", path);
    let dst = [path.as_bytes(), b"/", name].concat();
    match entry {
        Entry::String => fs
            .apply_link(&source, std::str::from_utf8(&dst).unwrap())
            .map_err(|e| e.to_errno()),
        Entry::Bytes => fs
            .apply_link_bytes(source.as_bytes(), &dst)
            .map_err(|e| e.to_errno()),
        Entry::At => fs
            .apply_link_at(target, parent, name)
            .map_err(|e| e.to_errno()),
        Entry::CPath => {
            let source = CString::new(source).unwrap();
            let dst = CString::new(dst).unwrap();
            if unsafe { fs_ext4_link(h, source.as_ptr(), dst.as_ptr()) } == 0 {
                Ok(())
            } else {
                Err(fs_ext4_last_errno())
            }
        }
        Entry::CAt => {
            let mut attr: fs_ext4_attr_t = unsafe { std::mem::zeroed() };
            if c_at(h, parent, target, name, &mut attr) == 0 {
                assert_eq!(attr.inode, target.ino);
                assert_eq!(attr.generation, target.generation.unwrap());
                Ok(())
            } else {
                Err(fs_ext4_last_errno())
            }
        }
    }
}

fn stable(a: &[u8], b: &[u8], fields: &[(usize, usize)]) {
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    for &(at, len) in fields {
        if at + len <= a.len() {
            a[at..at + len].fill(0);
            b[at..at + len].fill(0);
        }
    }
    assert_eq!(a, b);
}

fn sequence(
    flavor: features::FsFlavor,
    bs: u32,
    extra: Option<u16>,
    high: u32,
    expected: [(u32, u32); 3],
) -> usize {
    let mut successful = 0;
    for entry in ENTRIES {
        let Fixture {
            dev,
            fs,
            clock,
            parent,
            target,
            path,
        } = fixture(flavor, bs, extra, u32::MAX - 1, high);
        let fs = with_test_mount(fs, |h, fs| {
            let free = counters(&dev);
            let groups = group_counters(fs);
            let original = fs.read_inode_verified(target.ino).unwrap().1;
            let ea = fs.stat_ino(target).unwrap().file_acl;
            let ea_bytes = fs.read_block(ea).unwrap();
            let peer = fs.resolve(b"/peer").unwrap();
            let peer_raw = fs.read_inode_verified(peer).unwrap().1;
            let bitmaps = (
                fs.read_block(fs.groups[0].inode_bitmap).unwrap(),
                fs.read_block(fs.groups[0].block_bitmap).unwrap(),
            );
            for (phase, (low, high)) in expected.into_iter().enumerate() {
                let before = fs.read_inode_verified(parent).unwrap().1;
                let (table, at) = crate::bgd::locate_inode(&fs.sb, &fs.groups, parent).unwrap();
                let (other, target_at) =
                    crate::bgd::locate_inode(&fs.sb, &fs.groups, target.ino).unwrap();
                assert_eq!(table, other);
                let table_before = fs.read_block(table).unwrap();
                clock.now.store(STAMP + phase as i64, Ordering::SeqCst);
                let calls = clock.calls.load(Ordering::SeqCst);
                let name = format!("alias-{phase}");
                invoke((parent, target, &path), fs, h, entry, name.as_bytes()).unwrap();
                successful += 1;
                assert_eq!(clock.calls.load(Ordering::SeqCst), calls + 1);
                let (parent_now, after) = fs.read_inode_verified(parent).unwrap();
                assert_eq!(u32::from_le_bytes(after[36..40].try_into().unwrap()), low);
                if after.len() >= 156 {
                    assert_eq!(
                        u32::from_le_bytes(after[152..156].try_into().unwrap()),
                        high
                    );
                }
                let mut allowed = vec![(12, 4), (16, 4), (36, 4)];
                if fs.csum.enabled {
                    allowed.push((124, 2));
                    if extra.is_some_and(|n| n >= 4) {
                        allowed.push((130, 2));
                    }
                }
                if extra.is_some_and(|n| n >= 8) {
                    allowed.push((132, 4));
                }
                if extra.is_some_and(|n| n >= 12) {
                    allowed.push((136, 4));
                }
                if extra.is_some_and(|n| n >= 28) {
                    allowed.push((152, 4));
                }
                stable(&before, &after, &allowed);
                stable(
                    &table_before,
                    &fs.read_block(table).unwrap(),
                    &[
                        (at as usize, fs.sb.inode_size as usize),
                        (target_at as usize, fs.sb.inode_size as usize),
                    ],
                );
                assert_eq!(
                    (parent_now.ctime, parent_now.mtime),
                    (STAMP + phase as i64, STAMP + phase as i64)
                );
                assert_eq!(fs.stat_ino(target).unwrap().ctime, STAMP + phase as i64);
                source_preserved(
                    fs,
                    target.ino,
                    &original,
                    u16::from_le_bytes(original[26..28].try_into().unwrap()) + phase as u16,
                );
                for prior in 0..=phase {
                    assert_eq!(
                        fs.lookup_at(parent, format!("alias-{prior}").as_bytes())
                            .unwrap(),
                        target.ino
                    );
                }
                assert_eq!(fs.lookup_at(parent, b"source").unwrap(), target.ino);
                let mut data = vec![0; DATA.len()];
                assert_eq!(fs.read_ino(target, 0, &mut data).unwrap(), DATA.len());
                assert_eq!(data, DATA);
                assert_eq!(counters(&dev), free);
                assert_eq!(group_counters(fs), groups);
                assert_eq!(fs.read_block(fs.groups[0].inode_bitmap).unwrap(), bitmaps.0);
                assert_eq!(fs.read_block(fs.groups[0].block_bitmap).unwrap(), bitmaps.1);
                assert_eq!(fs.read_block(ea).unwrap(), ea_bytes);
                assert_eq!(fs.read_inode_verified(peer).unwrap().1, peer_raw);
            }
        });
        fs.finish().unwrap();
    }
    successful
}

#[test]
fn low_word_carry_in_128_byte_inodes_preserves_identity_and_allocation() {
    let mut successes = 0;
    for flavor in [features::FsFlavor::Ext2, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            successes += sequence(flavor, bs, None, 0, [(u32::MAX, 0), (0, 0), (1, 0)]);
        }
    }
    assert_eq!(successes, 60);
    eprintln!("native counter 128-byte boundaries: {successes} links, all five forms");
}

#[test]
fn covered_high_word_carry_and_full_width_wrap_preserve_adjacent_fields() {
    let mut successes = 0;
    for bs in [1024, 4096] {
        for extra in [28, 32] {
            successes += sequence(
                features::FsFlavor::Ext4,
                bs,
                Some(extra),
                0x13572468,
                [(u32::MAX, 0x13572468), (0, 0x13572469), (1, 0x13572469)],
            );
            successes += sequence(
                features::FsFlavor::Ext4,
                bs,
                Some(extra),
                u32::MAX,
                [(u32::MAX, u32::MAX), (0, 0), (1, 0)],
            );
        }
    }
    assert_eq!(successes, 120);
    eprintln!(
        "native covered counter boundaries: {successes} links, both coverage sizes and full wrap"
    );
}

#[test]
fn uncovered_high_word_bytes_are_not_counter_storage() {
    let mut successes = 0;
    for bs in [1024, 4096] {
        for (extra, canary) in [(0, 0x5a5a5a5a), (24, crate::xattr::EXT4_XATTR_MAGIC)] {
            successes += sequence(
                features::FsFlavor::Ext4,
                bs,
                Some(extra),
                canary,
                [(u32::MAX, canary), (0, canary), (1, canary)],
            );
        }
    }
    assert_eq!(successes, 60);
    eprintln!("native uncovered counter bytes: {successes} links, both declared short-area cases");
}

#[test]
fn boundary_refusals_preserve_counter_clock_bytes_and_health() {
    let mut refusals = 0;
    for bs in [1024, 4096] {
        for (flavor, extra, high) in [
            (features::FsFlavor::Ext2, None, 0),
            (features::FsFlavor::Ext3, None, 0),
            (features::FsFlavor::Ext4, Some(0), 0x5a5a5a5a),
            (
                features::FsFlavor::Ext4,
                Some(24),
                crate::xattr::EXT4_XATTR_MAGIC,
            ),
            (features::FsFlavor::Ext4, Some(28), u32::MAX),
            (features::FsFlavor::Ext4, Some(32), u32::MAX),
        ] {
            let f = fixture(flavor, bs, extra, u32::MAX, high);
            let Fixture {
                dev,
                fs,
                clock,
                parent,
                target,
                ..
            } = f;
            let fs = with_test_mount(fs, |h, fs| {
                let bytes = dev.bytes();
                let status = fs.chmod_operation().unwrap().operation.status();
                for c in [false, true] {
                    for name in [
                        b"".as_slice(),
                        b"bad/name",
                        b"bad\0name",
                        &[b'x'; 256],
                        b"source",
                    ] {
                        let got = if c {
                            let mut attr: fs_ext4_attr_t = unsafe { std::mem::zeroed() };
                            let rc = c_at(h, parent, target, name, &mut attr);
                            assert_eq!(rc, -1);
                            fs_ext4_last_errno()
                        } else {
                            fs.apply_link_at(target, parent, name)
                                .unwrap_err()
                                .to_errno()
                        };
                        let want = if name == b"source" {
                            crate::error::errno::EEXIST
                        } else if name.len() > 255 {
                            crate::error::errno::ENAMETOOLONG
                        } else {
                            crate::error::errno::EINVAL
                        };
                        assert_eq!(got, want);
                        refusals += 1;
                        assert_eq!(dev.bytes(), bytes);
                        assert_eq!(&dev.counts()[1..3], &[0, 0]);
                        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
                        assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
                    }
                }
            });
            fs.finish().unwrap();
        }
    }
    assert_eq!(refusals, 120);
    eprintln!("native counter-boundary refusals: {refusals}, no clock/effect/health/byte delta");
}
