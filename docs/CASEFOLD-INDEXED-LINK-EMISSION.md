# Ordinary indexed-parent hard-link emission

This bounded Stage 4–5 checkpoint prepares independent Rust/C driver evidence
against the admitted Stage 0 Linux input. It does not enable casefold writes or
qualify fallback, recovery, Windows, a physical card, or SteamOS.

## Matrix and input

The focused host test creates four healthy Linux filesystems: 1 KiB/4 KiB
blocks, 256-byte inodes with metadata checksums, and direct/journaled layouts.
For each layout Linux creates a root-only ordinary directory index, performs an
allocation-free insertion, and advances one long link at a time. The exact image
immediately before Linux's first leaf split is retained and copied unchanged to
the Rust and C public-link paths. The exact destination name that split Linux's
leaf is used in both driver copies. This produces eight driver profiles from
four shared inputs rather than independently shaped inputs.

Legacy inodes, inline parents, intermediate indexes, full roots, index dropping,
external extent nodes, uninitialized-group allocation, exhaustion, interrupted
writes, ownership/admission integration, and casefold directories are outside
this matrix.

## Required outcome

Each Rust/C profile first requires existing-name, overlong-name, and directory-
target refusals to leave image bytes and the runtime clock unchanged. The one
successful hard link must then:

- retain `EXT4_INDEX_FL` and add exactly one root route and directory block;
- preserve every old route, name and inode while routing the new spelling to
  the existing source inode;
- increment only the source link count and the qualified parent/source time and
  version fields;
- debit one free block and its exact bitmap/group/global counters, allocate no
  inode, and preserve tracked inode-table, EA, payload, and root-directory data;
- match the Linux split's complete inventory, logical/physical mapping, route
  count, allocator state, protected bytes, and parent/source structural fields;
- close to a clean non-recovery image accepted by non-repairing e2fsck; and
- remain fully resolvable during a separate cold Linux observer mount, with
  driver bytes kept separate from the observer copy.

The existing native indexed-growth control accepts a supported linear fallback.
This host matrix does not: losing INDEX, appending without a new route, or
changing more than one allocation bit fails the test.

## Retained evidence and status

The `link-index-emission` job pins the existing sibling revisions and Rust
toolchain, runs one exact test with a one-test floor, records the source commit
and SHA-256 for every tracked source/test-support/dependency/workflow input, and
uploads all shared Linux inputs plus Rust/C input, refused, dirty, closed and
cold images, routing inventories, inode/index reports, clean e2fsck output,
guest observations, and the complete tier log even on failure.

Local compilation with the pinned core succeeds. The Linux/e2fsprogs/KVM matrix
is intentionally left to the prescribed CI harness and is pending at this
preparation checkpoint. A successful focused run and saved-byte audit are
required before this matrix qualifies driver emission or advances the bounded
indexed-growth evidence.
