//! Public timestamp dispatch shares the permission/owner mount gate.
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
fn overlap_precedes_timestamp_inode_conversion() {
    let (dev, fs, ino) = scratch(features::FsFlavor::Ext4);
    let held = fs.chmod_operation().unwrap();
    let calls = AtomicUsize::new(0);
    let result = fs.apply_utimens_ino(Convert { calls: &calls, ino }, 5, 0, 6, 0);
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "conversion bypassed held gate"
    );
    assert!(matches!(result, Err(Error::Unsupported(_))));
    assert_eq!(dev.counts(), [0; 4]);
    drop(held);
}

#[test]
fn rust_timestamp_entries_refuse_held_context_before_callbacks() {
    let (dev, fs, ino, clock) = with_clock();
    let held = fs.chmod_operation().unwrap();
    assert!(
        matches!(
            fs.apply_utimens_bytes(b"/f\xff", 5, 0, 6, 0),
            Err(Error::Unsupported(_))
        ),
        "timestamp byte path bypassed held gate"
    );
    assert!(matches!(
        fs.apply_utimens("/", 5, 0, 6, 0),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        fs.apply_utimens_ino(ino, 5, 0, 6, 0),
        Err(Error::Unsupported(_))
    ));
    assert_eq!(dev.counts(), [0; 4]);
    assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
    drop(held);
}

#[test]
fn capi_timestamp_entries_refuse_held_context_before_callbacks() {
    let (dev, fs, ino, clock) = with_clock();
    let _fs = with_test_mount(fs, |handle, fs| {
        let held = fs.chmod_operation().unwrap();
        let path = CString::new(b"/f\xff".as_slice()).unwrap();
        assert_eq!(
            unsafe { fs_ext4_utimens(handle, path.as_ptr(), 5, 0, 6, 0) },
            -1,
            "C timestamp path bypassed held gate"
        );
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::ENOTSUP);
        assert_eq!(
            unsafe { fs_ext4_utimens_ino(handle, ino, FS_EXT4_GEN_ANY, 5, 0, 6, 0) },
            -1
        );
        assert_eq!(dev.counts(), [0; 4]);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        drop(held);
    });
}

#[test]
fn both_omitted_fields_do_not_write_a_clean_mount_marker() {
    let (dev, fs, ino, clock) = with_clock();
    let before = dev.bytes();
    fs.apply_utimens_ino(ino, TIME_OMIT, u32::MAX, i64::MAX, UTIME_OMIT)
        .unwrap();
    assert_eq!(
        &dev.counts()[1..3],
        &[0, 0],
        "no-op wrote or flushed the marker"
    );
    assert_eq!(dev.bytes(), before);
    assert!(!fs.marked_not_clean.load(Ordering::SeqCst));
    assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
    let context = fs.chmod_operation().unwrap();
    assert_eq!(context.operation.status().epoch, 0);
}

fn rust_call(fs: &Filesystem, r: InodeRef, by_path: bool, times: [(i64, u32); 2]) -> Result<()> {
    let [(a, an), (m, mn)] = times;
    if by_path {
        fs.apply_utimens_bytes(b"/f\xff", a, an, m, mn)
    } else {
        fs.apply_utimens_ino(r, a, an, m, mn)
    }
}
fn c_call(handle: *mut fs_ext4_fs_t, r: InodeRef, by_path: bool, times: [(i64, u32); 2]) -> i32 {
    let [(a, an), (m, mn)] = times;
    let path = CString::new(b"/f\xff".as_slice()).unwrap();
    unsafe {
        if by_path {
            fs_ext4_utimens(handle, path.as_ptr(), a, an, m, mn)
        } else {
            fs_ext4_utimens_ino(
                handle,
                r.ino,
                r.generation.unwrap_or(FS_EXT4_GEN_ANY),
                a,
                an,
                m,
                mn,
            )
        }
    }
}
fn time(raw: &[u8], base: usize, extra: usize) -> (i64, u32) {
    let end = if raw.len() >= 130 {
        128 + u16::from_le_bytes(raw[128..130].try_into().unwrap()) as usize
    } else {
        128
    };
    let word = if extra + 4 <= raw.len().min(end) {
        u32::from_le_bytes(raw[extra..extra + 4].try_into().unwrap())
    } else {
        0
    };
    let base = i32::from_le_bytes(raw[base..base + 4].try_into().unwrap()) as i64;
    (base + (((word & 3) as i64) << 32), word >> 2)
}

