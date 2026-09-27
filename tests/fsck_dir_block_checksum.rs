//! fsck verifies directory-block checksums, reports a mismatch, and never
//! restamps a block it did not verify (#344).
//!
//! Three repairs edited directory blocks and recomputed their checksums
//! without checking the old one first, and the audit never looked at a
//! directory checksum at all. A repair that touched a block whose checksum
//! was already wrong replaced it with a valid checksum over whatever the
//! block held, and the damage was never reported: the same laundering the
//! write engine was cured of in #161 and #322.
//!
//! e2fsck is the target. Its pass 2 reports "directory passes checks but
//! fails checksum" for a linear block and "root node fails checksum" for an
//! htree index, each against the directory inode and logical block, and it
//! only rewrites a checksum it has reported. `fsck_dir_block_checksum_oracle`
//! holds the repaired image up against e2fsck's own repair; these tests pin
//! the behaviour without the VM.

use fs_ext4::block_io::{BlockDevice, FileDevice};
use fs_ext4::fs::Filesystem;
use fs_ext4::fsck::{self, Anomaly};
use fs_ext4::mkfs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Arc;

const BLOCK_SIZE: u32 = 4096;
const IMAGE_BYTES: u64 = 8 * 1024 * 1024;

/// A fresh metadata_csum volume made by this crate's mkfs.
fn fresh_volume(tag: &str) -> String {
    let path =
        fs_ext4_test_support::temp_path!("fs_ext4_fsck_dircsum_{tag}_{}.img", std::process::id());
    std::fs::File::create(&path)
        .and_then(|f| f.set_len(IMAGE_BYTES))
        .expect("create image");
    let dev = FileDevice::open_rw(&path).expect("open rw");
    mkfs::format_filesystem(
        &dev as &dyn BlockDevice,
        None,
        None,
        IMAGE_BYTES,
        BLOCK_SIZE,
    )
    .expect("format");
    path
}

/// A copy of a fixture image, to corrupt.
fn fixture_copy(name: &str, tag: &str) -> String {
    let src = fs_ext4_test_support::fixture(env!("CARGO_MANIFEST_DIR"), name);
    let dst =
        fs_ext4_test_support::temp_path!("fs_ext4_fsck_dircsum_{tag}_{}.img", std::process::id());
    std::fs::copy(&src, &dst).unwrap_or_else(|e| panic!("copy {src} -> {dst}: {e}"));
    dst
}

fn mount(path: &str) -> Filesystem {
    let dev = FileDevice::open_rw(path).expect("open rw");
    Filesystem::mount(Arc::new(dev)).expect("mount")
}

fn resolve(fs: &Filesystem, path: &str) -> u32 {
    let mut reader = |ino: u32| fs.read_inode_verified(ino).map(|(inode, _)| inode);
    fs_ext4::path::lookup(fs.dev.as_ref(), &fs.sb, &mut reader, path).expect("resolve")
}

/// Physical block, block size and inode generation of `ino`'s logical
/// block `logical`.
fn dir_block(fs: &Filesystem, ino: u32, logical: u64) -> (u64, usize, u32) {
    let (inode, _) = fs.read_inode_verified(ino).expect("read dir inode");
    let phys = fs
        .map_inode_logical(&inode, logical)
        .expect("map")
        .expect("block is mapped");
    (phys, fs.sb.block_size() as usize, inode.generation)
}

fn read_raw(path: &str, phys: u64, bs: usize) -> Vec<u8> {
    let mut f = std::fs::File::open(path).expect("open raw");
    f.seek(SeekFrom::Start(phys * bs as u64)).expect("seek");
    let mut block = vec![0u8; bs];
    f.read_exact(&mut block).expect("read");
    block
}

/// Write `bytes` at `(phys, off)` behind the filesystem's back.
fn write_raw(path: &str, phys: u64, bs: usize, off: usize, bytes: &[u8]) {
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .expect("open raw");
    f.seek(SeekFrom::Start(phys * bs as u64 + off as u64))
        .expect("seek");
    f.write_all(bytes).expect("write");
}

/// Offset of the record named `name` in a directory block.
fn dirent_offset(block: &[u8], name: &[u8]) -> usize {
    let mut off = 0;
    while off + 8 <= block.len() {
        let rec_len = u16::from_le_bytes([block[off + 4], block[off + 5]]) as usize;
        let name_len = block[off + 6] as usize;
        if rec_len < 8 || off + rec_len > block.len() {
            break;
        }
        if 8 + name_len <= rec_len && &block[off + 8..off + 8 + name_len] == name {
            return off;
        }
        off += rec_len;
    }
    panic!("no record named {:?}", String::from_utf8_lossy(name));
}

