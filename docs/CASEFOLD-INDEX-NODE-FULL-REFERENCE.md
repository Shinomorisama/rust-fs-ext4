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

This checkpoint is complete: source **`5cc71e7`** records the reviewed boundary
and Linux-only reference; measured native-byte repair **`a5cbbfc`** passes fresh
full CI. The independent Linux/raw-image and complete-retention reviews below
pass. This closure changes only documentation; the 132 validated focused inputs
and repaired chore configuration are unchanged from their tested bytes.

The Linux-only reference/exact focused job is implemented.
Compilation, formatting, full lint/Unicode and all 1,681 native-unit occurrences
pass with an identical baseline multiset, within unchanged limits. Semantic YAML
comparison preserves all 21 preceding jobs and top-level settings; three workflow
guards pass. Independent Linux execution and raw evidence review pass below.
Baseline `bbda780` passed focused 42264765485/full 42264765536 with
2,538 whole successes and 1,681 units. Latest docs `a0b700a` have byte-identical
validated source inputs; their full follow-up 42277073283 was cancelled as
superseded after the new source's complete validation was queued.

## Exact-source Linux results and raw audit — 2026-11-06

Source **`5cc71e72411ac305ef115ebab7b5372e9db09fac`**, focused run
**42278162173**, job **128099825235**, passes the exact test and floor one.
The actual wrapper passes **487 lines/63,856 bytes** within the unchanged
4,200/340,000 ceiling. Artifact **13534002288** passes API source/run binding,
ZIP CRC and advertised/verified SHA-256
`d77bd7b626b0d69d390993c1d1e4e6bad3a07ab91a4cecb0f74fa47ece954b7f`.
All **132** unique source/support/dependency/workflow paths match the exact Git
tree, without missing or extra paths. All **36** images pass the primary raw
audit, including the six preparation phases and three full-node phases/profile.

| Blocks / journal | Aliases before → after | Full child routes → two child counts | New directory blocks |
|---|---:|---:|---:|
| 1 KiB / no | 252 → 253 | 126 → 63, 64 | 17029, 17030 |
| 1 KiB / yes | 263 → 264 | 126 → 63, 64 | 16971, 16972 |
| 4 KiB / no | 5385 → 5386 | 510 → 255, 256 | 783, 784 |
| 4 KiB / yes | 5450 → 5451 | 510 → 256, 255 | 1807, 1808 |

These are seed/layout-specific observations. The additional full-node search
uses 3/4/4/6 single aliases, within the 128-name bound. INDEX and depth one are
retained; root routes and intermediate-node count grow one→two. Every original
physical directory mapping and existing extent-node address remains. Exactly
one new intermediate node and one new leaf are allocated, with no new extent-
map node in these observations. The flattened global leaf boundaries preserve
every old route, including the boundary made implicit at the new child head;
the one new leaf contributes exactly one new boundary.

Independent CRC32C checks cover superblock, descriptors, initialized bitmaps,
tracked inodes, extent nodes and all directory root/node/leaf blocks. Saved
debugfs hashes independently route every stored spelling to its actual leaf.
The root changes only its count, new route and checksum; among old directory
blocks, only the full intermediate node and selected full leaf change.
Exact bitmap/group/global debit consumes two previously free blocks, without
inode allocation. Parent size/sectors and source links agree; parent links
remain unchanged and observed parent version increments once in every layout.
All protected inode-table slices, EAs, source/peer payloads and root-directory
bytes remain intact, including actual parent/source table-block sharing.

All 36 retained non-repairing e2fsck reports have their five passes; actual
judged calls and the source-bound test establish clean verdicts. Journal heads
are clean after close. Cold Linux agrees on every alias/inode and exact tracked
metadata, flags, EAs, payloads and free counts; its directory/map/allocator/table/
protected bytes equal the grown image. The triggering guest reports match raw
before/after metadata and the single link. No host oracle/mount or prohibited
implementation source was used.

Primary `/tmp/node-full-reference/audit_images.py` has SHA-256
`f187d397c3790bc98e75b6f1adad1d2e8f29d6c0672fe667d14facee48ea71f8`;
its passing JSON report has SHA-256
`0081e130e9559e7ad2f20289255c550b61969b4bb616e5531e6a5759a62ac23c`.
The existing indexed parser/root-reference and last-slot/protected-byte audit
helpers are reused from the preceding source-bound reviews. Focused retention
and the measured native byte-limit repair's fresh full CI also pass below.

## Focused retention and usage-interruption recovery — 2026-11-06

Focused **42278162173**, final attempt 2, passes all **22 jobs** and **37 actual
positive floor verdicts**. Twenty-one successful executions retain their exact
original start/end times. Create-emission job **128099825889** passed its test
(893 lines/113,247 bytes, floor one), uploaded its artifact and completed all
steps successfully, but its 25-minute job bound cancelled the overall result
after 25m11s. QEMU setup alone took 19m49s. Targeted retry **128116842949** passes
at the same source; no tests, floors, caps, timeouts or code were changed.
The initial timing cancellation is independent of the model usage interruption.

