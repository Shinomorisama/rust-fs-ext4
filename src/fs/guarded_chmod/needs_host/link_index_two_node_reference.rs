//! Independent Linux two-node reference; Rust/C emission remains separate.
use super::*;
use std::path::PathBuf;

use crate::fs::guarded_chmod::two_node_query as query;

#[path = "link_index_two_node_emission.rs"]
mod emissions;

const GUEST: &str = include_str!("link_index_two_node_guest.py");

struct Prepared {
    input: PathBuf,
    before: Snapshot,
    query: query::Selection,
    input_hash: String,
}

fn sha(path: &Path) -> String {
    fs_ext4_test_support::sha256_hex(&std::fs::read(path).unwrap())
}

fn capture(image: &Path, base: &Path, phase: &str) -> Snapshot {
    // Keep the image before any assertion/report can fail.
    std::fs::copy(image, base.with_extension(format!("{phase}.img"))).unwrap();
    std::fs::write(base.with_extension(format!("{phase}.sha256")), sha(image)).unwrap();
    let state = observe(image);
    let out = oracle("e2fsck").arg("-fn").arg(image).judged();
    std::fs::write(
        base.with_extension(format!("{phase}.e2fsck.txt")),
        &out.output.stdout,
    )
    .unwrap();
    std::fs::write(
        base.with_extension(format!("{phase}.e2fsck.stderr.txt")),
        &out.output.stderr,
    )
    .unwrap();
    std::fs::write(
        base.with_extension(format!("{phase}.e2fsck.status.txt")),
        format!("{:?}\n", out.output.status.code()),
    )
    .unwrap();
    out.clean("Linux two-node reference");
    report(image, base, phase, &state.view.records);
    for (n, (_, _, raw)) in state.view.records.iter().enumerate() {
        std::fs::write(base.with_extension(format!("{phase}.{n}.inode.bin")), raw).unwrap();
    }
    let out = oracle("debugfs")
        .args(["-R", "htree_dump /policy"])
        .arg(image)
        .output();
    std::fs::write(
        base.with_extension(format!("{phase}.htree.stderr.txt")),
        &out.stderr,
    )
    .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("Root node dump") && text.contains("Number of entries"));
    std::fs::write(base.with_extension(format!("{phase}.htree.txt")), text).unwrap();
    let rows: Vec<_> = state
        .view
        .entries
        .iter()
        .map(|(name, ino)| {
            format!(
                "[\"{}\",{ino}]",
                name.iter().map(|b| format!("{b:02x}")).collect::<String>()
            )
        })
        .collect();
    std::fs::write(base.with_extension(format!("{phase}.inventory.json")),format!("{{\"depth\":{},\"root_count\":{},\"root_limit\":{},\"index_nodes\":{:?},\"leaves\":{:?},\"mapping\":{:?},\"extent_nodes\":{:?},\"entries\":[{}]}}\n",state.depth,state.view.roots,state.limit,state.index_nodes,state.leaves,state.view.mapping,state.map_nodes.iter().map(|n|n.0).collect::<Vec<_>>(),rows.join(","))).unwrap();
    state
}

fn prepare(image: &Path, base: &Path, range: usize, split: bool) -> Prepared {
    // This distinct base must identify layout, root range and room/split case.
    let before = capture(image, base, "two-node-input");
    assert_eq!((before.depth, before.view.roots), (1, 2));
    let fs = mount(image);
    let search = query::select(
        &fs,
        before.view.records[0].0,
        before.view.records[1].0,
        range,
        split,
        0..query::MAX_CANDIDATES,
    )
    .unwrap();
    std::fs::write(base.with_extension("two-node-search.jsonl"), search.trace).unwrap();
    let query = search
        .selected
        .expect("bounded two-node query search exhausted; trace retained");
    std::fs::write(base.with_extension("two-node-query.json"), query.json()).unwrap();
    let input = base.with_extension("two-node-input.img");
    let input_hash = sha(&input);
    std::fs::write(base.with_extension("two-node-input.sha256"), &input_hash).unwrap();
    Prepared {
        input,
        before,
        query,
        input_hash,
    }
}