fn dirent_inode(block: &[u8], name: &[u8]) -> u32 {
    let off = dirent_offset(block, name);
    u32::from_le_bytes(block[off..off + 4].try_into().unwrap())
}

fn is_checksum_finding(a: &Anomaly, ino: u32, logical: u64, htree: bool) -> bool {
    matches!(a,
        Anomaly::DirBlockChecksumMismatch { dir_ino, logical_block, htree: h }
            if *dir_ino == ino && *logical_block == logical && *h == htree)
}

/// Run the repair pass, returning its report and every finding it streamed
/// before repairing.
fn repair(path: &str) -> (fsck::AuditReport, Vec<Anomaly>) {
    let fs = mount(path);
    let mut found = Vec::new();
    let report = fsck::audit_with_repair(
        &fs,
        u32::MAX,
        u32::MAX,
        |_, _, _| {},
        |a| found.push(a.clone()),
        true,
    )
    .expect("repair");
    (report, found)
}

#[test]
fn audit_reports_a_directory_block_whose_checksum_does_not_match() {
    let path = fresh_volume("audit");
    let sub = mount(&path).apply_mkdir("/sub", 0o755).expect("mkdir");
    let (phys, bs, _) = dir_block(&mount(&path), sub, 0);
    let mut block = read_raw(&path, phys, bs);
    block[bs - 1] ^= 0xFF;
    write_raw(&path, phys, bs, bs - 1, &block[bs - 1..]);

    let report = fsck::audit(&mount(&path), u32::MAX, u32::MAX).expect("audit");
    assert!(
        report
            .anomalies
            .iter()
            .any(|a| is_checksum_finding(a, sub, 0, false)),
        "a directory block whose tail checksum is wrong must be reported against \
         inode {sub}, logical block 0; the audit found {:#?}",
        report.anomalies
    );
    let _ = std::fs::remove_file(&path);
}

/// The issue's case: a `..` that points at the wrong directory, in a block
/// whose checksum no longer matches. e2fsck reports both, fixes both, and
/// leaves nothing behind.
#[test]
fn a_wrong_dotdot_repair_reports_the_checksum_it_restamps() {
    let path = fresh_volume("dotdot");
    let (other, sub) = {
        let fs = mount(&path);
        (
            fs.apply_mkdir("/other", 0o755).expect("mkdir other"),
            fs.apply_mkdir("/sub", 0o755).expect("mkdir sub"),
        )
    };
    let (phys, bs, _) = dir_block(&mount(&path), sub, 0);
    let block = read_raw(&path, phys, bs);
    let off = dirent_offset(&block, b"..");
    // No restamp: the block's checksum no longer matches its contents.
    write_raw(&path, phys, bs, off, &other.to_le_bytes());

    let (report, found) = repair(&path);
    assert!(
        found.iter().any(|a| is_checksum_finding(a, sub, 0, false)),
        "the checksum mismatch in /sub's block 0 must be reported, not restamped \
         silently; the audit found {found:#?}"
    );
    assert!(
        found
            .iter()
            .any(|a| matches!(a, Anomaly::WrongDotDot { dir_ino, .. } if *dir_ino == sub)),
        "the wrong `..` must be reported; the audit found {found:#?}"
    );
    assert_eq!(
        report.anomalies_count, 0,
        "one repair pass must leave the volume clean, as e2fsck -fy does; \
         still found {:#?}",
        report.anomalies
    );

    let fs = mount(&path);
    let after = read_raw(&path, phys, bs);
    assert_eq!(dirent_inode(&after, b".."), 2, "`..` must point at root");
    assert!(
        fs.csum
            .verify_dir_entry_tail(sub, dir_block(&fs, sub, 0).2, &after),
        "the repaired block's checksum must match"
    );
    let clean = fsck::audit(&fs, u32::MAX, u32::MAX).expect("audit");
    assert!(clean.is_clean(), "re-audit: {:#?}", clean.anomalies);
    let _ = std::fs::remove_file(&path);
}

