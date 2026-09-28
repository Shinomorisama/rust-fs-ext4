//! A pwrite that runs out of space part-way leaves the volume as it was (#389).
//!
//! When a pwrite over a hole needs more than one extent insert and the tree
//! is deeper than the inode, each insert plans against the nodes the one
//! before it rewrote. Those nodes were written straight to the device so the
//! next plan could read them, ahead of the transaction. When a later sub-run
//! then found no space, the transaction was dropped, but the tree on disk
//! already mapped blocks whose bitmap bits, counters and `i_blocks` never
//! landed: the file claimed blocks the bitmap calls free.
//!
//! The volume comes from `mkfs.ext4`, without a journal so the write is one
//! transaction rather than journal-sized chunks, and `e2fsck -fn` must accept
//! it after the failed write. The e2fsprogs tools run in the harness VM; a
//! test fails when it cannot reach them.

use fs_ext4::block_io::FileDevice;
use fs_ext4::fs::Filesystem;
use std::sync::Arc;

const BS: u64 = 4096;

fn mkfs() -> String {
    let path = fs_ext4_test_support::temp_path!("fs_ext4_pwrite_enospc_{}.img", std::process::id());
    std::fs::File::create(&path)
        .and_then(|f| f.set_len(16 * 1024 * 1024))
        .unwrap();
    let out = fs_ext4_test_support::oracle("mkfs.ext4")
        .args(["-q", "-F", "-b", "4096", "-O", "^has_journal"])
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

#[test]
fn a_pwrite_that_runs_out_of_space_leaves_a_clean_volume() {
    let path = mkfs();

    // Six one-block extents, one every other block: a depth-1 tree.
    let ino = {
        let fs = mount(&path);
        let ino = fs.apply_create("/f", 0o644).expect("create");
        for lb in [0u64, 2, 4, 6, 8, 10] {
            fs.apply_pwrite("/f", lb * BS, &[1u8; BS as usize])
                .expect("pwrite");
        }
        let (inode, _) = fs.read_inode_verified(ino).unwrap();
        let depth = u16::from_le_bytes(inode.block[6..8].try_into().unwrap());
        assert!(depth >= 1, "precondition: depth >= 1, got {depth}");
        ino
    };

    // Fill the volume, leaving a few blocks free.
    {
        let fs = mount(&path);
        let mut n = fs.sb.free_blocks_count.saturating_sub(4);
        loop {
            let r = fs.apply_create("/fill", 0o644).and_then(|_| {
                fs.apply_replace_file_content("/fill", &vec![0u8; (n * BS) as usize])
            });
            match r {
                Ok(_) => break,
                Err(_) => {
                    let _ = fs.apply_unlink("/fill");
                    n -= 1;
                }
            }
        }
    }
    fs_ext4_test_support::assert_e2fsck_clean(&path, "e2fsck -fn before the failed pwrite");

    // A write over the hole at block 20 bigger than the space left.
    let left = {
        let fs = mount(&path);
        let left = fs.sb.free_blocks_count;
        let r = fs.apply_pwrite("/f", 20 * BS, &vec![2u8; ((left + 8) * BS) as usize]);
        assert!(r.is_err(), "precondition: ENOSPC ({left} free)");
        left
    };
    fs_ext4_test_support::assert_e2fsck_clean(&path, "e2fsck -fn after the failed pwrite");

    let fs = mount(&path);
    let (inode, _) = fs.read_inode_verified(ino).unwrap();
    assert_eq!(
        fs.map_inode_logical(&inode, 20).unwrap(),
        None,
        "a failed pwrite left logical block 20 mapped ({left} were free)"
    );
    drop(fs);
    let _ = std::fs::remove_file(&path);
}
