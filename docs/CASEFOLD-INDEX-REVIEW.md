# Root-only casefold index admission review

Reviewed on 2026-10-12 using Sol 6.1, at commit
`1596988860edf2de9f0da02d8e0472d4adfddc89` on `casefold-rw`.

## Decision

Keep public casefold writes blocked. Physical-range admission is now addressed
by the follow-up below; qualify block roles and mapping structure before extending the indexed
insertion primitive to one intermediate level. The current healthy-image Linux
evidence supports the existing root-only emitted bytes; it does not establish
safe admission of arbitrary corrupted or staged block maps.

The original review inspected source and retained CI evidence without changing
production code or executing new malformed-image regressions. The follow-up
below records the subsequent red-to-green boundary fix. Other inferred
counterexamples still need a failing test before their fixes.

## Boundary-fix follow-up — 2026-10-12

Finding 1 is now reproduced and fixed for this private admission path.
Four new negative tests were red with acceptance (`Ok(())`) before the fix:
invalid staged directory addresses, valid directory bytes beyond the filesystem
on a larger device, an out-of-range external map node, and an out-of-range node
reachable only through a changed staged index while both views return identical
data mappings. A fifth positive test keeps the last in-bounds directory block
admissible. All five now pass.

`CasefoldMapDevice` limits external map-node reads to `Filesystem::byte_ceiling`
in both disk and staged views. Admission also checks every mapped directory
block against the effective full-block limit before consulting staged data.
The existing checked map, checksum and disk/staged agreement checks remain, and
the admitted selected-leaf address is still reused. Refusal tests compare the
entire dirty map, pending allocation flags and device bytes.

The unit tier increased from **796 to 801**, with no lost passing test
occurrences (compared as a multiset). Lint and release CLI installation passed.
These runs used Rust/core pins from the handoff through the temporary source
view described in `extprogress.md`; no sibling source or lock was changed.
The new source still requires exact-commit focused Linux and full CI outcomes.
Findings 2 and 3 remain open; this is not allocation/role validation, full map
qualification, deeper index support or permission to enable casefold writes.

## Evidence examined

- `src/fs.rs`: `BlockBuffer`, `StagedTreeNodes`,
  `buffer_add_dir_entry_indexed`, `buffered_block`, `buffered_inode_verified`,
  and commit handling.
- `src/path.rs`: `casefold_insert_leaf` and `casefold_index_leaves`.
- `src/extent.rs`: `lookup_verified` and `map_logical_verified`.
- `src/htree.rs`, `src/casefold/lookup.rs`, `src/dir.rs`, and relevant existing
  planner and Linux integration tests.
- Full CI passed for budget-repair commit `34e6a85` in
  [run 38317933016](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38317933016).
- The reviewed source, tests, and focused workflow are unchanged from mapping
  fix `6402d5d82dcf5768105964b985cb1d914d351d80`. Its retained
  [focused artifact](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38134122031)
  contains the exact insertion test's successful result: **1 passed, 0 failed,
  0 ignored**, with four saved images covering 1 KiB/4 KiB and strict/non-strict
  encoding. The log is 133 lines / 7,953 bytes, with 16 oracle and 8 kernel
  invocation markers. The test requires judged non-repairing e2fsck results,
  cold Linux name/alias lookup, inode and payload agreement, duplicate refusal,
  and preservation of all 1,000 original files.

The integration test writes a prepared hard-link mutation to a scratch image;
it does not qualify public namespace APIs, journal commit, concurrent operations,
or interrupted-write recovery. No host mount, host e2fsprogs invocation, or
kernel/e2fsprogs implementation source was used in this review.

## Findings to resolve before broadening admission

### Staged superblock stability follow-up — 2026-10-14

At `1a6e8d1`, a fresh 820-test baseline passed. Four new negative planner tests
reproduced `Ok(())` despite staged geometry, feature/policy, other non-counter
or checksum changes. All five focused tests now pass, covering the relevant
layout/Unicode/hash/checksum fields, reserved bytes and truncation, with a
free-counter control for inline and external maps. Refusals preserve every
dirty block, allocation flag and device byte; the control verifies free counts,
stored name/inode and mutation of only the selected leaf.

After read-only directory/mapping admission and before the descriptor check or
insertion edits, the helper verifies a full primary superblock region and its
checksum, then compares all pre-checksum bytes with the mounted raw superblock.
Only the existing free-counter ranges are exempt; their checksum must verify.
Other edits, including informational/orphan fields, are unsupported until
qualified. Unknown/reserved bytes are not silently admitted. This stabilizes
cached layout, identity and comparison/checksum policy without parsing a new
unqualified geometry, and preserves the existing corruption refusals.

