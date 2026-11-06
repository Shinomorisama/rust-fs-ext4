# Ordinary intermediate-index hard-link qualification

## Reviewed boundary — 2026-11-06

The next five-task batch covers one intermediate index level, ordinary names,
and the split that consumes the selected node's last free route slot. It does
not change production code or enable casefold writes. The existing public link
plan retains the mount operation through preparation, publication and readback.
Insertion validates the selected root, intermediate node and leaf, prepares a
leaf split and a node route insertion, then appends the new leaf in one buffer.
An already-full node makes the private split helper decline without effects;
the caller's existing index-dropping fallback remains a separate qualification.
Corrupt selected leaf records/checksums must propagate errors before fallback.

## Five checkpoints and acceptance contract

1. Review this boundary and record a bounded design before implementing tests.
2. Add six distinct-leaf native controls at 1/4 KiB: full-node private decline,
   Rust/C retained-index last-slot splits, and corrupt selected-leaf refusals.
   Preserve buffer/publication/device/clock/marker/health on decline or error;
   require exact names, mappings, link count and allocation on success.
3. Prepare four Linux profiles (1/4 KiB, checksummed, 256-byte inodes, direct
   and journaled) and feed the exact last-slot predecessor to both public Rust
   and C calls. Retain independent Linux predecessor/output and eight driver
   input/refusal/dirty/closed/cold image sets. Require the full node, retained
   INDEX, unchanged depth/root and one new leaf; validate every route/name,
   checksum, allocation debit, protected inode-table slice, EA and payload.
   Require duplicate/overlong/directory-source refusals without byte/clock
   changes, one successful clock observation and raw-consistent C attributes.
4. Add one pinned focused exact-test/floor-one CI job, preserve all existing
   selections/floors/caps, and run focused plus complete CI at the pushed source.
5. Audit actual source-bound saved images and complete test-name multisets;
   document passed/failed/pending evidence and the remaining qualification gates.

Linux preparation reuses the measured root-promotion reference under a unique
scratch prefix, then seeks the second subsequent leaf split: the first consumes
the penultimate free node slot and the second fills the last slot. Bound this
search to 64 additional aliases, 16,384 total names, 32 MiB images, 1,024 directory
blocks and four external extent nodes; unexpected shapes fail with retained
evidence. Canonical intermediate limits are 126/510 routes at 1/4 KiB. Linux
and driver namespace inventories must agree. Keep the root unchanged, preserve
old physical maps and node addresses, and account for any new extent-map node
separately. Unobserved map promotion is not qualified by a permissive assertion.

Cold Linux must find every spelling/inode and agree on exact metadata, flags,
EAs, payloads and free counts. Use judged non-repairing e2fsck and separate cold
copies; compare protected allocation/table/map/directory bytes after readback.
Only the prescribed CI guest harness may run Linux/e2fsprogs or mounts. No host
oracle, local KVM or kernel/e2fsprogs implementation source is needed or allowed.

Each task is committed and pushed before the next. Actual output overflow, if
any, requires identical saved-log old-red/new-green replay before changing its
ceiling. Native fixtures alone are not Linux-clean evidence. Healthy layouts,
caller exclusion and closed reference handles remain scope assumptions; broader
ownership/admission, lifecycle/recovery, ignored fuzz and platform gates remain.
Stages 4–5 are partial and not quantified; stages 6–7 have not been reached.

## Status

Step 1 records the reviewed design (`d451146`). Step 2's six native controls
pass, and the complete native-only tier retains every previous 1,675 occurrence
plus exactly six: 1,681 successes, 1,929 lines/150,107 bytes within unchanged
limits. Full lint and Unicode verification pass. The first broad local command
incorrectly selected guest-only tests (missing VM/fixtures); the supported
native-only tier passes. Steps 3–5 remain pending. Previous full-root
source `ff0b46f` passed focused 42202840346 and full 42202840602; its docs closure
`6e2de61` also passed full 42213821292. These are baseline evidence, not results
for the new intermediate boundary.
