//! Owner/group dispatch must share the chmod mount gate.
use super::*;
use crate::capi::*;
use std::ffi::{CStr, CString};

struct Convert<'a> {
    calls: &'a AtomicUsize,
    ino: u32,
}

impl From<Convert<'_>> for InodeRef {
    fn from(value: Convert<'_>) -> Self {
        value.calls.fetch_add(1, Ordering::SeqCst);
        value.ino.into()
    }
}

#[test]
fn overlap_precedes_chown_inode_conversion() {
    let (dev, fs, ino) = scratch(features::FsFlavor::Ext4);
    let held = fs.chmod_operation().unwrap();
    let calls = AtomicUsize::new(0);
    let result = fs.apply_chown_ino(Convert { calls: &calls, ino }, 123, 456);
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "conversion bypassed held chmod gate"
    );
    assert!(matches!(result, Err(Error::Unsupported(_))));
    assert_eq!(dev.counts(), [0; 4]);
    drop(held);
}

#[test]
fn rust_chown_entries_refuse_held_chmod_before_callbacks() {
    let (dev, fs, ino, clock) = with_clock();
    let held = fs.chmod_operation().unwrap();
    assert!(
        matches!(
            fs.apply_chown_bytes(b"/f\xff", 123, 456),
            Err(Error::Unsupported(_))
        ),
        "byte chown bypassed held chmod gate"
    );
    assert!(matches!(
        fs.apply_chown("/", 123, 456),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        fs.apply_chown_ino(ino, 123, 456),
        Err(Error::Unsupported(_))
    ));
    assert_eq!(dev.counts(), [0; 4]);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
    drop(held);
}

#[test]
fn capi_chown_entries_refuse_held_chmod_before_callbacks() {
    let (dev, fs, ino, clock) = with_clock();
    let fs = with_test_mount(fs, |handle, fs| {
        let held = fs.chmod_operation().unwrap();
        let path = CString::new(b"/f\xff".as_slice()).unwrap();
        assert_eq!(
            unsafe { fs_ext4_chown(handle, path.as_ptr(), 123, 456) },
            -1,
            "C chown bypassed held chmod gate"
        );
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::ENOTSUP);
        assert!(unsafe { CStr::from_ptr(fs_ext4_last_error()) }
            .to_bytes()
            .windows(24)
            .any(|part| part == b"operation already active"));
        assert_eq!(
            unsafe { fs_ext4_chown_ino(handle, ino, FS_EXT4_GEN_ANY, 123, 456) },
            -1
        );
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        drop(held);
    });
    assert!(!fs.operation_failed.load(Ordering::SeqCst));
}

const UID: u32 = 0xfedc_9876;
const GID: u32 = 0x89ab_cdef;

fn retired_both(dev: &Device, fs: Filesystem, ino: u32) {
    assert!(fs.operation_failed.load(Ordering::SeqCst));
    dev.reset();
    let before = dev.bytes();
    let fs = with_test_mount(fs, |handle, fs| {
        assert!(matches!(fs.apply_chown("/", 1, 2), Err(Error::ReadOnly)));
        assert!(matches!(
            fs.apply_chown_bytes(b"/f\xff", 1, 2),
            Err(Error::ReadOnly)
        ));
        assert!(matches!(
            fs.apply_chown_ino(ino, 1, 2),
            Err(Error::ReadOnly)
        ));
        let path = CString::new(b"/f\xff".as_slice()).unwrap();
        assert_eq!(unsafe { fs_ext4_chown(handle, path.as_ptr(), 1, 2) }, -1);
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EROFS);
        assert_eq!(
            unsafe { fs_ext4_chown_ino(handle, ino, FS_EXT4_GEN_ANY, 1, 2) },
            -1
        );
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EROFS);
        assert_eq!(
            unsafe { fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o777) },
            -1
        );
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EROFS);
        assert_eq!(unsafe { fs_ext4_flush(handle) }, -1);
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
    });
    retired(dev, fs, ino);
    assert_eq!(dev.bytes(), before);
}

