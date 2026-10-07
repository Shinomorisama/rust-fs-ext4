#!/usr/bin/env bash
# Run from the pinned baseline checkout on a KVM-capable Linux runner.
set -euo pipefail
test "$(git rev-parse HEAD)" = 3b9bc5d19dcfe05a00289dd8b1cebbf11e49b742
test "$(git -C ../rust-fs-core rev-parse HEAD)" = 9e900154a7dff2a6c8f136db31def33bed9f09cc
test "$(git -C ../fs-linux-test-harness rev-parse HEAD)" = 7e0a84291c76fe4c762143f6f6d5787cc11749e7
test "$(cargo llvm-cov --version)" = 'cargo-llvm-cov 0.9.1'
chore vm:host:check
mkdir -p tmp/casefold-coverage target
{
    git rev-parse HEAD
    git -C ../rust-fs-core rev-parse HEAD
    git -C ../fs-linux-test-harness rev-parse HEAD
    rustc --version
    cargo llvm-cov --version
    sha256sum Cargo.lock
} > tmp/casefold-coverage/versions.txt

# A fresh target prevents profiles from another run contaminating this baseline.
export CARGO_TARGET_DIR
CARGO_TARGET_DIR="$(mktemp -d "$PWD/target/casefold-coverage.XXXXXX")"
eval "$(cargo llvm-cov show-env --sh)"
# Include every executable Rust test, including the intentionally slow fuzz
# test. Documentation snippets are outside this stable-toolchain metric.
scripts/tier.sh casefold:coverage casefold-coverage 3700 183000 -- \
    scripts/test.sh --locked --release --lib --bins --tests -- --include-ignored
bash scripts/core.sh test-floor casefold-coverage 1060
python3 - <<'PY'
from pathlib import Path
import re
log = Path('tmp/logs/casefold-coverage.log').read_text()
counts = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', log)
assert counts and sum(int(c[0]) for c in counts) >= 1060
assert all(int(c[1]) == 0 and int(c[2]) == 0 for c in counts), counts
names = sorted(re.findall(r'^test .+ \.\.\. ok$', log, re.M))
assert len(names) == sum(int(c[0]) for c in counts)
Path('tmp/casefold-coverage/passing-tests.txt').write_text('\n'.join(names) + '\n')
PY
cargo llvm-cov report --release --json --summary-only \
    --output-path tmp/casefold-coverage/coverage.json
cargo llvm-cov report --release > tmp/casefold-coverage/coverage.txt
