//! Public dispatch controls, sharing the actual scratch-mount fault device.
use super::*;
use crate::capi::*;
use std::ffi::{CStr, CString};

struct Convert<'a> {
    calls: &'a AtomicUsize,
    ino: u32,
    hook: Option<&'a dyn Fn()>,
}

impl From<Convert<'_>> for InodeRef {
    fn from(value: Convert<'_>) -> Self {
        value.calls.fetch_add(1, Ordering::SeqCst);
        if let Some(hook) = value.hook {
            hook();
        }
        value.ino.into()
    }
}

#[test]
fn overlap_precedes_generic_inode_conversion() {
    let (dev, fs, ino) = scratch(features::FsFlavor::Ext4);
    let held = fs.chmod_operation().unwrap();
    let calls = AtomicUsize::new(0);
    let result = fs.apply_chmod_ino(
        Convert {
            calls: &calls,
            ino,
            hook: None,
        },
        0o600,
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "conversion ran while gate held"
    );
    assert!(matches!(result, Err(Error::Unsupported(_))));
    assert_eq!(dev.counts(), [0; 4]);
    drop(held);
}

#[test]
fn public_paths_refuse_overlap_before_callbacks() {
    let (dev, fs, ino, clock) = with_clock();
    let held = fs.chmod_operation().unwrap();
    assert!(
        matches!(
            fs.apply_chmod_bytes(b"/f\xff", 0o600),
            Err(Error::Unsupported(_))
        ),
        "public byte path bypassed active operation"
    );
    assert!(matches!(
        fs.apply_chmod("/", 0o700),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        fs.apply_chmod_ino(ino, 0o600),
        Err(Error::Unsupported(_))
    ));
    assert_eq!(dev.counts(), [0; 4]);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
    drop(held);
}

#[test]
fn capi_path_and_inode_refuse_overlap_before_callbacks() {
    let (dev, fs, ino, clock) = with_clock();
    let fs = with_test_mount(fs, |handle, fs| {
        let held = fs.chmod_operation().unwrap();
        let path = CString::new(b"/f\xff".as_slice()).unwrap();
        assert_eq!(
            unsafe { fs_ext4_chmod(handle, path.as_ptr(), 0o600) },
            -1,
            "C path bypassed active operation"
        );
        assert!(unsafe { CStr::from_ptr(fs_ext4_last_error()) }
            .to_bytes()
            .windows(24)
            .any(|part| part == b"operation already active"));
        assert_ne!(fs_ext4_last_errno(), 0);
        assert_eq!(
            unsafe { fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o600) },
            -1
        );
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        drop(held);
    });
    assert!(!fs.operation_failed.load(Ordering::SeqCst));
}

#[test]
fn volume_refusal_precedes_conversion() {
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
        assert!(fs
            .apply_chmod_ino(
                Convert {
                    calls: &calls,
                    ino,
                    hook: None
                },
                0o600
            )
            .is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert_eq!(&dev.counts()[..3], &[0; 3]);
        assert_eq!(dev.bytes(), before);
        assert!(!fs.operation_failed.load(Ordering::SeqCst));
    }
}

#[test]
fn conversion_runs_inside_context_before_marker() {
    let (dev, fs, ino) = scratch(features::FsFlavor::Ext4);
    let calls = AtomicUsize::new(0);
    let hook = || {
        assert!(!fs.marked_not_clean.load(Ordering::SeqCst));
        assert_eq!(&dev.counts()[1..3], &[0; 2]);
        assert!(matches!(
            fs.apply_chmod("/", 0o700),
            Err(Error::Unsupported(_))
        ));
    };
    fs.apply_chmod_ino(
        Convert {
            calls: &calls,
            ino,
            hook: Some(&hook),
        },
        0o600,
    )
    .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(fs.stat_ino(ino).unwrap().mode & 0xfff, 0o600);
    let context = fs.chmod_operation().unwrap();
    assert_eq!(context.operation.status().epoch, 2);
}

