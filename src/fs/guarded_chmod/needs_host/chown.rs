//! Independent full-width owner-field evidence on supported inode layouts.
use super::*;
use std::sync::atomic::{AtomicI64, Ordering};

const FIRST_UID: u32 = 0xfedc_9876;
const FIRST_GID: u32 = 0x89ab_cdef;
const FINAL_UID: u32 = 0x7654_3210;
const FINAL_GID: u32 = 0x8765_4321;
const DIR_UID: u32 = 0x3456_7890;
const DIR_GID: u32 = 0xcdef_0123;

struct OwnerClock(AtomicI64);
impl crate::runtime::Runtime for OwnerClock {
    fn now_unix_seconds(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
    fn next_inode_generation(&self) -> u32 {
        17
    }
}

#[test]
fn linux_checks_public_chown_metadata() {
    for capi in [false, true] {
        for bs in [1024, 4096] {
            for inode_size in [128, 256] {
                for journal in [false, true] {
                    check_profile(capi, bs, inode_size, journal);
                }
            }
        }
    }
}

fn change(
    fs: &Filesystem,
    handle: *mut crate::capi::fs_ext4_fs_t,
    capi: bool,
    path: Option<&str>,
    r: InodeRef,
    uid: u32,
    gid: u32,
) {
    if capi {
        use crate::capi::*;
        let result = unsafe {
            if let Some(path) = path {
                let path = std::ffi::CString::new(path).unwrap();
                fs_ext4_chown(handle, path.as_ptr(), uid, gid)
            } else {
                fs_ext4_chown_ino(handle, r.ino, r.generation.unwrap(), uid, gid)
            }
        };
        assert_eq!(result, 0);
        assert_eq!(fs_ext4_last_errno(), 0);
    } else if let Some(path) = path {
        fs.apply_chown(path, uid, gid).unwrap();
    } else {
        fs.apply_chown_ino(r, uid, gid).unwrap();
    }
}

fn check_profile(capi: bool, bs: u32, inode_size: u16, journal: bool) {
    let label = if capi { "capi" } else { "rust" };
    let base = fs_ext4_test_support::temp_dir()
        .join(format!("public-{label}-chown-{bs}-{inode_size}-{journal}"));
    let image = base.with_extension("img");
    let image_str = image.to_str().unwrap();
    std::fs::File::create(&image)
        .unwrap()
        .set_len(32 * 1024 * 1024)
        .unwrap();
    let mut features = if inode_size == 256 {
        "none,extent,64bit,filetype,metadata_csum"
    } else {
        "none,filetype"
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
printf 'owner adapter data' > "$MNT/target"
mkdir "$MNT/directory"
chown 1001:1002 "$MNT/target"
chown 1003:1004 "$MNT/directory"
chmod 6751 "$MNT/target"
chmod 2750 "$MNT/directory"
python3 - "$MNT" <<'TIMES_PY'
import os, sys
for name in ['target', 'directory']:
    os.utime(sys.argv[1] + '/' + name, ns=(1600000000000000000, 1600000001000000000))
TIMES_PY
"#,
    );
    assert!(
        populated.status.success(),
        "{}",
        String::from_utf8_lossy(&populated.stderr)
    );
    assert_e2fsck_clean(image_str, "Linux chown input");
    let clock = Arc::new(OwnerClock(AtomicI64::new(NOW)));
    let fs = Filesystem::mount_with_runtime(
        Arc::new(FileDevice::open_rw(image_str).unwrap()),
        clock.clone(),
    )
    .unwrap();
    assert_eq!(fs.sb.inode_size, inode_size);
    assert_eq!(fs.journal.is_some(), journal);
    let file = fs.lookup_path_bytes(b"/target").unwrap();
    let directory = fs.lookup_path_bytes(b"/directory").unwrap();
    let before_file = fs.read_inode_verified(file).unwrap().0;
    let before_dir = fs.read_inode_verified(directory).unwrap().0;
    assert_eq!(
        (
            before_file.uid,
            before_file.gid,
            before_dir.uid,
            before_dir.gid
        ),
        (1001, 1002, 1003, 1004)
    );
    let before_bytes = std::fs::read(&image).unwrap();
    let fs = crate::capi::with_test_mount(fs, |handle, fs| {
        let held = fs.chmod_operation().unwrap();
        if capi {
            use crate::capi::*;
            let path = std::ffi::CString::new("/target").unwrap();
            assert_eq!(unsafe { fs_ext4_chown(handle, path.as_ptr(), 1, 2) }, -1);
            assert_eq!(
                unsafe { fs_ext4_chown_ino(handle, file, before_file.generation, 1, 2) },
                -1
            );
        } else {
            assert!(matches!(
                fs.apply_chown("/target", 1, 2),
                Err(Error::Unsupported(_))
            ));
            assert!(matches!(
                fs.apply_chown_ino(file, 1, 2),
                Err(Error::Unsupported(_))
            ));
        }
        assert_eq!(std::fs::read(&image).unwrap(), before_bytes);
        drop(held);
        for (step, (uid, gid, wanted_uid, wanted_gid)) in [
            (FIRST_UID, FIRST_GID, FIRST_UID, FIRST_GID),
            (u32::MAX, FINAL_GID, FIRST_UID, FINAL_GID),
            (FINAL_UID, u32::MAX, FINAL_UID, FINAL_GID),
            (u32::MAX, u32::MAX, FINAL_UID, FINAL_GID),
        ]
        .into_iter()
        .enumerate()
        {
            clock.0.store(NOW + step as i64, Ordering::SeqCst);
            change(
                fs,
                handle,
                capi,
                (step % 2 == 0).then_some("/target"),
                InodeRef::new(file, before_file.generation),
                uid,
                gid,
            );
            let after = fs.read_inode_verified(file).unwrap().0;
            assert_eq!((after.uid, after.gid), (wanted_uid, wanted_gid));
            let wanted_time = if inode_size == 128 {
                i32::MAX as i64
            } else {
                NOW + step as i64
            };
            assert_eq!(after.ctime, wanted_time);
        }
        clock.0.store(NOW + 4, Ordering::SeqCst);
        change(
            fs,
            handle,
            capi,
            None,
            InodeRef::new(directory, before_dir.generation),
            DIR_UID,
            DIR_GID,
        );
        let context = fs.chmod_operation().unwrap();
        assert_eq!(context.operation.status().epoch, 6); // marker + five commits
    });
    for (ino, before) in [(file, &before_file), (directory, &before_dir)] {
        let after = fs.read_inode_verified(ino).unwrap().0;
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
    }
    fs.finish().unwrap();
    assert_e2fsck_clean(image_str, "public chown emitted metadata");
    for (path, kind, mode, generation, uid, gid, step) in [
        (
            "target",
            "regular",
            0o6751,
            before_file.generation,
            FINAL_UID,
            FINAL_GID,
            3,
        ),
        (
            "directory",
            "directory",
            0o2750,
            before_dir.generation,
            DIR_UID,
            DIR_GID,
            4,
        ),
    ] {
        let out = oracle("debugfs")
            .args(["-R", &format!("stat /{path}")])
            .arg(&image)
            .env("TZ", "GMT")
            .output();
        assert!(out.status.success());
        let report = String::from_utf8(out.stdout).unwrap();
        std::fs::write(base.with_extension(format!("{path}.debugfs.txt")), &report).unwrap();
        let header = report.lines().find(|line| line.contains("Type:")).unwrap();
        assert!(header.contains(&format!("Type: {kind}")), "{report}");
        assert_eq!(
            u16::from_str_radix(
                header
                    .split("Mode:")
                    .nth(1)
                    .unwrap()
                    .split_whitespace()
                    .next()
                    .unwrap(),
                8
            )
            .unwrap(),
            mode
        );
        for (label, expected) in [("User:", uid), ("Group:", gid), ("Generation:", generation)] {
            let found = report
                .split(label)
                .nth(1)
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap()
                .parse::<u32>()
                .unwrap();
            assert_eq!(found, expected, "{report}");
        }
        let wanted_time = if inode_size == 128 {
            i32::MAX as i64
        } else {
            NOW + step
        };
        let ctime = report
            .lines()
            .find(|line| line.trim_start().starts_with("ctime:"))
            .unwrap();
        assert!(
            ctime.contains(&format!("0x{:08x}", wanted_time as u32)),
            "{report}"
        );
        assert_eq!(
            report.contains("Inode checksum:"),
            inode_size == 256,
            "{report}"
        );
    }
    let checked = guest_kernel_write(
        image_str,
        &format!(
            r#"
python3 - "$MNT" <<'CHOWN_PY'
import os, stat, sys
root = sys.argv[1]
for name, kind, mode, uid, gid, step in [
    ('target', stat.S_IFREG, 0o6751, {FINAL_UID}, {FINAL_GID}, 3),
    ('directory', stat.S_IFDIR, 0o2750, {DIR_UID}, {DIR_GID}, 4),
]:
    info = os.stat(root + '/' + name)
    assert stat.S_IFMT(info.st_mode) == kind, info
    assert stat.S_IMODE(info.st_mode) == mode, info
    assert (info.st_uid, info.st_gid) == (uid, gid), info
    expected_ctime = 2147483647 if {inode_size} == 128 else 2147483658 + step
    assert info.st_ctime_ns == expected_ctime * 1000000000, info
    assert info.st_atime_ns == 1600000000000000000, info
    assert info.st_mtime_ns == 1600000001000000000, info
    print(name, info.st_uid, info.st_gid, oct(info.st_mode), info.st_ctime_ns)
with open(root + '/target', 'rb') as stream:
    assert stream.read() == b'owner adapter data'
CHOWN_PY
"#
        ),
    );
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    std::fs::write(base.with_extension("linux.txt"), &checked.stdout).unwrap();
    assert_e2fsck_clean(image_str, "Linux readback after public chown");
}
