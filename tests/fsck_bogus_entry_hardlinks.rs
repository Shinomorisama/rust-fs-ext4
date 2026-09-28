//! Two directory-typed dirents that name one regular file (#325).
//!
//! The audit walk used to mark an inode visited before checking that it
//! was a directory, so the second dirent claiming the same regular file
//! was skipped: one `BogusEntry` instead of two, one repaired, and the
//! post-repair rescan still reporting the other. Repair needed two runs
//! to converge and `initial - repaired != remaining` looked like a
//! repair bug.
//!
//! The fabrication: create `/bogus_a`, hard-link it as `/bogus_b`, then
//! flip both root dirents' file_type byte to Directory and re-seal the
//! root block's checksum tail. e2fsck, run in the harness VM, is the
//! independent judge: it must reject the fabricated image and accept the
//! repaired one.

use fs_ext4::block_io::FileDevice;
use fs_ext4::dir::DirEntryType;
use fs_ext4::extent;
use fs_ext4::features;
use fs_ext4::fs::Filesystem;
use fs_ext4::fsck::{self, Anomaly};
use fs_ext4::path::EXT4_ROOT_INODE;
use std::fs;
use std::io::{Seek, SeekFrom, Write};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

const NAMES: [&[u8]; 2] = [b"bogus_a", b"bogus_b"];

fn mount(path: &str) -> Filesystem {
    let dev = FileDevice::open_rw(path).expect("open rw");
    Filesystem::mount(Arc::new(dev)).expect("mount")
}

fn poke(path: &str, at: u64, bytes: &[u8]) {
    let mut f = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("open raw");
    f.seek(SeekFrom::Start(at)).expect("seek");
    f.write_all(bytes).expect("write");
}

/// (physical block, offset in block, inode) of the dirent `name` in
/// directory `dir_ino`.
fn find_dirent(fs: &Filesystem, dir_ino: u32, name: &[u8]) -> (u64, usize, u32) {
    let (inode, _) = fs.read_inode_verified(dir_ino).expect("read dir inode");
    assert!(
        fs.sb.feature_incompat & features::Incompat::FILETYPE.bits() != 0,
        "the fixture must carry file_type bytes in its dirents"
    );
    let bs = fs.sb.block_size();
    let mut buf = vec![0u8; bs as usize];
    for logical in 0..inode.size.div_ceil(bs as u64) {
        let Some(phys) =
            extent::map_logical(&inode.block, fs.dev.as_ref(), bs, logical).expect("map")
        else {
            continue;
        };
        fs.dev.read_at(phys * bs as u64, &mut buf).expect("read");
        let mut off = 0usize;
        while off + 8 <= buf.len() {
            let ino = u32::from_le_bytes(buf[off..off + 4].try_into().unwrap());
            let rec_len = u16::from_le_bytes(buf[off + 4..off + 6].try_into().unwrap()) as usize;
            if rec_len < 8 || off + rec_len > buf.len() {
                break;
            }
            let name_len = buf[off + 6] as usize;
            if ino != 0 && 8 + name_len <= rec_len && &buf[off + 8..off + 8 + name_len] == name {
                return (phys, off, ino);
            }
            off += rec_len;
        }
    }
    panic!("dirent {name:?} not found in dir {dir_ino}");
}

/// Recompute the dirent-block checksum tail of `phys` so the patched
/// block still verifies.
fn reseal_dir_block(path: &str, fs: &Filesystem, dir_ino: u32, phys: u64) {
    if !fs.csum.enabled {
        return;
    }
    let bs = fs.sb.block_size() as usize;
    let block = fs.read_block(phys).expect("read block");
    if !fs_ext4::dir::has_csum_tail(&block) {
        return;
    }
    let (inode, _) = fs.read_inode_verified(dir_ino).expect("read inode");
    let mut c = fs_ext4::checksum::linux_crc32c(fs.csum.seed, &dir_ino.to_le_bytes());
    c = fs_ext4::checksum::linux_crc32c(c, &inode.generation.to_le_bytes());
    c = fs_ext4::checksum::linux_crc32c(c, &block[..bs - 12]);
    poke(path, phys * bs as u64 + (bs as u64 - 4), &c.to_le_bytes());
}

