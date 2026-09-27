#!/usr/bin/env bash
# No matrix leg touches the GitHub release.
#
# A matrix leg that creates the release and uploads to it publishes its
# own asset whether or not its siblings succeed. With `fail-fast: false`
# the darwin leg could publish while the linux leg later failed, leaving a
# release that is half there (#329). The legs build and upload ARTIFACTS;
# one job that `needs:` them all creates the release, and it only runs
# when every leg passed.
#
# Refused inside any job with `strategy.matrix`: a `gh release` command in
# a `run:` block (comment lines are not read), and the release-publishing
# actions.
#
#   bash tests/scripts/test-release-outside-matrix.sh
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

fail() { echo "FAIL  $*" >&2; exit 1; }

command -v python3 >/dev/null 2>&1 || fail "python3 is required; run 'chore tools'"
python3 -c 'import yaml' 2>/dev/null ||
    fail "the python3 yaml module is required (pip install pyyaml)"

scan() {
    python3 - "$@" <<'PY'
import re, sys, yaml

GH_RELEASE = re.compile(r"(^|[\s;&|(`])gh\s+release\b")
ACTIONS = ("softprops/action-gh-release", "ncipollo/release-action",
           "actions/create-release", "actions/upload-release-asset")

for path in sys.argv[1:]:
    doc = yaml.safe_load(open(path)) or {}
    for name, job in (doc.get("jobs") or {}).items():
        if not (job.get("strategy") or {}).get("matrix"):
            continue
        for i, step in enumerate(job.get("steps") or []):
            label = step.get("name") or step.get("uses") or f"step {i + 1}"
            where = f"{path}: matrix job {name}, {label}"
            uses = step.get("uses") or ""
            if uses.split("@")[0] in ACTIONS:
                print(f"{where}: uses {uses}")
            for line in str(step.get("run") or "").splitlines():
                if line.strip().startswith("#"):
                    continue
                if GH_RELEASE.search(line):
                    print(f"{where}: runs `{line.strip()}`")
PY
}

SANDBOX="$(mktemp -d)"
trap 'rm -rf "$SANDBOX"' EXIT HUP INT TERM

# --- 1. The scan refuses what it exists to refuse. ------------------------
cat > "$SANDBOX/bad.yml" <<'EOF'
on: push
jobs:
  package:
    strategy:
      fail-fast: false
      matrix:
        os: [a, b]
    runs-on: ${{ matrix.os }}
    steps:
      - name: upload
        run: |
          # gh release is what this used to say
          gh release view "$tag" || gh release create "$tag"
      - uses: softprops/action-gh-release@v2
  release:
    needs: [package]
    runs-on: ubuntu-latest
    steps:
      - run: gh release upload "$tag" dist/*
EOF

found="$(scan "$SANDBOX/bad.yml")" || fail "the scan itself failed"
expect=(
    "matrix job package, upload: runs \`gh release view"
    "matrix job package, softprops/action-gh-release@v2: uses"
)
for e in "${expect[@]}"; do
    grep -qF "$e" <<<"$found" || fail "the scan missed '$e':"$'\n'"$found"
done
count="$(grep -c . <<<"$found")"
[[ "$count" -eq ${#expect[@]} ]] ||
    fail "the scan found $count problems, expected ${#expect[@]}:"$'\n'"$found"

# --- 2. The real workflows. -----------------------------------------------
shopt -s nullglob
workflows=("$REPO"/.github/workflows/*.yml "$REPO"/.github/workflows/*.yaml)
[[ ${#workflows[@]} -gt 0 ]] || fail "no workflows under .github/workflows"

found="$(scan "${workflows[@]}")" || fail "the scan itself failed"
if [[ -n "$found" ]]; then
    echo "FAIL  a matrix leg writes to the release, so one leg can publish while another fails:" >&2
    printf '%s\n' "${found//$REPO\//}" >&2
    exit 1
fi

echo "PASS  no matrix leg touches the release"
