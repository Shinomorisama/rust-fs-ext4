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

### Scoped internal-journal role refusal — 2026-10-14

At `da2ffcd`, a fresh 839-test baseline passed. A self-contained declared-journal
fixture reproduced acceptance of a directory root, leaf and external map node
at nonzero journal data block 522. Journal block zero remains intact and parses
as clean. The fixture first allocates storage with ordinary file APIs, transfers
it to checksummed reserved inode 8, releases the temporary file inode/name,
then declares journal/casefold policy before a read-only mount. This is a planner
fixture, not a new Linux-clean image claim or public-write bypass.

All three negatives failed on `Ok(())`; the disjoint-storage control passed.
The same refusal and complete unchanged-buffer/flag/device assertions now pass.
The expanded focused run passes 68 tests: all 55 preceding fs casefold tests
plus 13 journal tests. The full local tier passes 852 tests with all 839 fresh
baseline occurrences retained. Lint and locked release CLI build/install pass;
new-source exact-commit focused Linux/full CI remain pending. `extprogress.md`
records the measured logs and continuation task.

After existing read-only admission and allocation checks, before insertion edits,
the new guard inspects the filesystem journal declaration rather than the absent
live writer. The initial profile requires zero external-device/UUID fields,
consistent journal presence/inode declarations, and a stable verified regular
inode with only the extent flag and no external attribute storage. It verifies
the current inode bitmap checksum/used bit and initialization state. A journal
inode change anywhere in its buffered slice is unsupported; changes to another
inode sharing its table block remain admissible.

The journal map must be complete, initialized and inline (one to four records).
Every record is inspected: capacity, nonzero lengths, ordered contiguous logical
coverage, exact file-size coverage, physical bounds and disjoint physical ranges.
External/legacy maps, unwritten extents, size/preallocation outside this profile
and external journals remain unsupported. Reserved/fixed filesystem metadata
cannot be journal data. Every owned block must have current allocation evidence.
All admitted directory data and mapping nodes visited in either view must be
disjoint from those journal ranges, including unselected leaves and staged-only
nodes. Alias refusal does not become a growth/index-drop signal.

The shared current-descriptor reader retains the allocation checkpoint's
checksum, pointer-stability, pending-flag and unknown-flag checks. Fixed metadata
classification now handles intervals, with the point check delegating to it;
this avoids testing every journal block against every group descriptor. There
are at most four ranges and 131,072 bitmap-bit probes from initialized inline
extent lengths. The guard does not scan other files or build a volume owner map.

Controls cover disk/current inode checksums and truncated buffered slices,
checksum-valid staged inode edits, malformed/unsupported complete inline maps,
metadata/physical overlaps, current inode bitmap checksum/size/flags/used bit,
an unallocated journal tail, inconsistent/external declarations, two-record
healthy maps, shared inode-table blocks, an unselected journal-backed leaf and
a map node reached only in the staged view while data mappings agree. Refusals
preserve the entire buffer/flags/device; successful insertion changes only its
selected leaf and preserves the name/inode assertions.

The existing four Linux-created profiles remain the independent healthy-image
matrix; new-source exact-commit focused/full CI is required after push. The
synthetic multi-record control does not extend the independent profile matrix.
JBD2 contents/lifecycle, concurrent/external mutation, other inode/attribute
ownership and complete directory map shape remain unqualified. In particular,
the three other-owner counterexamples below are **not fixed** by journal-only
protection. Next design a bounded transaction-current owner-map strategy before
its refusal implementation; preserve those tests and their disjoint control.
Public casefold writes, deeper insertion, splitting/growth and index dropping
remain blocked.

### Current ownership review and reproduced aliases — 2026-10-14

The journal checkpoint above supersedes this section's next task. The three
cross-inode aliases and their saved patch remain unresolved; journal protection
does not establish a general owner proof.

Reviewed at `96b3b72b7a51ba49f5918044038551942893eafa`. Its exact-commit
[focused Linux insertion](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38656514666)
and [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38656514635)
passed. The downloaded insertion log confirms one passed test, zero failed or
ignored, across the four existing profiles (133 lines / 7,953 bytes). Those
results qualify the allocation checkpoint; this review changes no production code.

