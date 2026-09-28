//! The repair of a wrong `..` in a directory block whose checksum fails
//! agrees with e2fsck's repair of the same image (#344).
//!
//! e2fsprogs makes the volume (`mkfs.ext4 -d`), the test points `/sub`'s
//! `..` at `/other` without restamping the block, and two copies are
//! repaired: one by `e2fsck -fy`, the oracle, and one by this crate's
//! repair pass. e2fsck must see what the audit reports, a block that
//! "passes checks but fails checksum" and a `..` to fix; the block this
//! crate writes must be byte-for-byte the one e2fsck writes; and
//! `e2fsck -fn` must find nothing left on either. Runs in the harness VM.

use fs_ext4::block_io::FileDevice;
use fs_ext4::fs::Filesystem;
use fs_ext4::fsck::{self, Anomaly};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Arc;

fn run(tool: &str, args: &[&str]) -> (Option<i32>, String) {
    let out = fs_ext4_test_support::oracle(tool).args(args).output();
    (
        out.status.code(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

fn mount(path: &str) -> Filesystem {
    let dev = FileDevice::open_rw(path).expect("open rw");
    Filesystem::mount(Arc::new(dev)).expect("mount")
}

fn resolve(fs: &Filesystem, path: &str) -> u32 {
    let mut reader = |ino: u32| fs.read_inode_verified(ino).map(|(inode, _)| inode);
    fs_ext4::path::lookup(fs.dev.as_ref(), &fs.sb, &mut reader, path).expect("resolve")
}

fn read_raw(path: &str, phys: u64, bs: usize) -> Vec<u8> {
    let mut f = std::fs::File::open(path).expect("open raw");
    f.seek(SeekFrom::Start(phys * bs as u64)).expect("seek");
    let mut block = vec![0u8; bs];
    f.read_exact(&mut block).expect("read");
    block
}

fn dotdot_offset(block: &[u8]) -> usize {
    // `.` is the first record; `..` follows it.
    u16::from_le_bytes([block[4], block[5]]) as usize
}

#[test]
fn a_wrong_dotdot_under_a_bad_checksum_is_repaired_as_e2fsck_repairs_it() {
    let root = fs_ext4_test_support::temp_path!("fs_ext4_dircsum_oracle_{}", std::process::id());
    for dir in ["other", "sub"] {
        std::fs::create_dir_all(std::path::Path::new(&root).join(dir)).unwrap();
    }
    std::fs::write(std::path::Path::new(&root).join("sub/file"), b"f").unwrap();
    let image = format!("{root}.img");
    std::fs::File::create(&image)
        .and_then(|f| f.set_len(16 * 1024 * 1024))
        .unwrap();
    let (code, log) = run(
        "mkfs.ext4",
        &[
            "-q",
            "-F",
            "-b",
            "4096",
            "-O",
            "metadata_csum,^metadata_csum_seed",
            "-d",
            &root,
            &image,
        ],
    );
    assert_eq!(code, Some(0), "mkfs.ext4: {log}");
    fs_ext4_test_support::assert_e2fsck_clean(&image, "fresh");

    let (other, sub, phys, bs) = {
        let fs = mount(&image);
        let other = resolve(&fs, "/other");
        let sub = resolve(&fs, "/sub");
        let (inode, _) = fs.read_inode_verified(sub).expect("read sub");
        let phys = fs
            .map_inode_logical(&inode, 0)
            .expect("map")
            .expect("mapped");
        (other, sub, phys, fs.sb.block_size() as usize)
    };
    let block = read_raw(&image, phys, bs);
    let off = dotdot_offset(&block);
    assert_eq!(
        &block[off + 8..off + 10],
        b"..",
        "fixture: second record is .."
    );
    {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .open(&image)
            .unwrap();
        f.seek(SeekFrom::Start(phys * bs as u64 + off as u64))
            .unwrap();
        f.write_all(&other.to_le_bytes()).unwrap();
    }

    let theirs = format!("{root}.e2fsck.img");
    let ours = format!("{root}.ours.img");
    std::fs::copy(&image, &theirs).unwrap();
    std::fs::copy(&image, &ours).unwrap();

    // The oracle: what e2fsck finds, and what it writes.
    let judged = fs_ext4_test_support::oracle("e2fsck")
        .args(["-fy", &theirs])
        .judged();
    let code = judged.output.status.code();
    let log = judged.repaired("e2fsck -fy");
    assert_eq!(code, Some(1), "e2fsck -fy must correct the image: {log}");
    assert!(
        log.contains(&format!("Directory inode {sub}, block #0"))
            && log.contains("passes checks but fails checksum"),
        "e2fsck must report the checksum of /sub's block 0: {log}"
    );
    assert!(
        log.contains(&format!("'..' in /sub ({sub})")),
        "e2fsck must report /sub's `..`: {log}"
    );
    fs_ext4_test_support::assert_e2fsck_clean(&theirs, "e2fsck -fy");

    // This crate's repair of the same image.
    let (report, found) = {
        let fs = mount(&ours);
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
    };
    assert!(
        found.iter().any(|a| matches!(a,
            Anomaly::DirBlockChecksumMismatch { dir_ino, logical_block: 0, htree: false }
                if *dir_ino == sub)),
        "the audit must report what e2fsck reports: {found:#?}"
    );
    assert!(
        found
            .iter()
            .any(|a| matches!(a, Anomaly::WrongDotDot { dir_ino, .. } if *dir_ino == sub)),
        "the audit must report the wrong `..`: {found:#?}"
    );
    assert_eq!(
        report.anomalies_count, 0,
        "left behind: {:#?}",
        report.anomalies
    );
    fs_ext4_test_support::assert_e2fsck_clean(&ours, "repair pass");
    assert!(
        read_raw(&ours, phys, bs) == read_raw(&theirs, phys, bs),
        "/sub's block 0 after this crate's repair differs from e2fsck's"
    );

    for path in [&image, &theirs, &ours] {
        let _ = std::fs::remove_file(path);
    }
    let _ = std::fs::remove_dir_all(&root);
}