#[test]
fn full_width_ids_sentinels_and_metadata_survive_both_inode_layouts() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for capi in [false, true] {
            let clock = Arc::new(Clock {
                calls: AtomicUsize::new(0),
                now: std::sync::atomic::AtomicI64::new(1_700_000_000),
                hook: Mutex::new(None),
            });
            let (dev, fs, ino) = scratch_with_runtime(flavor, clock.clone());
            fs.apply_chmod_ino(ino, 0o6751).unwrap();
            let before = fs.read_inode_verified(ino).unwrap().0;
            let fs = with_test_mount(fs, |handle, fs| {
                for (step, (uid, gid, wanted_uid, wanted_gid)) in [
                    (UID, GID, UID, GID),
                    (u32::MAX, 17, UID, 17),
                    (42, u32::MAX, 42, 17),
                    (u32::MAX, u32::MAX, 42, 17),
                ]
                .into_iter()
                .enumerate()
                {
                    let now = (1i64 << 31) + 10 + step as i64;
                    clock.now.store(now, Ordering::SeqCst);
                    let r = InodeRef::new(ino, before.generation);
                    if capi {
                        let path = CString::new(b"/f\xff".as_slice()).unwrap();
                        let result = unsafe {
                            if step % 2 == 0 {
                                fs_ext4_chown(handle, path.as_ptr(), uid, gid)
                            } else {
                                fs_ext4_chown_ino(handle, ino, r.generation.unwrap(), uid, gid)
                            }
                        };
                        assert_eq!(result, 0);
                        assert_eq!(fs_ext4_last_errno(), 0);
                    } else if step % 2 == 0 {
                        fs.apply_chown_bytes(b"/f\xff", uid, gid).unwrap();
                    } else {
                        fs.apply_chown_ino(r, uid, gid).unwrap();
                    }
                    let (after, raw) = fs.read_inode_verified(ino).unwrap();
                    assert_eq!((after.uid, after.gid), (wanted_uid, wanted_gid));
                    assert_eq!(
                        u16::from_le_bytes(raw[2..4].try_into().unwrap()),
                        wanted_uid as u16
                    );
                    assert_eq!(
                        u16::from_le_bytes(raw[0x78..0x7a].try_into().unwrap()),
                        (wanted_uid >> 16) as u16
                    );
                    assert_eq!(
                        u16::from_le_bytes(raw[0x18..0x1a].try_into().unwrap()),
                        wanted_gid as u16
                    );
                    assert_eq!(
                        u16::from_le_bytes(raw[0x7a..0x7c].try_into().unwrap()),
                        (wanted_gid >> 16) as u16
                    );
                    assert_eq!(
                        (
                            after.mode,
                            after.atime,
                            after.mtime,
                            after.crtime,
                            after.generation,
                            after.flags,
                            after.size
                        ),
                        (
                            before.mode,
                            before.atime,
                            before.mtime,
                            before.crtime,
                            before.generation,
                            before.flags,
                            before.size
                        )
                    );
                    assert_eq!(
                        after.ctime,
                        if raw.len() == 128 {
                            i32::MAX as i64
                        } else {
                            now
                        }
                    );
                }
                fs.apply_chown("/", UID, GID).unwrap();
                assert_eq!(
                    fs.read_inode_verified(2).unwrap().0.mode & crate::inode::S_IFMT,
                    crate::inode::S_IFDIR
                );
            });
            drop(fs);
            let fs = Filesystem::mount(dev).unwrap();
            let after = fs.read_inode_verified(ino).unwrap().0;
            assert_eq!((after.uid, after.gid, after.mode), (42, 17, before.mode));
        }
    }
}

#[test]
fn volume_verdicts_precede_conversion_for_rust_and_c_entries() {
    for profile in 0..4 {
        let (dev, mut fs, ino, clock) = with_clock();
        match profile {
            0 => dev.writable.store(false, Ordering::SeqCst),
            1 => fs.sb.feature_incompat |= features::Incompat::CASEFOLD.bits(),
            2 => fs.sb.feature_ro_compat |= features::RoCompat::QUOTA.bits(),
            _ => fs.replay_pending.store(true, Ordering::SeqCst),
        }
        let before = dev.bytes();
        let calls = AtomicUsize::new(0);
        let fs = with_test_mount(fs, |handle, fs| {
            assert!(fs
                .apply_chown_ino(Convert { calls: &calls, ino }, UID, GID)
                .is_err());
            assert!(fs.apply_chown_bytes(b"/f\xff", UID, GID).is_err());
            assert!(fs.apply_chown("/", UID, GID).is_err());
            let path = CString::new(b"/f\xff".as_slice()).unwrap();
            assert_eq!(
                unsafe { fs_ext4_chown(handle, path.as_ptr(), UID, GID) },
                -1
            );
            assert_eq!(
                unsafe { fs_ext4_chown_ino(handle, ino, FS_EXT4_GEN_ANY, UID, GID) },
                -1
            );
        });
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert_eq!(&dev.counts()[..3], &[0; 3]);
        assert_eq!(dev.bytes(), before);
        assert!(!fs.marked_not_clean.load(Ordering::SeqCst));
        assert!(!fs.operation_failed.load(Ordering::SeqCst));
    }
}