#[test]
fn explicit_and_sentinel_updates_keep_times_and_other_metadata() {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        for capi in [false, true] {
            let clock = Arc::new(Clock {
                calls: AtomicUsize::new(0),
                now: std::sync::atomic::AtomicI64::new(1_700_000_000),
                hook: Mutex::new(None),
            });
            let (_dev, fs, ino) = scratch_with_runtime(flavor, clock.clone());
            let before = fs.read_inode_verified(ino).unwrap().0;
            clock.calls.store(0, Ordering::SeqCst);
            let wide = fs.sb.inode_size == 256;
            let start = if wide {
                (1i64 << 31) + 10
            } else {
                1_700_000_100
            };
            let precision = |n| if wide { n } else { 0 };
            let cases = [
                (
                    [(-123, 123_456_789), (-456, 987_654_321)],
                    [
                        (-123, precision(123_456_789)),
                        (-456, precision(987_654_321)),
                    ],
                ),
                (
                    [(TIME_OMIT, u32::MAX), (7, 8)],
                    [(-123, precision(123_456_789)), (7, precision(8))],
                ),
                (
                    [(i64::MAX, UTIME_NOW), (TIME_OMIT, UTIME_NOW)],
                    [(start + 2, 0), (7, precision(8))],
                ),
                (
                    [(i64::MAX, UTIME_OMIT), (i64::MAX, UTIME_NOW)],
                    [(start + 2, 0), (start + 3, 0)],
                ),
                (
                    [
                        (crate::inode::MIN_ENCODABLE_TIME, 999_999_999),
                        (
                            if wide {
                                crate::inode::MAX_ENCODABLE_TIME
                            } else {
                                i32::MAX as i64
                            },
                            42,
                        ),
                    ],
                    [
                        (crate::inode::MIN_ENCODABLE_TIME, precision(999_999_999)),
                        (
                            if wide {
                                crate::inode::MAX_ENCODABLE_TIME
                            } else {
                                i32::MAX as i64
                            },
                            precision(42),
                        ),
                    ],
                ),
            ];
            let _fs = with_test_mount(fs, |handle, fs| {
                for (step, (times, wanted)) in cases.into_iter().enumerate() {
                    clock.now.store(start + step as i64, Ordering::SeqCst);
                    let r = InodeRef::new(ino, before.generation);
                    if capi {
                        assert_eq!(c_call(handle, r, step % 2 == 0, times), 0);
                        assert_eq!(fs_ext4_last_errno(), 0);
                    } else {
                        rust_call(fs, r, step % 2 == 0, times).unwrap();
                    }
                    let (after, raw) = fs.read_inode_verified(ino).unwrap();
                    assert_eq!([time(&raw, 8, 0x8c), time(&raw, 0x10, 0x88)], wanted);
                    assert_eq!(time(&raw, 0xc, 0x84), (start + step as i64, 0));
                    assert_eq!(
                        (
                            after.mode,
                            after.uid,
                            after.gid,
                            after.generation,
                            after.flags,
                            after.size,
                            after.crtime
                        ),
                        (
                            before.mode,
                            before.uid,
                            before.gid,
                            before.generation,
                            before.flags,
                            before.size,
                            before.crtime
                        )
                    );
                    assert_eq!(clock.calls.load(Ordering::SeqCst), step + 1);
                }
                // A valid C call clears the preceding validation error.
                if capi {
                    assert_eq!(
                        c_call(handle, ino.into(), false, [(0, 1_000_000_000), (0, 0)]),
                        -1
                    );
                    assert_eq!(fs_ext4_last_errno(), crate::error::errno::EINVAL);
                    assert_eq!(
                        c_call(handle, ino.into(), false, [(TIME_OMIT, 0), (TIME_OMIT, 0)]),
                        0
                    );
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
fn rust_c_noops_validate_identity_and_keep_clean_or_prior_dirty_state() {
    for prior in [false, true] {
        for capi in [false, true] {
            for by_path in [false, true] {
                let (dev, fs, ino, clock) = with_clock();
                if prior {
                    fs.apply_chown_ino(ino, 123, 456).unwrap();
                }
                let before = dev.bytes();
                let held = fs.chmod_operation().unwrap();
                let status = held.operation.status();
                drop(held);
                dev.reset();
                clock.calls.store(0, Ordering::SeqCst);
                let _fs = with_test_mount(fs, |handle, fs| {
                    let times = [(TIME_OMIT, u32::MAX), (i64::MAX, UTIME_OMIT)];
                    if capi {
                        assert_eq!(c_call(handle, ino.into(), by_path, times), 0);
                    } else {
                        rust_call(fs, ino.into(), by_path, times).unwrap();
                    }
                    assert_eq!(dev.bytes(), before);
                    assert_eq!(&dev.counts()[1..3], &[0, 0]);
                    assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
                    let held = fs.chmod_operation().unwrap();
                    assert_eq!(held.operation.status(), status);
                    assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), prior);
                });
            }
        }
    }
}

#[test]
fn short_extra_fields_preserve_precision_and_epoch_refusals() {
    for extra in [0u16, 8, 12, 16] {
        for field in 0..2 {
            let (dev, fs, ino, clock) = with_clock();
            let (inode, mut raw) = fs.read_inode_verified(ino).unwrap();
            raw[128..130].copy_from_slice(&extra.to_le_bytes());
            fs.finalize_inode_raw(ino, inode.generation, &mut raw)
                .unwrap();
            fs.refuse_write().unwrap();
            fs.commit_inode_write(ino, &raw).unwrap();
            drop(fs);
            let fs = Filesystem::mount_with_runtime(dev.clone(), clock).unwrap();
            dev.reset();
            fs.apply_utimens_ino(ino, -123, 123_456_789, -456, 987_654_321)
                .unwrap();
            let raw = fs.read_inode_verified(ino).unwrap().1;
            assert_eq!(
                time(&raw, 8, 0x8c),
                (-123, if extra >= 16 { 123_456_789 } else { 0 })
            );
            assert_eq!(
                time(&raw, 0x10, 0x88),
                (-456, if extra >= 12 { 987_654_321 } else { 0 })
            );
            let before = dev.bytes();
            dev.reset();
            let mut times = [(TIME_OMIT, 0); 2];
            times[field] = ((1i64 << 31) + 10, 42);
            let result = rust_call(&fs, ino.into(), false, times);
            if extra >= (if field == 0 { 16 } else { 12 }) {
                result.unwrap();
                let raw = fs.read_inode_verified(ino).unwrap().1;
                assert_eq!(
                    if field == 0 {
                        time(&raw, 8, 0x8c)
                    } else {
                        time(&raw, 0x10, 0x88)
                    },
                    ((1i64 << 31) + 10, 42)
                );
            } else {
                assert!(matches!(result, Err(Error::InvalidArgument(_))));
                assert_eq!(dev.bytes(), before);
                assert_eq!(&dev.counts()[1..3], &[0, 0]);
                assert!(!fs.operation_failed.load(Ordering::SeqCst));
            }
        }
    }
}

#[test]
fn invalid_time_priority_conversion_and_stale_missing_refusals_are_untouched() {
    let (dev, fs, ino, clock) = with_clock();
    let before = dev.bytes();
    let flags = fs.uninit_cleared.lock().unwrap().cleared.clone();
    let calls = AtomicUsize::new(0);
    let _fs = with_test_mount(fs, |handle, fs| {
        let missing = CString::new("/missing").unwrap();
        for (sec, nsec) in [
            (0, 1_000_000_000),
            (0, u32::MAX),
            (crate::inode::MIN_ENCODABLE_TIME - 1, 0),
            (crate::inode::MAX_ENCODABLE_TIME + 1, 0),
        ] {
            assert!(matches!(
                fs.apply_utimens("/missing", sec, nsec, 0, 0),
                Err(Error::InvalidArgument(_))
            ));
            assert!(matches!(
                fs.apply_utimens_ino(Convert { calls: &calls, ino }, sec, nsec, 0, 0),
                Err(Error::InvalidArgument(_))
            ));
            assert_eq!(
                unsafe { fs_ext4_utimens(handle, missing.as_ptr(), sec, nsec, 0, 0) },
                -1
            );
            assert_eq!(fs_ext4_last_errno(), crate::error::errno::EINVAL);
        }
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert!(matches!(
            fs.apply_utimens("/missing", 0, 0, 0, 0),
            Err(Error::NotFound)
        ));
        assert!(matches!(
            fs.apply_utimens_ino(InodeRef::new(ino, 0), TIME_OMIT, 0, TIME_OMIT, 0),
            Err(Error::Stale)
        ));
        assert_eq!(
            c_call(handle, InodeRef::new(ino, 0), false, [(TIME_OMIT, 0); 2]),
            -1
        );
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::ESTALE);
        assert_eq!(dev.bytes(), before);
        assert_eq!(&dev.counts()[1..3], &[0, 0]);
        assert_eq!(fs.uninit_cleared.lock().unwrap().cleared, flags);
        assert_eq!(fs.chmod_operation().unwrap().operation.status().epoch, 0);
    });
}

#[test]
fn volume_refusals_precede_conversion_clock_and_even_noop_planning() {
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
        let _fs = with_test_mount(fs, |handle, fs| {
            assert!(fs
                .apply_utimens_ino(Convert { calls: &calls, ino }, TIME_OMIT, 0, TIME_OMIT, 0)
                .is_err());
            assert!(fs.apply_utimens_bytes(b"/f\xff", 5, 0, 6, 0).is_err());
            assert_eq!(c_call(handle, ino.into(), false, [(TIME_OMIT, 0); 2]), -1);
            assert_eq!(c_call(handle, ino.into(), true, [(5, 0), (6, 0)]), -1);
            assert!(!fs.marked_not_clean.load(Ordering::SeqCst));
            assert!(!fs.operation_failed.load(Ordering::SeqCst));
        });
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        assert_eq!(&dev.counts()[..3], &[0; 3]);
        assert_eq!(dev.bytes(), before);
    }
}

struct HookConvert<'a> {
    ino: u32,
    hook: &'a dyn Fn(),
}
impl From<HookConvert<'_>> for InodeRef {
    fn from(v: HookConvert<'_>) -> Self {
        (v.hook)();
        v.ino.into()
    }
}
#[test]
fn conversion_is_inside_shared_context_before_clock_marker_and_unwind() {
    for panic in [false, true] {
        let (dev, fs, ino, clock) = with_clock();
        let before = dev.bytes();
        let hook = || {
            assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
            assert_eq!(&dev.counts()[1..3], &[0, 0]);
            assert!(matches!(
                fs.apply_chmod_ino(ino, 0o777),
                Err(Error::Unsupported(_))
            ));
            assert!(matches!(
                fs.apply_chown_ino(ino, 1, 2),
                Err(Error::Unsupported(_))
            ));
            assert!(matches!(
                fs.apply_utimens_ino(ino, 5, 0, 6, 0),
                Err(Error::Unsupported(_))
            ));
            assert!(!panic, "timestamp conversion panic");
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            fs.apply_utimens_ino(HookConvert { ino, hook: &hook }, 5, 0, 6, 0)
        }));
        if panic {
            assert!(result.is_err());
            assert_eq!(dev.bytes(), before);
            retired_all(&dev, fs, ino);
        } else {
            result.unwrap().unwrap();
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
fn rust_callback_workers_refuse_all_three_families_in_both_directions() {
    for outer in 0..3 {
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
                    fs.apply_chown("/", 1, 2),
                    fs.apply_chown_bytes(b"/f\xff", 1, 2),
                    fs.apply_chown_ino(ino, 1, 2),
                    fs.apply_utimens("/", 5, 0, 6, 0),
                    fs.apply_utimens_bytes(b"/f\xff", 5, 0, 6, 0),
                    fs.apply_utimens_ino(ino, 5, 0, 6, 0),
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
        let result = match outer {
            0 => fs.apply_chmod_ino(ino, 0o600),
            1 => fs.apply_chown_ino(ino, 1, 2),
            _ => fs.apply_utimens_ino(ino, 5, 0, 6, 0),
        };
        *clock.hook.lock().unwrap() = None;
        workers.finish();
        assert!(responsive.load(Ordering::SeqCst));
        result.unwrap();
        assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn capi_callback_workers_refuse_all_three_families_with_thread_local_errors() {
    for outer in 0..3 {
        let (dev, fs, ino, clock) = with_clock();
        let _fs = with_test_mount(fs, |handle, _| {
            let workers = Workers::default();
            let handles = workers.0.clone();
            let responsive = Arc::new(AtomicBool::new(true));
            let observed_responsive = responsive.clone();
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
                    for entry in 0..6 {
                        let result = unsafe {
                            match entry {
                                0 => fs_ext4_chmod(handle, path.as_ptr(), 0o777),
                                1 => fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o777),
                                2 => fs_ext4_chown(handle, path.as_ptr(), 1, 2),
                                3 => fs_ext4_chown_ino(handle, ino, FS_EXT4_GEN_ANY, 1, 2),
                                4 => fs_ext4_utimens(handle, path.as_ptr(), 5, 0, 6, 0),
                                _ => fs_ext4_utimens_ino(handle, ino, FS_EXT4_GEN_ANY, 5, 0, 6, 0),
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
                match outer {
                    0 => fs_ext4_chmod_ino(handle, ino, FS_EXT4_GEN_ANY, 0o600),
                    1 => fs_ext4_chown_ino(handle, ino, FS_EXT4_GEN_ANY, 1, 2),
                    _ => fs_ext4_utimens_ino(handle, ino, FS_EXT4_GEN_ANY, 5, 0, 6, 0),
                }
            };
            let error = fs_ext4_last_errno();
            *clock.hook.lock().unwrap() = None;
            workers.finish();
            assert!(responsive.load(Ordering::SeqCst));
            assert_eq!((result, error), (0, 0));
        });
        assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
    }
}

fn retired_all(dev: &Device, fs: Filesystem, ino: u32) {
    assert!(fs.operation_failed.load(Ordering::SeqCst));
    dev.reset();
    let before = dev.bytes();
    let fs = with_test_mount(fs, |handle, fs| {
        let calls = AtomicUsize::new(0);
        assert!(matches!(
            fs.apply_utimens_ino(Convert { calls: &calls, ino }, TIME_OMIT, 0, TIME_OMIT, 0),
            Err(Error::ReadOnly)
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(fs.apply_utimens("/", 5, 0, 6, 0).is_err());
        assert!(fs.apply_utimens_bytes(b"/f\xff", 5, 0, 6, 0).is_err());
        assert!(fs.apply_chown_ino(ino, 1, 2).is_err());
        assert_eq!(c_call(handle, ino.into(), false, [(TIME_OMIT, 0); 2]), -1);
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EROFS);
        assert_eq!(c_call(handle, ino.into(), true, [(5, 0), (6, 0)]), -1);
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EROFS);
        assert_eq!(
            unsafe { fs_ext4_chown_ino(handle, ino, FS_EXT4_GEN_ANY, 1, 2) },
            -1
        );
        assert_eq!(unsafe { fs_ext4_flush(handle) }, -1);
        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
    });
    retired(dev, fs, ino);
    assert_eq!(dev.bytes(), before);
}
fn every_fault(capi: bool, flush: bool) {
    for flavor in [features::FsFlavor::Ext4, features::FsFlavor::Ext3] {
        let (dev, fs, ino) = scratch(flavor);
        fs.apply_utimens_ino(ino, 5, 0, 6, 0).unwrap();
        let index = if flush { 2 } else { 1 };
        let total = dev.counts()[index];
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
                        assert_eq!(c_call(handle, ino.into(), by_path, [(5, 0), (6, 0)]), -1);
                        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
                    })
                } else {
                    assert!(rust_call(&fs, ino.into(), by_path, [(5, 0), (6, 0)]).is_err());
                    fs
                };
                assert!(
                    dev.counts()[index] >= nth,
                    "injected callback was not reached"
                );
                retired_all(&dev, fs, ino);
            }
        }
    }
}
#[test]
fn rust_every_marker_commit_write_fault_retires_all_families() {
    every_fault(false, false);
}
#[test]
fn rust_every_marker_commit_flush_fault_retires_all_families() {
    every_fault(false, true);
}
#[test]
fn capi_every_marker_commit_write_fault_retires_all_families() {
    every_fault(true, false);
}
#[test]
fn capi_every_marker_commit_flush_fault_retires_all_families() {
    every_fault(true, true);
}

