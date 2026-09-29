#!/usr/bin/env bash
# No tracked file names Linux kernel or e2fsprogs internals.
#
#   scripts/check-provenance.sh [ROOT]      # ROOT defaults to this repository
#
# This crate is written from public sources only: the kernel.org ext4 and
# JBD2 format documentation, RFCs and papers, permissively licensed code, and
# the observed behaviour of e2fsprogs and a real kernel used as black-box
# oracles (AGENTS.md, "Clean room"). A comment that says a routine "mirrors
# ext4_foo() in fs/ext4/bar.c" makes the code read as derived from GPL source
# whether or not it is, and it tells the next reader to go and open that
# source. So this reads EVERY tracked text file -- source, tests, scripts,
# docs, workflows, the changelog -- and fails on any line that names:
#
#   path      a kernel or e2fsprogs source path, or a link to one;
#   ident     an ext4_ / ext4fs_ / jbd2_ / ext2fs_ / e2p_ style identifier
#             that is not a documented on-disk format name (see ALLOW);
#   name      a kernel function with no such prefix (do_split, ...);
#   macro     a kernel-internal macro (EXT4_FITS_IN_INODE, ...), or any
#             EXT4_/EXT2_/JBD2_ name used like a function-like macro;
#   c-api     kernel C API spelling (le32_to_cpu, buffer_head, ->tv_sec).
#
# WHAT IS ALLOWED, AND WHY. The on-disk format has names, and the public
# documentation uses them: `struct ext4_extent_header`, `journal_header_t`,
# `s_inodes_count`, `EXT4_EXTENTS_FL`, `JBD2_FEATURE_INCOMPAT_CSUM_V3`. Those
# describe the bytes, not anybody's code, and every other ext4 implementation
# uses them too. Uppercase flag and feature names are therefore not scanned
# at all, except the explicit kernel-internal ones below. Lowercase
# identifiers are scanned by prefix and pass only through ALLOW, where each
# entry says why it is a format name or a name of our own. A kernel FUNCTION
# name is never allowed, even where the documentation mentions one in
# passing: describe the behaviour, and cite the section of the documentation
# or the oracle that shows it.
#
# THE LISTS LIVE HERE AND NOWHERE ELSE. This file and its test
# (tests/scripts/test-check-provenance.sh) are the only files allowed to
# spell the denied names, and the only ones the scan does not read.
#
# AN ALLOW ENTRY NOTHING USES FAILS the check, so the list cannot quietly
# grow exemptions for names that have left the tree. The whole-file
# exemptions (EXEMPT) are not held to that: they name documents such as the
# provenance audit, which must say what it compared against.
# PROVENANCE_ALLOW_UNUSED=1 lifts it, for the script's own test.
set -uo pipefail

ROOT="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
command -v python3 >/dev/null || { echo "check-provenance: python3 is required" >&2; exit 2; }
git -C "$ROOT" rev-parse --git-dir >/dev/null 2>&1 ||
    { echo "check-provenance: $ROOT is not a git work tree" >&2; exit 2; }

exec python3 - "$ROOT" <<'PY'
import fnmatch, os, re, subprocess, sys

root = sys.argv[1]
allow_unused = os.environ.get("PROVENANCE_ALLOW_UNUSED") == "1"

# ---------------------------------------------------------------------------
# DENY. Each rule is (kind, regex). Matching is case-sensitive.
# ---------------------------------------------------------------------------
B = r"(?<![A-Za-z0-9_])"          # not preceded by an identifier character
P = r"(?<![A-Za-z0-9_./-])"       # not preceded by a path character

