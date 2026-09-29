#!/usr/bin/env bash
# scripts/check-provenance.sh recognises every kind of name it refuses, and
# lets the documented format names through.
#
# Without this a pattern that matched nothing -- a typo, a lookbehind that
# excludes too much -- would pass the real tree having checked nothing. So a
# sandbox repository is planted with one line of each kind, beside lines
# that must NOT be reported, and the report is compared line by line.
#
# This file spells the denied names, so the scan exempts it; it is the only
# file besides the script itself allowed to.
#
#   bash tests/scripts/test-check-provenance.sh
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECK="$REPO/scripts/check-provenance.sh"

fail() { echo "FAIL  $*" >&2; exit 1; }

mkdir -p "$REPO/tmp"
SANDBOX="$(mktemp -d "$REPO/tmp/check-provenance.XXXXXX")"
trap 'rm -rf "$SANDBOX"' EXIT HUP INT TERM

git -C "$SANDBOX" init -q
mkdir -p "$SANDBOX/src" "$SANDBOX/docs" "$SANDBOX/tests/oracle-reports"

# --- what must be reported, one kind per line ---------------------------
cat > "$SANDBOX/src/bad.rs" <<'EOF'
//! Per `fs/ext4/extents.c` the tail is last.
//! See https://github.com/torvalds/linux/blob/master/fs/jbd2/recovery.c
//! The rule `ext4_ext_correct_indexes` applies.
//! As the kernel's `__ext4_read_dirblock` does.
//! Replay as `do_one_pass` does.
//! This is the kernel's `EXT4_FITS_IN_INODE`.
//! Called as EXT4_BLOCK_SIZE(sb) there.
//! extra = ((time->tv_sec - (s32)time->tv_sec) >> 32) & 3;
//! Read with le32_to_cpu before use.
//! e2fsprogs opens it in lib/ext2fs/openfs.c first.
//! The journal code in include/linux/jbd2.h.
//! The seed is the kernel's `__crc32c_le(~0, uuid, 16)`.
EOF
expect=(
    "src/bad.rs:1: path "
    "src/bad.rs:2: path "
    "src/bad.rs:3: ident "
    "src/bad.rs:4: ident "
    "src/bad.rs:5: name "
    "src/bad.rs:6: macro "
    "src/bad.rs:7: macro "
    "src/bad.rs:8: c-api "
    "src/bad.rs:9: c-api "
    "src/bad.rs:10: path "
    "src/bad.rs:11: path "
    "src/bad.rs:12: name "
)

# --- what must NOT be reported ------------------------------------------
# Documented format names, the documentation's own URL, this crate's C ABI
# prefix, uppercase flag names, tool output under its allowlisted path, and
# a token that is one of the repository's own file names.
cat > "$SANDBOX/src/good.rs" <<'EOF'
//! A leaf starts with `struct ext4_extent_header`, then `ext4_extent`s,
//! and ends in `ext4_extent_tail`; a directory block in `ext4_dir_entry_2`
//! records. `EXT4_EXTENTS_FL`, `JBD2_FEATURE_INCOMPAT_CSUM_V3`,
//! `s_inodes_count`, `dx_root`, `journal_header_t`.
//! https://docs.kernel.org/filesystems/ext4/dynamic.html
//! fs_ext4_mount and fs_ext4_stat are this crate's C ABI.
//! See tests/jbd2_basic.rs for the journal.
pub const EXT4_LINK_MAX: u16 = 65000;
EOF
printf 'ext2fs_open2: Bad magic number in super-block\n' \
    > "$SANDBOX/tests/oracle-reports/e2fsck_zeros.txt"
printf 'fn main() {}\n' > "$SANDBOX/tests/jbd2_basic.rs"

# --- an untracked file is not the repository's ---------------------------
printf 'fs/ext4/inode.c\n' > "$SANDBOX/docs/untracked.md"

git -C "$SANDBOX" add src/bad.rs src/good.rs tests/oracle-reports/e2fsck_zeros.txt \
    tests/jbd2_basic.rs

found="$(PROVENANCE_ALLOW_UNUSED=1 bash "$CHECK" "$SANDBOX" 2>&1)"
status=$?
[[ $status -eq 1 ]] || fail "the check exited $status, expected 1:"$'\n'"$found"

for e in "${expect[@]}"; do
    grep -qF "  $e" <<<"$found" || fail "the check missed '$e':"$'\n'"$found"
done
# One report per name, so count the distinct lines named.
reported="$(grep -oE '^  [^ ]+:[0-9]+: ' <<<"$found" | sort -u | grep -c .)"
[[ "$reported" -eq ${#expect[@]} ]] ||
    fail "the check reported $reported lines, expected ${#expect[@]}:"$'\n'"$found"

# --- an allowlist entry nothing uses is itself a failure -----------------
git -C "$SANDBOX" rm -q --cached src/bad.rs
rm "$SANDBOX/src/bad.rs"
found="$(bash "$CHECK" "$SANDBOX" 2>&1)"
status=$?
[[ $status -eq 1 ]] && grep -qF "match nothing" <<<"$found" ||
    fail "an unused allowlist entry was not reported (status $status):"$'\n'"$found"

# --- and with that lifted, the clean sandbox passes -----------------------
found="$(PROVENANCE_ALLOW_UNUSED=1 bash "$CHECK" "$SANDBOX" 2>&1)" ||
    fail "a sandbox holding only format names failed:"$'\n'"$found"

echo "PASS  check-provenance reports every kind it refuses and nothing it allows"
