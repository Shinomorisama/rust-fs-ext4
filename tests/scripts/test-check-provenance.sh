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
# PROVENANCE_CHECK runs these cases against another copy of the script, to
# show which of them an older version missed.
CHECK="${PROVENANCE_CHECK:-$REPO/scripts/check-provenance.sh}"

fail() { echo "FAIL  $*" >&2; exit 1; }

mkdir -p "$REPO/tmp"
SANDBOX="$(mktemp -d "$REPO/tmp/check-provenance.XXXXXX")"
trap 'rm -rf "$SANDBOX"' EXIT HUP INT TERM

git -C "$SANDBOX" init -q
mkdir -p "$SANDBOX/src" "$SANDBOX/docs" "$SANDBOX/scripts" \
    "$SANDBOX/tests/oracle-reports" "$SANDBOX/tests/lwext4"

# --- what must be reported, one kind per line ---------------------------
# The C lines are made up for the test; none is quoted from anywhere.
cat > "$SANDBOX/src/bad.rs" <<'EOF'
//! Per `fs/ext4/extents.c` the tail is last.
//! See https://github.com/torvalds/linux/blob/master/fs/jbd2/recovery.c
//! The rule `ext4_ext_correct_indexes` applies.
//! As the kernel's `__ext4_read_dirblock` does.
//! Replay as `do_one_pass` does.
//! This is the kernel's `EXT4_FITS_IN_INODE`.
//! Called as EXT4_BLOCK_SIZE(sb) there.
//! Take the seconds from ts->tv_sec first.
//! Read with le32_to_cpu before use.
//! e2fsprogs opens it in lib/ext2fs/openfs.c first.
//! The journal code in include/linux/jbd2.h.
//! The seed is the kernel's `__crc32c_le(~0, uuid, 16)`.
//! Ordered as the kernel's tid_gt orders them.
//! Looked up by xattr_find_entry(..., sorted=1).
//! Seeded from j_csum_seed.
//! The kernel's IS_LAST_ENTRY: a zero word.
//! e2fsck calls it PR_2_SET_FILETYPE and flags it PR_NO_NOMSG.
//! Rebuilt the way e2fsck_rehash_dir does it.
//! Capped at EXT4_LINK_MAX.
//! Refused with EFSCORRUPTED, or EFSBADCRC on a bad checksum.
//! Hashed as ext4_xattr_rehash does, a name a file below is named after.
//! n = (u32)x + 1;
//! lo = p->s_first;
//! for (i = 0; i < n; i++) sum += i;
//! while (n) { n >>= 1; }
//! if (count--) step();
//! return -EIO;
//! goto out;
//! if (unlikely(!p)) step();
//! #define ROUND(x) ((x) + 1)
//! #include <linux/types.h>
//! struct foo *bar = NULL;
//! static int helper(void)
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
    "src/bad.rs:8: c-code "
    "src/bad.rs:9: c-api "
    "src/bad.rs:10: path "
    "src/bad.rs:11: path "
    "src/bad.rs:12: name "
    "src/bad.rs:13: name "
    "src/bad.rs:14: name "
    "src/bad.rs:15: name "
    "src/bad.rs:16: macro "
    "src/bad.rs:17: macro "
    "src/bad.rs:18: ident "
    "src/bad.rs:19: macro "
    "src/bad.rs:20: macro "
    "src/bad.rs:21: ident "
    "src/bad.rs:22: c-code "
    "src/bad.rs:23: c-code "
    "src/bad.rs:24: c-code "
    "src/bad.rs:25: c-code "
    "src/bad.rs:26: c-code "
    "src/bad.rs:27: c-code "
    "src/bad.rs:28: c-code "
    "src/bad.rs:29: c-code "
    "src/bad.rs:30: c-code "
    "src/bad.rs:31: c-code "
    "src/bad.rs:32: c-code "
    "src/bad.rs:33: c-code "
)

# A file named after a kernel function does not make that name this
# repository's own: the file name is itself reported, and the mention on
# src/bad.rs:21 above is too.
printf 'fn main() {}\n' > "$SANDBOX/tests/ext4_xattr_rehash.rs"
expect+=("tests/ext4_xattr_rehash.rs:0: ident ")

# The C client of the third implementation may call that library's API,
# and nothing else: a kernel path or function in it is reported.
cat > "$SANDBOX/tests/lwext4/report.c" <<'EOF'
/* fs/ext4/namei.c: ext4_add_entry */
ext4_fopen(&f, "/x", "rb");
n = (u32)x;
EOF
expect+=("tests/lwext4/report.c:1: path " "tests/lwext4/report.c:1: ident ")

# C in a script's comments and in Markdown.
printf '#!/usr/bin/env bash\n# n = (u32)x;\nfor ((i = 0; i < 3; i++)); do :; done\n' \
    > "$SANDBOX/scripts/bad.sh"
cat > "$SANDBOX/docs/bad.md" <<'EOF'
A cast in prose: (s32)secs.

```c
int x;
```
EOF
expect+=("scripts/bad.sh:2: c-code " "docs/bad.md:1: c-code " "docs/bad.md:3: c-code ")