DENY = [
    # Kernel source trees. `filesystems/ext4/` (the documentation) does not
    # match: the rule needs `fs/` to start a path component.
    ("path", P + r"(?:linux/)?fs/(?:ext[234]|jbd2?|unicode|crypto|verity)\b"),
    ("path", P + r"include/(?:uapi/)?linux/"),
    ("path", P + r"lib/(?:crc16|crc32c?|unicode|ext2fs|e2p)\b"),
    # e2fsprogs source files, by directory.
    ("path", P + r"(?:e2fsck|misc|debugfs|resize|lib/ext2fs)/[A-Za-z0-9_-]+\.[ch]\b"),
    # Links to either source tree.
    ("path", r"torvalds/linux|git\.kernel\.org|elixir\.bootlin\.com"
             r"|tytso/e2fsprogs|e2fsprogs\.git|lxr\.[a-z.-]+/"),

    ("ident", B + r"(?:__)?(?:ext4|ext4fs|jbd2|ext2fs|e2p)_[a-z0-9_]+"),

    # Kernel functions whose names carry no ext4 prefix. Add to this list
    # when one turns up; a name here is banned everywhere.
    ("name", B + r"(?:do_one_pass|scan_revoke_records|journal_check_superblock"
                 r"|journal_tag_bytes|need_check_commit_time|descriptor_loc"
                 r"|do_split|dx_insert_block|dx_fallback|dx_probe|dx_make_map"
                 r"|get_dx_countlimit|fs_umode_to_ftype|utf8_casefold_hash"
                 r"|utf8_casefold|(?:__)?crc32c_le|str2hashbuf|dx_hack_hash"
                 r"|TEA_transform|half_md4_transform)\b"),

    # Kernel-internal macros: helpers and limits that live only in the
    # kernel's (or e2fsprogs') headers, not in the format documentation.
    ("macro", B + r"(?:EXT4_FITS_IN_INODE|EXT4_E?INODE_[GS]ET_XTIME"
                  r"|EXT4_INLINE_DOTDOT_SIZE|EXT4_DIR_LINK_MAX"
                  r"|EXT4_MAX_CLUSTER_LOG_SIZE|EXT4_MIN_BLOCK_LOG_SIZE"
                  r"|EXT4_CASEFOLD_HASH_SEED_SLOT|EXT2_MAX_BLOCKS_PER_GROUP"
                  r"|EXT4_FT_DIR_CSUM|EXT4_SB|EXT4_I"
                  r"|EXT_(?:FIRST|LAST)_(?:INDEX|EXTENT)|EXT_MAX_(?:INDEX|EXTENT))\b"),
    # Any format-prefixed name written as a macro call: EXT4_BLOCK_SIZE(sb).
    ("macro", B + r"(?:EXT[234]|JBD2)_[A-Z0-9_]+\((?!\))"),

    ("c-api", B + r"(?:le(?:16|32|64)_to_cpu|cpu_to_le(?:16|32|64)"
                  r"|buffer_head|sb_bread|brelse|mark_buffer_dirty)\b"),
    ("c-api", r"->tv_sec\b"),
]

# ---------------------------------------------------------------------------
# ALLOW. (path glob, token regex, justification). The token is the whole
# matched text and must match the regex in full. `*` as the glob means every
# file. Only `ident` hits are ever allowed; the other kinds have no
# legitimate use.
# ---------------------------------------------------------------------------
FORMAT_DOC = ("named as an on-disk structure in the kernel.org ext4 format "
              "documentation (docs.kernel.org/filesystems/ext4/)")
ALLOW = [
    ("*", r"ext4_super_block|ext4_group_desc", FORMAT_DOC),
    ("*", r"ext4_extent_header|ext4_extent_idx|ext4_extent|ext4_extent_tail", FORMAT_DOC),
    ("*", r"ext4_dir_entry_2|ext4_dir_entry_tail|ext4_extended_dir_entry_2", FORMAT_DOC),
    ("*", r"ext4_xattr_entry", FORMAT_DOC),
    ("*", r"jbd2_journal_block_tail", FORMAT_DOC),

    ("*", r"ext4_rs",
     "the name of a separately published, MIT-licensed Rust crate cited as "
     "prior art (github.com/yuoo655/ext4_rs)"),

    # Names of this crate's own, derived from its fixtures and modules.
    ("tests/all_images_rw_smoke.rs",
     r"ext4_(?:basic|acl|csum_seed|deep_extents|htree|inline|largedir"
     r"|manyfiles|no_csum|xattr)",
     "one test per fixture, each named after its test-disks/ext4-*.img"),
    ("tests/htree_root_with_kernel_tail_bytes.rs", r"ext4_manyfiles",
     "the all_images_rw_smoke test of that name"),
    ("src/extent.rs", r"ext4_basic_root_inode_has_extent",
     "a unit test here, named after the ext4-basic.img fixture"),
    ("src/fs_core_bridge.rs", r"ext4_to_fs_core_error",
     "this crate's error conversion into fs-core's"),

    # The third implementation the oracle tier cross-validates against is a
    # BSD-licensed C library whose public API happens to use the ext4_
    # prefix. Calling it and quoting its reports is using an oracle.
    ("tests/lwext4_cross_validate.rs", r"ext4_(?:mount|fread|blocks_get_direct)",
     "that library's public API, which this test drives"),
    ("tests/oracle_verdicts.rs", r"ext4_mount", "quotes that library's report"),
    ("tests/support/src/lwext4.rs", r"ext4_mount", "quotes that library's report"),
    ("tests/support/src/verdict.rs", r"ext4_mount", "parses that library's report"),
    ("scripts/vm-setup.sh", r"ext4_config", "a header that library's build generates"),

    # Tool output, quoted as the tool prints it.
    ("tests/oracle-reports/*", r"ext2fs_open2|ext4_mount",
     "reports captured from the oracle tools in the guest, verbatim"),
    ("tests/mkfs_e2fsck_oracle.rs", r"ext2fs_open2",
     "quotes e2fsck's own error message, which begins with that prefix"),

    # A header of an application-side C bridge the smoke test builds against.
    ("tests/c_smoke/*", r"ext4_bridge", "the bridge header's file name"),
]

