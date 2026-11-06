//! Synthetic split-boundary controls; these are not Linux-clean input claims.
use super::*;

#[path = "link_index_capacity_native.rs"]
mod root_capacity;

#[derive(Clone, Copy, Debug)]
enum Case {
    FullParent,
    DuplicateBound,
    OneHash,
    NonFittingHalf,
    BadChecksum,
    BadRecord,
}

fn long(n: usize) -> Vec<u8> {
    format!("{n:04}-{}", "i".repeat(250)).into_bytes()
}

fn leaf(bs: u32, target: u32, names: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = vec![0; bs as usize];
    bytes[4..6].copy_from_slice(&((bs - 12) as u16).to_le_bytes());
    for name in names {
        crate::dir::add_entry_to_block(
            &mut bytes,
            target,
            name,
            crate::dir::DirEntryType::RegFile,
            true,
            12,
        )
        .unwrap();
    }
    bytes
}

fn fixture(bs: u32) -> (Arc<Device>, Filesystem, InodeRef, InodeRef, Arc<Clock>) {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, bs);
    let target = target(&fs);
    let ino = fs.apply_mkdir("/indexed", 0o750).unwrap();
    let parent = fs.stat_ino(ino).unwrap();
    let root_phys = fs.map_inode_logical(&parent, 0).unwrap().unwrap();
    let mut empty = leaf(bs, target.ino, &[]);
    assert!(fs
        .csum
        .patch_dir_entry_tail(ino, parent.generation, &mut empty));
    let mut buf = BlockBuffer::new(bs);
    fs.buffer_append_dir_block(&mut buf, ino, empty).unwrap();
    let (_, mut raw) = fs.buffered_inode_verified(&buf, ino).unwrap();
    raw[32..36]
        .copy_from_slice(&(parent.flags | crate::inode::InodeFlags::INDEX.bits()).to_le_bytes());
    fs.finalize_inode_raw(ino, parent.generation, &mut raw)
        .unwrap();
    fs.buffer_write_inode(&mut buf, ino, &raw).unwrap();
    let mut root = fs.read_block(root_phys).unwrap();
    root[16..18].copy_from_slice(&((bs - 12) as u16).to_le_bytes());
    root[24..].fill(0);
    root[28] = 1;
    root[29] = 8;
    root[32..34].copy_from_slice(&(((bs - 40) / 8) as u16).to_le_bytes());
    root[34..36].copy_from_slice(&1u16.to_le_bytes());
    root[36..40].copy_from_slice(&1u32.to_le_bytes());
    assert!(fs.csum.patch_dx_tail(ino, parent.generation, &mut root, 32));
    buf.put(root_phys, root);
    fs.commit_block_buffer(buf).unwrap();
    drop(fs);
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
    dev.reset();
    clock.calls.store(0, Ordering::SeqCst);
    (
        dev,
        fs,
        InodeRef::new(ino, parent.generation),
        target,
        clock,
    )
}

fn staged(
    fs: &Filesystem,
    parent: InodeRef,
    target: InodeRef,
    case: Case,
) -> (BlockBuffer, Vec<u8>) {
    let inode = fs.stat_ino(parent.ino).unwrap();
    let bs = fs.sb.block_size();
    let capacity = ((bs - 12) / 264) as usize;
    let names = match case {
        // Repeated names deliberately isolate the private planner's hash-run
        // boundary; they are not admitted public namespaces.
        Case::OneHash => vec![long(0); capacity],
        Case::NonFittingHalf => {
            let mut names = vec![long(0); capacity];
            names.push(b"s".to_vec());
            names
        }
        _ => (0..capacity).map(long).collect(),
    };
    let mut bytes = leaf(bs, target.ino, &names);
    let mut query = long(999);
    let version =
        crate::hash::effective_version(crate::hash::HashVersion::HalfMd4, fs.sb.unsigned_hash());
    if matches!(case, Case::NonFittingHalf) {
        let split = crate::htree_mut::plan_leaf_split(
            &bytes,
            version,
            &fs.sb.hash_seed,
            true,
            bs as usize,
            12,
        )
        .unwrap();
        let big_right = crate::hash::name_hash(&long(0), version, &fs.sb.hash_seed).major
            >= split.split_out_hash;
        query = (100..1000)
            .map(long)
            .find(|name| {
                (crate::hash::name_hash(name, version, &fs.sb.hash_seed).major
                    >= split.split_out_hash)
                    == big_right
            })
            .unwrap();
        let mut half = if big_right {
            split.right_bytes
        } else {
            split.left_bytes
        };
        assert!(matches!(
            crate::dir::add_entry_to_block(
                &mut half,
                target.ino,
                &query,
                crate::dir::DirEntryType::RegFile,
                true,
                12
            ),
            Err(Error::OutOfBounds)
        ));
    }
    if matches!(case, Case::BadRecord) {
        bytes[4..6].copy_from_slice(&2u16.to_le_bytes());
    }
    assert!(fs
        .csum
        .patch_dir_entry_tail(parent.ino, inode.generation, &mut bytes));
    if matches!(case, Case::BadChecksum) {
        bytes[8] ^= 1;
    }
    let root_phys = fs.map_inode_logical(&inode, 0).unwrap().unwrap();
    let leaf_phys = fs.map_inode_logical(&inode, 1).unwrap().unwrap();
    let mut root = fs.read_block(root_phys).unwrap();
    match case {
        Case::FullParent => {
            // Fill the physical route array. Repeated leaf references isolate
            // capacity handling; this is not a healthy full-root Linux fixture.
            let limit = u16::from_le_bytes(root[32..34].try_into().unwrap());
            root[34..36].copy_from_slice(&limit.to_le_bytes());
            for n in 1..usize::from(limit) {
                let at = 32 + n * 8;
                root[at..at + 4].copy_from_slice(&(n as u32 * 2).to_le_bytes());
                root[at + 4..at + 8].copy_from_slice(&1u32.to_le_bytes());
            }
        }
        Case::DuplicateBound => {
            let split = crate::htree_mut::plan_leaf_split(
                &bytes,
                version,
                &fs.sb.hash_seed,
                true,
                bs as usize,
                12,
            )
            .unwrap();
            root[34..36].copy_from_slice(&2u16.to_le_bytes());
            root[40..44].copy_from_slice(&split.split_out_hash.to_le_bytes());
            root[44..48].copy_from_slice(&1u32.to_le_bytes());
        }
        _ => {}
    }
    assert!(fs
        .csum
        .patch_dx_tail(parent.ino, inode.generation, &mut root, 32));
    let mut buf = BlockBuffer::new(bs);
    buf.put(root_phys, root);
    buf.put(leaf_phys, bytes);
    // An existing publication entry must also survive every private decline.
    buf.uninit_cleared.insert(0, 0);
    (buf, query)
}

