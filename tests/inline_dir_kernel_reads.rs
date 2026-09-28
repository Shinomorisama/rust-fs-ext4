//! Inline-data directories MADE BY THE KERNEL are read by this crate's
//! lookup and readdir as the kernel reads them (#427).
//!
//! An inline directory's `i_block` starts with its parent's inode number
//! (the implicit `..`); its entries start at byte 4 and continue in the
//! `system.data` xattr when they outgrow the 56 bytes left. Lookup parsed
//! entries from byte 0, so the parent number was read as a record and
//! every name failed with `bad rec_len`; readdir refused the directory
//! outright. A test over a layout this crate wrote would only prove the
//! reader agrees with the writer, so here the directories are made by the
//! real kernel in the harness VM, and the kernel's own `find -printf %i`
//! is the answer each name must resolve to.

use fs_ext4::block_io::FileDevice;
use fs_ext4::capi::*;
use fs_ext4::fs::Filesystem;
use fs_ext4_test_support::{guest_kernel_write, oracle, temp_path};
use std::collections::BTreeMap;
use std::ffi::{CStr, CString};
use std::sync::Arc;

/// `small` fits in `i_block`; `spill` holds more than its 56 bytes, so the
/// kernel continues it in `system.data`.
const SCRIPT: &str = r#"
mkdir "$MNT/small"
: > "$MNT/small/a"
mkdir "$MNT/small/sub"
mkdir "$MNT/spill"
for n in f001 f002 f003 f004 f005 f006; do : > "$MNT/spill/$n"; done
cd "$MNT"
find small spill -printf '%i\t%p\n'
"#;

/// Path -> inode number, as the kernel reported it.
fn kernel_made_volume() -> (String, BTreeMap<String, u32>) {
    let image = temp_path!("fs_ext4_inline_dir_kernel_{}.img", std::process::id());
    let _ = std::fs::remove_file(&image);
    std::fs::File::create(&image)
        .and_then(|f| f.set_len(32 * 1024 * 1024))
        .unwrap();
    let made = oracle("mkfs.ext4")
        .args(["-q", "-F", "-b", "4096", "-I", "256"])
        .args(["-O", "inline_data,metadata_csum"])
        .arg(&image)
        .output();
    assert!(
        made.status.success(),
        "mkfs.ext4: {}",
        String::from_utf8_lossy(&made.stderr)
    );
    let out = guest_kernel_write(&image, SCRIPT);
    assert!(
        out.status.success(),
        "the kernel could not populate {image}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let inodes = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|line| {
            let (ino, path) = line.split_once('\t').expect("ino<TAB>path");
            (format!("/{path}"), ino.parse().expect("inode number"))
        })
        .collect();
    (image, inodes)
}

#[test]
fn kernel_made_inline_directories_resolve_and_list_as_the_kernel_sees_them() {
    let (image, kernel) = kernel_made_volume();
    let fs = Filesystem::mount(Arc::new(FileDevice::open(&image).unwrap())).unwrap();
    let inode = |ino: u32| fs.read_inode_verified(ino).unwrap().0;
    for dir in ["/small", "/spill"] {
        assert!(
            inode(kernel[dir]).has_inline_data(),
            "{dir}: the kernel made a block directory, so this test proves nothing"
        );
    }
    assert!(
        inode(kernel["/spill"]).size > 60,
        "/spill: the kernel kept it within i_block, so the continuation is untested"
    );

    // Lookup: every name, and the dots, resolve to the kernel's inode.
    let mut reader = |ino: u32| fs.read_inode_verified(ino).map(|(i, _)| i);
    let mut want: Vec<(String, u32)> = kernel.iter().map(|(p, i)| (p.clone(), *i)).collect();
    for dir in ["/small", "/spill"] {
        want.push((format!("{dir}/."), kernel[dir]));
        want.push((format!("{dir}/.."), 2));
    }
    for (path, ino) in &want {
        let got = fs_ext4::path::lookup(fs.dev.as_ref(), &fs.sb, &mut reader, path);
        assert_eq!(got.map_err(|e| format!("{e:?}")), Ok(*ino), "lookup {path}");
    }
    drop(fs);

    // Readdir: exactly the kernel's children, plus `.` and `..`.
    let c_image = CString::new(image.as_str()).unwrap();
    let handle = unsafe { fs_ext4_mount(c_image.as_ptr()) };
    assert!(!handle.is_null(), "fs_ext4_mount {image}");
    for dir in ["/small", "/spill"] {
        let mut expected: Vec<(String, u32)> = kernel
            .iter()
            .filter_map(|(p, i)| {
                let name = p.strip_prefix(dir)?.strip_prefix('/')?;
                (!name.contains('/')).then(|| (name.to_string(), *i))
            })
            .collect();
        expected.push((".".into(), kernel[dir]));
        expected.push(("..".into(), 2));
        expected.sort();
        let c_dir = CString::new(dir).unwrap();
        let mut listed = Vec::new();
        unsafe {
            let it = fs_ext4_dir_open(handle, c_dir.as_ptr());
            assert!(
                !it.is_null(),
                "dir_open {dir}: {}",
                CStr::from_ptr(fs_ext4_last_error()).to_string_lossy()
            );
            loop {
                let e = fs_ext4_dir_next(it);
                if e.is_null() {
                    break;
                }
                let name = CStr::from_ptr((*e).name.as_ptr())
                    .to_string_lossy()
                    .into_owned();
                listed.push((name, (*e).inode));
            }
            fs_ext4_dir_close(it);
        }
        listed.sort();
        assert_eq!(listed, expected, "readdir {dir}");
    }
    unsafe { fs_ext4_umount(handle) };
    let _ = std::fs::remove_file(&image);
}
