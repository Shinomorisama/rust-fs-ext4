#!/usr/bin/env bash
# The release tarball is the install prefix an installer copies as-is --
# bin/rust-fs-ext4 and each dotted name a relative symlink to it, the man
# pages and completions under share/, share/rust-fs-ext4/CAVEATS and the
# licence, nothing outside bin/, share/ and LICENSE -- and every name in it
# runs and identifies itself.
#
# No cargo target name is renamed any more: the multi-call binary's target
# is the repository's name, and the dotted names are symlinks the packaging
# makes. The binary lists them itself (`generate names`), and writes its own
# pages and completions (`generate man|completions`).
#
# This runs the real packaging script against a stand-in multi-call binary
# in a sandbox: one that behaves, and one for each way a build can be wrong
# (missing, --help failing, a version other than the tag's, a generator that
# writes nothing). The release workflow and CI's `cli` job run the same
# script against the real binary, so the checks here are the checks a
# release makes.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PACKAGE="$ROOT/scripts/package-cli.sh"
pass=0
fail=0

ok()  { pass=$((pass + 1)); }
bad() { fail=$((fail + 1)); printf 'FAIL %s\n' "$*"; }

sandbox="$(mktemp -d)"
trap 'rm -rf "$sandbox"' EXIT

crate="$(sed -n 's/^name = "\(.*\)"$/\1/p' "$ROOT/Cargo.toml" | head -n 1)"
[ "$crate" = "am-fs-ext4" ] && ok || bad "crate name read from Cargo.toml: '$crate'"

# A stand-in multi-call binary, named rust-fs-ext4. $1 is the version it
# reports, $2 the exit status of --help, $4 (optional) "nopages" for a man
# generator that writes nothing. It answers under whatever name it is run
# as, as the real one does.
stub() {
    local path="$sandbox/$3"
    mkdir -p "$(dirname "$path")"
    cat > "$path" <<STUB
#!/usr/bin/env bash
me="\$(basename "\$0")"
case "\$1" in
    --help)    echo "Usage: \$me"; exit $2 ;;
    --version) echo "\$me ($crate) $1" ;;
    generate)
        case "\$2" in
            names) printf '%s\\n' mkfs.ext4 fsck.ext4 fs.ext4 ;;
            man)
                [ "${4:-}" = nopages ] && exit 0
                mkdir -p "\$3/man/man8" "\$3/man/man1"
                for n in mkfs.ext4 fsck.ext4; do echo page > "\$3/man/man8/\$n.8"; done
                for n in fs.ext4 rust-fs-ext4; do echo page > "\$3/man/man1/\$n.1"; done ;;
            completions)
                mkdir -p "\$3/zsh/site-functions" "\$3/bash-completion/completions" "\$3/fish/vendor_completions.d"
                for n in mkfs.ext4 fsck.ext4 fs.ext4 rust-fs-ext4; do
                    echo c > "\$3/zsh/site-functions/_\$n"
                    echo c > "\$3/bash-completion/completions/\$n"
                    echo c > "\$3/fish/vendor_completions.d/\$n.fish"
                done ;;
        esac ;;
    *) exit 2 ;;
esac
STUB
    chmod +x "$path"
    printf '%s\n' "$path"
}

# Runs the packaging script in a fresh output directory. It prints the
# tarball's name relative to that directory, so this prints the absolute path.
#
# STALE=<name> first leaves a file of that name in the output directory, as a
# previous run would have. On failure it prints whatever the script printed,
# so a refusal that names a tarball anyway is seen, and it records the output
# directory in $sandbox/package-out for the leftover-tarball check.
package() {
    local out="$sandbox/out-$RANDOM$RANDOM" name status
    mkdir -p "$out"
    printf '%s\n' "$out" > "$sandbox/package-out"
    [ -z "${STALE:-}" ] || echo stale > "$out/$STALE"
    name="$(cd "$out" && bash "$PACKAGE" "$@" 2>"$sandbox/stderr")"
    status=$?
    if [ "$status" -ne 0 ]; then
        printf '%s' "$name"
        return "$status"
    fi
    printf '%s\n' "$out/$name"
}

[ -f "$PACKAGE" ] && ok || bad "scripts/package-cli.sh exists"

# --- A good build: the tarball, its name, and its contents. --------------
good="$(stub 9.9.9 0 good/rust-fs-ext4)"
if tarball="$(package 9.9.9 darwin-arm64 "$(dirname "$good")")"; then
    ok
else
    bad "a good build packages: $(cat "$sandbox/stderr")"
    tarball=""
fi

case "$(basename "$tarball")" in
    "$crate-9.9.9-darwin-arm64.tar.gz") ok ;;
    *) bad "tarball is named <crate>-<version>-<label>.tar.gz, got '$tarball'" ;;
esac

