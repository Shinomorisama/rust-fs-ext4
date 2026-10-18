//! Public inode flags share the metadata mount guard.
use super::*;
const USER_FLAGS: u32 =
    crate::inode::InodeFlags::NODUMP.bits() | crate::inode::InodeFlags::NOATIME.bits();
use crate::capi::*;
use std::ffi::{CStr, CString};

fn current(fs: &Filesystem, ino: u32) -> u32 {
    fs.read_inode_verified(ino).unwrap().0.flags
}
fn c_call(handle: *mut fs_ext4_fs_t, flags: u32) -> i32 {
    let path = CString::new(b"/f\xff".as_slice()).unwrap();
    unsafe { fs_ext4_set_flags(handle, path.as_ptr(), flags) }
}

#[test]
fn rust_string_flags_refuse_held_context_before_callbacks() {
    let (dev, fs, _, clock) = with_clock();
    let flags = current(&fs, 2) | 0x40;
    dev.reset();
    let held = fs.chmod_operation().unwrap();
    assert!(matches!(
        fs.apply_set_flags("/", flags),
        Err(Error::Unsupported(_))
    ));
    assert_eq!(dev.counts(), [0; 4]);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
    drop(held);
}
#[test]
fn rust_byte_flags_refuse_held_context_before_callbacks() {
    let (dev, fs, ino, clock) = with_clock();
    let flags = current(&fs, ino) | 0x40;
    dev.reset();
    let held = fs.chmod_operation().unwrap();
    assert!(matches!(
        fs.apply_set_flags_bytes(b"/f\xff", flags),
        Err(Error::Unsupported(_))
    ));
    assert_eq!(dev.counts(), [0; 4]);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
    drop(held);
}
#[test]
fn capi_flags_refuse_held_context_before_callbacks() {
    let (dev, fs, ino, clock) = with_clock();
    let flags = current(&fs, ino) | 0x40;
    dev.reset();
    let _fs = with_test_mount(fs, |handle, fs| {
        let held = fs.chmod_operation().unwrap();
        assert_eq!(c_call(handle, flags), -1);
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::ENOTSUP);
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        drop(held);
    });
}
#[test]
fn refused_flags_leave_clean_mount_bytes_and_marker_untouched() {
    for capi in [false, true] {
        let (dev, fs, ino, clock) = with_clock();
        let flags = current(&fs, ino) ^ crate::inode::InodeFlags::CASEFOLD.bits();
        dev.reset();
        let before = dev.bytes();
        let _fs = with_test_mount(fs, |handle, fs| {
            if capi {
                assert_eq!(c_call(handle, flags), -1);
                assert_eq!(fs_ext4_last_errno(), crate::error::errno::EINVAL);
            } else {
                assert!(matches!(
                    fs.apply_set_flags_bytes(b"/f\xff", flags),
                    Err(Error::InvalidArgument(_))
                ));
            }
            assert!(
                dev.bytes() == before,
                "refused flag plan changed clean mount"
            );
            assert_eq!(&dev.counts()[1..3], &[0, 0]);
            assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
            assert!(!fs.marked_not_clean.load(Ordering::SeqCst));
            assert!(!fs.operation_failed.load(Ordering::SeqCst));
        });
    }
}

