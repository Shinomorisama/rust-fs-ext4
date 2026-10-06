//! `rust-fs-ext4`: the command-line tools for ext4, one multi-call binary.
//!
//! Installed as `rust-fs-ext4` and linked as each dotted name. The
//! dispatch and the output contract every tool shares are `fs_core::cli`
//! (rust-fs-core's `cli` feature); `ext4` is the tools themselves.

mod ext4;

use fs_core::cli;
use std::process::ExitCode;

static FAMILY: cli::Family = cli::Family {
    repo: "rust-fs-ext4",
    crate_name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    about: "ext4 tools: work on an ext4 image or device directly, without mounting it",
    install_hints: &[
        "`chore cli:install` from a checkout of this repository",
        "`brew install antimatter-studios/tap/rust-fs-ext4`",
    ],
    tools: &[ext4::mkfs::TOOL, ext4::fsck::TOOL, ext4::fs::TOOL],
};

fn main() -> ExitCode {
    cli::main(&FAMILY)
}