#[test]
fn conversion_unwind_retires_without_writing() {
    let (dev, fs, ino) = scratch(features::FsFlavor::Ext4);
    let calls = AtomicUsize::new(0);
    let before = dev.bytes();
    let hook = || panic!("injected conversion panic");
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        fs.apply_chmod_ino(
            Convert {
                calls: &calls,
                ino,
                hook: Some(&hook),
            },
            0o600,
        )
    }))
    .is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(&dev.counts()[1..3], &[0; 2]);
    assert_eq!(dev.bytes(), before);
    retired(&dev, fs, ino);
}

#[test]
fn planning_refusals_preserve_bytes_flags_and_health() {
    let (dev, fs, ino, clock) = with_clock();
    let before = dev.bytes();
    let flags = fs.uninit_cleared.lock().unwrap().cleared.clone();
    assert!(matches!(
        fs.apply_chmod("/missing", 0o600),
        Err(Error::NotFound)
    ));
    assert!(matches!(
        fs.apply_chmod_ino(InodeRef::new(ino, 1), 0o600),
        Err(Error::Stale)
    ));
    assert_eq!(dev.bytes(), before);
    assert_eq!(fs.uninit_cleared.lock().unwrap().cleared, flags);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
    assert_eq!(&dev.counts()[1..3], &[0; 2]);
    let context = fs.chmod_operation().unwrap();
    assert_eq!(context.operation.status().epoch, 0);
    drop(context);
    fs.apply_chmod_bytes(b"/f\xff", 0o600).unwrap();
}

#[test]
fn rust_callback_worker_refuses_all_public_entries() {
    let (dev, fs, ino, clock) = with_clock();
    let fs = Arc::new(fs);
    let weak = Arc::downgrade(&fs);
    let observed = dev.clone();
    *clock.hook.lock().unwrap() = Some(Arc::new(move || {
        let fs = weak.upgrade().unwrap();
        let before = observed.counts();
        let (send, receive) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            let results = [
                fs.apply_chmod("/", 0o700),
                fs.apply_chmod_bytes(b"/f\xff", 0o777),
                fs.apply_chmod_ino(ino, 0o777),
            ];
            send.send(
                results
                    .into_iter()
                    .all(|r| matches!(r, Err(Error::Unsupported(_)))),
            )
            .unwrap();
        });
        assert!(receive
            .recv_timeout(Duration::from_secs(5))
            .expect("public callback worker blocked"));
        worker.join().unwrap();
        assert_eq!(observed.counts(), before);
    }));
    fs.apply_chmod_bytes(b"/f\xff", 0o16751).unwrap();
    let inode = fs.stat_ino(ino).unwrap();
    assert_eq!(inode.mode, crate::inode::S_IFREG | 0o6751);
    assert_eq!(inode.ctime, (1i64 << 31) + 10);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
}

fn capi_refuses_failed_mount(handle: *mut fs_ext4_fs_t, dev: &Device, ino: u32) {
    dev.reset();
    let bytes = dev.bytes();
    let path = CString::new(b"/f\xff".as_slice()).unwrap();
    assert_eq!(unsafe { fs_ext4_chmod(handle, path.as_ptr(), 0o777) }, -1);
    assert_eq!(fs_ext4_last_errno(), crate::error::errno::EROFS);
    assert_eq!(
        unsafe { fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o777) },
        -1
    );
    assert_eq!(fs_ext4_last_errno(), crate::error::errno::EROFS);
    assert_eq!(unsafe { fs_ext4_flush(handle) }, -1);
    assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
    assert_eq!(dev.counts(), [0; 4]);
    assert_eq!(dev.bytes(), bytes);
}

fn every_public_fault(capi: bool, flush: bool) {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        let (dev, fs, ino) = scratch(flavor);
        fs.apply_chmod_ino(ino, 0o600).unwrap();
        let total = dev.counts()[if flush { 2 } else { 1 }];
        assert!(total >= 2);
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
                                fs_ext4_chmod(handle, path.as_ptr(), 0o600)
                            } else {
                                fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o600)
                            }
                        };
                        assert_eq!(result, -1, "public C fault {nth}/{total} was not reached");
                        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
                        capi_refuses_failed_mount(handle, &dev, ino);
                    })
                } else {
                    let result = if by_path {
                        fs.apply_chmod_bytes(b"/f\xff", 0o600)
                    } else {
                        fs.apply_chmod_ino(ino, 0o600)
                    };
                    assert!(
                        result.is_err(),
                        "public Rust fault {nth}/{total} was not reached"
                    );
                    fs
                };
                retired(&dev, fs, ino);
            }
        }
    }
}

