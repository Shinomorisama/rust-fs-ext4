//! Public Rust/C retained-index split against an exact Linux-built input.
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

struct DriverClock {
    stamp: i64,
    calls: AtomicUsize,
}

impl crate::runtime::Runtime for DriverClock {
    fn now_unix_seconds(&self) -> i64 {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.stamp
    }

    fn next_inode_generation(&self) -> u32 {
        panic!("a hard link must retain the existing inode generation")
    }
}

fn long_name(n: usize) -> Vec<u8> {
    let mut source = format!("alias-{n:04}-").into_bytes();
    source.extend(std::iter::repeat_n(b'n', 244));
    assert_eq!(source.len(), 255);
    let result = [b"growth-".as_slice(), &source[7..]].concat();
    assert_eq!(result.len(), 255);
    result
}

fn make(image: &Path, bs: u32, journal: bool) {
    std::fs::File::create(image)
        .unwrap()
        .set_len(32 * 1024 * 1024)
        .unwrap();
    let features = if journal {
        "none,extent,64bit,filetype,metadata_csum,dir_index,has_journal"
    } else {
        "none,extent,64bit,filetype,metadata_csum,dir_index"
    };
    let made = oracle("mke2fs")
        .args([
            "-q",
            "-F",
            "-t",
            "ext4",
            "-b",
            &bs.to_string(),
            "-I",
            "256",
            "-m",
            "0",
            "-O",
            features,
            "-E",
            "lazy_itable_init=0,lazy_journal_init=0",
        ])
        .arg(image)
        .output();
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
}

fn prepare_linux_input(bs: u32, journal: bool) -> (std::path::PathBuf, View, Vec<u8>) {
    let base =
        fs_ext4_test_support::temp_dir().join(format!("link-index-driver-source-{bs}-{journal}"));
    let image = base.with_extension("working.img");
    let candidate = base.with_extension("candidate.img");
    make(&image, bs, journal);
    guest(&image, &base, "input", 0, "");
    let input = saved(&image, &base, "input");
    guest(&image, &base, "inplace", 0, "");
    let inplace = saved(&image, &base, "inplace");
    transition(&input, &inplace, &image, false);
    let mut before = inplace;
    for n in 0..64 {
        std::fs::copy(&image, &candidate).unwrap();
        guest(&image, &base, "step", n, "");
        let after = view(&image);
        let grew = after.records[0].1.size > before.records[0].1.size;
        transition(&before, &after, &image, grew);
        if grew {
            let prepared = saved(&candidate, &base, "growth-input");
            transition(&prepared, &after, &image, true);
            let linux = saved(&image, &base, "linux-grown");
            std::fs::remove_file(&image).unwrap();
            std::fs::remove_file(&candidate).unwrap();
            return (base.with_extension("growth-input.img"), linux, long_name(n));
        }
        before = after;
    }
    panic!("bounded Linux preparation never split a leaf")
}

fn dispatch(
    fs: &Filesystem,
    handle: *mut fs_ext4_fs_t,
    capi: bool,
    target: InodeRef,
    parent: InodeRef,
    name: &[u8],
) -> std::result::Result<Option<fs_ext4_attr_t>, i32> {
    let alias = Alias {
        phase: "indexed",
        source: 1,
        parent: 0,
        name: name.to_vec(),
        entry: 1,
    };
    call(
        fs,
        handle,
        capi,
        &alias,
        (target, parent),
        (b"/source", &[b"/policy/".as_slice(), name].concat(), name),
    )
}

fn compare_linux(driver: &View, linux: &View) {
    assert_eq!(driver.entries, linux.entries);
    assert_eq!(driver.mapping, linux.mapping);
    assert_eq!(driver.roots, linux.roots);
    assert_eq!(driver.free, linux.free);
    assert_eq!(driver.allocation, linux.allocation);
    assert_eq!(driver.eas, linux.eas);
    assert_eq!(driver.payloads, linux.payloads);
    assert_eq!(driver.root_directory, linux.root_directory);
    assert_eq!(
        (
            driver.records[0].1.flags,
            driver.records[0].1.size,
            driver.records[0].1.blocks,
            driver.records[0].1.links_count,
            driver.records[1].1.links_count,
        ),
        (
            linux.records[0].1.flags,
            linux.records[0].1.size,
            linux.records[0].1.blocks,
            linux.records[0].1.links_count,
            linux.records[1].1.links_count,
        )
    );
}

