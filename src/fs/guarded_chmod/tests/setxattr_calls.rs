//! Promoted public controls; native evidence is not Linux qualification.
//! Four constructed SB cases are ordering diagnostics, not valid allocations.
use super::*;
use crate::capi::*;
use std::ffi::CString;

const NAME: &str = "user.audit";
const STAMP: i64 = 1_700_000_100;

fn specimen(flavor: features::FsFlavor, bs: u32) -> (Arc<Device>, Filesystem, u32, Arc<Clock>) {
    let size = if bs == 1024 {
        8 * 1024 * 1024
    } else {
        32 * 1024 * 1024
    };
    let dev = Arc::new(Device {
        bytes: Mutex::new(vec![0; size]),
        calls: std::array::from_fn(|_| AtomicUsize::new(0)),
        fail_write: AtomicUsize::new(0),
        fail_flush: AtomicUsize::new(0),
        panic_flush: AtomicUsize::new(0),
        writable: AtomicBool::new(true),
    });
    crate::mkfs::format_filesystem_with_flavor(
        dev.as_ref(),
        None,
        Some([31; 16]),
        size as u64,
        bs,
        flavor,
    )
    .unwrap();
    let clock = Arc::new(Clock {
        calls: AtomicUsize::new(0),
        now: std::sync::atomic::AtomicI64::new(1_700_000_000),
        hook: Mutex::new(None),
    });
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
    let ino = fs.apply_create_bytes(b"/f\xff", 0o640).unwrap();
    drop(fs);
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
    dev.reset();
    clock.calls.store(0, Ordering::SeqCst);
    assert_ne!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
    assert_eq!(compat(&dev) & features::Compat::EXT_ATTR.bits(), 0);
    (dev, fs, ino, clock)
}
fn state(dev: &Device) -> u16 {
    u16::from_le_bytes(dev.bytes()[1082..1084].try_into().unwrap())
}
fn compat(dev: &Device) -> u32 {
    u32::from_le_bytes(dev.bytes()[1116..1120].try_into().unwrap())
}
fn free(dev: &Device) -> u64 {
    let b = dev.bytes();
    u64::from(u32::from_le_bytes(b[1036..1040].try_into().unwrap()))
        | (u64::from(u32::from_le_bytes(b[1368..1372].try_into().unwrap())) << 32)
}
pub(super) fn c_set(handle: *mut fs_ext4_fs_t, path: &[u8], name: &str, value: &[u8]) -> i32 {
    let path = CString::new(path).unwrap();
    let name = CString::new(name).unwrap();
    unsafe {
        fs_ext4_setxattr(
            handle,
            path.as_ptr(),
            name.as_ptr(),
            value.as_ptr().cast(),
            value.len(),
        )
    }
}

