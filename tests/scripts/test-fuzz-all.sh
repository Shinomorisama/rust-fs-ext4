#!/usr/bin/env bash
# scripts/fuzz-all.sh refuses a bad budget before fuzzing, and tells a
# crash apart from a target it could not fuzz.
#
# 1. THE BUDGET IS DATA, NOT SHELL (#304). fuzz.yml used to paste the
#    dispatch input into its `run:` line, so `1; false` was two commands.
#    The workflow now passes it through env:, and the script is the
#    second wall: anything but a positive whole number of seconds, or a
#    number that times the target count would overrun the job's timeout,
#    is refused before cargo is ever started. `0` is refused too:
#    libFuzzer reads -max_total_time=0 as "no limit".
#
# 2. A NON-ZERO EXIT IS NOT A FINDING. libFuzzer exits 1 on a crash, and
#    also exits 1 when a corpus directory does not exist; a build that
#    failed exits non-zero as well. Reading every non-zero exit as a
#    crash made a sibling repository report three never-fuzzed targets
#    as crashing every night for a week. A target with no seed corpus is
#    named as missing and never handed to the fuzzer, a non-zero exit
#    that left no reproducer is a failed run, and only a new file under
#    fuzz/artifacts/<target>/ is a crash.
#
# The real script runs in a sandbox against a stand-in `cargo` that
# exits the way libFuzzer does.
#
#   bash tests/scripts/test-fuzz-all.sh
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fails=0
fail() { echo "FAIL  $*" >&2; fails=$((fails + 1)); }

mkdir -p "$REPO/tmp"
SANDBOX="$(mktemp -d "$REPO/tmp/fuzz-all.XXXXXX")"
trap 'rm -rf "$SANDBOX"' EXIT HUP INT TERM

# A stand-in for `cargo +nightly fuzz run <target> <dir>... -- <flags>`:
# libFuzzer's exit on a missing corpus directory, a crash (which leaves
# an artifact) and a build that failed (which leaves nothing).
mkdir -p "$SANDBOX/bin"
cat > "$SANDBOX/bin/cargo" <<'STUB'
#!/usr/bin/env bash
target="$4"
echo "$target" >> "$FUZZ_TEST_LOG"
shift 4
for arg in "$@"; do
    [ "$arg" = "--" ] && break
    if [ ! -d "$arg" ]; then
        echo "No such file or directory: $arg; exiting"
        exit 1
    fi
done
case "$target" in
    crashes)
        mkdir -p "fuzz/artifacts/$target"
        printf 'boom' > "fuzz/artifacts/$target/crash-0000"
        exit 1 ;;
    broken)
        echo "error: could not compile" >&2
        exit 101 ;;
esac
exit 0
STUB
chmod +x "$SANDBOX/bin/cargo"
export FUZZ_TEST_LOG="$SANDBOX/invoked"

# A repository shaped like this one, declaring <targets...>, each with a
# seed unless it is named `unseeded` or `emptied`.
make_repo() {
    local root="$1" t
    shift
    rm -rf "$root"
    mkdir -p "$root/scripts" "$root/fuzz/corpus"
    cp "$REPO/scripts/fuzz-all.sh" "$root/scripts/fuzz-all.sh"
    printf '[package]\nname = "sandbox-fuzz"\n' > "$root/fuzz/Cargo.toml"
    for t in "$@"; do
        printf '\n[[bin]]\nname = "%s"\n' "$t" >> "$root/fuzz/Cargo.toml"
        case "$t" in
            unseeded) ;;
            emptied) mkdir -p "$root/fuzz/corpus/$t" ;;
            *) mkdir -p "$root/fuzz/corpus/$t"
               printf 'seed' > "$root/fuzz/corpus/$t/one.bin" ;;
        esac
    done
}

# Run the sandboxed script with <args...>; sets $out and $status.
run() {
    : > "$FUZZ_TEST_LOG"
    out="$(PATH="$SANDBOX/bin:$PATH" bash "$SANDBOX/repo/scripts/fuzz-all.sh" "$@" 2>&1)"
    status=$?
}

# --- 1. A bad budget is refused before anything is fuzzed. ---------------
make_repo "$SANDBOX/repo" a b c d e
budget="$(sed -n 's/^total_budget=\([0-9]*\).*/\1/p' "$REPO/scripts/fuzz-all.sh")"
[ -n "$budget" ] || fail "scripts/fuzz-all.sh sets no total_budget=<seconds>"
over=$(( ${budget:-0} / 5 + 1 ))
for bad in '1; false' '$(false)' '' ' 1' '1 ' '-1' '0' '1.5' '1e3' '99999999999999999999' "$over"; do
    run "$bad"
    [ "$status" -ne 0 ] || fail "budget '$bad' was accepted"
    [ ! -s "$FUZZ_TEST_LOG" ] || fail "budget '$bad' reached the fuzzer"
done
run 1
[ "$status" -eq 0 ] || fail "budget '1' was refused: $out"
[ "$(grep -c . "$FUZZ_TEST_LOG")" -eq 5 ] || fail "budget '1' did not fuzz all five targets"
run "$(( ${budget:-5} / 5 ))"
[ "$status" -eq 0 ] || fail "the largest budget that fits was refused: $out"

# --- 2. A missing corpus and a failed run are not crashes. ---------------
make_repo "$SANDBOX/repo" seeded unseeded emptied crashes broken
run 1
[ "$status" -ne 0 ] || fail "a run with a missing corpus exited 0"
grep -q "fuzz/corpus/unseeded" <<<"$out" || fail "the missing fuzz/corpus/unseeded is not named"
grep -q "fuzz/corpus/emptied" <<<"$out" || fail "the empty fuzz/corpus/emptied is not named"
if grep -Eq "'(unseeded|emptied)' found an input|crashes in:.*(unseeded|emptied)" <<<"$out"; then
    fail "a target with no seed corpus was reported as a crash"
fi
if grep -qx -e unseeded -e emptied "$FUZZ_TEST_LOG"; then
    fail "a target with no seed corpus was handed to the fuzzer"
fi
grep -qx seeded "$FUZZ_TEST_LOG" || fail "a seeded target was not fuzzed because another had no corpus"
grep -q "seeded: clean" <<<"$out" || fail "the seeded target is not reported clean"
grep -Eq "crashes in:.* crashes( |$)" <<<"$out" || fail "a genuine crash is no longer reported as one"
if grep -Eq "crashes in:.*broken" <<<"$out"; then
    fail "a run that failed without leaving a reproducer was reported as a crash"
fi
grep -q "'broken'" <<<"$out" || fail "a run that failed without a reproducer is not named"

if [ "$fails" -gt 0 ]; then
    echo "--- the last run of scripts/fuzz-all.sh said:" >&2
    echo "$out" >&2
    exit 1
fi
echo "PASS  fuzz-all.sh refuses a bad budget before fuzzing and tells a crash from a run that could not fuzz"