#[test]
fn missing_stale_and_c_success_preserve_bytes_flags_and_errors() {
    let (dev, fs, ino, clock) = with_clock();
    let before = dev.bytes();
    let flags = fs.uninit_cleared.lock().unwrap().cleared.clone();
    let fs = with_test_mount(fs, |handle, fs| {
        assert!(matches!(
            fs.apply_chown("/missing", UID, GID),
            Err(Error::NotFound)
        ));
        assert!(matches!(
            fs.apply_chown_ino(InodeRef::new(ino, 0), UID, GID),
            Err(Error::Stale)
        ));
        let missing = CString::new("/missing").unwrap();
        assert_eq!(
            unsafe { fs_ext4_chown(handle, missing.as_ptr(), UID, GID) },
            -1
        );
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::ENOENT);
        assert_eq!(unsafe { fs_ext4_chown_ino(handle, ino, 0, UID, GID) }, -1);
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::ESTALE);
        assert_eq!(dev.bytes(), before);
        assert_eq!(fs.uninit_cleared.lock().unwrap().cleared, flags);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert_eq!(&dev.counts()[1..3], &[0; 2]);
        let context = fs.chmod_operation().unwrap();
        assert_eq!(context.operation.status().epoch, 0);
        drop(context);
        let path = CString::new(b"/f\xff".as_slice()).unwrap();
        assert_eq!(unsafe { fs_ext4_chown(handle, path.as_ptr(), UID, GID) }, 0);
        assert_eq!(fs_ext4_last_errno(), 0);
        assert!(unsafe { CStr::from_ptr(fs_ext4_last_error()) }
            .to_bytes()
            .is_empty());
        let generation = fs.read_inode_verified(ino).unwrap().0.generation;
        assert_eq!(
            unsafe { fs_ext4_chown_ino(handle, ino, generation, u32::MAX, 7) },
            0
        );
        assert_eq!(fs_ext4_last_errno(), 0);
    });
    assert!(!fs.operation_failed.load(Ordering::SeqCst));
}

struct HookConvert<'a> {
    ino: u32,
    hook: &'a dyn Fn(),
}
impl From<HookConvert<'_>> for InodeRef {
    fn from(value: HookConvert<'_>) -> Self {
        (value.hook)();
        value.ino.into()
    }
}

#[test]
fn conversion_holds_shared_gate_before_marker_and_unwind_retires() {
    for panic in [false, true] {
        let (dev, fs, ino) = scratch(features::FsFlavor::Ext4);
        let before = dev.bytes();
        let hook = || {
            assert!(!fs.marked_not_clean.load(Ordering::SeqCst));
            assert_eq!(&dev.counts()[1..3], &[0; 2]);
            let counts = dev.counts();
            assert!(matches!(
                fs.apply_chmod_ino(ino, 0o777),
                Err(Error::Unsupported(_))
            ));
            assert!(matches!(
                fs.apply_chown_ino(ino, 1, 2),
                Err(Error::Unsupported(_))
            ));
            assert_eq!(dev.counts(), counts);
            assert!(!panic, "injected chown conversion panic");
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            fs.apply_chown_ino(HookConvert { ino, hook: &hook }, UID, GID)
        }));
        if panic {
            assert!(result.is_err());
            assert_eq!(dev.bytes(), before);
            retired_both(&dev, fs, ino);
        } else {
            result.unwrap().unwrap();
            assert_eq!(fs.read_inode_verified(ino).unwrap().0.uid, UID);
        }
    }
}

// The owned handle must outlive every worker, including timeout and panic paths.
// A timeout is recorded inside the callback, then checked only after the outer
// operation has released its guard and the workers have joined.
#[derive(Default)]
struct Workers(Arc<Mutex<Vec<std::thread::JoinHandle<bool>>>>);
impl Workers {
    fn finish(&self) {
        let workers = std::mem::take(&mut *self.0.lock().unwrap());
        let results: Vec<_> = workers.into_iter().map(|worker| worker.join()).collect();
        assert!(results.into_iter().all(|result| matches!(result, Ok(true))));
    }
}
impl Drop for Workers {
    fn drop(&mut self) {
        for worker in self.0.lock().unwrap().drain(..) {
            let _ = worker.join();
        }
    }
}

