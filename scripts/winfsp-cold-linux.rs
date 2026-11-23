//! Cold-read the exact Windows image through the pinned Linux VM.
//! Run under `vm.sh session` from the repository root.

use std::{fs, path::Path, process::Command};

#[allow(dead_code)]
#[path = "../tests/support/src/verdict.rs"]
mod verdict;

use verdict::{Judge, Verdict};

const VM: &str = "../fs-linux-test-harness/scripts/vm.sh";
const EVIDENCE: &str = "tmp/winfsp-cold-linux";
const GUEST: &str = r#"
set -eu
cd /repo/tmp/winfsp-cold-linux
export LC_ALL=C
report() {
    tag="$1"
    shift
    status=0
    "$@" > "$tag.stdout" 2> "$tag.stderr" || status=$?
    printf '%s\n' "$status" > "$tag.status"
}
report oracle debugfs -R 'dump /oracle oracle.bin' checked.img
report winfsp debugfs -R 'dump /winfsp-checked.bin winfsp.bin' checked.img
report checker e2fsck -fn checked.img
"#;

fn verify(dir: &Path) {
    for (name, judge) in [
        ("oracle", Judge::Debugfs),
        ("winfsp", Judge::Debugfs),
        ("checker", Judge::E2fsck),
    ] {
        let status = fs::read_to_string(dir.join(format!("{name}.status"))).unwrap();
        let code = status.trim().parse::<i32>().unwrap();
        let stdout = fs::read_to_string(dir.join(format!("{name}.stdout"))).unwrap();
        let stderr = fs::read_to_string(dir.join(format!("{name}.stderr"))).unwrap();
        assert_eq!(
            judge.read(Some(code), &stdout, &stderr),
            Verdict::Clean,
            "{name}: Linux VM report was not clean"
        );
    }
    for name in ["oracle", "winfsp"] {
        let expected = fs::read(dir.join(format!("expected-{name}.bin"))).unwrap();
        let actual = fs::read(dir.join(format!("{name}.bin"))).unwrap();
        assert!(
            !expected.is_empty() && actual == expected,
            "{name}: Linux cold-read payload mismatch"
        );
        println!("Linux cold read {name}: {} bytes matched", actual.len());
    }
    println!("Linux non-repairing filesystem check passed");
}

fn main() {
    assert!(Command::new(VM).arg("up").status().unwrap().success());
    assert!(Command::new(VM)
        .args(["exec", GUEST])
        .status()
        .unwrap()
        .success());
    verify(Path::new(EVIDENCE));
}
