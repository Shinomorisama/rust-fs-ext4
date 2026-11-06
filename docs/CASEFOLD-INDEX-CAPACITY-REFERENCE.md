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

## Exact-source Linux observation and saved-byte audit — 2026-11-05

Source **`3a131c93c7531d02c3ee5177097c5854f24bad15`** passes the exact focused
test in run **42098063990**, job **127493234974**. Its actual wrapper passes
at **297 lines/33,323 bytes**, with one executed test/floor one and zero failures.
Artifact **13449630990** has API-advertised and locally verified ZIP SHA-256
**`9d9dd49ae76053f7aaae63bd96b716c02117033b2d13f19c83fc9883a9610c5e`**.
ZIP CRCs pass. The retained source commit and all **127** unique source hashes
match the exact Git tree, with no missing or extra paths in the selected set.

The independent saved-byte audit verifies all four layouts and sixteen images:

| Blocks / journal | Root entries at limit | Aliases immediately before → after | Directory blocks before → after | New physical blocks | Existing external extent nodes |
|---|---:|---:|---:|---|---|
| 1 KiB / no | 123 | 246 → 247 | 124 → 126 | 8833, 8834 | None |
| 1 KiB / yes | 123 | 245 → 246 | 124 → 126 | 16967, 16968 | 17414 |
| 4 KiB / no | 507 | 5357 → 5358 | 508 → 510 | 779, 780 | 1041 |
| 4 KiB / yes | 507 | 5369 → 5370 | 508 → 510 | 1803, 1804 | 2065 |

Each trigger is exactly the next zero-based name: ASCII `cap-{n:05d}-` followed
by 245 literal `n` bytes, where `n` is the table's before-alias count. The
preceding root is full; that single link retains INDEX and promotes depth zero
to one, leaving one root entry, one interior node and one additional leaf.
These exact predecessor counts depend on the saved filesystem's hash seed and
name distribution; they are not universal directory-entry thresholds. All
four transitions allocate exactly two directory blocks, preserve the previous
logical-to-physical prefix and external extent-node addresses, and debit two
previously free bitmap bits with no inode allocation. No extent-root promotion
or new extent-map allocation was observed in these four transitions.

The primary audit parses raw superblocks, descriptors, initialized bitmaps,
tracked/root inodes, extent trees, dx nodes and leaf records; verifies CRC32C
checksums; and reconstructs the complete mapping/role graph and every stored
name. It matches all **33,699 inventory rows** across sixteen phases to raw
entries and to guest observations. For every name, saved debugfs major hashes
select its actual leaf through the raw root/node boundaries, providing an
oracle comparison independent of the Rust lookup assertions. The sixteen
non-repairing e2fsck reports contain all five passes; sixteen htree dumps,
48 tracked debugfs reports and 48 raw inode records agree with the images.

Exact global/group free counts and bitmap flips, complete inode-table slices,
external EAs, payloads and root-directory bytes satisfy the observation contract.
Unused INODE_UNINIT group flags/bytes remain unchanged; their absent inode-bitmap
checksums are not treated as initialized checksums. Cold observers match all
48 tracked metadata rows, flags, EAs, payloads, spellings and free counts.
Directory/extent nodes, complete inode tables, allocation metadata and protected
data remain byte-identical between grown and cold images. Journals are clean.
The successful alias-call counts including replays are **2,264 / 2,261 / 7,245 /
7,345**, all below the stated per-profile bound.

The read-only audit uses no host oracle or mount. Its scratch script
`/tmp/index-capacity-ci/audit_capacity.py` has SHA-256
`8cdbeceb17d58fd7c0ee8405b1bd231b88f37e3b8aac1c4b29a5cd2724fe72f3`;
its passing JSON report has SHA-256
`84ba90066d0d71f9ee1c8117b3acc8bb1cf1812fa1c5106f33defd39c040733c`.
A Terra medium helper independently checks identity, report completeness and
actual wrapper/floor evidence; the primary repeats consequential identity checks
and performs the filesystem audit. Helper usage/cost data is unavailable.

The exact full run's unit artifact **13449340328**, SHA-256
`eab73c674d6fdd57dfe58348b79b1f78f5f48c7fe122605f94b9ba56dfb53f94`,
passes advertised digest/ZIP CRC checks and preserves the complete baseline
multiset: **1,669** successes, **1,666** distinct names, all eighteen native
`indexed_boundary` controls once. The primary corrects an over-restrictive
helper name filter and confirms these controls by their actual module prefix.

