//! Independent Rust/C unlink emission, including surviving links and shared EAs.
use super::setxattr::make;
use super::*;
use crate::inode::Inode;
use std::path::Path;
use std::sync::atomic::{AtomicI64, Ordering};

struct UnlinkClock(AtomicI64);
impl crate::runtime::Runtime for UnlinkClock {
    fn now_unix_seconds(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
    fn next_inode_generation(&self) -> u32 {
        17
    }
}

#[test]
fn linux_checks_public_unlink_metadata() {
    for capi in [false, true] {
        for bs in [1024, 4096] {
            for size in [128, 256] {
                for journal in [false, true] {
                    profile(capi, bs, size, journal);
                }
            }
        }
    }
}

fn free(image: &Path) -> (u64, u32) {
    let b = std::fs::read(image).unwrap();
    (
        u64::from(u32::from_le_bytes(b[1036..1040].try_into().unwrap()))
            | (u64::from(u32::from_le_bytes(b[1368..1372].try_into().unwrap())) << 32),
        u32::from_le_bytes(b[1040..1044].try_into().unwrap()),
    )
}
fn profile(capi: bool, bs: u32, size: u16, journal: bool) {
    let base = fs_ext4_test_support::temp_dir().join(format!(
        "unlink-emission-{}-{bs}-{size}-{journal}",
        if capi { "capi" } else { "rust" }
    ));
    let image = base.with_extension("img");
    make(&image, bs, size, journal);
    let populated = guest_kernel_write(
        image.to_str().unwrap(),
        r#"
python3 - "$MNT" <<'UNLINK_INPUT'
import os,sys
root=sys.argv[1]; os.mkdir(root+'/p')
for name in ['exclusive','shared','shared-peer']:
    p=root+'/p/'+name
    with open(p,'wb') as f: f.write(name.encode()+b':'+b'x'*16384)
    os.setxattr(p,'user.audit',(b'a' if name=='exclusive' else b'b')*512)
    if name!='shared-peer': os.link(p,p+'-alias')
    os.utime(p,ns=(1600000000000000000,1600000001000000000))
os.utime(root+'/p',ns=(1600000000000000000,1600000001000000000))
os.sync()
UNLINK_INPUT
"#,
    );
    std::fs::write(base.with_extension("input.stdout.txt"), &populated.stdout).unwrap();
    assert!(
        populated.status.success(),
        "{}",
        String::from_utf8_lossy(&populated.stderr)
    );
    assert_e2fsck_clean(image.to_str().unwrap(), "Linux unlink emission input");
    std::fs::copy(&image, base.with_extension("input.img")).unwrap();
    let fs = Filesystem::mount(Arc::new(FileDevice::open_path(&image).unwrap())).unwrap();
    let parent = fs.resolve(b"/p").unwrap();
    let parent_before = fs.read_inode_verified(parent).unwrap().0;
    let targets: Vec<(&str, u32, Inode)> = ["exclusive", "shared"]
        .into_iter()
        .map(|name| {
            let ino = fs.resolve(format!("/p/{name}").as_bytes()).unwrap();
            let inode = fs.read_inode_verified(ino).unwrap().0;
            assert_eq!(inode.links_count, 2);
            assert_ne!(inode.file_acl, 0);
            (name, ino, inode)
        })
        .collect();
    let peer = fs.resolve(b"/p/shared-peer").unwrap();
    let (peer_inode, peer_raw) = fs.read_inode_verified(peer).unwrap();
    assert_eq!(
        peer_inode.file_acl, targets[1].2.file_acl,
        "Linux must create actual shared attribute storage"
    );
    for (index, refs) in [(0, 1u32), (1, 2u32)] {
        let block = fs.read_block(targets[index].2.file_acl).unwrap();
        assert_eq!(u32::from_le_bytes(block[4..8].try_into().unwrap()), refs);
    }
    let first = targets
        .iter()
        .map(|(_, _, i)| i.ctime)
        .chain([parent_before.ctime])
        .max()
        .unwrap()
        + 100;
    assert!(first + 100 < i64::from(i32::MAX));
    drop(fs);
    let initial_free = free(&image);
    let clock = Arc::new(UnlinkClock(AtomicI64::new(first)));
    for (phase, stamp, final_link) in [("surviving", first, false), ("final", first + 100, true)] {
        clock.0.store(stamp, Ordering::SeqCst);
        let fs = Filesystem::mount_with_runtime(
            Arc::new(FileDevice::open_path_rw(&image).unwrap()),
            clock.clone(),
        )
        .unwrap();
        assert_eq!(fs.journal.is_some(), journal);
        let fs = crate::capi::with_test_mount(fs, |handle, fs| {
            let before = std::fs::read(&image).unwrap();
            let long = [b'n'; 256];
            if capi {
                assert_eq!(
                    unsafe {
                        crate::capi::fs_ext4_unlink_at(
                            handle,
                            parent,
                            crate::capi::FS_EXT4_GEN_ANY,
                            long.as_ptr().cast(),
                            long.len(),
                        )
                    },
                    -1
                );
                assert_eq!(
                    crate::capi::fs_ext4_last_errno(),
                    crate::error::errno::ENAMETOOLONG
                );
            } else {
                assert!(matches!(
                    fs.apply_unlink_at(parent, &long),
                    Err(Error::NameTooLong)
                ));
            }
            assert_eq!(std::fs::read(&image).unwrap(), before);
            for (name, ino, original) in &targets {
                let component = if final_link {
                    format!("{name}-alias")
                } else {
                    (*name).to_owned()
                };
                if capi {
                    assert_eq!(
                        unsafe { crate::capi::fs_ext4_unlink(handle, std::ptr::null()) },
                        -1
                    );
                    if final_link {
                        assert_eq!(
                            unsafe {
                                crate::capi::fs_ext4_unlink_at(
                                    handle,
                                    parent,
                                    crate::capi::FS_EXT4_GEN_ANY,
                                    component.as_ptr().cast(),
                                    component.len(),
                                )
                            },
                            0
                        );
                    } else {
                        let path = std::ffi::CString::new(format!("/p/{component}")).unwrap();
                        assert_eq!(
                            unsafe { crate::capi::fs_ext4_unlink(handle, path.as_ptr()) },
                            0
                        );
                    }
                    assert_eq!(crate::capi::fs_ext4_last_errno(), 0);
                } else if final_link {
                    fs.apply_unlink_at(parent, component.as_bytes()).unwrap();
                } else {
                    fs.apply_unlink(&format!("/p/{component}")).unwrap();
                }
                assert!(matches!(
                    fs.resolve(format!("/p/{component}").as_bytes()),
                    Err(Error::NotFound)
                ));
                if !final_link {
                    let after = fs.read_inode_verified(*ino).unwrap().0;
                    assert_eq!(after.links_count, 1);
                    assert_eq!((after.ctime, after.ctime_nsec), (stamp, 0));
                    assert_eq!(
                        (
                            after.atime,
                            after.mtime,
                            after.generation,
                            after.blocks,
                            after.file_acl
                        ),
                        (
                            original.atime,
                            original.mtime,
                            original.generation,
                            original.blocks,
                            original.file_acl
                        )
                    );
                } else {
                    assert!(!fs.inode_bit_is_set(*ino).unwrap());
                }
            }
            let after = fs.read_inode_verified(parent).unwrap().0;
            assert_eq!(
                (after.mtime, after.ctime, after.atime),
                (stamp, stamp, parent_before.atime)
            );
            assert_eq!(fs.read_inode_verified(peer).unwrap().1, peer_raw);
            let expected_blocks = if final_link {
                targets
                    .iter()
                    .map(|(_, _, i)| i.blocks / (u64::from(bs) / 512))
                    .sum::<u64>()
                    - 1
            } else {
                0
            };
            assert_eq!(
                free(&image),
                (
                    initial_free.0 + expected_blocks,
                    initial_free.1 + if final_link { 2 } else { 0 }
                )
            );
            std::fs::copy(&image, base.with_extension(format!("{phase}.dirty.img"))).unwrap();
        });
        fs.finish().unwrap();
        std::fs::copy(&image, base.with_extension(format!("{phase}.img"))).unwrap();
        assert_e2fsck_clean(
            image.to_str().unwrap(),
            &format!("public unlink {phase} emission"),
        );
        let report = oracle("debugfs")
            .args(["-R", "stat /p"])
            .arg(&image)
            .output();
        assert!(report.status.success());
        std::fs::write(
            base.with_extension(format!("{phase}.parent.debugfs.txt")),
            &report.stdout,
        )
        .unwrap();
        let command = format!(
            "python3 - \"$MNT\" {stamp} {} {} {} <<'UNLINK_READBACK'\n{}\nUNLINK_READBACK\n",
            u8::from(final_link),
            parent,
            peer,
            include_str!("unlink_guest.py")
        );
        let checked = guest_kernel_write(image.to_str().unwrap(), &command);
        std::fs::write(
            base.with_extension(format!("{phase}.linux.jsonl")),
            &checked.stdout,
        )
        .unwrap();
        std::fs::write(
            base.with_extension(format!("{phase}.stderr.txt")),
            &checked.stderr,
        )
        .unwrap();
        assert!(
            checked.status.success(),
            "{base:?} {phase}: {}\n{}",
            String::from_utf8_lossy(&checked.stdout),
            String::from_utf8_lossy(&checked.stderr)
        );
        assert_e2fsck_clean(
            image.to_str().unwrap(),
            &format!("Linux unlink {phase} readback"),
        );
    }
    std::fs::remove_file(image).unwrap();
}
