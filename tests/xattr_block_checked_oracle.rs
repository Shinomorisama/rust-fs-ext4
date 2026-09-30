//! The external xattr block is checked before it is edited (#378).
//!
//! Every writer of the block read it off the device and edited it without
//! checking what it read: a block without the xattr magic was formatted as
//! an empty xattr block, and one whose checksum failed was edited and
//! restamped, blessing the corruption. They now refuse, as the kernel
//! does.
//!
//! What only an independent writer can prove is that the check accepts a
//! GOOD block: a checksum recipe that disagreed with e2fsprogs would refuse
//! every block the kernel or `debugfs` ever wrote. So `debugfs` writes the
//! block here, the driver edits it, and `e2fsck -fn` judges the result.
//! Then one byte of the block is flipped: the driver refuses to edit it,
//! and `e2fsck` agrees that the block is bad. Every e2fsprogs call runs in
//! the harness VM.

use fs_ext4::block_io::{BlockDevice, FileDevice};
use fs_ext4::fs::Filesystem;
use fs_ext4::Error;
use fs_ext4_test_support::oracle;
use std::sync::Arc;

const BLOCK: u64 = 4096;

fn mkfs() -> String {
    let path =
        fs_ext4_test_support::temp_path!("fs_ext4_xattr_block_checked_{}.img", std::process::id());
    std::fs::File::create(&path)
        .and_then(|f| f.set_len(64 * 1024 * 1024))
        .unwrap();
    let out = oracle("mkfs.ext4")
        .args(["-q", "-F", "-b", "4096", "-I", "256", "-O", "metadata_csum"])
        .arg(&path)
        .output();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    path
}

fn mount(image: &str) -> Filesystem {
    Filesystem::mount(Arc::new(FileDevice::open_rw(image).unwrap())).unwrap()
}

fn file_acl(fs: &Filesystem, path: &str) -> u64 {
    let mut r = |i: u32| fs.read_inode_verified(i).map(|(x, _)| x);
    let ino = fs_ext4::path::lookup(fs.dev.as_ref(), &fs.sb, &mut r, path).unwrap();
    fs.read_inode_verified(ino).unwrap().0.file_acl
}

fn get(fs: &Filesystem, path: &str, name: &str) -> Option<Vec<u8>> {
    let mut r = |i: u32| fs.read_inode_verified(i).map(|(x, _)| x);
    let ino = fs_ext4::path::lookup(fs.dev.as_ref(), &fs.sb, &mut r, path).unwrap();
    let (inode, raw) = fs.read_inode_verified(ino).unwrap();
    fs_ext4::xattr::get_resolved(fs, &inode, &raw, name).unwrap()
}

#[test]
fn a_block_debugfs_wrote_is_edited_and_a_damaged_one_is_refused() {
    let image = mkfs();
    let written_by_debugfs = "d".repeat(200);
    oracle("debugfs")
        .args(["-w", "-f", "-"])
        .arg(&image)
        .stdin(format!(
            "write /dev/null f\nea_set /f user.theirs {written_by_debugfs}\n"
        ))
        .judged()
        .clean("debugfs ea_set");

    // e2fsprogs' block, checksum and all, passes the driver's check.
    {
        let fs = mount(&image);
        assert_ne!(file_acl(&fs, "/f"), 0, "fixture: debugfs used the block");
        fs.apply_setxattr("/f", "user.ours", &[b'o'; 200])
            .expect("editing a block e2fsprogs wrote");
        assert_eq!(
            get(&fs, "/f", "user.theirs"),
            Some(written_by_debugfs.clone().into_bytes())
        );
        assert_eq!(get(&fs, "/f", "user.ours"), Some(vec![b'o'; 200]));
    }
    fs_ext4_test_support::assert_e2fsck_clean(&image, "debugfs block edited by the driver");

    // One byte of the value area flipped: the block no longer verifies.
    let block_nr = file_acl(&mount(&image), "/f");
    let dev = FileDevice::open_rw(&image).unwrap();
    let mut block = vec![0u8; BLOCK as usize];
    dev.read_at(block_nr * BLOCK, &mut block).unwrap();
    block[BLOCK as usize - 1] ^= 0xFF;
    dev.write_at(block_nr * BLOCK, &block).unwrap();
    dev.flush().unwrap();
    drop(dev);

    {
        let fs = mount(&image);
        let r = fs.apply_setxattr("/f", "user.more", &[b'm'; 200]);
        assert!(
            matches!(r, Err(Error::BadChecksum { .. })),
            "an edit of a block that fails its checksum must be refused, got {r:?}"
        );
    }
    let mut after = vec![0u8; BLOCK as usize];
    FileDevice::open(&image)
        .unwrap()
        .read_at(block_nr * BLOCK, &mut after)
        .unwrap();
    assert_eq!(after, block, "the refused edit left the block as it was");
    let said = oracle("e2fsck")
        .args(["-fn", &image])
        .judged()
        .findings("damaged xattr block");
    assert!(
        said.to_lowercase().contains("extended attribute"),
        "e2fsck found something, but not in the xattr block:\n{said}"
    );
    let _ = std::fs::remove_file(&image);
}
