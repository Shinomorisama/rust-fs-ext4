# Ordinary Linux root-capacity reference

This Stage 0 prerequisite extends the partial Stage 4–5 evidence. It creates
and mutates scratch filesystems only through the existing harness guest; the
Rust driver opens them read-only. It does not qualify driver fallback, deeper
driver writes, recovery, ownership admission or casefold write activation.

## Profile and resource limits

- Four 32 MiB ordinary ext4 profiles: 1/4 KiB blocks, 256-byte inodes, FILETYPE,
  metadata checksums, extent maps and `dir_index`, direct or journaled.
- At most 16,384 distinct 255-byte aliases to one regular source inode per
  profile, below the link-count limit. The initial index must appear within
  128 names. No per-alias inode or payload allocation is requested.
- At most 64 growth batches of at most 256 links. On the first observed depth
  change, replay the exact batch input and bisect at most eight times. Replay
  the last root-only prefix, then perform exactly one triggering link.
  Including replayed prefixes, at most 18,689 successful alias-link calls are
  requested per profile; the live-name limit above counts distinct names.
- At most 1,024 mapped directory blocks, with each classified exactly once as
  root, interior index or leaf. Htree depth is zero/one; extent-map depth is
  zero/one with at most four external map blocks. All map/data block addresses
  must be unique, in filesystem bounds, fully mapped and checksummed.
- Retain four 32 MiB phase images per profile (512 MiB total before compression):
  initial index, exact capacity input, grown output and cold readback. Reusable
  working/batch/replay/cold images add at most 128 MiB while a profile runs and
  remain available on failure. Search records and guest JSON accompany images.

These limits deliberately exclude larger/deeper maps, collision continuation
boundaries, malformed inputs, uninitialized-group transitions, exhaustion and
interrupted writes. Reaching a limit without the required observation fails;
there is no skip or silently substituted fixture.

## Observation and evidence contract

1. Linux creates the directory, source and protected peer, long aliases,
   external audit EAs and payloads. The source and parent must actually share
   an inode-table block. Record the initial root-only index and complete state.
2. Grow in batches while retaining the exact current batch input. Record each
   actual root depth/count/limit and source link count. An unexpected INDEX drop,
   refusal or structure fails with the search trace, stdout/stderr and images
   retained. This positive reference must observe a root-only full root followed
   by its first one-intermediate-level state, rather than presuming Linux uses
   this driver's ordinary index-drop fallback.
3. Locate the exact triggering link by prefix replay. Require the preceding
   count to equal the real root limit. The final single link must retain all
   existing names, inodes and mappings; add one leaf and one interior directory
   block; and route every live entry to its actual leaf with the effective
   filesystem hash version/seed. The complete root/node graph must classify
   every mapped directory block, with no repeated/unreferenced blocks.
4. Account for every new directory or extent-map block, including an optional
   extent-root promotion. Require two or three newly allocated physical blocks,
   exact group/global free-block debits and previously free bitmap bits, unchanged
   inode allocation, and matching size/sector accounting. Preserve protected
   inode-table slices, source/peer payloads, external EAs and root-directory
   bytes; allow only the qualified parent/source metadata changes.
5. Retain judged non-repairing e2fsck, debugfs inode and htree reports, raw tracked
   inode records, complete logical/physical/role/inventory manifests and guest
   observations for the four phases. A separate cold Linux observer must stat
   every expected spelling to the source inode and check exact tracked metadata,
   flags, payloads, EAs and free counts. Its tables, directory bytes, extent nodes,
   allocator and protected data must equal the grown image afterward.

## Execution status

The exact one-test focused job `link-index-capacity-reference` pins the existing
siblings and Rust toolchain, records the source commit and all tracked
source/test-support/dependency/workflow hashes, enforces a one-test floor and
keeps evidence even on failure. Existing jobs, selection, floors and caps are
unchanged; the initial focused ceiling reuses 4,200 lines/340,000 bytes from the
existing bounded reference tier. Whole-suite output must be measured in CI.

## Exact-source Linux observation and saved-byte audit — 2026-11-05