#[test]
fn rust_every_marker_commit_write_failure_retires() {
    every_public_fault(false, false);
}
#[test]
fn rust_every_marker_commit_flush_failure_retires() {
    every_public_fault(false, true);
}
#[test]
fn capi_every_marker_commit_write_failure_retires() {
    every_public_fault(true, false);
}
#[test]
fn capi_every_marker_commit_flush_failure_retires() {
    every_public_fault(true, true);
}

#[test]
fn capi_worker_reentry_refuses_with_thread_local_errors() {
    let (dev, fs, ino, clock) = with_clock();
    let fs = with_test_mount(fs, |handle, _| {
        let address = handle as usize;
        let observed = dev.clone();
        *clock.hook.lock().unwrap() = Some(Arc::new(move || {
            let before = observed.counts();
            let (send, receive) = mpsc::channel();
            // This worker finishes before the borrowed test handle is released.
            let worker = std::thread::spawn(move || {
                let handle = address as *mut fs_ext4_fs_t;
                let path = CString::new(b"/f\xff".as_slice()).unwrap();
                let a = unsafe { fs_ext4_chmod(handle, path.as_ptr(), 0o777) };
                let first_errno = fs_ext4_last_errno();
                let b = unsafe { fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o777) };
                send.send((a, first_errno, b, fs_ext4_last_errno()))
                    .unwrap();
            });
            let (a, first, b, second) = receive
                .recv_timeout(Duration::from_secs(5))
                .expect("C callback worker blocked");
            worker.join().unwrap();
            assert_eq!((a, b), (-1, -1));
            assert_ne!(first, 0);
            assert_ne!(second, 0);
            assert_eq!(observed.counts(), before);
        }));
        assert_eq!(
            unsafe { fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o600) },
            0
        );
        assert_eq!(fs_ext4_last_errno(), 0);
        *clock.hook.lock().unwrap() = None;
    });
    assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
    assert_eq!(fs.stat_ino(ino).unwrap().mode & 0xfff, 0o600);
}

#[test]
fn capi_catches_marker_commit_rust_panics_and_retires() {
    // Panic is in the Rust BlockDevice implementation, inside ffi_guard;
    // it never unwinds through a non-unwinding extern C callback.
    for by_path in [false, true] {
        for nth in [1, 2] {
            let (dev, fs, ino) = scratch(features::FsFlavor::Ext4);
            dev.panic_flush.store(nth, Ordering::SeqCst);
            let fs = with_test_mount(fs, |handle, _| {
                let path = CString::new(b"/f\xff".as_slice()).unwrap();
                let result = unsafe {
                    if by_path {
                        fs_ext4_chmod(handle, path.as_ptr(), 0o600)
                    } else {
                        fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o600)
                    }
                };
                assert_eq!(result, -1);
                assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
                assert!(unsafe { CStr::from_ptr(fs_ext4_last_error()) }
                    .to_string_lossy()
                    .contains("panic:"));
                capi_refuses_failed_mount(handle, &dev, ino);
            });
            retired(&dev, fs, ino);
        }
    }
}

#[test]
fn capi_catches_planning_panic_before_any_write() {
    let (dev, fs, ino, clock) = with_clock();
    let bytes = dev.bytes();
    *clock.hook.lock().unwrap() = Some(Arc::new(|| panic!("public clock panic")));
    let fs = with_test_mount(fs, |handle, _| {
        let path = CString::new(b"/f\xff".as_slice()).unwrap();
        assert_eq!(unsafe { fs_ext4_chmod(handle, path.as_ptr(), 0o600) }, -1);
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
        assert!(unsafe { CStr::from_ptr(fs_ext4_last_error()) }
            .to_string_lossy()
            .contains("public clock panic"));
        assert_eq!(dev.bytes(), bytes);
        capi_refuses_failed_mount(handle, &dev, ino);
    });
    retired(&dev, fs, ino);
}