# The provenance record may name what the audit compared against, but it
# is still read for C.
cat > "$SANDBOX/PROVENANCE.md" <<'EOF'
The audit compared against fs/ext4/hash.c and ext4fs_dirhash.

```c
u32 a;
```
EOF
expect+=("PROVENANCE.md:3: c-code ")

# --- what must NOT be reported ------------------------------------------
# Documented format names, the documentation's own URL, this crate's C ABI
# prefix and its C ABI in prose, uppercase flag names, tool output under its
# allowlisted path, references to the repository's own files, Rust in
# comments, and C in the C ABI documentation's C code blocks.
cat > "$SANDBOX/src/good.rs" <<'EOF'
//! A leaf starts with `struct ext4_extent_header`, then `ext4_extent`s,
//! and ends in `ext4_extent_tail`; a directory block in `ext4_dir_entry_2`
//! records. `EXT4_EXTENTS_FL`, `JBD2_FEATURE_INCOMPAT_CSUM_V3`,
//! `s_inodes_count`, `dx_root`, `journal_header_t`.
//! https://docs.kernel.org/filesystems/ext4/dynamic.html
//! fs_ext4_mount and fs_ext4_stat are this crate's C ABI.
//! See tests/jbd2_basic.rs for the journal, or (tests/
//! jbd2_basic.rs) where a comment wraps the path.
//! `cfg->block_size` and `opts->on_finding` are set by the caller.
//! Link libfs_ext4.a and #include "fs_ext4.h"; #define FS_EXT4_X 1.
//! Mode bits: OTHER(r--). A count (u32) and a `fn f() -> u32`.
//! for x in 0..n { step(x); }
pub const MAX_LINKS: u16 = 65000; // (u32) wide enough
EOF
printf 'ext2fs_open2: Bad magic number in super-block\n' \
    > "$SANDBOX/tests/oracle-reports/e2fsck_zeros.txt"
printf 'fn main() {}\n' > "$SANDBOX/tests/jbd2_basic.rs"
cat > "$SANDBOX/README.md" <<'EOF'
Link `libfs_ext4.a` and `#include "fs_ext4.h"`.

```c
cfg->block_size = 4096;
for (i = 0; i < n; i++) fs_ext4_close(h[i]);
```

```rust
fn f() -> u32 { 0 }
```
EOF

# --- an untracked file is not the repository's ---------------------------
printf 'fs/ext4/inode.c\n' > "$SANDBOX/docs/untracked.md"

git -C "$SANDBOX" add src/bad.rs src/good.rs tests/oracle-reports/e2fsck_zeros.txt \
    tests/jbd2_basic.rs tests/ext4_xattr_rehash.rs tests/lwext4/report.c \
    scripts/bad.sh docs/bad.md PROVENANCE.md README.md

found="$(PROVENANCE_ALLOW_UNUSED=1 bash "$CHECK" "$SANDBOX" 2>&1)"
status=$?
[[ $status -eq 1 ]] || fail "the check exited $status, expected 1:"$'\n'"$found"

missed=()
for e in "${expect[@]}"; do
    grep -qF "  $e" <<<"$found" || missed+=("$e")
done
[[ ${#missed[@]} -eq 0 ]] ||
    fail "the check missed ${#missed[@]} of ${#expect[@]}:"$'\n'"$(printf '  %s\n' "${missed[@]}")"$'\n'"$found"
# One report per line and kind, so count the distinct ones named.
reported="$(grep -oE '^  [^ ]+:[0-9]+: [a-z-]+ ' <<<"$found" | sort -u | grep -c .)"
[[ "$reported" -eq ${#expect[@]} ]] ||
    fail "the check reported $reported line kinds, expected ${#expect[@]}:"$'\n'"$found"

# --- an allowlist entry nothing uses is itself a failure -----------------
git -C "$SANDBOX" rm -q --cached src/bad.rs tests/ext4_xattr_rehash.rs scripts/bad.sh \
    docs/bad.md PROVENANCE.md
rm "$SANDBOX/src/bad.rs" "$SANDBOX/tests/ext4_xattr_rehash.rs" "$SANDBOX/scripts/bad.sh" \
    "$SANDBOX/docs/bad.md" "$SANDBOX/PROVENANCE.md"
printf 'ext4_fopen(&f, "/x", "rb");\nn = (u32)x;\n' > "$SANDBOX/tests/lwext4/report.c"
found="$(bash "$CHECK" "$SANDBOX" 2>&1)"
status=$?
[[ $status -eq 1 ]] && grep -qF "match nothing" <<<"$found" ||
    fail "an unused allowlist entry was not reported (status $status):"$'\n'"$found"

# --- and with that lifted, the clean sandbox passes -----------------------
found="$(PROVENANCE_ALLOW_UNUSED=1 bash "$CHECK" "$SANDBOX" 2>&1)" ||
    fail "a sandbox holding only format names failed:"$'\n'"$found"

echo "PASS  check-provenance reports every kind it refuses and nothing it allows"