## Full CI and complete-suite retention — 2026-11-05

Exact reference source **`3a131c9`** passes full run **42098064190**, and the
documentation-only audit tip **`a155801`** passes full **42100775042**. Both
runs complete all eleven jobs, including `ci-ok`. Their implementation, tests,
scripts, dependencies, task limits and workflows are byte-identical. Fourteen
current log archives and two archives from closed baseline **`730abdc`**, run
**42073101277**, pass API run/commit binding, advertised SHA-256, ZIP CRC and
extracted-file identity checks.

| Run / whole tier | Artifact | Passed | Actual lines / bytes | Existing limit | Actual floor |
|---|---:|---:|---:|---:|---:|
| 42098064190 / native | 13451214055 | 2524 | 3958 / 242897 | 4000 / 245000 | 1060 |
| 42098064190 / guest | 13453680299 | 2524 | 4127 / 252501 | 4160 / 255000 | 1060 |
| 42100775042 / native | 13454866671 | 2524 | 3958 / 242848 | 4000 / 245000 | 1060 |
| 42100775042 / guest | 13454177578 | 2524 | 4127 / 252501 | 4160 / 255000 | 1060 |

The four whole-suite archives have these API-advertised and verified ZIP digests:

- **13451214055:** `cd5f83f80c1b5dc229975ae611acd019c205b4a30b53099fa3c34516c09a3377`
- **13453680299:** `d3b97df55cd0f36bfbf00ca904aa747d5039f8bfea64c5cc355a9e2529b064e3`
- **13454866671:** `545c4877b5a16b0c4efd35a413cf686159ede1a1e497d835a44d7094015aa63e`
- **13454177578:** `6113dbae033a813894f40f0d496b3e8d25eaa5bf2e20e2fad61a9ef755bee53b`

Every whole suite retains the exact **2,523-success** baseline multiset and
adds exactly one occurrence of
`fs::guarded_chmod::needs_host::link_emission::indexed_reference::root_capacity::linux_records_first_root_capacity_transition`.
No preceding test name or repeated occurrence is lost. The one ignored
`deep_walk_byte_flip_fuzz_never_panics` remains identical; no fuzz qualification
is inferred. Standalone/native/aarch64 units retain every **1,669** successful
occurrence in each run. All eighteen `indexed_boundary` controls occur once in
each of the five whole/unit contexts per run, ten contexts across both runs.

Actual job logs, not just aggregate status, establish every stated wrapper and
floor. Each native job continues past the whole suite to all **twenty** shell
guards and **159** installed-CLI checks/floor 150, then reports six green tiers
and **4,622** test executions. Standalone CLI retains 159; oracle 225/floor 140,
kernel 36/floor 33, third implementation nine/floor seven, aarch64 images
527/floor 450 and wasm five/floor four pass. Every unit floor observes
1,669/floor 440. No output ceiling, selection, test body or floor changes.

A Terra medium helper performs the archive identity and multiset comparison;
the primary independently checks all sixteen ZIPs/files, raw counters, actual
wrappers/floors, trailing logs and unchanged source. Helper usage/cost remains
unavailable. Scratch identities for this verification are:

- `/tmp/index-capacity-retention/verify_retention.py`: SHA-256 `ee2eefbcffe80e9107bab83d21f07464b5b02e930f2b59f313dd771bbfa1a992`
- `/tmp/index-capacity-retention/verified-retention.json`: SHA-256 `f01c2cded5529d76b72a00174ad7dc4e537ae642ddf1ea14a76a233f4e1272eb`
- `/tmp/index-capacity-retention/helper/manifest.json`: the complete artifact/run/source/digest matrix.

Focused run **42098063990** first completes seventeen jobs successfully,
with 32 actual positive floor verdicts in the saved job logs, including the
capacity reference and its audited sixteen images. Two jobs
exhaust their existing total-job timeouts: unlink **127493234783** stops in QEMU
host setup without executing its emission test; rmdir **127493234490** spends
about 23 minutes in host setup, then stops during its test at the 25-minute job
limit. Its partial artifact **13449643164** is not counted as completed evidence.
Only these two jobs are retried at the same source in attempt two. Unlink
**127585062884** succeeds with its actual **285-line/28,951-byte** wrapper and
one-test floor. Artifact **13460456170** contains eighty images and passes
API source/run identity, advertised SHA-256
`d0b68c42dee06ace94bed72a33fba91d33d6c27c54f063595d441ac9f8c60770`
and ZIP CRC checks. This verifies complete retention and execution, not a new
raw-byte qualification of those older unlink profiles. Rmdir **127585063579**
subsequently completes its test and artifact upload, but exceeds the total-job
deadline as recorded below. Successful first-attempt evidence is retained.

