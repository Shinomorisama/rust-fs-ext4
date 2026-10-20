//! Adapted from upstream 4db7413 (#558): Linux judges path traversal,
//! namespace edits and loop limits through both Rust and the C ABI.
//! The newer CLI namespace commands are outside this backport.
mod cli_support;
use cli_support::*;
use fs_ext4::{block_io::FileDevice, capi::*, Error, Filesystem};
use fs_ext4_test_support::{assert_e2fsck_clean, guest_kernel_write, oracle};
use std::{collections::BTreeMap, ffi::CString, mem::MaybeUninit, sync::Arc};

struct CMount(*mut fs_ext4_fs_t);
impl CMount {
    fn open(img: &str, writable: bool) -> Self {
        let name = CString::new(img).unwrap();
        let handle = unsafe {
            if writable {
                fs_ext4_mount_rw(name.as_ptr())
            } else {
                fs_ext4_mount(name.as_ptr())
            }
        };
        assert!(
            !handle.is_null(),
            "C mount failed: {}",
            fs_ext4_last_errno()
        );
        Self(handle)
    }
    fn stat(&self, path: &str) -> Result<fs_ext4_attr_t, i32> {
        let path = CString::new(path).unwrap();
        let mut attr = MaybeUninit::uninit();
        let rc = unsafe { fs_ext4_stat(self.0, path.as_ptr(), attr.as_mut_ptr()) };
        if rc == 0 {
            Ok(unsafe { attr.assume_init() })
        } else {
            Err(fs_ext4_last_errno())
        }
    }
}
impl Drop for CMount {
    fn drop(&mut self) {
        unsafe { fs_ext4_umount(self.0) };
    }
}

/// Every path on the kernel's mount, the root included, one line each:
/// `path|type|links|inode|content`, the content being a regular file's
/// SHA-256 and a symlink's target.
const LISTING: &str = r#"find . | LC_ALL=C sort | while read -r p; do
    case "$(stat -c %F -- "$p")" in
        regular*) content="$(sha256sum <"$p" | cut -d' ' -f1)" ;;
        symbolic*) content="$(readlink -- "$p")" ;;
        *) content=- ;;
    esac
    q="${p#.}"
    printf '%s|%s|%s\n' "${q:-/}" "$(stat -c '%F|%h|%i' -- "$p")" "$content"
done
"#;

fn listing(out: &std::process::Output) -> BTreeMap<String, String> {
    assert!(
        out.status.success(),
        "the kernel could not mount and list the volume:\n{}{}",
        stdout(out),
        stderr(out)
    );
    stdout(out)
        .lines()
        .map(|line| {
            let (name, rest) = line
                .split_once('|')
                .unwrap_or_else(|| panic!("a guest listing line: {line}"));
            (name.to_string(), rest.to_string())
        })
        .collect()
}

fn fresh_volume(tag: &str) -> String {
    let img = sized_image(tag, 32 << 20);
    let made = oracle("mkfs.ext4")
        .args(["-q", "-F", "-b", "4096", "-I", "256", &img])
        .output();
    assert!(made.status.success(), "mkfs.ext4: {}", stderr(&made));
    img
}

#[test]
fn a_symlink_mid_path_is_followed_as_the_kernel_follows_it() {
    let img = fresh_volume("path-symlink-kernel");
    let fill = "cd \"$MNT\"\n\
                mkdir full full/sub\n\
                head -c 5000 /dev/urandom > full/x\n\
                head -c 7000 /dev/urandom > full/y\n\
                ln -s y full/ylink\n\
                ln -s full dirlink\n\
                ln -s dirlink chain\n\
                sync\n";
    let filled = guest_kernel_write(&img, fill);
    assert!(
        filled.status.success(),
        "the kernel could not fill the volume:\n{}{}",
        stdout(&filled),
        stderr(&filled)
    );
    let kernel_img = format!("{img}.kernel");
    std::fs::copy(&img, &kernel_img).expect("copy the filled volume");

    // Each removal goes through at least one link mid-path; the last names
    // a symlink as its final component, which is removed, not followed.
    let removals = [
        ("rm", "/dirlink/x", "unlink dirlink/x"),
        ("rmdir", "/chain/sub", "rmdir chain/sub"),
        ("rm", "/chain/ylink", "unlink chain/ylink"),
    ];
    let mut script = String::from("set -e\ncd \"$MNT\"\n");
    for (_, _, command) in removals {
        script.push_str(command);
        script.push('\n');
    }
    script.push_str("sync\n");
    script.push_str(LISTING);
    let kernel = listing(&guest_kernel_write(&kernel_img, &script));

    for capi in [false, true] {
        let ours_img = format!("{img}.{}", if capi { "c" } else { "rust" });
        std::fs::copy(&img, &ours_img).unwrap();
        if capi {
            let handle = CMount::open(&ours_img, true);
            for (verb, path, _) in removals {
                let path = CString::new(path).unwrap();
                let rc = unsafe {
                    if verb == "rmdir" {
                        fs_ext4_rmdir(handle.0, path.as_ptr())
                    } else {
                        fs_ext4_unlink(handle.0, path.as_ptr())
                    }
                };
                assert_eq!(rc, 0, "C removal errno {}", fs_ext4_last_errno());
            }
        } else {
            let fs = Filesystem::mount(Arc::new(FileDevice::open_rw(&ours_img).unwrap())).unwrap();
            for (verb, path, _) in removals {
                if verb == "rmdir" {
                    fs.apply_rmdir(path).unwrap();
                } else {
                    fs.apply_unlink(path).unwrap();
                }
            }
        }
        assert_e2fsck_clean(&ours_img, "namespace edits through symbolic links");
        let ours = listing(&guest_kernel_write(
            &ours_img,
            &format!("cd \"$MNT\"\n{LISTING}"),
        ));
        assert!(kernel.contains_key("/full/y") && !kernel.contains_key("/full/x"));
        assert_eq!(
            ours, kernel,
            "C ABI={capi}: directory tree differs from Linux"
        );
    }
}