#[test]
fn rust_string_set_must_refuse_held_context_before_callbacks() {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
    let _held = fs.chmod_operation().unwrap();
    let result = fs.apply_setxattr("/", NAME, b"x");
    eprintln!("string set result={result:?} callbacks={:?}", dev.counts());
    assert!(
        matches!(result, Err(Error::Unsupported(_))),
        "string set bypassed held context"
    );
    assert_eq!(dev.counts(), [0; 4]);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn rust_byte_set_must_refuse_held_context_before_callbacks() {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
    let _held = fs.chmod_operation().unwrap();
    let result = fs.apply_setxattr_bytes(b"/f\xff", NAME, b"x");
    eprintln!("byte set result={result:?} callbacks={:?}", dev.counts());
    assert!(
        matches!(result, Err(Error::Unsupported(_))),
        "byte set bypassed held context"
    );
    assert_eq!(dev.counts(), [0; 4]);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn capi_set_must_refuse_held_context_before_callbacks() {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
    let _fs = with_test_mount(fs, |handle, fs| {
        let _held = fs.chmod_operation().unwrap();
        let result = c_set(handle, b"/f\xff", NAME, b"x");
        eprintln!(
            "C set result={result} errno={} callbacks={:?}",
            fs_ext4_last_errno(),
            dev.counts()
        );
        assert_eq!(result, -1, "C set bypassed held context");
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::ENOTSUP);
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
    });
}

fn refusal(capi: bool, missing: bool) {
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
    let before = dev.bytes();
    let path: &[u8] = if missing { b"/missing" } else { b"/f\xff" };
    let name = if missing { NAME } else { "no.prefix" };
    let _fs = with_test_mount(fs, |handle, fs| {
        if capi {
            assert_eq!(c_set(handle, path, name, b"x"), -1);
            assert_eq!(
                fs_ext4_last_errno(),
                if missing {
                    crate::error::errno::ENOENT
                } else {
                    crate::error::errno::EINVAL
                }
            );
        } else {
            let result = fs.apply_setxattr_bytes(path, name, b"x");
            assert!(if missing {
                matches!(result, Err(Error::NotFound))
            } else {
                matches!(result, Err(Error::InvalidArgument(_)))
            });
        }
        eprintln!(
            "refused set: capi={capi} missing={missing} callbacks={:?}",
            dev.counts()
        );
        assert_eq!(
            &dev.counts()[1..3],
            &[0, 0],
            "planning refusal wrote/flushed marker"
        );
        assert!(dev.bytes() == before);
        assert!(!fs.marked_not_clean.load(Ordering::SeqCst));
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
    });
}
#[test]
fn rust_missing_path_must_leave_marker_and_bytes_untouched() {
    refusal(false, true);
}
#[test]
fn capi_missing_path_must_leave_marker_and_bytes_untouched() {
    refusal(true, true);
}
#[test]
fn rust_invalid_prefix_must_leave_marker_and_bytes_untouched() {
    refusal(false, false);
}
#[test]
fn capi_invalid_prefix_must_leave_marker_and_bytes_untouched() {
    refusal(true, false);
}

fn constructed(flavor: features::FsFlavor, bs: u32) {
    let (dev, fs, _, _) = specimen(flavor, bs);
    let mut context = fs.chmod_operation().unwrap();
    let before = free(&dev);
    let mut buf = BlockBuffer::new(bs);
    // Synthetic feature + counter edits before a marker. No bitmap/inode
    // allocation is represented; current public set marks BEFORE it plans.
    fs.buffer_patch_sb_compat(&mut buf, features::Compat::EXT_ATTR.bits())
        .unwrap();
    fs.buffer_patch_sb_counters(&mut buf, -1, 0).unwrap();
    fs.commit_setxattr_in(&mut context, buf).unwrap();
    assert_ne!(compat(&dev) & features::Compat::EXT_ATTR.bits(), 0);
    assert_eq!(free(&dev), before - 1);
    let bytes = dev.bytes();
    if fs.csum.enabled {
        assert_eq!(
            u32::from_le_bytes(bytes[2044..2048].try_into().unwrap()),
            crate::checksum::linux_crc32c(!0, &bytes[1024..2044])
        );
    }
    eprintln!(
        "constructed set buffer: bs={bs} journal={} feature/counter/checksum retained; state={}",
        fs.journal.is_some(),
        state(&dev)
    );
    assert_eq!(
        state(&dev) & crate::superblock::EXT4_VALID_FS,
        0,
        "constructed clean SB overwrote dirty marker"
    );
}
#[test]
fn constructed_direct_1k_feature_counter_buffer_must_preserve_dirty_marker() {
    constructed(features::FsFlavor::Ext4, 1024);
}
#[test]
fn constructed_direct_4k_feature_counter_buffer_must_preserve_dirty_marker() {
    constructed(features::FsFlavor::Ext4, 4096);
}
#[test]
fn constructed_journal_1k_feature_counter_buffer_must_preserve_dirty_marker() {
    constructed(features::FsFlavor::Ext3, 1024);
}
#[test]
fn constructed_journal_4k_feature_counter_buffer_must_preserve_dirty_marker() {
    constructed(features::FsFlavor::Ext3, 4096);
}

#[test]
fn existing_external_create_and_replace_retain_feature_counters_and_dirty_marker() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            for capi in [false, true] {
                let (dev, fs, ino, clock) = specimen(flavor, bs);
                let initial = free(&dev);
                let fs = with_test_mount(fs, |handle, fs| {
                    let mut acl = 0;
                    for (length, stamp) in [(600, STAMP), (700, STAMP + 100)] {
                        clock.now.store(stamp, Ordering::SeqCst);
                        let value = vec![b'v'; length];
                        if capi {
                            assert_eq!(c_set(handle, b"/f\xff", NAME, &value), 0);
                        } else {
                            fs.apply_setxattr_bytes(b"/f\xff", NAME, &value).unwrap();
                        }
                        let (inode, raw) = fs.read_inode_verified(ino).unwrap();
                        assert_ne!(inode.file_acl, 0);
                        if acl != 0 {
                            assert_eq!(inode.file_acl, acl);
                        }
                        acl = inode.file_acl;
                        assert_eq!(
                            crate::xattr::get_resolved(fs, &inode, &raw, NAME).unwrap(),
                            Some(value)
                        );
                        assert_eq!(inode.blocks, u64::from(bs) / 512);
                        assert_eq!((inode.ctime, inode.ctime_nsec), (stamp, 0));
                        assert_eq!(free(&dev), initial - 1);
                        assert_ne!(compat(&dev) & features::Compat::EXT_ATTR.bits(), 0);
                        assert_eq!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
                    }
                });
                drop(fs);
                assert_ne!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
                let fs = Filesystem::mount(dev).unwrap();
                let (inode, raw) = fs.read_inode_verified(ino).unwrap();
                assert_eq!(
                    crate::xattr::get_resolved(&fs, &inode, &raw, NAME).unwrap(),
                    Some(vec![b'v'; 700])
                );
            }
        }
    }
}

