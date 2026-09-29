#!/usr/bin/env bash
# package-cli.sh <version> <label> [target-dir]
#
# Package the built command-line tools as a release tarball in the current
# directory, check it, and print its file name on stdout.
#
#   <version>     the release version, without the leading `v`
#   <label>       the platform, e.g. darwin-arm64 or linux-x86_64
#   [target-dir]  where cargo put the release build (default: target/release)
#
# THE TARBALL IS THE CONTRACT with whatever installs it. Its layout is a
# prefix, so an installer copies it as-is and needs to know nothing about
# which tools are in it:
#
#   bin/<repo>                               the multi-call binary, the real file
#   bin/<tool> -> <repo>                     each dotted name, a relative symlink
#   share/man/man8/<mkfs.*|fsck.*>.8         man pages, section 8
#   share/man/man1/<everything else>.1       man pages, section 1
#   share/zsh/site-functions/_<name>         completions, for every name
#   share/bash-completion/completions/<name>
#   share/fish/vendor_completions.d/<name>.fish
#   share/<repo>/CAVEATS                     at most four lines an installer shows
#   LICENSE
#
# <repo> is the repository's name, from Cargo.toml's `repository`; it is
# also the binary's own name and the cargo target's, so no build-system name
# is renamed away here. THE BINARY SAYS WHAT ELSE IT IS: the dotted names
# come from `<repo> generate names`, the pages and completions from `<repo>
# generate man|completions`, so this script names no tool.
#
# THEN IT CHECKS WHAT IT BUILT, because a tarball whose tools do not run is
# worse than no tarball: the failure would surface as a user's bug report
# rather than a red release. Every path above must be there and nothing
# outside bin/, share/ and LICENSE; every dotted name must be a relative
# symlink to bin/<repo>; every name must answer --help and report `<name>
# (<crate>) <version>` from --version, which identifies it among same-named
# tools from other packages and catches a tag that disagrees with
# Cargo.toml; and CAVEATS must be one to four lines. On any failure nothing
# is printed on stdout and no tarball is left. tests/scripts/
# test-package-cli.sh holds this script to all of that, and CI's `cli` job
# runs it on every pull request, so the release is not the first time the
# tarball is put together.
set -euo pipefail

version="${1:-}"
label="${2:-}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
target_dir="${3:-$root/target/release}"
licences=(LICENSE)

die() { echo "package-cli: $*" >&2; exit 1; }

[ -n "$version" ] || die "usage: package-cli.sh <version> <label> [target-dir]"
[ -n "$label" ] || die "usage: package-cli.sh <version> <label> [target-dir]"

crate="$(sed -n 's/^name = "\(.*\)"$/\1/p' "$root/Cargo.toml" | head -n 1)"
[ -n "$crate" ] || die "no package name in $root/Cargo.toml"
repo="$(sed -n 's/^repository = "\(.*\)"$/\1/p' "$root/Cargo.toml" | head -n 1)"
repo="${repo%/}"
repo="${repo##*/}"
[ -n "$repo" ] || die "no repository in $root/Cargo.toml"

tarball="$crate-$version-$label.tar.gz"
work="$(mktemp -d)"

# ON ANY FAILURE, NO TARBALL: not a partial one, and not one a previous run
# left under the same name, which a caller could otherwise take for this
# run's output.
cleanup() {
    local status=$?
    rm -rf "$work"
    [ "$status" -eq 0 ] || rm -f "$tarball"
    return "$status"
}
trap cleanup EXIT

built="$target_dir/$repo"
[ -x "$built" ] || die "no built $repo at $built (cargo build --release --locked --features cli --bin $repo)"

stage="$work/stage"
mkdir -p "$stage/bin" "$stage/share/$repo" "$work/unpacked"
cp "$built" "$stage/bin/$repo"
chmod 755 "$stage/bin/$repo"
names="$("$stage/bin/$repo" generate names)" || die "$repo generate names failed"
[ -n "$names" ] || die "$repo generate names listed no tools"
for name in $names; do
    ln -s "$repo" "$stage/bin/$name"
done
"$stage/bin/$repo" generate man "$stage/share" >/dev/null || die "$repo generate man failed"
"$stage/bin/$repo" generate completions "$stage/share" >/dev/null || die "$repo generate completions failed"
cp "$root/packaging/CAVEATS" "$stage/share/$repo/CAVEATS"
for f in "${licences[@]}"; do
    cp "$root/$f" "$stage/$f"
done

# What must be there, named here rather than read back from the staging
# directory, so a generator that quietly wrote nothing fails the check.
want=("bin/$repo" "share/$repo/CAVEATS" "${licences[@]}")
for name in $names "$repo"; do
    case "$name" in mkfs.* | fsck.*) section=8 ;; *) section=1 ;; esac
    [ "$name" = "$repo" ] || want+=("bin/$name")
    want+=("share/man/man$section/$name.$section"
        "share/zsh/site-functions/_$name"
        "share/bash-completion/completions/$name"
        "share/fish/vendor_completions.d/$name.fish")
done

# COPYFILE_DISABLE keeps macOS tar from adding ._ AppleDouble members.
COPYFILE_DISABLE=1 tar -czf "$tarball" -C "$stage" bin share "${licences[@]}"

# Members only: whether a tar lists the directories themselves varies by tar.
members="$(tar -tzf "$tarball" | sed 's|^\./||' | grep -v '/$' | sort)"
for path in "${want[@]}"; do
    grep -qxF -- "$path" <<<"$members" || die "$tarball has no $path"
done
stray="$(grep -vE '^(bin/|share/|LICENSE$)' <<<"$members" || true)"
[ -z "$stray" ] || die "$tarball holds files outside bin/, share/ and LICENSE: $(echo $stray)"

tar -xzf "$tarball" -C "$work/unpacked"
caveats="$work/unpacked/share/$repo/CAVEATS"
lines="$(wc -l <"$caveats" | tr -d ' ')"
[ -s "$caveats" ] && [ "$lines" -le 4 ] || die "share/$repo/CAVEATS must be one to four lines, is $lines"
[ -x "$work/unpacked/bin/$repo" ] && [ ! -L "$work/unpacked/bin/$repo" ] ||
    die "bin/$repo is not an executable file in $tarball"
for name in $names "$repo"; do
    exe="$work/unpacked/bin/$name"
    if [ "$name" != "$repo" ]; then
        [ -L "$exe" ] || die "bin/$name is not a symlink in $tarball"
        [ "$(readlink "$exe")" = "$repo" ] || die "bin/$name links to '$(readlink "$exe")', not '$repo'"
    fi
    "$exe" --help >/dev/null || die "$name --help failed"
    reported="$("$exe" --version)" || die "$name --version failed"
    [ "$reported" = "$name ($crate) $version" ] ||
        die "$name --version says '$reported', expected '$name ($crate) $version'"
done

printf '%s\n' "$tarball"
