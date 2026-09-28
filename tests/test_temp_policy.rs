//! Where scratch files go, and why there is only one answer.
//!
//! The oracle tools run inside the fs-linux-test-harness VM, which sees
//! this repository at the path the host knows it by and nothing else of
//! the host. An image under `/tmp`, or under `$RUNNER_TEMP` on CI, is a
//! path `e2fsck` cannot open when it is asked to read it. So the scratch
//! root is inside the repository on every machine, and a caller that
//! names one outside it is refused rather than left to fail later with
//! "No such file or directory" in a guest.

use fs_ext4_test_support::{materialize_temp_dir, select_temp_dir};
use std::ffi::OsStr;
use std::path::Path;

#[test]
fn scratch_lives_in_the_repository_by_default() {
    let worktree = Path::new("/worktree");
    assert_eq!(select_temp_dir(None, worktree), worktree.join("tmp"));
    assert_eq!(
        select_temp_dir(Some(OsStr::new("")), worktree),
        worktree.join("tmp"),
        "an empty FS_EXT4_TEST_TMPDIR is no choice at all"
    );
}

#[test]
fn an_explicit_directory_inside_the_repository_is_taken_exactly() {
    let worktree = Path::new("/worktree");
    assert_eq!(
        select_temp_dir(Some(OsStr::new("/worktree/scratch/run-1")), worktree),
        Path::new("/worktree/scratch/run-1")
    );
}

#[test]
#[should_panic(expected = "which is outside")]
fn an_explicit_directory_outside_the_repository_is_refused() {
    select_temp_dir(Some(OsStr::new("/tmp/elsewhere")), Path::new("/worktree"));
}

#[test]
fn every_non_explicit_root_creates_a_unique_child() {
    let base = fs_ext4_test_support::temp_dir()
        .join(format!("fs-ext4-temp-policy-test.{}", std::process::id()));
    let first = materialize_temp_dir(None, &base).expect("first managed child");
    let second = materialize_temp_dir(None, &base).expect("second managed child");

    assert_eq!(first.parent(), Some(base.as_path()));
    assert_eq!(second.parent(), Some(base.as_path()));
    assert_ne!(first, second);

    std::fs::remove_dir_all(&first).expect("remove first managed child");
    std::fs::remove_dir_all(&second).expect("remove second managed child");
    std::fs::remove_dir(&base).expect("remove test base");
}

#[test]
fn explicit_directory_is_preserved_exactly() {
    let exact = fs_ext4_test_support::temp_dir().join(format!(
        "fs-ext4-explicit-policy-test.{}",
        std::process::id()
    ));
    let selected = materialize_temp_dir(Some(OsStr::new("configured")), &exact)
        .expect("create exact directory");

    assert_eq!(selected, exact);
    std::fs::remove_dir(&selected).expect("remove exact directory");
}

/// The name the child runs of this binary select, and the marker its
/// scratch directory is printed after.
const CHILD: &str = "a_test_process_writes_into_its_scratch_directory";
const MARKER: &str = "scratch directory: ";

/// Run as a test on its own, and as the child process the tests below
/// start: take the scratch directory, leave a file in it, and say where
/// it is.
#[test]
fn a_test_process_writes_into_its_scratch_directory() {
    let dir = fs_ext4_test_support::temp_dir();
    std::fs::write(dir.join("left-behind.img"), b"image").expect("write into scratch");
    println!("{MARKER}{}", dir.display());
}

/// Run this binary again, selecting only [`CHILD`], with `env` applied,
/// and return the scratch directory it reported once it has exited.
fn scratch_of_a_child(env: &[(&str, Option<&OsStr>)]) -> std::path::PathBuf {
    let this_test_binary = std::env::current_exe().expect("own binary");
    let mut child = std::process::Command::new(this_test_binary);
    child.args([CHILD, "--exact", "--nocapture", "--test-threads=1"]);
    for (name, value) in env {
        match value {
            Some(value) => child.env(name, value),
            None => child.env_remove(name),
        };
    }
    let output = child.output().expect("run the child test process");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "the child test process failed: {}\n{stdout}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let line = stdout
        .lines()
        .find_map(|line| line.split_once(MARKER).map(|(_, path)| path))
        .unwrap_or_else(|| panic!("the child did not report its scratch directory:\n{stdout}"));
    std::path::PathBuf::from(line)
}

/// A plain `cargo test` -- no `scripts/test.sh`, so no
/// `FS_EXT4_TEST_TMPDIR` -- gives each test process a directory of its
/// own under `tmp/`. That process created it, so that process removes it
/// when it exits; before this, every run left one `fs-ext4-tests.*`
/// behind, and `tmp/` grew by one directory per test binary per run.
#[test]
fn a_process_removes_the_scratch_directory_it_created_when_it_exits() {
    let dir = scratch_of_a_child(&[("FS_EXT4_TEST_TMPDIR", None), ("RFE_KEEP_IMAGES", None)]);
    let name = dir.file_name().unwrap_or_default().to_string_lossy();
    assert!(
        name.starts_with("fs-ext4-tests."),
        "the child did not use a per-process directory: {}",
        dir.display()
    );
    assert!(
        !dir.exists(),
        "the test process exited and left its scratch directory behind: {}",
        dir.display()
    );
}

/// `RFE_KEEP_IMAGES` asks for the images a run wrote to survive it, so
/// a directory with something in it is kept.
#[test]
fn rfe_keep_images_keeps_the_scratch_directory_and_what_is_in_it() {
    let dir = scratch_of_a_child(&[
        ("FS_EXT4_TEST_TMPDIR", None),
        ("RFE_KEEP_IMAGES", Some(OsStr::new("1"))),
    ]);
    let kept = dir.join("left-behind.img").is_file();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        kept,
        "RFE_KEEP_IMAGES was set and the image went: {}",
        dir.display()
    );
}

/// A directory the caller named is the caller's: `scripts/test.sh`
/// supplies one and removes it itself.
#[test]
fn a_directory_the_caller_named_is_not_removed() {
    let exact = fs_ext4_test_support::temp_dir().join(format!(
        "fs-ext4-caller-owned-policy-test.{}",
        std::process::id()
    ));
    let dir = scratch_of_a_child(&[
        ("FS_EXT4_TEST_TMPDIR", Some(exact.as_os_str())),
        ("RFE_KEEP_IMAGES", None),
    ]);
    assert_eq!(dir, exact);
    let kept = exact.join("left-behind.img").is_file();
    let _ = std::fs::remove_dir_all(&exact);
    assert!(
        kept,
        "a caller-supplied scratch directory was emptied: {}",
        exact.display()
    );
}