#[test]
fn inline_only_set_must_stamp_change_time() {
    let (_, fs, ino, clock) = specimen(features::FsFlavor::Ext4, 4096);
    clock.now.store(STAMP, Ordering::SeqCst);
    fs.apply_setxattr_bytes(b"/f\xff", NAME, b"x").unwrap();
    let inode = fs.read_inode_verified(ino).unwrap().0;
    assert_eq!(inode.file_acl, 0);
    assert_eq!(
        (inode.ctime, inode.ctime_nsec),
        (STAMP, 0),
        "inline-only set omitted change time"
    );
}
#[test]
fn first_inline_attribute_must_publish_ext_attr_feature() {
    let (dev, fs, ino, _) = specimen(features::FsFlavor::Ext4, 4096);
    fs.apply_setxattr_bytes(b"/f\xff", NAME, b"x").unwrap();
    let (inode, raw) = fs.read_inode_verified(ino).unwrap();
    assert_eq!(inode.file_acl, 0);
    assert_eq!(
        crate::xattr::get_resolved(&fs, &inode, &raw, NAME).unwrap(),
        Some(b"x".to_vec())
    );
    assert_ne!(
        compat(&dev) & features::Compat::EXT_ATTR.bits(),
        0,
        "inline-only set omitted feature publication"
    );
}
#[test]
fn capi_external_planning_panic_must_latch_mount_failure() {
    let (_, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
    let _fs = with_test_mount(fs, |handle, fs| {
        *clock.hook.lock().unwrap() = Some(Arc::new(|| panic!("set planning panic")));
        assert_eq!(c_set(handle, b"/f\xff", NAME, &[b'v'; 600]), -1);
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
        *clock.hook.lock().unwrap() = None;
        assert!(
            fs.operation_failed.load(Ordering::SeqCst),
            "planning unwind did not latch mount failure"
        );
    });
}

// Extend the retained five-family tests to all six families without removing
// any baseline test names. The shared fixtures below are cfg(test) only.
use super::removexattr_calls::{branch_specimen, retired_five};

fn prepared(
    branch: u8,
    flavor: features::FsFlavor,
    bs: u32,
) -> (Arc<Device>, Filesystem, u32, Arc<Clock>) {
    if branch < 6 {
        let (dev, fs, ino, clock, _) = branch_specimen(flavor, bs, branch);
        (dev, fs, ino, clock)
    } else {
        let (dev, fs, ino, clock) = specimen(flavor, bs);
        if branch == 8 {
            fs.apply_setxattr_bytes(b"/f\xff", NAME, b"x").unwrap();
            drop(fs);
            let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
            dev.reset();
            (dev, fs, ino, clock)
        } else {
            (dev, fs, ino, clock)
        }
    }
}
fn value(branch: u8) -> Vec<u8> {
    vec![b'n'; if branch == 6 || branch == 8 { 600 } else { 1 }]
}
fn branches(flavor: features::FsFlavor) -> Vec<u8> {
    // The legacy 128-byte profile has neither inline nor extra fields.
    if matches!(flavor, features::FsFlavor::Ext4) {
        (0..9).collect()
    } else {
        vec![0, 1, 2, 3, 6, 7, 8]
    }
}
#[test]
fn actual_setter_layouts_retain_metadata_counters_sharing_and_close() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            for branch in branches(flavor) {
                for capi in [false, true] {
                    let (dev, fs, ino, clock) = prepared(branch, flavor, bs);
                    let (before, _) = fs.read_inode_verified(ino).unwrap();
                    let free_before = free(&dev);
                    let peer = fs
                        .lookup_path_bytes(b"/peer")
                        .ok()
                        .map(|ino| (ino, fs.read_inode_verified(ino).unwrap().1));
                    let wanted = value(branch);
                    clock.now.store(STAMP, Ordering::SeqCst);
                    let fs = with_test_mount(fs, |handle, fs| {
                        if capi {
                            assert_eq!(c_set(handle, b"/f\xff", NAME, &wanted), 0);
                        } else {
                            fs.apply_setxattr_bytes(b"/f\xff", NAME, &wanted).unwrap();
                        }
                        let (after, raw) = fs.read_inode_verified(ino).unwrap();
                        assert_eq!(
                            crate::xattr::get_resolved(fs, &after, &raw, NAME).unwrap(),
                            Some(wanted.clone())
                        );
                        assert_eq!((after.ctime, after.ctime_nsec), (STAMP, 0));
                        assert_eq!(
                            (
                                after.mode,
                                after.uid,
                                after.gid,
                                after.size,
                                after.flags,
                                after.generation,
                                after.atime,
                                after.mtime,
                                after.crtime,
                                after.links_count
                            ),
                            (
                                before.mode,
                                before.uid,
                                before.gid,
                                before.size,
                                before.flags,
                                before.generation,
                                before.atime,
                                before.mtime,
                                before.crtime,
                                before.links_count
                            )
                        );
                        let legacy = matches!(flavor, features::FsFlavor::Ext3);
                        let (delta, sectors) = if legacy {
                            if branch == 1 || branch == 2 || (branch >= 6 && before.file_acl == 0) {
                                (
                                    -1,
                                    if before.file_acl == 0 {
                                        i64::from(bs / 512)
                                    } else {
                                        0
                                    },
                                )
                            } else {
                                (0, 0)
                            }
                        } else {
                            match branch {
                                0 => (0, 0),
                                1 => (-1, 0),
                                2 => (0, -i64::from(bs / 512)),
                                3..=5 => (1, -i64::from(bs / 512)),
                                6 | 8 => (-1, i64::from(bs / 512)),
                                _ => (0, 0),
                            }
                        };
                        assert_eq!(
                            free(&dev) as i64,
                            free_before as i64 + delta,
                            "branch {branch}"
                        );
                        assert_eq!(
                            after.blocks as i64,
                            before.blocks as i64 + sectors,
                            "branch {branch}"
                        );
                        if let Some((peer, raw)) = &peer {
                            assert_eq!(&fs.read_inode_verified(*peer).unwrap().1, raw);
                            let block = fs.read_block(before.file_acl).unwrap();
                            assert!(fs.csum.verify_xattr_block(before.file_acl, &block));
                            assert_eq!(u32::from_le_bytes(block[4..8].try_into().unwrap()), 1);
                        }
                        assert_ne!(compat(&dev) & features::Compat::EXT_ATTR.bits(), 0);
                        assert_eq!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
                    });
                    drop(fs);
                    assert_ne!(state(&dev) & crate::superblock::EXT4_VALID_FS, 0);
                    let fs = Filesystem::mount(dev).unwrap();
                    let (after, raw) = fs.read_inode_verified(ino).unwrap();
                    assert_eq!(
                        crate::xattr::get_resolved(&fs, &after, &raw, NAME).unwrap(),
                        Some(wanted)
                    );
                }
            }
        }
    }
}
fn every_fault(capi: bool, flush: bool) {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            for branch in branches(flavor) {
                let (dev, fs, _, _) = prepared(branch, flavor, bs);
                fs.apply_setxattr_bytes(b"/f\xff", NAME, &value(branch))
                    .unwrap();
                let index = if flush { 2 } else { 1 };
                let total = dev.counts()[index];
                assert!(total >= 2);
                drop(fs);
                for nth in 1..=total {
                    let (dev, fs, ino, _) = prepared(branch, flavor, bs);
                    (if flush {
                        &dev.fail_flush
                    } else {
                        &dev.fail_write
                    })
                    .store(nth, Ordering::SeqCst);
                    let fs = if capi {
                        with_test_mount(fs, |handle, _| {
                            assert_eq!(c_set(handle, b"/f\xff", NAME, &value(branch)), -1);
                            assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
                        })
                    } else {
                        assert!(fs
                            .apply_setxattr_bytes(b"/f\xff", NAME, &value(branch))
                            .is_err());
                        fs
                    };
                    assert!(dev.counts()[index] >= nth);
                    retired_five(&dev, fs, ino);
                }
            }
        }
    }
}
#[test]
fn rust_each_setter_write_fault_retires_all_six_families() {
    every_fault(false, false);
}
#[test]
fn rust_each_setter_flush_fault_retires_all_six_families() {
    every_fault(false, true);
}
#[test]
fn c_each_setter_write_fault_retires_all_six_families() {
    every_fault(true, false);
}
#[test]
fn c_each_setter_flush_fault_retires_all_six_families() {
    every_fault(true, true);
}