fn private_boundary(bs: u32, case: Case) {
    let (dev, fs, parent, target, clock) = fixture(bs);
    let (mut buf, name) = staged(&fs, parent, target, case);
    let dirty = buf.dirty.clone();
    let pending = buf.uninit_cleared.clone();
    let before = dev.bytes();
    let status = fs.chmod_operation().unwrap().operation.status();
    dev.reset();
    let result = fs.buffer_split_htree_leaf_and_add_entry(
        &mut buf,
        parent.ino,
        &name,
        target.ino,
        crate::dir::DirEntryType::RegFile,
        true,
    );
    assert_eq!(buf.dirty, dirty);
    assert_eq!(buf.uninit_cleared, pending);
    assert_eq!(dev.bytes(), before);
    assert_eq!(dev.counts()[1..3], [0, 0]);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
    assert!(!fs.marked_not_clean.load(Ordering::SeqCst));
    assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
    eprintln!("private indexed boundary bs={bs} case={case:?}: result={result:?}; buffer/device/clock/marker/health unchanged");
    match case {
        Case::BadChecksum => assert!(matches!(result, Err(Error::BadChecksum { .. }))),
        Case::BadRecord => assert!(
            matches!(result, Err(Error::CorruptDirEntry(_))),
            "malformed leaf must propagate corruption, never request fallback: {result:?}"
        ),
        _ => assert!(matches!(result, Ok(false))),
    }
}

fn public_refusal(bs: u32, case: Case) {
    for dirty in [false, true] {
        let (dev, fs, parent, target, clock) = fixture(bs);
        let (mut buf, name) = staged(&fs, parent, target, case);
        buf.uninit_cleared.clear();
        fs.commit_block_buffer(buf).unwrap();
        if dirty {
            fs.mark_not_clean_once().unwrap();
        }
        let before = dev.bytes();
        let status = fs.chmod_operation().unwrap().operation.status();
        let marker = fs.marked_not_clean.load(Ordering::SeqCst);
        dev.reset();
        let _fs = with_test_mount(fs, |h, fs| {
            let path = [b"/indexed/".as_slice(), &name].concat();
            let src = CString::new("/ascii").unwrap();
            let dst = CString::new(path.clone()).unwrap();
            for entry in ENTRIES {
                let result = match entry {
                    Entry::String => fs
                        .apply_link("/ascii", std::str::from_utf8(&path).unwrap())
                        .map_err(|e| e.to_errno()),
                    Entry::Bytes => fs
                        .apply_link_bytes(b"/ascii", &path)
                        .map_err(|e| e.to_errno()),
                    Entry::At => fs
                        .apply_link_at(target, parent, &name)
                        .map_err(|e| e.to_errno()),
                    Entry::CPath => {
                        assert_eq!(unsafe { fs_ext4_link(h, src.as_ptr(), dst.as_ptr()) }, -1);
                        Err(fs_ext4_last_errno())
                    }
                    Entry::CAt => {
                        assert_eq!(
                            unsafe {
                                fs_ext4_link_at(
                                    h,
                                    target.ino,
                                    target.generation.unwrap(),
                                    parent.ino,
                                    parent.generation.unwrap(),
                                    name.as_ptr().cast(),
                                    name.len(),
                                    std::ptr::null_mut(),
                                )
                            },
                            -1
                        );
                        Err(fs_ext4_last_errno())
                    }
                };
                assert_eq!(result, Err(crate::error::errno::EIO));
                assert_eq!(dev.bytes(), before);
                assert_eq!(dev.counts()[1..3], [0, 0]);
                assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
                assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), marker);
                assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
                assert!(!fs.operation_failed.load(Ordering::SeqCst));
                eprintln!("public indexed refusal bs={bs} case={case:?} dirty={dirty} entry={entry:?}: EIO; bytes/clock/marker/health unchanged");
            }
        });
    }
}

