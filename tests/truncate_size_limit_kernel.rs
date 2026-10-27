//! Adapted from upstream 3322f7b (#587), exercising Rust and C without newer CLI APIs.
use fs_ext4::{block_io::FileDevice, capi::*, error::errno::EFBIG, Filesystem};
use fs_ext4_test_support::{assert_e2fsck_clean, guest_kernel_write, oracle};
use std::{ffi::CString, sync::Arc};

#[test]
fn sparse_truncate_matches_linux_size_limits_for_all_eight_layouts() {
    for bs in [1024u32, 4096] {
        for extents in [false, true] {
            for huge in [false, true] {
                let tag = format!("{bs}-{extents}-{huge}");
                let image = fs_ext4_test_support::temp_path!("truncate-limit-{tag}.img");
                std::fs::File::create(&image)
                    .unwrap()
                    .set_len(32 << 20)
                    .unwrap();
                let features = format!(
                    "{},{}",
                    if extents {
                        "extent,^64bit"
                    } else {
                        "^extent,^64bit"
                    },
                    if huge { "huge_file" } else { "^huge_file" }
                );
                let made = oracle("mkfs.ext4")
                    .args([
                        "-q",
                        "-F",
                        "-b",
                        &bs.to_string(),
                        "-I",
                        "256",
                        "-O",
                        &features,
                    ])
                    .arg(&image)
                    .output();
                assert!(
                    made.status.success(),
                    "{tag}: {}",
                    String::from_utf8_lossy(&made.stderr)
                );
                // Bisect Linux's accepted length instead of deriving the expected limit from our code.
                let reference = guest_kernel_write(
                    &image,
                    r#"
cd "$MNT"
: > kernel-limit
: > ours
: > capi
lo=0; hi=$((1 << 50))
while [ $((hi-lo)) -gt 1 ]; do
  mid=$((lo+(hi-lo)/2))
  if truncate -s "$mid" kernel-limit 2>/dev/null; then lo=$mid; else hi=$mid; fi
done
truncate -s "$lo" kernel-limit
if message=$(LC_ALL=C truncate -s $((lo+1)) kernel-limit 2>&1); then exit 1; fi
case "$message" in *'File too large'*) ;; *) echo "$message" >&2; exit 1 ;; esac
test "$(stat -c %s kernel-limit)" = "$lo"
echo "kernel-limit=$lo"
"#,
                );
                assert!(
                    reference.status.success(),
                    "{tag}: {}",
                    String::from_utf8_lossy(&reference.stderr)
                );
                std::fs::write(format!("{image}.reference.txt"), &reference.stdout).unwrap();
                let text = String::from_utf8(reference.stdout).unwrap();
                let max: u64 = text
                    .lines()
                    .find_map(|l| l.strip_prefix("kernel-limit="))
                    .expect("kernel boundary")
                    .parse()
                    .unwrap();
                assert_e2fsck_clean(&image, "Linux sparse size boundary");
                {
                    let fs =
                        Filesystem::mount(Arc::new(FileDevice::open_rw(&image).unwrap())).unwrap();
                    let ino = fs.lookup_path_bytes(b"/ours").unwrap();
                    assert_eq!(fs.max_file_size_ino(ino).unwrap(), max, "{tag}");
                    fs.apply_truncate_ino(ino, max).unwrap();
                    let before = std::fs::read(&image).unwrap();
                    assert_eq!(
                        fs.apply_truncate_grow(ino, max + 1).unwrap_err().to_errno(),
                        EFBIG
                    );
                    assert!(
                        std::fs::read(&image).unwrap() == before,
                        "Rust refusal wrote bytes: {tag}"
                    );
                }
                let name = CString::new(image.as_str()).unwrap();
                let path = CString::new("/capi").unwrap();
                unsafe {
                    let fs = fs_ext4_mount_rw(name.as_ptr());
                    assert!(!fs.is_null());
                    assert_eq!(
                        fs_ext4_truncate(fs, path.as_ptr(), max),
                        0,
                        "C accepted boundary: {tag}"
                    );
                    let before = std::fs::read(&image).unwrap();
                    assert_eq!(fs_ext4_truncate(fs, path.as_ptr(), max + 1), -1);
                    assert_eq!(fs_ext4_last_errno(), EFBIG);
                    assert!(
                        std::fs::read(&image).unwrap() == before,
                        "C refusal wrote bytes: {tag}"
                    );
                    fs_ext4_umount(fs);
                }
                assert_e2fsck_clean(&image, "Rust/C sparse growth at Linux size limit");
                let check = guest_kernel_write(&image, &format!("test \"$(stat -c %s \"$MNT/ours\")\" = {max}\ntest \"$(stat -c %s \"$MNT/capi\")\" = {max}\n"));
                assert!(
                    check.status.success(),
                    "Linux readback {tag}: {}",
                    String::from_utf8_lossy(&check.stderr)
                );
            }
        }
    }
}