/// An image whose root holds two dirents typed Directory that both name
/// one regular file. Returns (image path, file inode).
fn two_dir_typed_links_to_one_file() -> (String, u32) {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let src = fs_ext4_test_support::fixture(env!("CARGO_MANIFEST_DIR"), "ext4-basic.img");
    let path = fs_ext4_test_support::temp_path!(
        "fs_ext4_fsck_bogus_hardlinks_{}_{}.img",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    fs::copy(&src, &path).unwrap_or_else(|e| panic!("copy {src} -> {path}: {e}"));

    let file_ino = {
        let fs = mount(&path);
        let ino = fs.apply_create("/bogus_a", 0o644).expect("create");
        fs.apply_link("/bogus_a", "/bogus_b").expect("link");
        ino
    };

    let slots: Vec<_> = {
        let fs = mount(&path);
        NAMES
            .iter()
            .map(|n| find_dirent(&fs, EXT4_ROOT_INODE, n))
            .collect()
    };
    let bs = mount(&path).sb.block_size() as u64;
    for &(phys, off, ino) in &slots {
        assert_eq!(ino, file_ino, "both dirents name the new file");
        poke(
            &path,
            phys * bs + off as u64 + 7,
            &[DirEntryType::Directory as u8],
        );
    }
    {
        let fs = mount(&path);
        let mut blocks: Vec<u64> = slots.iter().map(|s| s.0).collect();
        blocks.dedup();
        for phys in blocks {
            reseal_dir_block(&path, &fs, EXT4_ROOT_INODE, phys);
        }
    }
    (path, file_ino)
}

fn bogus_names(anomalies: &[Anomaly], file_ino: u32) -> Vec<Vec<u8>> {
    let mut names: Vec<Vec<u8>> = anomalies
        .iter()
        .filter_map(|a| match a {
            Anomaly::BogusEntry {
                parent_ino,
                child_ino,
                name,
            } if *parent_ino == EXT4_ROOT_INODE && *child_ino == file_ino => Some(name.clone()),
            _ => None,
        })
        .collect();
    names.sort();
    names
}

#[test]
fn every_dir_typed_link_to_one_file_is_reported_and_one_repair_clears_them() {
    let (path, file_ino) = two_dir_typed_links_to_one_file();
    let expected: Vec<Vec<u8>> = NAMES.iter().map(|n| n.to_vec()).collect();

    let audit = fsck::audit(&mount(&path), u32::MAX, u32::MAX).expect("audit");
    assert_eq!(
        bogus_names(&audit.anomalies, file_ino),
        expected,
        "each dirent typed Directory is its own BogusEntry: {:#?}",
        audit.anomalies
    );

    let repaired = fsck::audit_with_repair(
        &mount(&path),
        u32::MAX,
        u32::MAX,
        |_, _, _| {},
        |_| {},
        true,
    )
    .expect("repair");
    assert_eq!(
        repaired.anomalies_count, 0,
        "one repair pass converges (initial {}, repaired {}): {:#?}",
        repaired.initial_anomalies_count, repaired.repaired_count, repaired.anomalies
    );
    assert_eq!(
        repaired.initial_anomalies_count - repaired.repaired_count,
        repaired.anomalies_count,
        "every initial finding was repaired"
    );

    let after = fsck::audit(&mount(&path), u32::MAX, u32::MAX).expect("re-audit");
    assert!(
        after.anomalies.is_empty(),
        "a fresh audit after one repair is clean: {:#?}",
        after.anomalies
    );

    let _ = fs::remove_file(&path);
}

#[test]
fn e2fsck_rejects_the_fabrication_and_accepts_the_repair() {
    let (path, _) = two_dir_typed_links_to_one_file();

    let before = fs_ext4_test_support::oracle("e2fsck")
        .args(["-fn", &path])
        .judged();
    let code = before.output.status.code();
    let report = before.report();
    before.findings("e2fsck -fn must find the Directory-typed dirents");
    assert_eq!(
        code,
        Some(4),
        "e2fsck -fn must leave the Directory-typed dirents uncorrected:\n{report}"
    );

    let repaired = fsck::audit_with_repair(
        &mount(&path),
        u32::MAX,
        u32::MAX,
        |_, _, _| {},
        |_| {},
        true,
    )
    .expect("repair");
    assert_eq!(repaired.anomalies_count, 0, "{:#?}", repaired.anomalies);

    fs_ext4_test_support::assert_e2fsck_clean(&path, "fsck_bogus_entry_hardlinks");
    let _ = fs::remove_file(&path);
}
