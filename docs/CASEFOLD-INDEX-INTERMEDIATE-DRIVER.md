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

All five checkpoints are complete. Design **`d451146`**, native controls
**`889ffb1`**, emission **`5caeab1`**, focused launch **`bbda780`** and this
evidence closure were each committed/pushed before the next task. Exact launch
source passes all focused/full CI and the saved-byte/retention reviews below.
This closure changes only this document and `docs/extprogress.md`; every one
of the 131 validated source/support/dependency/workflow inputs is byte-identical
to the tested launch commit. A follow-up documentation CI run is separate.

Step 4 added the twenty-first focused job, `link-index-intermediate-emission`,
with the existing sibling/toolchain pins, source hashes, exact test/floor one,
40-minute bound and always-retained evidence. Its initial 4,200-line/340,000-byte
ceiling reuses the bounded capacity tier. Every preceding job, selection, floor
and ceiling remains unchanged. Step 5 completes the source-bound audits below.

Step 1 records the reviewed design (`d451146`). Step 2's six native controls
pass, and the complete native-only tier retains every previous 1,675 occurrence
plus exactly six: 1,681 successes, 1,929 lines/150,107 bytes within unchanged
limits. Full lint and Unicode verification pass. The first broad local command
incorrectly selected guest-only tests (missing VM/fixtures); the supported
native-only tier passes. Step 2 is `889ffb1`. Step 3 implements the bounded
Linux last-slot search and reuses the existing Rust/C emitter's refusal, clock,
attribute, allocation and protected-byte checks. Its new indexed branch keeps
the root unchanged, fills the intermediate node, verifies old routes and requires
exactly the selected full old leaf to change. Input/closed/cold snapshots use
the complete depth-one observer and independent non-repairing reports. Eight
driver profiles and all 64 saved source/driver images now pass independent
Linux and raw-byte review. Debug compilation
of all library tests, formatting, full lint and eight Unicode checks pass;
the 1,681 native-only occurrences were verified in step 2. Previous full-root
source `ff0b46f` passed focused 42202840346 and full 42202840602; its docs closure
`6e2de61` also passed full 42213821292. These are baseline evidence, not results
for the new intermediate boundary.

## Independent Linux execution and raw audit — 2026-11-06

Exact launch source **`bbda78059920be7535978d42d929d41a44f60e47`**, focused
run **42264765485**, job **128053674208**, passes the exact test and one-test
floor at **514 lines/70,863 bytes** within the unchanged focused ceiling.
Artifact **13526737825** passes API run/source binding, ZIP CRC and advertised
SHA-256 **`7b015b4696fdf576a09f85b00f0698e2dca1083f4ab78d3e7796d45202dcab69`**.
All **131** source/support/dependency/workflow hashes match the exact Git tree,
with no missing, duplicate or extra paths.

All **64 images** pass the primary saved-byte audit: twenty-four Linux source
phases and forty Rust/C input/refusal/dirty/closed/cold phases. The search needs
5/3/25/7 additional aliases for the four measured layouts, within the 64-name
bound. Both APIs independently use each exact Linux last-slot predecessor.

| Blocks / journal | Aliases before → after | Intermediate routes | New leaf block (both APIs) |
|---|---:|---:|---:|
| 1 KiB / no | 264 → 265 | 125 → 126 | 8711 |
| 1 KiB / yes | 249 → 250 | 125 → 126 | 16970 |
| 4 KiB / no | 5455 → 5456 | 509 → 510 | 782 |
| 4 KiB / yes | 5304 → 5305 | 509 → 510 | 1806 |

These are seed/layout-specific observations. Every result retains INDEX, the
one-level depth and exact root bytes, preserves all old node routes and physical
maps, changes only the selected full old leaf, and adds exactly one leaf. No
new extent-map node is observed. All stored names, inode identities and actual
hash routes agree with independently retained debugfs output; Linux and driver
namespace inventories agree. Root/node/leaf, inode, extent, superblock,
descriptor and initialized-bitmap CRC32C are checked independently.

Exact global/group free-block debits and bitmap flips consume one previously
free bit, without inode allocation. Parent/source share an inode-table block;
all other table slices, EAs, source/peer payloads and root-directory bytes remain
protected. Source links increment once, parent links stay unchanged, parent
mtime/ctime and source ctime agree with the single clock observation, and parent
version increments once. All four C attribute records agree with raw source
generation, mode, ownership, size, blocks, flags, links and timestamp fields.

