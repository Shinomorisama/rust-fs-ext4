//! Linux-created casefold fixtures establish the initial oracle profile.
//! Published format: https://docs.kernel.org/filesystems/ext4/super.html
//! All oracle tools and mounts go through the existing harness helpers.

use fs_ext4::{block_io::FileDevice, error::Error, fs::Filesystem, inode::InodeFlags};
use fs_ext4_test_support::{assert_e2fsck_clean, guest_kernel_write, oracle, temp_path};
use std::{fs, os::unix::fs::FileExt, sync::Arc};

fn generate(block_size: u32, strict: bool) {
    let label = format!(
        "casefold-{block_size}-{}",
        if strict { "strict" } else { "opaque" }
    );
    let image = temp_path!("{label}.img");
    fs::File::create(&image)
        .unwrap()
        .set_len(64 * 1024 * 1024)
        .unwrap();
    let extended = format!(
        "encoding=utf8-12.1,hash_seed=a1b2c3d4-e5f6-7890-abcd-ef1234567890,lazy_itable_init=0,lazy_journal_init=0{}",
        if strict { ",encoding_flags=strict" } else { "" }
    );
    let out = oracle("mke2fs")
        .args(["-q", "-F", "-t", "ext4", "-b", &block_size.to_string(), "-I", "256", "-m", "0",
            "-U", "e4f1c0de-0000-4000-8000-00000000cf00", "-O",
            "none,has_journal,extent,64bit,dir_index,filetype,sparse_super,large_file,huge_file,metadata_csum,casefold",
            "-E", &extended, &image])
        .output();
    assert!(
        out.status.success(),
        "mke2fs: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let script = |phase| {
        format!(
            "python3 - {phase} <<'CASEFOLD_PY'\n{}\nCASEFOLD_PY\n",
            include_str!("casefold_fixture_guest.py")
        )
    };
    let populated = guest_kernel_write(&image, &script("populate"));
    assert!(
        populated.status.success(),
        "kernel fixture: {}",
        String::from_utf8_lossy(&populated.stderr)
    );
    // A separate mount prevents population's dentry cache from hiding an
    // incorrect on-disk index when the kernel resolves the variant spellings.
    let verified = guest_kernel_write(&image, &script("verify"));
    assert!(
        verified.status.success(),
        "kernel fixture readback: {}",
        String::from_utf8_lossy(&verified.stderr)
    );
    assert_e2fsck_clean(&image, &label);

    let mut sb = [0_u8; 1024];
    fs::File::open(&image)
        .unwrap()
        .read_exact_at(&mut sb, 1024)
        .unwrap();
    assert_eq!(u16::from_le_bytes(sb[56..58].try_into().unwrap()), 0xef53);
    assert_eq!(u16::from_le_bytes(sb[0x27c..0x27e].try_into().unwrap()), 1);
    assert_eq!(
        u16::from_le_bytes(sb[0x27e..0x280].try_into().unwrap()),
        u16::from(strict)
    );
    let incompat = u32::from_le_bytes(sb[0x60..0x64].try_into().unwrap());
    assert_eq!(u32::from_le_bytes(sb[0x5c..0x60].try_into().unwrap()), 0x24);
    assert_eq!(incompat, 0x200c2);
    assert_eq!(
        u32::from_le_bytes(sb[0x64..0x68].try_into().unwrap()),
        0x40b
    );
    assert_eq!(sb[0xfc], 1, "the measured default hash version");
    assert_eq!(u32::from_le_bytes(sb[0x160..0x164].try_into().unwrap()), 1);
    assert_eq!(
        hex::encode(&sb[0xec..0xfc]),
        "a1b2c3d4e5f67890abcdef1234567890"
    );
    assert_eq!(
        1024_u32 << u32::from_le_bytes(sb[24..28].try_into().unwrap()),
        block_size
    );
    {
        let mounted = Filesystem::mount(Arc::new(FileDevice::open(&image).unwrap())).unwrap();
        for (path, folded, indexed) in [
            ("/ordinary", false, false),
            ("/fold_small", true, false),
            ("/fold_indexed", true, true),
        ] {
            let mut read = |ino| mounted.read_inode_verified(ino).map(|(inode, _)| inode);
            let ino =
                fs_ext4::path::lookup(mounted.dev.as_ref(), &mounted.sb, &mut read, path).unwrap();
            let (inode, _) = mounted.read_inode_verified(ino).unwrap();
            assert_eq!(inode.flags & 0x4000_0000 != 0, folded, "{path}");
            assert_eq!(
                inode.flags & InodeFlags::INDEX.bits() != 0,
                indexed,
                "{path}"
            );
        }
    }
    let before = fs_ext4_test_support::sha256_hex(&fs::read(&image).unwrap());
    match Filesystem::mount(Arc::new(FileDevice::open_rw(&image).unwrap())) {
        Err(Error::UnsupportedIncompat(bits)) => assert_eq!(bits, 0x20000),
        Err(other) => panic!("wrong write refusal: {other:?}"),
        Ok(_) => panic!("casefold writes must remain disabled"),
    }
    assert_eq!(
        before,
        fs_ext4_test_support::sha256_hex(&fs::read(&image).unwrap()),
        "refusal changed the image"
    );

    let saved = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tmp/casefold-stage0");
    fs::create_dir_all(&saved).unwrap();
    fs::copy(&image, saved.join(format!("{label}.img"))).unwrap();
    fs::write(saved.join(format!("{label}.json")), &verified.stdout).unwrap();
    fs::write(saved.join(format!("{label}.superblock.bin")), sb).unwrap();
    fs::write(
        saved.join(format!("{label}.sha256")),
        format!("{before}  {label}.img\n"),
    )
    .unwrap();
    println!("[casefold fixture] {label}: Linux namespace, encoding, layout, e2fsck and unchanged write refusal verified");
    measure_behavior(&image, &label, strict, &saved);
}