#[test]
fn setter_planning_and_effect_unwinds_retire_six_families_and_close() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for capi in [false, true] {
            for prior in [false, true] {
                let (dev, fs, _, _) = specimen(flavor, 4096);
                fs.apply_setxattr_bytes(b"/f\xff", NAME, &[b'x'; 600])
                    .unwrap();
                let total = dev.counts()[2];
                drop(fs);
                for phase in 0..=total {
                    let (dev, fs, ino, clock) = specimen(flavor, 4096);
                    if prior {
                        fs.apply_chown_ino(ino, 1, 2).unwrap();
                    }
                    dev.reset();
                    let before = dev.bytes();
                    if phase == 0 {
                        *clock.hook.lock().unwrap() =
                            Some(Arc::new(|| panic!("setter planning panic")));
                    } else {
                        dev.panic_flush.store(phase, Ordering::SeqCst);
                    }
                    // Prior-dirty calls have one fewer marker flush.
                    if prior && phase == total {
                        continue;
                    }
                    let fs = if capi {
                        with_test_mount(fs, |handle, _| {
                            assert_eq!(c_set(handle, b"/f\xff", NAME, &[b'x'; 600]), -1);
                            assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
                        })
                    } else {
                        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                            || fs.apply_setxattr_bytes(b"/f\xff", NAME, &[b'x'; 600])
                        ))
                        .is_err());
                        fs
                    };
                    if phase == 0 {
                        assert!(dev.bytes() == before);
                        assert_eq!(&dev.counts()[1..3], &[0, 0]);
                    } else {
                        assert!(dev.counts()[2] >= phase);
                    }
                    retired_five(&dev, fs, ino);
                }
            }
        }
    }
}
#[test]
fn setter_publication_unwind_latches_with_a_retained_context() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        let (dev, fs, ino, _) = specimen(flavor, 4096);
        let buf = fs
            .plan_setxattr(b"/f\xff", NAME, &[b'x'; 600])
            .unwrap()
            .unwrap();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _held = fs.uninit_cleared.lock().unwrap();
            panic!("setter publication poison");
        }))
        .is_err());
        let mut context = fs.chmod_operation().unwrap();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || fs.commit_setxattr_in(&mut context, buf)
        ))
        .is_err());
        assert_eq!(context.operation.status().health, Health::Uncertain);
        assert!(fs.operation_failed.load(Ordering::SeqCst));
        drop(context);
        retired_five(&dev, fs, ino);
    }
}
#[test]
fn setter_foreign_context_and_malformed_superblocks_refuse_before_effects() {
    let (_, first, _, _) = specimen(features::FsFlavor::Ext4, 4096);
    let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
    let mut foreign = first.chmod_operation().unwrap();
    assert!(matches!(
        fs.setxattr_bytes_in(&mut foreign, b"/f\xff", NAME, b"x"),
        Err(Error::Stale)
    ));
    assert!(matches!(
        fs.commit_setxattr_in(&mut foreign, BlockBuffer::new(4096)),
        Err(Error::Stale)
    ));
    let mut context = fs.chmod_operation().unwrap();
    let status = context.operation.status();
    for bytes in [vec![], vec![0; 1024], vec![0; 4096], vec![0; 4097]] {
        let mut buf = BlockBuffer::new(4096);
        buf.put(0, bytes);
        assert!(matches!(
            fs.commit_setxattr_in(&mut context, buf),
            Err(Error::Corrupt(_))
        ));
        assert_eq!(context.operation.status(), status);
    }
    assert_eq!(dev.counts(), [0; 4]);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn setter_refusals_keep_clean_or_dirty_state_and_priorities() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for prior in [false, true] {
            let (dev, fs, ino, clock) = specimen(flavor, 4096);
            if prior {
                fs.apply_chown_ino(ino, 1, 2).unwrap();
            }
            let status = fs.chmod_operation().unwrap().operation.status();
            let before = dev.bytes();
            dev.reset();
            clock.calls.store(0, Ordering::SeqCst);
            let fs = with_test_mount(fs, |handle, fs| {
                assert!(matches!(
                    fs.apply_setxattr_bytes(b"/missing", "no.prefix", b"x"),
                    Err(Error::NotFound)
                ));
                assert!(matches!(
                    fs.apply_setxattr_bytes(b"/f\xff", "no.prefix", b"x"),
                    Err(Error::InvalidArgument(_))
                ));
                assert!(matches!(
                    fs.apply_setxattr_bytes(b"/f\xff", NAME, &vec![0; 4096]),
                    Err(Error::NoSpaceLeftOnDevice)
                ));
                assert_eq!(c_set(handle, b"/missing", NAME, b"x"), -1);
                assert_eq!(fs_ext4_last_errno(), crate::error::errno::ENOENT);
                assert_eq!(
                    unsafe {
                        fs_ext4_setxattr(
                            handle,
                            std::ptr::null(),
                            std::ptr::null(),
                            std::ptr::null(),
                            0,
                        )
                    },
                    -1
                );
                assert_eq!(fs_ext4_last_errno(), crate::error::errno::EINVAL);
                let path = CString::new(b"/f\xff".as_slice()).unwrap();
                let name = CString::new(NAME).unwrap();
                assert_eq!(
                    unsafe {
                        fs_ext4_setxattr(
                            handle,
                            path.as_ptr(),
                            name.as_ptr(),
                            b"x".as_ptr().cast(),
                            65537,
                        )
                    },
                    -1
                );
                assert_eq!(fs_ext4_last_errno(), crate::error::errno::EINVAL);
                assert!(dev.bytes() == before);
                assert_eq!(&dev.counts()[1..3], &[0, 0]);
                assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
                assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
                assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), prior);
                assert_eq!(c_set(handle, b"/f\xff", NAME, b""), 0);
                assert_eq!(fs_ext4_last_errno(), 0);
                assert!(unsafe { std::ffi::CStr::from_ptr(fs_ext4_last_error()) }
                    .to_bytes()
                    .is_empty());
                assert_eq!(
                    fs.chmod_operation().unwrap().operation.status().epoch,
                    status.epoch + if prior { 1 } else { 2 }
                );
            });
            drop(fs);
        }
    }
}
#[test]
fn setter_volume_refusals_precede_lookup_and_clock() {
    for prior in [false, true] {
        for profile in 0..4 {
            let (dev, mut fs, ino, clock) = specimen(features::FsFlavor::Ext4, 4096);
            if prior {
                fs.apply_chmod_ino(ino, 0o600).unwrap();
            }
            match profile {
                0 => dev.writable.store(false, Ordering::SeqCst),
                1 => fs.sb.feature_incompat |= features::Incompat::CASEFOLD.bits(),
                2 => fs.sb.feature_ro_compat |= features::RoCompat::QUOTA.bits(),
                _ => fs.replay_pending.store(true, Ordering::SeqCst),
            }
            let before = dev.bytes();
            dev.reset();
            clock.calls.store(0, Ordering::SeqCst);
            let _fs = with_test_mount(fs, |handle, fs| {
                assert!(fs
                    .apply_setxattr_bytes(b"/missing", "no.prefix", b"x")
                    .is_err());
                assert_eq!(c_set(handle, b"/f\xff", NAME, b"x"), -1);
                assert_eq!(&dev.counts()[..3], &[0; 3]);
                assert!(dev.bytes() == before);
                assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
                assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), prior);
                assert!(!fs.operation_failed.load(Ordering::SeqCst));
            });
        }
    }
}
#[test]
fn setter_normalization_preserves_every_other_planned_byte() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            let (dev, fs, _, _) = specimen(flavor, bs);
            let mut buf = BlockBuffer::new(bs);
            fs.buffer_patch_sb_compat(&mut buf, features::Compat::EXT_ATTR.bits())
                .unwrap();
            fs.buffer_patch_sb_counters(&mut buf, -1, -1).unwrap();
            let nr = 1024 / u64::from(bs);
            let offset = (1024 % u64::from(bs)) as usize;
            let block = buf.dirty.get_mut(&nr).unwrap();
            block[offset + 58..offset + 60].copy_from_slice(&0xa5a7u16.to_le_bytes());
            if bs == 4096 {
                block[8] = 0x5a;
                block[3000] = 0x6b;
            }
            let mut wanted = block.clone();
            wanted[offset + 58..offset + 60].copy_from_slice(&0xa5a6u16.to_le_bytes());
            if fs.csum.enabled {
                let sum = crate::checksum::linux_crc32c(!0, &wanted[offset..offset + 1020]);
                wanted[offset + 1020..offset + 1024].copy_from_slice(&sum.to_le_bytes());
            }
            dev.reset();
            fs.prepare_setxattr_buffer(&mut buf).unwrap();
            assert_eq!(buf.dirty[&nr], wanted);
            assert_eq!(dev.counts(), [0; 4]);
            // A feature-present inline plan stays inode-only on a fresh mount.
            drop(fs);
            let fs = Filesystem::mount(dev).unwrap();
            fs.apply_setxattr_bytes(b"/f\xff", NAME, b"x").unwrap();
            drop(fs);
        }
    }
}
#[test]
fn setter_corrupt_and_ea_inode_blocks_discard_local_inline_changes() {
    for prior in [false, true] {
        for profile in 0..4 {
            let (dev, fs, ino, clock, _) = branch_specimen(features::FsFlavor::Ext4, 4096, 3);
            // Deliberately seed the same name inline while retaining its external
            // copy: a later external refusal must discard this local inline edit.
            let (inode, mut raw) = fs.read_inode_verified(ino).unwrap();
            let start = 128 + u16::from_le_bytes(raw[128..130].try_into().unwrap()) as usize;
            crate::xattr::plan_set_in_inode_region(&mut raw[start..], NAME, b"duplicate").unwrap();
            fs.finalize_inode_raw(ino, inode.generation, &mut raw)
                .unwrap();
            fs.write_inode_raw(ino, &raw).unwrap();
            let nr = inode.file_acl;
            let mut block = fs.read_block(nr).unwrap();
            match profile {
                0 => block[0] ^= 1,
                1 => block[8..12].copy_from_slice(&2u32.to_le_bytes()),
                2 => block[4095] ^= 1,
                _ => block[36..40].copy_from_slice(&ino.to_le_bytes()),
            }
            if profile != 2 {
                fs.csum.patch_xattr_block(nr, &mut block);
            }
            fs.dev.write_at(nr * 4096, &block).unwrap();
            drop(fs); // End fixture writes before testing a clean mount.
            let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
            if prior {
                fs.apply_chown_ino(ino, 1, 2).unwrap();
            }
            let before = dev.bytes();
            let status = fs.chmod_operation().unwrap().operation.status();
            let cleared = fs.uninit_cleared.lock().unwrap().cleared.clone();
            dev.reset();
            clock.calls.store(0, Ordering::SeqCst);
            let _fs = with_test_mount(fs, |handle, fs| {
                assert!(fs.apply_setxattr_bytes(b"/f\xff", NAME, b"x").is_err());
                assert_eq!(c_set(handle, b"/f\xff", NAME, b"x"), -1);
                assert!(dev.bytes() == before);
                assert_eq!(&dev.counts()[1..3], &[0, 0]);
                assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
                assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
                assert_eq!(fs.uninit_cleared.lock().unwrap().cleared, cleared);
                assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), prior);
            });
        }
    }
}

