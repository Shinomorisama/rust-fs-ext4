//! A preallocation that needs a fifth extent deepens the tree (#423).
//!
//! An inode's inline extent root holds four extents. `pwrite` promotes the
//! root to an index node when a fifth is needed; `fallocate(KEEP_SIZE)`
//! stopped with `LEAF_FULL_NEEDS_PROMOTION` instead, and a file whose tree
//! was already deeper than the root was refused outright, because the
//! inline-only insert does not descend.
//!
//! The volume is formatted by this crate, the file is given four one-block
//! extents with holes between them, and two preallocations follow: the
//! first needs the root promoted, the second has to descend the tree the
//! first built. `e2fsck -fn` in the harness VM judges the result.

use fs_ext4::block_io::BlockDevice;
use fs_ext4::error::Result;
use fs_ext4::extent;
use fs_ext4::file_io;
use fs_ext4::fs::Filesystem;
use fs_ext4::mkfs;
use std::sync::{Arc, Mutex};

const BLOCK_SIZE: u32 = 4096;
const BS: u64 = BLOCK_SIZE as u64;
const IMAGE_BYTES: u64 = 64 * 1024 * 1024;

struct MemDev {
    bytes: Mutex<Vec<u8>>,
}

impl BlockDevice for MemDev {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<()> {
        let b = self.bytes.lock().unwrap();
        let start = offset as usize;
        buf.copy_from_slice(&b[start..start + buf.len()]);
        Ok(())
    }
    fn write_at(&self, offset: u64, buf: &[u8]) -> Result<()> {
        let mut b = self.bytes.lock().unwrap();
        let start = offset as usize;
        b[start..start + buf.len()].copy_from_slice(buf);
        Ok(())
    }
    fn size_bytes(&self) -> u64 {
        IMAGE_BYTES
    }
    fn is_writable(&self) -> bool {
        true
    }
    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

fn depth(root: &[u8]) -> u16 {
    u16::from_le_bytes([root[6], root[7]])
}

/// Every block of `[first, first + count)` is mapped by an uninitialized
/// extent.
fn assert_preallocated(fs: &Filesystem, root: &[u8], first: u64, count: u64, what: &str) {
    for lb in first..first + count {
        let e = extent::lookup(root, fs.dev.as_ref(), BLOCK_SIZE, lb)
            .expect("lookup")
            .unwrap_or_else(|| panic!("[{what}] logical block {lb} is not mapped"));
        assert!(e.uninitialized, "[{what}] logical block {lb}: {e:?}");
    }
}

#[test]
fn fallocate_promotes_a_full_inline_root_and_descends_a_deep_one() {
    let dev = Arc::new(MemDev {
        bytes: Mutex::new(vec![0u8; IMAGE_BYTES as usize]),
    });
    mkfs::format_filesystem(dev.as_ref(), None, None, IMAGE_BYTES, BLOCK_SIZE).expect("mkfs");
    let fs = Filesystem::mount(dev.clone()).expect("mount");
    let ino = fs.apply_create("/f", 0o644).expect("create");

    // Four one-block extents, a hole after each, so none can merge.
    let data = |i: u64| vec![0x41 + i as u8; BS as usize];
    for i in 0..4u64 {
        fs.apply_pwrite("/f", i * 2 * BS, &data(i)).expect("pwrite");
    }
    let (inode, _) = fs.read_inode_verified(ino).expect("inode");
    assert_eq!(depth(&inode.block), 0, "four extents fit the inline root");
    let (free_before, blocks_before) = (fs.sb.free_blocks_count, inode.blocks);

    // A fifth extent: the root is full and has to become an index node.
    fs.apply_fallocate_keep_size(ino, 16 * BS, 8 * BS)
        .expect("fallocate into a full inline root");
    let (inode, _) = fs.read_inode_verified(ino).expect("inode");
    assert!(depth(&inode.block) >= 1, "the root was promoted");
    assert_preallocated(&fs, &inode.block, 16, 8, "promoted");

    // A sixth, into the tree the first preallocation built.
    fs.apply_fallocate_keep_size(ino, 32 * BS, 8 * BS)
        .expect("fallocate into a tree deeper than the root");
    let (inode, _) = fs.read_inode_verified(ino).expect("inode");
    assert_preallocated(&fs, &inode.block, 16, 8, "first range");
    assert_preallocated(&fs, &inode.block, 32, 8, "second range");

    // KEEP_SIZE leaves the size alone; i_blocks and the free count move by
    // the sixteen data blocks plus the leaf the promotion allocated.
    let sectors = BS / 512;
    assert_eq!(inode.size, 7 * BS, "KEEP_SIZE left i_size where it was");
    let taken = fs.sb.free_blocks_count;
    let fs_used = free_before - taken;
    assert_eq!(fs_used, 17, "sixteen data blocks and one leaf");
    assert_eq!(inode.blocks - blocks_before, fs_used * sectors);

    // The written blocks still read back.
    let mut buf = vec![0u8; (7 * BS) as usize];
    let n = file_io::read(&fs, &inode, 0, buf.len() as u64, &mut buf).expect("read");
    assert_eq!(n, buf.len() as u64);
    for i in 0..4u64 {
        let at = (i * 2 * BS) as usize;
        assert_eq!(&buf[at..at + BS as usize], &data(i)[..], "block {}", i * 2);
    }

    let report = fs_ext4::fsck::audit(&fs, u32::MAX, u32::MAX).expect("audit");
    assert!(report.is_clean(), "audit: {:?}", report.anomalies);
    drop(fs);
    let image = fs_ext4_test_support::temp_path!("fs_ext4_falloc_five_{}.img", std::process::id());
    std::fs::write(&image, &*dev.bytes.lock().unwrap()).unwrap();
    fs_ext4_test_support::assert_e2fsck_clean(&image, "fallocate past four extents");
    let _ = std::fs::remove_file(&image);
}