## Completed rmdir evidence and outer CI allowance repair — 2026-11-05

Attempt two's rmdir job **127585063579** spends **20 minutes 36 seconds** in
host setup and **4 minutes 13 seconds** in its test. Its actual wrapper passes
at **669 lines/80,887 bytes**, one test/floor one, and its upload completes.
Every recorded step succeeds, but the overall job is cancelled. GitHub's
check annotation explicitly states: **"The job has exceeded the maximum
execution time of 25m0s"**. The earlier attempt spent about 23 minutes in
setup and stopped during the test. Neither overall cancellation is credited
as a successful aggregate job.

The completed attempt-two artifact **13462410143**, advertised and verified
SHA-256 **`8a46b4f8cf3fa4c1f4e929ffacaaa4de80620c9a7e31f0f24348e621c36db95f`**,
passes ZIP CRC and API run/source binding to **42098063990 / `3a131c9`**.
Its exact **561-file** inventory has no missing/extra paths: sixteen Rust/C ×
1/4 KiB × 128/256-byte inode × direct/journaled layouts, **112** input/dirty/
closed images, **320** debugfs reports, sixteen input/ledger reports and
48 removal observations. Every phase/step/name set and block size agrees with
the existing test; input JSON plus the appended `LEDGER` lines agree on free
counts. All 48 removal confirmations and the one-pass/zero-failure verdict are
retained. This checks completed execution and evidence retention; it does not
repeat the older rmdir raw-byte qualification or extend its supported scope.

A Terra medium helper checks identity and report completeness; the primary
repeats consequential checks against the archive and actual job log. A helper
parser correction was needed; usage/cost is unavailable. Primary scratch
`/tmp/index-capacity-rmdir-retry/verify_reports.py` has SHA-256
`61babbe7f306b9c7786acff7164469e19eab906143fc41c1b8cbd613dc449457`;
its passing JSON report has SHA-256
`3aa4f81ae359aa119cef07d3e8ad0421fcc0267e5bafcad1f9fc62d9ba938144`.

Only the **outer `rmdir-emission` CI job allowance** changes, **25 → 30 minutes**,
giving room for the measured setup, test and upload durations. Semantic YAML
comparison proves every other setting and all nineteen job bodies, pins,
test timeouts, output caps, floors and selections identical. All three existing
workflow safety guards and whitespace checks pass. Filesystem implementation,
test bodies, dependencies and `chores.yml` remain unchanged; no native or oracle
test is rerun locally for this configuration-only change.

The isolated old-source retry, attempt three/job **127673144277**, subsequently
passes with the old allowance, and the repaired workflow passes as recorded
in the closure below.
Documentation tip **`16d6063`** separately passes full **42125608601**, all
eleven jobs/`ci-ok`; its filesystem/test sources match the previously audited
full runs. No fresh multiset audit is inferred from that additional status.

## Repaired CI and fresh saved-evidence closure — 2026-11-06

Repair source **`7e36864323b0775ee97f74103572d0593e91af81`** passes focused
**42151670764**, all **nineteen** jobs, and full **42151670829**, all **eleven**
jobs including `ci-ok`. Saved focused job logs contain **34** actual positive
floor verdicts. Old-source focused **42098063990**, attempt three, also closes
successfully after isolated rmdir job **127673144277** passes its actual
669-line/80,887-byte wrapper and one-test floor. No timeout cancellation is
counted as a passing aggregate job.

The repaired run's rmdir job **127676123462** passes the same wrapper/floor.
Completed artifact **13472314985** has verified API source/run binding, ZIP CRC
and advertised SHA-256
**`a892a2becae6c03b508b1287148dffcfb275d6a21378742146e92bc09c7da6cb`**.
Its exact 561-file set retains all sixteen layouts, 112 images, 320 debugfs
reports, sixteen input/ledger reports and 48 correctly phased removal reports.
All ledgers, phase/step/name sets and confirmations pass the retained-report
check. This closes execution/retention for the previously qualified rmdir test;
it does not extend that operation's raw-byte or ownership/recovery scope.

