# Ordinary full-intermediate-node Linux reference

## Boundary review — 2026-11-06

The preceding last-slot matrix leaves a healthy ordinary index with one full
intermediate node (126/510 routes at 1/4 KiB). The next leaf split must also
handle routing-parent capacity. The driver's existing private split helper
returns `Ok(false)` before staging when node insertion cannot fit; its public
caller then drops INDEX, converts root/interior spanning records and appends
one linear leaf in the same buffer. The mount operation remains retained
through preparation, optional-superblock staging, commit and publication.
Corrupt selected leaf records/checksums propagate errors before fallback.
Casefold index dropping and casefold writes remain refused.

That trace does not establish Linux's growth representation or qualify the
driver's public full-node fallback. This reference-only checkpoint first
measures Linux at the exact full-node predecessor. It changes no production
path and opens reference images read-only in Rust.

## Bounded measurement contract

- Four ordinary ext4 layouts: 32 MiB, 1/4 KiB blocks, 256-byte inodes,
  FILETYPE/metadata checksums/extents/dir_index, direct or journaled. Reuse the
  independently checked root promotion and last-slot preparation under a unique
  scratch prefix, preserving all preceding test behavior and scratch names.
- Seek the next root-count change with at most 128 additional single aliases,
  retaining the exact predecessor before every call. Preserve the existing
  preparation bounds (16,384 live names, 256-link batches/eight bisections,
  64 last-slot seeks), 1,024 directory blocks and four external extent nodes.
  Unexpected flags, depth, roles or exhausted bounds fail with retained evidence.
- Require one full intermediate node immediately before the triggering link;
  measure a retained one-intermediate-level index with two root routes/two
  intermediate nodes afterward, one new node and one new leaf. Verify every
  old spelling/inode, complete hash routing and old physical map prefix; record
  actual child counts rather than inferring a split ratio.
- Preserve old global leaf route boundaries, except for the added leaf boundary.
  Normalize each child node's first implicit hash with its parent root boundary
  when comparing the flattened route inventories. Root changes are limited to
  the count, new route and checksum; only the selected full old leaf and old
  intermediate node may change among old directory blocks.
- Account for two directory blocks and any separately observed extent-map node,
  exact bitmap/group/global debit, no inode allocation, parent/source metadata
  and version changes, and every protected table slice, EA, payload and root
  directory byte. Parent and source must actually share an inode-table block.
- Retain full-node/grown/cold phase images, judged non-repairing e2fsck, debugfs
  inode/htree reports, raw inode bytes, full role/map/inventory manifests and
  Linux guest JSON. Together with six preparation phases this is 36 saved
  images across four profiles. Cold Linux must find every alias and agree on
  exact metadata/flags/EAs/payloads/free counts while protected bytes stay equal.
- Run one exact focused CI test with floor one, existing tool/sibling pins,
  complete source hashes and always-retained evidence. Preserve all earlier
  jobs, selections, floors and ceilings. Any actual whole-log overflow requires
  saved-log identical old-red/new-green replay before a ceiling change.

Only the prescribed CI guest runs Linux/e2fsprogs or mounts. No local KVM, host
oracle/mount, or kernel/e2fsprogs implementation source is used. Saved bytes and
source-bound complete-suite retention must be reviewed before closure. Ordinary
public fallback emission, deeper writes, general ownership/admission, lifecycle/
recovery, ignored fuzz and Windows/SteamOS remain separate. Stages 4–5 stay
partial/not quantified and casefold writes remain blocked.

## Status

Boundary reviewed and the Linux-only reference/exact focused job implemented.
Compilation, formatting, full lint/Unicode and all 1,681 native-unit occurrences
pass with an identical baseline multiset, within unchanged limits. Semantic YAML
comparison preserves all 21 preceding jobs and top-level settings; three workflow
guards pass. Independent Linux execution and raw evidence review remain pending.
Baseline `bbda780` passed focused 42264765485/full 42264765536 with
2,538 whole successes and 1,681 units. Latest docs `a0b700a` have byte-identical
validated source inputs; their full follow-up 42277073283 is still in progress.