# The content checks below need the tarball. Without it they fail rather than
# fall silent, since a check that does not run reads like one that passed.
[ -f "$tarball" ] && ok || bad "the packaged tarball exists at '$tarball'"
if [ -f "$tarball" ]; then
    files="$(tar -tzf "$tarball" | sed 's|^\./||' | grep -v '/$' | sort | tr '\n' ' ')"
    want="LICENSE bin/fs.ext4 bin/fsck.ext4 bin/mkfs.ext4 bin/rust-fs-ext4 \
share/bash-completion/completions/fs.ext4 share/bash-completion/completions/fsck.ext4 \
share/bash-completion/completions/mkfs.ext4 share/bash-completion/completions/rust-fs-ext4 \
share/fish/vendor_completions.d/fs.ext4.fish share/fish/vendor_completions.d/fsck.ext4.fish \
share/fish/vendor_completions.d/mkfs.ext4.fish share/fish/vendor_completions.d/rust-fs-ext4.fish \
share/man/man1/fs.ext4.1 share/man/man1/rust-fs-ext4.1 share/man/man8/fsck.ext4.8 \
share/man/man8/mkfs.ext4.8 share/rust-fs-ext4/CAVEATS share/zsh/site-functions/_fs.ext4 \
share/zsh/site-functions/_fsck.ext4 share/zsh/site-functions/_mkfs.ext4 \
share/zsh/site-functions/_rust-fs-ext4 "
    [ "$files" = "$want" ] && ok || bad "the tarball's members: got [$files], expected [$want]"
    case "$files" in
        *mkfs_ext4*) bad "a cargo target name reached the tarball: $files" ;;
        *) ok ;;
    esac
    unpacked="$sandbox/unpacked"
    mkdir -p "$unpacked"
    tar -xzf "$tarball" -C "$unpacked"
    [ -x "$unpacked/bin/rust-fs-ext4" ] && [ ! -L "$unpacked/bin/rust-fs-ext4" ] && ok \
        || bad "bin/rust-fs-ext4 is an executable file, not a link"
    cmp -s "$unpacked/bin/rust-fs-ext4" "$good" && ok || bad "bin/rust-fs-ext4 is the built binary"
    for name in mkfs.ext4 fsck.ext4 fs.ext4; do
        [ -L "$unpacked/bin/$name" ] && [ "$(readlink "$unpacked/bin/$name")" = rust-fs-ext4 ] && ok \
            || bad "bin/$name is a relative symlink to rust-fs-ext4"
    done
    cmp -s "$unpacked/share/rust-fs-ext4/CAVEATS" "$ROOT/packaging/CAVEATS" && ok \
        || bad "share/rust-fs-ext4/CAVEATS is packaging/CAVEATS"
    cmp -s "$unpacked/LICENSE" "$ROOT/LICENSE" && ok || bad "LICENSE is the repository's"
fi

# The CAVEATS a formula prints: one to four lines.
caveat_lines="$(wc -l < "$ROOT/packaging/CAVEATS" | tr -d ' ')"
[ "$caveat_lines" -ge 1 ] && [ "$caveat_lines" -le 4 ] && ok \
    || bad "packaging/CAVEATS is one to four lines, is $caveat_lines"
grep -q 'rust-fs-ext4 doctor' "$ROOT/packaging/CAVEATS" && ok \
    || bad "packaging/CAVEATS points at rust-fs-ext4 doctor"

# --- Each way a build can be wrong is refused, with no tarball left. ------
refused() {
    local why="$1"; shift
    local stdout out_dir left
    if stdout="$(package "$@")"; then
        bad "$why is refused, but packaging succeeded: $stdout"
    else
        ok
        [ -z "$stdout" ] && ok || bad "$why leaves no tarball named on stdout: $stdout"
        out_dir="$(cat "$sandbox/package-out")"
        left="$(find "$out_dir" -maxdepth 1 -name '*.tar.gz')"
        [ -z "$left" ] && ok || bad "$why leaves no tarball behind, found: $left"
    fi
}

refused "a missing binary" 9.9.9 darwin-arm64 "$sandbox/nowhere"
refused "a binary whose --help fails" 9.9.9 darwin-arm64 "$(dirname "$(stub 9.9.9 1 helpfails/rust-fs-ext4)")"
refused "a binary reporting a version other than the tag's" 9.9.9 darwin-arm64 "$(dirname "$(stub 1.0.0 0 wrongver/rust-fs-ext4)")"
refused "a man generator that writes nothing" 9.9.9 darwin-arm64 "$(dirname "$(stub 9.9.9 0 nopages/rust-fs-ext4 nopages)")"
refused "a missing label" 9.9.9 "" "$(dirname "$good")"
refused "a missing version" "" darwin-arm64 "$(dirname "$good")"
STALE="$crate-9.9.9-darwin-arm64.tar.gz" refused "a failure beside a previous run's tarball" 9.9.9 darwin-arm64 "$sandbox/nowhere"

# --- The release workflow packages through this script. ------------------
release="$ROOT/.github/workflows/release.yml"
grep -q 'scripts/package-cli.sh' "$release" && ok \
    || bad "release.yml packages through scripts/package-cli.sh"
grep -q 'cargo build --release --locked --features cli --bin rust-fs-ext4' "$release" && ok \
    || bad "release.yml builds the rust-fs-ext4 target, with the cli feature"
grep -qE 'uses: actions/attest-build-provenance@[0-9a-f]{40}' "$release" && ok \
    || bad "release.yml attests the tarballs' build provenance, with the action pinned to a commit"
grep -q 'attestations: write' "$release" && ok \
    || bad "release.yml grants the release job attestations: write"

printf 'package-cli: %d passed, %d failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