#[test]
fn rust_callback_workers_refuse_both_families_in_both_directions() {
    for outer_chown in [false, true] {
        let (dev, fs, ino, clock) = with_clock();
        let fs = Arc::new(fs);
        let weak = Arc::downgrade(&fs);
        let workers = Workers::default();
        let handles = workers.0.clone();
        let responsive = Arc::new(AtomicBool::new(true));
        let observed_responsive = responsive.clone();
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
                    fs.apply_chown("/", UID, GID),
                    fs.apply_chown_bytes(b"/f\xff", UID, GID),
                    fs.apply_chown_ino(ino, UID, GID),
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
                observed_responsive.store(false, Ordering::SeqCst);
            }
        }));
        let result = if outer_chown {
            fs.apply_chown_ino(ino, UID, GID)
        } else {
            fs.apply_chmod_ino(ino, 0o600)
        };
        *clock.hook.lock().unwrap() = None;
        workers.finish();
        assert!(
            responsive.load(Ordering::SeqCst),
            "callback worker timed out"
        );
        result.unwrap();
        assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn capi_callback_workers_refuse_both_families_with_thread_local_errors() {
    for outer_chown in [false, true] {
        let (dev, fs, ino, clock) = with_clock();
        let fs = with_test_mount(fs, |handle, _| {
            let workers = Workers::default(); // Drops before the borrowed handle.
            let handles = workers.0.clone();
            let responsive = Arc::new(AtomicBool::new(true));
            let observed_responsive = responsive.clone();
            let address = handle as usize;
            let observed = dev.clone();
            *clock.hook.lock().unwrap() = Some(Arc::new(move || {
                let before = observed.counts();
                let observed = observed.clone();
                let (send, receive) = mpsc::channel();
                let worker = std::thread::spawn(move || {
                    let handle = address as *mut fs_ext4_fs_t;
                    let path = CString::new(b"/f\xff".as_slice()).unwrap();
                    let mut refused = true;
                    for entry in 0..4 {
                        let result = unsafe {
                            match entry {
                                0 => fs_ext4_chmod(handle, path.as_ptr(), 0o777),
                                1 => fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o777),
                                2 => fs_ext4_chown(handle, path.as_ptr(), UID, GID),
                                _ => fs_ext4_chown_ino(handle, ino, FS_EXT4_GEN_ANY, UID, GID),
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
                    observed_responsive.store(false, Ordering::SeqCst);
                }
            }));
            let result = unsafe {
                if outer_chown {
                    fs_ext4_chown_ino(handle, ino, FS_EXT4_GEN_ANY, UID, GID)
                } else {
                    fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o600)
                }
            };
            let error = fs_ext4_last_errno();
            *clock.hook.lock().unwrap() = None;
            workers.finish();
            assert!(
                responsive.load(Ordering::SeqCst),
                "C callback worker timed out"
            );
            assert_eq!((result, error), (0, 0));
        });
        assert!(!fs.operation_failed.load(Ordering::SeqCst));
        assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
    }
}

fn every_fault(capi: bool, flush: bool) {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        let (dev, fs, ino) = scratch(flavor);
        fs.apply_chown_ino(ino, UID, GID).unwrap();
        let total = dev.counts()[if flush { 2 } else { 1 }];
        assert!(total >= 2, "both marker and commit must execute");
        drop(fs);
        for by_path in [false, true] {
            for nth in 1..=total {
                let (dev, fs, ino) = scratch(flavor);
                if flush {
                    &dev.fail_flush
                } else {
                    &dev.fail_write
                }
                .store(nth, Ordering::SeqCst);
                let fs = if capi {
                    with_test_mount(fs, |handle, _| {
                        let path = CString::new(b"/f\xff".as_slice()).unwrap();
                        let result = unsafe {
                            if by_path {
                                fs_ext4_chown(handle, path.as_ptr(), UID, GID)
                            } else {
                                fs_ext4_chown_ino(handle, ino, FS_EXT4_GEN_ANY, UID, GID)
                            }
                        };
                        assert_eq!(result, -1, "C fault {nth}/{total} not reached");
                        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
                    })
                } else {
                    let result = if by_path {
                        fs.apply_chown_bytes(b"/f\xff", UID, GID)
                    } else {
                        fs.apply_chown_ino(ino, UID, GID)
                    };
                    assert!(result.is_err(), "Rust fault {nth}/{total} not reached");
                    fs
                };
                retired_both(&dev, fs, ino);
            }
        }
    }
}
#[test]
fn rust_every_marker_commit_write_failure_retires_both_families() {
    every_fault(false, false);
}
#[test]
fn rust_every_marker_commit_flush_failure_retires_both_families() {
    every_fault(false, true);
}
#[test]
fn capi_every_marker_commit_write_failure_retires_both_families() {
    every_fault(true, false);
}
#[test]
fn capi_every_marker_commit_flush_failure_retires_both_families() {
    every_fault(true, true);
}

#[test]
fn rust_and_c_marker_commit_panics_retire_both_families() {
    for capi in [false, true] {
        for by_path in [false, true] {
            for nth in [1, 2] {
                let (dev, fs, ino) = scratch(features::FsFlavor::Ext4);
                dev.panic_flush.store(nth, Ordering::SeqCst);
                let fs = if capi {
                    with_test_mount(fs, |handle, _| {
                        let path = CString::new(b"/f\xff".as_slice()).unwrap();
                        let result = unsafe {
                            if by_path {
                                fs_ext4_chown(handle, path.as_ptr(), UID, GID)
                            } else {
                                fs_ext4_chown_ino(handle, ino, FS_EXT4_GEN_ANY, UID, GID)
                            }
                        };
                        assert_eq!(result, -1);
                        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
                        assert!(unsafe { CStr::from_ptr(fs_ext4_last_error()) }
                            .to_string_lossy()
                            .contains("panic:"));
                    })
                } else {
                    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        if by_path {
                            fs.apply_chown_bytes(b"/f\xff", UID, GID)
                        } else {
                            fs.apply_chown_ino(ino, UID, GID)
                        }
                    }))
                    .is_err());
                    fs
                };
                retired_both(&dev, fs, ino);
            }
        }
    }
}