The Terra medium helper reached its usage limit during read-only evidence
collection. The primary confirmed that all 132 tested source inputs were
unchanged, including manifest/lock/workflow inputs; only the expected draft
documentation differed. All four then-cached tier ZIPs passed advertised hashes,
source binding, CRC and extracted-byte equality. The primary resumed collection
with atomic writes, refreshed completed job logs and independently validated
downloaded bytes before crediting them. The later architecture archive and both
completed unit contexts retain every one of the 1,681 prior occurrences and all
30 index-boundary controls. The primary subsequently completed the full-suite
evidence review recorded below.
No helper implementation, workflow mutation or tracked-file edit occurred.
Usage/cost data is unavailable.

Primary focused verification report `/tmp/node-full-reference/focused-verified.json`
has SHA-256 `9072be3790feb84d3819451df869692a447725d19f12a1d2db29ccdf7a912ffc`.

## Measured native output-limit repair — 2026-11-06

Full run **42278162281** retains all 2,539 successes in both whole suites.
Guest passes its actual wrapper/floor at **4,144 lines/254,577 bytes** (artifact
**13539000983**). Native artifact **13538423818**, SHA-256
`d5c57491ed518325acf5da8746305fd3cba40bf6739636a5f4f9b04225d51280`, also
retains the exact prior 2,538-success multiset plus the new host test. Its
**3,977-line/245,220-byte** log exceeds the 245,000-byte ceiling by 220 bytes;
the actual native wrapper exits 65, so its following floor/trailing stages and
aggregate `ci-ok` do not pass. There is no native filesystem-test failure.

The primary replays the identical complete saved log: 4,000/245,000 returns 65;
4,000/248,000 returns zero. Both replay logs are byte-identical to the archive
entry, SHA-256 `c921fba787208c24bbfd6654438cf621e2225683703139932a3504ad668c469e`.
The unchanged executed-test floor observes 2,539/floor 1,060. Only the native
byte ceiling is repaired, with a small margin for timing-dependent progress
lines. Line limits, guest/unit/focused caps, test output, selections and floors
remain unchanged. `/tmp/node-full-reference/budget-replay.json` records statuses
and bytes. A fresh full run must execute the repaired wrapper, all trailing
checks and `ci-ok` before closure; that fresh run now passes below. This measured log-size failure is separate
from the usage interruption and the retained create-job timing retry.

## Fresh full-CI retention and scoped closure — 2026-11-06

Repair source **`a5cbbfc7db742410610a7f70ea7e880d61d31939`** passes full
**42290244875**, all **eleven jobs** including `ci-ok`. The sole semantic chore
change from `5cc71e7` is native bytes 245,000→248,000; every one of the **132**
focused source/support/dependency/workflow inputs remains byte-identical.
Focused **42278162173** therefore retains its exact-source 22-job/37-floor
evidence for the unchanged reference implementation. No focused rerun was
substituted for complete validation of the repaired native pipeline.

All seven fresh tier archives pass API source/run binding, advertised SHA-256,
ZIP CRC and extracted-byte verification. Native **13543801615**, SHA-256
`e6db45b7b5a4eade2c7c9e975ff798e341262cc2985067973cbb8fa2ce17f01b`, and
guest **13542946898**, SHA-256
`536b00882843fca81964ae843456183b0069ab7de7babaeb44a6b2827419355d`, each
retain the exact preceding **2,538-success multiset** plus the single new host
test: **2,539 successes**, none lost. The same one ignored slow fuzz case
remains. All three unit contexts retain the identical **1,681** successes; all
**30** indexed-boundary controls occur once in all five whole/unit contexts.

