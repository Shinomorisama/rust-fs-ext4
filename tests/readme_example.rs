//! The README's "Using from Rust" section is checked against the crate.
//!
//! It drifted twice over and nothing noticed: the dependency line asked for
//! `rust-fs-ext4 = "0.5"` with the crate at 0.7, and the example called
//! `Filesystem::mount` with a path and a `stat` method, neither of which
//! exists. The README is text, so nothing compiled it (#469).
//!
//! Now the example is `examples/readme.rs`, which `cargo clippy
//! --all-targets` builds, and this file holds the README to it: the README's
//! ```rust block must be that file below its header, and the ```toml block
//! must ask for the version `Cargo.toml` declares.

use std::path::PathBuf;

fn read(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The body of the first fenced block opened with exactly "```" + `lang`.
fn fenced(markdown: &str, lang: &str) -> String {
    let open = format!("```{lang}");
    let mut lines = markdown.lines().skip_while(|l| l.trim_end() != open);
    assert!(lines.next().is_some(), "README.md has no {open} block");
    let body: Vec<&str> = lines.take_while(|l| l.trim_end() != "```").collect();
    body.join("\n") + "\n"
}

#[test]
fn the_readme_rust_example_is_the_compiled_example() {
    let example = read("examples/readme.rs");
    let code: String = example
        .lines()
        .skip_while(|l| l.starts_with("//"))
        .skip_while(|l| l.is_empty())
        .map(|l| format!("{l}\n"))
        .collect();
    assert_eq!(
        fenced(&read("README.md"), "rust"),
        code,
        "README.md's ```rust block differs from examples/readme.rs below its header; \
         change both together"
    );
}

#[test]
fn the_readme_asks_for_the_version_cargo_toml_declares() {
    let manifest = read("Cargo.toml");
    let version = manifest
        .lines()
        .find_map(|l| l.strip_prefix("version = \""))
        .and_then(|v| v.strip_suffix('"'))
        .expect("Cargo.toml declares a version");
    let mut parts = version.split('.');
    let (major, minor) = (parts.next().unwrap(), parts.next().unwrap());
    // A 0.x crate's compatibility boundary is the minor, so "0.7" is the
    // requirement a user should write; a 1.x crate's would be "1".
    let want = if major == "0" {
        format!("0.{minor}")
    } else {
        major.to_string()
    };
    let toml = fenced(&read("README.md"), "toml");
    assert!(
        toml.contains(&format!("rust-fs-ext4 = \"{want}\"")),
        "README.md's ```toml block should ask for rust-fs-ext4 = \"{want}\" (Cargo.toml is \
         {version}), and says:\n{toml}"
    );
}