#[test]
fn allowed_changes_and_same_flags_preserve_metadata_but_update_ctime() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for capi in [false, true] {
            let clock = Arc::new(Clock {
                calls: AtomicUsize::new(0),
                now: std::sync::atomic::AtomicI64::new(1_700_000_000),
                hook: Mutex::new(None),
            });
            let (_dev, fs, ino) = scratch_with_runtime(flavor, clock.clone());
            let before = fs.read_inode_verified(ino).unwrap().0;
            let start = if fs.sb.inode_size == 128 {
                1_700_000_100
            } else {
                (1i64 << 31) + 10
            };
            clock.calls.store(0, Ordering::SeqCst);
            let _fs = with_test_mount(fs, |handle, fs| {
                for (step, flags) in [
                    before.flags | USER_FLAGS,
                    before.flags | USER_FLAGS,
                    before.flags,
                ]
                .into_iter()
                .enumerate()
                {
                    clock.now.store(start + step as i64, Ordering::SeqCst);
                    if capi {
                        assert_eq!(c_call(handle, flags), 0);
                        assert_eq!(fs_ext4_last_errno(), 0);
                    } else {
                        fs.apply_set_flags_bytes(b"/f\xff", flags).unwrap();
                    }
                    let after = fs.read_inode_verified(ino).unwrap().0;
                    assert_eq!(after.flags, flags);
                    assert_eq!(after.ctime, start + step as i64);
                    assert_eq!(
                        (
                            after.mode,
                            after.uid,
                            after.gid,
                            after.generation,
                            after.size,
                            after.atime,
                            after.mtime,
                            after.crtime
                        ),
                        (
                            before.mode,
                            before.uid,
                            before.gid,
                            before.generation,
                            before.size,
                            before.atime,
                            before.mtime,
                            before.crtime
                        )
                    );
                    assert_eq!(clock.calls.load(Ordering::SeqCst), step + 1);
                    assert_eq!(
                        fs.chmod_operation().unwrap().operation.status().epoch,
                        step as u64 + 2
                    );
                }
                // Root string entry also preserves already-set managed bits.
                let root_flags = current(fs, 2);
                fs.apply_set_flags("/", root_flags | 0x40).unwrap();
                assert_eq!(current(fs, 2), root_flags | 0x40);
                if capi {
                    assert_eq!(
                        c_call(
                            handle,
                            before.flags ^ crate::inode::InodeFlags::EXTENTS.bits()
                        ),
                        -1
                    );
                    assert_eq!(fs_ext4_last_errno(), crate::error::errno::EINVAL);
                    assert_eq!(c_call(handle, before.flags), 0);
                    assert_eq!(fs_ext4_last_errno(), 0);
                    assert!(unsafe { CStr::from_ptr(fs_ext4_last_error()) }
                        .to_bytes()
                        .is_empty());
                }
            });
        }
    }
}

#[test]
fn every_managed_bit_change_is_refused_without_marker_or_clock() {
    let (dev, fs, ino, clock) = with_clock();
    let flags = current(&fs, ino);
    let before = dev.bytes();
    dev.reset();
    let _fs = with_test_mount(fs, |handle, fs| {
        for bit in 0..32 {
            let changed = 1u32 << bit;
            if changed & crate::inode::USER_MODIFIABLE_FLAGS != 0 {
                continue;
            }
            assert!(matches!(
                fs.apply_set_flags_bytes(b"/f\xff", flags ^ changed),
                Err(Error::InvalidArgument(_))
            ));
            assert_eq!(c_call(handle, flags ^ changed), -1);
            assert_eq!(fs_ext4_last_errno(), crate::error::errno::EINVAL);
        }
        assert!(dev.bytes() == before);
        assert_eq!(&dev.counts()[1..3], &[0, 0]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert_eq!(fs.chmod_operation().unwrap().operation.status().epoch, 0);
    });
}

#[test]
fn volume_and_missing_path_refusals_preserve_clean_or_prior_dirty_state() {
    for prior in [false, true] {
        for profile in 0..5 {
            let (dev, mut fs, ino, clock) = with_clock();
            let flags = current(&fs, ino);
            if prior {
                fs.apply_chown_ino(ino, 123, 456).unwrap();
            }
            let status = fs.chmod_operation().unwrap().operation.status();
            match profile {
                0 => dev.writable.store(false, Ordering::SeqCst),
                1 => fs.sb.feature_incompat |= features::Incompat::CASEFOLD.bits(),
                2 => fs.sb.feature_ro_compat |= features::RoCompat::QUOTA.bits(),
                3 => fs.replay_pending.store(true, Ordering::SeqCst),
                _ => (),
            }
            dev.reset();
            clock.calls.store(0, Ordering::SeqCst);
            let before = dev.bytes();
            let _fs = with_test_mount(fs, |handle, fs| {
                if profile == 4 {
                    // Missing path wins over a flag word that would change managed bits.
                    assert!(matches!(
                        fs.apply_set_flags("/missing", u32::MAX),
                        Err(Error::NotFound)
                    ));
                    let path = CString::new("/missing").unwrap();
                    assert_eq!(
                        unsafe { fs_ext4_set_flags(handle, path.as_ptr(), u32::MAX) },
                        -1
                    );
                    assert_eq!(fs_ext4_last_errno(), crate::error::errno::ENOENT);
                    assert!(matches!(
                        fs.apply_set_flags_bytes(
                            b"/f\xff",
                            flags ^ crate::inode::InodeFlags::CASEFOLD.bits()
                        ),
                        Err(Error::InvalidArgument(_))
                    ));
                    assert_eq!(
                        c_call(handle, flags ^ crate::inode::InodeFlags::CASEFOLD.bits()),
                        -1
                    );
                    assert_eq!(fs_ext4_last_errno(), crate::error::errno::EINVAL);
                } else {
                    assert!(fs.apply_set_flags_bytes(b"/f\xff", flags).is_err());
                    assert_eq!(c_call(handle, flags), -1);
                    assert_eq!(&dev.counts()[..3], &[0; 3]);
                }
                assert!(dev.bytes() == before);
                assert_eq!(&dev.counts()[1..3], &[0, 0]);
                assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
                assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
                assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), prior);
                assert!(!fs.operation_failed.load(Ordering::SeqCst));
            });
        }
    }
}

