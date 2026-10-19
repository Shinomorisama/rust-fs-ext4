//! Independent Linux/e2fsprogs evidence for the existing public removal API.
use super::super::timestamp_report::debugfs_time;
use super::*;
use crate::inode::Inode;
use std::path::Path;

const REMOVE: &str = "user.remove";
const STAMP: i64 = 1_700_000_100;
struct RemovalClock;
impl crate::runtime::Runtime for RemovalClock {
    fn now_unix_seconds(&self) -> i64 {
        STAMP
    }
    fn next_inode_generation(&self) -> u32 {
        17
    }
}
#[derive(Clone, Copy, Debug)]
enum Branch {
    Inline,
    Rewrite,
    Free,
    Cow,
    Release,
    Duplicate,
    ZeroExtra,
    Peer,
}
struct Target {
    path: &'static str,
    branch: Branch,
    ino: u32,
    before: Inode,
    raw: Vec<u8>,
    attrs: Vec<(&'static str, u8, usize)>,
}

#[test]
fn linux_checks_public_removexattr_metadata() {
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
fn mount(image: &Path) -> Filesystem {
    Filesystem::mount_with_runtime(
        Arc::new(FileDevice::open_rw(image.to_str().unwrap()).unwrap()),
        Arc::new(RemovalClock),
    )
    .unwrap()
}
fn lookup(fs: &Filesystem, path: &str) -> (u32, Inode) {
    let ino = fs.lookup_path_bytes(path.as_bytes()).unwrap();
    (ino, fs.read_inode_verified(ino).unwrap().0)
}
fn share(fs: &Filesystem, source: &str, peer: &str) {
    let (_, source) = lookup(fs, source);
    assert_ne!(source.file_acl, 0);
    let (ino, inode) = lookup(fs, peer);
    assert_eq!(inode.file_acl, 0);
    fs.refuse_write().unwrap();
    let mut block = fs.read_block(source.file_acl).unwrap();
    assert_eq!(u32::from_le_bytes(block[4..8].try_into().unwrap()), 1);
    block[4..8].copy_from_slice(&2u32.to_le_bytes());
    fs.csum.patch_xattr_block(source.file_acl, &mut block);
    fs.dev
        .write_at(source.file_acl * u64::from(fs.sb.block_size()), &block)
        .unwrap();
    let (_, mut raw) = fs.read_inode_verified(ino).unwrap();
    Filesystem::write_file_acl(&mut raw, source.file_acl).unwrap();
    Filesystem::patch_inode_size_and_blocks(
        &mut raw,
        inode.size,
        inode.blocks + u64::from(fs.sb.block_size()) / 512,
    )
    .unwrap();
    fs.finalize_inode_raw(ino, inode.generation, &mut raw)
        .unwrap();
    fs.write_inode_raw(ino, &raw).unwrap();
}
fn free_blocks(image: &Path) -> u64 {
    let b = std::fs::read(image).unwrap();
    u64::from(u32::from_le_bytes(b[1036..1040].try_into().unwrap()))
        | (u64::from(u32::from_le_bytes(b[1368..1372].try_into().unwrap())) << 32)
}
fn allocated(fs: &Filesystem, block: u64) -> bool {
    let relative = block - u64::from(fs.sb.first_data_block);
    let group = (relative / u64::from(fs.sb.blocks_per_group)) as usize;
    let bit = relative % u64::from(fs.sb.blocks_per_group);
    let bitmap = fs
        .read_block(fs.allocation_groups()[group].block_bitmap)
        .unwrap();
    bitmap[bit as usize / 8] & (1 << (bit % 8)) != 0
}
fn refs(fs: &Filesystem, block: u64) -> u32 {
    let b = fs.read_block(block).unwrap();
    assert!(fs.csum.verify_xattr_block(block, &b));
    u32::from_le_bytes(b[4..8].try_into().unwrap())
}
fn report(image: &Path, base: &Path, phase: &str, target: &Target, inode: &Inode) {
    let out = oracle("debugfs")
        .args(["-R", &format!("stat {}", target.path)])
        .arg(image)
        .env("TZ", "GMT")
        .output();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    std::fs::write(
        base.with_extension(format!("{phase}.{}.debugfs.txt", &target.path[1..])),
        &text,
    )
    .unwrap();
    let field = |label: &str| {
        text.split(label)
            .nth(1)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
    };
    assert_eq!(field("Inode:").parse::<u32>().unwrap(), target.ino);
    assert_eq!(
        field("Generation:").parse::<u32>().unwrap(),
        inode.generation
    );
    assert_eq!(
        u16::from_str_radix(field("Mode:"), 8).unwrap(),
        inode.mode & 0xfff
    );
    assert_eq!(
        super::super::owner_report::owner_id(field("User:")).unwrap(),
        inode.uid
    );
    assert_eq!(
        super::super::owner_report::owner_id(field("Group:")).unwrap(),
        inode.gid
    );
    assert_eq!(field("File ACL:").parse::<u64>().unwrap(), inode.file_acl);
    assert_eq!(field("Blockcount:").parse::<u64>().unwrap(), inode.blocks);
    assert_eq!(field("Size:").parse::<u64>().unwrap(), inode.size);
    assert_eq!(
        u32::from_str_radix(field("Flags:").trim_start_matches("0x"), 16).unwrap(),
        inode.flags
    );
    for (name, value) in [
        ("atime", (inode.atime, inode.atime_nsec)),
        ("mtime", (inode.mtime, inode.mtime_nsec)),
        ("ctime", (inode.ctime, inode.ctime_nsec)),
    ] {
        assert_eq!(debugfs_time(&text, name), value);
    }
    let out = oracle("debugfs")
        .args(["-R", &format!("ea_list {}", target.path)])
        .arg(image)
        .output();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    std::fs::write(
        base.with_extension(format!("{phase}.{}.xattrs.txt", &target.path[1..])),
        &text,
    )
    .unwrap();
    let count = text.lines().filter(|line| line.contains(REMOVE)).count();
    assert_eq!(count, usize::from(matches!(target.branch, Branch::Peer)));
    for (name, _, length) in &target.attrs {
        assert!(
            text.lines()
                .any(|line| line.contains(name) && line.contains(&format!("({length})"))),
            "{text}"
        );
    }
}
fn profile(capi: bool, bs: u32, size: u16, journal: bool) {
    let base = fs_ext4_test_support::temp_dir().join(format!(
        "{}-{bs}-{size}-{journal}",
        if capi { "capi" } else { "rust" }
    ));
    let image = base.with_extension("img");
    let image_str = image.to_str().unwrap();
    std::fs::File::create(&image)
        .unwrap()
        .set_len(32 * 1024 * 1024)
        .unwrap();
    let mut features = if size == 128 {
        "none,filetype,ext_attr"
    } else {
        "none,extent,64bit,filetype,metadata_csum,ext_attr"
    }
    .to_owned();
    if journal {
        features.push_str(",has_journal");
    }
    let made = oracle("mke2fs")
        .args([
            "-q",
            "-F",
            "-t",
            "ext4",
            "-b",
            &bs.to_string(),
            "-I",
            &size.to_string(),
            "-m",
            "0",
            "-O",
            &features,
            "-E",
            "lazy_itable_init=0,lazy_journal_init=0",
        ])
        .arg(&image)
        .output();
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
    // Peers start without attributes. The explicit sharing fixture follows the
    // existing shared_xattr_block test and must pass independent e2fsck first.
    let made = guest_kernel_write(
        image_str,
        r#"
python3 - "$MNT" <<'REMOVE_INPUT_PY'
import os,sys
root=sys.argv[1]
for name in ['inline','rewrite','free','cow','cow-peer','release','release-peer','duplicate','zero']:
    path=root+'/'+name
    with open(path,'wb') as f: f.write(('data:'+name).encode())
    os.chmod(path,0o640)
    os.chown(path,1001,1002)
    os.utime(path,ns=(1600000000000000000,1600000001000000000))
for name,value in [('inline',b'i'),('rewrite',b'r'*400),('free',b'f'*400),('cow',b'c'*400),('release',b's'*400),('duplicate',b'd'*400),('zero',b'z'*400)]:
    os.setxattr(root+'/'+name,'user.remove',value)
for name in ['rewrite','cow']: os.setxattr(root+'/'+name,'user.keep',b'k'*200)
REMOVE_INPUT_PY
"#,
    );
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
    let fs = mount(&image);
    assert_eq!(fs.journal.is_some(), journal);
    share(&fs, "/cow", "/cow-peer");
    share(&fs, "/release", "/release-peer");
    if size == 256 {
        let (ino, inode) = lookup(&fs, "/zero");
        assert_ne!(inode.file_acl, 0);
        let (_, mut raw) = fs.read_inode_verified(ino).unwrap();
        super::super::zero_extra_fixture::prepare(&mut raw).unwrap();
        fs.finalize_inode_raw(ino, inode.generation, &mut raw)
            .unwrap();
        fs.write_inode_raw(ino, &raw).unwrap();
    }
    fs.finish().unwrap();
    assert_e2fsck_clean(
        image_str,
        "shared removal input before legacy duplicate seed",
    );
    std::fs::copy(&image, base.with_extension("input.img")).unwrap();
    let fs = mount(&image);
    if size == 256 {
        let (ino, inode) = lookup(&fs, "/duplicate");
        let (_, mut raw) = fs.read_inode_verified(ino).unwrap();
        let start = 128 + u16::from_le_bytes(raw[128..130].try_into().unwrap()) as usize;
        crate::xattr::plan_set_in_inode_region(&mut raw[start..], REMOVE, b"old").unwrap();
        fs.finalize_inode_raw(ino, inode.generation, &mut raw)
            .unwrap();
        fs.write_inode_raw(ino, &raw).unwrap();
    }
    fs.finish().unwrap();
    std::fs::copy(&image, base.with_extension("prepared.img")).unwrap();
    let fs = mount(&image);
    let specifications = [
        ("/inline", Branch::Inline),
        ("/rewrite", Branch::Rewrite),
        ("/free", Branch::Free),
        ("/cow", Branch::Cow),
        ("/release", Branch::Release),
        ("/duplicate", Branch::Duplicate),
        ("/zero", Branch::ZeroExtra),
        ("/cow-peer", Branch::Peer),
        ("/release-peer", Branch::Peer),
    ];
    let mut targets: Vec<_> = specifications
        .into_iter()
        .map(|(path, mut branch)| {
            let (ino, before) = lookup(&fs, path);
            // The explicit legacy matrix has no inline area or duplicate seed.
            if size == 128
                && matches!(
                    branch,
                    Branch::Inline | Branch::Duplicate | Branch::ZeroExtra
                )
            {
                branch = Branch::Free;
            }
            let attrs = match path {
                "/rewrite" | "/cow" => vec![("user.keep", b'k', 200)],
                "/cow-peer" => vec![(REMOVE, b'c', 400), ("user.keep", b'k', 200)],
                "/release-peer" => vec![(REMOVE, b's', 400)],
                _ => vec![],
            };
            Target {
                path,
                branch,
                ino,
                before,
                raw: fs.read_inode_verified(ino).unwrap().1,
                attrs,
            }
        })
        .collect();
    for t in &targets {
        match t.branch {
            Branch::Inline => assert_eq!(t.before.file_acl, 0),
            _ => assert_ne!(t.before.file_acl, 0),
        }
    }
    assert_eq!(targets[3].before.file_acl, targets[7].before.file_acl);
    assert_eq!(targets[4].before.file_acl, targets[8].before.file_acl);
    assert_eq!(refs(&fs, targets[3].before.file_acl), 2);
    assert_eq!(refs(&fs, targets[4].before.file_acl), 2);
    if size == 256 {
        let (_, raw) = fs.read_inode_verified(targets[6].ino).unwrap();
        assert_eq!(&raw[128..130], &[0, 0]);
    }
    let before = std::fs::read(&image).unwrap();
    let mut observations = String::new();
    let fs = crate::capi::with_test_mount(fs, |handle, fs| {
        let held = fs.chmod_operation().unwrap();
        if capi {
            let path = std::ffi::CString::new("/free").unwrap();
            let name = std::ffi::CString::new(REMOVE).unwrap();
            assert_eq!(
                unsafe { crate::capi::fs_ext4_removexattr(handle, path.as_ptr(), name.as_ptr()) },
                -1
            );
            assert_eq!(
                crate::capi::fs_ext4_last_errno(),
                crate::error::errno::ENOTSUP
            );
        } else {
            assert!(matches!(
                fs.apply_removexattr("/free", REMOVE),
                Err(Error::Unsupported(_))
            ));
        }
        drop(held);
        assert!(std::fs::read(&image).unwrap() == before);
        for t in targets.iter().filter(|t| !matches!(t.branch, Branch::Peer)) {
            let free_before = free_blocks(&image);
            if t.before.file_acl != 0 {
                assert!(allocated(fs, t.before.file_acl));
            }
            if capi {
                let path = std::ffi::CString::new(t.path).unwrap();
                let name = std::ffi::CString::new(REMOVE).unwrap();
                assert_eq!(
                    unsafe {
                        crate::capi::fs_ext4_removexattr(handle, path.as_ptr(), name.as_ptr())
                    },
                    0
                );
                assert_eq!(crate::capi::fs_ext4_last_errno(), 0);
            } else {
                fs.apply_removexattr(t.path, REMOVE).unwrap();
            }
            let after = fs.read_inode_verified(t.ino).unwrap().0;
            let (delta, sectors) = match t.branch {
                Branch::Inline => {
                    assert_eq!(after.file_acl, 0);
                    (0, 0)
                }
                Branch::Rewrite => {
                    assert_eq!(after.file_acl, t.before.file_acl);
                    assert_eq!(refs(fs, after.file_acl), 1);
                    (0, 0)
                }
                Branch::Cow => {
                    assert_ne!(after.file_acl, 0);
                    assert_ne!(after.file_acl, t.before.file_acl);
                    assert!(allocated(fs, after.file_acl));
                    assert_eq!(refs(fs, after.file_acl), 1);
                    assert_eq!(refs(fs, t.before.file_acl), 1);
                    (-1, 0)
                }
                Branch::Release => {
                    assert_eq!(after.file_acl, 0);
                    assert!(allocated(fs, t.before.file_acl));
                    assert_eq!(refs(fs, t.before.file_acl), 1);
                    (0, u64::from(bs) / 512)
                }
                Branch::Free | Branch::Duplicate | Branch::ZeroExtra => {
                    assert_eq!(after.file_acl, 0);
                    assert!(!allocated(fs, t.before.file_acl));
                    (1, u64::from(bs) / 512)
                }
                Branch::Peer => unreachable!(),
            };
            assert_eq!(free_blocks(&image) as i64, free_before as i64 + delta);
            assert_eq!(after.blocks, t.before.blocks - sectors);
            observations.push_str(&format!(
                "{} {:?} ino={} old_acl={} new_acl={} old_sectors={} new_sectors={} free_before={} free_after={} delta={}\n",
                t.path, t.branch, t.ino, t.before.file_acl, after.file_acl,
                t.before.blocks, after.blocks, free_before, free_blocks(&image), delta
            ));
            assert_eq!(
                (
                    after.mode,
                    after.uid,
                    after.gid,
                    after.size,
                    after.flags,
                    after.generation,
                    after.atime,
                    after.atime_nsec,
                    after.mtime,
                    after.mtime_nsec,
                    after.crtime,
                    after.crtime_nsec
                ),
                (
                    t.before.mode,
                    t.before.uid,
                    t.before.gid,
                    t.before.size,
                    t.before.flags,
                    t.before.generation,
                    t.before.atime,
                    t.before.atime_nsec,
                    t.before.mtime,
                    t.before.mtime_nsec,
                    t.before.crtime,
                    t.before.crtime_nsec
                )
            );
            assert_eq!(after.links_count, t.before.links_count);
            assert_eq!((after.ctime, after.ctime_nsec), (STAMP, 0));
            let bytes = std::fs::read(&image).unwrap();
            assert_eq!(
                u16::from_le_bytes(bytes[1082..1084].try_into().unwrap())
                    & crate::superblock::EXT4_VALID_FS,
                0
            );
            if fs.csum.enabled {
                assert_eq!(
                    u32::from_le_bytes(bytes[2044..2048].try_into().unwrap()),
                    crate::checksum::linux_crc32c(!0, &bytes[1024..2044])
                );
            }
        }
        for t in targets.iter().filter(|t| matches!(t.branch, Branch::Peer)) {
            let after = fs.read_inode_verified(t.ino).unwrap().0;
            let (_, raw) = fs.read_inode_verified(t.ino).unwrap();
            assert_eq!(raw, t.raw, "unaffected sharing inode changed");
            assert_eq!(after.file_acl, t.before.file_acl);
            assert_eq!(after.blocks, t.before.blocks);
            assert_eq!(
                (after.ctime, after.ctime_nsec),
                (t.before.ctime, t.before.ctime_nsec)
            );
            for (name, value, len) in &t.attrs {
                assert_eq!(
                    crate::xattr::get_resolved(fs, &after, &raw, name).unwrap(),
                    Some(vec![*value; *len])
                );
            }
        }
        assert_eq!(fs.chmod_operation().unwrap().operation.status().epoch, 8);
    });
    std::fs::write(base.with_extension("observations.txt"), &observations).unwrap();
    // Save the pre-close image separately for marker/crash review.
    std::fs::copy(&image, base.with_extension("dirty.img")).unwrap();
    fs.finish().unwrap();
    let closed = std::fs::read(&image).unwrap();
    assert_ne!(
        u16::from_le_bytes(closed[1082..1084].try_into().unwrap())
            & crate::superblock::EXT4_VALID_FS,
        0
    );
    assert_e2fsck_clean(image_str, "public removal emitted image");
    let fs = mount(&image);
    let mut rows = String::new();
    for t in &mut targets {
        let after = fs.read_inode_verified(t.ino).unwrap().0;
        report(&image, &base, "removed", t, &after);
        let attrs = t
            .attrs
            .iter()
            .map(|(name, value, length)| format!("{name:?}: bytes([{value}])*{length}"))
            .collect::<Vec<_>>()
            .join(",");
        rows.push_str(&format!(
            "({path:?},{ino},{mode},{uid},{gid},{blocks},{atime},{mtime},{ctime},{{{attrs}}}),\n",
            path = &t.path[1..],
            ino = t.ino,
            mode = after.mode,
            uid = after.uid,
            gid = after.gid,
            blocks = after.blocks,
            atime = i128::from(after.atime) * 1_000_000_000 + i128::from(after.atime_nsec),
            mtime = i128::from(after.mtime) * 1_000_000_000 + i128::from(after.mtime_nsec),
            ctime = i128::from(after.ctime) * 1_000_000_000 + i128::from(after.ctime_nsec)
        ));
    }
    drop(fs);
    let checked = guest_kernel_write(
        image_str,
        &format!(
            r#"
python3 - "$MNT" <<'REMOVE_READ_PY'
import os,sys
root=sys.argv[1]
for name,ino,mode,uid,gid,blocks,atime,mtime,ctime,attrs in [
{rows}
]:
    path=root+'/'+name
    info=os.stat(path)
    assert (info.st_ino,info.st_mode,info.st_uid,info.st_gid,info.st_blocks)==(ino,mode,uid,gid,blocks),info
    actual_times=(info.st_atime_ns,info.st_mtime_ns,info.st_ctime_ns)
    assert actual_times==(atime,mtime,ctime),(name,'timestamps',actual_times,(atime,mtime,ctime))
    assert set(os.listxattr(path))==set(attrs),(name,os.listxattr(path),attrs)
    for key,value in attrs.items(): assert os.getxattr(path,key)==value,(name,key)
    fd=os.open(path,os.O_RDONLY|os.O_NOATIME)
    try: assert os.read(fd,1024)==('data:'+name).encode(),name
    finally: os.close(fd)
    print(name,info.st_ino,info.st_blocks,info.st_ctime_ns,sorted(attrs))
REMOVE_READ_PY
"#
        ),
    );
    // Keep partial observations and the exact mismatch even on failed readback.
    std::fs::write(base.with_extension("linux.txt"), &checked.stdout).unwrap();
    std::fs::write(base.with_extension("linux.stderr.txt"), &checked.stderr).unwrap();
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    assert_e2fsck_clean(image_str, "Linux public removal readback");
    let fs = mount(&image);
    if size == 256 {
        let (_, raw) = fs.read_inode_verified(targets[6].ino).unwrap();
        assert_eq!(&raw[128..130], &[0, 0]);
    }
}
