#![no_main]
//! The jbd2 journal superblock: `JournalSuperblock::parse` and nothing
//! else. The journal's block headers, descriptor tags and revoke records
//! are parsed privately, during replay, and are reached only through the
//! `image` target, which mounts whole images.
//!
//! A journal is read at mount, before the filesystem has been
//! established, and it is a structure the format expects to be
//! partially written -- so it is parsed with a corruption tolerance the
//! other structures do not have, which is exactly what a fuzzer abuses.
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = fs_ext4::jbd2::JournalSuperblock::parse(data);
});