The guard does not audit counter accounting, allocation, journal/other ownership,
unused branches, complete mapping shape, external mutation or serialization.
Findings 2 and 3 remain open. Next review current allocation/ownership requirements
and reproduce one bounded missing-allocation counterexample; existing synthetic
planner fixtures do not establish correct allocation. Preserve all baseline
occurrences and update fixtures explicitly for any new allocation precondition.
Previous focused Linux and full CI passed, and the insertion log was inspected;
new-source independent CI remains required. Final local validation passed 825
units with all 820 baseline occurrences retained, lint and the release CLI build.
Keep public writes, deeper insertion, collisions and growth blocked.

### Staged descriptor relocation follow-up — 2026-10-14

Follow-up: exact-commit focused Linux and full CI at `1a6e8d1` now passed; the
staged-superblock checkpoint supersedes its next task and pending status below.

At `986808c`, a fresh 815-test baseline passed. Four new negative planner tests
reproduced `Ok(())` for staged bitmap/inode-table relocations, high pointer changes
and a broken descriptor checksum. Cases also cover metadata moved into visited
map-node storage and a truncated descriptor block. All five focused tests now
pass, including the counter-only control for inline and external maps. Refusals
preserve the complete dirty map, allocation flags and device bytes; the control
verifies stored name/inode and changes only the selected leaf.

After read-only directory/mapping admission and before editing/staging insertion,
the private helper validates each staged primary descriptor's block/slice size
and checksum, parses the existing format and compares complete bitmap/inode-table
addresses against the mounted descriptors. Metadata relocation is unsupported;
checksum/corruption errors refuse malformed descriptors. Checksummed counter-only
updates remain admissible. Existing fixed-role and mapping corruption refusals
keep their earlier read-only checks; no ordinary-directory path is changed.

This does not build a general transaction-current ownership map. Non-address
descriptor fields, staged superblock layout/policy, allocation, journal/other
ownership, unused branches, complete mapping shape and serialization remain
unqualified. Findings 2 and 3 stay open. Next reproduce staged superblock changes
and define a bounded refusal that preserves valid counter-only updates. Previous
exact-commit focused Linux and full CI passed, and the insertion log was inspected;
new-source independent CI remains required. Final local validation passed 820
units with all 815 baseline occurrences retained, lint and the release CLI build.
Keep public writes, deeper insertion, collisions and growth blocked.

### Fixed-metadata mapping-node refusal follow-up — 2026-10-14

Follow-up: exact-commit focused Linux and full CI at `986808c` now passed; the
staged-descriptor follow-up supersedes its next task and pending status below.

Another fixed-location subset of finding 2 is reproduced and corrected. At
`8f94cf8`, a fresh 810-test baseline passed. Three new negative tests returned
`Ok(())` for checksummed extent nodes at superblock block 0, bitmap block 2 and
inode-table block 5. Cases also cover the descriptor, inode bitmap and last
inode-table block, with disk-only and identical-staged nodes. All five focused
tests pass, including a positive node exactly after the inode table and a node
reachable only in the staged view while both views return identical directory
data mappings. The full tier passes 815 tests with all 810 baseline occurrences
retained; lint and the release CLI build also pass.
Refusals preserve the complete dirty map, allocation flags and device bytes;
the positive cases verify stored name/inode and change only the selected leaf.
These are synthetic planner fixtures; clean cache invalidation makes direct
backing-byte changes visible without bypassing extent checksum verification.

The private mapping wrapper now refuses any read that touches fixed mounted
metadata before reading disk or staged bytes. It reuses the existing role
classifier; bounds, extent checksums, disk/staged mapping agreement, visited
mapping-node/data disjointness and selected-leaf address reuse remain intact.
Both previous full CI runs and the latest focused Linux insertion run passed;
the downloaded insertion log confirms the exact test executed. The new source
requires its own independent CI. Final local results are in `extprogress.md`.

Finding 2 remains open for transaction-current descriptors/allocation,
journal/other ownership and unused mapping branches; complete mapping shape
in finding 3 and operation-wide serialization remain unqualified. Next
reproduce staged descriptor relocation and define a scoped refusal that retains
valid counter-only updates. Keep public writes, deeper insertion and growth blocked.

### Visited mapping-node/data overlap follow-up — 2026-10-14