// Audit probe derived from all eight saved Linux same-value observations.
fn same_value_policy_probe(capi: bool) {
    let (_dev, fs, ino, clock) = specimen(features::FsFlavor::Ext4, 4096);
    let _fs = with_test_mount(fs, |handle, fs| {
        clock.now.store(STAMP, Ordering::SeqCst);
        if capi {
            assert_eq!(c_set(handle, b"/f\xff", NAME, b"n"), 0);
        } else {
            fs.apply_setxattr_bytes(b"/f\xff", NAME, b"n").unwrap();
        }
        let before = fs.read_inode_verified(ino).unwrap().0;
        assert_eq!((before.ctime, before.ctime_nsec), (STAMP, 0));
        clock.now.store(STAMP + 10, Ordering::SeqCst);
        if capi {
            assert_eq!(c_set(handle, b"/f\xff", NAME, b"n"), 0);
            assert_eq!(fs_ext4_last_errno(), 0);
        } else {
            fs.apply_setxattr_bytes(b"/f\xff", NAME, b"n").unwrap();
        }
        let (after, raw) = fs.read_inode_verified(ino).unwrap();
        assert_eq!(
            crate::xattr::get_resolved(fs, &after, &raw, NAME).unwrap(),
            Some(b"n".to_vec())
        );
        assert_eq!(
            (after.ctime, after.ctime_nsec),
            (before.ctime, before.ctime_nsec),
            "identical inline value must preserve ctime, as all eight Linux reference profiles do"
        );
    });
}
#[test]
fn rust_same_value_policy_probe() {
    same_value_policy_probe(false);
}
#[test]
fn capi_same_value_policy_probe() {
    same_value_policy_probe(true);
}