# Whole files the scan does not read.
EXEMPT = [
    # The lists themselves.
    ("scripts/check-provenance.sh", "this script: it spells what it bans"),
    ("tests/scripts/test-check-provenance.sh", "its test, which plants each kind"),
    # C code written for this crate that drives the third implementation
    # through that library's public ext4_* API; every such name is its API.
    ("tests/lwext4/*", "a C client of the BSD-licensed third implementation"),
    # The provenance record has to say what the audit compared against.
    ("PROVENANCE.md", "the provenance record"),
    ("docs/provenance-audit-*.md", "the provenance audit report"),
]

def exempt(path):
    return any(fnmatch.fnmatchcase(path, g) for g, _ in EXEMPT)

files = subprocess.run(
    ["git", "-C", root, "ls-files", "-z"], check=True, capture_output=True
).stdout.decode("utf-8", "surrogateescape").split("\0")
files = [f for f in files if f and not exempt(f)]

# A token that is the name of one of this repository's own files -- a test
# binary such as tests/jbd2_basic.rs, a fixture -- is this crate's name, not
# the kernel's, wherever it is mentioned.
own_stems = set()
for f in files:
    base = os.path.basename(f)
    own_stems.add(base)
    own_stems.add(base.split(".", 1)[0])

deny = [(k, re.compile(r)) for k, r in DENY]
allow = [(g, re.compile(r), why) for g, r, why in ALLOW]
used = [False] * len(allow)
hits = []
seen = set()

for path in files:
    full = os.path.join(root, path)
    if not os.path.isfile(full):
        continue          # a tracked file deleted in the work tree
    with open(full, "rb") as fh:
        data = fh.read()
    if b"\0" in data[:8192]:
        continue          # binary: disk images and the like
    text = data.decode("utf-8", "replace")
    for n, line in enumerate(text.splitlines(), 1):
        for kind, rx in deny:
            for m in rx.finditer(line):
                tok = m.group(0)
                ok = False
                if kind == "ident" and tok in own_stems:
                    ok = True
                elif kind == "ident":
                    for i, (g, arx, _) in enumerate(allow):
                        if (g == "*" or fnmatch.fnmatchcase(path, g)) and arx.fullmatch(tok):
                            used[i] = True
                            ok = True
                            break
                if not ok and (path, n, kind, tok) not in seen:
                    seen.add((path, n, kind, tok))
                    hits.append((path, n, kind, tok, line.strip()))

status = 0
if hits:
    status = 1
    print("check-provenance: these name Linux kernel or e2fsprogs internals.", file=sys.stderr)
    print("  Describe the on-disk fact or the observed behaviour instead, citing the", file=sys.stderr)
    print("  kernel.org format documentation or the oracle that shows it (AGENTS.md,", file=sys.stderr)
    print("  \"Clean room\"). A documented format name can be allowed in", file=sys.stderr)
    print("  scripts/check-provenance.sh, with the reason.", file=sys.stderr)
    for path, n, kind, tok, line in hits:
        print(f"  {path}:{n}: {kind} `{tok}`: {line[:160]}", file=sys.stderr)

unused = [a for a, u in zip(ALLOW, used) if not u]
if unused and not allow_unused:
    status = 1
    print("check-provenance: these allowlist entries match nothing; remove them:", file=sys.stderr)
    for g, r, why in unused:
        print(f"  {g}  {r}  ({why})", file=sys.stderr)

if status == 0:
    print(f"check-provenance: {len(files)} tracked files name no kernel or e2fsprogs internals")
sys.exit(status)
PY
