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

Finding 1 is addressed by the boundary-fix follow-up above; check its fresh
focused Linux and full CI outcomes. The next implementation checkpoint should
reproduce and close the scoped metadata-role gap in finding 2. Keep finding 3
open until its mapping-shape tests and contract are satisfied.
Deeper index insertion, splitting/growth and write activation remain later work.