#[test]
fn a_symlink_loop_mid_path_is_refused_as_the_kernel_refuses_it() {
    let img = fresh_volume("path-symlink-loop-kernel");
    // l0 names `full`; each l<i> names l<i-1>, so `l<i>/x` follows i + 1
    // links. The kernel follows at most 40 in one lookup.
    let script = "cd \"$MNT\"\n\
                  mkdir full\n\
                  printf x > full/x\n\
                  ln -s self self\n\
                  ln -s b a\n\
                  ln -s a b\n\
                  ln -s full l0\n\
                  i=1\n\
                  while [ $i -le 40 ]; do ln -s l$((i - 1)) l$i; i=$((i + 1)); done\n\
                  sync\n\
                  printf 'inode|%s\\n' \"$(stat -c %i l39/x)\"\n\
                  for p in self/x a/x l40/x; do\n\
                  \x20   if out=\"$(LC_ALL=C unlink \"$p\" 2>&1)\"; then\n\
                  \x20       printf '%s|NOT REFUSED\\n' \"$p\"\n\
                  \x20   else\n\
                  \x20       printf '%s|%s\\n' \"$p\" \"${out##*: }\"\n\
                  \x20   fi\n\
                  done\n";
    let out = guest_kernel_write(&img, script);
    assert!(
        out.status.success(),
        "the kernel could not fill the volume or try the loops:\n{}{}",
        stdout(&out),
        stderr(&out)
    );
    let kernel = stdout(&out);

    let image = std::fs::read(&img).unwrap();
    let mut lines = kernel.lines();
    let expected: u32 = lines
        .next()
        .unwrap()
        .strip_prefix("inode|")
        .unwrap()
        .parse()
        .unwrap();
    let fs = Filesystem::mount(Arc::new(FileDevice::open(&img).unwrap())).unwrap();
    let handle = CMount::open(&img, false);
    assert_eq!(fs.lookup_path_bytes(b"/l39/x").unwrap(), expected);
    assert_eq!(handle.stat("/l39/x").unwrap().inode, expected);
    let mut tried = Vec::new();
    for line in lines {
        let (path, reason) = line.split_once('|').unwrap();
        assert_eq!(reason, "Too many levels of symbolic links");
        let path = format!("/{path}");
        assert!(matches!(
            fs.lookup_path_bytes(path.as_bytes()),
            Err(Error::SymlinkLoop)
        ));
        assert_eq!(handle.stat(&path).err(), Some(fs_ext4::error::errno::ELOOP));
        assert_eq!(
            std::fs::read(&img).unwrap(),
            image,
            "a read-only loop lookup changed bytes"
        );
        tried.push(path);
    }
    assert_eq!(tried, ["/self/x", "/a/x", "/l40/x"]);
}

#[test]
fn expanded_symlink_targets_keep_linux_casefold_reads_and_write_refusal() {
    let img = sized_image("symlink-casefold-kernel", 64 << 20);
    let made = oracle("mkfs.ext4")
        .args([
            "-q",
            "-F",
            "-b",
            "4096",
            "-I",
            "256",
            "-O",
            "casefold",
            "-E",
            "encoding=utf8-12.1,encoding_flags=strict,lazy_itable_init=0,lazy_journal_init=0",
            &img,
        ])
        .output();
    assert!(made.status.success(), "mkfs: {}", stderr(&made));
    let fill = r#"cd "$MNT"
mkdir Fold
chattr +F Fold
mkdir Fold/Child
printf data > Fold/Child/ReadMe
ln -s /Fold/CHILD Alias
ln -s ../CHILD Fold/Child/back
sync
printf 'inode|%s\n' "$(stat -c %i Alias/back/readme)"
"#;
    let out = guest_kernel_write(&img, fill);
    assert!(
        out.status.success(),
        "kernel casefold setup: {}",
        stderr(&out)
    );
    let expected: u32 = stdout(&out)
        .trim()
        .strip_prefix("inode|")
        .unwrap()
        .parse()
        .unwrap();
    let before = std::fs::read(&img).unwrap();
    let fs = Filesystem::mount(Arc::new(FileDevice::open(&img).unwrap())).unwrap();
    let handle = CMount::open(&img, false);
    for path in ["/Alias/readme", "/Alias/back/README", "/Fold/child/ReadMe"] {
        assert_eq!(fs.lookup_path_bytes(path.as_bytes()).unwrap(), expected);
        assert_eq!(handle.stat(path).unwrap().inode, expected);
    }
    assert!(
        matches!(Filesystem::mount(Arc::new(FileDevice::open_rw(&img).unwrap())),
        Err(Error::UnsupportedIncompat(bits)) if bits & fs_ext4::features::Incompat::CASEFOLD.bits() != 0)
    );
    let path = CString::new(img.clone()).unwrap();
    assert!(unsafe { fs_ext4_mount_rw(path.as_ptr()) }.is_null());
    assert_eq!(std::fs::read(&img).unwrap(), before);
    drop(handle);
    drop(fs);
    assert_e2fsck_clean(
        &img,
        "unchanged Linux casefold image after symlink reads/refusals",
    );
}
