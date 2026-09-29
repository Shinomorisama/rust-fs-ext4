# Provenance

This document records where the code in this crate comes from, the rules it
is written under, and the results of provenance audits. It exists so the
crate's permissive licence can be relied on: anyone can check how the code
came to be and what was done when a problem was found.

## History

- **Origins in an application.** The driver started life as the ext4 support
  inside a macOS application that mounts disk images and remote storage as
  Finder volumes through FSKit.
- **A separate library (2026-04-18).** The driver was extracted from that
  application into this repository as a generic, host-independent Rust crate
  with a C ABI. Commit `32061f5` is the first commit here; history before the
  extraction is not part of this repository.
- **Since then** the crate has been developed here as an independent library,
  with its own releases; the application it came from consumes it like any
  other user.

Everything in the crate today, including code that arrived with the initial
import, is covered by the audits below. They examine the code as it exists,
not only its history.

## Licence and sources

The crate is MIT-licensed and has no GPL, LGPL or AGPL dependencies.

The rule for all code written here, including every new contribution, is
that it comes from public, permissively usable sources only:

- the ext4 and JBD2 on-disk format documentation published at
  kernel.org (docs.kernel.org/filesystems/ext4/, the rendered documentation);
- RFCs and papers (RFC 1320 MD4, CRC32C, the TEA paper, the SipHash paper)
  and the Unicode Character Database;
- permissively licensed implementations (BSD/MIT/Apache), with attribution;
- black-box oracles: running `mke2fs`, `e2fsck`, `debugfs`, `dumpe2fs` and a
  real Linux mount, and comparing their outputs and the bytes they write.
  Observing what a tool does is not copying its source.

Linux kernel and e2fsprogs *source code* is not a permitted input. The rule,
with what it forbids and permits in detail, is the "Clean room" section of
`AGENTS.md`, and `scripts/check-provenance.sh` enforces the part of it a
denylist can check.

Releases before 0.6.0 did not meet this rule: the 2026-09-29 audit below
found two exceptions, in `src/hash.rs` and `src/inode.rs`. Both were
remediated before 0.6.0 (see "Remediation" and "Published versions"
below).

## Audit of 2026-09-29

A full provenance audit compared every module against the Linux `fs/ext4`,
`fs/jbd2` and `fs/unicode` sources, as of commit `72d3fcf8` (2026-09-27). The
report, [docs/provenance-audit-2026-09-29.md](docs/provenance-audit-2026-09-29.md),
describes every similarity in prose and contains no kernel source text.

**Result.** The crate is overwhelmingly independent work. Its extent tree,
indirect blocks, xattr layout, checksums, directories and htree, allocator,
mkfs, fsck, journal replay and writer, orphans, casefold folding, ACLs and C
API differ from the kernel in design, and match it only where the on-disk
format requires. Contributions from outside contributors were audited as
well and contain no translated code.

**Findings that must be remediated:**

| Finding | What was found | Remediation |
|---|---|---|
| `src/hash.rs` | The htree name hashes (legacy, half-MD4, TEA) are a translation of the kernel's implementation into Rust, as the module's own documentation and commit `410abe0` state. | Clean-room re-implementation from BSD references, RFC 1320 and the TEA paper, verified against the existing `debugfs`-generated vectors in `tests/htree_hash_vectors.rs`. |
| `src/inode.rs` | Two doc comments quote one line of kernel C verbatim (the extra-time encoding), and the timestamp helpers implement that expression. | Remove the quotes; re-derive the helpers from the kernel.org inode timestamp documentation. |

**Findings to restate for a clean record** (similar in structure to a kernel
routine, but dictated by the format): the xattr entry/block hash, the JBD2
tag-size and checksum-declaration checks, superblock descriptor-location
helpers, index-block classification, and the `BLOCK_UNINIT` bitmap rebuild.
About 60 comments and docs name kernel functions or link to kernel source;
they will be reworded to cite the format documentation or observed oracle
behaviour instead.

