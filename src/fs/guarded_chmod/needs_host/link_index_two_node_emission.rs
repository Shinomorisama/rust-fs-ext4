//! Separate ordinary Rust/C emission on each exact completed Linux input/query.
use super::*;
use crate::block_io::BlockDevice;
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
        panic!("a hard link must retain the source generation")
    }
}

// Count actual device effects as well as comparing the complete mounted bytes.
struct Device {
    file: FileDevice,
    writes: AtomicUsize,
    flushes: AtomicUsize,
}
impl Device {
    fn counts(&self) -> (usize, usize) {
        (
            self.writes.load(Ordering::SeqCst),
            self.flushes.load(Ordering::SeqCst),
        )
    }
}
impl BlockDevice for Device {
    fn read_at(&self, offset: u64, bytes: &mut [u8]) -> crate::error::Result<()> {
        self.file.read_at(offset, bytes)
    }
    fn size_bytes(&self) -> u64 {
        self.file.size_bytes()
    }
    fn write_at(&self, offset: u64, bytes: &[u8]) -> crate::error::Result<()> {
        self.writes.fetch_add(1, Ordering::SeqCst);
        self.file.write_at(offset, bytes)
    }
    fn flush(&self) -> crate::error::Result<()> {
        self.flushes.fetch_add(1, Ordering::SeqCst);
        self.file.flush()
    }
    fn is_writable(&self) -> bool {
        true
    }
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
        phase: "two-node",
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

fn snapshot(image: &Path, base: &Path, phase: &str) -> String {
    let path = base.with_extension(format!("{phase}.img"));
    std::fs::copy(image, &path).unwrap();
    let hash = sha(&path);
    std::fs::write(base.with_extension(format!("{phase}.sha256")), &hash).unwrap();
    hash
}

fn attribute(base: &Path, attr: &fs_ext4_attr_t) {
    std::fs::write(base.with_extension("capi-attr.json"),format!(
        "{{\"inode\":{},\"generation\":{},\"file_type\":{},\"mode\":{},\"uid\":{},\"gid\":{},\"size\":{},\"blocks_512\":{},\"link_count\":{},\"inode_flags\":{},\"atime\":{},\"atime_nsec\":{},\"mtime\":{},\"mtime_nsec\":{},\"ctime\":{},\"ctime_nsec\":{},\"crtime\":{},\"crtime_nsec\":{}}}\n",
        attr.inode,attr.generation,attr.file_type as u32,attr.mode,attr.uid,attr.gid,
        attr.size,attr.blocks_512,attr.link_count,attr.inode_flags,
        attr.atime,attr.atime_nsec,attr.mtime,attr.mtime_nsec,attr.ctime,attr.ctime_nsec,attr.crtime,attr.crtime_nsec)).unwrap();
}

fn boundary(state: &Snapshot, prepared: &Prepared) -> Option<u32> {
    if !prepared.query.split {
        return None;
    }
    let node = prepared.query.node as usize;
    let (_, routes) = crate::htree::parse_node_entries(&state.view.directory[node]).unwrap();
    let logical = prepared.before.view.mapping.len() as u32;
    let added: Vec<_> = routes.iter().filter(|e| e.block == logical).collect();
    assert_eq!(added.len(), 1);
    Some(added[0].hash)
}

fn emit(
    reference: &CompletedCase,
    reference_base: &Path,
    base: &Path,
    capi: bool,
    journal: bool,
) -> String {
    let prepared = &reference.prepared;
    let before = &prepared.before;
    let q = &prepared.query;
    let image = base.with_extension("working.img");
    let linux_path = reference_base.with_extension("two-node-output.img");
    let linux_hash = sha(&linux_path);
    assert_eq!(sha(&prepared.input), prepared.input_hash);
    std::fs::copy(&prepared.input, &image).unwrap();
    // Capture before mounting a writer; do not run the selector a second time.
    let input = capture(&image, base, "input");
    assert_eq!(sha(&image), prepared.input_hash);
    assert_eq!(input.view.entries, before.view.entries);
    std::fs::write(base.with_extension("query.json"), q.json()).unwrap();
    std::fs::write(base.with_extension("pair.json"),format!(
        "{{\"reference\":\"{}\",\"input_file\":\"{}\",\"input_sha256\":\"{}\",\"linux_output_sha256\":\"{linux_hash}\",\"capi\":{capi},\"query\":{}}}\n",
        reference_base.file_name().unwrap().to_str().unwrap(),prepared.input.file_name().unwrap().to_str().unwrap(),prepared.input_hash,q.json().trim())).unwrap();
    let clock = Arc::new(DriverClock {
        stamp: before.view.records.iter().map(|r| r.1.ctime).max().unwrap() + 100,
        calls: AtomicUsize::new(0),
    });
    assert!(clock.stamp < i64::from(i32::MAX));
    let device = Arc::new(Device {
        file: FileDevice::open_path_rw(&image).unwrap(),
        writes: AtomicUsize::new(0),
        flushes: AtomicUsize::new(0),
    });
    let fs = Filesystem::mount_with_runtime(device.clone(), clock.clone()).unwrap();
    assert_eq!(fs.journal.is_some(), journal);
    let target = InodeRef::new(
        before.view.records[1].0,
        before.view.records[1].1.generation,
    );
    let parent = InodeRef::new(
        before.view.records[0].0,
        before.view.records[0].1.generation,
    );
    let fs = crate::capi::with_test_mount(fs, |handle, fs| {
        let mounted = std::fs::read(&image).unwrap();
        let counts = device.counts();
        let marker = fs.marked_not_clean.load(Ordering::SeqCst);
        let status = fs.chmod_operation().unwrap().operation.status();
        let mut rows = Vec::new();
        for (label, source, name, expected) in [
            (
                "duplicate",
                target,
                before.view.entries[0].0.as_slice(),
                crate::error::errno::EEXIST,
            ),
            (
                "overlong",
                target,
                &[b'x'; 256],
                crate::error::errno::ENAMETOOLONG,
            ),
            (
                "directory",
                parent,
                b"directory-link".as_slice(),
                crate::error::errno::EISDIR,
            ),
        ] {
            let error = dispatch(fs, handle, capi, source, parent, name).err();
            rows.push(format!("{{\"case\":\"{label}\",\"expected_errno\":{expected},\"actual_errno\":{},\"clock_calls\":{},\"writes\":{},\"flushes\":{}}}",error.map_or_else(||"null".to_owned(),|e|e.to_string()),clock.calls.load(Ordering::SeqCst),device.counts().0-counts.0,device.counts().1-counts.1));
            std::fs::write(
                base.with_extension("refusals.jsonl"),
                format!("{}\n", rows.join("\n")),
            )
            .unwrap();
            assert_eq!(error, Some(expected));
            assert_eq!(std::fs::read(&image).unwrap(), mounted);
            assert_eq!(device.counts(), counts);
            assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
            assert_eq!(fs.marked_not_clean.load(Ordering::SeqCst), marker);
            assert_eq!(fs.chmod_operation().unwrap().operation.status(), status);
        }
        assert_eq!(
            snapshot(&image, base, "refused"),
            fs_ext4_test_support::sha256_hex(&mounted)
        );
        let result = dispatch(fs, handle, capi, target, parent, &q.name);
        // Preserve post-call bytes and errno before any positive assertion.
        snapshot(&image, base, "dirty");
        std::fs::write(
            base.with_extension("call.json"),
            format!(
                "{{\"errno\":{},\"clock_calls\":{},\"writes\":{},\"flushes\":{}}}\n",
                result
                    .as_ref()
                    .err()
                    .map_or_else(|| "null".to_owned(), |e| e.to_string()),
                clock.calls.load(Ordering::SeqCst),
                device.counts().0 - counts.0,
                device.counts().1 - counts.1
            ),
        )
        .unwrap();
        let attr = result.unwrap();
        assert_eq!(attr.is_some(), capi);
        assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
        assert!(device.counts().0 > counts.0);
        assert!(fs.marked_not_clean.load(Ordering::SeqCst));
        assert_eq!(
            fs.chmod_operation().unwrap().operation.status().health,
            crate::operation::Health::Healthy
        );
        assert_eq!(fs.lookup_at(parent.ino, &q.name).unwrap(), target.ino);
        let source = fs.stat_ino(target.ino).unwrap();
        if let Some(attr) = attr {
            attribute(base, &attr);
            check_attr(&attr, target.ino, &source);
        }
        let raw = std::fs::read(&image).unwrap();
        assert_eq!(
            u16::from_le_bytes(raw[1082..1084].try_into().unwrap())
                & crate::superblock::EXT4_VALID_FS,
            0
        );
        if fs.csum.enabled {
            assert_eq!(
                u32::from_le_bytes(raw[2044..2048].try_into().unwrap()),
                crate::checksum::linux_crc32c(!0, &raw[1024..2044])
            );
        }
    });
    fs.finish().unwrap();
    assert_eq!(clock.calls.load(Ordering::SeqCst), 1);
    drop(device);
    let closed = capture(&image, base, "closed");
    let allocated = super::transition(prepared, &closed, &image);
    assert_eq!(closed.view.entries, reference.linux.view.entries);
    let parent = &closed.view.records[0].1;
    let source = &closed.view.records[1].1;
    assert_eq!(
        (
            parent.mtime,
            parent.mtime_nsec,
            parent.ctime,
            parent.ctime_nsec
        ),
        (clock.stamp, 0, clock.stamp, 0)
    );
    assert_eq!((source.ctime, source.ctime_nsec), (clock.stamp, 0));
    let actual_boundary = boundary(&closed, prepared);
    assert_eq!(actual_boundary, q.boundary);
    let linux_boundary = boundary(&reference.linux, prepared);
    let closed_hash = sha(&base.with_extension("closed.img"));
    assert_eq!(
        sha(&image),
        closed_hash,
        "closed observers changed driver bytes"
    );
    let cold = base.with_extension("cold-working.img");
    std::fs::copy(&image, &cold).unwrap();
    super::guest(&cold, base, "cold", "cold", prepared, &closed);
    let observed = capture(&cold, base, "cold");
    assert_eq!(observed.view.entries, closed.view.entries);
    assert_eq!(sha(&cold), closed_hash);
    assert_eq!(sha(&base.with_extension("cold.img")), closed_hash);
    assert_eq!(sha(&image), closed_hash);
    assert_eq!(sha(&prepared.input), prepared.input_hash);
    assert_eq!(sha(&linux_path), linux_hash);
    let extent_added = closed.map_nodes.len() - before.map_nodes.len();
    let option = |value: Option<u32>| value.map_or_else(|| "null".to_owned(), |n| n.to_string());
    let row=format!("{{\"case\":\"{}\",\"reference\":\"{}\",\"capi\":{capi},\"input_sha256\":\"{}\",\"closed_sha256\":\"{closed_hash}\",\"clock_calls\":1,\"stamp\":{},\"refusals\":3,\"allocated\":{allocated},\"extent_added\":{extent_added},\"linux_boundary\":{},\"driver_boundary\":{},\"query\":{}}}",base.file_name().unwrap().to_str().unwrap(),reference_base.file_name().unwrap().to_str().unwrap(),prepared.input_hash,clock.stamp,option(linux_boundary),option(actual_boundary),q.json().trim());
    std::fs::write(base.with_extension("observation.json"), format!("{row}\n")).unwrap();
    std::fs::remove_file(image).unwrap();
    std::fs::remove_file(cold).unwrap();
    row
}

#[cfg(test)]
#[test]
fn linux_checks_public_two_node_indexed_links() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tmp/link-index-two-node-emission");
    let run = fs_ext4_test_support::create_unique_temp_dir(&root).unwrap();
    let mut references = Vec::new();
    let mut emissions = Vec::new();
    let manifest = |references: &[String], emissions: &[String]| {
        std::fs::write(run.join("matrix.json"),format!("{{\"expected_references\":16,\"expected_emissions\":32,\"references\":[{}],\"emissions\":[{}]}}\n",references.join(","),emissions.join(","))).unwrap();
    };
    manifest(&references, &emissions);
    for bs in [1024, 4096] {
        for journal in [false, true] {
            let prefix = format!(
                "two-node-emission-source-{}-{bs}-{journal}",
                std::process::id()
            );
            let result =
                std::panic::catch_unwind(|| node_full::profile_named(bs, journal, &prefix));
            retain_bootstrap(&prefix, &run);
            if let Err(error) = result {
                std::panic::resume_unwind(error);
            }
            let source = run
                .join(format!("{prefix}-{bs}-{journal}"))
                .with_extension("node-grown.img");
            let source_hash = sha(&source);
            std::fs::write(source.with_extension("sha256"), &source_hash).unwrap();
            for range in 0..2 {
                for split in [false, true] {
                    let base = run.join(format!("reference-{bs}-{journal}-{range}-{split}"));
                    std::fs::write(base.with_extension("source.json"),format!("{{\"file\":\"{}\",\"sha256\":\"{source_hash}\",\"block_size\":{bs},\"journal\":{journal},\"root_slot\":{range},\"split\":{split}}}\n",source.file_name().unwrap().to_str().unwrap())).unwrap();
                    let reference = super::run_case(&source, &base, range, split);
                    references.push(reference.row.clone());
                    manifest(&references, &emissions);
                    for capi in [false, true] {
                        let driver = run.join(format!(
                            "{}-{bs}-{journal}-{range}-{split}",
                            if capi { "capi" } else { "rust" }
                        ));
                        emissions.push(emit(&reference, &base, &driver, capi, journal));
                        assert_eq!(sha(&source), source_hash);
                        manifest(&references, &emissions);
                    }
                }
            }
        }
    }
    assert_eq!((references.len(), emissions.len()), (16, 32));
}