Actual native/guest wrappers pass **3,977 lines/245,220 bytes** and
**4,143/254,458**, with **2,539/floor 1,060**. Native is within repaired
4,000/248,000; guest stays within unchanged 4,160/255,000. All twenty trailing
shell guards and 159 installed-CLI checks/floor 150 execute, followed by the
actual six-tier/**4,649-execution** verdict. Oracle 225/floor 140, kernel
36/floor 33, third implementation nine/floor seven, architecture images
527/floor 450, wasm five/floor four and Unicode eight/floor eight all pass.
Unit floor 440, all selections, line limits, output and every other byte ceiling
remain intact. Only the demonstrated native-byte ceiling changed.

The primary finished evidence collection and independently checked complete
name multisets, actual wrappers/floors/trailing verdicts and source identity
after the helper's usage failure. The raw reference ZIP and every extracted
file were rechecked against its verified archive, as were all 132 source inputs.
Transient GitHub HTTP 502 reads were retried without crediting incomplete data.
No partial source edit, manifest/lock change or damaged evidence from the usage
interruption was found. Primary final verifier/report SHA-256 are
`317c87557c9632457a814d7d66d0f8c05e99d50235a8171220bcd75fb39d344b` /
`5d8d9b38d45ef6fe31ef47e09071e531314585eaf0fddfd29e01d257b1de5b81`, at
`/tmp/node-full-reference/verify_final.py` and `retention-verified.json`.
The identical-log replay report has SHA-256
`8df5f05c28599fa364d52fc3920b1430859dd4d4cdd1d25120ba917da8c18a28`.

This closes the Linux first-full-node-growth reference for the four measured
healthy ordinary layouts. It does not qualify the driver's public full-node
fallback or new extent-map promotion. Production paths and casefold-write
gates remain unchanged; stages 4–5 are partial/not quantified, stages 6–7 not
reached. Next: native positive and selected-corruption refusal controls for the
ordinary full-node public fallback, preserving existing private no-effect
declines; independent Rust/C emitted-byte qualification follows separately.

## Next public-fallback qualification contract

Use each measured `node-full.img` and its exact next alias as a common Linux
predecessor for Rust and C link-at calls. Add native positive/refusal controls
on distinct full-node fixtures before emitted-byte qualification. A full-node
private decline is already covered; public success must additionally prove
INDEX clearing, root/interior record conversion, one appended linear leaf,
unchanged old maps/leaves/names, source links and one clock observation.
Corrupt selected checksums/records must still refuse before fallback effects.

Retain driver input/refusal/dirty/closed/cold images, duplicate/overlong/directory-
source refusals, C attributes and exact allocation/protected-byte accounting.
Compare namespace inventories with this Linux two-block growth while measuring
the driver's representation/debit separately. Check the converted interior
fake record span/tail as well as the root span/tail and INDEX flag. Cold Linux
must find every spelling and agree on exact metadata, EAs, payloads and free
counts. Extend the test emitter's operation mode explicitly: both retained-index
last-slot inputs and full-node fallback inputs have depth one, so depth alone
cannot select the appropriate transition assertions. Preserve both preceding
root-only fallback and last-slot branches and their full test-name multisets.

This contract does not implement or qualify public fallback. Multi-node input
parents, deeper trees, new extent-map promotion, collision continuation, malformed
layouts, general ownership/admission, lifecycle/recovery, ignored fuzz and
casefold-write/platform activation remain separate.

## Native public-fallback control preparation — 2026-11-06

Four new native controls exercise the existing full-node public path at 1/4 KiB:
`fallback_1k/4k` and `corrupt_full_1k/4k`. They reuse the distinct, checksummed
126/510-route node fixtures and genuinely full selected leaves, and preserve
all existing private declines, root-only controls and retained-index last-slot
tests. Both Rust `apply_link_at` and C `fs_ext4_link_at` execute independently:
four successful fallback profiles and eight checksum/record refusals.

Success requires INDEX clearing; precise root/interior record-span conversion
with valid directory tails; only dot/dotdot in the root and an empty converted
interior block; unchanged old leaf bytes and physical mappings; one appended
linear leaf; every old/new name resolving to the source; and preserved peer
inode bytes. Exact global/group debit is one block, one previously free bitmap
bit flips, inode bitmaps stay identical and used-directory/inode counts stay
unchanged. Parent/source actually share an inode-table block. Source raw changes
are limited to link count/ctime/checksum; parent raw fields are bounded, version
increments once, parent mtime/ctime and source ctime use the single clock stamp.
C attribute identity, generation, flags, mode/ownership, size/blocks/links and
atime/mtime/ctime agree with the post-call source. Remount finds every alias.

Checksum failures and checksum-valid malformed selected records both return
EIO through Rust/C, preserving the complete device bytes, zero writes/flushes,
zero clock calls, dirty-marker state and operation health. Successful mounts
remain healthy. These fixtures are synthetic; they are not Linux-clean claims
or journal/recovery qualification.

All ten intermediate controls pass. The native-only tier retains the exact
prior **1,681-success multiset** plus exactly four: **1,685 successes**, with
all **34** index-boundary controls once. Actual local wrapper **1,932 lines /
150,431 bytes**, floor 440, remains within unchanged 2,000/155,000 limits.
Formatting, full lint and eight Unicode checks pass. An initial bitmap-index
type mismatch was corrected before the passing runs; no production change.
Baseline docs `f10ea83` pass full CI 42300980842. New-source focused/full CI
results remain pending, and independent Linux checking of public
full-node fallback emissions remains the next checkpoint.

Next, use the measured `node-full.img` predecessors for eight Rust/C profiles,
extending the emitter's operation mode explicitly to preserve existing root-
only fallback and retained-index branches. Retain refusal/dirty/closed/cold
images, C attributes, exact allocator/protected-byte checks and complete test-
name multisets. Keep unsupported geometry, general ownership/admission,
lifecycle/recovery and casefold-write/platform gates unchanged. Stages 4–5
remain partial/not quantified; stages 6–7 have not been reached.