**Correctness notes from the audit:** the external xattr block sort order
(#379, fixed in #396) and the extent merge length cap (#387) were already
fixed on `main`; the casefold hash premise is #438.

## Remediation (2026-09-29 – [date merged])

| Item | What was done | Status |
|---|---|---|
| `src/hash.rs` | Re-implemented clean-room. The implementer worked from RFC 1320, the TEA paper, the kernel.org directory documentation, and the BSD-licensed FreeBSD `ext2_hash.c` and lwext4 `ext4_hash.c` (for format facts: constants, byte packing, block sizes, output words), with `debugfs dx_hash` as the oracle. They were barred from the previous implementation, Linux and e2fsprogs source, and the audit report, and the independent verification below confirmed from the record of what they read that they consulted none of them. Verified against 180 fixed and 14,400 random `debugfs` vectors across all six hash versions. One difference from `debugfs` is deliberate: a major hash of `0xFFFFFFFE`, which the directory index reserves and `debugfs` prints unchanged, is given as `0xFFFFFFFC`, as the BSD references do. | done (#451) |
| `src/inode.rs` timestamps | The quoted kernel C line was removed. The timestamp helpers were restated from the kernel.org inode timestamp table, in the same way as the routines in the next row (not clean-room), and pinned to the raw words a Linux kernel writes and reads back for times from 1901 to 2446. | done (#452) |
| Format-driven routines (xattr entry/block hash, JBD2 tag size and checksum-declaration rules, descriptor placement and group-head size, directory block roles, `BLOCK_UNINIT` bitmap) | Restated with a new structure and names by an author who had the previous implementation in view. They were not written clean-room. Each is now checked black-box against e2fsprogs tools and a Linux kernel in a VM. The meaning of `s_first_meta_bg` when it is nonzero, which came over from the previous implementation, was confirmed against volumes the Linux kernel converted to META_BG by online resize, as read by `dumpe2fs`, and is pinned by `tests/group_layout_oracle.rs`. | done (#452) |
| Comments and docs naming kernel internals | Reworded to state the format rule or the observed behaviour. A CI check (`scripts/check-provenance.sh`) rejects kernel and e2fsprogs source paths, links and internal identifiers, in file contents and file names, and C written out in comments and docs. It is a denylist and cannot catch everything. | done (#453) |
| Clean-room rule | Added to `AGENTS.md`. | done (#453) |
| Independent verification | A separate review of the merged result compared the new code with the Linux v6.17 sources and the previous implementation. It found no routine derived from those sources; the resemblances that remain are the ones the on-disk format dictates. The corrections it required before release (five comments still naming kernel or e2fsprogs internals, one comment citing the wrong documentation page, gaps in the CI check) were made before release. | done (2026-09-29) |

### Published versions

| Versions | Where | What they contain | Status |
|---|---|---|---|
| 0.1.0 to 0.3.1 | Git tags `v0.1.0`–`v0.3.1` and a draft GitHub release of `v0.1.0`, under the package name `fs-ext4`, which was never published to crates.io | The previous `src/hash.rs`, whose half-MD4 transform followed the Linux kernel's implementation. | Git tags cannot be yanked. They remain in the repository and carry this code. |
| 0.3.2, 0.3.3, 0.4.0, 0.4.1 | crates.io (`am-fs-ext4`) and their git tags | The same `src/hash.rs`. | To be yanked from crates.io once 0.6.0 is published. |
| 0.5.0, 0.5.1 | crates.io (`am-fs-ext4`) and their git tags | The same `src/hash.rs`, and one line of kernel C quoted in two doc comments in `src/inode.rs`. | To be yanked from crates.io once 0.6.0 is published. |
| 0.6.0 and later | crates.io | None of the above. | Use these. |

Every version before 0.6.0 carries hash code derived from the Linux kernel.
The full transcription of the kernel's hash code (commits `410abe0` and
`00d9121`, 2026-09-17) came after 0.5.1 and was never released. Version 0.6.0
is the first release that contains none of this code.

### Repository history

Git history before the remediation commits still contains the previous
implementations, including the transcription (`410abe0`, `00d9121`) and the
quoted C line (`4b09f56`). History that has been published is not rewritten.
None of it is part of any release from 0.6.0 onwards.
