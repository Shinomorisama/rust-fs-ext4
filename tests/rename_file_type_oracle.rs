//! A renamed special file keeps its directory entry's file type (#386).
//!
//! `apply_rename` mapped the inode's mode to the entry's type byte knowing
//! only regular files, directories and symlinks, so a renamed FIFO, socket
//! or device node was filed under type 0 (unknown). On a `filetype` volume
//! the entry then disagrees with its inode, and e2fsck pass 2 says so
//! ("Setting filetype for entry ... to 5."). The volume comes from
//! `mkfs.ext4`, the driver makes and renames one of each kind -- to a new
//! name and over an existing file -- and e2fsck must find nothing to fix.
//!
//! `e2fsck -fy`, not `-fn`: `e2fsck -n` does not report an entry whose type
//! byte is 0, and calls this volume clean. Under `-y` it fixes the entry,
//! says so and exits 1. The image is this test's own scratch copy. Fails without
//! e2fsprogs (they run in the harness VM).

use fs_ext4::block_io::FileDevice;
use fs_ext4::fs::Filesystem;
use fs_ext4_test_support::oracle;
use std::sync::Arc;

fn renames_keep_the_type(tag: &str, features: &str) {
    let image = fs_ext4_test_support::temp_path!("fs_ext4_386_{tag}_{}.img", std::process::id());
    std::fs::File::create(&image)
        .and_then(|f| f.set_len(32 * 1024 * 1024))
        .unwrap();
    let out = oracle("mkfs.ext4")
        .args(["-q", "-F", "-O", features, &image])
        .output();
    assert!(
        out.status.success(),
        "mkfs.ext4: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    {
        let fs = Filesystem::mount(Arc::new(FileDevice::open_rw(&image).unwrap())).unwrap();
        fs.apply_mkdir("/elsewhere", 0o755).unwrap();
        for (name, mode) in [
            ("fifo", 0o010644u16),
            ("sock", 0o140644),
            ("chr", 0o020644),
            ("blk", 0o060644),
        ] {
            fs.apply_mknod(&format!("/{name}"), mode, 1, 3).unwrap();
            fs.apply_rename(&format!("/{name}"), &format!("/elsewhere/{name}"), false)
                .expect("rename to a new name");
            fs.apply_create(&format!("/{name}_victim"), 0o644).unwrap();
            fs.apply_rename(
                &format!("/elsewhere/{name}"),
                &format!("/{name}_victim"),
                true,
            )
            .expect("rename over a file");
        }
    }
    oracle("e2fsck")
        .args(["-fy", &image])
        .judged()
        .clean(&format!("{tag}: renamed special files"));
    let _ = std::fs::remove_file(&image);
}

#[test]
fn renamed_special_files_keep_their_type_with_metadata_csum() {
    renames_keep_the_type("csum", "metadata_csum");
}

#[test]
fn renamed_special_files_keep_their_type_without_metadata_csum() {
    renames_keep_the_type("nocsum", "^metadata_csum");
}
