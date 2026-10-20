//! Black-box Linux policy observations, independent of driver emission.
use super::setxattr::{lookup, make, mount, set};
use super::*;

#[test]
fn linux_records_same_value_setxattr_policy() {
    for bs in [1024, 4096] {
        for size in [128, 256] {
            for journal in [false, true] {
                reference(bs, size, journal);
            }
        }
    }
}

struct PolicyClock(std::sync::atomic::AtomicUsize);
impl crate::runtime::Runtime for PolicyClock {
    fn now_unix_seconds(&self) -> i64 {
        1_700_000_100 + 10 * self.0.fetch_add(1, Ordering::SeqCst) as i64
    }
    fn next_inode_generation(&self) -> u32 {
        17
    }
}

fn reference(bs: u32, size: u16, journal: bool) {
    let base = fs_ext4_test_support::temp_dir().join(format!("policy-{bs}-{size}-{journal}"));
    let image = base.with_extension("img");
    make(&image, bs, size, journal);
    let populated = guest_kernel_write(
        image.to_str().unwrap(),
        r#"
python3 - "$MNT" <<'POLICY_INPUT'
import os,sys
root=sys.argv[1]
for name in ['inline','empty','external','shared','shared-peer','duplicate']:
    p=root+'/'+name
    with open(p,'wb') as f:f.write(('data:'+name).encode())
    os.chmod(p,0o640)
    if name!='shared-peer':os.setxattr(p,'user.audit',b's'*600 if name=='shared' else b'' if name=='empty' else b'n'*(600 if name in ['external','duplicate'] else 1))
POLICY_INPUT
"#,
    );
    assert!(
        populated.status.success(),
        "{}",
        String::from_utf8_lossy(&populated.stderr)
    );
    let fs = mount(&image);
    super::removexattr::share(&fs, "/shared", "/shared-peer");
    fs.finish().unwrap();
    assert_e2fsck_clean(image.to_str().unwrap(), "same-value reference input");
    std::fs::copy(&image, base.with_extension("input.img")).unwrap();
    let duplicate = base.with_extension("duplicate.img");
    if size == 256 {
        std::fs::copy(&image, &duplicate).unwrap();
        let fs = mount(&duplicate);
        let (ino, inode) = lookup(&fs, "/duplicate");
        let (_, mut raw) = fs.read_inode_verified(ino).unwrap();
        let start = 128 + u16::from_le_bytes(raw[128..130].try_into().unwrap()) as usize;
        crate::xattr::plan_set_in_inode_region(&mut raw[start..], "user.audit", b"n").unwrap();
        fs.finalize_inode_raw(ino, inode.generation, &mut raw)
            .unwrap();
        fs.write_inode_raw(ino, &raw).unwrap();
        fs.finish().unwrap();
        std::fs::copy(&duplicate, base.with_extension("duplicate.input.img")).unwrap();
    }
    let result = guest_kernel_write(
        image.to_str().unwrap(),
        r#"
python3 - "$MNT" <<'POLICY_SAME'
import os,sys,time,json
root=sys.argv[1]
peer=os.getxattr(root+'/shared-peer','user.audit')
for name in ['inline','empty','external','shared']:
    p=root+'/'+name;value=os.getxattr(p,'user.audit');before=os.stat(p)
    time.sleep(0.02);os.setxattr(p,'user.audit',value);after=os.stat(p)
    assert os.getxattr(p,'user.audit')==value
    assert (before.st_ino,before.st_mode,before.st_blocks,before.st_size)==(after.st_ino,after.st_mode,after.st_blocks,after.st_size)
    assert (before.st_atime_ns,before.st_mtime_ns)==(after.st_atime_ns,after.st_mtime_ns)
    assert after.st_ctime_ns==before.st_ctime_ns,(name,before.st_ctime_ns,after.st_ctime_ns)
    print(json.dumps(dict(name=name,length=len(value),before=before.st_ctime_ns,after=after.st_ctime_ns,changed=after.st_ctime_ns!=before.st_ctime_ns,ino=after.st_ino,blocks=after.st_blocks)))
assert os.getxattr(root+'/shared-peer','user.audit')==peer
POLICY_SAME
"#,
    );
    std::fs::write(base.with_extension("reference.txt"), &result.stdout).unwrap();
    std::fs::write(base.with_extension("reference.stderr.txt"), &result.stderr).unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_e2fsck_clean(image.to_str().unwrap(), "same-value Linux reference");
    // A deliberately duplicated name is a diagnostic, not a healthy-image
    // qualification. Preserve both images and the Linux answer, including errno.
    if size == 256 {
        let result = guest_kernel_write(
            duplicate.to_str().unwrap(),
            r#"
python3 - "$MNT" <<'POLICY_DUPLICATE'
import os,sys,time,json
p=sys.argv[1]+'/duplicate';before=os.stat(p);value=os.getxattr(p,'user.audit')
time.sleep(0.02)
try:
    os.setxattr(p,'user.audit',value);error=0
except OSError as e:error=e.errno
after=os.stat(p)
print(json.dumps(dict(name='duplicate',length=len(value),before=before.st_ctime_ns,after=after.st_ctime_ns,changed=after.st_ctime_ns!=before.st_ctime_ns,error=error)))
POLICY_DUPLICATE
"#,
        );
        std::fs::write(
            base.with_extension("duplicate.reference.txt"),
            &result.stdout,
        )
        .unwrap();
        std::fs::write(base.with_extension("duplicate.stderr.txt"), &result.stderr).unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    for capi in [false, true] {
        let emitted = base.with_extension(if capi { "capi.img" } else { "rust.img" });
        std::fs::copy(base.with_extension("input.img"), &emitted).unwrap();
        let before = std::fs::read(&emitted).unwrap();
        let clock = Arc::new(PolicyClock(std::sync::atomic::AtomicUsize::new(0)));
        let fs = Filesystem::mount_with_runtime(
            Arc::new(FileDevice::open_rw(emitted.to_str().unwrap()).unwrap()),
            clock.clone(),
        )
        .unwrap();
        let status = fs.chmod_operation().unwrap().operation.status();
        let fs = crate::capi::with_test_mount(fs, |handle, fs| {
            for path in ["/inline", "/empty", "/external", "/shared"] {
                let (ino, inode) = lookup(fs, path);
                let raw = fs.read_inode_verified(ino).unwrap().1;
                let value = crate::xattr::get_resolved(fs, &inode, &raw, "user.audit")
                    .unwrap()
                    .unwrap();
                set(fs, handle, capi, path, &value);
                assert_eq!(fs.read_inode_verified(ino).unwrap().1, raw);
                assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
            }
        });
        assert_eq!(clock.0.load(Ordering::SeqCst), 0);
        assert!(std::fs::read(&emitted).unwrap() == before);
        fs.finish().unwrap();
        assert!(std::fs::read(&emitted).unwrap() == before);
        assert_e2fsck_clean(emitted.to_str().unwrap(), "same-value driver emission");
    }
}
