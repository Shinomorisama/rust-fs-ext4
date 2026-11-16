//! Prepared two-node guest protocol. Matrix execution and qualification are pending.
use super::*;
use std::path::PathBuf;

use crate::fs::guarded_chmod::two_node_query as query;

const GUEST: &str = include_str!("link_index_two_node_guest.py");

struct Prepared {
    input: PathBuf,
    before: Snapshot,
    query: query::Selection,
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
    Prepared {
        input: base.with_extension("two-node-input.img"),
        before,
        query,
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
        "NAMES = [{names}]\nQUERY_HEX = \"{query_hex}\"\nSOURCE = {}\nEXPECTED = [{}]\nFLAGS = {:?}\nFREE = ({}, {})\n",
        state.view.records[1].0,
        TRACKED.iter().zip(&state.view.records).map(|(p, (ino, i, _))| literal(p, *ino, i)).collect::<Vec<_>>().join(","),
        state.view.records.iter().map(|r| r.1.flags).collect::<Vec<_>>(),
        state.view.free.0, state.view.free.1,
    );
    std::fs::write(
        base.with_extension(format!("{label}.expected.py")),
        &expected,
    )
    .unwrap();
    let out = guest_kernel_write(image.to_str().unwrap(), &format!(
        "python3 - \"$MNT\" {mode} <<'TWO_NODE_REFERENCE'\n{expected}\n{GUEST}\nTWO_NODE_REFERENCE\n"
    ));
    std::fs::write(base.with_extension(format!("{label}.jsonl")), &out.stdout).unwrap();
    std::fs::write(
        base.with_extension(format!("{label}.stderr.txt")),
        &out.stderr,
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
