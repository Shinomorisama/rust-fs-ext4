//! Count backing-device requests for an ordinary checked image lifecycle.
//!
//! Run with `cargo run --release --locked --example checked_recovery_io -- INPUT NEW_COPY`.
//! The input image is read-only. NEW_COPY must not already exist.
//! The counts exclude copying the input and reopening the result for readback.

use fs_ext4::block_io::{BlockDevice, FileDevice};
use fs_ext4::features::Incompat;
use fs_ext4::fs::Filesystem;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};

const CREATED_PATH: &str = "/checked-io-probe";

#[derive(Default)]
struct Counts {
    reads: u64,
    read_bytes: u64,
    writes: u64,
    written_bytes: u64,
    flushes: u64,
}

struct Counted {
    file: FileDevice,
    counts: Mutex<Counts>,
}

impl Counted {
    fn report_and_reset(&self, phase: &str) {
        let counts = std::mem::take(&mut *self.counts.lock().unwrap());
        println!(
            "{phase}: reads={} read_bytes={} writes={} written_bytes={} flushes={}",
            counts.reads, counts.read_bytes, counts.writes, counts.written_bytes, counts.flushes
        );
    }
}

impl BlockDevice for Counted {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> fs_ext4::Result<()> {
        {
            let mut counts = self.counts.lock().unwrap();
            counts.reads += 1;
            counts.read_bytes += buf.len() as u64;
        }
        self.file.read_at(offset, buf)
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> fs_ext4::Result<()> {
        {
            let mut counts = self.counts.lock().unwrap();
            counts.writes += 1;
            counts.written_bytes += buf.len() as u64;
        }
        self.file.write_at(offset, buf)
    }

    fn flush(&self) -> fs_ext4::Result<()> {
        self.counts.lock().unwrap().flushes += 1;
        self.file.flush()
    }

    fn size_bytes(&self) -> u64 {
        self.file.size_bytes()
    }

    fn is_writable(&self) -> bool {
        self.file.is_writable()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let [input, copy] = args.as_slice() else {
        return Err("usage: checked_recovery_io INPUT NEW_COPY".into());
    };
    let input = Path::new(input);
    let copy = Path::new(copy);
    if !input.is_file() {
        return Err("INPUT must be a regular image".into());
    }
    let original = Filesystem::mount_lazy(Arc::new(FileDevice::open_path(input)?))?;
    let sb = &original.sb;
    if sb.block_size() != 4096
        || sb.feature_incompat & (Incompat::CASEFOLD | Incompat::RECOVER).bits() != 0
        || !sb.is_clean()
        || !fs_ext4::jbd2::read_superblock(&original)?
            .is_some_and(|journal| journal.uses_csum_v3() && journal.is_clean())
    {
        return Err("INPUT must be a clean ordinary 4 KiB checksum-v3 image".into());
    }
    let allocated_inodes = sb
        .inodes_count
        .checked_sub(sb.free_inodes_count)
        .ok_or("Free inode count exceeds inode capacity")?;
    println!(
        "image: block_size={} blocks={} blocks_per_group={} groups={} allocated_inodes={}",
        sb.block_size(),
        sb.blocks_count,
        sb.blocks_per_group,
        sb.block_group_count(),
        allocated_inodes
    );
    drop(original);
    {
        let mut source = File::open(input)?;
        let mut working = File::create_new(copy)?;
        std::io::copy(&mut source, &mut working)?;
        working.sync_all()?;
    }
    let counted = Arc::new(Counted {
        file: FileDevice::open_path_rw(copy)?,
        counts: Mutex::default(),
    });
    let fs = Filesystem::mount_recovering(counted.clone())?;
    counted.report_and_reset("mount");
    let inode = fs.apply_create(CREATED_PATH, 0o644)?;
    counted.report_and_reset("create");
    fs.finish()?;
    counted.report_and_reset("finish");
    drop(counted);

    let reopened = Filesystem::mount(Arc::new(FileDevice::open_path(copy)?))?;
    let found = reopened.lookup_path_bytes(CREATED_PATH.as_bytes())?;
    let (created, _) = reopened.read_inode_verified(found)?;
    let journal = fs_ext4::jbd2::read_superblock(&reopened)?;
    assert_eq!(found, inode);
    assert!(created.is_file());
    assert_eq!(created.size, 0);
    assert!(reopened.sb.is_clean());
    assert_eq!(reopened.sb.feature_incompat & Incompat::RECOVER.bits(), 0);
    assert!(journal.is_some_and(|journal| journal.uses_csum_v3() && journal.is_clean()));
    println!(
        "readback: inode={found} size={} journal=clean",
        created.size
    );
    Ok(())
}