fn measure_behavior(image: &str, label: &str, strict: bool, saved: &std::path::Path) {
    // Keep the original fixtures and their manifests intact. Only Linux writes
    // this separate experiment image; the production driver's gate stays shut.
    let experiment = temp_path!("{label}-behavior.img");
    fs::copy(image, &experiment).unwrap();
    let mode = if strict { "strict" } else { "opaque" };
    let script = |phase| {
        format!(
            "python3 - {phase} {mode} <<'CASEFOLD_PY'\n{}\nCASEFOLD_PY\n",
            include_str!("casefold_behavior_guest.py")
        )
    };
    let measured = guest_kernel_write(&experiment, &script("measure"));
    // Preserve the actual Linux-written image even if a new probe fails.
    fs::copy(&experiment, saved.join(format!("{label}-behavior.img"))).unwrap();
    fs::write(
        saved.join(format!("{label}-behavior-measure.stderr")),
        &measured.stderr,
    )
    .unwrap();
    assert!(
        measured.status.success(),
        "kernel behavior measurement: {}",
        String::from_utf8_lossy(&measured.stderr)
    );
    let verified = guest_kernel_write(&experiment, &script("verify"));
    fs::copy(&experiment, saved.join(format!("{label}-behavior.img"))).unwrap();
    fs::write(
        saved.join(format!("{label}-behavior.json")),
        &verified.stdout,
    )
    .unwrap();
    assert!(
        verified.status.success(),
        "kernel behavior readback: {}",
        String::from_utf8_lossy(&verified.stderr)
    );
    assert_e2fsck_clean(&experiment, &format!("{label}-behavior"));
    let digest = fs_ext4_test_support::sha256_hex(&fs::read(&experiment).unwrap());
    fs::write(
        saved.join(format!("{label}-behavior.sha256")),
        format!("{digest}  {label}-behavior.img\n"),
    )
    .unwrap();
    println!("[casefold behavior] {label}: 75 filename probes and namespace observations match the pinned reference; remount and e2fsck passed");
}

#[test]
fn linux_casefold_profiles_are_clean_and_remain_write_protected() {
    for block_size in [1024, 4096] {
        for strict in [false, true] {
            generate(block_size, strict);
        }
    }
}