Eight refusal images equal their inputs, covering 24 duplicate/overlong/directory-
source refusals without clock observations. Dirty images have the expected
unclean filesystem bit and valid superblock CRC while committed table/directory/
allocator/map/protected bytes agree with closed output. This observes post-call
state; it is not interruption/recovery evidence. Journal heads are clean after
close. All 48 retained non-repairing e2fsck reports contain their five passes;
actual judged invocations and the source-bound test establish clean verdicts.
Cold Linux agrees on every name/inode, metadata field, flag, EA, payload and free
count; closed/cold allocation, complete tables, directory/map and protected bytes
are identical. Rust and C inventories agree in every paired layout.

Primary scratch `/tmp/intermediate-batch/audit_images.py`, SHA-256
`69a20126916281eb5878471aae5df5479caa61c6d9c5dc4e5b473d353ebe7930`, reuses
the independent indexed parser/reference audit, SHA-256
`8cdbeceb17d58fd7c0ee8405b1bd231b88f37e3b8aac1c4b29a5cd2724fe72f3`.
The passing JSON report has SHA-256
`fd568e56b822a9fff0cdb9095d3acbf819c17dffdb62e8345f9dc700afb14d49`.
One scratch namespace setup was corrected before the passing run. No host
oracle/mount or prohibited implementation source was used. Complete-suite
retention and all focused-job verdicts also pass as recorded below.

## Complete retention and scoped closure — 2026-11-06

Focused **42264765485**, final attempt 2, passes all **21 jobs** and **36 actual
positive floor verdicts**. Nineteen actual successful executions are retained
with identical original start/end times; two setup-cancelled jobs were retried
at the same source. The mkdir-policy job reached its 20-minute bound in QEMU
setup before testing. After the other real tests finished, the remaining stalled
link-counter setup was cancelled to permit a targeted retry. Both retries pass.
The setup logs do not identify the underlying download cause. No source, test
selection, timeout, cap or floor was changed for these retries. Superseded batch
runs were also cancelled to free runner capacity.

Full **42264765536** passes all **eleven jobs**, including `ci-ok`. All seven
tier archives pass API source/run binding, advertised SHA-256, ZIP CRC and
extracted-byte verification. Native artifact **13533240018**, SHA-256
`7f7617dcba402bd3ac01b2ac44897b1e3ec0d1ba8d6c409edebe805a533133c1`, and
guest **13532989242**, SHA-256
`cf4560892e3a2f39ae8ffe354cf796beddd80ac3c8247a3caa6f6800741fcd4a`, each
retain the exact preceding **2,531-success multiset** plus six native controls
and one host test: **2,538 successes**, none lost. The same single ignored slow
fuzz test remains. All three unit contexts retain every prior 1,675 occurrence
plus exactly six: **1,681**. All **30** old/new indexed-boundary controls occur
once in all five whole/unit contexts.

Actual native/guest wrappers pass at **3,971 lines/244,345 bytes** and
**4,143/254,399**, both observing **2,538/floor 1,060**, within unchanged
4,000/245,000 and 4,160/255,000 ceilings. All twenty trailing shell guards,
159 installed-CLI checks/floor 150 and the final six-tier/**4,648-execution**
verdict execute and pass. Oracle 225/floor 140, kernel 36/floor 33, third
implementation nine/floor seven, architecture images 527/floor 450, wasm
five/floor four and Unicode eight/floor eight remain green. Unit floor 440 and
all other existing selections, floors, ceilings and the ignored-fuzz gate stay
unchanged. No output-budget repair was required.

One Terra medium evidence helper at a time checked identities, multisets and
actual report/floor completeness. The environment resumed during the wait;
saved audit hashes persisted, and the interrupted helper was replaced. The
primary independently reverified consequential archives, counts, wrappers,
trailing checks and final verdicts, and performed the raw image audit. Usage/
cost data is unavailable. Primary final verifier/report SHA-256 are
`25a7c7c89e7cb279bf7f9e3942a1fc3369ff6de5bac6f6360e0c3205f1dad442` /
`d47fd1838540e7a7b5dd404cd0e569e310551865a1d6ac9ca01a1e7b062e23a9`, at
`/tmp/intermediate-batch/verify_final.py` and `retention-verified.json`.

The existing ordinary retained-index last-slot split is scoped to healthy
one-intermediate-node layouts, caller exclusion, closed reference handles and
the measured 1/4 KiB, checksummed, 256-byte-inode direct/journaled profiles.
No new extent-map node allocation/promotion was observed or newly qualified.
The previous root-only fallback's unchanged branch remains covered by its
retained focused/whole tests. Already-full-node public fallback, deeper writes,
general ownership/admission, lifecycle/recovery, ignored fuzz and platform work
remain separate. Production implementation and casefold write gates are
unchanged. Stages 4–5 remain partial and not quantified; stages 6–7 are not
reached. Next: review and independently measure the already-full intermediate
node's Linux growth before its public Rust/C fallback qualification matrix.