/// A block whose checksum fails and which does not pass the structural
/// checks is not one e2fsck would restamp as it stands, and neither may
/// this: a repair that wants to edit it must leave it alone.
#[test]
fn a_repair_never_restamps_a_block_whose_checksum_it_is_not_repairing() {
    let path = fresh_volume("refuse");
    let (other, sub) = {
        let fs = mount(&path);
        let other = fs.apply_mkdir("/other", 0o755).expect("mkdir other");
        let sub = fs.apply_mkdir("/sub", 0o755).expect("mkdir sub");
        fs.apply_create("/sub/file", 0o644).expect("create");
        (other, sub)
    };
    let (phys, bs, _) = dir_block(&mount(&path), sub, 0);
    let block = read_raw(&path, phys, bs);
    write_raw(
        &path,
        phys,
        bs,
        dirent_offset(&block, b".."),
        &other.to_le_bytes(),
    );
    // A rec_len no record may have: the block no longer passes the checks.
    let file = dirent_offset(&block, b"file");
    write_raw(&path, phys, bs, file + 4, &3u16.to_le_bytes());
    let before = read_raw(&path, phys, bs);

    let (report, found) = repair(&path);
    assert!(
        found.iter().any(|a| is_checksum_finding(a, sub, 0, false)),
        "the checksum mismatch must be reported; the audit found {found:#?}"
    );
    assert_eq!(
        read_raw(&path, phys, bs),
        before,
        "the repair pass rewrote a block whose checksum it had not verified and \
         was not repairing"
    );
    assert!(
        report
            .anomalies
            .iter()
            .any(|a| is_checksum_finding(a, sub, 0, false)),
        "the unrepaired mismatch must still be reported after the pass; got {:#?}",
        report.anomalies
    );
    let _ = std::fs::remove_file(&path);
}

/// The htree root of `/bigdir` in the htree fixture, with its `..` pointed
/// at lost+found. Returns (image, bigdir, lost+found, phys, bs, generation).
fn htree_with_wrong_dotdot(tag: &str) -> (String, u32, u32, u64, usize, u32) {
    let path = fixture_copy("ext4-htree.img", tag);
    let fs = mount(&path);
    let bigdir = resolve(&fs, "/bigdir");
    let lost = resolve(&fs, "/lost+found");
    let (inode, _) = fs.read_inode_verified(bigdir).expect("read bigdir");
    assert_ne!(
        inode.flags & fs_ext4::inode::InodeFlags::INDEX.bits(),
        0,
        "fixture: /bigdir must be indexed"
    );
    let (phys, bs, generation) = dir_block(&fs, bigdir, 0);
    drop(fs);
    let block = read_raw(&path, phys, bs);
    write_raw(
        &path,
        phys,
        bs,
        dirent_offset(&block, b".."),
        &lost.to_le_bytes(),
    );
    (path, bigdir, lost, phys, bs, generation)
}

/// An htree root whose checksum fails is reported as one. Repairing it
/// means rebuilding the index, as e2fsck does, which this pass does not
/// do; so the `..` repair must leave the root alone rather than stamp a
/// fresh checksum over it.
#[test]
fn an_htree_root_that_fails_its_checksum_is_reported_and_left_alone() {
    let (path, bigdir, _lost, phys, bs, _) = htree_with_wrong_dotdot("dx_bad");
    let before = read_raw(&path, phys, bs);

    let (report, found) = repair(&path);
    assert!(
        found
            .iter()
            .any(|a| is_checksum_finding(a, bigdir, 0, true)),
        "the htree root's checksum mismatch must be reported; the audit found {found:#?}"
    );
    assert_eq!(
        read_raw(&path, phys, bs),
        before,
        "the repair pass rewrote an htree root whose checksum it had not verified"
    );
    assert!(
        report
            .anomalies
            .iter()
            .any(|a| is_checksum_finding(a, bigdir, 0, true)),
        "the unrepaired mismatch must still be reported; got {:#?}",
        report.anomalies
    );
    let _ = std::fs::remove_file(&path);
}

/// An htree root that verifies may be edited, and its checksum is the
/// index's `dx_tail`, not a dirent tail.
#[test]
fn a_wrong_dotdot_in_a_verified_htree_root_is_repaired_under_its_dx_checksum() {
    let (path, bigdir, _lost, phys, bs, generation) = htree_with_wrong_dotdot("dx_good");
    {
        let fs = mount(&path);
        let mut block = read_raw(&path, phys, bs);
        assert!(
            fs.csum.patch_dx_tail(bigdir, generation, &mut block, 32),
            "fixture: the htree root must carry a dx_tail"
        );
        write_raw(&path, phys, bs, 0, &block);
    }

    let (_report, found) = repair(&path);
    assert!(
        !found
            .iter()
            .any(|a| is_checksum_finding(a, bigdir, 0, true)),
        "a root whose checksum matches must not be reported; the audit found {found:#?}"
    );
    let after = read_raw(&path, phys, bs);
    assert_eq!(dirent_inode(&after, b".."), 2, "`..` must point at root");
    let fs = mount(&path);
    assert_eq!(
        fs.csum.verify_dx_tail(bigdir, generation, &after, 32),
        Some(true),
        "the repaired root's dx_tail checksum must match"
    );
    let clean = fsck::audit(&fs, u32::MAX, u32::MAX).expect("audit");
    assert!(
        !clean
            .anomalies
            .iter()
            .any(|a| matches!(a, Anomaly::WrongDotDot { .. })),
        "re-audit: {:#?}",
        clean.anomalies
    );
    let _ = std::fs::remove_file(&path);
}
