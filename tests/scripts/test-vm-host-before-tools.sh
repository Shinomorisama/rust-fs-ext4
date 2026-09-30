#!/usr/bin/env bash
# The VM host is set up before `chore tools` checks for it.
#
# `chore tools` (scripts/tools.sh) fails on a Linux host that has no
# Vagrant, no QEMU, no qemu-img or no /dev/kvm, and on a runner it is
# ../fs-linux-test-harness/scripts/ci-setup-linux.sh that installs them.
# A job that runs the check first fails before any test has run. That is
# how the v0.6.0 release run failed (36731070363): release.yml checked the
# tools one step before it installed them, while ci.yml, which runs every
# pull request, had them the right way round, so nothing had caught it.
#
# Refused in every workflow: a job with a step that runs `chore tools`
# (not `chore tools:<variant>`) and no earlier step that runs
# ci-setup-linux.sh.
#
#   bash tests/scripts/test-vm-host-before-tools.sh
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

fail() { echo "FAIL  $*" >&2; exit 1; }

command -v python3 >/dev/null 2>&1 || fail "python3 is required; run 'chore tools'"
python3 -c 'import yaml' 2>/dev/null ||
    fail "the python3 yaml module is required (pip install pyyaml)"

scan() {
    python3 - "$@" <<'PY'
import re, sys, yaml

TOOLS = re.compile(r"(^|[\s;&|(])chore\s+tools(\s|$|[;&|)])")
SETUP = re.compile(r"ci-setup-linux\.sh")

def lines(step):
    return [l for l in str(step.get("run") or "").splitlines()
            if not l.strip().startswith("#")]

for path in sys.argv[1:]:
    doc = yaml.safe_load(open(path)) or {}
    for name, job in (doc.get("jobs") or {}).items():
        ready = False
        for i, step in enumerate(job.get("steps") or []):
            body = lines(step)
            if any(TOOLS.search(l) for l in body) and not ready:
                label = step.get("name") or f"step {i + 1}"
                print(f"{path}: job {name}, {label}: chore tools before ci-setup-linux.sh")
            if any(SETUP.search(l) for l in body):
                ready = True
PY
}

SANDBOX="$(mktemp -d)"
trap 'rm -rf "$SANDBOX"' EXIT HUP INT TERM

# --- 1. The scan refuses what it exists to refuse. ------------------------
cat > "$SANDBOX/bad.yml" <<'EOF2'
on: push
jobs:
  early:
    runs-on: ubuntu-latest
    steps:
      - name: chore tools
        run: chore tools
      - name: KVM, QEMU and Vagrant
        run: ../fs-linux-test-harness/scripts/ci-setup-linux.sh
  never:
    runs-on: ubuntu-latest
    steps:
      - run: |
          set -eu
          chore tools
  fine:
    runs-on: ubuntu-latest
    steps:
      - run: ../fs-linux-test-harness/scripts/ci-setup-linux.sh
      - run: chore tools
  wasm:
    runs-on: ubuntu-latest
    steps:
      - run: chore tools:wasm
      - run: |
          # chore tools is checked later
          true
EOF2

found="$(scan "$SANDBOX/bad.yml")" || fail "the scan itself failed"
expect="$SANDBOX/bad.yml: job early, chore tools: chore tools before ci-setup-linux.sh
$SANDBOX/bad.yml: job never, step 1: chore tools before ci-setup-linux.sh"
[ "$found" = "$expect" ] ||
    fail "the scan found the wrong things in a known-bad workflow:
--- expected
$expect
--- found
$found"

# --- 2. This repository's workflows. --------------------------------------
found="$(scan "$REPO"/.github/workflows/*.yml)" || fail "the scan itself failed"
[ -z "$found" ] || fail "a job checks for the VM host before installing it:
$found"

echo "PASS  every job sets up the VM host before chore tools checks it"