fn guest(
    image: &Path,
    base: &Path,
    label: &str,
    mode: &str,
    prepared: &Prepared,
    state: &Snapshot,
) {
    assert!(["audit", "link", "cold"].contains(&mode));
    let names = state
        .view
        .entries
        .iter()
        .map(|(n, _)| {
            format!(
                "\"{}\"",
                n.iter().map(|b| format!("{b:02x}")).collect::<String>()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let query_hex = prepared
        .query
        .name
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let expected = format!(
        "NAMES = [{names}]\nQUERY_HEX = \"{query_hex}\"\nSOURCE = {}\nEXPECTED = [{}]\nFLAGS = {:?}\nFREE = ({}, {})\nSPLIT = {}\nBLOCK_SIZE = {}\n",
        state.view.records[1].0,
        TRACKED.iter().zip(&state.view.records).map(|(p, (ino, i, _))| literal(p, *ino, i)).collect::<Vec<_>>().join(","),
        state.view.records.iter().map(|r| r.1.flags).collect::<Vec<_>>(),
        state.view.free.0, state.view.free.1,
        if prepared.query.split {"True"} else {"False"}, state.view.directory[0].len(),
    );
    std::fs::write(
        base.with_extension(format!("{label}.expected.py")),
        &expected,
    )
    .unwrap();
    // A complete explicit namespace already exceeds Linux's single-argument
    // limit in the smallest profile. The guest sees repository files at the
    // same path; keep bulk data in a retained program, not the shell argument.
    let program = base.with_extension(format!("{label}.guest.py"));
    std::fs::write(&program, format!("{expected}\n{GUEST}")).unwrap();
    let script = format!(
        "python3 {} \"$MNT\" {mode}\n",
        fs_ext4_test_support::guest_quote(program.to_str().unwrap())
    );
    let out = if mode == "link" {
        guest_kernel_write(image.to_str().unwrap(), &script)
    } else {
        fs_ext4_test_support::guest_kernel_read(image.to_str().unwrap(), &script)
    };
    std::fs::write(base.with_extension(format!("{label}.jsonl")), &out.stdout).unwrap();
    std::fs::write(
        base.with_extension(format!("{label}.stderr.txt")),
        &out.stderr,
    )
    .unwrap();
    std::fs::write(
        base.with_extension(format!("{label}.status.txt")),
        format!("{:?}\n", out.status.code()),
    )
    .unwrap();
    assert!(
        out.status.success(),
        "two-node {label}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|s| s.starts_with('{'))
            .count(),
        1
    );
}

// This independent answer binds the exact immutable input, effective version,
// unchanged seed and selected raw name before Linux gets its writable copy.
fn independent_hash(prepared: &Prepared, base: &Path) {
    let fs = mount(&prepared.input);
    let bytes: Vec<_> = fs
        .sb
        .hash_seed
        .iter()
        .flat_map(|w| w.to_le_bytes())
        .collect();
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let seed = format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    );
    let name = std::str::from_utf8(&prepared.query.name).unwrap();
    assert!(name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'));
    let request = format!(
        "dx_hash -h {} -s {seed} -- \"{name}\"",
        prepared.query.version
    );
    std::fs::write(base.with_extension("hash.request.txt"), &request).unwrap();
    let out = oracle("debugfs")
        .args(["-R", &request])
        .arg(&prepared.input)
        .output();
    std::fs::write(base.with_extension("hash.stdout.txt"), &out.stdout).unwrap();
    std::fs::write(base.with_extension("hash.stderr.txt"), &out.stderr).unwrap();
    std::fs::write(
        base.with_extension("hash.status.txt"),
        format!("{:?}\n", out.status.code()),
    )
    .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert_eq!(text.lines().count(), 1);
    let answer = text
        .trim_end()
        .strip_prefix(&format!("Hash of {name} is 0x"))
        .unwrap();
    let (major, minor) = answer.split_once(" (minor 0x").unwrap();
    let major = u32::from_str_radix(major, 16).unwrap();
    let minor = u32::from_str_radix(minor.strip_suffix(')').unwrap(), 16).unwrap();
    let major = if major == 0xffff_fffe {
        0xffff_fffc
    } else {
        major
    };
    assert_eq!((major, minor), (prepared.query.major, prepared.query.minor));
    assert_eq!(sha(&prepared.input), prepared.input_hash);
}

fn format_identity(input: &Path, output: &Path) {
    let a = &mount(input).sb;
    let b = &mount(output).sb;
    assert_eq!(a.unsigned_hash(), b.unsigned_hash());
    assert_eq!(
        (a.uuid, a.hash_seed, a.default_hash_version, a.checksum_seed),
        (b.uuid, b.hash_seed, b.default_hash_version, b.checksum_seed)
    );
    assert_eq!(
        (
            a.feature_compat,
            a.feature_incompat,
            a.feature_ro_compat,
            a.journal_inode
        ),
        (
            b.feature_compat,
            b.feature_incompat,
            b.feature_ro_compat,
            b.journal_inode
        )
    );
    assert_eq!(
        (
            a.blocks_count,
            a.inodes_count,
            a.log_block_size,
            a.inode_size,
            a.desc_size,
            a.first_data_block,
            a.blocks_per_group,
            a.inodes_per_group
        ),
        (
            b.blocks_count,
            b.inodes_count,
            b.log_block_size,
            b.inode_size,
            b.desc_size,
            b.first_data_block,
            b.blocks_per_group,
            b.inodes_per_group
        )
    );
}

fn transition(prepared: &Prepared, after: &Snapshot, image: &Path) -> usize {
    use crate::fs::guarded_chmod::two_node_shape as shape;
    let before = &prepared.before;
    let q = &prepared.query;
    let (a, b) = (&before.view, &after.view);
    let fs = mount(image);
    let bs = fs.sb.block_size();
    assert_eq!((before.depth, after.depth, a.roots, b.roots), (1, 1, 2, 2));
    assert_eq!(before.index_nodes, after.index_nodes);
    let appended = shape::check(
        shape::View {
            blocks: &a.directory,
            mapping: &a.mapping,
        },
        shape::View {
            blocks: &b.directory,
            mapping: &b.mapping,
        },
        shape::Case {
            root_slot: q.root_slot,
            leaf: q.leaf,
            major: q.major,
            name: &q.name,
            split: q.split,
        },
    )
    .unwrap();
    assert_eq!(appended.is_some(), q.split);
    assert_eq!(
        after.leaves.len(),
        before.leaves.len() + usize::from(q.split)
    );
    assert!(before.leaves.iter().all(|n| after.leaves.contains(n)));
    let mut names = a.entries.clone();
    names.push((q.name.clone(), a.records[1].0));
    names.sort();
    assert_eq!(b.entries, names);
    assert!(names.windows(2).all(|w| w[0].0 != w[1].0));
    let mut added: Vec<_> = appended.into_iter().collect();
    for (p, raw) in &before.map_nodes {
        let new = after.map_nodes.iter().find(|n| n.0 == *p).unwrap();
        // Appending may extend the final extent leaf; every other old extent
        // leaf must retain its exact bytes as well as its physical address.
        if !q.split || Some(*p) != before.map_nodes.last().map(|n| n.0) {
            assert_eq!(raw, &new.1);
        }
    }
    added.extend(
        after
            .map_nodes
            .iter()
            .filter(|n| !before.map_nodes.iter().any(|old| old.0 == n.0))
            .map(|n| n.0),
    );
    assert!(if q.split {
        (1..=2).contains(&added.len())
    } else {
        added.is_empty()
    });
    assert_eq!(b.free, (a.free.0 - added.len() as u64, a.free.1));
    assert_eq!(
        b.records[0].1.size,
        a.records[0].1.size + u64::from(bs) * u64::from(q.split)
    );
    assert_eq!(
        b.records[0].1.blocks,
        a.records[0].1.blocks + added.len() as u64 * u64::from(bs / 512)
    );
    for n in 0..3 {
        assert_eq!(a.records[n].0, b.records[n].0);
    }
    assert_eq!(b.records[0].1.links_count, a.records[0].1.links_count);
    assert_eq!(b.records[1].1.links_count, a.records[1].1.links_count + 1);
    assert_eq!(version(&b.records[0].2), version(&a.records[0].2) + 1);
    stable(
        &a.records[1].2,
        &b.records[1].2,
        &[(12, 4), (26, 2), (124, 2), (130, 2), (132, 4)],
    );
    let mut allowed = vec![(12, 8), (36, 4), (124, 2), (130, 2), (132, 8), (152, 4)];
    if q.split {
        allowed.extend([(4, 4), (28, 4), (40, 60), (108, 4), (116, 2)]);
    }
    stable(&a.records[0].2, &b.records[0].2, &allowed);
    assert_eq!(a.records[2].2, b.records[2].2);
    assert_eq!(a.eas, b.eas);
    assert_eq!(a.payloads, b.payloads);
    assert_eq!(a.root_directory, b.root_directory);
    let locations: Vec<_> = a.records[..2]
        .iter()
        .map(|(ino, _, _)| crate::bgd::locate_inode(&fs.sb, &fs.groups, *ino).unwrap())
        .collect();
    assert_eq!(a.tables.len(), b.tables.len());
    for ((block, x), (other, y)) in a.tables.iter().zip(&b.tables) {
        assert_eq!(block, other);
        let allowed: Vec<_> = locations
            .iter()
            .filter(|(p, _)| p == block)
            .map(|(_, off)| (*off as usize, usize::from(fs.sb.inode_size)))
            .collect();
        stable(x, y, &allowed);
    }
    assert_eq!(a.allocation.len(), b.allocation.len());
    for group in 0..fs.groups.len() {
        let row = group * 3;
        let mut bitmap = a.allocation[row + 2].clone();
        let mut debit = 0;
        for p in &added {
            let rel = *p - u64::from(fs.sb.first_data_block);
            if rel / u64::from(fs.sb.blocks_per_group) == group as u64 {
                let bit = rel % u64::from(fs.sb.blocks_per_group);
                let byte = (bit / 8) as usize;
                let mask = 1u8 << (bit % 8);
                assert_eq!(bitmap[byte] & mask, 0);
                bitmap[byte] |= mask;
                debit += 1;
            }
        }
        assert_eq!(bitmap, b.allocation[row + 2]);
        assert_eq!(a.allocation[row + 1], b.allocation[row + 1]);
        let (block, offset) = fs.sb.descriptor_location(group as u64);
        let old = crate::bgd::BlockGroupDescriptor::parse(
            &a.allocation[row][offset..offset + usize::from(fs.sb.desc_size)],
            fs.sb.desc_size,
        )
        .unwrap();
        let new = crate::bgd::BlockGroupDescriptor::parse(
            &b.allocation[row][offset..offset + usize::from(fs.sb.desc_size)],
            fs.sb.desc_size,
        )
        .unwrap();
        assert_eq!(new.free_blocks_count, old.free_blocks_count - debit);
        let mut allowed = Vec::new();
        for n in 0..fs.groups.len() {
            let (nr, at) = fs.sb.descriptor_location(n as u64);
            if nr == block
                && added.iter().any(|p| {
                    (*p - u64::from(fs.sb.first_data_block)) / u64::from(fs.sb.blocks_per_group)
                        == n as u64
                })
            {
                allowed.extend([
                    (at + 12, 2),
                    (at + 24, 2),
                    (at + 30, 2),
                    (at + 44, 2),
                    (at + 56, 2),
                ]);
            }
        }
        stable(&a.allocation[row], &b.allocation[row], &allowed);
    }
    format_identity(&prepared.input, image);
    added.len()
}

struct CompletedCase {
    prepared: Prepared,
    linux: Snapshot,
    row: String,
}

fn run_case(source: &Path, base: &Path, range: usize, split: bool) -> CompletedCase {
    let working = base.with_extension("working.img");
    std::fs::copy(source, &working).unwrap();
    let prepared = prepare(&working, base, range, split);
    assert_eq!(sha(&working), prepared.input_hash);
    independent_hash(&prepared, base);
    guest(
        &prepared.input,
        base,
        "audit",
        "audit",
        &prepared,
        &prepared.before,
    );
    assert_eq!(sha(&prepared.input), prepared.input_hash);
    guest(&working, base, "link", "link", &prepared, &prepared.before);
    // capture() keeps output bytes before any geometry or accounting assertion.
    let after = capture(&working, base, "two-node-output");
    let allocated = transition(&prepared, &after, &working);
    let output_hash = sha(&base.with_extension("two-node-output.img"));
    assert_eq!(
        sha(&working),
        output_hash,
        "output observers changed Linux bytes"
    );
    let cold = base.with_extension("cold.img");
    std::fs::copy(&working, &cold).unwrap();
    guest(&cold, base, "cold", "cold", &prepared, &after);
    let observed = capture(&cold, base, "two-node-cold");
    assert_eq!(after.view.entries, observed.view.entries);
    assert_eq!(after.view.directory, observed.view.directory);
    assert_eq!(after.view.tables, observed.view.tables);
    assert_eq!(after.view.allocation, observed.view.allocation);
    assert_eq!(after.map_nodes, observed.map_nodes);
    assert_eq!(sha(&cold), output_hash);
    assert_eq!(sha(&working), output_hash);
    assert_eq!(sha(&prepared.input), prepared.input_hash);
    std::fs::write(base.with_extension("two-node-output.sha256"), &output_hash).unwrap();
    std::fs::write(base.with_extension("two-node-cold.sha256"), sha(&cold)).unwrap();
    std::fs::remove_file(working).unwrap();
    std::fs::remove_file(cold).unwrap();
    let row = format!("{{\"case\":\"{}\",\"root_slot\":{range},\"split\":{split},\"input_sha256\":\"{}\",\"output_sha256\":\"{output_hash}\",\"allocated\":{allocated},\"query\":{}}}",base.file_name().unwrap().to_str().unwrap(),prepared.input_hash,prepared.query.json().trim());
    CompletedCase {
        prepared,
        linux: after,
        row,
    }
}

fn case(source: &Path, base: &Path, range: usize, split: bool) -> String {
    run_case(source, base, range, split).row
}

fn retain_bootstrap(prefix: &str, destination: &Path) {
    for entry in std::fs::read_dir(fs_ext4_test_support::temp_dir()).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name().to_str().unwrap().starts_with(prefix) {
            assert!(entry.file_type().unwrap().is_file());
            std::fs::rename(entry.path(), destination.join(entry.file_name())).unwrap();
        }
    }
}

#[cfg(test)]
#[test]
fn linux_records_two_node_indexed_link_reference() {
    // Outside scripts/test.sh's owned scratch, so failures survive cleanup in
    // focused, native whole-suite and suite-in-VM paths alike.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tmp/link-index-two-node-reference");
    let run = fs_ext4_test_support::create_unique_temp_dir(&root).unwrap();
    let mut completed = Vec::new();
    let manifest = |rows: &[String]| {
        std::fs::write(
            run.join("matrix.json"),
            format!("{{\"expected\":16,\"completed\":[{}]}}\n", rows.join(",")),
        )
        .unwrap();
    };
    manifest(&completed);
    for bs in [1024, 4096] {
        for journal in [false, true] {
            let prefix = format!("two-node-bootstrap-{}-{bs}-{journal}", std::process::id());
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
                    let base = run.join(format!("case-{bs}-{journal}-{range}-{split}"));
                    std::fs::write(base.with_extension("source.json"),format!("{{\"file\":\"{}\",\"sha256\":\"{source_hash}\",\"block_size\":{bs},\"journal\":{journal},\"root_slot\":{range},\"split\":{split}}}\n",source.file_name().unwrap().to_str().unwrap())).unwrap();
                    completed.push(case(&source, &base, range, split));
                    assert_eq!(sha(&source), source_hash);
                    manifest(&completed);
                }
            }
        }
    }
    assert_eq!(completed.len(), 16);
}
