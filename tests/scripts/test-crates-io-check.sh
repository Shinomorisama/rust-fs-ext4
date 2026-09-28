#!/usr/bin/env bash
# The release's "already on crates.io?" check answers only when crates.io
# answered.
#
# `curl -sf` exits non-zero alike on 404, 429, a 5xx, a DNS failure and a
# timeout, and the check read every one of them as "not published" (#329).
# Re-running the release of a published version while crates.io was rate
# limited then went on to `cargo publish`, which failed with "already
# exists" -- a red run blaming the wrong thing. 200 means published, 404
# means publish, and anything else is not an answer: retried, and then the
# step fails.
#
# THIS RUNS THE WORKFLOW'S OWN STEP, not the script it calls: the `run:` of
# the step with id `already` in release.yml's `publish` job, under the
# shell GitHub gives `shell: bash`, with `curl` replaced by a stub on PATH.
# A script that is right but no longer called would otherwise pass.
#
#   bash tests/scripts/test-crates-io-check.sh
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WORKFLOW="$REPO/.github/workflows/release.yml"

fail() { echo "FAIL  $*" >&2; exit 1; }

command -v python3 >/dev/null 2>&1 || fail "python3 is required; run 'chore tools'"
python3 -c 'import yaml' 2>/dev/null ||
    fail "the python3 yaml module is required (pip install pyyaml)"

SANDBOX="$(mktemp -d)"
trap 'rm -rf "$SANDBOX"' EXIT HUP INT TERM

python3 - "$WORKFLOW" > "$SANDBOX/step.sh" <<'PY' ||
import sys, yaml
doc = yaml.safe_load(open(sys.argv[1])) or {}
steps = ((doc.get("jobs") or {}).get("publish") or {}).get("steps") or []
runs = [s.get("run") for s in steps if s.get("id") == "already"]
if len(runs) != 1 or not runs[0]:
    sys.exit("no single step with id `already` and a `run:` in the publish job")
print(runs[0])
PY
    fail "cannot read the crates.io check out of release.yml"

# A curl that answers from $CURL_CODES, one status per call (the last one
# repeats), and behaves as curl does: `-w` prints the code, `-f` exits 22
# on a status of 400 or more, and a code of 000 is a connection that never
# completed -- curl prints 000 for %{http_code} and exits 6.
mkdir -p "$SANDBOX/bin"
cat > "$SANDBOX/bin/curl" <<'EOF'
#!/usr/bin/env bash
out=/dev/stdout fmt="" failflag=0
while [ $# -gt 0 ]; do
    case "$1" in
        -o|--output) out="$2"; shift ;;
        -w|--write-out) fmt="$2"; shift ;;
        -H|--header|-A|--user-agent|-m|--max-time|--connect-timeout|--retry) shift ;;
        --fail|--fail-with-body) failflag=1 ;;
        --*) ;;
        -*f*) failflag=1 ;;
    esac
    shift
done
echo call >> "$CURL_CALLS"
code="$(head -n1 "$CURL_CODES")"
if [ "$(wc -l < "$CURL_CODES")" -gt 1 ]; then
    tail -n +2 "$CURL_CODES" > "$CURL_CODES.next" && mv "$CURL_CODES.next" "$CURL_CODES"
fi
report() { [ -z "$fmt" ] || printf '%s' "${fmt//%\{http_code\}/$code}"; }
if [ "$code" = 000 ]; then report; exit 6; fi
[ "$code" = 200 ] && printf '{"version":{}}' > "$out" || printf '{"errors":[]}' > "$out"
if [ "$failflag" = 1 ] && [ "$code" -ge 400 ]; then report; exit 22; fi
report
exit 0
EOF
chmod +x "$SANDBOX/bin/curl"

# run <codes...>: the step's exit status in $status, its outputs in $outputs.
run() {
    printf '%s\n' "$@" > "$SANDBOX/codes"
    : > "$SANDBOX/calls"
    : > "$SANDBOX/output"
    (
        cd "$REPO" &&
        PATH="$SANDBOX/bin:$PATH" CURL_CODES="$SANDBOX/codes" CURL_CALLS="$SANDBOX/calls" \
        GITHUB_OUTPUT="$SANDBOX/output" CRATES_IO_RETRY_DELAY=0 \
            bash --noprofile --norc -eo pipefail "$SANDBOX/step.sh"
    ) > "$SANDBOX/log" 2>&1
    status=$?
    outputs="$(cat "$SANDBOX/output")"
    calls="$(grep -c . "$SANDBOX/calls")"
}

run 200
[[ $status -eq 0 && "$outputs" == "published=true" ]] ||
    fail "200 should be published=true, got status $status, outputs '$outputs':"$'\n'"$(cat "$SANDBOX/log")"

run 404
[[ $status -eq 0 && "$outputs" == "published=false" ]] ||
    fail "404 should be published=false, got status $status, outputs '$outputs':"$'\n'"$(cat "$SANDBOX/log")"

for code in 503 429 000; do
    run "$code"
    [[ $status -ne 0 ]] ||
        fail "a $code from crates.io was read as an answer: status 0, outputs '$outputs'"
    [[ -z "$outputs" ]] ||
        fail "a $code from crates.io still wrote '$outputs'"
    [[ $calls -gt 1 ]] ||
        fail "a $code from crates.io was not retried ($calls call)"
done

run 503 200
[[ $status -eq 0 && "$outputs" == "published=true" ]] ||
    fail "a 503 then a 200 should be published=true, got status $status, outputs '$outputs':"$'\n'"$(cat "$SANDBOX/log")"

echo "PASS  the crates.io check answers 200 and 404, and fails on anything else"