The current allocation guard accepts three demonstrated cross-inode aliases:

| Planner storage | Other current owner | Observed result |
|---|---|---|
| Directory leaf at block 517 | `lost+found` directory inode 11 maps this data block on disk and in the buffer | `Ok(())` |
| External directory-map node at block 521 | Ordinary file `/owner` maps this data block on disk and in the buffer | `Ok(())` |
| External directory-map node at block 549 | `/owner` maps this block only through its checksummed buffered inode; its disk inode still maps block 521 | `Ok(())` |

These are self-contained synthetic planner fixtures built through this crate's
formatter and ordinary file APIs. The file's payload is valid extent-node bytes;
its allocation and payload write precede the read-only casefold mount. The map
cases move directory root/leaf storage to explicitly allocated blocks 8190/8191
so they do not also alias `lost+found`. Each negative verifies the current
checksummed owner inode and mappings, and the production allocation guard passes
before insertion. They do not bypass the insertion guard or enable public writes.
The pending-owner case demonstrates why mounted ownership alone is insufficient.
These are not Linux-clean staged operations or whole-operation qualifications.

All three refusal tests fail specifically on acceptance. Their post-refusal
dirty-map, pending-flag and device-byte assertions have **not passed**, because
execution stops at `Ok(())`. A fourth, positive control passes: allocated
directory/map storage is disjoint from the named ordinary file and `lost+found`,
the inserted spelling/inode is correct, and only the selected leaf changes.
This control is not a global owner-map audit. All four tests are retained in
[CASEFOLD-OWNERSHIP-RED.patch](CASEFOLD-OWNERSHIP-RED.patch); the active source is
restored unchanged and its baseline is revalidated in `extprogress.md`.

#### Journal findings and initial refusal contract

1. A read-only casefold mount deliberately does not open `JournalWriter`:
   `Filesystem::mount` opens it only after `write_refusal()` succeeds. Therefore
   `fs.journal.is_none()` cannot establish absence of journal ownership. Consult
   the filesystem journal declaration, including inode/device/UUID fields and
   feature policy; an external or inconsistent declaration must not silently
   become the no-journal case.
2. `JournalWriter`'s private physical vector describes the active ring through
   mounted/device mappings. It is not a current-buffer owner proof, does not
   identify the journal's external mapping nodes and cannot be reused blindly.
   `jbd2::read_superblock` reads/parses an unbuffered raw inode; a new owner guard
   needs the applicable inode checksum, current allocation and mapping validation.
3. Protect the declared journal inode's complete allocated storage, including
   mapping nodes and preallocation, rather than only payload selected by a
   directory lookup or only the active ring length. Compare against **every**
   admitted directory block and every visited directory mapping node, including
   unselected leaves and staged-only roles. A used bit or a valid extent checksum
   cannot substitute for this role separation.
4. The existing Linux images offer a bounded initial journal profile. Raw bytes
   from the retained insertion images show inode 8, size 4 MiB, depth zero and
   one initialized inline extent beginning at logical block zero:

   | Block size | Strict/non-strict profiles | Journal physical start | Blocks |
   |---|---|---|---|
   | 1 KiB | Both | 16899 | 4096 |
   | 4 KiB | Both | 1033 | 1024 |

   These are observations from the already validated images, not a new journal
   corruption test or permission to assume all journals have that shape. A first
   guard can qualify complete inline extent records and explicitly refuse other
   shapes. Validate the entire declared inline record set, logical coverage,
   initialized lengths, physical bounds/overlap, and fixed-metadata separation.
   Do not copy the selected-lookup limitation identified in finding 3 into the
   owner proof. Legacy/external journal maps need separate qualification.
5. Use verified current inode bytes and mapping policy. Initially refuse staged
   journal inode/map relocation rather than trusting a mounted address vector.
   Updating another inode in the same inode-table block is not itself a journal
   change: inspect the journal's complete inode slice. Keep checksum and extent
   errors, buffer/device immutability, and no-growth refusal behavior intact.
