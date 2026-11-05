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

The reference is prepared, with local compilation/syntax/native validation
recorded in `extprogress.md`. Actual root capacity, triggering name/count,
allocation layout and depth-transition results are pending independent CI and
saved-evidence review. None of the expected outcomes above is credited as a
measured Linux result yet. Preserve every existing write and platform gate.
