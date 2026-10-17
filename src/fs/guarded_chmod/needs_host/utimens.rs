//! Independent public timestamp metadata, including partial extra fields.
use super::*;
use std::sync::atomic::{AtomicI64, Ordering};

type Time = (i64, u32);
struct TimestampClock(AtomicI64);
impl crate::runtime::Runtime for TimestampClock {
    fn now_unix_seconds(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
    fn next_inode_generation(&self) -> u32 {
        17
    }
}
struct Target {
    path: &'static str,
    ino: u32,
    generation: u32,
    mode: u16,
    uid: u32,
    gid: u32,
}

#[test]
fn linux_checks_public_utimens_metadata() {
    for capi in [false, true] {
        for bs in [1024, 4096] {
            for (inode_size, extra) in [(128, 0), (256, 12), (256, 32)] {
                for journal in [false, true] {
                    check_profile(capi, bs, inode_size, extra, journal);
                }
            }
        }
    }
}
fn update(
    fs: &Filesystem,
    handle: *mut crate::capi::fs_ext4_fs_t,
    capi: bool,
    target: &Target,
    by_path: bool,
    times: [Time; 2],
) {
    let [(a, an), (m, mn)] = times;
    if capi {
        use crate::capi::*;
        let path = std::ffi::CString::new(target.path).unwrap();
        let result = unsafe {
            if by_path {
                fs_ext4_utimens(handle, path.as_ptr(), a, an, m, mn)
            } else {
                fs_ext4_utimens_ino(handle, target.ino, target.generation, a, an, m, mn)
            }
        };
        assert_eq!(result, 0);
        assert_eq!(fs_ext4_last_errno(), 0);
    } else if by_path {
        fs.apply_utimens(target.path, a, an, m, mn).unwrap();
    } else {
        fs.apply_utimens_ino(InodeRef::new(target.ino, target.generation), a, an, m, mn)
            .unwrap();
    }
}
fn debugfs_time(report: &str, label: &str) -> Time {
    let prefix = format!("{label}:");
    let line = report
        .lines()
        .find(|line| line.trim_start().starts_with(&prefix))
        .unwrap();
    let encoded = line.split_whitespace().nth(1).unwrap();
    let (base, extra) = encoded
        .split_once(':')
        .map_or((encoded, "0"), |(base, extra)| (base, extra));
    let base = u32::from_str_radix(base.trim_start_matches("0x"), 16).unwrap();
    let extra = u32::from_str_radix(extra, 16).unwrap();
    (
        i64::from(base as i32) + (i64::from(extra & 3) << 32),
        extra >> 2,
    )
}
fn reports(
    image: &std::path::Path,
    base: &std::path::Path,
    phase: &str,
    targets: &[Target; 2],
    wanted: &[[Time; 3]; 2],
) {
    for (target, times) in targets.iter().zip(wanted) {
        let out = oracle("debugfs")
            .args(["-R", &format!("stat {}", target.path)])
            .arg(image)
            .env("TZ", "GMT")
            .output();
        assert!(out.status.success());
        let report = String::from_utf8(out.stdout).unwrap();
        std::fs::write(
            base.with_extension(format!("{phase}.{}.debugfs.txt", &target.path[1..])),
            &report,
        )
        .unwrap();
        let field = |label: &str| {
            report
                .split(label)
                .nth(1)
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap()
        };
        assert_eq!(field("Inode:").parse::<u32>().unwrap(), target.ino);
        assert_eq!(
            field("Generation:").parse::<u32>().unwrap(),
            target.generation
        );
        assert_eq!(
            u16::from_str_radix(field("Mode:"), 8).unwrap(),
            target.mode & 0xfff
        );
        assert_eq!(
            super::super::owner_report::owner_id(field("User:")).unwrap(),
            target.uid
        );
        assert_eq!(
            super::super::owner_report::owner_id(field("Group:")).unwrap(),
            target.gid
        );
        assert_eq!(
            [
                debugfs_time(&report, "atime"),
                debugfs_time(&report, "mtime"),
                debugfs_time(&report, "ctime")
            ],
            *times,
            "{report}"
        );
    }
}
fn mounted(image: &std::path::Path, clock: &Arc<TimestampClock>) -> Filesystem {
    Filesystem::mount_with_runtime(
        Arc::new(FileDevice::open_rw(image.to_str().unwrap()).unwrap()),
        clock.clone(),
    )
    .unwrap()
}
fn check_profile(capi: bool, bs: u32, inode_size: u16, extra: u16, journal: bool) {
    let interface = if capi { "capi" } else { "rust" };
    let base = fs_ext4_test_support::temp_dir()
        .join(format!("{interface}-{bs}-{inode_size}-{extra}-{journal}"));
    let image = base.with_extension("img");
    let image_str = image.to_str().unwrap();
    std::fs::File::create(&image)
        .unwrap()
        .set_len(32 * 1024 * 1024)
        .unwrap();
    let mut features = if inode_size == 128 {
        "none,filetype"
    } else {
        "none,extent,64bit,filetype,metadata_csum"
    }
    .to_string();
    if journal {
        features.push_str(",has_journal");
    }
    let made = oracle("mke2fs")
        .args([
            "-q",
            "-F",
            "-t",
            "ext4",
            "-b",
            &bs.to_string(),
            "-I",
            &inode_size.to_string(),
            "-m",
            "0",
            "-O",
            &features,
            "-E",
            "lazy_itable_init=0,lazy_journal_init=0",
        ])
        .arg(&image)
        .output();
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
    let populated = guest_kernel_write(
        image_str,
        r#"
printf 'timestamp adapter data' > "$MNT/target"
mkdir "$MNT/directory"
chown 1001:1002 "$MNT/target"
chown 1003:1004 "$MNT/directory"
chmod 6751 "$MNT/target"
chmod 2750 "$MNT/directory"
python3 - "$MNT" <<'INPUT_PY'
import os, sys
for name in ['target','directory']:
    os.utime(sys.argv[1]+'/'+name, ns=(1600000000000000000,1600000001000000000))
INPUT_PY
"#,
    );
    assert!(
        populated.status.success(),
        "{}",
        String::from_utf8_lossy(&populated.stderr)
    );
    if inode_size == 256 && extra == 12 {
        oracle("debugfs")
            .args(["-w", "-f", "-"])
            .arg(&image)
            .stdin("sif /target extra_isize 12\nsif /directory extra_isize 12\n")
            .judged()
            .clean("debugfs constructs the supported short-extra timestamp profile");
    }
    assert_e2fsck_clean(image_str, "Linux timestamp input");
    let clock = Arc::new(TimestampClock(AtomicI64::new(NOW)));
    let fs = mounted(&image, &clock);
    assert_eq!(fs.journal.is_some(), journal);
    let target = |path, mode, uid, gid| {
        let ino = fs.lookup_path_bytes(path).unwrap();
        let (inode, raw) = fs.read_inode_verified(ino).unwrap();
        assert_eq!(raw.len(), inode_size as usize);
        if inode_size == 256 {
            assert_eq!(u16::from_le_bytes(raw[128..130].try_into().unwrap()), extra);
        }
        assert_eq!((inode.mode, inode.uid, inode.gid), (mode, uid, gid));
        Target {
            path: if path == b"/target" {
                "/target"
            } else {
                "/directory"
            },
            ino,
            generation: inode.generation,
            mode,
            uid,
            gid,
        }
    };
    let targets = [
        target(b"/target", crate::inode::S_IFREG | 0o6751, 1001, 1002),
        target(b"/directory", crate::inode::S_IFDIR | 0o2750, 1003, 1004),
    ];
    let has_a = extra >= 16;
    let has_m = extra >= 12;
    let p = |n, has| if has { n } else { 0 };
    let file_m = if has_m { NOW + 100 } else { i32::MAX as i64 };
    let explicit = [
        [(-123, 123_456_789), (file_m, 987_654_321)],
        [(-234, 345_678_901), (1_600_000_000, 456_789_012)],
    ];
    let wanted_a = [
        [
            (-123, p(123_456_789, has_a)),
            (file_m, p(987_654_321, has_m)),
            (if extra >= 8 { NOW } else { i32::MAX as i64 }, 0),
        ],
        [
            (-234, p(345_678_901, has_a)),
            (1_600_000_000, p(456_789_012, has_m)),
            (if extra >= 8 { NOW } else { i32::MAX as i64 }, 0),
        ],
    ];
    let before = std::fs::read(&image).unwrap();
    let fs = crate::capi::with_test_mount(fs, |handle, fs| {
        let held = fs.chmod_operation().unwrap();
        if capi {
            let path = std::ffi::CString::new("/target").unwrap();
            assert_eq!(
                unsafe { crate::capi::fs_ext4_utimens(handle, path.as_ptr(), 5, 0, 6, 0) },
                -1
            );
        } else {
            assert!(matches!(
                fs.apply_utimens("/target", 5, 0, 6, 0),
                Err(Error::Unsupported(_))
            ));
        }
        assert_eq!(std::fs::read(&image).unwrap(), before);
        drop(held);
        if !has_a {
            assert!(matches!(
                fs.apply_utimens_ino(targets[0].ino, NOW, 0, TIME_OMIT, 0),
                Err(Error::InvalidArgument(_))
            ));
            assert_eq!(std::fs::read(&image).unwrap(), before);
        }
        for (index, target) in targets.iter().enumerate() {
            update(fs, handle, capi, target, index == 0, explicit[index]);
        }
        assert_eq!(fs.chmod_operation().unwrap().operation.status().epoch, 3);
    });
    fs.finish().unwrap();
    assert_e2fsck_clean(image_str, "explicit public timestamp output");
    reports(&image, &base, "explicit", &targets, &wanted_a);
    let now = if has_a { NOW + 200 } else { 1_700_000_200 };
    clock.0.store(now, Ordering::SeqCst);
    let fs = mounted(&image, &clock);
    let mixed = [
        [(i64::MAX, UTIME_NOW), (TIME_OMIT, u32::MAX)],
        [(i64::MAX, UTIME_OMIT), (i64::MAX, UTIME_NOW)],
    ];
    let wanted_b = [
        [(now, 0), wanted_a[0][1], (now, 0)],
        [wanted_a[1][0], (now, 0), (now, 0)],
    ];
    let fs = crate::capi::with_test_mount(fs, |handle, fs| {
        for (index, target) in targets.iter().enumerate() {
            update(fs, handle, capi, target, index != 0, mixed[index]);
        }
        assert_eq!(fs.chmod_operation().unwrap().operation.status().epoch, 3);
    });
    fs.finish().unwrap();
    assert_e2fsck_clean(image_str, "sentinel public timestamp output");
    reports(&image, &base, "sentinel", &targets, &wanted_b);
    clock.0.store(NOW + 400, Ordering::SeqCst);
    let fs = mounted(&image, &clock);
    let before = std::fs::read(&image).unwrap();
    let fs = crate::capi::with_test_mount(fs, |handle, fs| {
        for (index, target) in targets.iter().enumerate() {
            update(
                fs,
                handle,
                capi,
                target,
                index == 0,
                [(TIME_OMIT, u32::MAX), (i64::MAX, UTIME_OMIT)],
            );
        }
        assert_eq!(std::fs::read(&image).unwrap(), before);
        assert_eq!(fs.chmod_operation().unwrap().operation.status().epoch, 0);
        assert!(!fs.marked_not_clean.load(Ordering::SeqCst));
    });
    fs.finish().unwrap();
    let nanos = |t: Time| i128::from(t.0) * 1_000_000_000 + i128::from(t.1);
    let [file_times, dir_times] = wanted_b;
    let checked = guest_kernel_write(
        image_str,
        &format!(
            r#"
python3 - "$MNT" <<'TIMESTAMP_PY'
import os, stat, sys
root=sys.argv[1]
for name,ino,generation,mode,uid,gid,a,m,c in [
    ('target',{file_ino},{file_gen},{file_mode},1001,1002,{file_a},{file_m},{file_c}),
    ('directory',{dir_ino},{dir_gen},{dir_mode},1003,1004,{dir_a},{dir_m},{dir_c}),
]:
    info=os.stat(root+'/'+name)
    assert info.st_ino == ino, info
    assert info.st_mode == mode, info
    assert (info.st_uid,info.st_gid)==(uid,gid), info
    assert (info.st_atime_ns,info.st_mtime_ns,info.st_ctime_ns)==(a,m,c), info
    print(name,info.st_ino,info.st_mode,info.st_uid,info.st_gid,info.st_atime_ns,info.st_mtime_ns,info.st_ctime_ns)
fd=os.open(root+'/target',os.O_RDONLY | os.O_NOATIME)
try:
    assert os.read(fd,1024) == b'timestamp adapter data'
finally:
    os.close(fd)
TIMESTAMP_PY
"#,
            file_ino = targets[0].ino,
            file_gen = targets[0].generation,
            file_mode = targets[0].mode,
            dir_ino = targets[1].ino,
            dir_gen = targets[1].generation,
            dir_mode = targets[1].mode,
            file_a = nanos(file_times[0]),
            file_m = nanos(file_times[1]),
            file_c = nanos(file_times[2]),
            dir_a = nanos(dir_times[0]),
            dir_m = nanos(dir_times[1]),
            dir_c = nanos(dir_times[2])
        ),
    );
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    std::fs::write(base.with_extension("linux.txt"), &checked.stdout).unwrap();
    assert_e2fsck_clean(image_str, "Linux readback after public timestamps");
}