6. Before implementing that scoped guard, reproduce a journal-storage alias
   using a Linux-created fixture or a self-contained declared-journal planner
   fixture. Include a nonzero journal data block so the demonstration does not
   depend on corrupting the journal superblock parser. Require negative root/
   leaf/map-role cases, staged-policy refusals and healthy/no-journal controls;
   preserve the existing four-profile Linux cold lookup/e2fsck test.

#### Other-owner proof remains a separate design requirement

The three reproductions concern other inodes, not journal data. A journal-only
guard will leave them unresolved. Before general ownership admission, define a
bounded, transaction-current owner map or a deliberately narrower profile.
Current inode bitmap/checksum and initialization state must determine which
inodes exist; account for reserved inode roles without interpreting every reserved
slot as a normal file. Include newly allocated/freed inodes and buffered inode
updates, all external map nodes, unwritten/preallocated extents (also beyond
file size), and external attribute storage. Fast symlinks/inline data are not
block-address arrays, and legal attribute sharing is not directory storage.

A scan of cached mounted inodes would miss the pending-owner counterexample.
An unbounded full-volume scan on every insertion is not a practical proof for
the intended 1.5 TB card. Specify memory/work bounds, invalidation and incremental
updates under operation-wide serialization, from duplicate checking through
allocation/planning/commit. Concurrent or external changes remain unqualified.
Do not claim finding 2 closed from the journal subset or allocation bits alone;
finding 3's unused/shadowed mappings and complete map shape remain open.

