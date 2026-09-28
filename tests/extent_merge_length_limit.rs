//! Merging contiguous extents stops at the extent length limit (#387).
//!
//! `ee_len` is 16 bits, and a value above 32768 is what marks an extent
//! uninitialized. So an initialized extent holds at most 32768 blocks and an
//! uninitialized one at most 32767, and the kernel refuses to merge past
//! either. The insert path merged any physically contiguous neighbours with
//! no cap: an initialized run past 32768 read back as an uninitialized one of
//! `len - 32768`, and two uninitialized ones past 32767 overflowed `ee_len`.
//!
//! The volume comes from `mkfs.ext4` with no backup superblocks
//! (`sparse_super2`, `num_backup_sb=0`) and 1 KiB blocks, so the groups past
//! the first are free end to end and adjacent preallocations land physically
//! contiguous — the layout that grows one extent past the limit. `e2fsck -fn`
//! must accept the result. The e2fsprogs tools run in the harness VM; a test
//! fails when it cannot reach them.

use fs_ext4::block_io::FileDevice;
use fs_ext4::fs::Filesystem;
use std::sync::Arc;

const BS: u64 = 1024;
/// Blocks per preallocation: half a 1 KiB-block group, so pieces tile the
/// free groups exactly and each one lands against the last.
const PIECE: u64 = 4096;
/// Ten pieces. The first lands in the tail of group 1, which is not
/// contiguous with the rest; the next eight run on end to end from group 2,
/// 32768 blocks, one past what an uninitialized extent holds.
const PIECES: u64 = 10;

fn mkfs() -> String {
    let path = fs_ext4_test_support::temp_path!("fs_ext4_merge_limit_{}.img", std::process::id());
    std::fs::File::create(&path)
        .and_then(|f| f.set_len(128 * 1024 * 1024))
        .unwrap();
    let out = fs_ext4_test_support::oracle("mkfs.ext4")
        .args(["-q", "-F", "-b", "1024", "-O", "sparse_super2"])
        .args(["-E", "num_backup_sb=0"])
        .arg(&path)
        .output();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    path
}

fn mount(path: &str) -> Filesystem {
    Filesystem::mount(Arc::new(FileDevice::open_rw(path).unwrap())).unwrap()
}

/// Adjacent preallocations totalling past 32767 blocks stay more than one
/// extent, every one of them uninitialized, and the volume stays clean.
#[test]
fn adjacent_preallocations_do_not_merge_past_the_uninit_extent_limit() {
    let path = mkfs();
    let ino = {
        let fs = mount(&path);
        let ino = fs.apply_create("/f", 0o644).expect("create");
        for i in 0..PIECES {
            fs.apply_fallocate_keep_size(ino, i * PIECE * BS, PIECE * BS)
                .unwrap_or_else(|e| panic!("preallocation {i}: {e:?}"));
        }
        ino
    };

    let fs = mount(&path);
    let (inode, _) = fs.read_inode_verified(ino).unwrap();
    let mut extents = Vec::new();
    let mut lb = 0u64;
    while lb < PIECES * PIECE {
        let e = fs_ext4::extent::lookup(&inode.block, fs.dev.as_ref(), BS as u32, lb)
            .unwrap()
            .unwrap_or_else(|| panic!("logical block {lb} is not mapped: {extents:?}"));
        assert!(e.length > 0, "a zero-length extent at {lb}: {e:?}");
        lb = e.logical_block as u64 + e.length as u64;
        extents.push(e);
    }
    assert!(
        extents.iter().all(|e| e.uninitialized),
        "a preallocation reads back initialized: {extents:?}"
    );
    assert!(
        extents.iter().all(|e| e.length <= 32767),
        "an uninitialized extent past 32767 blocks: {extents:?}"
    );
    // The layout this test exists for: two neighbours physically contiguous
    // whose sum passes the limit, so the cap is what kept them apart.
    assert!(
        extents.windows(2).any(|w| {
            w[0].physical_block + w[0].length as u64 == w[1].physical_block
                && w[0].length as u32 + w[1].length as u32 > 32767
        }),
        "no contiguous run reached the limit, so the cap was never tested: {extents:?}"
    );
    drop(fs);
    fs_ext4_test_support::assert_e2fsck_clean(&path, "e2fsck -fn after the preallocations");
    let _ = std::fs::remove_file(&path);
}