fn profile(source: &Path, linux: &View, name: &[u8], capi: bool, bs: u32, journal: bool) {
    let base = fs_ext4_test_support::temp_dir().join(format!(
        "link-index-driver-{}-{bs}-{journal}",
        if capi { "capi" } else { "rust" }
    ));
    let image = base.with_extension("img");
    std::fs::copy(source, &image).unwrap();
    let input = saved(&image, &base, "input");
    assert_eq!(
        input.entries.len() + 1,
        usize::from(input.records[1].1.links_count)
    );
    let clock = Arc::new(DriverClock {
        stamp: input.records.iter().map(|r| r.1.ctime).max().unwrap() + 100,
        calls: AtomicUsize::new(0),
    });
    assert!(clock.stamp < i64::from(i32::MAX));
    let fs = Filesystem::mount_with_runtime(
        Arc::new(FileDevice::open_path_rw(&image).unwrap()),
        clock.clone(),
    )
    .unwrap();
    assert_eq!(fs.journal.is_some(), journal);
    assert_eq!((fs.sb.block_size(), fs.sb.inode_size), (bs, 256));
    let parent = InodeRef::new(input.records[0].0, input.records[0].1.generation);
    let target = InodeRef::new(input.records[1].0, input.records[1].1.generation);
    let mut attr = None;
    let fs = crate::capi::with_test_mount(fs, |handle, fs| {
        let mounted = std::fs::read(&image).unwrap();
        for (candidate, expected) in [
            (input.entries[0].0.as_slice(), crate::error::errno::EEXIST),
            (&[b'x'; 256], crate::error::errno::ENAMETOOLONG),
        ] {
            assert_eq!(
                dispatch(fs, handle, capi, target, parent, candidate).err(),
                Some(expected)
            );
            assert_eq!(std::fs::read(&image).unwrap(), mounted);
            assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        }
        assert_eq!(
            dispatch(fs, handle, capi, parent, parent, b"directory-link").err(),
            Some(crate::error::errno::EISDIR)
        );
        assert_eq!(std::fs::read(&image).unwrap(), mounted);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        std::fs::copy(&image, base.with_extension("refused.img")).unwrap();
        attr = dispatch(fs, handle, capi, target, parent, name).unwrap();
        assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
        assert_eq!(fs.lookup_at(parent.ino, name).unwrap(), target.ino);
        let (parent_after, parent_raw) = fs.read_inode_verified(parent.ino).unwrap();
        assert_ne!(
            parent_after.flags & crate::inode::InodeFlags::INDEX.bits(),
            0,
            "driver dropped the index instead of splitting the leaf"
        );
        assert_eq!(parent_after.size, input.records[0].1.size + u64::from(bs));
        assert_eq!(
            parent_after.blocks,
            input.records[0].1.blocks + u64::from(bs / 512)
        );
        assert_eq!(
            (
                parent_after.mtime,
                parent_after.mtime_nsec,
                parent_after.ctime,
                parent_after.ctime_nsec,
            ),
            (clock.stamp, 0, clock.stamp, 0)
        );
        assert_eq!(version(&parent_raw), version(&input.records[0].2) + 1);
        let target_after = fs.stat_ino(target.ino).unwrap();
        assert_eq!(target_after.links_count, input.records[1].1.links_count + 1);
        assert_eq!(
            (target_after.ctime, target_after.ctime_nsec),
            (clock.stamp, 0)
        );
        if let Some(ref value) = attr {
            check_attr(value, target.ino, &target_after);
        }
        std::fs::copy(&image, base.with_extension("dirty.img")).unwrap();
    });
    fs.finish().unwrap();
    assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
    std::fs::copy(&image, base.with_extension("closed.img")).unwrap();
    let closed = saved(&image, &base, "closed");
    transition(&input, &closed, &image, true);
    compare_linux(&closed, linux);
    std::fs::write(
        base.with_extension("observation.json"),
        format!(
            "{{\"block_size\":{bs},\"journal\":{journal},\"capi\":{capi},\"name\":\"{}\",\"entries_before\":{},\"entries_after\":{},\"routes_before\":{},\"routes_after\":{},\"clock_calls\":1,\"refusals\":3}}\n",
            name.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            input.entries.len(),
            closed.entries.len(),
            input.roots,
            closed.roots,
        ),
    )
    .unwrap();
    let original = std::fs::read(&image).unwrap();
    let cold = base.with_extension("cold.img");
    std::fs::copy(&image, &cold).unwrap();
    guest(&cold, &base, "observe", 0, "");
    let observed = saved(&cold, &base, "cold-readback");
    assert_eq!(closed.directory, observed.directory);
    assert_eq!(closed.tables, observed.tables);
    assert_eq!(closed.allocation, observed.allocation);
    assert_eq!(closed.eas, observed.eas);
    assert_eq!(closed.payloads, observed.payloads);
    assert_eq!(closed.root_directory, observed.root_directory);
    assert_eq!(std::fs::read(&image).unwrap(), original);
    eprintln!(
        "driver indexed split capi={capi} bs={bs} journal={journal}: retained index, one route/block, three refusals, clean close and cold Linux observation"
    );
    std::fs::remove_file(image).unwrap();
}

#[test]
fn linux_checks_public_indexed_link_split() {
    for bs in [1024, 4096] {
        for journal in [false, true] {
            let (source, linux, name) = prepare_linux_input(bs, journal);
            for capi in [false, true] {
                profile(&source, &linux, &name, capi, bs, journal);
            }
        }
    }
}