#[test]
fn same_value_policy_preserves_clean_dirty_bytes_clock_and_status() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            for prior in [false, true] {
                for dispatch in 0..3 {
                    for length in [0, 1, 600] {
                        let (dev, fs, _, clock) = specimen(flavor, bs);
                        let path: &[u8] = if dispatch == 0 { b"/" } else { b"/f\xff" };
                        let ino = fs.resolve(path).unwrap();
                        let value = vec![b'n'; length];
                        clock.now.store(STAMP, Ordering::SeqCst);
                        fs.apply_setxattr_bytes(path, NAME, &value).unwrap();
                        drop(fs);
                        let fs =
                            Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
                        if prior {
                            fs.apply_chmod_ino(ino, 0o700).unwrap();
                        }
                        let before = dev.bytes();
                        let status = fs.chmod_operation().unwrap().operation.status();
                        let cleared = fs.uninit_cleared.lock().unwrap().cleared.clone();
                        dev.reset();
                        clock.calls.store(0, Ordering::SeqCst);
                        clock.now.store(STAMP + 10, Ordering::SeqCst);
                        let _fs = with_test_mount(fs, |handle, fs| {
                            match dispatch {
                                0 => fs.apply_setxattr("/", NAME, &value).unwrap(),
                                1 => fs.apply_setxattr_bytes(path, NAME, &value).unwrap(),
                                _ => {
                                    assert_eq!(c_set(handle, path, NAME, &value), 0);
                                    assert_eq!(fs_ext4_last_errno(), 0);
                                }
                            }
                            assert!(
                                dev.bytes() == before,
                                "same-value call changed device bytes"
                            );
                            assert_eq!(&dev.counts()[1..3], &[0, 0]);
                            assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
                            assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
                            assert_eq!(fs.uninit_cleared.lock().unwrap().cleared, cleared);
                            assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), prior);
                            // A real change of the same length still stamps, writes and invalidates.
                            let changed = if length == 0 {
                                vec![b'x']
                            } else {
                                vec![b'x'; length]
                            };
                            fs.apply_setxattr_bytes(path, NAME, &changed).unwrap();
                            assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
                            assert!(dev.counts()[1] > 0 && dev.counts()[2] > 0);
                            assert_eq!(fs.read_inode_verified(ino).unwrap().0.ctime, STAMP + 10);
                            assert!(
                                fs.chmod_operation().unwrap().operation.status().epoch
                                    > status.epoch
                            );
                        });
                    }
                }
            }
        }
    }
}

