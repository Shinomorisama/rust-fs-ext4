//! A label `Filesystem::set_volume_label` writes is the one `dumpe2fs`
//! reads, in the primary superblock and in a backup, and `e2fsck -fn` passes
//! the volume afterwards (#447).
//!
//! The volumes are made by `mke2fs` in the harness VM, in every flavour and
//! with more than one block group, so the backups are laid out by the tool
//! rather than by this crate's formatter. A twin of each is relabelled with
//! `tune2fs -L`, and the backup superblock of both is read with
//! `dumpe2fs -o superblock=`: the driver's write matches the tool's, backup
//! included. `tests/volume_label.rs` checks the same write from the bytes.
//!
//! A label of all 16 bytes, set by `tune2fs -L`, is the one the C ABI's
//! `fs_ext4_get_volume_info` reports, byte for byte as `dumpe2fs` reads it
//! (#463).

use fs_ext4::block_io::FileDevice;
use fs_ext4::capi::{
    fs_ext4_get_volume_info, fs_ext4_mount, fs_ext4_umount, fs_ext4_volume_info_t,
};
use fs_ext4::fs::Filesystem;
use fs_ext4_test_support::{assert_e2fsck_clean, oracle, temp_path};
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::sync::Arc;

fn run(tool: &str, args: &[&str], tag: &str) -> String {
    let out = oracle(tool).args(args).output();
    assert!(
        out.status.success(),
        "[{tag}] {tool} {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// `Filesystem volume name:` from `dumpe2fs -h`, of the primary superblock
/// or of the backup at `backup` (a block number, in `block_size` blocks).
fn label(image: &str, backup: Option<(u64, u32)>, tag: &str) -> String {
    let mut args = Vec::new();
    let (sb, bs);
    if let Some((block, block_size)) = backup {
        sb = format!("superblock={block}");
        bs = format!("blocksize={block_size}");
        args.extend(["-o", sb.as_str(), "-o", bs.as_str()]);
    }
    args.extend(["-h", image]);
    let report = run("dumpe2fs", &args, tag);
    report
        .lines()
        .find_map(|l| l.strip_prefix("Filesystem volume name:"))
        .unwrap_or_else(|| panic!("[{tag}] no volume name in dumpe2fs -h:\n{report}"))
        .trim()
        .to_string()
}

fn mke2fs(tag: &str, who: &str, kind: &str, block_size: u32, size: u64) -> String {
    let path = temp_path!("fs_ext4_label_{who}_{tag}_{}.img", std::process::id());
    std::fs::File::create(&path)
        .and_then(|f| f.set_len(size))
        .unwrap_or_else(|e| panic!("create {path}: {e}"));
    run(
        "mke2fs",
        &[
            "-q",
            "-F",
            "-t",
            kind,
            "-b",
            &block_size.to_string(),
            "-L",
            "before",
            &path,
        ],
        tag,
    );
    path
}

fn matches_tune2fs(tag: &str, kind: &str, block_size: u32, size: u64) {
    let ours = mke2fs(tag, "ours", kind, block_size, size);
    let theirs = mke2fs(tag, "tune2fs", kind, block_size, size);

    let group_one = {
        let fs =
            Filesystem::mount(Arc::new(FileDevice::open(&ours).expect("open"))).expect("mount");
        assert!(
            fs.sb.blocks_count > u64::from(fs.sb.blocks_per_group),
            "[{tag}] one group only: no backup to check"
        );
        u64::from(fs.sb.first_data_block) + u64::from(fs.sb.blocks_per_group)
    };
    let backup = Some((group_one, block_size));

    {
        let mut fs = Filesystem::mount(Arc::new(FileDevice::open_rw(&ours).expect("open rw")))
            .expect("mount rw");
        fs.set_volume_label(b"after").expect("set the label");
        fs.finish().expect("finish");
    }
    run("tune2fs", &["-L", "after", &theirs], tag);

    assert_eq!(label(&ours, None, tag), "after", "[{tag}] the primary");
    assert_eq!(
        label(&ours, backup, tag),
        label(&theirs, backup, tag),
        "[{tag}] group 1's backup: ours, then tune2fs's"
    );
    assert_eq!(
        label(&ours, backup, tag),
        "after",
        "[{tag}] group 1's backup"
    );
    assert_e2fsck_clean(&ours, tag);

    let _ = std::fs::remove_file(&ours);
    let _ = std::fs::remove_file(&theirs);
}

#[test]
fn extents_4k_blocks() {
    matches_tune2fs("extents_4k", "ext4", 4096, 320 << 20);
}

#[test]
fn extents_1k_blocks() {
    matches_tune2fs("extents_1k", "ext4", 1024, 32 << 20);
}

#[test]
fn ext3_4k_blocks() {
    matches_tune2fs("ext3_4k", "ext3", 4096, 320 << 20);
}

#[test]
fn ext2_1k_blocks() {
    matches_tune2fs("ext2_1k", "ext2", 1024, 32 << 20);
}

#[test]
fn a_sixteen_byte_label_tune2fs_sets_is_reported_whole() {
    let tag = "sixteen";
    let image = mke2fs(tag, "tune2fs", "ext4", 4096, 32 << 20);
    run("tune2fs", &["-L", "0123456789abcdef", &image], tag);
    assert_eq!(
        label(&image, None, tag),
        "0123456789abcdef",
        "[{tag}] dumpe2fs"
    );

    let path = CString::new(image.as_str()).unwrap();
    let fs = unsafe { fs_ext4_mount(path.as_ptr()) };
    assert!(!fs.is_null(), "[{tag}] mount");
    let mut info = MaybeUninit::<fs_ext4_volume_info_t>::uninit();
    assert_eq!(unsafe { fs_ext4_get_volume_info(fs, info.as_mut_ptr()) }, 0);
    let info = unsafe { info.assume_init() };
    unsafe { fs_ext4_umount(fs) };
    let ours = unsafe { CStr::from_ptr(info.volume_name.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    assert_eq!(
        ours,
        label(&image, None, tag),
        "[{tag}] the C ABI, then dumpe2fs"
    );

    let _ = std::fs::remove_file(&image);
}
