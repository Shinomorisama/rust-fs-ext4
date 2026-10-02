// The README's "Using from Rust" example, verbatim below this header.
//
// It lives here so that it is compiled: the README's copy is text, and it
// showed `Filesystem::mount` taking a path and a `stat` method, neither of
// which exists, for releases with nothing to notice (#469). `cargo clippy
// --all-targets` builds this file, and tests/readme_example.rs fails if the
// README's block and the code below this header differ.
//
//   cargo run --example readme
use std::sync::Arc;

use fs_ext4::block_io::FileDevice;
use fs_ext4::Filesystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fs = Filesystem::mount(Arc::new(FileDevice::open("/path/to/disk.img")?))?;
    let ino = fs.lookup_path_bytes(b"/hello.txt")?;
    let attrs = fs.stat_ino(ino)?;
    println!("size={} mode={:o}", attrs.size, attrs.mode);
    Ok(())
}