Follow-up: its exact-commit focused and full CI at `8f94cf8` now passed. The
fixed-metadata mapping-node checkpoint above supersedes its next task; older
pending statuses below describe that checkpoint's original pause point.

Another bounded subset of finding 2 now has a red-to-green correction. At
`3be872e`, a fresh 807-test baseline passed. Two synthetic thirteen-block
legacy-map fixtures reproduced `Ok(())` when leaf 1's physical block 517 also
held the single-indirect pointers read for logical block 12. The selected-leaf
and unselected-leaf cases both failed their required refusal; a separate-node
positive control passed. All three focused tests now pass in both disk-only
and identical-staged-node cases. Refusals preserve every dirty block,
allocation flag and device byte. The positive control changes only the selected
leaf and verifies the new name/inode; it does not qualify legacy write profiles.

The private mapping wrapper records successfully read physical block locations
from both views in one per-call set. After full directory admission and before
editing/staging the copied leaf, its visited mapping-block set must be disjoint
from all admitted directory-data addresses. This catches a mapping node first
discovered after its data role was admitted, including an unselected leaf.
Repeated node reads remain valid. Bounds, checksum checks, disk/staged mapping
agreement and selected-leaf address reuse remain unchanged.

Unused/shadowed mapping branches, fixed-metadata roles of mapping nodes,
transaction-current descriptor/allocation state, journal/other ownership,
complete mapping shape and operation-wide serialization remain unqualified.
Findings 2 and 3 remain open. The preceding fixed-metadata source's exact-commit
focused Linux run passed and its insertion artifact was inspected; its full CI
still has two VM suites pending. New-source independent CI is also required.
See `extprogress.md` for final local validation and exact checkpoint results.
Public casefold writes, deeper insertion, collisions and growth remain blocked.

### Scoped fixed-metadata follow-up — 2026-10-14

Part of finding 2 now has a red-to-green correction. A fresh 802-test baseline
passed, and the saved metadata-leaf regressions again returned `Ok(())` before
the fix. Expanded tests also reproduced metadata-root acceptance with an
ordinary leaf. Four negative tests and the positive inode-table boundary test
now pass; refusals preserve the complete dirty map, allocation flags and device
bytes. These are synthetic planner regressions, not Linux-clean malformed images.

Every mapped directory root/leaf is refused before staged-byte admission if its
address belongs to fixed mounted metadata: group-head superblock/descriptor
copies or reserved GDT, any descriptor's bitmaps, or any complete inode table.
The existing group layout helper supplies head geometry, and every descriptor
is checked to account for FLEX_BG placement. The preceding bounds, checksums,
disk/staged map agreement and admitted-leaf reuse remain in place.

This checks mounted fixed locations only. It does not validate transaction-current
descriptor/allocation state, journal/other ownership, external map-node/data
aliases or complete mapping shape. Findings 2 and 3 remain open. Full baseline,
lint and CLI results are recorded in `extprogress.md`; exact-commit independent
Linux and full CI for this new code remain pending. The prerequisite full CI
at `643fd2a`, `1ee1fed` and `d6ae8d8` passed; the downloaded last run's suite log
confirms healthy-image insertion executed successfully on the preceding source.
Keep public writes, deeper insertion and growth blocked.

### 1. Physical filesystem bounds are not part of admission

In the reviewed `src/fs.rs` at lines 3947–3992, the mapper returns a physical
address; admission checks disk/staged agreement, presence, and uniqueness, then
reads its bytes. There is no check against `sb.blocks_count` in this path.
`buffered_block` at lines 7514–7517 clones a staged block without an I/O read.
An inline extent can be mapped without reading any external node, and
`extent::map_logical_verified` has no filesystem-boundary parameter.

Therefore device I/O cannot be relied on as the admission boundary: staged
bytes bypass it, and a device may contain more bytes than the declared
filesystem. Checksums validate a byte representation, not its physical location.
The final `buf.put` uses the admitted address; it does not add a bounds check.

**First regression to reproduce:** adapt the existing synthetic staged
root/leaf fixture so its current inline map points outside the filesystem,
while valid checksummed root/leaf bytes are staged at those addresses. Test
that insertion refuses without changing `dirty`, `uninit_cleared`, or device
bytes. Cover `blocks_count` itself, beyond it, and a backing device larger than
the filesystem. Also cover out-of-range external map-node reads in both views.
This counterexample was initially inferred from the source; the follow-up
records its reproduction and fix.

**Correction to qualify:** constrain mapping reads and admitted data addresses
to the filesystem's declared geometry before using staged bytes. Keep the
checked mapping and selected-leaf address stable through final staging. Do not
alter the shared public mapper merely to solve this private admission problem.

