//! Independent evidence for existing public flag updates.
use super::super::timestamp_report::debugfs_time;
use super::*;
const USER_FLAGS: u32 =
    crate::inode::InodeFlags::NODUMP.bits() | crate::inode::InodeFlags::NOATIME.bits();
use std::sync::atomic::{AtomicI64, Ordering};
struct FlagClock(AtomicI64);
impl crate::runtime::Runtime for FlagClock {
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
    flags: u32,
}
fn update(
    fs: &Filesystem,
    handle: *mut crate::capi::fs_ext4_fs_t,
    capi: bool,
    path: &str,
    flags: u32,
) {
    if capi {
        let path = std::ffi::CString::new(path).unwrap();
        assert_eq!(
            unsafe { crate::capi::fs_ext4_set_flags(handle, path.as_ptr(), flags) },
            0
        );
        assert_eq!(crate::capi::fs_ext4_last_errno(), 0);
    } else {
        fs.apply_set_flags(path, flags).unwrap();
    }
}
#[test]
fn linux_checks_public_flags_metadata() {
    for capi in [false, true] {
        for bs in [1024, 4096] {
            for size in [128, 256] {
                for journal in [false, true] {
                    profile(capi, bs, size, journal);
                }
            }
        }
    }
}
fn profile(capi: bool, bs: u32, size: u16, journal: bool) {
    let base = fs_ext4_test_support::temp_dir().join(format!(
        "{}-{bs}-{size}-{journal}",
        if capi { "capi" } else { "rust" }
    ));
    let image = base.with_extension("img");
    let image_str = image.to_str().unwrap();
    std::fs::File::create(&image)
        .unwrap()
        .set_len(32 * 1024 * 1024)
        .unwrap();
    let mut features = if size == 128 {
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
            &size.to_string(),
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
printf 'flag adapter data' > "$MNT/target"
mkdir "$MNT/directory"
chown 1001:1002 "$MNT/target"
chown 1003:1004 "$MNT/directory"
chmod 6751 "$MNT/target"
chmod 2750 "$MNT/directory"
python3 - "$MNT" <<'FLAGS_INPUT_PY'
import os,sys
for name in ['target','directory']:
    os.utime(sys.argv[1]+'/'+name,ns=(1600000000000000000,1600000001000000000))
FLAGS_INPUT_PY
"#,
    );
    assert!(
        populated.status.success(),
        "{}",
        String::from_utf8_lossy(&populated.stderr)
    );
    assert_e2fsck_clean(image_str, "Linux public flags input");
    let start = if size == 128 { 1_700_000_100 } else { NOW };
    let clock = Arc::new(FlagClock(AtomicI64::new(start)));
    let mount = || {
        Filesystem::mount_with_runtime(
            Arc::new(FileDevice::open_rw(image_str).unwrap()),
            clock.clone(),
        )
        .unwrap()
    };
    let fs = mount();
    assert_eq!(fs.journal.is_some(), journal);
    let target = |path: &'static str, mode, uid, gid| {
        let ino = fs.lookup_path_bytes(path.as_bytes()).unwrap();
        let (inode, raw) = fs.read_inode_verified(ino).unwrap();
        assert_eq!(raw.len(), size as usize);
        if size == 256 {
            assert_eq!(u16::from_le_bytes(raw[128..130].try_into().unwrap()), 32);
        }
        assert_eq!((inode.mode, inode.uid, inode.gid), (mode, uid, gid));
        Target {
            path,
            ino,
            generation: inode.generation,
            mode,
            uid,
            gid,
            flags: inode.flags,
        }
    };
    let targets = [
        target("/target", crate::inode::S_IFREG | 0o6751, 1001, 1002),
        target("/directory", crate::inode::S_IFDIR | 0o2750, 1003, 1004),
    ];
    let before = std::fs::read(&image).unwrap();
    let fs = crate::capi::with_test_mount(fs, |handle, fs| {
        let held = fs.chmod_operation().unwrap();
        if capi {
            let path = std::ffi::CString::new("/target").unwrap();
            assert_eq!(
                unsafe {
                    crate::capi::fs_ext4_set_flags(handle, path.as_ptr(), targets[0].flags | 0x40)
                },
                -1
            );
            assert_eq!(
                crate::capi::fs_ext4_last_errno(),
                crate::error::errno::ENOTSUP
            );
        } else {
            assert!(matches!(
                fs.apply_set_flags("/target", targets[0].flags | 0x40),
                Err(Error::Unsupported(_))
            ));
        }
        assert!(std::fs::read(&image).unwrap() == before);
        drop(held);
        for target in &targets {
            let flags = target.flags ^ crate::inode::InodeFlags::CASEFOLD.bits();
            if capi {
                let path = std::ffi::CString::new(target.path).unwrap();
                assert_eq!(
                    unsafe { crate::capi::fs_ext4_set_flags(handle, path.as_ptr(), flags) },
                    -1
                );
                assert_eq!(
                    crate::capi::fs_ext4_last_errno(),
                    crate::error::errno::EINVAL
                );
            } else {
                assert!(matches!(
                    fs.apply_set_flags(target.path, flags),
                    Err(Error::InvalidArgument(_))
                ));
            }
            assert!(std::fs::read(&image).unwrap() == before);
        }
        assert!(!fs.marked_not_clean.load(Ordering::SeqCst));
        assert_eq!(fs.chmod_operation().unwrap().operation.status().epoch, 0);
    });
    fs.finish().unwrap();
    for (step, phase) in ["set", "same", "restore"].into_iter().enumerate() {
        let now = start + step as i64;
        clock.0.store(now, Ordering::SeqCst);
        let fs = mount();
        let wanted = targets.each_ref().map(|t| {
            if phase == "restore" {
                t.flags
            } else {
                t.flags | USER_FLAGS
            }
        });
        let fs = crate::capi::with_test_mount(fs, |handle, fs| {
            for (t, flags) in targets.iter().zip(wanted) {
                update(fs, handle, capi, t.path, flags);
            }
            assert_eq!(fs.chmod_operation().unwrap().operation.status().epoch, 3);
        });
        fs.finish().unwrap();
        assert_e2fsck_clean(image_str, "emitted public flags phase");
        for (target, flags) in targets.iter().zip(wanted) {
            let out = oracle("debugfs")
                .args(["-R", &format!("stat {}", target.path)])
                .arg(&image)
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
                u32::from_str_radix(field("Flags:").trim_start_matches("0x"), 16).unwrap(),
                flags
            );
            assert_eq!(field("User:").parse::<u32>().unwrap(), target.uid);
            assert_eq!(field("Group:").parse::<u32>().unwrap(), target.gid);
            assert_eq!(
                [
                    debugfs_time(&report, "atime"),
                    debugfs_time(&report, "mtime"),
                    debugfs_time(&report, "ctime")
                ],
                [(1600000000, 0), (1600000001, 0), (now, 0)]
            );
        }
        // Public Linux ioctl ABI for the 64-bit harness VM; no kernel headers
        // or implementation source is used. stat and GETFLAGS are independent
        // of this crate and of the debugfs report decoder.
        let checked = guest_kernel_write(
            image_str,
            &format!(
                r#"
python3 - "$MNT" <<'FLAGS_READ_PY'
import array,fcntl,os,sys
root=sys.argv[1]
assert array.array('L').itemsize==8
for name,ino,mode,uid,gid,flags in [
 ('target',{file_ino},{file_mode},1001,1002,{file_flags}),
 ('directory',{dir_ino},{dir_mode},1003,1004,{dir_flags}),
]:
 info=os.stat(root+'/'+name)
 assert (info.st_ino,info.st_mode,info.st_uid,info.st_gid)==(ino,mode,uid,gid),info
 assert (info.st_atime_ns,info.st_mtime_ns,info.st_ctime_ns)==(1600000000000000000,1600000001000000000,{ctime}),info
 fd=os.open(root+'/'+name,os.O_RDONLY|os.O_NOATIME)
 try:
  word=array.array('L',[0])
  fcntl.ioctl(fd,0x80086601,word,True)
  assert word[0]==flags,(name,word[0],flags)
  if name=='target': assert os.read(fd,1024)==b'flag adapter data'
 finally:
  os.close(fd)
 print(name,info.st_ino,info.st_mode,info.st_uid,info.st_gid,word[0],info.st_atime_ns,info.st_mtime_ns,info.st_ctime_ns)
FLAGS_READ_PY
"#,
                file_ino = targets[0].ino,
                file_mode = targets[0].mode,
                file_flags = wanted[0],
                dir_ino = targets[1].ino,
                dir_mode = targets[1].mode,
                dir_flags = wanted[1],
                ctime = i128::from(now) * 1_000_000_000
            ),
        );
        assert!(
            checked.status.success(),
            "{}",
            String::from_utf8_lossy(&checked.stderr)
        );
        std::fs::write(
            base.with_extension(format!("{phase}.linux.txt")),
            &checked.stdout,
        )
        .unwrap();
        assert_e2fsck_clean(image_str, "Linux public flags phase readback");
    }
}
