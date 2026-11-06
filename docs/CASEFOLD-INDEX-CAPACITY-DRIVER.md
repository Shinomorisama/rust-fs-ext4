# Ordinary full-root hard-link qualification

## Reviewed boundary — 2026-11-06

The measured Linux reference promotes a full root to one intermediate level.
The existing ordinary driver follows a different path: `plan_link_at` builds
the owned `LinkPlan`; insertion's full-leaf signal reaches
`extend_dir_and_add_entry`; `buffer_split_htree_leaf_and_add_entry` returns
`Ok(false)` when the routing parent cannot accept another entry; then
`buffer_drop_htree_index` converts index records, clears INDEX and appends one
linear directory block in the same buffer. `commit_link_in` retains the mount
operation through optional-superblock preparation, commit, publication and
readback. Corrupt selected-leaf records/checksums propagate errors before that
fallback. Casefold index dropping and public casefold writes remain refused.

This batch qualifies that existing ordinary fallback on healthy root-only
inputs; it does not implement Linux-style promotion. Earlier full-parent
native controls repeat one leaf reference to isolate a private capacity check.
New controls must use all 123/507 physical route slots with distinct leaves,
strict even boundaries and names routed to their actual leaves. Synthetic
native fixtures are not Linux-clean claims.

## Native contract

- At 1/4 KiB, build a complete full root with distinct mapped/checksummed leaves
  and a genuinely full selected leaf. Bound name generation by the root limit
  and the maximum number of 255-byte names per leaf.
- Require private decline to preserve the entire buffer/publication state,
  device, clock, dirty marker and operation health.
- For Rust and C public links, require success, INDEX cleared, exactly one
  appended directory block, unchanged old mappings and names, one source-link
  increment, no inode allocation and a single clock observation.
- Corrupt selected records/checksums must refuse with no publication, device
  writes, allocation, clock observation or health change. Retain all preceding
  safety controls and write gates.

## Independent emitted-byte contract

Reuse the bounded Linux reference's exact full-root predecessor and triggering
name, not a fabricated capacity limit. Four Linux profiles (1/4 KiB, 256-byte
inodes, checksummed, direct/journaled) feed both public Rust and C calls: eight
driver profiles. Preserve the original Linux predecessor and promoted output.
Retain driver input, refusal, dirty, closed and separate cold-readback images.

Before mutation, require the complete healthy root-only role/map/inventory
contract, genuine capacity and the actual selected full leaf. Exercise duplicate,
overlong-name and directory-source refusals without byte/clock changes. Require
the driver's ordinary fallback to retain every old spelling/source identity,
clear INDEX, append exactly one linear leaf, preserve old physical mappings and
extent-node addresses, account for any new extent-map node, and preserve all
protected inode-table slices, EAs, payloads and root-directory bytes. Source and
parent actually share an inode-table block. Verify exact group/global bitmap
debits and no inode allocation, source/parent metadata and C attribute readback.

Use judged non-repairing e2fsck and cold Linux lookup of every spelling, payload,
EA, flags, exact metadata and free count. Separate cold copies must preserve
the closed driver's protected bytes and mapping/allocator/table state. Linux
and driver namespace inventories must agree; their INDEX state and allocation
representations are deliberately measured separately.

Keep 32 MiB images, at most 16,384 live names, 256-link batches, eight replay
bisections, 1,024 directory blocks and four external extent nodes. The focused
job pins the existing siblings/toolchain, records exact source hashes, enforces
one executed test and always retains evidence. Existing jobs/caps/floors stay
unchanged. Full-suite output changes require actual measured replay before any
ceiling adjustment. No host oracle/mount or local KVM substitution is allowed.

## Status

The batch commits are design **`b47591c`**, native controls **`45aacef`**,
emission implementation **`a8a8ff7`** and focused launch **`ff0b46f`**. Each
was pushed before the next task. Superseded batch CI runs were cancelled to
free runners; final-source focused/full validation now passes. This closure
completes step 5 and the five-task batch.

Steps 1–2 are complete. Six distinct-leaf native controls pass at both block
sizes, including private decline, Rust/C positive fallback and corrupt selected
leaf refusals. Full units retain every 1,669 baseline occurrence and add exactly
six controls: 1,675 successes, 1,923 lines/149,531 bytes within unchanged limits.
Full lint and formatting pass. Step 3 implements the eight-profile driver
test with independent Linux preparation, exact one-block fallback accounting,
protected bytes, retained C attributes and cold metadata/inventory checks.
Its unique scratch prefix permits concurrent complete-suite execution alongside
the original reference. Debug compilation, full 1,675-unit retention, Python
syntax, formatting and lint pass. Steps 4–5 launch the focused job and audit
its actual Linux evidence and complete-suite retention below. Stages 4–5 remain partial and
unquantified; ownership/admission, lifecycle/recovery, ignored-fuzz, casefold-write
and platform gates remain in place.

Step 4 adds `link-index-capacity-emission`, the twentieth focused job. It pins
the existing siblings and toolchain, hashes all tracked source/support/dependency/
workflow inputs, runs the one exact test with floor one, and always retains
evidence. Its initial 4,200-line/340,000-byte ceiling reuses the bounded capacity
tier; every preceding job/body/cap/floor is preserved. Publication launches both
focused and full CI. Step 5's source-bound saved-byte and retention audits pass
as recorded below.

## Independent Linux execution and raw audit — 2026-11-06