Source **`3a131c93c7531d02c3ee5177097c5854f24bad15`** passes the exact focused
test in run **42098063990**, job **127493234974**. Its actual wrapper passes
at **297 lines/33,323 bytes**, with one executed test/floor one and zero failures.
Artifact **13449630990** has API-advertised and locally verified ZIP SHA-256
**`9d9dd49ae76053f7aaae63bd96b716c02117033b2d13f19c83fc9883a9610c5e`**.
ZIP CRCs pass. The retained source commit and all **127** unique source hashes
match the exact Git tree, with no missing or extra paths in the selected set.

The independent saved-byte audit verifies all four layouts and sixteen images:

| Blocks / journal | Root entries at limit | Aliases immediately before → after | Directory blocks before → after | New physical blocks | Existing external extent nodes |
|---|---:|---:|---:|---|---|
| 1 KiB / no | 123 | 246 → 247 | 124 → 126 | 8833, 8834 | None |
| 1 KiB / yes | 123 | 245 → 246 | 124 → 126 | 16967, 16968 | 17414 |
| 4 KiB / no | 507 | 5357 → 5358 | 508 → 510 | 779, 780 | 1041 |
| 4 KiB / yes | 507 | 5369 → 5370 | 508 → 510 | 1803, 1804 | 2065 |

Each trigger is exactly the next zero-based name: ASCII `cap-{n:05d}-` followed
by 245 literal `n` bytes, where `n` is the table's before-alias count. The
preceding root is full; that single link retains INDEX and promotes depth zero
to one, leaving one root entry, one interior node and one additional leaf.
These exact predecessor counts depend on the saved filesystem's hash seed and
name distribution; they are not universal directory-entry thresholds. All
four transitions allocate exactly two directory blocks, preserve the previous
logical-to-physical prefix and external extent-node addresses, and debit two
previously free bitmap bits with no inode allocation. No extent-root promotion
or new extent-map allocation was observed in these four transitions.

The primary audit parses raw superblocks, descriptors, initialized bitmaps,
tracked/root inodes, extent trees, dx nodes and leaf records; verifies CRC32C
checksums; and reconstructs the complete mapping/role graph and every stored
name. It matches all **33,699 inventory rows** across sixteen phases to raw
entries and to guest observations. For every name, saved debugfs major hashes
select its actual leaf through the raw root/node boundaries, providing an
oracle comparison independent of the Rust lookup assertions. The sixteen
non-repairing e2fsck reports contain all five passes; sixteen htree dumps,
48 tracked debugfs reports and 48 raw inode records agree with the images.

Exact global/group free counts and bitmap flips, complete inode-table slices,
external EAs, payloads and root-directory bytes satisfy the observation contract.
Unused INODE_UNINIT group flags/bytes remain unchanged; their absent inode-bitmap
checksums are not treated as initialized checksums. Cold observers match all
48 tracked metadata rows, flags, EAs, payloads, spellings and free counts.
Directory/extent nodes, complete inode tables, allocation metadata and protected
data remain byte-identical between grown and cold images. Journals are clean.
The successful alias-call counts including replays are **2,264 / 2,261 / 7,245 /
7,345**, all below the stated per-profile bound.

The read-only audit uses no host oracle or mount. Its scratch script
`/tmp/index-capacity-ci/audit_capacity.py` has SHA-256
`8cdbeceb17d58fd7c0ee8405b1bd231b88f37e3b8aac1c4b29a5cd2724fe72f3`;
its passing JSON report has SHA-256
`84ba90066d0d71f9ee1c8117b3acc8bb1cf1812fa1c5106f33defd39c040733c`.
A Terra medium helper independently checks identity, report completeness and
actual wrapper/floor evidence; the primary repeats consequential identity checks
and performs the filesystem audit. Helper usage/cost data is unavailable.

The exact full run's unit artifact **13449340328**, SHA-256
`eab73c674d6fdd57dfe58348b79b1f78f5f48c7fe122605f94b9ba56dfb53f94`,
passes advertised digest/ZIP CRC checks and preserves the complete baseline
multiset: **1,669** successes, **1,666** distinct names, all eighteen native
`indexed_boundary` controls once. The primary corrects an over-restrictive
helper name filter and confirms these controls by their actual module prefix.

Full run **42098064190** and the remaining focused jobs are still running at
this checkpoint's inspection. Complete-suite retention, actual whole-output
budgets and aggregate completion remain separate. This closes only the bounded
Linux observation/saved-byte review. Driver root-capacity behavior, deeper
emissions, ownership/admission, recovery and casefold write activation remain
unqualified. Preserve every existing write and platform gate.
