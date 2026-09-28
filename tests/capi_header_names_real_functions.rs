//! `include/fs_ext4.h` is the only documentation a C caller reads, so a
//! function it names has to be one the library exports (#374).
//!
//! The header told callers that a NULL `cfg->flush` makes "synchronize()" a
//! no-op. No such function existed, and a caller looking for the durability
//! barrier the sentence implied found nothing to call.
//!
//! Two directions are checked, both as text, because a header comment is
//! text and no compiler reads it:
//!
//! - every `name()` or `fs_ext4_name(...)` the header mentions, in a comment
//!   or a declaration, is a function the header declares;
//! - every function the header declares is a `#[no_mangle] extern "C"`
//!   function in `src/capi.rs`, so the declaration links.

use std::collections::BTreeSet;

const HEADER: &str = include_str!("../include/fs_ext4.h");
const CAPI: &str = include_str!("../src/capi.rs");

/// Names the header mentions with a call's parentheses that are not C
/// functions of this library, each with the reason it is allowed.
const NOT_OURS: &[(&str, &str)] = &[(
    "is_writable",
    "a method of the sister crate's device, named to explain RO vs RW",
)];

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Every identifier immediately followed by `(`, with the byte offset of the
/// parenthesis.
fn called_names(text: &str) -> Vec<(String, usize)> {
    let bytes: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    for (i, &c) in bytes.iter().enumerate() {
        if c != '(' || i == 0 || !is_ident(bytes[i - 1]) {
            continue;
        }
        let mut start = i;
        while start > 0 && is_ident(bytes[start - 1]) {
            start -= 1;
        }
        let name: String = bytes[start..i].iter().collect();
        if !name.chars().next().unwrap().is_ascii_digit() {
            out.push((name, i));
        }
    }
    out
}

/// Function names the header declares: an `fs_ext4_*` identifier followed by
/// `(` at the start of a declaration, outside comments.
fn declared(header: &str) -> BTreeSet<String> {
    let mut code = String::new();
    let mut rest = header;
    while let Some(open) = rest.find("/*") {
        code.push_str(&rest[..open]);
        match rest[open..].find("*/") {
            Some(close) => rest = &rest[open + close + 2..],
            None => {
                rest = "";
                break;
            }
        }
    }
    code.push_str(rest);
    let code: String = code
        .lines()
        .map(|l| l.split("//").next().unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    called_names(&code)
        .into_iter()
        .map(|(n, _)| n)
        .filter(|n| n.starts_with("fs_ext4_"))
        .collect()
}

#[test]
fn every_function_the_header_names_is_one_it_declares() {
    let declared = declared(HEADER);
    assert!(
        declared.len() > 30,
        "found only {} declarations; the parser is broken",
        declared.len()
    );
    let mut unknown = BTreeSet::new();
    for (name, _) in called_names(HEADER) {
        let c_like = HEADER.contains(&format!("{name}()")) || name.starts_with("fs_ext4_");
        if !c_like || declared.contains(&name) || NOT_OURS.iter().any(|(n, _)| *n == name) {
            continue;
        }
        unknown.insert(name);
    }
    assert!(
        unknown.is_empty(),
        "include/fs_ext4.h names functions it does not declare: {unknown:?}"
    );
}

#[test]
fn every_function_the_header_declares_is_exported() {
    let missing: Vec<String> = declared(HEADER)
        .into_iter()
        .filter(|name| {
            let sig = format!("extern \"C\" fn {name}(");
            match CAPI.find(&sig) {
                None => true,
                Some(at) => !CAPI[..at]
                    .lines()
                    .rev()
                    .skip(1)
                    .take_while(|l| {
                        l.trim_start().starts_with("//") || l.trim_start().starts_with("#[")
                    })
                    .any(|l| l.trim() == "#[no_mangle]"),
            }
        })
        .collect();
    assert!(
        missing.is_empty(),
        "include/fs_ext4.h declares functions src/capi.rs does not export: {missing:?}"
    );
}