#[test]
fn capi_close_keeps_dirty_marker_after_chown_planning_panic() {
    for prior_write in [false, true] {
        let (dev, fs, ino, clock) = with_clock();
        let fs = with_test_mount(fs, |handle, fs| {
            if prior_write {
                assert_eq!(
                    unsafe { fs_ext4_chown_ino(handle, ino, FS_EXT4_GEN_ANY, UID, GID) },
                    0
                );
            }
            *clock.hook.lock().unwrap() = Some(Arc::new(|| panic!("chown clock panic")));
            let path = CString::new(b"/f\xff".as_slice()).unwrap();
            let before = dev.bytes();
            assert_eq!(unsafe { fs_ext4_chown(handle, path.as_ptr(), 1, 2) }, -1);
            assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
            assert_eq!(dev.bytes(), before);
            assert!(fs.operation_failed.load(Ordering::SeqCst));
            assert!(!fs.direct_commit_failed.load(Ordering::SeqCst));
            assert!(fs.journal.is_none());
            assert!(fs.apply_chmod_ino(ino, 0o777).is_err());
            assert!(fs.apply_chown_ino(ino, 1, 2).is_err());
        });
        dev.reset();
        let before = dev.bytes();
        assert_eq!(
            u16::from_le_bytes(before[1024 + 0x3a..1024 + 0x3c].try_into().unwrap())
                & crate::superblock::EXT4_VALID_FS
                != 0,
            !prior_write
        );
        unmount_test_mount(fs);
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(dev.bytes(), before);
    }
}

#[test]
fn chown_context_identity_and_later_refusal_preserve_prior_effect() {
    let (a, first, ino) = scratch(features::FsFlavor::Ext4);
    let (b, second, other) = scratch(features::FsFlavor::Ext4);
    let mut context = first.chmod_operation().unwrap();
    assert!(matches!(
        second.chown_ino_in(&mut context, other.into(), UID, GID),
        Err(Error::Stale)
    ));
    assert!(matches!(
        second.chown_bytes_in(&mut context, b"/f\xff", UID, GID),
        Err(Error::Stale)
    ));
    assert_eq!((a.counts(), b.counts()), ([0; 4], [0; 4]));
    first
        .chown_ino_in(&mut context, ino.into(), UID, GID)
        .unwrap();
    let before = a.bytes();
    let status = context.operation.status();
    assert!(matches!(
        first.chown_bytes_in(&mut context, b"/missing", 1, 2),
        Err(Error::NotFound)
    ));
    assert_eq!(a.bytes(), before);
    assert_eq!(context.operation.status(), status);
    drop(context);
    first.apply_chmod_ino(ino, 0o600).unwrap();
    // Failure must also bridge from the already-migrated chmod family.
    let (dev, fs, ino) = scratch(features::FsFlavor::Ext4);
    dev.fail_flush.store(1, Ordering::SeqCst);
    assert!(fs.apply_chmod_ino(ino, 0o600).is_err());
    let calls = AtomicUsize::new(0);
    assert!(matches!(
        fs.apply_chown_ino(Convert { calls: &calls, ino }, UID, GID),
        Err(Error::ReadOnly)
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    retired_both(&dev, fs, ino);
}