#[test]
fn rust_and_c_planning_marker_commit_panics_retire_all_families() {
    for capi in [false, true] {
        for by_path in [false, true] {
            for phase in 0..3 {
                let (dev, fs, ino, clock) = with_clock();
                if phase == 0 {
                    *clock.hook.lock().unwrap() =
                        Some(Arc::new(|| panic!("timestamp clock panic")));
                } else {
                    dev.panic_flush.store(phase, Ordering::SeqCst);
                }
                let before = dev.bytes();
                let fs = if capi {
                    with_test_mount(fs, |handle, _| {
                        assert_eq!(c_call(handle, ino.into(), by_path, [(5, 0), (6, 0)]), -1);
                        assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
                        assert!(unsafe { CStr::from_ptr(fs_ext4_last_error()) }
                            .to_string_lossy()
                            .contains("panic:"));
                    })
                } else {
                    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        rust_call(&fs, ino.into(), by_path, [(5, 0), (6, 0)])
                    }))
                    .is_err());
                    fs
                };
                if phase == 0 {
                    assert_eq!(dev.bytes(), before);
                    assert_eq!(&dev.counts()[1..3], &[0, 0]);
                }
                retired_all(&dev, fs, ino);
            }
        }
    }
}

#[test]
fn exported_c_close_preserves_clean_or_dirty_state_after_planning_panic() {
    for prior in [false, true] {
        let (dev, fs, ino, clock) = with_clock();
        let fs = with_test_mount(fs, |handle, fs| {
            if prior {
                assert_eq!(c_call(handle, ino.into(), false, [(5, 0), (6, 0)]), 0);
            }
            *clock.hook.lock().unwrap() =
                Some(Arc::new(|| panic!("timestamp close planning panic")));
            let before = dev.bytes();
            assert_eq!(c_call(handle, ino.into(), true, [(5, 0), (6, 0)]), -1);
            assert_eq!(fs_ext4_last_errno(), crate::error::errno::EIO);
            assert_eq!(dev.bytes(), before);
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
        assert_eq!(dev.bytes(), before);
    }
}

#[test]
fn foreign_context_noop_and_prior_family_failure_preserve_shared_state() {
    let (a, first, ino) = scratch(features::FsFlavor::Ext4);
    let (b, second, other) = scratch(features::FsFlavor::Ext4);
    let mut context = first.chmod_operation().unwrap();
    assert!(matches!(
        second.utimens_ino_in(&mut context, other.into(), [(TIME_OMIT, 0); 2]),
        Err(Error::Stale)
    ));
    assert!(matches!(
        second.utimens_bytes_in(&mut context, b"/f\xff", [(5, 0), (6, 0)]),
        Err(Error::Stale)
    ));
    assert_eq!((a.counts(), b.counts()), ([0; 4], [0; 4]));
    first
        .utimens_ino_in(&mut context, ino.into(), [(5, 0), (6, 0)])
        .unwrap();
    let before = a.bytes();
    let status = context.operation.status();
    first
        .utimens_ino_in(&mut context, ino.into(), [(TIME_OMIT, 0); 2])
        .unwrap();
    assert_eq!(context.operation.status(), status);
    assert_eq!(a.bytes(), before);
    assert!(matches!(
        first.utimens_bytes_in(&mut context, b"/missing", [(5, 0), (6, 0)]),
        Err(Error::NotFound)
    ));
    assert_eq!(context.operation.status(), status);
    drop(context);
    for chown in [false, true] {
        let (dev, fs, ino) = scratch(features::FsFlavor::Ext4);
        dev.fail_flush.store(1, Ordering::SeqCst);
        assert!(if chown {
            fs.apply_chown_ino(ino, 1, 2)
        } else {
            fs.apply_chmod_ino(ino, 0o600)
        }
        .is_err());
        retired_all(&dev, fs, ino);
    }
}
