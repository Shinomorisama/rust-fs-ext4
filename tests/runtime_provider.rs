use fs_ext4::{block_io::BlockDevice, error::Result, runtime::Runtime, Filesystem};
use std::sync::{Arc, Mutex};
/// In-memory R/W block device backed by a single Vec<u8>.
struct MemDev {
    bytes: Mutex<Vec<u8>>,
    size: u64,
}

impl MemDev {
    fn new(size: u64) -> Arc<Self> {
        Arc::new(Self {
            bytes: Mutex::new(vec![0u8; size as usize]),
            size,
        })
    }
}

impl BlockDevice for MemDev {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<()> {
        let b = self.bytes.lock().unwrap();
        let start = offset as usize;
        let end = start + buf.len();
        assert!(end <= b.len(), "read past EOF");
        buf.copy_from_slice(&b[start..end]);
        Ok(())
    }
    fn size_bytes(&self) -> u64 {
        self.size
    }
    fn write_at(&self, offset: u64, buf: &[u8]) -> Result<()> {
        let mut b = self.bytes.lock().unwrap();
        let start = offset as usize;
        let end = start + buf.len();
        assert!(end <= b.len(), "write past EOF");
        b[start..end].copy_from_slice(buf);
        Ok(())
    }
    fn flush(&self) -> Result<()> {
        Ok(())
    }
    fn is_writable(&self) -> bool {
        true
    }
}

struct Fixed;
impl Runtime for Fixed {
    fn now_unix_seconds(&self) -> u32 {
        1_700_000_123
    }
    fn next_inode_generation(&self) -> u32 {
        0x76543210
    }
}
#[test]
fn caller_runtime_controls_created_inode_metadata() {
    let size = 32 * 1024 * 1024;
    let dev = MemDev::new(size);
    fs_ext4::mkfs::format_filesystem(dev.as_ref(), None, Some([7; 16]), size, 4096).unwrap();
    let fs = Filesystem::mount_with_runtime(dev, Arc::new(Fixed)).unwrap();
    let ino = fs.apply_create("/runtime.txt", 0o600).unwrap();
    let (inode, raw) = fs.read_inode_verified(ino).unwrap();
    assert_eq!(inode.generation, 0x76543210);
    assert_eq!(
        u32::from_le_bytes(raw[0x0c..0x10].try_into().unwrap()),
        1_700_000_123
    );
}

/// A clock the test can move: every automatic timestamp the driver writes
/// comes from here.
struct Clock(std::sync::atomic::AtomicI64);
impl Clock {
    fn at(secs: i64) -> Arc<Self> {
        Arc::new(Self(std::sync::atomic::AtomicI64::new(secs)))
    }
    fn set(&self, secs: i64) {
        self.0.store(secs, std::sync::atomic::Ordering::SeqCst);
    }
}
impl Runtime for Clock {
    fn now_unix_seconds(&self) -> u32 {
        self.0.load(std::sync::atomic::Ordering::SeqCst) as u32
    }
    fn next_inode_generation(&self) -> u32 {
        1
    }
}

/// 2038-01-19 03:14:18 UTC: ten seconds past what a signed 32-bit base
/// holds, so it is stored only if the epoch bits are.
const PAST_2038: i64 = (1i64 << 31) + 10;
/// Later still, so an update is told apart from the value set at creation.
const LATER: i64 = PAST_2038 + 100;

fn formatted(flavor: fs_ext4::features::FsFlavor, clock: Arc<Clock>) -> Filesystem {
    let size = 32 * 1024 * 1024;
    let dev = MemDev::new(size);
    fs_ext4::mkfs::format_filesystem_with_flavor(
        dev.as_ref(),
        None,
        Some([9; 16]),
        size,
        4096,
        flavor,
    )
    .unwrap();
    Filesystem::mount_with_runtime(dev, clock).unwrap()
}

/// Every automatic stamp past 2038 carries its epoch bits: creation sets
/// all four times, a write moves mtime and ctime, a chmod moves ctime.
#[test]
fn automatic_timestamps_past_2038_read_back_as_themselves() {
    let clock = Clock::at(PAST_2038);
    let fs = formatted(fs_ext4::features::FsFlavor::Ext4, clock.clone());
    let file = fs.apply_create("/f", 0o644).unwrap();
    let dir = fs.apply_mkdir("/d", 0o755).unwrap();
    let link = fs.apply_symlink("f", "/l").unwrap();
    for (what, ino) in [("file", file), ("dir", dir), ("symlink", link)] {
        let (inode, _) = fs.read_inode_verified(ino).unwrap();
        assert_eq!(inode.atime, PAST_2038, "{what} atime");
        assert_eq!(inode.mtime, PAST_2038, "{what} mtime");
        assert_eq!(inode.ctime, PAST_2038, "{what} ctime");
        assert_eq!(inode.crtime, PAST_2038, "{what} crtime");
    }

    clock.set(LATER);
    fs.apply_pwrite("/f", 0, b"hello").unwrap();
    let (inode, _) = fs.read_inode_verified(file).unwrap();
    assert_eq!(inode.mtime, LATER, "mtime after a write");
    assert_eq!(inode.ctime, LATER, "ctime after a write");
    assert_eq!(inode.crtime, PAST_2038, "crtime is birth, not change");

    clock.set(LATER + 1);
    fs.apply_chmod("/f", 0o600).unwrap();
    let (inode, _) = fs.read_inode_verified(file).unwrap();
    assert_eq!(inode.ctime, LATER + 1, "ctime after a chmod");
    assert_eq!(inode.mtime, LATER, "chmod leaves mtime alone");
}

/// A 128-byte inode has no `*_extra` fields and so no epoch bits. The
/// kernel clamps such a time to the signed 32-bit range rather than
/// letting it wrap to 1901, and so must we.
#[test]
fn small_inodes_clamp_a_time_past_2038_instead_of_wrapping() {
    let fs = formatted(fs_ext4::features::FsFlavor::Ext2, Clock::at(PAST_2038));
    let ino = fs.apply_create("/f", 0o644).unwrap();
    let (inode, raw) = fs.read_inode_verified(ino).unwrap();
    assert_eq!(raw.len(), 128, "the Ext2 flavor formats 128-byte inodes");
    assert_eq!(inode.atime, i32::MAX as i64, "atime");
    assert_eq!(inode.mtime, i32::MAX as i64, "mtime");
    assert_eq!(inode.ctime, i32::MAX as i64, "ctime");
}
