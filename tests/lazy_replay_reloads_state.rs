//! A lazy mount's replay leaves the mount planning against the replayed
//! metadata, not the pre-replay copy it read at mount (#376).
//!
//! The journal holds a committed `mkdir` into a group that was still
//! `INODE_UNINIT`: the transaction clears the flag, sets the directory's
//! bit in the inode bitmap and writes its inode. A lazy mount read the
//! descriptors before the replay, so it still saw the flag, synthesised an
//! all-zero bitmap for that group, and handed the directory's own inode out
//! again to the next create inside it. The verdict is e2fsprogs' own.
//!
//! The e2fsprogs tools run in the harness VM; a test fails when it cannot reach them.

#![cfg(unix)]

use fs_ext4::block_io::{BlockDevice, FileDevice};
use fs_ext4::error::Result;
use fs_ext4::Filesystem;
use fs_ext4_test_support::oracle;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// Once armed, keeps writes only until the second flush: a commit's journal
/// blocks (flush one) and its dirty journal superblock (flush two). Then the
/// power goes.
struct CutAfterDirtyJournal {
    inner: Arc<dyn BlockDevice>,
    armed: AtomicBool,
    flushes: AtomicUsize,
}

impl BlockDevice for CutAfterDirtyJournal {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<()> {
        self.inner.read_at(offset, buf)
    }
    fn size_bytes(&self) -> u64 {
        self.inner.size_bytes()
    }
    fn write_at(&self, offset: u64, buf: &[u8]) -> Result<()> {
        if self.armed.load(Ordering::SeqCst) && self.flushes.load(Ordering::SeqCst) >= 2 {
            return Ok(());
        }
        self.inner.write_at(offset, buf)
    }
    fn flush(&self) -> Result<()> {
        if self.armed.load(Ordering::SeqCst) {
            self.flushes.fetch_add(1, Ordering::SeqCst);
        }
        self.inner.flush()
    }
    fn is_writable(&self) -> bool {
        true
    }
}

fn run(tool: &str, args: &[&str]) -> (Option<i32>, String) {
    let out = oracle(tool).args(args).output();
    (
        out.status.code(),
        format!(
            "{tool} {args:?}: {}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

fn lookup(fs: &Filesystem, path: &str) -> Option<u32> {
    let mut reader = |ino: u32| fs.read_inode_verified(ino).map(|(i, _)| i);
    fs_ext4::path::lookup(fs.dev.as_ref(), &fs.sb, &mut reader, path).ok()
}

const INODE_UNINIT: u16 = 0x0001;

/// A fresh 1 KiB-block image, so it has several groups and all but the
/// first start `INODE_UNINIT`, whose journal holds a committed
/// `mkdir /committed` that never reached its final location.
fn image_with_committed_mkdir_into_an_uninit_group() -> String {
    let image =
        fs_ext4_test_support::temp_path!("fs_ext4_lazy_replay_reload_{}.img", std::process::id());
    std::fs::File::create(&image)
        .and_then(|f| f.set_len(64 * 1024 * 1024))
        .unwrap();
    let (code, log) = run("mkfs.ext4", &["-q", "-F", "-b", "1024", &image]);
    assert_eq!(code, Some(0), "{log}");
    // A CSUM_V3 journal, as a kernel mount leaves it.
    let script = format!("{image}.cmds");
    std::fs::write(&script, "jo -c\njc\n").unwrap();
    let (code, log) = run("debugfs", &["-w", "-f", &script, &image]);
    let _ = std::fs::remove_file(&script);
    assert_eq!(code, Some(0), "{log}");
    oracle("e2fsck")
        .args(["-fy", &image])
        .judged()
        .repaired("e2fsck -fy after the debugfs script");

    let dev = Arc::new(CutAfterDirtyJournal {
        inner: Arc::new(FileDevice::open_rw(&image).unwrap()),
        armed: AtomicBool::new(false),
        flushes: AtomicUsize::new(0),
    });
    let fs = Filesystem::mount(dev.clone()).expect("mount rw");
    assert!(
        fs.journal.is_some(),
        "the mkdir must go through the journal"
    );
    // The first write of a mount marks the volume not clean, with a flush of
    // its own (#85). A chmod rather than a mkdir, so it wakes no group.
    fs.apply_chmod("/", 0o755).expect("warm-up chmod");
    dev.armed.store(true, Ordering::SeqCst);
    fs.apply_mkdir("/committed", 0o755).expect("mkdir");
    drop(fs);
    image
}

#[test]
fn a_create_after_a_lazy_replay_does_not_reuse_an_inode_the_replay_allocated() {
    let image = image_with_committed_mkdir_into_an_uninit_group();

    let mut fs =
        Filesystem::mount_lazy(Arc::new(FileDevice::open_rw(&image).unwrap())).expect("lazy");
    assert_eq!(
        lookup(&fs, "/committed"),
        None,
        "the cut came too late: the mkdir reached its final location"
    );
    let flags_at_mount: Vec<u16> = fs.groups.iter().map(|g| g.flags).collect();
    assert!(fs.replay_journal_if_dirty().expect("replay") > 0);

    let dir = lookup(&fs, "/committed").expect("the replay put the directory in place");
    let group = ((dir - 1) / fs.sb.inodes_per_group) as usize;
    assert!(
        flags_at_mount[group] & INODE_UNINIT != 0,
        "fixture: /committed (inode {dir}) went to group {group}, which was not INODE_UNINIT \
         at mount, so the replay cleared no flag: {flags_at_mount:?}"
    );

    let file = fs
        .apply_create("/committed/f", 0o644)
        .expect("create after the replay");
    assert_ne!(
        file, dir,
        "the create was handed the inode the replayed mkdir allocated"
    );
    fs.apply_mkdir("/committed/d", 0o755)
        .expect("mkdir after the replay");
    drop(fs);

    fs_ext4_test_support::assert_e2fsck_clean(&image, "after writes following a lazy replay");
    let _ = std::fs::remove_file(&image);
}