### 2. Unique directory blocks do not establish safe block roles

The `physical_seen` set contains mapped directory root/leaf addresses. It does
not contain the external extent/indirect map-node addresses or establish that
directory data avoids inode tables, bitmaps, superblock/descriptor regions,
journal storage, or other allocated ownership. The admission closure consults
neither allocation state nor a role-aware map. Agreement between two views and
inode-salted checksums do not by themselves establish these ownership rules.

This is a qualification gap, not a demonstrated metadata overwrite on the four
healthy Linux profiles. Add deliberate alias fixtures and an explicit role
validation/refusal contract before treating this primitive as a general write
admission boundary. Include transaction-current allocation/metadata state and
shared-node/data aliases. Existing successful tests using synthetic staged
blocks are planner tests, not evidence that those blocks are correctly allocated.

#### Metadata-alias reproduction — 2026-10-13 (historical red)

At source `1ee1fedf90f1056cc9f14d5550ecea4cbf460030`, three synthetic planner
refusal tests reproduced acceptance of a staged, checksummed directory leaf at
primary superblock block 0, block bitmap block 2 and inode-table block 5.
The index root stays in its ordinary data block, and a valid two-extent map
selects the metadata leaf. Each test fails because insertion returns `Ok(())`.
This is now a demonstrated private-planner metadata-role gap, not an exposed
public volume write or a Linux-clean malformed-image fixture.

The reproducible test patch is
[CASEFOLD-METADATA-ROLE-RED.patch](CASEFOLD-METADATA-ROLE-RED.patch); it was saved
outside the active test source while the existing full-CI gate was pending.
`extprogress.md` records commands and the fresh 802-test baseline. At that historical checkpoint, refusal
atomicity assertions still needed a green result: the red run failed before
those assertions. Production admission, write gates, dependencies and siblings
were unchanged during preparation. Finding 2 remains open,
including staged role/allocation changes, journal ownership and map-node aliases.
The 2026-10-14 follow-up incorporates and extends these tests and fixes their
mounted fixed-location counterexamples; do not reapply the historical patch.

### 3. Checksummed lookup is not complete map-shape validation

`extent::lookup_verified` stops on the selected covering extent (reviewed lines
254–270). Its successful answer does not prove ordering/non-overlap of all
extents, exact parent/child depth transitions, or uniqueness of all map-node
roles. The directory pass checks every referenced logical data block, but a
lookup can still leave an unused or shadowed mapping record unexamined.

The legacy pointer path is also admitted even though the independent insertion
profiles use extent maps. Either qualify these shapes with malformed-map
refusals and independent healthy-image evidence, or narrow admission explicitly.
Retain all existing checksum/staged-remap refusals while doing so.

## Checks that currently support the restricted primitive

- The current buffered inode supplies encoding, generation and mapping; external
  extent tails are checked in both disk and staged views. Different selected
  mappings are refused, and the admitted selected-leaf address is reused.
- Root-only logical references are bounded and unique, with all declared
  directory blocks accounted for. Root layout, checksum/capacity, all leaf
  checksums/record spans, equivalent-name uniqueness and every live entry's
  frozen-key route are checked before insertion.
- Unsupported depth, repeated/odd collision boundaries, full-leaf growth,
  malformed names and unqualified policy shapes are refused. Full leaves return
  `Unsupported`, preventing the ordinary growth/index-drop fallback.
- Admission reads take immutable buffer references. Insertion edits an owned
  leaf copy and performs its only `buf.put` after all fallible checks. Existing
  refusal tests compare buffer/device snapshots. This supports refusal atomicity
  for the reviewed helper; it does not prove whole-operation serialization.

## Later write-activation requirements

The journal mutex protects commit handling; it does not serialize the entire
absence check, allocation, planning and commit sequence. Target inode validity,
link accounting, timestamps and stale-entry replacement remain the caller's
responsibility. Qualify complete namespace operations, operation-wide
serialization and crash recovery before relaxing any volume write gate.

Finding 1 is addressed by the boundary-fix follow-up above. Fixed metadata roles,
visited mapping-node/data overlap and staged descriptor/superblock changes have
scoped refusals recorded above; inspect the latest exact-commit CI before
broadening support. Next review current allocation/ownership and reproduce one
bounded missing-allocation case, respecting planner-fixture limits. Keep finding 3
open until its mapping-shape tests and contract are satisfied.
Deeper index insertion, splitting/growth and write activation remain later work.