#[derive(Default)]
struct Workers(Arc<Mutex<Vec<std::thread::JoinHandle<bool>>>>);
impl Workers {
    fn finish(&self) {
        let workers = std::mem::take(&mut *self.0.lock().unwrap());
        let results: Vec<_> = workers.into_iter().map(|w| w.join()).collect();
        assert!(results.into_iter().all(|r| matches!(r, Ok(true))));
    }
}
impl Drop for Workers {
    fn drop(&mut self) {
        for w in self.0.lock().unwrap().drain(..) {
            let _ = w.join();
        }
    }
}

#[test]
fn rust_workers_refuse_all_four_families_in_both_directions() {
    for outer in 0..4 {
        let (dev, fs, ino, clock) = with_clock();
        let flags = current(&fs, ino);
        dev.reset();
        let fs = Arc::new(fs);
        let weak = Arc::downgrade(&fs);
        let workers = Workers::default();
        let handles = workers.0.clone();
        let responsive = Arc::new(AtomicBool::new(true));
        let seen_responsive = responsive.clone();
        let observed = dev.clone();
        *clock.hook.lock().unwrap() = Some(Arc::new(move || {
            let fs = weak.upgrade().unwrap();
            let before = observed.counts();
            let observed = observed.clone();
            let (send, receive) = mpsc::channel();
            let worker = std::thread::spawn(move || {
                let results = [
                    fs.apply_chmod("/", 0o777),
                    fs.apply_chmod_bytes(b"/f\xff", 0o777),
                    fs.apply_chmod_ino(ino, 0o777),
                    fs.apply_chown("/", 1, 2),
                    fs.apply_chown_bytes(b"/f\xff", 1, 2),
                    fs.apply_chown_ino(ino, 1, 2),
                    fs.apply_utimens("/", 5, 0, 6, 0),
                    fs.apply_utimens_bytes(b"/f\xff", 5, 0, 6, 0),
                    fs.apply_utimens_ino(ino, 5, 0, 6, 0),
                    fs.apply_set_flags("/", flags),
                    fs.apply_set_flags_bytes(b"/f\xff", flags),
                ];
                let result = results
                    .into_iter()
                    .all(|r| matches!(r, Err(Error::Unsupported(_))))
                    && observed.counts() == before;
                let _ = send.send(());
                result
            });
            handles.lock().unwrap().push(worker);
            if receive.recv_timeout(Duration::from_secs(5)).is_err() {
                seen_responsive.store(false, Ordering::SeqCst);
            }
        }));
        let result = match outer {
            0 => fs.apply_chmod_ino(ino, 0o600),
            1 => fs.apply_chown_ino(ino, 1, 2),
            2 => fs.apply_utimens_ino(ino, 5, 0, 6, 0),
            _ => fs.apply_set_flags_bytes(b"/f\xff", flags | 0x40),
        };
        *clock.hook.lock().unwrap() = None;
        workers.finish();
        assert!(responsive.load(Ordering::SeqCst));
        result.unwrap();
    }
}

