//! Clearing a leftover recovery marker is a write even when the journal is
//! clean. A device becoming writable must not bypass feature write refusal.

use fs_ext4::{
    block_io::BlockDevice,
    features::{FsFlavor, Incompat},
    jbd2, mkfs,
    superblock::{Superblock, SUPERBLOCK_OFFSET},
    Error, Filesystem, Result,
};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};

const IMAGE_BYTES: u64 = 32 * 1024 * 1024;

struct SwitchableDevice {
    bytes: Mutex<Vec<u8>>,
    writable: AtomicBool,
    writes: AtomicUsize,
}

impl BlockDevice for SwitchableDevice {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<()> {
        let bytes = self.bytes.lock().unwrap();
        let start = offset as usize;
        buf.copy_from_slice(&bytes[start..start + buf.len()]);
        Ok(())
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> Result<()> {
        self.writes.fetch_add(1, Ordering::SeqCst);
        if !self.is_writable() {
            return Err(Error::ReadOnly);
        }
        let mut bytes = self.bytes.lock().unwrap();
        let start = offset as usize;
        bytes[start..start + buf.len()].copy_from_slice(buf);
        Ok(())
    }

    fn size_bytes(&self) -> u64 {
        IMAGE_BYTES
    }

    fn is_writable(&self) -> bool {
        self.writable.load(Ordering::SeqCst)
    }

    fn flush(&self) -> Result<()> {
        Ok(())
    }
}

fn volume(feature: u32) -> Arc<SwitchableDevice> {
    let dev = Arc::new(SwitchableDevice {
        bytes: Mutex::new(vec![0; IMAGE_BYTES as usize]),
        writable: AtomicBool::new(true),
        writes: AtomicUsize::new(0),
    });
    mkfs::format_filesystem_with_flavor(
        dev.as_ref(),
        Some("RECOVERY"),
        None,
        IMAGE_BYTES,
        4096,
        FsFlavor::Ext3,
    )
    .unwrap();
    let mut sb = Superblock::read(dev.as_ref()).unwrap();
    let bits = sb.feature_incompat | feature | Incompat::RECOVER.bits();
    sb.raw[0x60..0x64].copy_from_slice(&bits.to_le_bytes());
    if feature & Incompat::CASEFOLD.bits() != 0 {
        sb.raw[0x27c..0x27e].copy_from_slice(&1_u16.to_le_bytes());
        sb.raw[0x27e..0x280].copy_from_slice(&0_u16.to_le_bytes());
    }
    let checksum = fs_ext4::checksum::linux_crc32c(!0, &sb.raw[..0x3fc]);
    sb.raw[0x3fc..0x400].copy_from_slice(&checksum.to_le_bytes());
    dev.write_at(SUPERBLOCK_OFFSET, &sb.raw).unwrap();
    dev.writable.store(false, Ordering::SeqCst);
    dev.writes.store(0, Ordering::SeqCst);
    dev
}

#[test]
fn a_clean_journal_cannot_clear_recovery_on_a_write_protected_volume() {
    for feature in [Incompat::CASEFOLD, Incompat::MMP, Incompat::ENCRYPT] {
        let dev = volume(feature.bits());
        let before = dev.bytes.lock().unwrap().clone();
        let mut fs = Filesystem::mount_lazy(dev.clone()).unwrap();
        assert!(jbd2::read_superblock(&fs).unwrap().unwrap().is_clean());
        assert_ne!(fs.sb.feature_incompat & Incompat::RECOVER.bits(), 0);
        dev.writable.store(true, Ordering::SeqCst);
        let result = fs.replay_journal_if_dirty();
        drop(fs);
        assert_eq!(dev.writes.load(Ordering::SeqCst), 0, "{feature:?}");
        assert!(
            *dev.bytes.lock().unwrap() == before,
            "refused recovery changed the image: {feature:?}"
        );
        assert!(
            matches!(result, Err(Error::UnsupportedIncompat(bits)) if bits == feature.bits()),
            "{feature:?}: {result:?}"
        );
    }
}

#[test]
fn an_ordinary_volume_still_clears_a_leftover_recovery_marker() {
    let dev = volume(0);
    let mut fs = Filesystem::mount_lazy(dev.clone()).unwrap();
    assert!(jbd2::read_superblock(&fs).unwrap().unwrap().is_clean());
    dev.writable.store(true, Ordering::SeqCst);
    assert_eq!(fs.replay_journal_if_dirty().unwrap(), 0);
    drop(fs);
    assert!(dev.writes.load(Ordering::SeqCst) > 0);
    let sb = Superblock::read(dev.as_ref()).unwrap();
    assert_eq!(sb.feature_incompat & Incompat::RECOVER.bits(), 0);
}

#[test]
fn a_device_that_stays_read_only_keeps_the_recovery_marker() {
    let dev = volume(Incompat::CASEFOLD.bits());
    let before = dev.bytes.lock().unwrap().clone();
    let mut fs = Filesystem::mount_lazy(dev.clone()).unwrap();
    assert_eq!(fs.replay_journal_if_dirty().unwrap(), 0);
    drop(fs);
    assert_eq!(dev.writes.load(Ordering::SeqCst), 0);
    assert!(*dev.bytes.lock().unwrap() == before);
}