macro_rules! boundary {
    ($name:ident, $bs:expr, $case:ident) => {
        #[test]
        fn $name() {
            private_boundary($bs, Case::$case);
        }
    };
}
boundary!(full_parent_1k, 1024, FullParent);
boundary!(full_parent_4k, 4096, FullParent);
boundary!(duplicate_bound_1k, 1024, DuplicateBound);
boundary!(duplicate_bound_4k, 4096, DuplicateBound);
boundary!(one_hash_1k, 1024, OneHash);
boundary!(one_hash_4k, 4096, OneHash);
boundary!(non_fitting_half_1k, 1024, NonFittingHalf);
boundary!(non_fitting_half_4k, 4096, NonFittingHalf);
boundary!(bad_checksum_1k, 1024, BadChecksum);
boundary!(bad_checksum_4k, 4096, BadChecksum);
boundary!(bad_record_1k, 1024, BadRecord);
boundary!(bad_record_4k, 4096, BadRecord);

#[test]
fn public_bad_checksum_1k() {
    public_refusal(1024, Case::BadChecksum);
}
#[test]
fn public_bad_checksum_4k() {
    public_refusal(4096, Case::BadChecksum);
}
#[test]
fn public_bad_record_1k() {
    public_refusal(1024, Case::BadRecord);
}
#[test]
fn public_bad_record_4k() {
    public_refusal(4096, Case::BadRecord);
}

fn public_fallback(bs: u32) {
    for capi in [false, true] {
        let (dev, fs, parent, target, clock) = fixture(bs);
        let (mut buf, name) = staged(&fs, parent, target, Case::FullParent);
        buf.uninit_cleared.clear();
        let count = (bs - 12) / 264;
        let (source, mut raw) = fs.read_inode_verified(target.ino).unwrap();
        fs.patch_inode_nlink(target.ino, &mut raw, &source, count as i32)
            .unwrap();
        fs.buffer_write_inode(&mut buf, target.ino, &raw).unwrap();
        fs.commit_block_buffer(buf).unwrap();
        let before = counters(&dev);
        let old = fs.stat_ino(parent.ino).unwrap();
        let peer = fs
            .read_inode_verified(fs.resolve(b"/peer").unwrap())
            .unwrap()
            .1;
        let fs = with_test_mount(fs, |h, fs| {
            if capi {
                assert_eq!(
                    unsafe {
                        fs_ext4_link_at(
                            h,
                            target.ino,
                            target.generation.unwrap(),
                            parent.ino,
                            parent.generation.unwrap(),
                            name.as_ptr().cast(),
                            name.len(),
                            std::ptr::null_mut(),
                        )
                    },
                    0
                );
                assert_eq!(fs_ext4_last_errno(), 0);
            } else {
                fs.apply_link_at(target, parent, &name).unwrap();
            }
            let now = fs.stat_ino(parent.ino).unwrap();
            assert_eq!(
                now.flags,
                old.flags & !crate::inode::InodeFlags::INDEX.bits()
            );
            assert_eq!(now.size, old.size + u64::from(bs));
            assert_eq!(now.blocks, old.blocks + u64::from(bs / 512));
            assert_eq!(now.links_count, old.links_count);
            assert_eq!(fs.lookup_at(parent.ino, &name).unwrap(), target.ino);
            for n in 0..count as usize {
                assert_eq!(fs.lookup_at(parent.ino, &long(n)).unwrap(), target.ino);
            }
            assert_eq!(
                fs.stat_ino(target.ino).unwrap().links_count,
                (count + 2) as u16
            );
            assert_eq!(counters(&dev), (before.0 - 1, before.1));
            assert_eq!(
                fs.read_inode_verified(fs.resolve(b"/peer").unwrap())
                    .unwrap()
                    .1,
                peer
            );
            assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
            assert_eq!(
                fs.chmod_operation().unwrap().operation.status().health,
                crate::operation::Health::Healthy
            );
        });
        fs.finish().unwrap();
        let reopened = Filesystem::mount(dev).unwrap();
        assert_eq!(reopened.lookup_at(parent.ino, &name).unwrap(), target.ino);
        assert_eq!(
            reopened.stat_ino(parent.ino).unwrap().flags & crate::inode::InodeFlags::INDEX.bits(),
            0
        );
        eprintln!("synthetic full-root fallback bs={bs} capi={capi}: INDEX dropped, one block/link, old aliases and reopened lookup retained; no Linux qualification");
    }
}

#[test]
fn public_full_parent_fallback_1k() {
    public_fallback(1024);
}
#[test]
fn public_full_parent_fallback_4k() {
    public_fallback(4096);
}
