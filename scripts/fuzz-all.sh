#!/usr/bin/env bash
# Run every fuzz target for a bounded time.
#
# This is the explorer, not the gate. It is allowed to find something
# and fail; what it is not allowed to do is run for an unbounded time,
# which is why every target gets the same budget and the script reports
# which ones found something rather than stopping at the first.
#
# Anything it finds belongs in fuzz/corpus/, where tests/fuzz_decoders.rs
# replays it on every pull request from then on.
#
# Usage: scripts/fuzz-all.sh [seconds-per-target]
set -uo pipefail

# The most fuzzing one run may do, in seconds, across every target.
# fuzz.yml's job has a 90-minute timeout, and building cargo-fuzz and
# each target under the sanitizer takes most of the other half hour. A
# run that reaches the timeout is cancelled, and a cancelled run is one
# whose reproducers are at the mercy of the upload step, so the budget
# is checked here, before anything is fuzzed, rather than discovered by
# the timeout. tests/ci_profile.rs keeps the two numbers apart.
total_budget=3600

budget="${1-60}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$here"

targets=$(sed -n 's/^name = "\(.*\)"$/\1/p' fuzz/Cargo.toml | tail -n +2)
[ -n "$targets" ] || { echo "no fuzz targets declared in fuzz/Cargo.toml" >&2; exit 1; }
count=$(wc -w <<<"$targets" | tr -d ' ')

# THE BUDGET IS DATA. It arrives from a workflow_dispatch input, so it
# is held to a positive whole number of seconds before it is used for
# anything: `1; false` is refused here, not run. Zero is refused too,
# because libFuzzer reads -max_total_time=0 as "no limit". The length
# check comes first so the arithmetic below cannot overflow, and 10#
# stops a leading zero being read as octal.
if ! [[ "$budget" =~ ^[0-9]+$ ]] || [ "${#budget}" -gt 6 ] || [ $((10#$budget)) -eq 0 ]; then
    echo "::error::seconds per target must be a positive whole number, got '$budget'" >&2
    exit 2
fi
budget=$((10#$budget))
if [ $((budget * count)) -gt "$total_budget" ]; then
    echo "::error::${budget}s x $count targets is $((budget * count))s, over the ${total_budget}s this run may fuzz; use at most $((total_budget / count))s per target" >&2
    exit 2
fi

scratch="$(mktemp -d "${TMPDIR:-/tmp}/ext4-fuzz-scratch.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT

# Three verdicts, and they must not be confused with one another.
#
# A CRASH is libFuzzer finding an input: it writes the reproducer under
# fuzz/artifacts/<target>/ and exits non-zero. A MISSING CORPUS is a
# target with nothing under fuzz/corpus/<target>/: libFuzzer prints "No
# such file or directory ... exiting" and exits 1, the same status as a
# crash. And a run that FAILED -- a build error, a toolchain that is not
# there -- also exits non-zero and leaves no reproducer.
#
# So the corpus is checked before the fuzzer is started, and a target
# without one is named and fails the run. It is never created on the
# fly: an empty directory would let the fuzzer start from nothing and
# pass, which is a skip that looks like a run. The other targets are
# still fuzzed, so one gap does not cost the rest of the run.
artifacts_of() {
    find "fuzz/artifacts/$1" -type f 2>/dev/null | wc -l | tr -d ' '
}

failed=""
missing=""
broken=""
for target in $targets; do
    seeds="fuzz/corpus/$target"
    if [ ! -d "$seeds" ] || [ -z "$(find "$seeds" -type f -print -quit)" ]; then
        echo "::error::fuzz target '$target' has no seed corpus: $seeds is missing or empty"
        missing="$missing $target"
        continue
    fi
    echo "::group::fuzz $target (${budget}s)"
    # The committed corpus is a seed, not a scratch pad. libFuzzer
    # writes every coverage-expanding input back into the FIRST corpus
    # directory it is given, so it gets a throwaway one and the
    # committed seeds are passed after it, read-only. Without this a
    # local run leaves dozens of hash-named blobs in fuzz/corpus/, and
    # the curated seeds -- real structures, and the reproducer for each
    # defect ever found -- get lost among them.
    mkdir -p "$scratch/$target"
    before=$(artifacts_of "$target")
    if cargo +nightly fuzz run "$target" "$scratch/$target" "$seeds" \
        -- -max_total_time="$budget"; then
        echo "$target: clean"
    elif [ "$(artifacts_of "$target")" -gt "$before" ]; then
        echo "::error::fuzz target '$target' found an input that crashed or hung"
        failed="$failed $target"
    else
        echo "::error::fuzz target '$target' did not run: the fuzzer exited non-zero without leaving a reproducer under fuzz/artifacts/$target/"
        broken="$broken $target"
    fi
    echo "::endgroup::"
done

status=0
if [ -n "$missing" ]; then
    echo "::error::no seed corpus for:$missing"
    echo "These targets were NOT fuzzed. Each needs real structures under"
    echo "fuzz/corpus/<target>/ -- rebuild them with scripts/make-fuzz-corpus.sh."
    status=1
fi
if [ -n "$broken" ]; then
    echo "::error::the fuzzer failed to run for:$broken"
    status=1
fi
if [ -n "$failed" ]; then
    echo "::error::fuzzing found crashes in:$failed"
    echo "Each reproducer is under fuzz/artifacts/<target>/. Commit it to"
    echo "fuzz/corpus/<target>/ so tests/fuzz_decoders.rs replays it from now on."
    status=1
fi
[ "$status" -eq 0 ] || exit "$status"
echo "every target ran ${budget}s without finding a crash"
