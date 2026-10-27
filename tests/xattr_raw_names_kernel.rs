//! Independent Linux-created names for the core/C-list adaptation of upstream #590.
use fs_ext4::{block_io::FileDevice, capi::*, xattr, Filesystem};
use fs_ext4_test_support::{assert_e2fsck_clean, guest_kernel_write, oracle};
use std::{ffi::CString, sync::Arc};

#[test]
fn linux_raw_attribute_names_preserve_values_inline_content_and_mutation_identity() {
    for bs in [1024, 4096] {
        let image = fs_ext4_test_support::temp_path!("raw-xattrs-{bs}.img");
        std::fs::File::create(&image)
            .unwrap()
            .set_len(32 << 20)
            .unwrap();
        let made = oracle("mkfs.ext4")
            .args([
                "-q",
                "-F",
                "-b",
                &bs.to_string(),
                "-I",
                "256",
                "-O",
                "inline_data",
            ])
            .arg(&image)
            .output();
        assert!(
            made.status.success(),
            "{}",
            String::from_utf8_lossy(&made.stderr)
        );
        let reference = guest_kernel_write(
            &image,
            r#"
python3 - "$MNT" <<'PY'
import os,sys
root=os.fsencode(sys.argv[1])
for name,size in [(b'inline',70),(b'external',4096)]:
    p=root+b'/'+name
    with open(p,'wb') as f: f.write(b'q'*size)
    os.setxattr(p,b'user.raw-\xff',b'first')
    os.setxattr(p,b'user.raw-\xfe',b'second')
    os.setxattr(p,b'user.ok',b'fine')
    if name==b'external': os.setxattr(p,b'user.large',b'z'*700)
    assert os.getxattr(p,b'user.raw-\xff')==b'first'
    assert os.getxattr(p,b'user.raw-\xfe')==b'second'
    print(name.decode(),size)
PY
"#,
        );
        assert!(
            reference.status.success(),
            "{}",
            String::from_utf8_lossy(&reference.stderr)
        );
        std::fs::write(format!("{image}.reference.txt"), &reference.stdout).unwrap();
        for name in ["inline", "external"] {
            let path = format!("/{name}");
            let fs = Filesystem::mount(Arc::new(FileDevice::open_rw(&image).unwrap())).unwrap();
            let ino = fs.lookup_path_bytes(path.as_bytes()).unwrap();
            let (inode, raw) = fs.read_inode_verified(ino).unwrap();
            if name == "inline" {
                assert!(
                    inode.has_inline_data(),
                    "fixture did not create an inline file"
                );
            } else {
                assert_ne!(
                    inode.file_acl, 0,
                    "fixture did not create an external attribute block"
                );
            }
            for (key, value) in [
                (b"user.raw-\xff".as_slice(), b"first".as_slice()),
                (b"user.raw-\xfe", b"second"),
            ] {
                assert_eq!(
                    xattr::get_resolved_raw(&fs, &inode, &raw, key)
                        .unwrap()
                        .as_deref(),
                    Some(value)
                );
            }
            assert_eq!(
                xattr::get_resolved(&fs, &inode, &raw, "user.raw-\u{fffd}").unwrap(),
                None
            );
            let mut content = vec![0; inode.size as usize];
            assert_eq!(fs.read_ino(ino, 0, &mut content).unwrap(), content.len());
            assert!(content.iter().all(|b| *b == b'q'));
            fs.apply_setxattr(&path, "user.ok", b"updated").unwrap();
            fs.apply_removexattr(&path, "user.ok").unwrap();
            drop(fs);
            let image_c = CString::new(image.as_str()).unwrap();
            let path_c = CString::new(path).unwrap();
            unsafe {
                let handle = fs_ext4_mount(image_c.as_ptr());
                assert!(!handle.is_null());
                let required = fs_ext4_listxattr(handle, path_c.as_ptr(), std::ptr::null_mut(), 0);
                assert!(required > 0);
                let mut names = vec![0u8; required as usize];
                assert_eq!(
                    fs_ext4_listxattr(
                        handle,
                        path_c.as_ptr(),
                        names.as_mut_ptr().cast(),
                        names.len()
                    ),
                    required
                );
                let names: Vec<_> = names.split(|b| *b == 0).filter(|n| !n.is_empty()).collect();
                assert!(names.contains(&b"user.raw-\xff".as_slice()));
                assert!(names.contains(&b"user.raw-\xfe".as_slice()));
                assert!(!names.contains(&"user.raw-\u{fffd}".as_bytes()));
                fs_ext4_umount(handle);
            }
        }
        assert_e2fsck_clean(&image, "raw xattrs after guarded Rust metadata writes");
        let check = guest_kernel_write(
            &image,
            r#"
python3 - "$MNT" <<'PY'
import os,sys
root=os.fsencode(sys.argv[1])
for name,size in [(b'inline',70),(b'external',4096)]:
    p=root+b'/'+name
    assert open(p,'rb').read()==b'q'*size
    assert os.getxattr(p,b'user.raw-\xff')==b'first'
    assert os.getxattr(p,b'user.raw-\xfe')==b'second'
    assert b'user.ok' not in [os.fsencode(n) for n in os.listxattr(p)]
PY
"#,
        );
        assert!(
            check.status.success(),
            "Linux retained-byte readback: {}",
            String::from_utf8_lossy(&check.stderr)
        );
    }
}
