//! Independent emitted-metadata evidence for the guarded metadata families.
mod chown;
mod flags;
mod removexattr;
mod setxattr;
mod setxattr_policy;
mod unlink_policy;
mod utimens;
use super::*;
use crate::block_io::FileDevice;
use fs_ext4_test_support::{assert_e2fsck_clean, guest_kernel_write, oracle};

const NOW: i64 = (1i64 << 31) + 10;

struct Clock;
impl crate::runtime::Runtime for Clock {
    fn now_unix_seconds(&self) -> i64 {
        NOW
    }
    fn next_inode_generation(&self) -> u32 {
        17
    }
}

#[test]
fn linux_checks_private_chmod_metadata() {
    check_profiles(Dispatch::Private);
}

#[test]
fn linux_checks_public_chmod_metadata() {
    check_profiles(Dispatch::Rust);
    check_profiles(Dispatch::C);
}

#[derive(Clone, Copy)]
enum Dispatch {
    Private,
    Rust,
    C,
}

fn check_profiles(dispatch: Dispatch) {
    let prefix = match dispatch {
        Dispatch::Private => "private-chmod",
        Dispatch::Rust => "public-rust-chmod",
        Dispatch::C => "public-capi-chmod",
    };
    for bs in [1024, 4096] {
        for journal in [false, true] {
            let base = fs_ext4_test_support::temp_dir().join(format!("{prefix}-{bs}-{journal}"));
            let image = base.with_extension("img");
            let image_str = image.to_str().unwrap();
            std::fs::File::create(&image)
                .unwrap()
                .set_len(32 * 1024 * 1024)
                .unwrap();
            let mut features = String::from("none,extent,64bit,filetype,metadata_csum");
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
                    "256",
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
printf 'private adapter data' > "$MNT/target"
mkdir "$MNT/directory"
chmod 640 "$MNT/target"
chmod 755 "$MNT/directory"
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
            assert_e2fsck_clean(image_str, "Linux private chmod input");
            let fs = Filesystem::mount_with_runtime(
                Arc::new(FileDevice::open_rw(image_str).unwrap()),
                Arc::new(Clock),
            )
            .unwrap();
            assert_eq!(fs.journal.is_some(), journal);
            let file = fs.lookup_path_bytes(b"/target").unwrap();
            let directory = fs.lookup_path_bytes(b"/directory").unwrap();
            let before_file = fs.read_inode_verified(file).unwrap().0;
            let before_dir = fs.read_inode_verified(directory).unwrap().0;
            let before_bytes = std::fs::read(&image).unwrap();
            let fs = crate::capi::with_test_mount(fs, |handle, fs| {
                match dispatch {
                    Dispatch::Private => {
                        fs.guarded_chmod_bytes(b"/target", 0o16751).unwrap();
                        fs.guarded_chmod_ino(
                            InodeRef::new(directory, before_dir.generation),
                            0o12750,
                        )
                        .unwrap();
                    }
                    Dispatch::Rust => {
                        let held = fs.chmod_operation().unwrap();
                        assert!(matches!(
                            fs.apply_chmod("/target", 0o777),
                            Err(Error::Unsupported(_))
                        ));
                        assert!(matches!(
                            fs.apply_chmod_ino(file, 0o777),
                            Err(Error::Unsupported(_))
                        ));
                        assert_eq!(std::fs::read(&image).unwrap(), before_bytes);
                        drop(held);
                        fs.apply_chmod("/target", 0o16751).unwrap();
                        fs.apply_chmod_ino(
                            InodeRef::new(directory, before_dir.generation),
                            0o12750,
                        )
                        .unwrap();
                    }
                    Dispatch::C => {
                        use crate::capi::{fs_ext4_chmod, fs_ext4_chmod_ino, fs_ext4_last_errno};
                        let path = std::ffi::CString::new("/target").unwrap();
                        let held = fs.chmod_operation().unwrap();
                        assert_eq!(unsafe { fs_ext4_chmod(handle, path.as_ptr(), 0o777) }, -1);
                        assert_eq!(
                            unsafe {
                                fs_ext4_chmod_ino(handle, directory, before_dir.generation, 0o777)
                            },
                            -1
                        );
                        assert_eq!(std::fs::read(&image).unwrap(), before_bytes);
                        drop(held);
                        assert_eq!(unsafe { fs_ext4_chmod(handle, path.as_ptr(), 0o16751) }, 0);
                        assert_eq!(fs_ext4_last_errno(), 0);
                        assert_eq!(
                            unsafe {
                                fs_ext4_chmod_ino(handle, directory, before_dir.generation, 0o12750)
                            },
                            0
                        );
                        assert_eq!(fs_ext4_last_errno(), 0);
                    }
                }
                let context = fs.chmod_operation().unwrap();
                // Every actual dispatch traversed one marker and two commits.
                assert_eq!(context.operation.status().epoch, 3);
            });
            for (ino, before) in [(file, &before_file), (directory, &before_dir)] {
                let after = fs.read_inode_verified(ino).unwrap().0;
                assert_eq!(
                    (after.ctime, after.mtime, after.atime, after.generation),
                    (NOW, before.mtime, before.atime, before.generation)
                );
            }
            fs.finish().unwrap();
            assert_e2fsck_clean(image_str, "private chmod emitted metadata");
            for (path, kind, mode, generation) in [
                ("target", "regular", 0o6751, before_file.generation),
                ("directory", "directory", 0o2750, before_dir.generation),
            ] {
                let out = oracle("debugfs")
                    .args(["-R", &format!("stat /{path}")])
                    .arg(&image)
                    .env("TZ", "GMT")
                    .output();
                assert!(out.status.success());
                let report = String::from_utf8(out.stdout).unwrap();
                std::fs::write(base.with_extension(format!("{path}.debugfs.txt")), &report)
                    .unwrap();
                let header = report
                    .lines()
                    .find(|line| line.contains("Type:"))
                    .expect("debugfs inode header");
                assert!(header.contains(&format!("Type: {kind}")), "{report}");
                let found_mode = header
                    .split("Mode:")
                    .nth(1)
                    .unwrap()
                    .split_whitespace()
                    .next()
                    .unwrap();
                assert_eq!(
                    u16::from_str_radix(found_mode, 8).unwrap(),
                    mode,
                    "{report}"
                );
                let gen_line = report
                    .lines()
                    .find(|line| line.contains("Generation:"))
                    .unwrap();
                assert_eq!(
                    gen_line
                        .split("Generation:")
                        .nth(1)
                        .unwrap()
                        .split_whitespace()
                        .next()
                        .unwrap()
                        .parse::<u32>()
                        .unwrap(),
                    generation,
                    "{report}"
                );
                let ctime = report
                    .lines()
                    .find(|line| line.trim_start().starts_with("ctime:"))
                    .unwrap();
                assert!(ctime.ends_with("-- Tue Jan 19 03:14:18 2038"), "{report}");
                assert!(report.contains("Inode checksum:"), "{report}");
            }
            // A real Linux mount must find both saved modes, exact ctime and
            // the original file data. The helper copies back after unmount.
            let checked = guest_kernel_write(
                image_str,
                r#"
python3 - "$MNT" <<'CHMOD_PY'
import os, stat, sys
root = sys.argv[1]
for name, kind, mode in [('target', stat.S_IFREG, 0o6751), ('directory', stat.S_IFDIR, 0o2750)]:
    info = os.stat(root + '/' + name)
    assert stat.S_IFMT(info.st_mode) == kind, info
    assert stat.S_IMODE(info.st_mode) == mode, info
    assert info.st_ctime_ns == 2147483658 * 1000000000, info
    assert info.st_atime_ns == 1600000000000000000, info
    assert info.st_mtime_ns == 1600000001000000000, info
    print(name, oct(info.st_mode), info.st_ctime_ns)
with open(root + '/target', 'rb') as stream:
    assert stream.read() == b'private adapter data'
CHMOD_PY
"#,
            );
            assert!(
                checked.status.success(),
                "{}",
                String::from_utf8_lossy(&checked.stderr)
            );
            std::fs::write(base.with_extension("linux.txt"), &checked.stdout).unwrap();
            assert_e2fsck_clean(image_str, "Linux readback after private chmod");
        }
    }
}
