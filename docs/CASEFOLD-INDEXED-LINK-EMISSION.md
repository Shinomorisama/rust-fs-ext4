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
- match the Linux split's complete inventory, old logical/physical mappings,
  map length, route count, free counts, protected bytes, and parent/source
  structural fields; the independently valid new physical block may differ;
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

Local compilation with the pinned core succeeds. First focused run 41911766228
reached a clean, routed Rust 1 KiB direct split, then exposed an overstrict test
comparison: Linux chose new block 8712 while the driver independently chose
16903 from the same input map `[8709, 8710, 8711]`. Both exact transitions debit
one block and preserve all old mappings. The failed artifact's advertised
SHA-256, ZIP CRCs, 124 tracked-source hashes, exact input/refusal identity,
inventories and clean e2fsck output pass. The comparison now records both new
blocks while requiring the invariant old-map prefix and independent exact
allocator transitions. Corrected focused/full CI and the complete eight-profile
saved-byte audit remained required before this matrix could qualify driver
emission.

Corrected source `843a72c` passes focused run 41920573632, all eighteen jobs,
and full run 41920573687, all eleven jobs including `ci-ok`. Focused artifact
13366128685 matches advertised SHA-256
`427fbd4db07a90175394c69563f222884c9d81f5b99bee8f9688e87af9946062`;
all 723 ZIP members, CRCs and 124 tracked-source hashes pass.

The audit covers eight Rust/C driver links, 24 zero-effect refusals, eight cold
Linux observers, 64 retained images, forty clean e2fsck/index/inventory phases
and 120 raw inode records. Every profile grows from two to three routes, adds
one inventory entry and one independently accounted block, retains INDEX and
all old mappings, and preserves the tracked tables, EAs, payloads and root.
Rust and C outputs agree per profile. The 1 KiB direct driver/Linux allocators
choose different valid blocks (8711/16904); the other three profile pairs choose
the same new block (16907, 527 and 1551). All cold inventories equal their
closed driver inventories.

All eight full-run archives match their advertised digests and CRCs. Compared
with successful baseline 41907682056, both whole-suite multisets retain every
2,504 preceding success and add only this host test, for 2,505 successes; the
1,651-unit multiset is exact and the one existing ignored fuzz case is unchanged.
Actual focused/native/guest logs fit unchanged limits at 365 lines/42,289 bytes,
3,938/240,978 and 4,106/250,498. The bounded ordinary root-only retained-index
driver emission matrix is closed. Full-root fallback, deeper indexes,
uninitialized allocation, recovery and casefold writes remain unqualified.

## Scoped source and evidence review — 2026-11-05

Reviewed at `066377f`, whose changes since tested source `843a72c` are only
documentation and agent policy. This review finds no additional blocker within
the healthy ordinary root-only profile above. It does not qualify arbitrary
input admission or the split-decline/fallback cases below.

### Complete link staging and publication

`src/fs/guarded_chmod.rs::link_from_in` enters planning under the existing mount
operation context. `src/fs.rs::plan_link_at` checks source/parent identity,
directory-target refusal, link-count room and duplicate lookup before staging.
Only `NotFound` establishes absence. It stages the source count in a local
`LinkPlan` buffer, then attempts indexed insertion; only `OutOfBounds` requests
growth. Errors discard the local plan before the dirty marker or commit.

`buffer_split_htree_leaf_and_add_entry` obtains the buffered inode and index/leaf
bytes. It prepares owned split leaves and a parent route, inserts the new name,
restamps checksums, then appends the new leaf and stages the old leaf and route.
`buffer_append_dir_block` stages allocation, mapping and inode size/block counts.
After growth, `plan_link_at` rereads both source and parent through the buffer
before writing times/version/checksums. `buffer_write_inode` updates individual
slices in the current table block, preserving the actual shared-table case.