#[test]
fn same_value_policy_preserves_shared_external_blocks_and_peers() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for bs in [1024, 4096] {
            let (dev, fs, ino, clock, peer) = branch_specimen(flavor, bs, 1);
            let inode = fs.read_inode_verified(ino).unwrap().0;
            let nr = inode.file_acl;
            let before = dev.bytes();
            let peer = peer.unwrap();
            let peer_raw = fs.read_inode_verified(peer).unwrap().1;
            clock.now.store(STAMP, Ordering::SeqCst);
            let _fs = with_test_mount(fs, |handle, fs| {
                fs.apply_setxattr_bytes(b"/f\xff", NAME, &[b'v'; 600])
                    .unwrap();
                assert_eq!(c_set(handle, b"/f\xff", NAME, &[b'v'; 600]), 0);
                assert!(dev.bytes() == before);
                assert_eq!(&dev.counts()[1..3], &[0, 0]);
                assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
                assert_eq!(fs.read_inode_verified(peer).unwrap().1, peer_raw);
                assert_eq!(
                    u32::from_le_bytes(fs.read_block(nr).unwrap()[4..8].try_into().unwrap()),
                    2
                );
            });
        }
    }
}

#[test]
fn same_value_policy_keeps_external_validation_and_duplicate_cleanup() {
    for profile in 0..5 {
        let (dev, fs, ino, clock, _) = branch_specimen(features::FsFlavor::Ext4, 4096, 3);
        let (inode, mut raw) = fs.read_inode_verified(ino).unwrap();
        let start = 128 + u16::from_le_bytes(raw[128..130].try_into().unwrap()) as usize;
        crate::xattr::plan_set_in_inode_region(&mut raw[start..], NAME, b"n").unwrap();
        fs.finalize_inode_raw(ino, inode.generation, &mut raw)
            .unwrap();
        fs.write_inode_raw(ino, &raw).unwrap();
        if profile < 4 {
            let mut block = fs.read_block(inode.file_acl).unwrap();
            match profile {
                0 => block[0] ^= 1,
                1 => block[8..12].copy_from_slice(&2u32.to_le_bytes()),
                2 => block[4095] ^= 1,
                _ => block[36..40].copy_from_slice(&ino.to_le_bytes()),
            }
            if profile != 2 {
                fs.csum.patch_xattr_block(inode.file_acl, &mut block);
            }
            fs.dev.write_at(inode.file_acl * 4096, &block).unwrap();
        }
        drop(fs);
        let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
        let before = dev.bytes();
        dev.reset();
        clock.calls.store(0, Ordering::SeqCst);
        clock.now.store(STAMP, Ordering::SeqCst);
        let _fs = with_test_mount(fs, |handle, fs| {
            if profile < 4 {
                assert!(fs.apply_setxattr_bytes(b"/f\xff", NAME, b"n").is_err());
                assert_eq!(c_set(handle, b"/f\xff", NAME, b"n"), -1);
                assert!(dev.bytes() == before);
                assert_eq!(&dev.counts()[1..3], &[0, 0]);
                assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
            } else {
                assert_eq!(c_set(handle, b"/f\xff", NAME, b"n"), 0);
                let (after, raw) = fs.read_inode_verified(ino).unwrap();
                let attrs = crate::xattr::read_all_resolved(fs, &after, &raw).unwrap();
                assert_eq!(attrs.iter().filter(|a| a.name == NAME).count(), 1);
                assert_eq!(after.file_acl, 0);
                assert_eq!(after.ctime, STAMP);
                assert!(dev.bytes() != before);
                assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
            }
        });
    }
}

