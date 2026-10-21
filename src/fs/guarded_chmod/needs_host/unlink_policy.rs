//! Reference-only Linux policy: no driver mutation or constructed disk bytes.
use super::setxattr::make;
use super::*;
use std::path::Path;

#[test]
fn linux_records_unlink_policy() {
    for bs in [1024, 4096] {
        for size in [128, 256] {
            for journal in [false, true] {
                reference(bs, size, journal);
            }
        }
    }
}

fn reference(bs: u32, size: u16, journal: bool) {
    let base = fs_ext4_test_support::temp_dir().join(format!("unlink-{bs}-{size}-{journal}"));
    let image = base.with_extension("img");
    make(&image, bs, size, journal);
    let mut identity = None;
    let mut previous_ctime = None;
    for phase in ["input", "surviving", "final"] {
        let command = format!(
            "python3 - \"$MNT\" {phase} <<'UNLINK_POLICY'\n{}\nUNLINK_POLICY\n",
            include_str!("unlink_policy_guest.py")
        );
        let result = guest_kernel_write(image.to_str().unwrap(), &command);
        std::fs::write(
            base.with_extension(format!("{phase}.jsonl")),
            &result.stdout,
        )
        .unwrap();
        std::fs::write(
            base.with_extension(format!("{phase}.stderr.txt")),
            &result.stderr,
        )
        .unwrap();
        assert!(
            result.status.success(),
            "{base:?} {phase}: {}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert_e2fsck_clean(image.to_str().unwrap(), &format!("Linux unlink {phase}"));
        std::fs::copy(&image, base.with_extension(format!("{phase}.img"))).unwrap();
        // Read-only input/shape checks cannot emit or repair an image.
        let fs = Filesystem::mount(Arc::new(FileDevice::open_path(&image).unwrap())).unwrap();
        assert_eq!(fs.sb.block_size(), bs);
        assert_eq!(fs.sb.inode_size, size);
        assert_eq!(fs.csum.enabled, size == 256);
        assert_eq!(
            fs.sb.feature_compat & crate::features::Compat::HAS_JOURNAL.bits() != 0,
            journal
        );
        assert!(!fs.dev.is_writable());
        if phase == "input" {
            let ino = fs.lookup_path_bytes(b"/policy/target").unwrap();
            assert_eq!(fs.lookup_path_bytes(b"/policy/peer").unwrap(), ino);
            let inode = fs.read_inode_verified(ino).unwrap().0;
            assert_eq!(inode.links_count, 2);
            assert_ne!(inode.file_acl, 0, "external attribute block is required");
            identity = Some((ino, inode.generation));
            previous_ctime = Some((inode.ctime, inode.ctime_nsec));
        } else {
            assert!(matches!(
                fs.lookup_path_bytes(b"/policy/target"),
                Err(Error::NotFound)
            ));
            if phase == "surviving" {
                let (ino, generation) = identity.unwrap();
                assert_eq!(fs.lookup_path_bytes(b"/policy/peer").unwrap(), ino);
                let inode = fs.read_inode_verified(ino).unwrap().0;
                assert_eq!(inode.links_count, 1);
                assert_eq!(inode.generation, generation);
                assert!((inode.ctime, inode.ctime_nsec) > previous_ctime.unwrap());
            } else {
                assert!(matches!(
                    fs.lookup_path_bytes(b"/policy/peer"),
                    Err(Error::NotFound)
                ));
            }
        }
        let ino = identity.unwrap().0;
        drop(fs);
        report(&image, &base, phase, ino);
    }
    // The named phase copies contain every required byte; avoid a fourth image.
    std::fs::remove_file(image).unwrap();
}

fn report(image: &Path, base: &Path, phase: &str, ino: u32) {
    for (label, name) in [
        ("parent", "/policy".to_owned()),
        ("target", format!("<{ino}>")),
    ] {
        let out = oracle("debugfs")
            .args(["-R", &format!("stat {name}")])
            .arg(image)
            .env("TZ", "GMT")
            .output();
        std::fs::write(
            base.with_extension(format!("{phase}.{label}.debugfs.txt")),
            &out.stdout,
        )
        .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let text = String::from_utf8(out.stdout).unwrap();
        assert!(text.contains("Inode:"), "{text}");
        assert!(text.contains("ctime:"), "{text}");
    }
}
