//! Exact public setter emission and separate fresh first-attribute references.
use super::super::timestamp_report::debugfs_time;
use super::removexattr::{allocated, free_blocks, refs, share};
use super::*;
use crate::inode::Inode;
use std::path::Path;
const NAME: &str = "user.audit";
const STAMP: i64 = 1_700_000_100;
struct SetterClock;
impl crate::runtime::Runtime for SetterClock {
    fn now_unix_seconds(&self) -> i64 {
        STAMP
    }
    fn next_inode_generation(&self) -> u32 {
        17
    }
}
pub(super) fn mount(image: &Path) -> Filesystem {
    Filesystem::mount_with_runtime(
        Arc::new(FileDevice::open_rw(image.to_str().unwrap()).unwrap()),
        Arc::new(SetterClock),
    )
    .unwrap()
}
#[derive(Clone, Copy, Debug)]
enum Branch {
    Inline,
    Grow,
    Free,
    Rewrite,
    Cow,
    Release,
    External,
    SharedExternal,
    Duplicate,
    Zero,
    Empty,
    Same,
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
pub(super) fn lookup(fs: &Filesystem, path: &str) -> (u32, Inode) {
    let ino = fs.lookup_path_bytes(path.as_bytes()).unwrap();
    (ino, fs.read_inode_verified(ino).unwrap().0)
}
pub(super) fn set(
    fs: &Filesystem,
    handle: *mut crate::capi::fs_ext4_fs_t,
    capi: bool,
    path: &str,
    value: &[u8],
) {
    if capi {
        let path = std::ffi::CString::new(path).unwrap();
        let name = std::ffi::CString::new(NAME).unwrap();
        assert_eq!(
            unsafe {
                crate::capi::fs_ext4_setxattr(
                    handle,
                    path.as_ptr(),
                    name.as_ptr(),
                    value.as_ptr().cast(),
                    value.len(),
                )
            },
            0
        );
        assert_eq!(crate::capi::fs_ext4_last_errno(), 0);
    } else {
        fs.apply_setxattr(path, NAME, value).unwrap();
    }
}
pub(super) fn make(image: &Path, bs: u32, size: u16, journal: bool) {
    std::fs::File::create(image)
        .unwrap()
        .set_len(32 * 1024 * 1024)
        .unwrap();
    let mut features = if size == 128 {
        "none,filetype"
    } else {
        "none,extent,64bit,filetype,metadata_csum"
    }
    .to_owned();
    if journal {
        features.push_str(",has_journal");
    }
    let out = oracle("mke2fs")
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
        .arg(image)
        .output();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
#[test]
fn linux_checks_public_setxattr_metadata() {
    for capi in [false, true] {
        for bs in [1024, 4096] {
            for journal in [false, true] {
                first_attribute(capi, bs, journal);
                for size in [128, 256] {
                    profile(capi, bs, size, journal);
                }
            }
        }
    }
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
    let count = text.lines().filter(|line| line.contains(NAME)).count();
    assert_eq!(
        count,
        usize::from(target.attrs.iter().any(|(name, _, _)| *name == NAME))
    );
    for (name, _, length) in &target.attrs {
        assert!(
            text.lines()
                .any(|line| line.contains(name) && line.contains(&format!("({length})"))),
            "{text}"
        );
    }
}

fn profile(capi: bool, bs: u32, size: u16, journal: bool) {
    let base =
        fs_ext4_test_support::temp_dir().join(format!("setter-{capi}-{bs}-{size}-{journal}"));
    let image = base.with_extension("img");
    let image_str = image.to_str().unwrap();
    make(&image, bs, size, journal);
    let made = guest_kernel_write(
        image_str,
        r#"
python3 - "$MNT" <<'SET_INPUT_PY'
import os,sys
root=sys.argv[1]
for name in ['inline','grow','free','rewrite','cow','cow-peer','release','release-peer','external','shared','shared-peer','duplicate','zero','empty','same','untouched']:
    p=root+'/'+name
    with open(p,'wb') as f:f.write(('data:'+name).encode())
    os.chmod(p,0o640);os.chown(p,1001,1002)
    os.utime(p,ns=(1600000000000000000,1600000001000000000))
for name,ch,length in [('inline',105,1),('grow',103,1),('free',102,600),('rewrite',114,600),('cow',99,600),('release',115,600),('external',101,600),('shared',104,600),('duplicate',100,600),('zero',122,600),('empty',109,600),('same',110,1)]:
    os.setxattr(root+'/'+name,'user.audit',bytes([ch])*length)
for name in ['rewrite','cow']:os.setxattr(root+'/'+name,'user.keep',b'k'*200)
SET_INPUT_PY
"#,
    );
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
    let fs = mount(&image);
    assert_eq!(fs.journal.is_some(), journal);
    for (source, peer) in [
        ("/cow", "/cow-peer"),
        ("/release", "/release-peer"),
        ("/shared", "/shared-peer"),
    ] {
        share(&fs, source, peer);
    }
    if size == 256 {
        let (ino, inode) = lookup(&fs, "/zero");
        let (_, mut raw) = fs.read_inode_verified(ino).unwrap();
        super::super::zero_extra_fixture::prepare(&mut raw).unwrap();
        fs.finalize_inode_raw(ino, inode.generation, &mut raw)
            .unwrap();
        fs.write_inode_raw(ino, &raw).unwrap();
    }
    fs.finish().unwrap();
    assert_e2fsck_clean(image_str, "setter shared coherent input");
    std::fs::copy(&image, base.with_extension("input.img")).unwrap();
    let fs = mount(&image);
    if size == 256 {
        let (ino, inode) = lookup(&fs, "/duplicate");
        let (_, mut raw) = fs.read_inode_verified(ino).unwrap();
        let start = 128 + u16::from_le_bytes(raw[128..130].try_into().unwrap()) as usize;
        crate::xattr::plan_set_in_inode_region(&mut raw[start..], NAME, b"old").unwrap();
        fs.finalize_inode_raw(ino, inode.generation, &mut raw)
            .unwrap();
        fs.write_inode_raw(ino, &raw).unwrap();
    }
    fs.finish().unwrap();
    std::fs::copy(&image, base.with_extension("prepared.img")).unwrap();
    let fs = mount(&image);
    let specs = [
        ("/inline", Branch::Inline),
        ("/grow", Branch::Grow),
        ("/free", Branch::Free),
        ("/rewrite", Branch::Rewrite),
        ("/cow", Branch::Cow),
        ("/release", Branch::Release),
        ("/external", Branch::External),
        ("/shared", Branch::SharedExternal),
        ("/duplicate", Branch::Duplicate),
        ("/zero", Branch::Zero),
        ("/empty", Branch::Empty),
        ("/same", Branch::Same),
        ("/cow-peer", Branch::Peer),
        ("/release-peer", Branch::Peer),
        ("/shared-peer", Branch::Peer),
        ("/untouched", Branch::Peer),
    ];
    let targets: Vec<_> = specs
        .into_iter()
        .map(|(path, branch)| {
            let (ino, before) = lookup(&fs, path);
            let raw = fs.read_inode_verified(ino).unwrap().1;
            let attrs = match path {
                "/rewrite" | "/cow" => vec![(NAME, b'n', 1), ("user.keep", b'k', 200)],
                "/cow-peer" => vec![(NAME, b'c', 600), ("user.keep", b'k', 200)],
                "/release-peer" => vec![(NAME, b's', 600)],
                "/shared-peer" => vec![(NAME, b'h', 600)],
                "/untouched" => vec![],
                "/external" | "/shared" => vec![(NAME, b'n', 700)],
                "/grow" => vec![(NAME, b'n', 600)],
                "/empty" => vec![(NAME, b'n', 0)],
                _ => vec![(NAME, b'n', 1)],
            };
            Target {
                path,
                branch,
                ino,
                before,
                raw,
                attrs,
            }
        })
        .collect();
    let mut observations = String::new();
    let fs = crate::capi::with_test_mount(fs, |handle, fs| {
        let held = fs.chmod_operation().unwrap();
        let before = std::fs::read(&image).unwrap();
        assert!(matches!(
            fs.apply_setxattr("/free", NAME, b"x"),
            Err(Error::Unsupported(_))
        ));
        drop(held);
        assert!(std::fs::read(&image).unwrap() == before);
        for t in targets.iter().filter(|t| !matches!(t.branch, Branch::Peer)) {
            let oldfree = free_blocks(&image);
            let length = t.attrs[0].2;
            let wanted = vec![b'n'; length];
            set(fs, handle, capi, t.path, &wanted);
            let (after, raw) = fs.read_inode_verified(t.ino).unwrap();
            assert_eq!(
                crate::xattr::get_resolved(fs, &after, &raw, NAME).unwrap(),
                Some(wanted)
            );
            let (delta, sectors) = if size == 128 {
                match t.branch {
                    Branch::Cow | Branch::Release | Branch::SharedExternal => (-1, 0),
                    _ => (0, 0),
                }
            } else {
                match t.branch {
                    Branch::Grow => (-1, i64::from(bs / 512)),
                    Branch::Free | Branch::Duplicate | Branch::Zero | Branch::Empty => {
                        (1, -i64::from(bs / 512))
                    }
                    Branch::Cow | Branch::SharedExternal => (-1, 0),
                    Branch::Release => (0, -i64::from(bs / 512)),
                    _ => (0, 0),
                }
            };
            assert_eq!(
                free_blocks(&image) as i64,
                oldfree as i64 + delta,
                "{:?}",
                t.branch
            );
            assert_eq!(
                after.blocks as i64,
                t.before.blocks as i64 + sectors,
                "{:?}",
                t.branch
            );
            if after.file_acl != 0 {
                assert!(allocated(fs, after.file_acl));
                assert_eq!(refs(fs, after.file_acl), 1);
            }
            if matches!(
                t.branch,
                Branch::Cow | Branch::Release | Branch::SharedExternal
            ) {
                assert_ne!(after.file_acl, t.before.file_acl);
                assert!(allocated(fs, t.before.file_acl));
                assert_eq!(refs(fs, t.before.file_acl), 1);
            } else if delta == 1 {
                assert_eq!(after.file_acl, 0);
                assert!(!allocated(fs, t.before.file_acl));
            } else if matches!(t.branch, Branch::Rewrite | Branch::External) {
                assert_eq!(after.file_acl, t.before.file_acl);
            }
            assert_eq!(after.links_count, t.before.links_count);
            assert_eq!(
                (after.ctime, after.ctime_nsec),
                if matches!(t.branch, Branch::Same) {
                    (t.before.ctime, t.before.ctime_nsec)
                } else {
                    (STAMP, 0)
                }
            );
            assert_eq!(
                (
                    after.mode,
                    after.uid,
                    after.gid,
                    after.flags,
                    after.generation,
                    after.size,
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
                    t.before.flags,
                    t.before.generation,
                    t.before.size,
                    t.before.atime,
                    t.before.atime_nsec,
                    t.before.mtime,
                    t.before.mtime_nsec,
                    t.before.crtime,
                    t.before.crtime_nsec
                )
            );
            observations.push_str(&format!(
                "{} {:?} inode={} acl={}->{} sectors={}->{} free={}->{}\n",
                t.path,
                t.branch,
                t.ino,
                t.before.file_acl,
                after.file_acl,
                t.before.blocks,
                after.blocks,
                oldfree,
                free_blocks(&image)
            ));
            check_dirty(&image, fs);
        }
        for t in targets.iter().filter(|t| matches!(t.branch, Branch::Peer)) {
            assert_eq!(fs.read_inode_verified(t.ino).unwrap().1, t.raw);
        }
        assert_eq!(fs.chmod_operation().unwrap().operation.status().epoch, 12);
    });
    std::fs::write(base.with_extension("observations.txt"), observations).unwrap();
    std::fs::copy(&image, base.with_extension("dirty.img")).unwrap();
    fs.finish().unwrap();
    check_closed(&image);
    assert_e2fsck_clean(image_str, "public setter emission");
    validate(&image, &base, &targets);
}
fn check_dirty(image: &Path, fs: &Filesystem) {
    let b = std::fs::read(image).unwrap();
    assert_eq!(
        u16::from_le_bytes(b[1082..1084].try_into().unwrap()) & crate::superblock::EXT4_VALID_FS,
        0
    );
    assert_ne!(
        u32::from_le_bytes(b[1116..1120].try_into().unwrap()) & features::Compat::EXT_ATTR.bits(),
        0
    );
    if fs.csum.enabled {
        assert_eq!(
            u32::from_le_bytes(b[2044..2048].try_into().unwrap()),
            crate::checksum::linux_crc32c(!0, &b[1024..2044])
        );
    }
}
fn check_closed(image: &Path) {
    let b = std::fs::read(image).unwrap();
    assert_ne!(
        u16::from_le_bytes(b[1082..1084].try_into().unwrap()) & crate::superblock::EXT4_VALID_FS,
        0
    );
}
fn validate(image: &Path, base: &Path, targets: &[Target]) {
    let fs = mount(image);
    let mut rows = String::new();
    for t in targets {
        let after = fs.read_inode_verified(t.ino).unwrap().0;
        report(image, base, "set", t, &after);
        let attrs = t
            .attrs
            .iter()
            .map(|(name, value, len)| format!("{name:?}:bytes([{value}])*{len}"))
            .collect::<Vec<_>>()
            .join(",");
        rows.push_str(&format!(
            "({:?},{},{},{},{},{},{},{},{},{{{attrs}}}),\n",
            &t.path[1..],
            t.ino,
            after.mode,
            after.uid,
            after.gid,
            after.blocks,
            i128::from(after.atime) * 1_000_000_000 + i128::from(after.atime_nsec),
            i128::from(after.mtime) * 1_000_000_000 + i128::from(after.mtime_nsec),
            i128::from(after.ctime) * 1_000_000_000 + i128::from(after.ctime_nsec)
        ));
    }
    drop(fs);
    let checked = guest_kernel_write(
        image.to_str().unwrap(),
        &format!(
            r#"
python3 - "$MNT" <<'SET_READ_PY'
import os,sys
root=sys.argv[1]
for name,ino,mode,uid,gid,blocks,atime,mtime,ctime,attrs in [
{rows}
]:
    path=root+'/'+name;s=os.stat(path)
    assert (s.st_ino,s.st_mode,s.st_uid,s.st_gid,s.st_blocks)==(ino,mode,uid,gid,blocks),(name,s)
    assert (s.st_atime_ns,s.st_mtime_ns,s.st_ctime_ns)==(atime,mtime,ctime),(name,s,(atime,mtime,ctime))
    assert set(os.listxattr(path))==set(attrs),(name,os.listxattr(path),attrs)
    for key,value in attrs.items():assert os.getxattr(path,key)==value,(name,key)
    fd=os.open(path,os.O_RDONLY|os.O_NOATIME)
    try:assert os.read(fd,1024)==('data:'+name).encode(),name
    finally:os.close(fd)
    print(name,ino,blocks,ctime,sorted(attrs))
SET_READ_PY
"#
        ),
    );
    std::fs::write(base.with_extension("linux.txt"), &checked.stdout).unwrap();
    std::fs::write(base.with_extension("linux.stderr.txt"), &checked.stderr).unwrap();
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    assert_e2fsck_clean(image.to_str().unwrap(), "Linux public setter readback");
}

struct FirstClock(std::sync::atomic::AtomicI64);
impl crate::runtime::Runtime for FirstClock {
    fn now_unix_seconds(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
    fn next_inode_generation(&self) -> u32 {
        17
    }
}
fn first_attribute(capi: bool, bs: u32, journal: bool) {
    let base = fs_ext4_test_support::temp_dir().join(format!("first-{capi}-{bs}-{journal}"));
    let image = base.with_extension("img");
    let image_str = image.to_str().unwrap();
    make(&image, bs, 256, journal);
    let made = guest_kernel_write(
        image_str,
        r#"
python3 - "$MNT" <<'FIRST_INPUT_PY'
import os,sys
for name in ['first','same','empty']:
    p=sys.argv[1]+'/'+name
    with open(p,'wb') as f:f.write(('data:'+name).encode())
    os.chmod(p,0o640);os.chown(p,1001,1002)
    os.utime(p,ns=(1600000000000000000,1600000001000000000))
    assert os.listxattr(p)==[]
FIRST_INPUT_PY
"#,
    );
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
    let input = std::fs::read(&image).unwrap();
    assert_eq!(
        u32::from_le_bytes(input[1116..1120].try_into().unwrap())
            & features::Compat::EXT_ATTR.bits(),
        0,
        "first-attribute fixture already has EXT_ATTR"
    );
    std::fs::copy(&image, base.with_extension("input.img")).unwrap();
    let reference = base.with_extension("reference.img");
    std::fs::copy(&image, &reference).unwrap();
    let result = guest_kernel_write(
        reference.to_str().unwrap(),
        r#"
python3 - "$MNT" <<'FIRST_REF_PY'
import os,sys,time
root=sys.argv[1]
for name,value in [('first',b'n'),('same',b'n'),('empty',b'')]:
    p=root+'/'+name;before=os.stat(p).st_ctime_ns
    time.sleep(0.02);os.setxattr(p,'user.audit',value);after=os.stat(p).st_ctime_ns
    assert after>before,(name,before,after)
    assert os.getxattr(p,'user.audit')==value
    print(name,'first',before,after)
    if name=='same':
        time.sleep(0.02);os.setxattr(p,'user.audit',value);again=os.stat(p).st_ctime_ns
        assert again==after,(after,again)
        print(name,'same_value_changed',int(again!=after),after,again)
FIRST_REF_PY
"#,
    );
    std::fs::write(base.with_extension("reference.txt"), &result.stdout).unwrap();
    std::fs::write(base.with_extension("reference.stderr.txt"), &result.stderr).unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_e2fsck_clean(
        reference.to_str().unwrap(),
        "Linux first-attribute reference",
    );
    let reference_bytes = std::fs::read(&reference).unwrap();
    assert_ne!(
        u32::from_le_bytes(reference_bytes[1116..1120].try_into().unwrap())
            & features::Compat::EXT_ATTR.bits(),
        0
    );
    let ref_fs = mount(&reference);
    for name in ["/first", "/same", "/empty"] {
        let (ino, inode) = lookup(&ref_fs, name);
        assert_eq!(inode.file_acl, 0);
        let attrs = vec![(NAME, b'n', usize::from(name != "/empty"))];
        let target = Target {
            path: name,
            branch: Branch::Inline,
            ino,
            before: inode.clone(),
            raw: ref_fs.read_inode_verified(ino).unwrap().1,
            attrs,
        };
        report(&reference, &base, "reference", &target, &inode);
    }
    drop(ref_fs);
    let clock = Arc::new(FirstClock(std::sync::atomic::AtomicI64::new(STAMP)));
    let fs = Filesystem::mount_with_runtime(
        Arc::new(FileDevice::open_rw(image_str).unwrap()),
        clock.clone(),
    )
    .unwrap();
    let free_before = free_blocks(&image);
    let targets: Vec<_> = ["/first", "/same", "/empty"]
        .into_iter()
        .map(|path| {
            let (ino, before) = lookup(&fs, path);
            assert_eq!(before.file_acl, 0);
            Target {
                path,
                branch: Branch::Inline,
                ino,
                raw: fs.read_inode_verified(ino).unwrap().1,
                before,
                attrs: vec![(NAME, b'n', usize::from(path != "/empty"))],
            }
        })
        .collect();
    let fs = crate::capi::with_test_mount(fs, |handle, fs| {
        for t in &targets {
            let value = vec![b'n'; t.attrs[0].2];
            clock.0.store(STAMP, Ordering::SeqCst);
            set(fs, handle, capi, t.path, &value);
            if t.path == "/same" {
                let before = std::fs::read(&image).unwrap();
                let status = fs.chmod_operation().unwrap().operation.status();
                clock.0.store(STAMP + 10, Ordering::SeqCst);
                set(fs, handle, capi, t.path, &value);
                assert!(std::fs::read(&image).unwrap() == before);
                assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
            }
            let inode = fs.read_inode_verified(t.ino).unwrap().0;
            assert_eq!(inode.file_acl, 0);
            assert_eq!(inode.blocks, t.before.blocks);
            assert_eq!((inode.ctime, inode.ctime_nsec), (STAMP, 0));
            assert_eq!(free_blocks(&image), free_before);
            check_dirty(&image, fs);
        }
        assert_eq!(fs.chmod_operation().unwrap().operation.status().epoch, 4);
    });
    std::fs::copy(&image, base.with_extension("dirty.img")).unwrap();
    fs.finish().unwrap();
    check_closed(&image);
    assert_e2fsck_clean(image_str, "first inline setter emission");
    validate(&image, &base, &targets);
}
