//! fsck counts an uninit group's free blocks and inodes the way the format
//! defines them, not from bitmap bytes the format leaves unspecified (#391).
//!
//! A group flagged `BLOCK_UNINIT` has no stored block bitmap: `mkfs.ext4`
//! does not write it, and every reader rebuilds it from the group's own
//! metadata (backup superblock, descriptor table and reserved blocks, and
//! any bitmaps or inode table that live inside it). A group flagged
//! `INODE_UNINIT` has every inode free. Whatever bytes sit in those bitmap
//! blocks mean nothing.
//!
//! The volume is `mkfs.ext4`'s own, with several groups past the first left
//! uninit. Their block bitmaps are overwritten with zeros and their inode
//! bitmaps with ones, and `e2fsck -fn` is asked first whether that changed
//! anything (it must not). The audit must then find nothing, and a repair
//! pass must write nothing `e2fsck` objects to.

#![cfg(unix)]

use fs_ext4::bgd::BgdFlags;
use fs_ext4::block_io::FileDevice;
use fs_ext4::fs::Filesystem;
use std::sync::Arc;

const BS: u64 = 4096;

fn mount(path: &str) -> Filesystem {
    Filesystem::mount(Arc::new(FileDevice::open_rw(path).unwrap())).unwrap()
}

#[test]
fn fsck_reads_uninit_groups_as_the_format_defines_them() {
    let path = fs_ext4_test_support::temp_path!("fs_ext4_391_{}.img", std::process::id());
    std::fs::File::create(&path)
        .and_then(|f| f.set_len(256 * 1024 * 1024))
        .unwrap();
    let out = fs_ext4_test_support::oracle("mkfs.ext4")
        .args([
            "-q", "-F", "-b", "4096", "-g", "8192", "-N", "4096", "-I", "256", &path,
        ])
        .output();
    assert!(
        out.status.success(),
        "mkfs.ext4: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Scribble over every bitmap the format leaves unspecified.
    let (block_uninit, inode_uninit) = {
        let fs = mount(&path);
        let mut block_uninit = Vec::new();
        let mut inode_uninit = Vec::new();
        for (gi, g) in fs.groups.iter().enumerate() {
            if g.flags().contains(BgdFlags::BLOCK_UNINIT) {
                fs.dev
                    .write_at(g.block_bitmap * BS, &[0u8; BS as usize])
                    .unwrap();
                block_uninit.push(gi);
            }
            if g.flags().contains(BgdFlags::INODE_UNINIT) {
                fs.dev
                    .write_at(g.inode_bitmap * BS, &[0xFFu8; BS as usize])
                    .unwrap();
                inode_uninit.push(gi);
            }
        }
        fs.dev.flush().unwrap();
        (block_uninit, inode_uninit)
    };
    assert!(
        !block_uninit.is_empty() && !inode_uninit.is_empty(),
        "mkfs.ext4 left no uninit group, so this tests nothing \
         (BLOCK_UNINIT {block_uninit:?}, INODE_UNINIT {inode_uninit:?})"
    );
    fs_ext4_test_support::assert_e2fsck_clean(&path, "uninit groups with scribbled bitmaps");

    {
        let fs = mount(&path);
        let report = fs.audit(u32::MAX, u32::MAX).unwrap();
        assert!(
            report.is_clean(),
            "the audit counted an uninit group's bitmap bytes: {report:?}"
        );
        let repaired = fs.audit_repair(u32::MAX, u32::MAX, true).unwrap();
        assert_eq!(
            repaired.repaired_count, 0,
            "the repair pass rewrote a clean volume: {repaired:?}"
        );
    }
    fs_ext4_test_support::assert_e2fsck_clean(&path, "after an audit_repair pass");
    let _ = std::fs::remove_file(&path);
}