#[test]
fn capi_stale_missing_and_success_preserve_error_semantics() {
    let (dev, fs, ino, clock) = with_clock();
    let fs = with_test_mount(fs, |handle, fs| {
        let before = dev.bytes();
        let missing = CString::new("/missing").unwrap();
        assert_eq!(
            unsafe { fs_ext4_chmod(handle, missing.as_ptr(), 0o600) },
            -1
        );
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::ENOENT);
        assert_eq!(unsafe { fs_ext4_chmod_ino(handle, ino, 1, 0o600) }, -1);
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::ESTALE);
        assert_eq!(dev.bytes(), before);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        let path = CString::new(b"/f\xff".as_slice()).unwrap();
        assert_eq!(unsafe { fs_ext4_chmod(handle, path.as_ptr(), 0o16751) }, 0);
        assert_eq!(fs_ext4_last_errno(), 0);
        assert!(unsafe { CStr::from_ptr(fs_ext4_last_error()) }
            .to_bytes()
            .is_empty());
        let inode = fs.stat_ino(ino).unwrap();
        assert_eq!(inode.mode, crate::inode::S_IFREG | 0o6751);
        assert_eq!(inode.ctime, (1i64 << 31) + 10);
        assert_eq!(
            unsafe { fs_ext4_chmod_ino(handle, ino, inode.generation, 0o600) },
            0
        );
        assert_eq!(fs_ext4_last_errno(), 0);
    });
    assert!(!fs.operation_failed.load(Ordering::SeqCst));
}

#[test]
fn capi_volume_refusals_preserve_bytes_and_health() {
    for profile in 0..4 {
        let (dev, mut fs, ino, clock) = with_clock();
        match profile {
            0 => dev.writable.store(false, Ordering::SeqCst),
            1 => fs.sb.feature_incompat |= features::Incompat::CASEFOLD.bits(),
            2 => fs.sb.feature_ro_compat |= features::RoCompat::QUOTA.bits(),
            _ => fs.replay_pending.store(true, Ordering::SeqCst),
        }
        let before = dev.bytes();
        let fs = with_test_mount(fs, |handle, _| {
            let path = CString::new(b"/f\xff".as_slice()).unwrap();
            assert_eq!(unsafe { fs_ext4_chmod(handle, path.as_ptr(), 0o600) }, -1);
            assert_ne!(fs_ext4_last_errno(), 0);
            assert_eq!(
                unsafe { fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o600) },
                -1
            );
            assert_ne!(fs_ext4_last_errno(), 0);
        });
        assert_eq!(dev.bytes(), before);
        assert_eq!(&dev.counts()[..3], &[0; 3]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert!(!fs.operation_failed.load(Ordering::SeqCst));
    }
}

#[test]
fn capi_unmount_keeps_dirty_marker_after_planning_panic() {
    let (dev, fs, ino, clock) = with_clock();
    let fs = with_test_mount(fs, |handle, fs| {
        assert_eq!(
            unsafe { fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o600) },
            0
        );
        assert!(fs.marked_not_clean.load(Ordering::SeqCst));
        *clock.hook.lock().unwrap() = Some(Arc::new(|| panic!("close after planning panic")));
        assert_eq!(
            unsafe { fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o700) },
            -1
        );
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
        assert!(fs.operation_failed.load(Ordering::SeqCst));
        // The old direct/journal failure flags cannot explain close refusal.
        assert!(!fs.direct_commit_failed.load(Ordering::SeqCst));
        assert!(fs.journal.is_none());
    });
    dev.reset();
    let before = dev.bytes();
    assert_eq!(
        u16::from_le_bytes(before[1024 + 0x3a..1024 + 0x3c].try_into().unwrap())
            & crate::superblock::EXT4_VALID_FS,
        0
    );
    unmount_test_mount(fs);
    assert_eq!(dev.counts(), [0; 4]);
    assert_eq!(dev.bytes(), before);
}
