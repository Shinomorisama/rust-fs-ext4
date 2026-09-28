#!/usr/bin/env bash
# Is this crate's version already on crates.io? Prints `published=true` or
# `published=false` on stdout -- the shape of a GitHub step output -- and
# exits 0 only when crates.io actually answered.
#
#   200  published          the release skips `cargo publish`
#   404  not published      the release publishes
#   anything else           not an answer: retried, then exit 1
#
# The status is read from the response, not from curl's exit code. `curl -f`
# exits non-zero alike on a 404, a 429, a 5xx, a DNS failure and a timeout,
# and treating all of them as "not published" sent a rate-limited re-run of
# an already-published release on to `cargo publish`, which then failed with
# "already exists" (#329). A step that fails when crates.io cannot say is a
# red run that names the real cause.
#
#   scripts/crates-io-published.sh [<name> <version>]
#
# Without arguments the name and version come from Cargo.toml.
#
# CRATES_IO_ATTEMPTS (default 4) and CRATES_IO_RETRY_DELAY (seconds before
# the first retry, doubled each time; default 5) bound the retries.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [ $# -eq 2 ]; then
    name="$1" version="$2"
elif [ $# -eq 0 ]; then
    name="$(grep -m1 '^name = ' "$ROOT/Cargo.toml" | sed 's/name = //; s/"//g')"
    version="$(grep -m1 '^version = ' "$ROOT/Cargo.toml" | sed 's/version = //; s/"//g')"
else
    echo "usage: $0 [<name> <version>]" >&2
    exit 2
fi
[ -n "$name" ] && [ -n "$version" ] || { echo "crates-io-published: no name or version" >&2; exit 2; }

attempts="${CRATES_IO_ATTEMPTS:-4}"
delay="${CRATES_IO_RETRY_DELAY:-5}"
url="https://crates.io/api/v1/crates/$name/$version"

for ((attempt = 1; ; attempt++)); do
    # No -f: the status is the answer, and -f would fold it into the exit
    # code. A connection that never completes prints 000.
    code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 30 \
        -H 'User-Agent: release-workflow' "$url")" || true
    case "$code" in
        200)
            echo "$name $version is already on crates.io; nothing to publish." >&2
            echo "published=true"
            exit 0 ;;
        404)
            echo "$name $version is not on crates.io; publishing." >&2
            echo "published=false"
            exit 0 ;;
    esac
    if [ "$attempt" -ge "$attempts" ]; then
        echo "crates-io-published: $url answered HTTP ${code:-000} on each of $attempts attempts;" \
             "cannot tell whether $name $version is published" >&2
        exit 1
    fi
    echo "crates-io-published: HTTP ${code:-000} from $url; retrying in ${delay}s" >&2
    sleep "$delay"
    delay=$((delay * 2))
done