`commit_link_in` prepares a staged superblock when present, then places dirty
marking and the complete buffer commit under the operation's effect boundary.
Uncertain effect errors/unwinds retire the mount through the existing failure
latch. The counted C call obtains its optional owned inode snapshot under the
same context; an attribute-read error after commit does not undo the alias.
Other unguarded writers and separate mounts/devices still require caller
exclusion; this is not an ownership or device-exclusivity proof.

The same buffer reaching commit is not a power-loss atomicity guarantee on a
direct filesystem. `commit_block_buffer` uses the journal when available and
ordered writes/flushes otherwise. Broad atomicity wording on shared staging
helpers must be read with that distinction; the explicit direct-write limit in
`plan_link_at` governs this review. No interrupted indexed-split recovery test
was added or credited.

### Evidence boundaries and next controls

| Boundary | Review result |
|---|---|
| Root-only healthy split | Linux created each exact pre-split input. All eight driver runs retain INDEX, preserve prior mappings and names, and add one route/block with full accounting and cold readback. |
| API dispatch | This specific matrix uses Rust `apply_link_at` and C `fs_ext4_link_at` with owned attribute readback. Other path/byte entry forms have preceding family tests; they were not independently repeated on indexed inputs here. |
| In-place step | The allocation-free indexed step in this matrix is performed by Linux during preparation. It is not an additional driver in-place emission profile. |
| Split vs decline | The planner keeps equal major hashes together. Full parent, duplicate bound, unsplittable hash or non-fitting half may decline into index dropping. No focused profile accepts that outcome. |
| Malformed input | The split helper turns every leaf-planner error into `Ok(false)`; index insertion also uses `CorruptExtentTree` for expected declines. Public absence/checksum checks reject some damage first. A reachable malformed-image success has not been reproduced by this review; the healthy matrix cannot establish all corruption refusals. |
| Mapping and ownership | The ordinary split uses the shared mapper, unlike private casefold admission's checked overlay. External maps, role aliases and transaction-current map changes have not been admitted by this matrix. All tested maps remain at inline extent depth zero. |
| Cold observation | The guest checks every name/inode, payload and EA. After observation the Rust reader compares all tracked table, directory, allocation and protected bytes with the closed image. The observer works on a copy; successful cold mounting is not crash recovery. |

**Next bounded task:** prepare and execute native controls distinguishing an
expected split decline from corruption/error propagation. Cover a full parent,
duplicate routing bound, unsplittable hash, non-fitting half, bad checksum and
malformed selected-leaf records. Require exact outcomes plus unchanged
device/marker/clock/health for genuine refusals; inspect buffer changes at the
private helper boundary separately from a public plan discarded on error.
Keep controls that intentionally reach supported ordinary fallback distinct
from refusals, and assert the resulting INDEX state explicitly. First reproduce
any suspected defect, then make only the demonstrated repair. Preserve existing
passing tests and caps. Deeper-index/fallback Linux matrices and recovery remain
subsequent qualification tasks.

Fresh local checks used the pinned cloud runner and repository tier scripts:
`review-indexed-link` passes its exact one-test control (18 lines/769 bytes,
retained INDEX at both block sizes); `review-htree-plans` passes all nine
planner tests (29 lines/1,478 bytes). Logs are under `tmp/logs/`. These are native
controls, not fresh Linux evidence or a rerun of the full unit suite. Full lint,
the shared-agent-block integrity check and `git diff --check` also pass.

One Terra helper with medium reasoning and no conversation-history fork checked
CI identities, archive digest/CRCs and the 124 manifest entries against historical
Git object bytes. The primary agent independently repeated the artifact/hash
checks and retained all correctness conclusions. Helper usage/cost was not
exposed, so no cost saving is asserted. At the helper's observation, closure
commit `1ca3079` full run 41969074813 passed all eleven jobs; policy runs
41988244944 (`1509d13`) and 41988567874 (`066377f`) were still running. Their
eventual results are separate from the exact `843a72c` implementation evidence.
