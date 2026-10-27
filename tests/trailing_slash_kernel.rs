//! Linux lstat provides inode/error outcomes for the selective upstream #589 path fix.
use fs_ext4::{block_io::FileDevice, capi::*, inode::InodeFlags, Error, Filesystem};
use fs_ext4_test_support::{assert_e2fsck_clean, guest_kernel_write, oracle};
use std::{ffi::CString, mem::MaybeUninit, sync::Arc};

#[test]
fn trailing_slashes_match_linux_with_indexed_roots_and_casefold_names() {
    for bs in [1024, 4096] {
        for folded in [false, true] {
            let image = fs_ext4_test_support::temp_path!("trailing-slash-{bs}-{folded}.img");
            std::fs::File::create(&image)
                .unwrap()
                .set_len(64 << 20)
                .unwrap();
            let mut mkfs =
                oracle("mkfs.ext4").args(["-q", "-F", "-b", &bs.to_string(), "-I", "256"]);
            if folded {
                mkfs = mkfs.args(["-O", "casefold", "-E", "encoding=utf8"]);
            }
            let made = mkfs.arg(&image).output();
            assert!(
                made.status.success(),
                "{}",
                String::from_utf8_lossy(&made.stderr)
            );
            let script = format!(
                r#"
cd "$MNT"
mkdir full folded
{fold_command}
python3 - "$MNT" <<'PY'
import os,sys,errno
root=sys.argv[1]
for i in range(1000): open(root+'/root-entry-%04d'%i,'wb').close()
open(root+'/full/x','wb').close()
open(root+'/folded/ReadMe','wb').close()
for name,target in [('dirlink','full'),('chain','dirlink/'),('absolute','/dirlink/'),('filelink','full/x'),('bad-target','full/x/'),('dangling','missing'),('self','self')]:
    # Absolute ext4 targets are resolved inside the filesystem, not the host root.
    os.symlink(target,root+'/'+name)
os.chroot(root)
os.chdir('/')
root=''
for path in ['/', '/.', '/full/', '/dirlink/', '/dirlink//', '/chain/', '/absolute/', '/filelink', '/filelink/', '/bad-target', '/bad-target/', '/bad-target/.', '/dangling', '/dangling/', '/self', '/self/', '/full/x/', '/dirlink/x/', '/folded/ReadMe', '/folded/readme', '/folded/ReadMe/']:
    try: print(path+'|ok|'+str(os.lstat(root+path).st_ino))
    except OSError as e:
        kind={{errno.ENOENT:'missing',errno.ENOTDIR:'notdir',errno.ELOOP:'loop'}}[e.errno]
        print(path+'|'+kind+'|0')
PY
"#,
                fold_command = if folded { "chattr +F folded" } else { ":" }
            );
            let report = guest_kernel_write(&image, &script);
            assert!(
                report.status.success(),
                "{}",
                String::from_utf8_lossy(&report.stderr)
            );
            std::fs::write(format!("{image}.reference.txt"), &report.stdout).unwrap();
            assert_e2fsck_clean(&image, "Linux trailing slash reference");
            let before = std::fs::read(&image).unwrap();
            let fs = Filesystem::mount(Arc::new(FileDevice::open(&image).unwrap())).unwrap();
            assert!(
                fs.read_inode_verified(2)
                    .unwrap()
                    .0
                    .flag_set()
                    .contains(InodeFlags::INDEX),
                "reference root must be indexed"
            );
            let image_c = CString::new(image.as_str()).unwrap();
            let handle = unsafe { fs_ext4_mount(image_c.as_ptr()) };
            assert!(!handle.is_null());
            for row in String::from_utf8(report.stdout).unwrap().lines() {
                let parts: Vec<_> = row.split('|').collect();
                assert_eq!(parts.len(), 3, "unexpected Linux row: {row}");
                let path = parts[0];
                let expected = parts[2].parse::<u32>().unwrap();
                let rust = fs.lookup_path_bytes(path.as_bytes());
                let errno = match parts[1] {
                    "ok" => {
                        assert_eq!(*rust.as_ref().unwrap(), expected, "{bs}/{folded}: {path}");
                        0
                    }
                    "missing" => Error::NotFound.to_errno(),
                    "notdir" => Error::NotADirectory.to_errno(),
                    "loop" => Error::SymlinkLoop.to_errno(),
                    other => panic!("unexpected Linux outcome {other}"),
                };
                if errno != 0 {
                    assert_eq!(rust.unwrap_err().to_errno(), errno, "{bs}/{folded}: {path}");
                }
                let path_c = CString::new(path).unwrap();
                let mut attr = MaybeUninit::<fs_ext4_attr_t>::uninit();
                let rc = unsafe { fs_ext4_stat(handle, path_c.as_ptr(), attr.as_mut_ptr()) };
                if errno == 0 {
                    assert_eq!(rc, 0, "C stat: {path}");
                    assert_eq!(unsafe { attr.assume_init() }.inode, expected);
                } else {
                    assert_eq!(rc, -1);
                }
                assert_eq!(fs_ext4_last_errno(), errno, "C error: {path}");
            }
            unsafe { fs_ext4_umount(handle) };
            assert!(
                std::fs::read(&image).unwrap() == before,
                "read-only traversal changed bytes"
            );
        }
    }
}