Exact launch source **`ff0b46ff9084949af75de4282c865db3978b1a6a`**, focused
run **42202840346**, job **127842112107**, passes the exact test and one-test
floor. Its actual wrapper passes at **418 lines/54,163 bytes**. Artifact
**13495931406**, advertised and verified ZIP SHA-256
**`30ecbe7e25c64b2d15bc5dbf5e58341e658a93f3033fbd18a19b68e9cb3d6042`**,
passes CRC and API source/run binding. All **129** source/support/dependency/
workflow hashes match the exact Git tree with no missing or extra paths.

The archive retains **56 images**: sixteen Linux reference phases and forty
driver input/refusal/dirty/closed/cold phases. Both Rust `apply_link_at` and C
`fs_ext4_link_at` independently operate on copies of the same full-root input.
The primary saved-byte audit verifies all eight profiles:

| Blocks / journal | Aliases before → after | Driver block allocated (both APIs) | New map nodes |
|---|---:|---:|---:|
| 1 KiB / no | 239 → 240 | 16903 | 0 |
| 1 KiB / yes | 246 → 247 | 16967 | 0 |
| 4 KiB / no | 5345 → 5346 | 779 | 0 |
| 4 KiB / yes | 5462 → 5463 | 1803 | 0 |

The observations are seed/layout-specific. Every driver result clears INDEX,
retains all original physical directory mappings, appends exactly one linear
block and keeps existing extent-node addresses. Exact global/group free-block
debits and bitmap flips consume one previously free bit, with no inode or
extent-node allocation. The Linux control instead promotes depth and consumes
two directory blocks. Complete resulting namespace inventories agree.

Independent raw parsing verifies superblock/descriptor/initialized-bitmap,
inode, extent and directory CRC32C; every full-root source role/hash route;
all stored driver names/inodes/types; exact source-link increment; unchanged
parent links; parent/source timestamps and the allowed raw inode fields.
All other inode-table slices, EAs, source/peer payloads and root-directory bytes
remain protected. Original leaf bytes remain unchanged; only the root record
span/tail changes as it becomes linear. Journal heads are clean after close.

All **eight** refusal images equal their inputs, covering **24** tested refusals
with zero clock observations. Each success observes the clock once. All **four C attribute
records** match the raw closed source inode, including generation, flags,
ownership, size, blocks, link count and all timestamp seconds/nanoseconds.
Dirty captures have the expected uncleared filesystem state while their
committed directory/map, allocator and inode-table bytes match closed output.
This is an observed post-call state, not interrupted-write/recovery proof.

Cold Linux reports match every spelling/inode, all tracked metadata, flags,
EAs, payloads and free counts. Closed and cold directory/map nodes, allocation,
complete inode tables and protected data are byte-identical. All retained
non-repairing e2fsck reports contain their five passes; actual judged calls and
the source-bound test establish clean verdicts. Rust and C manifests agree in
every paired profile.

Primary scratch `/tmp/fullroot-batch-audit/audit_driver.py` has SHA-256
`eb5dc563290224834edddc236048be074ce165bf07a85479008a99737bb9356d`;
its passing JSON report has SHA-256
`28306b95c48d3b5dbea194568e7d17115997aba8e510161be487940face48f37`.
No host oracle, mount or prohibited implementation source was used.

## Full retention and scoped closure — 2026-11-06

Focused **42202840346** passes all **twenty** jobs and **35** actual positive
floor verdicts. Full **42202840602** passes all **eleven** jobs including
`ci-ok`. Seven full-log archives pass API source/run binding, advertised
SHA-256, ZIP CRC and extracted-file verification. Native artifact
**13498934438**, SHA-256
`ab688b7adca0aa6e785531b002615c78ca3d20e3fd7478f3c65fac30f7a55a39`,
and guest **13497652593**, SHA-256
`93df6382ba725c3b26a4966ad6794c782f14fbb98671480c1c3224b5140d0efd`,
each retain the exact prior **2,524-success** multiset plus six native controls
and the single new host test: **2,531** successes, none lost. The same single
ignored fuzz test remains. Each of three unit contexts retains all 1,669 old
occurrences plus exactly six controls: **1,675**. All eighteen legacy and six
new indexed-boundary controls occur once in all five whole/unit contexts.

Actual native/guest wrappers pass at **3,967 lines/244,015 bytes** and
**4,137/253,480**, both observing 2,531/floor 1,060 within unchanged ceilings.
All twenty trailing shell guards and 159 installed-CLI checks/floor 150 execute;
the actual final verdict reports six green tiers and **4,635** executions.
Standalone CLI 159, oracle 225/floor 140, kernel 36/floor 33, third implementation
nine/floor seven, arm images 527/floor 450, wasm five/floor four and Unicode
eight/floor eight pass. No test selection, output ceiling or floor is weakened.

A Terra medium helper checks log identity, floor/report completeness and exact
multisets; the primary independently verifies consequential archive/counter/
wrapper/trailing findings and performs the raw 56-image audit. A helper path
lookup was corrected; final checks agree. Usage/cost data is unavailable.
The complete matrix is `/tmp/fullroot-batch-audit/helper/summary.json`;
primary retention script/report SHA-256 are
`09bbac3fba0b602156f5cb81df2cc59dcc94048451eb801a433963384e88236b` /
`f1c2d9fa55a3f31cc5f401e69ee130e64aac2a2e78c941e8754fff5affa3607a`.

This closes the existing ordinary root-only fallback for the measured Rust/C
link-at profiles under caller exclusion, healthy layouts and closed reference
handles. No new extent-node allocation/promotion was observed; that combination
is not newly qualified. Intermediate index writes, general ownership/admission,
lifecycle/recovery, ignored fuzz and Windows/SteamOS integration remain separate.
Production implementation and casefold write gates are unchanged. Stages 4–5
remain partial and not quantified. Next: review the ordinary one-intermediate-
level insertion/split/full-node boundary before another native/emitted-byte matrix.