#[test]
fn capi_workers_refuse_all_four_families_with_thread_local_errors() {
    for outer in 0..4 {
        let (dev, fs, ino, clock) = with_clock();
        let flags = current(&fs, ino);
        dev.reset();
        let _fs = with_test_mount(fs, |handle, _| {
            let workers = Workers::default();
            let handles = workers.0.clone();
            let responsive = Arc::new(AtomicBool::new(true));
            let seen_responsive = responsive.clone();
            let observed = dev.clone();
            let address = handle as usize;
            *clock.hook.lock().unwrap() = Some(Arc::new(move || {
                let before = observed.counts();
                let observed = observed.clone();
                let (send, receive) = mpsc::channel();
                let worker = std::thread::spawn(move || {
                    let handle = address as *mut fs_ext4_fs_t;
                    let path = CString::new(b"/f\xff".as_slice()).unwrap();
                    let mut refused = true;
                    for entry in 0..7 {
                        let result = unsafe {
                            match entry {
                                0 => fs_ext4_chmod(handle, path.as_ptr(), 0o777),
                                1 => fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o777),
                                2 => fs_ext4_chown(handle, path.as_ptr(), 1, 2),
                                3 => fs_ext4_chown_ino(handle, ino, FS_EXT4_GEN_ANY, 1, 2),
                                4 => fs_ext4_utimens(handle, path.as_ptr(), 5, 0, 6, 0),
                                5 => fs_ext4_utimens_ino(handle, ino, FS_EXT4_GEN_ANY, 5, 0, 6, 0),
                                _ => fs_ext4_set_flags(handle, path.as_ptr(), flags),
                            }
                        };
                        refused &=
                            result == -1 && fs_ext4_last_errno() == crate::error::errno::ENOTSUP;
                    }
                    let result = refused && observed.counts() == before;
                    let _ = send.send(());
                    result
                });
                handles.lock().unwrap().push(worker);
                if receive.recv_timeout(Duration::from_secs(5)).is_err() {
                    seen_responsive.store(false, Ordering::SeqCst);
                }
            }));
            let result = unsafe {
                match outer {
                    0 => fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o600),
                    1 => fs_ext4_chown_ino(handle, ino, FS_EXT4_GEN_ANY, 1, 2),
                    2 => fs_ext4_utimens_ino(handle, ino, FS_EXT4_GEN_ANY, 5, 0, 6, 0),
                    _ => fs_ext4_set_flags(
                        handle,
                        CString::new(b"/f\xff".as_slice()).unwrap().as_ptr(),
                        flags | 0x40,
                    ),
                }
            };
            let error = fs_ext4_last_errno();
            *clock.hook.lock().unwrap() = None;
            workers.finish();
            assert!(responsive.load(Ordering::SeqCst));
            assert_eq!((result, error), (0, 0));
        });
    }
}

fn retired_four(dev: &Device, fs: Filesystem, ino: u32) {
    assert!(fs.operation_failed.load(Ordering::SeqCst));
    dev.reset();
    let before = dev.bytes();
    let fs = with_test_mount(fs, |handle, fs| {
        assert!(fs.apply_set_flags("/", 0).is_err());
        assert!(fs.apply_set_flags_bytes(b"/f\xff", 0).is_err());
        assert!(fs.apply_chown_ino(ino, 1, 2).is_err());
        assert!(fs
            .apply_utimens_ino(ino, TIME_OMIT, 0, TIME_OMIT, 0)
            .is_err());
        assert_eq!(c_call(handle, 0), -1);
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EROFS);
        assert_eq!(
            unsafe { fs_ext4_chown_ino(handle, ino, FS_EXT4_GEN_ANY, 1, 2) },
            -1
        );
        assert_eq!(
            unsafe {
                fs_ext4_utimens_ino(handle, ino, FS_EXT4_GEN_ANY, TIME_OMIT, 0, TIME_OMIT, 0)
            },
            -1
        );
        assert_eq!(unsafe { fs_ext4_flush(handle) }, -1);
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
    });
    retired(dev, fs, ino);
    assert!(dev.bytes() == before);
}
fn every_fault(capi: bool, flush: bool) {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        let (dev, fs, ino) = scratch(flavor);
        let flags = current(&fs, ino) | 0x40;
        dev.reset();
        fs.apply_set_flags_bytes(b"/f\xff", flags).unwrap();
        let index = if flush { 2 } else { 1 };
        let total = dev.counts()[index];
        assert!(total >= 2);
        drop(fs);
        for nth in 1..=total {
            let (dev, fs, ino) = scratch(flavor);
            let flags = current(&fs, ino) | 0x40;
            dev.reset();
            (if flush {
                &dev.fail_flush
            } else {
                &dev.fail_write
            })
            .store(nth, Ordering::SeqCst);
            let fs = if capi {
                with_test_mount(fs, |handle, _| {
                    assert_eq!(c_call(handle, flags), -1);
                    assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
                })
            } else {
                assert!(fs.apply_set_flags_bytes(b"/f\xff", flags).is_err());
                fs
            };
            assert!(
                dev.counts()[index] >= nth,
                "injected callback was not reached"
            );
            retired_four(&dev, fs, ino);
        }
    }
}
#[test]
fn rust_every_marker_commit_write_fault_retires_all_four_families() {
    every_fault(false, false);
}
#[test]
fn rust_every_marker_commit_flush_fault_retires_all_four_families() {
    every_fault(false, true);
}
#[test]
fn capi_every_marker_commit_write_fault_retires_all_four_families() {
    every_fault(true, false);
}
#[test]
fn capi_every_marker_commit_flush_fault_retires_all_four_families() {
    every_fault(true, true);
}

