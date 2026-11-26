//! Grow and measure a checksum-v3 reference image in the pinned harness guest.

use std::{fs, path::Path, process::Command};

#[allow(dead_code)]
#[path = "../tests/support/src/verdict.rs"]
mod verdict;

use verdict::{Judge, Verdict};

const VM: &str = "../fs-linux-test-harness/scripts/vm.sh";
const REPORTS: &str = "tmp/checked-io-reports";
const GUEST: &str = r#"
set -eu
export LC_ALL=C
reports=/repo/tmp/checked-io-reports
scratch=$(mktemp -d)
trap 'rm -rf -- "$scratch"' EXIT
report() {
    tag="$1"
    shift
    status=0
    "$@" > "$reports/$tag.stdout" 2> "$reports/$tag.stderr" || status=$?
    printf '%s\n' "$status" > "$reports/$tag.status"
    test "$status" -eq 0
}
install -m 755 /repo/tmp/checked-io-probe "$scratch/probe"
cd "$scratch"
cp --sparse=always /repo/tmp/checked-io-input.img small.img
cp --sparse=always /repo/tmp/checked-io-input.img large.img
truncate -s 9G large.img
report resize resize2fs large.img
for size in small large; do
    image="$size.img"
    report "$size-header" dumpe2fs -h "$image"
    report "$size-root" debugfs -R 'ls -p /' "$image"
    report "$size-before" e2fsck -fn "$image"
    report "$size-allocated-before" stat -c '%s %b' "$image"
    test "$(stat -c '%b' "$image")" -lt 1048576
    report "$size-measure" ./probe --working-copy "$image"
    report "$size-allocated-after" stat -c '%s %b' "$image"
    test "$(stat -c '%b' "$image")" -lt 1048576
    report "$size-after" e2fsck -fn "$image"
    report "$size-created" debugfs -R 'stat /checked-io-probe' "$image"
    report "$size-oracle" debugfs -R "dump /oracle $reports/$size-oracle.bin" "$image"
done
"#;

fn read_report(dir: &Path, tag: &str, judge: Judge) -> String {
    let status = fs::read_to_string(dir.join(format!("{tag}.status"))).unwrap();
    let code = status.trim().parse::<i32>().unwrap();
    let stdout = fs::read_to_string(dir.join(format!("{tag}.stdout"))).unwrap();
    let stderr = fs::read_to_string(dir.join(format!("{tag}.stderr"))).unwrap();
    assert_eq!(
        judge.read(Some(code), &stdout, &stderr),
        Verdict::Clean,
        "{tag}"
    );
    stdout
}

fn verify(dir: &Path) {
    let expected = fs::read(dir.join("expected-oracle.bin")).unwrap();
    assert_eq!(expected.len(), 4096);
    let mut roots = Vec::new();
    for (size, blocks, groups) in [("small", 16384, 1), ("large", 2359296, 72)] {
        let header = read_report(dir, &format!("{size}-header"), Judge::Dumpe2fs);
        let field = |name| {
            header
                .lines()
                .find_map(|line| line.strip_prefix(name))
                .unwrap_or_else(|| panic!("{size} {name} is missing"))
                .trim()
        };
        assert_eq!(field("Group descriptor size:"), "64");
        assert_eq!(field("Block size:"), "4096");
        assert_eq!(field("Blocks per group:"), "32768");
        assert_eq!(field("Block count:"), blocks.to_string());
        assert!(!field("Filesystem features:")
            .split_whitespace()
            .any(|f| f == "meta_bg"));
        roots.push(read_report(dir, &format!("{size}-root"), Judge::Debugfs));
        for when in ["before", "after"] {
            read_report(dir, &format!("{size}-{when}"), Judge::E2fsck);
        }
        let created = read_report(dir, &format!("{size}-created"), Judge::Debugfs);
        assert!(created.contains("Inode: 13") && created.contains("Type: regular"));
        read_report(dir, &format!("{size}-oracle"), Judge::Debugfs);
        assert_eq!(
            fs::read(dir.join(format!("{size}-oracle.bin"))).unwrap(),
            expected
        );
        assert_eq!(
            fs::read_to_string(dir.join(format!("{size}-measure.status")))
                .unwrap()
                .trim(),
            "0"
        );
        let counts = fs::read_to_string(dir.join(format!("{size}-measure.stdout"))).unwrap();
        let lines: Vec<_> = counts.lines().collect();
        assert_eq!(lines.len(), 5);
        assert_eq!(
            lines[0],
            format!("image: block_size=4096 blocks={blocks} blocks_per_group=32768 groups={groups} allocated_inodes=12")
        );
        for (line, phase) in lines[1..4].iter().zip(["mount:", "create:", "finish:"]) {
            assert_eq!(line.split_whitespace().next(), Some(phase));
        }
        assert_eq!(lines[4], "readback: inode=13 size=0 journal=clean");
        println!("{size} fixture\n{counts}");
    }
    assert!(!roots[0].is_empty() && roots[0] == roots[1]);
    println!("Both root listings matched, both payloads matched, and all four checks were clean.");
}

fn main() {
    assert!(Command::new(VM).arg("up").status().unwrap().success());
    assert!(Command::new(VM)
        .args(["exec", GUEST])
        .status()
        .unwrap()
        .success());
    verify(Path::new(REPORTS));
}
