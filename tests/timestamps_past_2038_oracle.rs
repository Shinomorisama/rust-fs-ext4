//! Automatic timestamps past 2038, read back by `debugfs`.
//!
//! The driver's own reader decodes the epoch bits with the same
//! understanding of the format the writer used, so it cannot catch a
//! misreading of them. `debugfs stat` is e2fsprogs' reading of the same
//! inode: if it prints the date the clock said, the base and the epoch
//! bits are where the format puts them.
//!
//! Two inode sizes, because they are two behaviours. A 256-byte inode has
//! `*_extra` fields, and a time past 2038 is stored exactly. A 128-byte
//! inode has none, and the kernel clamps to the signed 32-bit range —
//! 2038-01-19 03:14:07 — rather than wrapping to 1901.
//!
//! Every e2fsprogs call runs in the harness VM (`fs_ext4_test_support::oracle`).

use fs_ext4::block_io::{BlockDevice, FileDevice};
use fs_ext4::fs::Filesystem;
use fs_ext4::runtime::Runtime;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 2038-01-19 03:14:18 UTC: ten seconds past what a signed 32-bit base
/// holds, so it survives only if the epoch bits are written.
const PAST_2038: i64 = (1i64 << 31) + 10;

struct PastClock;
impl Runtime for PastClock {
    fn now_unix_seconds(&self) -> u32 {
        PAST_2038 as u32
    }
    fn next_inode_generation(&self) -> u32 {
        1
    }
}

fn run(tool: &str, args: &[&str]) -> (i32, String, String) {
    let out = fs_ext4_test_support::oracle(tool)
        .args(args)
        // debugfs prints dates in UTC only when told to.
        .env("TZ", "GMT")
        .env("E2FSPROGS_FAKE_TIME", "1700000000")
        .output();
    let code = out.status.code().unwrap_or(-1);
    eprintln!("[oracle] {tool} {} -> exit {code}", args.join(" "));
    (
        code,
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// A fresh ext4 image made by `mke2fs` with `inode_size`-byte inodes.
fn image(tag: &str, inode_size: u32) -> PathBuf {
    let dir = fs_ext4_test_support::temp_dir()
        .join(format!("fs_ext4_post2038_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    let image = dir.join("fs.img");
    std::fs::File::create(&image)
        .and_then(|f| f.set_len(32 * 1024 * 1024))
        .expect("size image");
    let size = inode_size.to_string();
    let (code, out, err) = run(
        "mke2fs",
        &[
            "-q",
            "-F",
            "-t",
            "ext4",
            "-b",
            "4096",
            "-I",
            &size,
            "-O",
            "^has_journal,^orphan_file",
            image.to_str().unwrap(),
        ],
    );
    assert_eq!(code, 0, "mke2fs failed: {out}{err}");
    image
}

/// Create `/f` through the driver with the clock at [`PAST_2038`].
fn create_with_past_clock(image: &Path) {
    let dev = FileDevice::open_rw(image.to_str().unwrap()).expect("open_rw");
    let fs =
        Filesystem::mount_with_runtime(Arc::new(dev) as Arc<dyn BlockDevice>, Arc::new(PastClock))
            .expect("mount rw");
    fs.apply_create("/f", 0o644).expect("create");
}

/// `debugfs -R 'stat /f'`'s output.
fn stat_lines(image: &Path) -> String {
    let (code, out, err) = run("debugfs", &["-R", "stat /f", image.to_str().unwrap()]);
    assert_eq!(code, 0, "debugfs stat failed: {err}");
    out
}

fn field_line<'a>(stat: &'a str, field: &str) -> &'a str {
    stat.lines()
        .find(|l| l.trim_start().starts_with(&format!("{field}:")))
        .unwrap_or_else(|| panic!("debugfs stat has no {field}: line\n{stat}"))
}

fn e2fsck_clean(image: &Path) {
    let (code, out, err) = run("e2fsck", &["-fn", image.to_str().unwrap()]);
    assert_eq!(code, 0, "e2fsck -fn is not clean:\n{out}{err}");
}

#[test]
fn debugfs_reads_a_created_files_times_past_2038_as_the_clock_said() {
    let img = image("large", 256);
    create_with_past_clock(&img);
    let stat = stat_lines(&img);
    for field in ["atime", "mtime", "ctime", "crtime"] {
        let line = field_line(&stat, field);
        eprintln!("[oracle] debugfs {line}");
        assert!(
            line.ends_with("-- Tue Jan 19 03:14:18 2038"),
            "debugfs reads {field} as another date:\n{line}"
        );
    }
    e2fsck_clean(&img);
    let _ = std::fs::remove_dir_all(img.parent().unwrap());
}

#[test]
fn debugfs_reads_a_small_inodes_time_past_2038_as_the_clamped_maximum() {
    let img = image("small", 128);
    create_with_past_clock(&img);
    let stat = stat_lines(&img);
    for field in ["atime", "mtime", "ctime"] {
        let line = field_line(&stat, field);
        eprintln!("[oracle] debugfs {line}");
        assert!(
            line.ends_with("0x7fffffff -- Tue Jan 19 03:14:07 2038"),
            "debugfs reads {field} as another date:\n{line}"
        );
    }
    e2fsck_clean(&img);
    let _ = std::fs::remove_dir_all(img.parent().unwrap());
}
