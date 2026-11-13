//! Test-only native screening. Usage: cargo run --release --example htree_descent_select -- NEW_OUTPUT_DIRECTORY
//! No filesystem oracle is called and no candidate is a correctness verdict.
#[path = "../tests/support/htree_descent_selector.rs"]
mod selector;
use selector::*;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

fn failure(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}

fn git(args: &[&str]) -> io::Result<String> {
    let out = Command::new("git").args(args).output()?;
    if !out.status.success() {
        return Err(failure("git source verification failed"));
    }
    String::from_utf8(out.stdout).map_err(|e| failure(e.to_string()))
}

fn file_sha(path: &Path) -> io::Result<String> {
    // Native streaming digest only; never a filesystem-format oracle tool.
    let out = Command::new("sha256sum").arg("--").arg(path).output()?;
    let text = String::from_utf8(out.stdout).map_err(|e| failure(e.to_string()))?;
    let hash = text.split_whitespace().next().unwrap_or("");
    if !out.status.success() || hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(failure("streaming SHA-256 failed"));
    }
    Ok(hash.into())
}

fn run(dir: &Path, started: Instant) -> io::Result<(u32, usize)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    std::env::set_current_dir(root)?;
    if !git(&["status", "--porcelain", "--untracked-files=normal"])?.is_empty() {
        return Err(failure(
            "source must be committed and clean before screening",
        ));
    }
    let source = git(&["rev-parse", "HEAD"])?;
    let source = source.trim();
    if source.len() != 40 || !source.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(failure("unexpected source commit"));
    }
    let inputs: &[(&str, &[u8])] = &[
        (
            "examples/htree_descent_select.rs",
            include_bytes!("htree_descent_select.rs"),
        ),
        (
            "tests/support/htree_descent_selector.rs",
            include_bytes!("../tests/support/htree_descent_selector.rs"),
        ),
        ("src/hash.rs", include_bytes!("../src/hash.rs")),
        (
            "tests/htree_index_descent_oracle.rs",
            include_bytes!("../tests/htree_index_descent_oracle.rs"),
        ),
    ];
    let mut sources = Vec::new();
    for (path, compiled) in inputs {
        let sha = fs_ext4_test_support::sha256_hex(compiled);
        if fs_ext4_test_support::sha256_hex(&fs::read(root.join(path))?) != sha {
            return Err(failure("compiled selector differs from current source"));
        }
        sources.push(format!("\"{path}\":\"{sha}\""));
    }
    let binary_sha = file_sha(&std::env::current_exe()?)?;
    let names = names();
    let encoded = names_hex(&names);
    if names.len() != 1500 || fs_ext4_test_support::sha256_hex(encoded.as_bytes()) != CORPUS_SHA {
        return Err(failure("original corpus changed"));
    }
    // Original corpus: <256 KiB hex, <=64-byte names. At most 1,500 hash
    // rows/groups per seed and <=1 MiB serialized records. Identity checks
    // use a single 1 MiB vector. These bounds conservatively fit in 8 MiB
    // of live screening data; the ledger is streamed, never accumulated.
    if encoded.len() > 256 * 1024 || names.iter().any(|n| n.len() > 64) {
        return Err(failure("unexpected corpus memory bound"));
    }
    let mut budget = Budget::default();
    let live_bound = 8 * 1024 * 1024;
    budget.check(started.elapsed(), live_bound, 0)?;
    for field in [0, 1] {
        let mut identities = Vec::with_capacity(SEEDS as usize);
        for i in 0..SEEDS {
            let (seed, uuid) = identity(i).map_err(failure)?;
            identities.push(if field == 0 { seed } else { uuid });
            if i % 1024 == 0 {
                budget.check(started.elapsed(), live_bound, 0)?;
            }
        }
        check_identities(&mut identities).map_err(failure)?;
    }
    budget.write(
        &mut File::create_new(dir.join("names.hex"))?,
        encoded.as_bytes(),
        started.elapsed(),
    )?;
    let mut ledger = File::create_new(dir.join("screen.tsv"))?;
    let mut candidates = File::create_new(dir.join("candidates.jsonl"))?;
    let mut specials = File::create_new(dir.join("special.jsonl"))?;
    budget.write(
        &mut ledger,
        b"index\tgroups\tpairs\tspecial_names\n",
        started.elapsed(),
    )?;
    let mut selection = Selection::default();
    let mut special_names = 0usize;
    while !selection.complete() {
        budget.check(started.elapsed(), live_bound, 0)?;
        let index = selection.scanned;
        let scan = analyse(&names, &identity(index).map_err(failure)?.0).map_err(failure)?;
        let pairs: usize = scan
            .groups
            .iter()
            .map(|g| g.len() * (g.len() - 1) / 2)
            .sum();
        let row = format!(
            "{index:05}\t{}\t{pairs}\t{}\n",
            scan.groups.len(),
            scan.special.len()
        );
        budget.write(&mut ledger, row.as_bytes(), started.elapsed())?;
        for (is_special, out) in [(false, &mut candidates), (true, &mut specials)] {
            if if is_special {
                !scan.special.is_empty()
            } else {
                !scan.groups.is_empty()
            } {
                let row = record(index, &scan, &names, is_special);
                if row.len() > 1024 * 1024 {
                    return Err(failure("incomplete: record data limit reached"));
                }
                budget.write(out, row.as_bytes(), started.elapsed())?;
            }
        }
        special_names += scan.special.len();
        selection.consider(index, &scan).map_err(failure)?;
    }
    // Flush/sync all evidence before hashing and publishing the completion marker.
    for file in [&ledger, &candidates, &specials] {
        file.sync_all()?;
    }
    let mut files = Vec::new();
    for name in [
        "names.hex",
        "screen.tsv",
        "candidates.jsonl",
        "special.jsonl",
    ] {
        files.push(format!("\"{name}\":\"{}\"", file_sha(&dir.join(name))?));
    }
    let indices: Vec<_> = selection
        .indices
        .iter()
        .map(|i| format!("\"{i:05}\""))
        .collect();
    let outcome = if selection.indices.len() == TARGET {
        "target_reached"
    } else if selection.indices.is_empty() {
        "no_candidate"
    } else {
        "domain_exhausted"
    };
    let manifest = format!(concat!(
        "{{\n\"schema\":1,\n\"status\":\"complete\",\n\"outcome\":\"{}\",\n",
        "\"source_commit\":\"{}\",\n\"source_sha256\":{{{}}},\n\"binary_sha256\":\"{}\",\n",
        "\"corpus_sha256\":\"{}\",\n\"hash_version\":1,\n\"identity_domain_checked\":65536,\n",
        "\"scanned\":{},\n\"last_index\":\"{:05}\",\n\"candidate_ids\":[{}],\n\"special_names\":{},\n",
        "\"limits\":{{\"seeds\":65536,\"candidates\":16,\"seconds\":300,\"live_data_bytes\":16777216,\"output_bytes\":33554432}},\n",
        "\"files_sha256\":{{{}}}\n}}\n"),
        outcome, source, sources.join(","), binary_sha, CORPUS_SHA, selection.scanned,
        selection.scanned - 1, indices.join(","), special_names, files.join(","));
    // Reserve marker allowance and check the deadline before creating its file.
    budget.check(started.elapsed(), live_bound, manifest.len() + 66)?;
    let mut marker = File::create_new(dir.join("manifest.json"))?;
    publish(
        &mut marker,
        &selection,
        manifest.as_bytes(),
        &mut budget,
        started.elapsed(),
    )?;
    marker.sync_all()?;
    let sha = fs_ext4_test_support::sha256_hex(manifest.as_bytes());
    budget.write(
        &mut File::create_new(dir.join("manifest.sha256"))?,
        format!("{sha}\n").as_bytes(),
        started.elapsed(),
    )?;
    Ok((selection.scanned, selection.indices.len()))
}

fn main() -> io::Result<()> {
    let started = Instant::now();
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err(failure("usage: htree_descent_select NEW_OUTPUT_DIRECTORY"));
    }
    let dir = std::path::absolute(&args[1])?;
    fs::create_dir(&dir)?; // Refuse any existing attempt, including a symlink.
    match run(&dir, started) {
        Ok((scanned, candidates)) => println!(
            "native screening complete: {scanned} seeds, {candidates} candidates, {:.3}s; {}",
            started.elapsed().as_secs_f64(),
            dir.display()
        ),
        Err(error) => {
            // A failed final write/sync must not leave a usable completion marker.
            for name in ["manifest.json", "manifest.sha256"] {
                if dir.join(name).exists() {
                    fs::remove_file(dir.join(name))?;
                }
            }
            fs::write(
                dir.join("incomplete.json"),
                format!(
                    "{{\"status\":\"incomplete\",\"reason_hex\":\"{}\"}}\n",
                    hex::encode(error.to_string())
                ),
            )?;
            return Err(error);
        }
    }
    Ok(())
}