Public format references used:
[superblock](https://docs.kernel.org/filesystems/ext4/super.html),
[journal](https://docs.kernel.org/filesystems/ext4/journal.html),
[inodes](https://docs.kernel.org/filesystems/ext4/inodes.html), and
[block maps](https://docs.kernel.org/filesystems/ext4/blockmap.html).
Own MIT source reviewed: `Filesystem::mount`, `BlockBuffer`,
`buffered_inode_verified`, indexed admission/allocation, `JournalWriter::open`,
`jbd2::read_superblock`, journal mapping, extent parsing/collection and the
formatter. No kernel/e2fsprogs implementation source or host oracle/mount was used.
The next bounded checkpoint is the journal alias reproduction and initial guard
qualification above. Public writes, deeper insertion, splitting/growth and index
dropping remain blocked.

### Current-allocation refusal follow-up — 2026-10-14

Follow-up: exact-commit focused Linux and full CI at `96b3b72` now passed; the
insertion log was downloaded and inspected. The ownership review above supersedes
this section's next task. Its other-owner counterexamples are not fixed yet.

The three saved allocation regressions were applied to `065f9a8` and rerun:
all three failed specifically on `Ok(())` after a fresh 825-test baseline.
Their allocation-bit/checksum and refusal assertions are now active in
`src/fs.rs`; do not reapply the historical patch. Both preceding full CI runs
(`b7b0325` and `065f9a8`) passed. New-source independent results remain separate.

After existing directory/map admission and staged superblock/descriptor checks,
before editing or staging insertion bytes, `casefold_check_allocation` checks
every admitted directory block and mapping block visited in either view. For
each containing group it reads the current buffered/device descriptor, verifies
its checksum and mounted pointer stability, reconciles pending initialization
flags, refuses unknown flags or `BLOCK_UNINIT`, then verifies a full current
bitmap and its checksum. Ordinary-block coverage and descriptor checksum width
follow the contract below. Group bitmaps are retained only for this call to
avoid repeated reads; mounted allocation snapshots do not replace current data.
A free bit is a corruption refusal. `BIGALLOC` remains explicitly unsupported.
No admission read dirties the buffer, and no ordinary-directory path changes.

Positive planner fixtures now explicitly stage bitmap/checksum/counter updates
for their external/legacy nodes, boundary directory blocks and unselected leaves.
Their name/inode, checksum and unchanged-state assertions remain. The prior
superblock counter test measures its one-count delta from current staged counters
so it still checks the same behavior after fixture allocation. No test-only
production bypass or ignored test is introduced. A separate control permits
newly staged allocation while the disk bit is still free; requiring both bits
would reject a valid transaction. Consistent buffered initialization clearing
also has a positive control. These are planner controls, not qualified owner maps.

Additional refusal controls cover current disk/staged bitmap checksums and
descriptor checksums, bitmap size, current uninitialized/unknown flags,
contradictory pending flags, bigalloc, an unselected free leaf, and a free map
node reached only in the staged view while both views agree on directory data.
Refusals compare the entire dirty map, initialization flags and device bytes;
success controls verify the stored name/inode and mutation of only one leaf.
The full local tier passes 839 tests with all 825 baseline occurrences retained
and exactly 14 new allocation tests, including a 32-byte descriptor checksum
control. All 55 fs casefold tests pass in that tier. Lint and the locked release
CLI build/install pass. Exact-commit focused Linux/full CI remain pending;
`extprogress.md` records measured logs and continuation instructions.

This closes the reproduced missing-allocation cases within the private guard's
scope. It does **not** prove exclusive inode/journal ownership, audit allocation
counter totals or prevent external changes/concurrent operations. Finding 2
remains open for owner roles; finding 3 remains open for complete map shape,
unused/shadowed branches and depth transitions. Inspect this source's focused
Linux and full CI before broadening support, then review/reproduce a bounded
allocated-storage ownership alias, beginning with journal roles. Public writes,
deeper insertion, collisions and splitting/growth remain blocked.

### Allocation/ownership review and reproduced gap — 2026-10-14

Historical preparation: the current-allocation follow-up above incorporates
the saved tests and supersedes this section's next task. Its patch targets the
reviewed source only. Subsequent full CI passed at `b7b0325` and `065f9a8`.

Reviewed the private planner at `b7b0325eeaa6d8e2f46f1e346c3ea680e875b3c3`.
Its exact-commit focused Linux insertion test passed across the four existing
profiles; the downloaded log contains one passed test, zero failed or ignored
(133 lines / 7,954 bytes). At final review, full CI has nine successful jobs;
the x86_64 oracle suite is still running, with no failures observed.
This checkpoint changes neither production admission nor the write gates.

The existing checks establish address bounds, fixed roles, visited map/data
disjointness and staged layout stability. None reads allocation bitmaps.
Three new refusal tests all reproduce `Ok(())`:

- A valid external extent node at block 549 is free in the on-disk bitmap.
- Valid staged directory root/leaf blocks at 8190/8191 are in bounds, outside
  fixed metadata, but free in the bitmap.
- Directory root block 516 is allocated on disk but free in a staged bitmap
  with refreshed bitmap and descriptor checksums.

The tests verify initialized descriptor flags, descriptor/bitmap checksums and
the actual group-relative bits before invoking insertion. The external extent
tail also verifies. They are synthetic planner counterexamples, not Linux-clean
images or public writes. Their required unchanged-buffer/device assertions are
after the refusal assertion and **have not passed**: all three currently stop
on acceptance. The complete failing tests are retained in
[CASEFOLD-ALLOCATION-RED.patch](CASEFOLD-ALLOCATION-RED.patch), outside active
test source until the bounded fix. No test is marked ignored or weakened;
the unchanged active suite is revalidated separately in `extprogress.md`.

#### Contract for the next bounded allocation refusal

1. Preserve the existing read-only admission and error checks. Before editing
   or staging the selected leaf, require allocation of **every** admitted
   directory block and every mapping block visited in either disk/staged view,
   including unselected leaves and staged-only nodes. A chosen leaf alone is
   insufficient. Do not turn a refusal into a growth/index-drop signal.
2. Resolve each containing group from validated mounted geometry, but obtain
   its descriptor and bitmap from the current buffer, falling back to current
   device reads. Require full blocks/slices, descriptor checksum and the existing
   stable-address contract. Mounted `groups` and `allocation_groups()` provide
   cached state; they do not replace current bitmap checksums or descriptor flags.
   Reconcile relevant `buf.uninit_cleared` state with the descriptor rather than
   silently preferring a contradictory flag source. A checksum-valid staged
   counter or bitmap edit does not itself prove allocation.
3. Initially refuse an admitted data/map block in a `BLOCK_UNINIT` group.
   Such a bitmap has no trustworthy ordinary allocation bits. Do not infer
   ownership by synthesizing an all-free bitmap or from the fixed-role helper.
   Initialized groups need current bitmap checksum verification using the
   mounted checksum policy. For the initial ordinary-block profiles, cover
   `ceil(blocks_per_group / 8)` bytes, compare the low 16 bits for a 32-byte
   descriptor and all 32 bits for a 64-byte descriptor. Do not silently truncate
   coverage to fit a malformed bitmap or apply this block rule to bigalloc.
   A missing used bit must be a corruption refusal, with no buffer/device edits.
4. Initially bound this guard to ordinary block allocation. Explicitly refuse
   `BIGALLOC` until cluster indexing, checksum coverage and sharing are qualified;
   one bitmap bit there represents a cluster, not one physical block. Retain
   bounds checks before indexing and reject truncated/out-of-range bitmaps.
5. Admit a correctly staged allocation using its **current** used bit; requiring
   the old disk bit too would reject legitimate allocations. Add positive
   controls for unchanged allocated storage and real buffered allocation with
   current checksums/flags. Update existing planner fixtures explicitly, including
   external/legacy nodes, relocated boundary blocks and unselected leaves. Keep
   every baseline test occurrence and its original behavioral assertion; do not
   add test-only bypasses or claim these fixtures prove ownership.

#### Ownership is a separate, still-open requirement

A used bit records allocation, not which inode owns the block. In particular,
the default planner fixture extends root inode 2 into block 517 without proving
that block belongs to it. Checksum-valid bytes do not establish ownership either.
The directory's complete data/map storage must eventually be disjoint from
the internal journal's data and mapping nodes, other inodes' data/map blocks,
extended attributes and other reserved ownership roles. Unsupported/external
journal profiles need an explicit refusal or separate qualification. No
filesystem-wide current owner index is supplied by the reviewed insertion path.

`JournalWriter` builds a private mount-time journal physical map; the insertion
planner neither consults it nor validates journal ownership through the current
transaction. Reusing that vector alone would omit current journal inode/map
changes and mapping-node storage. Define a verified, transaction-current owner
proof (or a narrower supported profile) before claiming this part of finding 2
closed. It also needs operation-wide serialization so ownership/allocation
cannot change between proof and commit. Finding 3's complete mapping-shape
validation remains separate: visited-node checks omit unused/shadowed branches.

Published format references, not implementation source:
[bitmaps](https://docs.kernel.org/filesystems/ext4/bitmaps.html),
[group descriptors](https://docs.kernel.org/filesystems/ext4/group_descr.html),
[checksums](https://docs.kernel.org/filesystems/ext4/checksums.html), and
[bigalloc](https://docs.kernel.org/filesystems/ext4/bigalloc.html).
Own MIT source reviewed: `buffer_add_dir_entry_indexed`, `allocation_groups`,
`plan_buffered_block_allocation_excluding`, `buffer_mark_block_run_used`,
`buffer_refresh_bitmap_csum`, `BlockBuffer`, `bgd`, `alloc`, `checksum`,
`superblock` and `journal_writer`. No host oracle/mount or forbidden source
was used. Allocation refusal is the next implementation checkpoint; journal/
other ownership, full map shape and serialization remain open afterward.

### Staged superblock stability follow-up — 2026-10-14

Follow-up: the allocation review above supersedes this section's next task.
The superblock checkpoint's focused Linux test now passed; full CI remains
pending at review time. No new production source is introduced by the review.

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
broadening support. The bounded allocation refusal and explicit planner fixture
updates above now address the reproduced missing-allocation cases. The scoped
journal guard addresses its three reproduced aliases. Next design bounded
transaction-current ownership tracking for the three other-inode gaps. Keep finding 3
open until its mapping-shape tests and contract are satisfied.
Deeper index insertion, splitting/growth and write activation remain later work.