Fresh capacity job **127676123640** passes at **297 lines/33,323 bytes**, one
executed test/floor one. Artifact **13472399508**, verified SHA-256
**`30908f9bc71e36a0466a738d419f248fd19863291b5cc9f43fcd6e5cadee99f5`**,
passes ZIP CRC and source identity, including every one of the **127** expected
source hashes with no missing/extra paths. The primary independently repeats
the complete raw-image audit on its sixteen fresh phase images: metadata and
directory/map CRCs, complete roles/maps, oracle hash routing, all **33,564**
inventory rows, exact allocation, protected bytes and cold guest metadata.
All checks pass with this measured second sample:

| Blocks / journal | Root limit | Predecessor → grown aliases | Added physical blocks | Existing external extent nodes |
|---|---:|---:|---|---|
| 1 KiB / no | 123 | 240 → 241 | 17025, 17026 | None |
| 1 KiB / yes | 123 | 240 → 241 | 16967, 16968 | 17414 |
| 4 KiB / no | 507 | 5350 → 5351 | 779, 780 | 1041 |
| 4 KiB / yes | 507 | 5342 → 5343 | 1803, 1804 | 2065 |

Each exact single link retains INDEX, promotes zero to one intermediate level,
adds one leaf and one interior block, debits exactly two previously free bits
and allocates no inode or new extent-map node. Protected inode-table slices,
EAs, payloads and root-directory bytes remain unchanged; cold directory/map,
allocator/table/protected bytes match grown output. These counts and addresses
are observations for the retained seeds/layouts, not portable thresholds or
addresses for a driver implementation.

Seven exact-source full-run log archives pass API run/commit binding,
advertised SHA-256, ZIP CRC and extracted-file checks. Native artifact
**13474863568**, SHA-256
`f961509b51ee79e873e3f42c900d068cf4378c0e1405872639cc53d08d7cf721`,
and guest **13474869814**, SHA-256
`11aa6ed2596d7ca0d181a62d23cbd2a55b23b980a63895e908f89e163fa9af7c`,
retain exactly the prior **2,524-success** multisets and same single ignored
fuzz test. The three unit contexts retain **1,669** successes exactly, and all
eighteen `indexed_boundary` controls occur once in all five whole/unit contexts.
Actual native/guest wrappers pass at **3,956 lines/242,614 bytes** and
**4,127/252,501**, each observing 2,524/floor 1,060 under unchanged output limits.
Twenty trailing shell guards, 159 installed-CLI checks/floor 150 and the actual
six-tier/4,622-execution verdict pass. Standalone CLI 159, oracle 225, kernel
36, third implementation nine, aarch64 images 527, wasm five and Unicode eight
pass their actual floors. No test body, output cap, floor, pin or selection
changes at this closure.

A Terra medium helper audits full-run identity/retention and actual floors;
the primary verifies consequential archive/counter/wrapper/trailing findings
and performs the fresh capacity-image audit and rmdir report checks. No helper
discrepancy is found in this checkpoint; usage/cost is unavailable. Scratch
evidence is under `/tmp/index-capacity-closure/`, including the full digest
matrix in `helper/summary.json`. The fresh capacity audit script/report have
SHA-256 `62d3a92c6496ac635d5087ab197023fc8680e6a6010b7f2ddf842c6a6926ccc4` /
`bf177e900fdc5112b097a017d60ef022ae67ab837407c9d9ec465ffde678d42f`;
primary full-retention script/report have SHA-256
`5115bda1c2a8ad8dcfeeddb3a94128685cde4d211406b1556a518438628ba8b5` /
`b4705175fda48ebd4039f1943410e7c9d16a6c8537e7afed1f2bce1a6330b2a2`.

This closes the bounded Linux capacity reference, saved-image review, measured
outer-job repair and exact-source CI/retention prerequisite. Next, review the
ordinary driver's full-root split/fallback/refusal and owned-plan/effect paths
against these real capacity inputs, then specify its bounded native and Rust/C
emitted-byte matrix before changing production. Linux's observed promotion
does not qualify this driver's index-drop fallback or deeper writes. Stages
4–5 remain partial/not quantified; ownership/admission, lifecycle, recovery,
ignored-fuzz, casefold-write and platform gates remain in place.