#[test]
fn planning_marker_commit_panics_retire_all_four_families() {
    for capi in [false, true] {
        for phase in 0..3 {
            let (dev, fs, ino, clock) = with_clock();
            let flags = current(&fs, ino) | 0x40;
            dev.reset();
            let before = dev.bytes();
            if phase == 0 {
                *clock.hook.lock().unwrap() = Some(Arc::new(|| panic!("flags planning panic")));
            } else {
                dev.panic_flush.store(phase, Ordering::SeqCst);
            }
            let fs = if capi {
                with_test_mount(fs, |handle, _| {
                    assert_eq!(c_call(handle, flags), -1);
                    assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
                    assert!(unsafe { CStr::from_ptr(fs_ext4_last_error()) }
                        .to_string_lossy()
                        .contains("panic:"));
                })
            } else {
                assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                    || fs.apply_set_flags_bytes(b"/f\xff", flags)
                ))
                .is_err());
                fs
            };
            if phase == 0 {
                assert!(dev.bytes() == before);
                assert_eq!(&dev.counts()[1..3], &[0, 0]);
            }
            retired_four(&dev, fs, ino);
        }
    }
}
#[test]
fn exported_c_close_preserves_clean_or_dirty_state_after_flags_planning_panic() {
    for prior in [false, true] {
        let (dev, fs, ino, clock) = with_clock();
        let flags = current(&fs, ino) | 0x40;
        let fs = with_test_mount(fs, |handle, fs| {
            if prior {
                assert_eq!(c_call(handle, flags), 0);
            }
            *clock.hook.lock().unwrap() = Some(Arc::new(|| panic!("flags close planning panic")));
            let before = dev.bytes();
            assert_eq!(c_call(handle, flags), -1);
            assert!(dev.bytes() == before);
            assert!(fs.operation_failed.load(Ordering::SeqCst));
            assert!(!fs.direct_commit_failed.load(Ordering::SeqCst));
            assert!(fs.journal.is_none());
        });
        dev.reset();
        let before = dev.bytes();
        assert_eq!(
            u16::from_le_bytes(before[1082..1084].try_into().unwrap())
                & crate::superblock::EXT4_VALID_FS
                != 0,
            !prior
        );
        unmount_test_mount(fs);
        assert_eq!(dev.counts(), [0; 4]);
        assert!(dev.bytes() == before);
    }
}
#[test]
fn foreign_flags_context_and_prior_family_failure_preserve_shared_state() {
    let (a, first, _) = scratch(features::FsFlavor::Ext4);
    let (b, second, other) = scratch(features::FsFlavor::Ext4);
    let flags = current(&second, other);
    b.reset();
    let mut context = first.chmod_operation().unwrap();
    assert!(matches!(
        second.flags_bytes_in(&mut context, b"/f\xff", flags),
        Err(Error::Stale)
    ));
    assert_eq!((a.counts(), b.counts()), ([0; 4], [0; 4]));
    drop(context);
    for family in 0..3 {
        let (dev, fs, ino) = scratch(features::FsFlavor::Ext4);
        dev.fail_flush.store(1, Ordering::SeqCst);
        assert!(match family {
            0 => fs.apply_chmod_ino(ino, 0o600),
            1 => fs.apply_chown_ino(ino, 1, 2),
            _ => fs.apply_utimens_ino(ino, 5, 0, 6, 0),
        }
        .is_err());
        retired_four(&dev, fs, ino);
    }
}
