# Ordinary indexed-parent hard-link reference

This bounded Stage 0 prerequisite supports the partial Stage 4–5 work. It does
not enable casefold writes, qualify driver emission, or change production code.

## Scope and baseline

The existing native control
`fs::guarded_chmod::tests::link_calls::positive_ordinary_index_insertion_and_actual_leaf_growth`
passes for 1 KiB and 4 KiB blocks and prints `index_retained=true` for both at
baseline `603248c`. Its assertion accepts either a split or fallback, however;
that result alone is not independent qualification of a retained-index write.

The new Linux-only reference covers four healthy profiles: 1 KiB/4 KiB blocks,
256-byte inodes with metadata checksums, and direct/journaled layouts. Filesystem
creation, population, links, mounts, and checking tools run in the prescribed
CI harness guest. The Rust driver only reads images. Legacy inodes, inline
parents, intermediate indexes, full-root fallback, external extent nodes,
uninitialized-group growth, exhaustion, interrupted writes, ownership/admission
integration, and Windows/card/SteamOS qualification are outside this checkpoint.

## Evidence contract

1. Create a filesystem with `dir_index`, populate long hard-link names until
   Linux has made a root-only index, and stop within a fixed bound. Allocate
   the source in the parent's group before moving its original name to root;
   check their actual inode-table block locations rather than assuming them.
2. Verify the clean input with judged non-repairing e2fsck, inode reports and
   `htree_dump`. Read and validate the inode, index checksum, every referenced
   leaf/checksum, complete unique logical/physical map, and complete name/inode
   inventory. Each live entry must be in the leaf its hash routes to, resolve
   to the source inode, and contribute to its actual link count.
3. Check existing-name, overlong-name and directory-link refusals. Linux
   metadata snapshots and complete raw tracked inode tables, directory blocks,
   root-directory blocks, allocation descriptors/bitmaps, external EAs and
   payloads must remain unchanged across these refused operations. Mount-time
   superblock bookkeeping is not claimed byte-identical.
4. Perform an allocation-free short-name insertion, then bounded individual
   long-name insertions until the first leaf growth. Check each transition,
   retaining the exact image immediately before the growth operation. Preserve
   original names/inodes, source identity, peer bytes, root bytes, EAs and
   payloads. A growth must add exactly one directory block and one root route,
   retain INDEX and all old mappings, debit exactly one previously free block
   bit/group count/global count, and allocate no inode. These are assertions
   awaiting independent execution, not inferred Linux results.
5. Retain input, allocation-free, pre-growth, grown and cold images. For those
   five phases retain clean e2fsck and debugfs inode/index reports, full inode
   records and explicit routing/inventory manifests. Save every guest step's
   before/after JSON, including the actual name and complete inventory.
6. On a separate cold-mounted copy, Linux must resolve every entry to the
   source inode, retain exact tracked metadata, payloads, EAs and free counts.
   The raw tables, directory blocks, allocator and protected data must also
   agree after that observer mount. Cold readback is not recovery testing.

The focused `link-index-reference` job runs exactly one named test with a
one-test floor, preserves all evidence on failure, and records the exact source
commit and SHA-256 of all tracked source/test-support and dependency/workflow
inputs. Existing focused jobs, pins, floors, and ceilings are unchanged. The
initial focused output ceiling borrows the established larger reference tier;
whole-suite output changes must be measured rather than guessed or silenced.

## Split/fallback boundary review

The ordinary insertion path first attempts the selected leaf in the current
buffer. A full leaf reaches `extend_dir_and_add_entry`, which attempts
`buffer_split_htree_leaf_and_add_entry` before calling `buffer_drop_htree_index`.
The split helper declines non-indexed, casefold/encrypted, too-deep, full-parent,
unsplittable-hash, and non-fitting-half shapes. A successful split stages its
allocation, map/size changes, both leaves and parent route in one buffer. A
fallback stages index conversion and append together. This is a source review
of the existing boundary, not a recovery/concurrency proof.

The selected Linux reference deliberately requires room in a root-only index
and retained INDEX. It cannot establish correct driver behavior on a full root
or qualify the fallback transformation. Casefold insertion independently refuses
leaf growth and casefold index dropping remains refused. No guard is relaxed.

## Validation status

The baseline native control passed before reference changes. Local compilation,
full unit retention, lint, and exact-commit independent CI must be recorded in
`docs/extprogress.md`; pending jobs are not evidence of a passed reference.
After the reference passes and its saved bytes/reports are audited, the next
bounded task is driver Rust/C emission against these same admitted Linux inputs,
with split versus fallback asserted explicitly. Preserve all existing write gates.