#[test]
fn same_value_policy_cannot_bypass_held_failed_or_volume_gates() {
    for profile in 0..5 {
        let (dev, fs, _, clock) = specimen(features::FsFlavor::Ext4, 4096);
        fs.apply_setxattr_bytes(b"/f\xff", NAME, b"n").unwrap();
        drop(fs);
        let mut fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
        let before = dev.bytes();
        dev.reset();
        clock.calls.store(0, Ordering::SeqCst);
        match profile {
            1 => fs.operation_failed.store(true, Ordering::SeqCst),
            2 => dev.writable.store(false, Ordering::SeqCst),
            3 => fs.sb.feature_incompat |= features::Incompat::CASEFOLD.bits(),
            4 => fs.replay_pending.store(true, Ordering::SeqCst),
            _ => {}
        }
        let _fs = with_test_mount(fs, |handle, fs| {
            let held = if profile == 0 {
                Some(fs.chmod_operation().unwrap())
            } else {
                None
            };
            assert!(fs.apply_setxattr_bytes(b"/f\xff", NAME, b"n").is_err());
            assert_eq!(c_set(handle, b"/f\xff", NAME, b"n"), -1);
            assert_eq!(&dev.counts()[..3], &[0; 3]);
            assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
            assert!(dev.bytes() == before);
            drop(held);
        });
    }
}

#[test]
fn same_value_policy_keeps_feature_publication_and_zero_extra_initialization() {
    for profile in 0..2 {
        let (dev, fs, ino, clock) = specimen(features::FsFlavor::Ext4, 4096);
        let value = if profile == 0 {
            vec![b'n']
        } else {
            vec![b'n'; 600]
        };
        fs.apply_setxattr_bytes(b"/f\xff", NAME, &value).unwrap();
        if profile == 1 {
            let (inode, mut raw) = fs.read_inode_verified(ino).unwrap();
            super::super::zero_extra_fixture::prepare(&mut raw).unwrap();
            fs.finalize_inode_raw(ino, inode.generation, &mut raw)
                .unwrap();
            fs.write_inode_raw(ino, &raw).unwrap();
        }
        drop(fs);
        if profile == 0 {
            let mut bytes = dev.bytes.lock().unwrap();
            let sb = &mut bytes[1024..2048];
            let compat = u32::from_le_bytes(sb[92..96].try_into().unwrap());
            sb[92..96]
                .copy_from_slice(&(compat & !features::Compat::EXT_ATTR.bits()).to_le_bytes());
            let sum = crate::checksum::linux_crc32c(!0, &sb[..1020]);
            sb[1020..].copy_from_slice(&sum.to_le_bytes());
        }
        let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
        let before = dev.bytes();
        dev.reset();
        clock.calls.store(0, Ordering::SeqCst);
        clock.now.store(STAMP, Ordering::SeqCst);
        fs.apply_setxattr_bytes(b"/f\xff", NAME, &value).unwrap();
        assert!(dev.bytes() != before);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
        assert_ne!(compat(&dev) & features::Compat::EXT_ATTR.bits(), 0);
        if profile == 1 {
            assert_eq!(
                u16::from_le_bytes(
                    fs.read_inode_verified(ino).unwrap().1[128..130]
                        .try_into()
                        .unwrap()
                ),
                crate::inode::EXTRA_ISIZE_DEFAULT
            );
        }
    }
}

#[test]
fn same_value_policy_preserves_valid_noncanonical_inline_storage() {
    let (dev, fs, ino, clock) = specimen(features::FsFlavor::Ext4, 4096);
    fs.apply_setxattr_bytes(b"/f\xff", NAME, b"n").unwrap();
    let (inode, mut raw) = fs.read_inode_verified(ino).unwrap();
    let start = 128 + u16::from_le_bytes(raw[128..130].try_into().unwrap()) as usize;
    // In-inode entry hashes and unused packing bytes do not change its value.
    raw[start + 16..start + 20].copy_from_slice(&0x12345678u32.to_le_bytes());
    raw[start + 64] = 0x5a;
    fs.finalize_inode_raw(ino, inode.generation, &mut raw)
        .unwrap();
    fs.write_inode_raw(ino, &raw).unwrap();
    drop(fs);
    let fs = Filesystem::mount_with_runtime(dev.clone(), clock.clone()).unwrap();
    let before = dev.bytes();
    dev.reset();
    clock.calls.store(0, Ordering::SeqCst);
    clock.now.store(STAMP, Ordering::SeqCst);
    fs.apply_setxattr_bytes(b"/f\xff", NAME, b"n").unwrap();
    assert!(dev.bytes() == before);
    assert_eq!(&dev.counts()[1..3], &[0, 0]);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
}
