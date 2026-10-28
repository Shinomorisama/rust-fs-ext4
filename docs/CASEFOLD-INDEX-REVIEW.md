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

### Upstream compatibility and kernel-budget evidence closure — 2026-10-27

Reviewed current `ceb3232` after recovery of the interrupted local audit.
Backport driver/test/workflow/dependency bytes remain identical to `fce2afb`;
`3e3d922` changes only the measured kernel budget and documentation. Exact full
CI `40550417452` passes all eleven jobs and ci-ok; retained Stage 0
`40502467008` attempt two passes all ten jobs, resolving the cancelled unlink
reference. The pre-review docs tip also passes full CI `40554014520`.

Five source/digest/all-CRC-verified archives preserve all **1,503 units**, every
old **2,337** whole-suite occurrence plus twelve new ones (**2,349 successes**
in each whole suite), and all **526 controls once in five contexts**. Actual
kernel/whole wrappers and floors pass, all twenty trailing native shell checks
and 159 installed-CLI checks execute successfully, and ARM retains all 527 image
successes. The existing ignored slow fuzz case remains unchanged and remains an
open qualification gate. Kernel output measures **631 lines / 55,214 bytes**;
no further cap change is needed.

All eleven retained archives have verified source identity, advertised digest
and every ZIP CRC. The completed metadata, setter/removal, policy, unlink,
rmdir, symlink and mkdir-reference audits remain recorded in the pre-outage
log. Recovery resumed only the unfinished mkdir emission, Linux create reference
and create emission audits. These now pass all **16/112/48**, **8/48/24** and
**16/112/48** layout/image/creation counts respectively. The reference retains
216 error observations; each emission retains 544 debugfs reports. Local
`image-audit-summary.json` binds the earlier completed-log hash to the verified
archives and the three resumed receipts. The original focused compatibility
run's fourteen images/reports also remain applicable to the unchanged source.

No additional blocker was found in the scoped ordinary-volume compatibility
backports. This closes their pending CI/retention/image review, not broader
ownership/admission, index growth, complete-operation lifecycle/recovery or
Windows/card qualification. No new production code, guard, test, dependency,
workflow, write gate or independent dispatch is introduced by this review.
The native link preparation below now executes/repeats the saved patch and
restores the **1,503-occurrence / 526-control** active baseline with lint.
Independent reference-only link policy collection is next; semantic repair
and emitted-image qualification remain later steps.

### Inline-parent native controls and exact Linux test preparation — 2026-10-28

Eight new native tests now cover same-table inline fit/convert/postfit across
forty API/layout cases, malformed/full-allocation planning refusals, 52 actual
planning-read failures, 176 writes, 56 flush errors, 56 flush unwinds, four C
readback errors and four post-commit publication unwinds. Fault positions are
measured on actual converting calls, with reached-position and retirement checks.
Synthetic declared journals exercise the transaction engine; they are not claimed
as Linux-clean volumes. The final focused run passes eight tests. The cached
readback probe, conservative exhaustion error assumption and poisoned-mount
inspector corrections are documented in the ledger; no production fix is claimed.

The separate `link_inline.rs`/guest test and exact `link-inline-conversion` job
implement the planned sixteen-layout/three-phase, 48-link/160-image matrix.
Actual input shape/shared-table/body-attribute checks, exact one-block accounting,
protected source/parent/EA/table/namespace metadata, dirty/closed/cold evidence,
24 source hashes, pinned tools/siblings and a one-test floor are required.
Top-level inline/helper push paths are included. Existing linear job bodies,
production writers, shared commit/allocation helpers, dependencies and gates
remain unchanged. The new Linux matrix has not executed locally; compile and
selection do not qualify it. Existing allocator exhaustion still returns its
conservative corruption error, not independently qualified Linux ENOSPC behavior.

See the [implementation/evidence ledger](extprogress.md#inline-parent-public-link-native-controls-and-independent-test-preparation--2026-10-28).
Next inspect exact new-source inline/repeated-linear/full CI, repair only real
failures and audit all source-bound saved bytes plus complete retention (expected
1,647 unit/2,497 whole and 670 controls once per context). No new independent
inline qualification, broader growth/counter-wrap/recovery/fuzz or Windows/card/
SteamOS compatibility is claimed; casefold writes remain blocked and stages
4–5 remain unquantified.

### Inline-parent link conversion contract/qualification plan — 2026-10-28

Reviewed current `43280ec` source without changing writer/test/workflow/pin bytes.
The guarded link planner stages source and parent together; inline insertion
tries head, continuation and possible expansion before building a new one-block
directory. Conversion removes only `system.data`, replaces INLINE_DATA with the
selected block map, allocates exactly one block through buffered accounting and
rereads staged source/parent metadata before final time/version/checksum updates.
The present native fit/conversion controls pass, but the older Linux inline test
links a small name and separately converts through create. Generic fault sweeps
use a plain parent; none establishes this exact conversion's write set.

The [bounded plan](extprogress.md#inline-parent-public-link-conversion-contract-and-qualification-plan--2026-10-28)
specifies sixteen Rust/C, 1/4 KiB, direct/journal, 256-byte checksummed,
head-only/populated-continuation layouts. In-place fit, 255-byte-name conversion
and post-conversion fit plan 48 driver links/160 snapshots, not executed results.
Input shape, continuation capacity, actual shared-table placement, complete
allocation/metadata/EA/checksum preservation, dirty/closed/cold Linux evidence,
conversion-specific errors/faults, exact floor-one job and source-bound retention
are required before qualification. Exclude 128-byte/no-body-xattr substitutes;
fail on unsupported guest input rather than silently converting it in setup.

Exact preceding full CI **40761780868** passes all eleven jobs/ci-ok; seven
source/digest/all-CRC-verified archives retain 1,639 units/2,488 whole successes,
662 controls in five contexts and actual wrappers/floors/trailing checks. Four
existing focused native tests freshly pass with exact one-test floors. No new
fixture, native red/fix, independent Linux dispatch or conversion qualification
is claimed. Next implement the bounded test/profile after native contract
preparation. Broader growth/counter-wrap/scoped review and all write, ownership,
recovery/fuzz and Windows/card/SteamOS gates remain; stages 4–5 stay unquantified.

### Corrected linear-growth saved-evidence closure — 2026-10-28

Source `09cf8dc` now passes all thirteen focused jobs in **40744071010** and
all eleven full jobs/ci-ok in **40744071049**; setup-only `aaf5e2a` also passes
full **40748257814**. Both focused archives are API-source/digest/all-CRC and
thirteen-file-hash verified against both trees. Read-only raw-format auditing
checks every **sixteen-layout / 352-image / 112-link** matrix, exact first-growth
one-bit/SB/group debit, unchanged inode allocation/tables/peer/EA bytes,
applicable metadata checksums, dirty/closed/journal state and actual cold Linux
metadata/C snapshots. Each matrix retains **1,536 debugfs / 240 five-pass
e2fsck** reports. Both actual exact-test wrappers and one-test floors pass.
Complete multiset retention against `2137e23` preserves **1,639 unit / 2,488
whole** successes and **662 controls once in five contexts** in both later full
runs, including actual floors and trailing shell/CLI checks. No writer, guard,
test, pin, budget or workflow changes during this checkpoint.

See the [full evidence ledger](extprogress.md#corrected-linear-growth-saved-byte-and-full-retention-closure--2026-10-28)
for artifact IDs/digests, detailed counts, checksum/telemetry treatment and limits.
This closes the corrected healthy linear-growth execution/evidence checkpoint;
it does not close the later scoped implementation review or broader stages 4–5.
Inline conversion qualification is next. Indexed/deeper/uninitialized-group
and counter-wrap growth, ownership/admission/seal/lifecycle/recovery/fuzz and
Windows/card/SteamOS remain open; casefold writes remain blocked.

### Linear-growth EA-sector expectation correction — 2026-10-28

First source 3fe3f92 fails exact growth job 122955259496 in Stage 0 40727685681
at the new input-shape assertion, before any driver link. All twelve other
Stage 0 jobs pass. Source/digest/all-eleven-CRC/thirteen-file-hash-verified
artifact 12833757215 retains a Linux-clean one-block parent with a separate
external EA: inode 2049 maps data block 8453 and EA block 8456, and both raw
inode and Linux stat report four sectors. The new test incorrectly expected
two. Exact native replay on that saved input reproduces red then passes after
requiring the separate EA and counting both blocks. The temporary probe is
restored; no production code, actual growth debit, test selection or floor is
changed. See the [correction ledger](extprogress.md#linear-growth-external-attribute-sector-expectation-correction--2026-10-28)
for source/digest, full evidence and bounded next checks.

Both exact full-suite contexts fail the same new library test (1,343 pass /
one fail); later integration/native trailing checks are unrun. Fresh corrected
native units retain all 1,639 occurrences/662 controls, with full lint passed.
Corrected-source focused Linux and full retention remain pending. This repairs
an input expectation only and does not qualify any driver-emitted growth.
Saved-byte audits, broader growth/counter-wrap and scoped review still follow;
all ownership/admission/seal/mapping/lifecycle/recovery/fuzz and real-platform
gates remain, with casefold writes blocked and stages 4–5 not quantified.

### Full-linear-parent public-link growth test preparation — 2026-10-28

A new exact `link_emission::growth::linux_checks_public_link_growth` test and
pinned `link-linear-growth` job prepare sixteen Rust/C full-linear-parent
layouts. Linux fills one block; raw record checks prove no 264-byte slot for
the first 255-byte alias. All five public forms cycle through a real growing
insertion across layouts. Exact one-block primary-SB/group/bitmap debit,
unchanged inode allocation/peer/EA bytes, complete source/parent metadata and
dirty/closed/journal/cold Linux/debugfs/judged-e2fsck evidence are asserted.
The original allocation-free test remains and shares the same checked runner.
No production writer, shared commit helper, manifest/lock or write gate changes.

All 1,639 native occurrences/662 controls remain; compile, exact one-test
selection, full lint and guest Python syntax pass. Planned 112 links/352
images are **not credited results**; actual focused/full CI and saved-byte
audits are pending. See the [growth preparation ledger](extprogress.md#full-linear-parent-public-hard-link-growth-test-preparation--2026-10-28)
for the complete plan, thirteen source hashes, accounting invariants and next
audits. Preceding cap source 2137e23 now passes exact full run 40649608984,
with 2,487 whole successes/all five-context controls/actual floors/trailing
checks retained in seven verified archives; new-source full retention should
add one host test (2,488), not infer a pass from that earlier source.

This is test preparation, not the scoped implementation/evidence review or
growth qualification. Independent real linear-growth execution/audit is next;
inline/index/indirect/extent/deep/uninit growth, counter wrap and later review
remain. All broader ownership/admission/seal/mapping/lifecycle/recovery/fuzz
and Windows/card/SteamOS gates remain, with casefold writes blocked and stages
4–5 not quantified. Both link patches stay historical/promoted.

### Public-link full-CI retention and native cap follow-up — 2026-10-28

Exact docs-source **741381c**, full run **40640607199**, now passes all eleven
jobs/ci-ok. Only two documentation files differ from emission source e6db258.
Seven source/digest/all-CRC-verified tier archives retain every **1,639 unit /
2,487 whole** success, the new host test once in both whole suites, all **662
controls once in five contexts**, actual wrappers/floors and trailing twenty
native shell/159 installed-CLI checks. Standalone CLI 159, ARM images 527,
oracle 225, kernel 36, third-implementation nine and wasm five also pass.
The same ignored slow fuzz case remains a qualification gate.

Exact e6db258 full run **40637075795** passes every test but native job
**122647162297** fails its complete log's byte cap: **3,916 / 238,064** against
3,950 / 238,000. Its actual later floor/shell/native-CLI steps are unrun.
The later identical-source native suite passes at **3,915 / 237,948**; output
variance does not negate the earlier failure. Complete unmodified log replay
reproduces 65 at the old limit and passes at **3,950 / 239,000**, retaining
all 2,487 successes and the 1,060 floor. Only that byte cap changes. Guest,
unit, other caps, complete output, selection and all floors remain unchanged.
See the [full retention ledger](extprogress.md#public-link-full-suite-retention-and-measured-native-budget-repair--2026-10-28)
for exact sources, artifact IDs/digests, log hash and missing-versus-executed
trailing checks. No new driver tests execute locally and no production source
changes. The new cap commit's actual CI remains pending until observed.

This closes full-suite retention for the new allocation-free host test; it
does not replace the pending scoped source/evidence review or qualify growth.
After exact cap-CI verification, independently qualify real full-linear-parent
hard-link allocation with exact allocator/SB debit and protected inode/peer/EA
preservation, followed by inline/index/legacy/extent/deep/uninit and counter
wrap in later bounded checkpoints. Broader ownership/admission/seal/mapping,
lifecycle/recovery/fuzz and Windows/card/SteamOS gates remain. Keep casefold
writes blocked and stages 4–5 not quantified; both link patches remain promoted.

### Allocation-free public-link exact Linux emission closure — 2026-10-28

Source **`e6db258`** passes exact focused job **122646396891** in **40637075787**.
Advertised SHA-256/source/eleven source hashes/all 4,275 ZIP CRCs bind artifact
**12793499218**. The complete saved-byte audit passes all **sixteen layouts /
112 links / 352 images / 1,536 debugfs reports / 240 e2fsck reports / 128 mounts**,
including 24 requested C target snapshots and 1,928 cold Linux rows. The actual
one-test wrapper/floor passes at **2,141/287,619** lines/bytes; no cap repair.
All expected source/parent/alias metadata/version/maps/content/EA and exact
zero-allocation accounting match, all unrelated allocated inode bytes remain,
and dirty/closed/journal/Linux-copy states and checksums are retained. Linux
observer atime changes affect only disposable copies; actual driver bytes stay
pristine between operations. See the [closure ledger](extprogress.md#allocation-free-public-hard-link-exact-linux-emission-and-saved-byte-closure--2026-10-28)
for exact source, scope, API/source-kind pairings, digest and evidence limits.

Implementation **afd24b8** also now passes all eleven full jobs/ci-ok in
**40633679897**, preserving all 1,639 unit/2,486 whole occurrences, 662 controls
in five contexts, actual floors/wrappers, twenty shell/159 native CLI/159
standalone CLI, 36 kernel and 527 ARM image successes. Native/guest whole logs
measure **3,914/237,852** and **4,081/247,052**;
existing caps fit. One unchanged ignored fuzz case remains a gate. The three
protected writer files are unchanged in e6db258, which adds one host test;
its separate **40637075795** full CI/2,487-whole retention remains pending.

This closes only the healthy ordinary allocation-free independent matrix and
the original implementation's full-CI gate. It does not close independent
growth/conversion/index/legacy/extent/deep/uninit or counter-wrap qualification,
the later scoped implementation/saved-evidence review, corruption/ownership/
admission/lifecycle/recovery/fuzz gates, or Windows/card/SteamOS qualification.
Keep casefold writes blocked and stages 4–5 not quantified. Both link patches
stay historical/promoted. This closure changes documentation only; no fresh
later-tip CI result is inferred.

### Allocation-free public-link independent test preparation — 2026-10-28

The new focused `link-emission` test/job prepares sixteen independent Rust/C
profiles using Linux-created inputs, seven driver-created aliases per layout
and saved dirty/closed driver and post-Linux-copy images. It checks all source
kinds/API forms, exact source/parent/target-attr semantics, zero allocation,
complete descriptor/bitmap/EA/peer preservation, debugfs metadata/version and
cold Linux alias/content/namespace agreement. Judged e2fsck outputs and exact
source hashes are retained. The [preparation ledger](extprogress.md#allocation-free-public-hard-link-emission-test-preparation--2026-10-28)
records the planned 112-link/352-image matrix and evidence limits.

Full native lint, Python syntax and all 1,639 native occurrences/662 controls
pass; production bytes stay at `afd24b8`. Actual independent execution, source/
digest/CRC bindings, saved-byte audit and implementation full-CI retention remain
pending. No pass is inferred from compilation or older Linux reference bytes.
The new focused cap is provisional until actual measurement; existing tier
caps are unchanged. Independent growth/counter-wrap qualification and the scoped
source/evidence review remain separate prerequisites. Keep all broader gates
and casefold write restrictions; stages 4–5 remain not quantified.

### Guarded public hard-link native migration — 2026-10-27

The bounded implementation now uses one shared context for all five Rust/C
forms, a completed link-only owned plan, measured source/parent metadata fixes,
optional full-primary-SB validation/preparation, complete marker/commit/
publication effects and optional C owned target readback. Both formerly dormant
patches are promoted historical artifacts; **do not reapply them**. Fresh reds
repeat 32 passes/91 failures; all 123 original controls and thirteen real helper/
Runtime diagnostics now pass. No public signature, shared nlink/allocator/path
helper, manifest/lock, CI selection/floor or write gate is changed.

The [implementation ledger](extprogress.md#guarded-public-hard-link-native-migration--2026-10-27)
records exact scope, corrected diagnostic/scanner assumptions, 216 remeasured
I/O injections, four actual post-commit publication profiles, all 1,639 native
occurrences/662 controls with prior-multiset retention, full lint and the measured
unit-only cap repair. Real planners distinguish no-SB fits from growth/conversion
SB/debits; foreign/malformed-present-SB failures precede callbacks, and actual
error/flush/publication failures latch before a retained context drops. Runtime
exclusion executes on the real new link clock path and the reverse family paths.
C readback stays under exclusion; errors after commit do not undo/retry a link.
Low-word directory-version wrap/high-word coverage remains an independent
qualification boundary; ordinary increment semantics are the measured policy.

This is a **native implementation checkpoint**, not closure of the scoped
implementation/evidence review. Exact-source full CI/retention is pending, as is
focused independent Rust/C driver-emitted-link/growth qualification and saved-
byte review. Reference-only Linux bytes and successful self-built fixtures do
not establish those outcomes. Review actual CI failures before changing budgets;
only the native unit cap was raised from its measured all-green wrapper failure.
Keep casefold writes blocked and stages 4–5 not yet quantified; broad ownership,
full mapping/index-growth, lifecycle/recovery/fuzz and platform gates remain.

### Executed native hard-link policy reds and reference-CI closure — 2026-10-27

Exact reference source `9fd59cf` now passes all eleven jobs/ci-ok in full run
`40615435814`; docs source `8f0006a` also passes exact run `40616704908`. Five
source/digest/all-CRC-verified archives retain every **2,350 whole-suite success**,
all **526 controls once in five contexts**, all 1,503 units and 527 ARM image
successes. Actual whole wrappers/1,060 floors, twenty shell/159 native installed-
CLI and 159 standalone CLI checks pass. One unchanged ignored fuzz case remains
a qualification gate. Native/guest whole logs are 3,778/225,383 and 3,945/234,723,
inside unchanged caps. The eight-layout/80-image/56-link Linux reference stays
unchanged; no repeated dispatch or emitted-image qualification is inferred.

The [new dormant policy patch](CASEFOLD-LINK-POLICY-RED.patch) actually executes
**39 controls / 12 passes / 27 semantic failures**, zero ignored. All five Rust/C
forms, five existing source kinds and six native profiles reach measured source
ctime, destination mtime/ctime/version and optional C target-attr expectations.
Each policy run actually performs **1,554 successful links / 516 refusals**;
1,290 links reach semantic comparisons. Identity/data/EA/owners/generation,
raw/255-byte names, no-allocation/old-parent/peer invariants and explicit API
path/name/dual-Into/C pointer/ceiling/generation priorities remain. Native six-
profile self-built fixtures are separate from the independent eight-profile VM
reference. No fake helper or unreached Runtime clock/generation hook is counted.

Policy→safety and safety→policy reproduce identical source and both **123-control
runs repeat 32 passes / 91 failures**, zero ignored, retaining every original
84 safety outcome, all 200 I/O/four actual publication injections, 54 original
positive profiles and C readback observations. Standalone/composed clippy passes;
both patches remain dormant and active source/test/workflow/manifest/lock bytes
are restored. All 1,503 native occurrences/526 controls plus full lint remain.
The [executed ledger](extprogress.md#measured-native-hard-link-policy-reds-and-composition--2026-10-27)
records exact patch/source/log hashes, profile counts and evidence limits.

Next implement shared-context public link and completed LinkPlan with measured
metadata repair, optional full SB when genuinely present, whole commit/effect/
publication and optional C owned target readback. Preserve source-before-parent/
name-before-both-Into/path/FFI contracts and source identity/EA ownership. Real
private-boundary and reached Runtime diagnostics come after those real helpers;
re-measure I/O positions when staging changes. Both repaired patches must turn
green before promotion and independent Rust/C emission/growth/full-CI review.
No link guard or semantic repair is yet implemented. Stages 4–5 remain not yet
quantified; casefold writes and broader ownership/lifecycle/recovery/platform
qualification remain gated.

### Independent Linux hard-link reference policy — 2026-10-27

Corrected source `9fd59cf5b5d3c1f00846c9015233594f06adb70e` passes focused job
`122576714109` in [run 40615435825](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40615435825).
Artifact `12785027612`, SHA-256
`78f9944e0e46f2a3ff4ac15fe188a600ae47b750f881e399ee5d77a25ff6ecff`,
verifies exact source/nine hashes and all 2,163 ZIP CRCs. All **eight layouts /
80 images / 56 Linux links / 560 error observations / 960 debugfs reports**
actually execute and pass a complete read-only saved-byte audit, including
80 judged non-repairing e2fsck checks and fresh cold alias identity/payload.
Driver reads of closed images preserve every image byte; no production writer
is invoked by this reference. Actual focused wrapper and floor one pass with
zero failed/ignored. This is non-growing ordinary-directory reference evidence.

Measured link/linkat(0) profiles preserve regular/fast/slow/dangling/FIFO inode
identity, generation/owners/data/EA and exact raw/255-byte spelling; nlink and
source ctime advance, destination-parent mtime/ctime and directory version
advance, parent nlink and old parent/peers remain. Exact bitmap/group/itable-unused
accounting proves no allocation/second EA reference. The first source `0183060`
failed an over-strict test assertion: saved Linux bytes show directory version
6→7 while source version stays 5. Corrected source explicitly requires the
measured directory increment, distinct from inode generation; all eight layouts
now execute and pass. No driver repair is hidden in that fixture correction.

Measured syscall errors remain separate from explicit Rust/C API contracts,
particularly source-directory errno, name/Into/source-before-parent ordering,
root-based paths/destination trailing splitting, pointer/ceiling/generation
checks and optional post-commit C target readback. Casefold write/growth gates
remain intact. This reference does not qualify new Rust/C emitted bytes,
arbitrary growth, ownership admission or crash/recovery behavior.

The [reference ledger](extprogress.md#independent-linux-hard-link-reference-and-saved-byte-audit--2026-10-27)
records source/error/format observations and replay/evidence limits. Fresh local
and exact CI units retain all **1,503 successes / 526 controls once** with lint
and 159 CI CLI checks passed. Preceding `8791f06` full run `40610284789` now
succeeds. Corrected-source full run `40615435814` remains running; complete
2,350-pass whole-suite/five-context/floor/trailing retention is pending.
Next verify that exact full evidence, then execute measured native policy reds
and compose them with the unchanged dormant 84-control safety patch before
LinkPlan/optional-SB/context/effect/readback migration. No link guard/native
semantic repair is implemented; stages 4–5 remain not yet quantified and
casefold writes/broader platform and lifecycle qualification remain gated.

### Executed dormant native public hard-link safety preparation — 2026-10-27

At fetched `90acfc1` (exact full CI `40575650714` now succeeds), the
[native link safety patch](CASEFOLD-LINK-RED.patch) actually executes twice
after reverse/reapply to identical bytes: **84 controls / 20 passes / 64 current
public safety failures / zero ignored**. All **200 measured Rust/C write/flush/
legal-unwind positions**, four proven post-commit publication unwinds and 54
primary positive profiles execute. Both Into priorities, ten qualified reverse
families, eight actual device/reentry profiles and C target readback are reached;
no unreached clock/generation hooks or fake future helper count as reds.

Positive storage controls cover legitimate allocation-free no-SB staging, inline
conversion, ordinary index leaf splits, shared source/parent table slices, legacy
indirect growth and fragmented depth-one extent promotion/later append, plus
uninit-group block allocation without new inode/EA/parent-link accounting.
Source identity/data/owners/generation and raw/255-byte names remain. C returned
readback failure proves one committed entry/count increment with output sentinels
retained; requested read exclusion and unwind retirement expose current gaps.
The unsupported-policy refusal is **not executed casefold growth**. Future
private optional-SB/growth diagnostics wait for an actual LinkPlan boundary.

Active source/tests/manifest/lock are restored; full native multisets retain
**1,503 successes / all 526 controls once**, zero failed/ignored, with active
full lint and dormant clippy passed. This is native self-consistency evidence,
not Linux reference/emission qualification, a guard or a timestamp repair.
The [executed checkpoint](extprogress.md#executed-dormant-public-hard-link-safety-preparation--2026-10-27)
records hashes, exact outcomes/profiles, replay and scope. The patch stays dormant;
prior promoted family patches must not be reapplied. Next execute reference-only
Linux link/linkat-zero-flags policy collection and audit its saved bytes before
native measured reds/composition, guarded migration and independent emission.
Stages 4–5 remain not yet quantified; casefold writes and broader qualification
gates remain blocked/open as before.

### Public hard-link staging, marker and dispatch design audit — 2026-10-27

**Follow-up after the approved upstream compatibility backports:** the normal
restored native baseline is now **1,503 occurrences / 526 controls**; the 1,494
counts below describe the original audit source. The shared walker now treats
a source path's trailing slash as a directory requirement and follows its final
symlink in that case; a source without that slash still preserves its final
symlink. Destination splitting and counted-name/Into priorities are unchanged.
Use those updated contracts for future dormant link controls. The upstream
compatibility closure above now closes the kernel-budget/full-CI and cancelled-
reference gates. The executed dormant native preparation above now supersedes
that next-task status; reference-only Linux link policy collection is next.

Audited clean fetched `c15e588e721ee6300c6dee86214d3a04b0939ec6` using Sol 6.1.
Its exact full CI `40476503989` is still in progress at inspection. Production,
test, workflow and dependency bytes match verified `38295af`, whose exact full
CI `40080977121` and scoped create closure pass. This is a current-source/helper/
existing-test design audit, **not executed link reds, a link guard, a timestamp/
pathname repair or fresh independent link evidence**. No forbidden historical,
kernel or oracle implementation source is read.

#### Five entry forms and actual priority contracts

| Entry | Observed behavior to preserve during guard migration |
|---|---|
| Rust `apply_link` / `apply_link_bytes` | String delegates to bytes. `refuse_write` first; split destination and reject final length; resolve source without following its final symlink; verify live source and reject a directory; resolve destination parent; call public counted API, repeating volume verdict. |
| Rust `apply_link_at` | `refuse_write`; validate counted name **before both Into conversions**; convert target, validate live identity/generation, reject directory and check link room; **only then convert destination parent**, check live directory/generation and duplicates. |
| C `fs_ext4_link` | Null fs/src/dst first, bounded source C string then bounded destination C string before filesystem entry. Paths remain bytes, maximum 4,096 bytes each. Success zero/failure -1 with existing errno mapping. |
| C `fs_ext4_link_at` | Handle check, counted-name null/FFI ceiling first; construct both generation refs and call counted API; optional `attr_out` reads the **existing target inode** after commit. It may return -1 although the link now exists. |

The counted name order is empty, slash, NUL, then length (255 accepted/256
refused). Invalid names prevent both arbitrary conversions; stale/directory/
full-count source prevents destination conversion. Source type/maximum precedes
destination generation/type/duplicate checks. Directory source maps to the
existing `IsADirectory` convention, not a new errno policy. Existing-name and
same-name attempts refuse; hard link has no rename-style same-inode no-op.
Only `NotFound` establishes destination absence; checksum/encoding/other lookup
errors propagate. Preserve `FS_EXT4_GEN_ANY`, explicit generations and symbolic
platform errno constants rather than substituting Linux numeric values.

Path priority differs from counted priority. Destination splitting/NUL/final
length precedes source lookup; source-directory refusal precedes destination
walking, but not those earlier destination checks. The second volume verdict
occurs only after path resolution. Destination splitting trims trailing slashes,
rejects empty/all-slash/separator-free paths, and accepts a non-leading-slash
parent such as `p/new` from root. Source walking accepts current root-based
forms, preserves its final link (including a dangling link), and follows only
intermediate symlinks; the destination-parent walk follows its final symlink.
Do not replace this with a generic two-parent resolver, absolute-only rule,
source final-symlink dereference, new trailing/NUL rule or Linux syscall error
order without separate measurement and executed semantic reds.

Current `refuse_write` marks the primary SB before these name/identity/duplicate/
allocation refusals. There is no shared link context; arbitrary Into/device
callbacks can run outside exclusion. The journal mutex is acquired during commit,
after duplicate/count planning, and is absent on direct mounts. These source
observations define controls; they are **not measured red results**. No claim is
made that every current failure path lacks existing journal/direct protection.

#### Completed staging, existing source identity and optional superblock

Link creates an entry for an **existing live non-directory inode**, not a new
inode. Regular files, fast/slow symlinks and supported special-file types must
not be silently restricted to regular files. Count zero/freed/reused identities
stay stale. `check_link_room`/`next_links_count` enforce **MAX_LINKS 65,000** for
files; `DIR_NLINK` does not make a non-directory source unbounded. A successful
link must retain source inode/generation, mode/owners, data/maps, size/sectors and
EA ownership/refcount; it increments that inode's link count once. Sharing a name
does not create a second inode or a second EA reference. Parent link count does
not increase for a non-directory entry.

`patch_inode_nlink` changes the count and its inode checksum only; it is shared
by mkdir/rmdir/rename accounting and must remain unchanged. The current link body
has no explicit source ctime or destination-parent mtime/ctime update and no
direct Runtime clock/generation call. Those timestamps are candidates for
independent Linux measurement, not established semantic reds. Do not copy
create's Clock/generation-unwind tests and count an unreached hook as a failure.
Later measured time patches must read the latest buffered source/parent inode
and preserve shared inode-table splices, inline entries and growth mappings.
A valid non-directory source cannot also be the same valid directory inode;
keep the existing rejection order for that input, without manufacturing a
same-inode two-role positive fixture.

Both current terminal branches already commit **one complete BlockBuffer**:
source nlink/CRC plus destination entry, including all supported parent growth.
The older “Not journaled” comment is not an accurate description of the body.
`buffer_add_dir_entry_inplace` handles linear/indexed or inline insertion/
expansion/conversion; **only OutOfBounds** requests the growth helper. Inline
conversion, ordinary htree split/fallback and legacy direct/single-indirect or
extent/deep append stage allocations, bitmaps, BGD/SB counters, mappings and
checksums in the same buffer. Read staged inode slices through
`buffered_inode_verified`; do not overwrite source edits when parent and source
share a table block. Unsupported casefold insertion/index dropping remain
refused, not an allocation/growth signal.

**Unlike create/mkdir, a non-growing link legitimately has no staged primary SB.**
It allocates no inode/block and changes no free or used-directory count; its
source/table/entry buffer may omit that block entirely. Inline conversion or
parent growth can stage a full primary-SB block with allocator debits. A future
link-only boundary therefore validates/dirties/restamps the **full block when
present**, following the optional-SB principle already used by unlink. Do not
mechanically copy create's mandatory-SB check. Actual growth/conversion planner
fixtures must prove their required SB/debits are staged; an optional check is
not an ownership seal or permission to drop allocator edits. Constructed missing-
SB rejection must not be called a public bug when the legitimate plan is
allocation-free. Pending uninit flags, staged bitmap padding/CRC, descriptor
updates and publication remain the existing allocator helpers' responsibilities.

`commit_block_buffer` clones pending flags, locks/builds/commits/checkpoints the
journal or orders/flushed direct writes, then publishes allocation overrides.
The entire call belongs in the latching effect, **even for an empty override
map**. Current publication can still run for a non-growing link. Returned errors
and legal marker/flush/publication unwinds must retire the mount across every
qualified family, flush and clean close. Ordered direct writes are not a
power-loss atomicity claim. Keep shared raw commit/effect code unchanged.

#### Chosen bounded migration design

1. All five Rust/C forms enter the existing nonblocking mount context once,
   before device/runtime/arbitrary target or destination Into hooks. Private
   nested byte/counted helpers borrow it, retaining both path volume verdicts,
   the early path source-directory check and exact counted conversion order.
2. Extract a nominal private **completed LinkPlan** owning the target identity
   needed by readback and the entire finished buffer. Keep shared nlink,
   allocator, insertion/growth and generic path helper contracts unchanged.
   Converge successful in-place/growth branches before one link-only commit.
3. That boundary checks context identity/health before callbacks/effects. No
   primary-SB block is valid for genuine allocation-free staging; when present,
   require full block length, correct SB offset/magic, preserve all allocator/
   state/outside-region bytes, clear only the clean bit and restamp its CRC.
   The trusted plan is not a generic seal. Required growth SB presence/debits
   must also be tested at the real planner, not inferred from an optional check.
4. Use the existing EffectFailure-protected context for one-time marking and
   the whole raw commit/checkpoint/flush/publication operation. Ordinary
   pre-effect refusals leave bytes and pending state untouched and release a
   healthy context; legal planning/conversion unwinds retain the existing
   conservative retirement policy. Never retry a partially uncertain link.
5. Add a **link-specific optional C owned-inode snapshot adapter** holding the
   same context through requested target `stat_ino` readback. Format only the
   snapshot afterward; do not refactor shared `attr_out` or recursively call a
   public guarded method. NULL skips extra reads. A normal read error can follow
   one committed entry/count increment and leaves attr sentinels untouched;
   an unwind crossing the held context latches retirement. Keep ABI signatures,
   source/parent generations, bytes and zero/-1 conventions unchanged.

#### Executed preparation and independent evidence required next

Existing coverage includes C identity/content/remount/chained links and error
cases (`capi_link*`), Rust/C inode lifetime and stale-generation parity,
`writes_verify_the_block_they_restamp`, link-count boundary native tests, byte
names, ordinary htree/inline insertion, mixed inode-API e2fsck and journal/crash
cases. Shared unlink fixtures create aliases before arming their callback hooks;
production qualified operations do not call public link internally. Preserve
all these tests and earlier successful occurrences. Prior mixed-operation/crash
tests do not substitute for focused operation-wide exclusion/retirement or new
measured hard-link timestamp/path policy evidence.

The next checkpoint must actually execute/save a dormant native safety/positive
patch, reverse/reapply to identical source and repeat complete names/outcomes,
then restore production/test bytes and verify the normal **1,494 occurrences /
526 controls** plus lint. Use `scripts/tier.sh`; compile/fixture failures and
unreached hooks are not driver reds. Do not promote intentionally red controls
into normal CI or introduce a fake private commit helper.

| Control group | Required executed coverage |
|---|---|
| Entry/conversion | Held context refuses all five forms before device properties/reads and both Into hooks; release permits success. Counted invalid names precede both conversions; invalid source/type/MAX_LINKS precedes parent conversion. |
| Refusal priorities | Clean/already-dirty invalid/long/duplicate/dot/stale source/stale parent/directory/non-directory/MAX_LINKS/corrupt-inode-or-dir/unsupported-growth errors retain exact bytes, source nlink, counters, pending flags and healthy ordinary reuse. Volume/second verdict and earlier C pointer/ceiling checks remain. |
| Reentry | Link callbacks cannot reenter itself or any **ten qualified families**; callbacks in each qualified family cannot enter link. Use actual observed size/read/writable/marker/commit/flush and conversion hooks; no fabricated Rust Runtime hooks or illegal unwinds through extern-C callbacks/overlapping ABI references. |
| Positive identities | Existing regular, fast/slow symlink and supported special-file identities; raw/255-byte names, aliases in same/different parents, source generation/mode/owner/data/EA/maps intact, nlink +1, unchanged parent nlink and no inode allocation. Existing names refuse rather than rename-style no-op. |
| Storage/planning | Real fresh non-growing linear/inline plans with no staged SB, same table-block source/parent slices, inline conversion, full parent and supported index/legacy/extent growth, multi-group/uninit accounting. Only actual parent data/map blocks are charged; keep source data/EA, inode bitmap/itable-unused and inode/used-directory counts unchanged. |
| Uncertainty | Measure a successful call's actual writes/flushes in each direct/journal × 1/4 KiB × clean/dirty Rust/C profile; inject every observed position, aggregate only after all ran. Do not import create's 304 count. Prove retirement across all eleven families/flush/close. |
| Publication/unwind | Actual public post-commit publication unwind asserts destination presence and source nlink increment before retirement. Inject only after last planning allocation-state reads; an early poisoned lock is not a publication result. Exercise both Into unwinds and retained-context returned errors/unwinds once the real helper exists. |
| C readback | Held context through optional target read; updated source nlink/identity attrs agree. NULL avoids reads; normal failed read leaves one link/count increment and unchanged output sentinel, without retry or rollback claim; legal readback unwind retires. |
| Future private boundary | Only after the actual helper exists: foreign identity, valid no-SB allocation-free plan, malformed present SB, growth SB/debits/state/checksum/outside-byte preservation and retained-context failures. Keep these separate from current public marker/refusal reds. |

After reproducible native safety preparation, execute **reference-only Linux
hard-link/linkat-with-zero-flags** measurements inside the existing CI VM. Linux
creates all reference bytes; initially the driver only reads closed images.
Measure source ctime, destination-parent mtime/ctime and unaffected old parent/
peers; explicit source mode/owner/generation/EA/data, no new inode/block for
non-growing parents, exact link counts and accounting; regular/fast/slow/dangling
symlinks and source final-link identity; source versus destination invalid/missing/
long/dot/trailing/rooted-path error priorities. Preserve API-versus-syscall and
symbolic errno distinctions rather than silently rewriting contracts. Save
before/after images, complete debugfs reports, judged non-repairing e2fsck and
cold kernel namespace/identity/content/metadata records. Audit the actual saved
bytes before executed measured semantic reds and their patch composition.

Only then migrate the guard/owned-plan/optional-SB/readback boundary and measured
policy repairs, retain every prior control, and independently qualify actual
Rust/C emissions across legacy/checksummed 1/4 KiB direct/journal profiles.
Qualify actual parent growth separately from no-allocation cases. Complete full
CI occurrence/floor/trailing retention and standalone source/evidence review
precede scoped link closure. No source/test/workflow/task/cap/floor/pin/lock or
write-gate change occurs in this design audit. Stages 4–5 remain not yet quantified;
casefold writes and broader ownership/alias/mapping/index-growth/lifecycle/
recovery/ignored-fuzz/Windows/SteamOS/card qualification remain open.

### Scoped regular-file create full-CI review closure — 2026-10-27

Budget repair `38295af4cc23ec93253c9edd963cffde73bbd692` passes every job and
ci-ok in [exact full run 40080977121](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40080977121).
Five source/advertised-digest/all-CRC-verified tier archives retain all **2,337
successful occurrences** in both whole suites, zero failed and one unchanged
ignored slow fuzz test. All **526 controls once in five contexts**, all 1,494
unit occurrences and 527 architecture image passes remain. Actual native and
guest wrappers and **2,337 executed / 1,060 floors pass**. Native **twenty shell
and 159 installed-CLI checks now execute and pass**, with exact prior names/
verdicts and CLI-file counts retained; standalone CLI also passes 159.

Native whole output is **3,746 lines / 223,509 bytes**; guest **3,915 / 232,808**,
inside the repaired caps. Implementation/test/workflow/manifest/lock/agent bytes
match independently audited `f436f4e`. Its sixteen-layout/112-image/48-creation
emission, retained-family image audits and completed source/native fault-boundary
reviews remain applicable; no unchanged large image audit is repeated here.
No further blocker is found under **caller exclusion, healthy layouts and closed
external reference handles**. **Scoped create source/evidence/full-CI review is
closed**, alongside unlink/rmdir/mkdir. All create/mkdir patches are historical/
promoted and must not be reapplied.

The [closure ledger](extprogress.md#scoped-create-full-ci-review-closure--2026-10-27)
records exact provenance, full name/control multisets, actual floors and trailing
checks, hashes and scope limits. Next is a bounded public hard-link (`link`)
Rust/C staging/marker/dispatch and source/destination identity design audit,
including shared helpers and optional readback, before new native reds or
migration. That audit is not claimed complete. Ownership/alias, mapping/index
growth/seals, other writers/retained handles, lifecycle/C ownership, recovery,
ignored-fuzz and Windows/SteamOS/card remain open. Casefold writes stay blocked;
stages 4–5 remain not yet quantified.

### Independent create emission and saved-byte audit — 2026-10-24

Exact `f436f4e0bb3a220f3f25dfb62cfebd2fc8041bff` passes focused create job
`120817437070` in [run 40075365381](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40075365381),
with one executed test/floor one, zero failed/ignored and actual wrapper output
**893 lines / 113,247 bytes**. Artifact `12553351249` has verified source,
advertised SHA-256 `4d74027244a5836504286224bc1e2e11ddb3f01cae08017dc6c92484e0be02e7`,
all 1,329 member CRCs and a complete read-only saved-byte audit.

All **sixteen Rust/C layouts / 112 images / 48 creations / 496 refusal
observations / 544 debugfs reports / 432 cold Linux inode records** pass. Raw
empty regular-file identity, byte/255-byte names, explicit modes, parent times/
atime/links/maps/EA, peers and journal dirty/clean state agree with independent
Linux/e2fsck. **280 exact block-bitmap and 280 inode-bitmap/itable-unused checks**
prove only the expected inode allocations with no block/used-directory delta.
All primary/BGD/inode/directory/EA/bitmap and present journal-SB checksums pass.
The matrix uses asserted non-growing parents; it does not qualify arbitrary
parent growth, casefold indexes or interrupted writes.

Every focused job and all fifteen main focused logs now pass. Fresh current-source
retained metadata/setter/removal/unlink/rmdir/mkdir/reference/symlink byte audits
pass. Complete verified CI archives retain **2,337 whole-suite successes** in
both suites: all 2,228 earlier occurrences plus 108 controls/one emission; zero
failed and one unchanged ignored slow fuzz test. All **526 controls in five
contexts**, 1,494 units and 527 architecture image passes remain; standalone CLI
passes 159. Actual native and guest wrappers fail only measured output caps,
exit 65; whole floors and twenty shell/159 native CLI trailing checks are unrun.

Native actual output is **3,745 lines / 223,395 bytes**; guest **3,913 / 232,693**.
Identical complete-log replays fail old and pass measured new **3,950 / 238,000**
and **4,120 / 248,000** limits. Only the two whole caps and their explanation
change; floors, selection, complete output, driver/test/workflow/pins and write
gates remain intact. Corrected full CI/ci-ok and actual trailing execution still
require a fresh run, so scoped create closure remains **pending**.

The [evidence ledger](extprogress.md#independent-regular-file-create-emission-evidence-and-saved-byte-audit--2026-10-24)
records provenance, raw preservation, full occurrence/control retention, actual
wrapper failures and replay proof. The completed source/evidence walk finds no
further scoped blocker under caller exclusion, healthy layouts and closed
external handles. Next verify the budget-repair commit's exact CI, full retention,
actual floors and trailing checks before closing this family review. All broader
ownership/alias/mapping/index-growth/seal/lifecycle/recovery/ignored-fuzz/Windows/
SteamOS/card gates remain open. Casefold writes stay blocked; all create/mkdir
patches remain historical/promoted.

### Guarded regular-file create repair and native validation — 2026-10-24

At fetched `e2d1cc0`, both preceding exact full runs `40053837295` and
`40051856797` report completed success. All 99 saved native outcomes reproduce
**28 passes / 71 failures** before production changes. The bounded repair now
turns them green and adds nine controls against the real create boundary:
**108 pass**, zero failed/ignored. Native units retain every 1,386 previous
successful occurrence and add precisely those controls: **1,494 passes / 526
operation controls once**, no failed/ignored; full lint passes.

All five Rust/C forms hold one existing context before hooks, through completed
owned `CreatePlan`, full staged primary-SB validation/dirty-state/checksum repair,
marker, entire raw commit/checkpoint/allocation publication and optional C owned
inode readback. Shared partial allocation, other creator/path helpers and raw
commit/effect machinery remain unchanged. Parent mtime/ctime use the latest
staged parent after entry/growth; regular/missing-prefix errors precede final
length only in create's path adapter. Counted Into-before-name, FFI priorities,
explicit modes/byte names and current root-walk/trailing contracts are retained.

The repaired success is remeasured before each sweep: **304 actual write/flush
injections**, zero retirement violations, plus **four public post-commit
publication unwinds**. All 54 primary positive profiles and 270 policy creations /
714 priority/refusal observations remain. New private controls exercise foreign/
malformed staging, full-SB debits/state/outside bytes, retained-context returned
errors/flush/publication unwinds and the second pure verdict. Constructed SB
controls diagnose the new real boundary; they are not pre-fix public clean bugs.

Actual normal unit output exceeds its old caps at **1,742 lines / 132,491 bytes**,
exit 65 with all tests green. Identical complete-log replay fails old and passes
new **1,860 / 143,000** caps, preserving selection/floor/output. No whole-suite
cap is raised without an actual measured overflow. A sixteen-layout Rust/C
regular-file emission test and exact floor-one focused job are added, planning
**112 saved images / 48 creations**, judged e2fsck/debugfs/cold Linux reads and
exact empty-file/parent/peer/allocation/journal preservation.

The [checkpoint ledger](extprogress.md#guarded-public-regular-file-creation-and-measured-policy-repair--2026-10-24)
records implementation, native outcomes, counts and log provenance. Actual
implementation CI/emission, saved-byte audit, full-suite/retained-family retention
and standalone scoped review are **pending**. Next inspect exact runs and qualify
those results before create closure. Both create red patches are historical
promoted artifacts; **do not reapply**, likewise mkdir's historical patches.
Ownership/alias/mapping/index-growth/seal/lifecycle/recovery/ignored-fuzz/actual
Windows/SteamOS/card gates remain open; casefold writes remain blocked.

### Executed measured native create policy reds — 2026-10-23

At fetched `365ffea`, the new dormant
[27-control policy patch](CASEFOLD-CREATE-POLICY-RED.patch) executes **fourteen
passes / thirteen failures / zero ignored**, exit 101, complete log **120 lines /
21,877 bytes**. Ten first/back-to-back timestamp controls cover all five forms;
three mixed pathname controls cover Rust string/bytes/C path. Every red completes
all six ext2/ext3/ext4 × 1/4 KiB profiles before aggregating mismatches. Parent
mtime/ctime remain stale; regular/missing prefixes lose to final-name length.
Independent Linux reference already establishes the intended corrected behavior.

All **270 policy creations / 714 refusal/priority/syntax observations** execute,
retaining empty regular-file fields and one-inode/no-child-block accounting,
parent links/atime/storage/EA/peers, raw/255-byte identities, explicit low-twelve
modes, counted **Into-before-name**, name-before-stale and FFI priorities. Current
empty/separator-free and rooted-parent/basename-trimming/trailing contracts stay
explicit. Linux numeric errno/empty/trailing answers do not silently override
these API contracts or shared `mknod`/symlink/generic parent helpers.

Policy→safety and safety→policy recreate byte-identical combined source. Both
actual **99-control runs report 28 passed / 71 failed / zero ignored**, exit 101,
**1,014 lines / 80,065 bytes** each. Every original 72 safety verdict, all **288
I/O injections / four real public publication unwinds**, safety-positive profiles
and new policy outcome is retained. No future constructed-SB diagnostic or
compile/fixture failure is counted as a current public red. Real private-boundary
foreign/missing/malformed-SB/retained-context controls wait for promotion.

Full lint passes with both audits active. Both patches are reversed again and
normal units retain all **1,386 occurrences / 418 controls once**, zero failed/
ignored. The original safety artifact stays byte-identical; the new 30,406-byte
policy patch has SHA-256
`36d6d325594c8548a2ee6b7b7a4b1a9712fc2628f41e7894733f27b567b9ab32`.
The [executed ledger](extprogress.md#executed-native-create-policy-red-preparation--2026-10-23)
records complete coverage, reproducibility, restoration and remaining work.

Next inspect exact checkpoint CI, reproduce saved reds, then implement the
reviewed create guard/private dispatch/completed owned plan, create-only full-SB
and complete commit/publication effects, advancing parent times and measured
create-local mixed-prefix correction. Keep optional C readback inside the same
context and preserve error-after-creation/null semantics. Add actual private-
boundary controls only when that helper exists; preserve all qualified families
and explicit API priorities. Independent Rust/C emitted-image/full-CI/standalone
review closure remains required after native repair. No production/task/workflow/
limit/pin/sibling/write-gate change here. Stages 4–5 remain not yet quantified;
casefold writes and broader ownership/mapping/lifecycle/recovery/Windows/SteamOS/
card qualification stay blocked. Do not reapply promoted mkdir patches.

### Verified independent Linux exclusive-create reference — 2026-10-23

Exact `e384ed9` passes all nine focused jobs in `40033593125`. The create policy
job `120687552982` executes one pass, zero failed/ignored, **669 lines / 80,503
bytes**, actual floor one passed. Archive `12536511380` matches source and
advertised SHA-256
`d202498a3e751014d440c5b039b3c1f0792293d1e4177bf796d2ea3d5d958ec7`;
all 553 member CRCs pass. Read-only compressed-image audit passes **eight
layouts / 48 images / 24 creations / 216 consistent errno observations** and
all **408 debugfs reports**, with final cold Linux byte namespaces/empty files.
Actual calls are eight mke2fs, 48 judged e2fsck, 408 debugfs and 48 kernel mounts.

Raw fields/mappings/times/CRCs/peers/EA agree with independent outputs. All 120
block-bitmap comparisons remain byte-identical; all 120 inode-bitmap/itable-
unused comparisons change only for actual new identities. Per-layout allocation
is **zero blocks / -3 inodes / zero used directories**; parent links/storage/
atime stay intact, parent mtime/ctime advance, and closed journal state is clean.
Requested/actual regular-file modes **02750/01770/0750 match in every profile**.
Regular-file/missing prefixes produce ENOTDIR/ENOENT before overlong final names;
valid parent/256-byte cases produce ENAMETOOLONG. Existing/trailing and empty
syscall observations are recorded separately from explicit API shortcuts and
counted Into/name/generation/FFI/mode contracts; no automatic rewriting follows.

Exact full run `40033593162` passes all jobs/ci-ok. Five source/digest/CRC-verified
current log archives retain every 2,227 preceding occurrence plus one oracle:
**2,228 passed / zero failed / one unchanged ignored fuzz** in both whole suites.
All 418 controls occur once in five contexts, all 1,386 unit occurrences and 527
architecture image passes remain, and trailing twenty shell/159 installed-CLI
checks actually pass. Whole native/guest output fits unchanged caps at
3,635/212,417 and 3,802/221,719; actual floor commands pass. Earlier design/native
safety full runs also now pass. The ignored fuzz case remains a broader gate.

The [verified ledger](extprogress.md#verified-independent-linux-create-policy-evidence--2026-10-23)
records provenance, actual policy table, checks and retention. Reference-only
qualification is complete within these ordinary healthy inputs; driver emission,
guarded create and broader ownership/mapping/lifecycle/recovery remain open.
Next execute/save native measured parent-timestamp and mixed-prefix policy reds,
compose with the unchanged dormant 72-control safety patch in both orders, retain
all original outcomes/288 I/O/four publication coverage, lint and restore all
1,386 units/418 controls before migration. Preserve explicit API/Into/FFI/rooted-
path/trailing contracts and other shared creation helpers; do not reapply promoted
mkdir patches. No production repair, limit/pin/sibling or write-gate change.
Stages 4–5 remain not yet quantified; casefold writes and Windows/SteamOS/card
qualification stay blocked.

### Independent Linux exclusive-create reference added — 2026-10-23

One host-only reference test now covers eight 1/4 KiB × 128-byte legacy/
256-byte checksummed × direct/journal profiles. Linux alone creates the bytes;
all Rust mounts are read-only. Six saved phases plan **48 images / 24 exclusive
regular-file creations / 216 actual error observations**, with a final fresh
kernel mount checking exact byte names and empty data. Existing peers, raw
inodes/EA, parent links/storage/atime, child fields/times/modes/mapping, bitmaps,
per-group free-inode debits/unchanged blocks and used dirs, checksums and clean
journal state are checked. Expected independent calls are eight mke2fs, 48
mounts, 48 judged e2fsck and 408 debugfs reports; none is claimed executed here.

The guest records uncertain error codes and full requested/actual special modes
under explicit exclusive-open flags/root/zero-umask/no-default-ACL parents.
It does not substitute mkdir or native API answers. An extra cold phase verifies
all three closed files after remount. A separate exact focused CI job enforces
floor one and retains every image/report/log; no global cap/floor/filter changes.

Python syntax, Rust compilation, 94 CI profile/contract checks, full lint and
all **1,386 native unit occurrences / 418 controls** pass locally. The dormant
72-control safety patch and production create/other helpers remain unchanged.
**Actual Linux execution, saved-byte audit and exact full CI remain pending**;
no semantic repair or independent pass is inferred. The
[collection ledger](extprogress.md#independent-linux-create-policy-oracle-added--2026-10-23)
records fixture shape, planned evidence, local validation and next audit.
Inspect exact-source CI and source/digest-verified saved artifacts before native
semantic reds or migration; fix actual failures without weakening expectations.
Stages 4–5 remain not yet quantified; casefold writes and broader qualification
stay blocked.

### Executed dormant public-create native safety controls — 2026-10-23

At fetched `0d38ff1`, the new [native safety patch](CASEFOLD-CREATE-RED.patch)
actually executes, reverses/reapplies to identical source and repeats all
**72 names/outcomes: fourteen passes / 58 failures / zero ignored**, exit 101,
complete logs **908 lines / 58,680 bytes** each. All 58 are current public
exclusion/refusal/reentry/unwind/C-readback/retirement assertions, not future
constructed-SB diagnostics or unmeasured Linux semantics. The real private
boundary and its foreign/missing/malformed-SB controls remain promotion work.

All **288 measured I/O injections** reach their actual callback positions:
per Rust/C interface, 88 write errors, 28 flush errors and 28 flush unwinds over
direct/journal × 1/4 KiB × clean/dirty layouts. Four real public publication
unwinds assert committed namespace before the aggregate retirement verdict.
The complete ten-family/flush/close shared-latch contract fails; this does not
claim existing partial direct/journal protections are absent. All nine-family
callback directions, self-worker/same-thread and optional C readback callbacks
execute. Clean refusals write the early marker; prior-dirty refusals preserve
bytes/errors/context. All twelve pathname observations execute before aggregate
assertion, and create-specific Into/name/volume/stale/FFI priorities are retained.

All **54 primary positive profiles** and additional same-table/max-link/rooted-
path/reuse/readback checks execute. Empty regular inode fields/mapping/mode/
identity/CRCs, exact group/SB/bitmap accounting, unchanged directory counts and
parent links, inline conversion/growth and peer/EA preservation pass. A real
constructed 2 KiB/two-group uninit allocation profile reaches two distinct
file inodes and one parent growth block with correct flags, padding/itable-
unused/counters/CRCs/publication/reopen. Four injected C readback errors preserve
one creation and attr sentinels; NULL avoids the extra read. Native positives
do not establish independent Linux policy or arbitrary ownership admission.
Two discovery legacy-listing test-harness failures are corrected with verified
mapped-block inspection and are not counted as create defects.

Full lint passes with the audit active. The patch is reversed again and restored
normal units retain every **1,386 occurrence / 418 controls**, zero failed/ignored.
No production/test/workflow/task/cap/floor/filter/pin/lock/sibling or write-gate
change remains active. Exact preceding CI `40025606526` still has the native job
running at inspection, with all other jobs passed; no ci-ok result is inferred.
The [executed ledger](extprogress.md#executed-native-public-create-safety-preparation--2026-10-23)
records provenance, repeatability, controls, fixture correction and scope.

Next actually execute and audit independent **reference-only Linux exclusive
regular-file creation** for advancing times, names/mixed errors, controlled
modes/owner/umask and resource accounting before measured semantic reds or the
reviewed guarded repair. Keep explicit API/Into/FFI/rooted-path contracts distinct;
keep the dormant patch inactive in ordinary CI and both mkdir patches historical.
Stages 4–5 remain not yet quantified; casefold writes and broader ownership/
mapping/lifecycle/recovery/ignored-fuzz/Windows/SteamOS/card gates remain blocked.

### Public-create staging, marker and dispatch design audit — 2026-10-23

Audited clean fetched `1248f71f1cb6032d1ebb1d7c0c9cd25e9d8f70b3` using Sol 6.1.
Its exact [full CI 40013659564](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40013659564)
now passes every job and ci-ok. The preceding repair-source occurrence/image
baseline and scoped mkdir closure remain applicable: implementation/test bytes
are unchanged. This is a current-source/helper/test design audit, **not executed
create reds, a guard migration, new Linux policy/emission evidence or ownership
admission**. No historical non-clean implementation or oracle source is read.

#### Observed entry contracts and effects

| Entry | Current source behavior | Required migration behavior |
|---|---|---|
| Rust `apply_create` / `apply_create_bytes` | String delegates directly to bytes. `refuse_write` runs before `resolve_new_parent`; bytes then call public `apply_create_at`, repeating the volume verdict. | One shared nonblocking context before device/runtime callbacks and reads, then private nested dispatch with both pure volume verdicts. No recursive public entry. |
| Rust `apply_create_at` | `refuse_write`, caller `Into<InodeRef>`, then `plan_new_inode_in_dir`: empty/slash/NUL/length validation, allocated inode/generation/type check, duplicate scan and allocation. | **Preserve conversion before counted-name validation**, unlike mkdir. Acquire context and pure verdict before conversion. Invalid counted names still precede parent liveness after conversion. |
| C `fs_ext4_create` | Null fs/path and FFI path-length checks precede the byte API; valid path bytes need not be UTF-8. Success returns the inode, failure zero and symbolic platform errno. | Keep pointer/string checks and return convention; use the same filesystem guard, not a second ABI lock. |
| C `fs_ext4_create_at` | `ino_call` checks the handle; `name_arg` validates null and the FFI slice ceiling before filesystem entry. It creates, then optional `attr_out` reads the inode. Success is zero, failure -1. | Keep those FFI priorities and optional output. Read an owned inode snapshot under the same context after commit, then format without further filesystem access. A readback error can follow successful creation. |

`refuse_write` calls `write_refusal` and then `mark_not_clean_once`. The first
allowed call can read/write/flush the primary SB before path syntax, parent,
duplicate, name, allocation or runtime refusal/unwind. Create has no shared
operation context. The journal mutex is acquired only inside commit, after
allocation and duplicate planning; it cannot serialize the full operation and
is absent on direct mounts. These are source observations defining future
controls, not measured red outcomes. Existing direct/journal failure checks may
already satisfy some controls; do not predict all faults will be red.

The exact existing path rules matter. `split_parent_and_base` trims trailing
slashes, rejects an empty/all-slash path and a path with no separator, and
rejects NUL in the basename. `resolve_new_parent` rejects a final component over
255 bytes **before** walking the parent, then follows parent symlinks. The
walker starts at root even for a non-leading-slash parent: `p/new` has a slash
and can resolve from root, while `new` fails at splitting. Do not silently
introduce an absolute-only rule, Unix current-directory semantics, a new trailing-
slash policy or mkdir's measured mixed-prefix ordering. Measure any proposed
pathname change independently first; keep generic helpers unchanged meanwhile.
Dot names reach the existing duplicate lookup rather than a new special-case
refusal. Only `NotFound` means absent; checksum/encoding/other lookup errors
propagate. Counted input remains byte-based, with 255 accepted/256 refused;
validation order is empty, slash, NUL, length, after `Into` but before liveness.
Volume refusal remains before those Rust checks; FFI null/ceiling remains earlier.

#### Completed-buffer and allocator obligations

`NewInodePlan` is currently **partial setup**, not a completed create result.
`plan_new_inode_in_dir` validates parent/name/absence, prefers the parent's group
and plans one **non-directory inode** (`is_dir=false`). It stages inode bitmap,
BGD/SB free-inode debits and zero used-directory delta. Reserved-inode/range
checks, full groups, uninit override views, bitmap padding/CRC, `itable_unused`
and descriptor CRC remain the existing helpers' responsibility. Shared callers
are regular create, special-file `mknod` and symlink; changing that helper's
priorities or adding create policy there would affect all three.

`build_regular_file_inode` constructs `S_IFREG | (mode & 0x0fff)`, one link,
zero size/sectors and zero initial owner/group. Ext4 uses the EXTENTS flag and
an empty depth-zero root; ext2/ext3 retain zero direct/indirect pointers and no
extent flag. Extra-isize is initialized; child timestamps and generation come
from the runtime, and the inode checksum is stamped for the allocated identity.
No child data block, directory dot records, CASEFOLD/INDEX/INLINE inheritance,
credential/umask/default-ACL policy or parent link increment is added here.
Preserve that API contract pending independently measured, separately reviewed
policy work; ordinary Linux syscall results must not silently redefine it.

The child inode is spliced through `buffer_write_inode`, retaining other edits
in the same table block. Parent insertion uses the same buffer:

- Inline insertion/expansion or conversion retains all entries and its staged
  inode. Conversion allocates/maps parent storage in that buffer and strips
  inline storage without overwriting the new child sharing its table block.
- In-place linear/indexed insertion preserves routing and checksum/error rules.
  **Only `OutOfBounds`** requests growth; other failures propagate.
- Growth attempts an ordinary htree split, may drop an ordinary index under the
  existing fallback, and appends with legacy indirect or extent/depth helpers.
  Casefold index dropping and unqualified insertion stay refused. A guard does
  not qualify casefold routing, collision splitting or arbitrary mapping shape.
- Later parent allocations consult staged bitmaps and uninit flags. Actual data
  and mapping-node allocations must be distinct and fully charged. Ext2/ext3
  single-indirect growth bounds and deeper extent refusal/propagation remain.
- Both terminal create branches currently call **one complete**
  `commit_block_buffer(buf)` and return the new inode. Older “Not journaled” /
  “commits early” comments do not describe the current body. The growth helpers
  stage into the supplied buffer rather than making a separate create commit.

The ordinary in-place create body does not explicitly stamp parent mtime/ctime;
that is a candidate for advancing-clock Linux measurement, not a demonstrated
semantic red in this audit. A later parent metadata patch must read the latest
`buffered_inode_verified` **after** insertion/growth and preserve child bytes,
parent mappings/inline entries, link count, atime and unrelated attributes.
Do not restage the pre-insertion inode snapshot over those edits.

`commit_block_buffer` clones pending uninit state, builds/commits the journal
transaction and checkpoints, or orders/flushed direct writes with SB last; then
it publishes allocation overrides. The entire call, including clone/lock/
publication errors or unwinds, belongs inside the latching effect. Holding only
the journal transaction is insufficient. Direct writes are ordered, not a
power-loss atomicity claim. Existing raw commit and the nine qualified metadata/
unlink/rmdir/mkdir families must remain unchanged in the bounded create work.

#### Chosen bounded design

1. All five Rust/C forms enter the existing mount operation mechanism once,
   before arbitrary `Into`, device property/read or runtime hooks. Use private
   byte/parent helpers borrowing that context and preserve both pure volume
   verdicts. In counted Rust dispatch keep `Into` before name validation; do
   not copy mkdir's different priority or edit shared planner callers.
2. Extract a private, nominal **completed create plan** containing the new
   inode and one finished buffer, retaining `NewInodePlan` for its existing
   shared setup. Complete child construction, entry insertion and any parent
   growth locally; converge the two terminal branches before one create-only
   commit helper. A plan is internal trusted staging, not a generic seal or
   ownership proof, and is not exposed through the public ABI.
3. Before effects, check context identity/health and require the staged full
   primary-SB block with valid shape/offset/magic. Preserve planned allocation
   debits, other state/features and bytes outside the SB region; clear only
   `EXT4_VALID_FS` and refresh its CRC. A future pre-marker planner could
   otherwise replay clean state over the marker. This is a **future-boundary
   diagnostic**, not a current public clean-bit regression: current create
   marks before building that buffer. Keep this boundary create-specific.
4. Run the one-time marker and the **whole** raw commit/checkpoint/direct-flush/
   allocation-publication call inside the existing context's failure-latching
   effects. Returned effect errors and legal unwinds retire the mount across
   all qualified families, flush and clean close; pre-effect ordinary refusals
   release healthy entry without any disk writes. Preserve the existing
   retained-context `EffectFailure` protection; do not refactor shared effects.
5. Add a create-specific C adapter retaining that context through requested
   `stat_ino` readback and returning an owned optional `Inode`. C field formatting
   then uses only the snapshot. NULL avoids the extra read. Preserve a possible
   error after successful creation, untouched attr on failed read, generation/
   byte identity and zero/-1 versus inode/zero conventions. Never retry creation,
   fabricate attrs or claim rollback. A normal readback error is distinct from
   an uncertain write; legal readback unwinds still cross the held context.

#### Next executed controls and independent qualification

First prepare **and actually execute** native safety/positive controls through
`scripts/tier.sh`, using the existing Device/Clock/C fixture machinery and
asserting fixture shape. Keep the patch dormant in normal CI until repaired.
Use ext2/ext3/ext4 × 1/4 KiB base profiles plus actual same-table, inline,
full-parent and constructed multi-group/uninit cases. Do not import another
family's fault counts as create measurements.

| Control group | Required observed coverage |
|---|---|
| Entry/exclusion | All five forms refuse a held context before device/runtime calls; counted Rust also precedes caller conversion. Releasing the held context allows creation. |
| Reentry | Legal same-thread/worker callbacks from writability/read/clock/generation/marker/commit/flush cannot reenter create or any of the nine qualified families. Never panic through extern C or fabricate overlapping mutable ABI references. |
| Untouched refusal | Clean and already-dirty missing/stale/non-directory/duplicate/invalid/long-name/bad-checksum/allocation refusals preserve complete device bytes and pending allocation state, make no write/flush calls and leave a healthy mount reusable. |
| Priorities and identity | Volume before Rust conversion/name; **conversion before name**; name before live generation/type; second path verdict; basename length before parent lookup; FFI null/ceiling first. Preserve dot/trailing/separator/root-walk behavior, explicit modes and raw/255-byte names. |
| Positives and accounting | One new regular inode, one link, zero data size/sectors, correct dialect map/flags/generation/times/CRCs, exact per-group/SB/bitmap/itable-unused accounting and unchanged used-directory/parent links. Preserve peer contents/EA, parent storage, shared inode-table splices and distinct back-to-back allocations. Charge actual parent growth/conversion allocations. |
| Uncertainty | Measure a successful call's actual writes/flushes, inject each observed position on direct/journal profiles, and prove every injection ran before aggregate assertions. Include native runtime/Into/publication unwinds and retained-context errors; after uncertainty no qualified family, flush or clean close may write. Some current protections may already pass. |
| C attributes | Readback callbacks observe the held context; committed attrs/identity agree. NULL avoids readback; injected read failure reports error while exactly one created entry remains and attr sentinel stays intact. |
| Future private boundary | Once the real helper exists, foreign context/missing/malformed SB and constructed clean-state/debit/outside-byte controls exercise that actual boundary. Do not add a fake helper to compile diagnostics before migration. |

Save complete red/positive logs and outcome/name multisets; reverse/reapply and
rerun to establish reproducibility, then restore production/test bytes and
retain all **1,386 native successful occurrences / 418 controls**, zero failed/
ignored, with lint passed. Compile/fixture failures are not driver reds. Record
actual red/pass/injection counts, including positives; keep semantic candidates
separate. Both promoted mkdir patches remain historical and must not be reapplied.

After native safety preparation, use **reference-only Linux exclusive regular-
file creation** in the existing CI VM to measure advancing parent/child times,
existing file/directory/symlink and dot/trailing/255/256-byte behavior, mixed
prefix/name errors, controlled zero-umask/root mode/ownership and exact resource
accounting. Linux creates all reference bytes; the driver initially reads only.
Measure modes/credentials/ACL and rooted path differences explicitly rather than
changing API semantics by assumption. Use fresh asserted non-growing parents
for empty-file allocation expectations; growth is separate. Save before/after
images, judged e2fsck, complete debugfs and cold kernel records and audit their
raw bytes before native policy reds and repair.

After executed reds and the bounded repair, qualify actual Rust/C emissions
across 1/4 KiB, legacy/checksummed and direct/journal layouts, with saved input/
dirty/closed images, byte names, empty-file identity, peer/EA/timestamp/map
preservation, per-group accounting/CRCs, journal state, judged e2fsck/debugfs and
cold Linux readback. Preserve existing create/inode/unique-allocation/uninit/
inline/index-growth/journal/crash and complete-suite evidence; qualify actual
parent growth separately rather than assuming every create costs only one inode.
Arbitrary ownership/aliases, mappings/seals, other writers, retained handles,
maintenance/C lifetimes, lifecycle, recovery, ignored fuzz and actual Windows/
SteamOS/card remain open. Stages 4–5 are not yet quantified; casefold writes stay
blocked. No source/test/workflow/cap/floor/filter/pin/lock/sibling change or new
oracle execution is part of this audit.

### Scoped mkdir full-CI review closure — 2026-10-23

Budget repair `7f9617dc3b4c4d2e3663f382d7a644bd681fd62e` passes every job and
ci-ok in [exact full run 39997261948](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39997261948).
Five source/advertised-digest/CRC-verified tier archives retain all **2,227**
whole-suite successful occurrences in both suites, zero failed and one unchanged
ignored slow fuzz case. All **418** controls occur once in five contexts; all
three unit contexts retain 1,386 occurrences and aarch64 images retain 527 passes.
Native whole output is 3,633 lines / 212,215 bytes; guest is 3,800 / 221,518.
Actual wrappers and unchanged 1,060 floors pass. Trailing native **twenty shell
checks and 159 installed-CLI checks now execute and pass**, with all prior shell
names/verdicts and CLI-file counts retained. Standalone CLI also passes 159.

Implementation/test/workflow bytes match independently verified `97356c9`.
The completed source walk, sixteen-layout/112-image/48-creation mkdir audit
and retained-family saved-byte audits above remain applicable; no unchanged
image audit or new native regression is claimed for this docs-only checkpoint.
No further blocker is found within **caller exclusion, healthy layouts and
closed reference handles**. The scoped mkdir source/evidence/full-CI review is
closed. Both mkdir patches remain historical promoted artifacts.

The [closure ledger](extprogress.md#scoped-mkdir-full-ci-review-closure--2026-10-23)
records exact provenance, occurrence multisets, actual floors and trailing checks.
Next is a bounded regular-file `create` Rust/C staging/marker/dispatch design
audit, including shared planner callers and optional C attributes, before new
red execution or migration. This checkpoint chooses that audit rather than
claiming it complete. Keep every qualified family intact. Arbitrary ownership/
corruption, retained handles, mapping/index growth, lifecycle/seal, recovery,
ignored-fuzz and actual Windows/SteamOS/card qualification remain open.
Stages 4–5 remain not yet quantified; **casefold writes stay blocked**.

### Mkdir scoped source/evidence review and whole-byte repair — 2026-10-23

Review at implementation `97356c9` finds no further blocker within healthy
ordinary layouts, caller exclusion and closed reference handles. All five
Rust/C forms share one context and owned plan; both pure verdicts and counted
validation priorities remain. Parent timestamps/link count converge after
inline/growth/same-table edits. The mkdir-only full-SB marker preserves staging
and spans complete commit/checkpoint/direct flush **and allocation publication**.
Optional C readback stays in the context, with post-commit read errors retaining
one creation. Explicit modes/name/generation/FFI and mkdir-local pathname
contracts stay intact. No other creation helper or guarded family is refactored.
All 104 promoted/native controls, 352 measured I/O uncertainties, four public
publication unwinds and independently audited 112-image emission support that
scope; arbitrary ownership, mapping/index growth and lifecycle remain open.

All eight focused jobs now pass. Eight freshly source/digest-verified retained
archives pass metadata/setter/removal/unlink/rmdir/reference/symlink saved-byte
audits at this implementation. Both implementation/docs full runs retain all
2,122 preceding successful occurrences plus 104 native controls/one oracle:
**2,227 passed / zero failed / one unchanged ignored slow fuzz test** in each
whole suite. Unit contexts retain 1,386 occurrences; all 418 controls occur
once in five contexts; architecture images retain 527 passes.

Full runs `39912448414` / `39913996529` fail only because native/guest whole
output exceeds old byte caps (65 after successful test commands). All four
complete logs fail old and pass measured new **220,000 / 230,000** byte caps,
with identical SHA-256 and replay floor 2,227 against 1,060. Line limits,
floors, output and selection stay unchanged. Fresh 94 CI profile/contract checks
and full lint pass. No source/test/workflow/pin/sibling changes.

**Full-CI/scoped closure remains pending:** old wrappers prevent actual whole
floor commands and trailing native twenty shell/159 installed-CLI checks.
Separate CLI success does not replace them. Inspect the new budget-repair
commit's exact full run, retain all occurrences/controls and require actual
floors/trailing checks/ci-ok before closing mkdir review. The
[ledger](extprogress.md#mkdir-full-suite-budget-repair-and-scoped-evidence-review--2026-10-23)
records measurements, provenance, review and limits. Reuse unchanged image
qualification; keep both promoted patches historical. Stages 4–5 remain not yet
quantified; casefold writes, recovery and Windows/SteamOS/card qualification stay
blocked. No host oracle, mount or forbidden implementation source is used.

### Verified public mkdir Linux emission; full retention pending — 2026-10-23

Exact source `97356c9` passes focused `mkdir-emission` job `120278298194` in
run `39912448406`: one executed pass, floor one, zero failed/ignored, complete
log 893 lines / 111,735 bytes. Artifact `12483279278` matches source and
advertised SHA-256 `4f1b9fcd9b3773ffa48f7cd54b1c1ebac8cb2437d07497ece0a7d3087efc4914`.
All 1,329 archive members are read/CRC checked directly from the compressed ZIP.

All sixteen Rust/C × 1/4 KiB × 128/256-byte × direct/journal layouts and 112
input/dirty/closed images pass raw-byte audit. All 48 creations, 544 complete
debugfs reports and 432 cold Linux metadata records agree on identities,
fields/times/mappings and byte names. Per-group bitmaps/free counts/used dirs,
all affected checksums, peer/existing inode/payload/EA preservation, parent
links/times/atime and unchanged storage agree. Final per-layout allocation is
-3 blocks/-3 inodes/+3 used dirs. Dirty and closed state is correct; all 56
journal headers are checkpointed and all 28 checksummed journal SBs pass CRC.
Explicit 02750/01770/0750 API modes survive independent Linux stat. Actual
calls are sixteen mke2fs, 112 judged e2fsck, 544 debugfs and 64 kernel mounts.
This is ordinary healthy emission evidence under caller exclusion, not power-
loss, arbitrary ownership, index growth, lifecycle or Windows/card qualification.

Source/digest/CRC-verified exact CI unit output retains every 1,386 local
successful occurrence and all 418 controls once, zero failed/ignored, within
unchanged limits. All other separate focused jobs and full unit/CLI/MSRV/wasm/
provenance/semver jobs pass. Main focused fixtures and full fixture/runner setup
remain running; complete-suite/aarch64/trailing evidence and ci-ok are pending.
No driver/test/workflow/task/global cap/filter/pin change is included here.

The [verified ledger](extprogress.md#verified-mkdir-linux-emission-and-saved-byte-audit--2026-10-23)
records artifact provenance, raw/independent checks, exact-source retention and
remaining gates. Next inspect remaining actual CI, repair real failures and
complete retained-family and standalone source/evidence review before scoped
mkdir closure. Reuse verified images while source stays unchanged. Stages 4–5
remain not yet quantified; casefold writes and broader qualification stay blocked.

### Guarded public mkdir implementation; independent evidence pending — 2026-10-23

At fetched `ea2b41b` every preceding full CI job/ci-ok passes. Fresh replay
retains all 100 saved outcomes (25 passes / 75 failures), then the reviewed
repair makes all original controls and four new private-boundary controls pass:
**104 passed / zero failed/ignored**, complete log 644 lines / 45,594 bytes.
All 352 observed I/O failures/unwinds and four actual public publication unwinds
now retire nine guarded families/flush/close. All 54 primary positive storage
profiles and additional same-table/uninit/C-readback controls remain executed.

All five Rust/C forms enter one shared context before callbacks/planning. One
owned `MkdirPlan` retains complete inode/data/parent/counter staging. The
mkdir-only commit boundary checks identity/health and full primary-SB shape/
magic, preserves debits/state/sentinels, clears clean state and refreshes CRC
before marker and complete commit/checkpoint/direct-flush/allocation-publication
effects. Both parent-storage branches converge before parent link/time updates,
preserving staged inline/growth/shared-table edits. Optional counted C attribute
readback remains in the same context; a later read error still leaves one
creation and NULL avoids that extra read. This is trusted planning, not generic
ownership/admission or lifecycle/recovery qualification.

Measured parent timestamps and pathname parent-type/length precedence now
pass, preserving the second pure volume verdict. Counted empty/slash/NUL/length
before generic conversion/liveness, explicit modes, generation/FFI priorities,
absolute-path syntax and platform errno mapping remain. Other creation
families' `resolve_new_parent` is unchanged. Four historically named future SB
probes now exercise the actual boundary; four new tests cover foreign/missing/
malformed SBs, retained-context error/unwind retirement and second-verdict
precedence. Both saved mkdir patches are historical promoted artifacts and
must not be reapplied.

Native normal units retain all 1,282 earlier occurrences plus 104 (**1,386
passes**, zero failed/ignored), with all **418 controls once**. An actual
1,634-line / 121,718-byte unit log exceeds old caps; identical complete-log
replay fails old and passes measured **1,740 / 131,000** limits. Selection,
floor/output and other tier limits stay unchanged. Full lint passes.

One sixteen-layout ordinary Rust/C Linux emission matrix and exact floor-one
job are added, retaining input/dirty/closed images and full debugfs/kernel
reports. Planned coverage is 112 images / 48 creations, per-group allocation,
parent/child times and identity, exact byte names, peers/attributes, refusals,
judged e2fsck and cold Linux namespace/statvfs checks. Python syntax and native
compilation pass; **actual CI, saved-byte audit, full retention and standalone
review closure remain pending**, not independent passes. The
[implementation ledger](extprogress.md#guarded-public-mkdir-implementation--2026-10-23)
records the executed red/green distinction, complete effects and next audit.
Stages 4–5 remain not yet quantified, with casefold writes and broader
ownership/index/lifecycle/recovery/Windows/SteamOS/card gates blocked.

### Executed measured native mkdir policy reds — 2026-10-22

At fetched `9937200`, the new dormant
[27-control policy patch](CASEFOLD-MKDIR-POLICY-RED.patch) executes **fourteen
passes / thirteen failures**, each red reaching all six native profiles. Ten
parent timestamp controls cover first/back-to-back creations through all five
Rust/C forms; three mixed pathname controls show regular/missing prefixes
currently lose to final-name length. Across the policy run, 252 creations and
714 refusal/precedence/syntax observations execute. Positive controls retain
raw-byte/255-byte identity, existing/dot/trailing errors, allocation accounting,
C attributes, parent storage/atime and protected peer data/attributes.

The review preserves explicit low-twelve-mode-bit behavior, absolute-path
syntax, counted empty/slash/NUL/length-before-conversion priorities, generation
validation, FFI null/slice checks and symbolic errno mapping. The measured
syscall SGID/empty-path differences do not override those contracts. The next
mixed-prefix correction is local to mkdir; `resolve_new_parent` keeps its
existing earlier-length semantics for other creation families. A discovery
positive-test failure caused by parent growth is corrected with fresh parents,
not counted as a driver defect.

Both final patch orders recreate identical complete source; both actual
combined runs execute the same **100 names/outcomes: 25 passes / 75 failures /
zero ignored**, exit 101, **1,037 lines / 80,845 bytes** each. All original 73
safety outcomes, 352 injected I/O faults, four actual publication unwinds and
original positives remain. Four failures are future SB-ordering diagnostics;
71 exercise current public safety/policy gaps. Full lint passes with both
audits active. Patches are reversed and all production/test/workflow/task/lock
bytes restored; normal units retain every **1,282 name / 314 control**, zero
failed/ignored. The original safety patch is byte-identical; the new policy
patch SHA-256 is `a90024d81eedffe73e09ba3628bf053e27d2a27463f74562406fc2297a573a6a`.
The [executed ledger](extprogress.md#executed-native-mkdir-policy-red-preparation--2026-10-22)
records complete coverage, composition, evidence and limitations.

Prior source `79ed456` now passes all full CI jobs/ci-ok (`39826871657`) and
all seven focused jobs (`39826871672`); docs `9937200` passes `39828986312`.
Five source/digest/CRC-verified log archives retain all 2,121 preceding whole-
suite occurrences plus the mkdir reference (2,122 passes, one unchanged ignored
fuzz test), all 314 controls in five contexts, all 1,282 unit names, 527 aarch64
image passes, twenty native shell checks and 159 CLI checks in both contexts.
Actual job logs confirm executed floors; complete whole logs fit unchanged caps.

Next: inspect this checkpoint's CI, implement the reviewed mkdir guard/owned-
plan and timestamp/path repairs, bind diagnostics to the actual private boundary,
then independently qualify real driver emissions and perform scoped review.
No repair or emission qualification is claimed here. Stages 4–5 remain not yet
quantified; casefold writes, broader ownership/lifecycle/recovery and Windows/
SteamOS/card gates remain blocked.

### Verified independent Linux mkdir policy evidence — 2026-10-22

Exact `79ed456` passes `mkdir-policy-reference` job `119992355646` in focused
run `39826871672`: one pass, zero failed/ignored, 565 lines / 64,658 bytes,
actual floor one passed. Its source/advertised-SHA-verified 1,675,744-byte ZIP
(`2f642a1c548844fad8ed4878613ae58a353ee065df073f798cc54f81c4a54195`)
retains forty images. Read-only audit passes every eight-layout phase, all
216 consistent error observations, 24 creations and 328 complete debugfs
reports against saved bytes, including times, permissions, identities, mappings,
SB/BGD/bitmap/CRC accounting, protected namespace/payload/attributes and clean
state. Actual calls are eight mke2fs, forty kernel mounts, forty judged e2fsck
and 328 debugfs. The final per-layout ledger is -3 blocks/-3 inodes/+3 directories.

Linux creation advances parent mtime/ctime, preserves atime, accepts the tested
raw-byte/255-byte and symlink/trailing cases, and refuses 256-byte components.
Regular/missing intermediate components precede final-name length in the
observed path errors. Root/zero-umask/non-setgid-parent observations store
requested 02750 as 0750, 01770 as 01770 and 0750 as 0750. Keep those measured
syscall facts separate from explicit counted-name/FFI/platform-errno and mode
API contracts; do not silently rewrite them. Symlink traversal atime/covering
CRC is the only allowed observer change in otherwise preserved symlink bytes.
The [verified ledger](extprogress.md#verified-independent-linux-mkdir-policy-evidence--2026-10-22)
records the exact checks, policy table, scope and artifact provenance.

The standalone reference audit is complete for ordinary healthy non-growing
layouts, not driver write emission or guarded mkdir qualification. No production
or test source changes follow the passing run. Native baseline retention and
lint remain passed; source aarch64 passes, while native/guest whole suites
and focused fixtures remain running. Next: inspect those outcomes,
then execute independently grounded native parent-timestamp policy reds and
review mixed-prefix/mode contracts before broader semantic changes. Keep the
73-control safety patch dormant until composition/repetition, and restore
source before implementation. Stages 4–5 remain not yet quantified, with casefold
writes and broader ownership/lifecycle/recovery/Windows/card gates blocked.

### Independent Linux mkdir policy oracle added — 2026-10-22

At fetched `cd22d9c`, preceding exact full CI `39813130020` reports success.
One new reference-only needs-host test and separate floor-one focused job
cover eight 1/4 KiB, 128-byte legacy/256-byte checksummed, direct/journal
layouts. Linux creates all bytes; driver mounts are read-only. Five saved
phases per layout require 216 error observations, 24 creations and 40 images.
Complete JSON/stderr, images and debugfs reports are retained even on failure.

The reference records parent/child times with advancing-clock legacy handling,
actual requested/special mode answers under controlled credentials/umask,
ordinary existing/dot/trailing/long/mixed-path errors, raw-byte and 255-byte
successes, namespace identity, peer payload/attributes, exact per-group
allocation/used-directory debits and clean journal/closed-image state.
Input shapes and unchanged parent storage are asserted; e2fsck is judged.
No special-mode, credential-inheritance or uncertain error repair is inferred
before actual measurements. Existing code and native safety reds remain intact.

Python syntax, Rust compilation, full lint and normal native retention pass:
all 1,282 names and 314 controls, zero failed/ignored. The
[reference ledger](extprogress.md#independent-linux-mkdir-policy-oracle-added--2026-10-22)
records coverage and the next actual-evidence audit. The VM job and saved-byte
inspection are **pending**, not Linux passes. Inspect exact CI and verified
artifacts, repair real measurement failures, then prepare independently grounded
policy reds before the bounded mkdir guard migration. Global floors/caps/
filters, pins/siblings and write gates are unchanged; Stages 4–5 remain not yet
quantified and casefold writes/Windows/SteamOS/card qualification stay blocked.

### Executed native mkdir safety red preparation — 2026-10-22

At fetched `166e43f`, exact full CI `39717854554` reports completed success.
The prescribed native baseline has 1,282 passes and all 314 controls. The new
dormant `CASEFOLD-MKDIR-RED.patch` is actually executed, reversed/reapplied with
byte-identical source and rerun: both final runs retain exactly **73 names,
11 passed / 62 failed / zero ignored**, exit 101, 931 lines / 59,622 bytes.
SHA-256 is `575d683b4e47d1028d426b4497632fb0b963fae2d1d298126f3043875bf88932`.

Fifty-eight failures exercise current public exclusion, untouched-refusal,
conversion/callback/worker/C-readback and complete-effect retirement gaps.
Four constructed pre-marker SB diagnostics preserve debits/other state but
restore clean state after the marker; they describe a future private-boundary
requirement, not a current public clean-bit regression. All **352** measured
write/flush error/unwind injections execute before aggregate assertions, with
352 retirement violations. Four actual public publication unwinds occur after
committed namespace writes; all initially leave the sticky mount latch false.
Every profile reaches its observation before the final aggregate failure.

Fifty-four positive creation profiles cover all five entries on six native
flavor/block-size layouts, Rust/C full-parent growth, inline edit/conversion
and DIR_NLINK unknown count. They retain child/parent fields, next-allocation
identity, counters/used-directory credits, peers/external attributes and reopen.
Two same-table cases assert their shape. One additional constructed 2 KiB/
two-group/uninit profile checks three distinct blocks, two inodes, exact group
debits, bitmap CRC/padding, itable-unused and flag publication. It is not an
ownership-qualified Linux fixture. Four optional C-readback callbacks admit
guarded workers (red); four injected post-commit reads preserve EIO with one
created entry and no repeated allocation (positive compatibility controls).

The [executed ledger](extprogress.md#executed-native-mkdir-safety-red-preparation--2026-10-22)
records the fixture/compile/probe corrections, actual source and diagnostics,
complete repeatability and remaining scope. Full lint passes with the audit
active; after reversal normal native units retain exactly 1,282 names and all
314 controls, zero failed/ignored. Only the dormant patch and review/handoff
remain changed. New private foreign/malformed-SB/retained-context controls are
deferred until the actual mkdir-only boundary exists; the four constructed
ordering controls must then call that real boundary. No fake helper or new
production guard is introduced just to compile future tests.

Next: independent reference-only Linux mkdir parent/child timestamp, name/path/
error and accounting measurement, with saved-byte audit, before measured policy
reds or semantic repair. Preserve the existing qualified families, pins, global
caps/floors/filters and write gates; do not reapply promoted rmdir/unlink patches.
No new Linux outcome, arbitrary corruption/ownership, lifecycle/recovery or
Windows/SteamOS/card qualification follows. Stages 4–5 remain not yet quantified
and public casefold writes remain blocked.

### Public-mkdir staging, marker and dispatch design audit — 2026-10-21

Audited clean fetched `a98fcdc3669b795b3ad8799cad4e3e395d57e4cf` using
Sol 6.1. Its exact full run `39711385558` now passes every job and `ci-ok`.
This is a source/helper/test design audit, not new native execution, Linux
policy measurement, a guard migration or ownership admission. The completed
rmdir review and its verified archives remain applicable; no unchanged image
matrix is rerun for this audit.

#### Existing entry and staging boundaries

| Path | Observed behavior | Migration obligation |
|---|---|---|
| Rust `apply_mkdir` / `apply_mkdir_bytes` | String delegates to bytes. Marker-writing `refuse_write` precedes `resolve_new_parent`; that helper splits/trims the path, rejects an overlong final name before parent resolution, then resolves parent symlinks. Bytes then call public `apply_mkdir_at`, repeating the volume verdict. | Enter one shared nonblocking mount operation before callbacks or reads; retain both pure volume verdicts through private nested dispatch. Do not call a second guarded public method. |
| Rust `apply_mkdir_at` | Volume verdict, `check_new_entry_name`, caller `Into<InodeRef>`, live allocated inode/generation/type, duplicate scan, parent link-room check, then allocation. Invalid/overlong counted names precede conversion and stale-parent checks. Literal dot names have no new rmdir-style special check here. | Preserve these priorities and generation/error propagation. Do not transplant rmdir name policy; measure uncertain semantic cases independently before changing them. |
| C `fs_ext4_mkdir` | Null/path-length checks precede the byte API; returns inode or zero with errno. The C string accepts non-UTF-8 bytes. | Keep pointer/string checks and return convention; delegate to the same guarded filesystem boundary, without an additional ABI lock. |
| C `fs_ext4_mkdir_at` | `ino_call` checks the handle; `name_arg` checks null and the FFI slice ceiling before the filesystem verdict. The filesystem creates the entry, then optional `attr_out` calls `stat_ino` and fills attributes; it may report an error after creation has committed. | Keep the counted-pointer/length priorities and zero/-1 convention. Hold the same context through optional attribute readback; distinguish that post-commit read error from failed creation or rollback. |

The current `refuse_write` can write/flush the primary superblock before any
name, stale-parent, duplicate, checksum, link-limit or allocation refusal. There
is no shared operation entry around mkdir planning. The journal mutex inside
`commit_block_buffer` begins after planning and exists only for journaled
mounts. Those source observations define controls; they are not executed reds.

The planner first selects a directory inode using the existing Orlov hint and
one data block, then stages their bitmaps, per-group/SB debits and used-directory
credit. `buffer_mark_inode_used` also preserves uninit handling, bitmap CRC,
`bg_itable_unused` and descriptor CRC. `build_directory_inode` masks mode to
the low twelve bits, sets directory type, two links, one-block size/sectors,
flavor-specific direct/extent mapping, extra-isize, runtime times/generation and
inode CRC. It inherits only the validated parent casefold policy, not INDEX or
INLINE_DATA. The seed contains `.`/`..` and the conditional directory tail.
Public casefold write refusal remains earlier than all this construction.

Parent insertion preserves several distinct paths:

- An inline parent is edited/expanded in its inode or converted in the same
  buffer. Conversion retains its other entries and strips only inline storage.
- Ordinary block insertion and indexed routing keep existing checksum/error
  behavior. Only `OutOfBounds` requests growth; other errors propagate.
- Growth may split an ordinary htree leaf or drop an ordinary index, then
  append through legacy indirect or extent helpers. Casefold index dropping
  remains explicitly refused; this audit does not qualify casefold growth.
- Further parent allocations use staged bitmaps **and staged uninit flags**;
  deep extent planning also reserves not-yet-staged data/metadata blocks. The
  child's allocation must remain visible to every later parent allocation.
- Both final branches re-read the parent through `buffered_inode_verified`
  before its link increment, preserving inline/growth edits and shared inode-
  table blocks. DIR_NLINK unknown-count and link-limit behavior remain intact.

Both branches currently commit their complete buffer once, then return the
new inode. Despite the old public comment saying “Not journaled”, actual
`commit_block_buffer` uses journal commit/checkpoint when configured, or ordered
direct writes/flushes otherwise, followed by allocation-flag publication.
Publication may allocate/unwind. A commit-only guard or effect ending before
publication is insufficient. The parent-link patch and directory-inode staging
helpers do not themselves stamp parent mtime/ctime; child times already use the
runtime. Parent timestamp semantics need independent advancing-clock evidence
before a correction. This audit does not infer a new Linux result.

#### Proposed bounded migration

1. Acquire the existing shared mount context once for each public Rust/C
   filesystem operation, before writability, reads, conversion and runtime
   callbacks. Context identity/freshness precedes even device property calls.
   Use private byte/path/parent helpers with the two existing pure volume
   verdicts and the counted-name ordering above. Contention is an untouched
   refusal; unrelated writers still require caller exclusion.
2. Extract an owned mkdir result containing the new inode number and one
   completed `BlockBuffer`. Keep all allocator, bitmap, seed, insertion,
   growth, buffered-parent and checksum helpers. Merge the two terminal commit
   branches into one private mkdir-only boundary, after every refusal-capable
   planning step. This internal result is not a generic sealed/admitted plan.
3. Before any effect, require the staged full primary-SB block, validate its
   size/offset/magic, clear only its clean bit and restamp the SB CRC. Preserve
   every planned debit/credit, other state and bytes outside the SB region.
   A plan built before the marker could otherwise overwrite its dirty state.
   Constructed clean-SB controls are future-ordering diagnostics, not a current
   public clean-bit regression: current mkdir marks before constructing it.
4. Run the one-time marker and **complete** raw commit/checkpoint/direct flush/
   allocation publication under the existing failure-latching effects. Preserve
   all eight qualified families and their boundaries. Returned effect errors,
   swallowed inner unwinds with a retained context and outer unwinds must
   retire the mount, including flush/clean close; ordinary pre-effect refusals
   leave it reusable. Do not change raw commit or broaden lifecycle guarantees.
5. For counted C creation with requested attributes, use a mkdir-specific
   guarded adapter that reads an owned `Inode` snapshot after successful commit
   and before releasing the context. C formatting can fill the output from that
   snapshot without filesystem reads. A null attr needs no additional read.
   Preserve the existing possible readback error after durable creation; do not
   retry creation, return fabricated attributes or claim rollback. Preserve C
   pointer validity, external lifetime/aliasing and maintenance obligations.

#### Concrete next controls and independent evidence

Prepare native controls with the existing Device/Clock/C fixture machinery and
assert actual fixture shape. At minimum cover ext2/ext3/ext4 × 1/4 KiB, with
targeted inline, full-parent, same-table and multi-group/uninit fixtures beyond
that six-profile base. Do not infer Linux compatibility from self-built images.

| Control group | Required observation |
|---|---|
| Entry/exclusion | All five string/byte/counted/C forms refuse a held context before device/runtime/Into calls; successful release permits creation. |
| Reentry | Writability/read/runtime-time/generation/marker/commit/flush hooks and a worker callback cannot enter mkdir or the eight already guarded families while mkdir holds the context. Use legal Rust/C lifetimes, not overlapping mutable ABI references. |
| Untouched refusal | Clean and already-dirty invalid/missing/stale/duplicate/long-name/bad-checksum/link-limit/allocation refusals preserve complete device bytes, make no write/flush calls and do not retire a healthy mount. |
| Priority | Volume before Rust name/Into; invalid/long counted name before conversion; live generation/type before duplicate; path basename limits before parent lookup; second verdict before resolved-parent work; existing FFI null/slice checks remain earlier. Test literal dots, slashes, NUL and raw-byte names without assuming rmdir policy. |
| Successful staging | Exact child identity, mode, generation/times, `.`/`..`, CRCs, two links, bitmap/SB/per-group deltas and parent links; peer bytes/attributes preserved; distinct back-to-back allocations; inline/growth edits survive same-table splicing. |
| Buffered allocation | Assert the chosen multi-group/uninit state and actual growth/conversion; child data, parent data and metadata remain distinct, flags/padding/itable-unused/CRCs correct, with no allocation publication on pre-effect refusal. |
| Private boundary | Foreign context and missing/malformed SB refuse before callbacks; constructed clean-SB credit plans preserve all counters/features/outside bytes, stay dirty until healthy close and keep the source boundary private. |
| Uncertain effects | Sweep every actually observed write/flush position on direct and journal paths, plus runtime/Into/publication unwinds and retained-context errors. Verify each injection executes before aggregate assertions; no subsequent guarded family, flush or clean close may write. |
| Counted C attributes | Requested attributes match committed inode/generation under the same context, including callback reentry at readback. Null attr avoids readback; injected post-commit read failure reports error while the new entry remains created, with no second allocation. |

Before implementation, actually execute/save the native red/positive patch,
reapply and rerun with identical source/name occurrences, then restore active
source and retain all 1,282 units/314 controls. Keep unmeasured semantic cases
separate from safety reds. Next add independent reference-only Linux mkdir
observations for parent/child times, dot/trailing/long-name and mixed-error
priorities, controlled mode/ownership expectations and resource accounting.
Do not silently broaden into umask, setgid/default-ACL/credential inheritance
without an explicit API contract and measured evidence.

After executed reds and a bounded repair, independent emitted-image evidence
must cover Rust/C, direct/journal, 1/4 KiB and legacy/checksummed layouts with
asserted input shapes. Audit every saved dirty/closed image: cold namespace and
`..`, child/parent fields, peers, complete per-group bitmap/free/used-directory
deltas, uninit transitions and CRCs, judged e2fsck/debugfs/kernel readback.
Charge actual parent growth/map-node allocations rather than assuming one block
for every case; Linux need not choose the same physical inode/block numbers.
Retain existing inline/index-growth/journal/crash/unique-inode/DIR_NLINK and
kernel readback tests. The existing mkdir crash sweep requires remount success
but adds no existence/accounting postcondition; it does not replace full accounting or
qualify arbitrary power loss. Finish source/saved-byte/full-CI review before
another family. Public casefold writes, ownership aliases, plan seals, ignored
fuzz, lifecycle/recovery and Windows/SteamOS/card gates remain unchanged.

### Scoped rmdir full-CI closure — 2026-10-22

Budget repair `0da4221` and evidence checkpoint `49d3484` now pass all eleven
jobs, including `ci-ok`, in full runs `39702689248` and `39703344673`.
All ten downloaded tier archives match advertised SHA-256 and workflow source.
Exact successful-name multisets retain 2,121 passes in each whole suite, all
1,282 unit names in three contexts, all 314 controls once in five contexts and
527 aarch64 image passes. The same one ignored slow fuzz case remains in both
whole suites and aarch64 images; it stays a broader qualification gate.
Both runs now execute all twenty trailing native shell checks and 159 installed
CLI checks; standalone CLI also passes 159. Actual latest native/guest job logs
confirm normal wrappers and unchanged floors. Complete native 3,526 / 201,673
and guest 3,693 / 210,993 logs fit repaired limits without removing output.
The [closure ledger](extprogress.md#scoped-rmdir-full-ci-review-closure--2026-10-22)
records both exact runs, measurements, artifact digests and local evidence.

Source/test/workflow/helper/pin bytes remain identical to independently tested
`fbc334e`; completed source and new/retained saved-image audits remain applicable.
The standalone rmdir review is **closed under caller exclusion, healthy layouts
and closed reference handles**, with no further scoped blocker found. No new
production/test/configuration repair or broader ownership/lifecycle/recovery
qualification is claimed. Casefold writes and Windows/SteamOS/card use remain
blocked; Stages 4–5 remain not yet quantified.

Next is the bounded public-mkdir staging/marker/dispatch design audit: trace all
Rust/C forms, name/generation/error priorities, inode/data allocation and parent
growth, inheritance/counters/checksums, owned-plan preparation and complete
effect/publication retirement. Select one shared-context private dispatch and
specific native red/positive controls before implementation. That audit and its
new tests have not yet executed. Independent Linux policy measurements precede
semantic corrections; preserve all already qualified boundaries and write gates.

### Rmdir emission/evidence verified; measured wrapper repair — 2026-10-21

At exact `fbc334e`, all focused jobs pass in `39684661329`. The new rmdir
job executes once (one pass/zero failed/ignored), 669 lines / 80,886 bytes.
Actual logs retain 16 mke2fs, 64 kernel mounts, 112 judged e2fsck and 320 debugfs
calls. The 4,335,231-byte compressed archive matches advertised SHA-256
`18d05ba6ab52464581c3971ff65de51e641811e660e89821f738750d8493018e`.
Read-only audit verifies every one of 16 ordinary Rust/C layouts / 112 images /
48 closed-target removals: namespace, parent times/links, complete per-group
bitmap/SB/BGD/used-directory credits, exclusive/shared attributes, peer inode/
payload/storage preservation, freed generation/dtime, dirty/clean state and CRCs
including initialized bitmap checksums. Final ledger per profile is +4 blocks,
+3 inodes, -3 used directories. Independent Linux/debugfs readbacks match.
Intervening Linux readlink may update surviving symlink atime; all other fields
and each dirty/closed pair remain checked, with that observer effect recorded in
[the evidence ledger](extprogress.md#verified-rmdir-linux-emission-and-whole-suite-budget-repair--2026-10-21).

The standalone source walk traces all five entries, private single-context
planning/volume/name/generation ordering, matched/inline/storage/parent staging,
required full-SB normalization and complete error/unwind/publication retirement.
No additional scoped source blocker is found under caller exclusion, healthy
asserted layouts and closed reference handles. This is not arbitrary ownership/
corruption/plan admission or unguarded-writer/lifecycle/recovery qualification.

Verified full archives in `39684661421` retain exactly 2,034 prior passes +87:
**2,121 passed, zero failed, one unchanged ignored** in each whole suite. All
314 controls occur once in five contexts; all units retain 1,282 names and
all 527 aarch64 image tests remain. Native 3,525 / 201,559 and guest 3,695 /
210,972 fail only old byte caps (195,000 / 203,000), actual exit 65. Identical
complete-log replay fails old and passes repaired 205,000 / 213,000 with floors,
line caps, commands, output and filters retained. Native trailing shell/installed
CLI checks did not run after that wrapper failure. Unit limits/pins/siblings and
write gates are unchanged; no production/test-body repair is included.

Every remaining exact-source focused archive is now downloaded and digest/
workflow-SHA verified. Fresh read-only audits pass all 16 setter/eight first-
attribute/84 removal-retained layouts, 32 ordinary setter-policy observations/
16 byte-identical Rust/C images/four duplicate diagnostics, all 16 unlink emission
layouts/80 images, unlink reference eight layouts/24 images/40 errors/eight
255-byte successes, rmdir reference eight layouts/32 images/184 errors/24 removals,
and all five symlink tests with their selected namespace/boundary saved images.
All fifteen main focused logs execute one successful test with no failed/ignored.
The standalone rmdir source/saved-evidence review is complete within the recorded
caller-exclusion, healthy-layout and closed-reference-handle contract; no further
scoped blocker is found. Broader gates remain unchanged.

The inherited API-helper/gh credential returned 401 after initial downloads, but
public REST reads/downloads succeed without Authorization, with normal proxy/CA,
redirect and digest checks. Default Git fetch/push also works. No credential or
environment setting is changed, and reconnect is unnecessary for this public read
workflow; authenticated Actions-write access is not verified by that fallback.

Budget repair `0da4221` is pushed and exact full CI `39702689248` is running.
Unit/CLI/fixtures/MSRV/semver/wasm/provenance jobs report success; whole suites and
aarch64 remain pending. **Scoped rmdir closure still requires** the repair's full
CI, exact occurrence retention and native trailing shell/installed-CLI checks.
Reuse the completed image/source reviews while implementation bytes are identical,
then close the scope before another family. Stages 4–5 remain not yet quantified;
casefold writes and broader Windows/card gates stay blocked. No host oracle or
new power-loss qualification is claimed.

### Guarded public rmdir implementation, native evidence — 2026-10-21

At fetched `482bbef`, the prescribed baseline retains all 1,196 names and its
preceding exact CI now passes. Both unmodified dormant rmdir audits are rerun
before repair (82 names, 15 pass / 67 fail, zero ignored, exit 101). Existing
public boundaries and independent-policy reds are addressed by one shared
mount context, private nested dispatch/two pure volume verdicts, owned deletion
planning and a rmdir-only marker/whole-commit boundary. Context identity precedes
callbacks. Required staged-SB shape/magic is checked and only clean state/CRC is
normalized; planned counters, other state and outside-SB bytes remain intact.
The existing empty/reclamation/attribute/matched-entry/group/parent-link staging
is retained. Measured parent mtime/ctime and ordinary long-name errors are added.
Path parent/dot ordering is corrected while preserving counted validation and
conversion/generation/volume/FFI priorities. Other unguarded writers retain caller
exclusion; no shared ownership readiness or generic sealed-plan admission.

All **86 native controls pass**: the original 82 are promoted, four constructed
credit tests now use the actual private boundary, and four new controls cover
foreign/malformed plans, retained-context returned errors/publication unwind and
second-verdict precedence. The last ordering edge was actually reproduced red
before correction; details and harness corrections are in the
[checkpoint ledger](extprogress.md#guarded-public-rmdir-implementation--2026-10-21).
All 340 original write/error/unwind injections execute with zero retirement
violations; all eight guarded families, flush and clean close retire. The normal
native run retains all 1,196 names and adds exactly 86 (1,282 passes, zero failed/
ignored, all 314 controls once). Full lint passes. Both rmdir patches retain
historical bytes and **must not be reapplied onto promoted source**.

Actual complete unit output (1,530 lines / 111,458 bytes) exceeds 1,460 / 106,500;
the real wrapper and identical-log replay return 65. Same-byte replay passes the
unit-only 1,560 / 115,500 repair with floor 440 and all output retained. Budget
shell and 87 CI-profile controls pass. Whole caps/filters/floors/pins/siblings and
write gates are unchanged. Final actual `chore test:unit` with the repaired
wrapper passes normally (1,529 lines / 111,398 bytes), retaining every occurrence
and all 314 controls with floor 440 passed. The new Rust/C emission test and Python syntax compile/parse locally;
no independent emission result is claimed before actual VM evidence.

The separate focused emission job plans 16 asserted ordinary block-directory
layouts, 48 closed-target removals and 112 saved input/dirty/closed images. It
checks per-group release, exclusive/shared attributes/peer preservation, parent
links/times, refusal bytes and real 255-byte success. Judged e2fsck, cold kernel
readback/statvfs and debugfs remain required. This is not Linux inline/indexed/
grown-empty or open-target qualification. **Next:** inspect exact focused/full
CI, verify archive digests and audit saved bytes plus retained matrices/complete
occurrence retention; repair actual failures, then do standalone rmdir family
review before closure. Stages 4–5 remain not yet quantified and casefold writes
remain blocked; broader ownership/mapping/lifecycle/recovery and Windows/card
gates stay open. No scoped rmdir closure is claimed by this implementation step.

### Native measured rmdir policy reds executed — 2026-10-21

At fetched `e5eab76`, the new dormant `CASEFOLD-RMDIR-POLICY-RED.patch` adds
27 actually executed native controls: **nine pass / 18 fail / zero ignored**,
exit 101, 209 lines / 31,321 bytes. Every red reaches all six native layouts.
Ten timestamp controls expose stale parent mtime/ctime after first and second
removals; five ordinary 256-byte controls expose ENOENT instead of the existing
symbolic ENAMETOOLONG. Three mixed-path controls retain earlier missing/non-dir
answers, but reproduce missing long-name refusals and `regular/.` EINVAL instead
of measured ENOTDIR. Both advancing times are checked in the second-removal tests.
All five forms also retain ordinary refusal metadata, actual 255-byte removals,
C error clearing and read-only-before-long-name priorities. Rust/C counted APIs
retain invalid-name/dot-before-generation checks, stale/non-directory-parent
before ordinary long-name policy, and counted-empty EINVAL. Counted NUL does not
pass through CString. New controls execute 120 valid removals and 564 refusals.

The path-dot discrepancy requires validation of the resolved path parent before
final-dot policy, while preserving counted dot-before-generation/conversion.
Do not change the shared path walker or unrelated families without their own
executed evidence. Parent atime, links, inode/block release, raw peer inode,
data/attributes, refusal metadata and reopen checks remain in the policy audit;
untouched whole-device refusal behavior is independently covered by the original
safety audit. This is native policy evidence, not Linux-emitted-image qualification.

Policy patch SHA-256 is
`2be5a176724603262e9d0a4c2dfc0552ddbb61c3db7c3173fcc39e2a018663d3`
(22,858 bytes / 592 patch lines). Original 55-control patch SHA-256 remains
`4cf8a2ac2e55d1595530c980cb3bf0adfb399cede3e81d68ee3dfe022bcd5317`.
Both application orders recreate identical source; two actual combined runs
repeat **82 names / 15 pass / 67 fail / zero ignored**, exit 101. All original
55 outcomes (six / 49) and all 340 uncertainty injections remain. Four failures
are future staged-SB diagnostics; 63 exercise actual public boundaries/policy.
Complete logs measure 1,151 lines / 101,888 and 101,887 bytes. Full lint passes
with both audits active. Both patches are reversed and source/workflows/pins
restored byte-identically. Restored native units retain the exact same 1,196
passing occurrences/all 228 controls, zero failed/ignored (1,444 lines / 102,845
bytes), within unchanged caps. Full evidence is in
[the checkpoint ledger](extprogress.md#executed-native-rmdir-policy-red-preparation--2026-10-21).
No production repair, global budget/floor/filter or dependency/sibling change.

All preceding focused jobs now pass in `39644593294`. Verified source-`ccef341`
guest evidence in `39644593304` retains 2,033 passes plus the new reference
oracle (2,034 / zero failed / one existing ignored), all 228 controls and the
unchanged output caps (3,605 lines / 202,180 bytes). Standalone/aarch64 units
retain all 1,196 names/228 controls; aarch64 retains all 527 image tests. The
full run now passes every job and ci-ok. Separately verified native whole evidence
also retains 2,033 + one = 2,034 passes, zero failed/one existing ignored,
3,438 lines / 192,859 bytes within unchanged caps. All 228 controls occur once
in five CI contexts, and native trailing checks pass twenty shell/159 CLI checks.
No output-budget repair is needed. The subsequent docs-only run remains pending;
no repeated retained-image audit or rmdir emitted-image qualification is claimed.

**Next bounded task:** inspect this checkpoint's exact CI results, then implement
the reviewed shared-context/whole-plan rmdir repair and measured semantics using both
patches. Add foreign-context/malformed-SB/retained-publication controls only once
the private boundary exists, preserve the native baseline and obtain independent
driver-emission evidence before standalone family closure. Keep the existing
caller-exclusion, healthy-layout and closed-reference-handle scope. Broader
ownership/mapping/lifecycle/recovery and Windows/SteamOS/card gates remain open.
Stages 4–5 remain not yet quantified; casefold writes stay blocked.

### Independent Linux rmdir reference evidence verified — 2026-10-21

Exact reference source `ccef341391c69c2bc49529e43766a070eba93c4b` passes the
`rmdir-policy-reference` job in focused run `39644593294`. The actual complete
log executes the exact test once: one passed, zero failed/ignored, **405 lines /
42,482 bytes**, floor one and unchanged isolated cap passed. Invocation evidence
retains **eight mke2fs runs, 32 kernel mounts, 32 judged non-repairing e2fsck
checks and 192 debugfs reports**. The archive matches GitHub's advertised SHA-256:
`73f4446c3da7365a1ef481c1c139d666387e682ed2449b7d3b8fb8d65a73ad63`.

Read-only saved-byte audit passes all **eight profiles / 32 images**, with
184 error records and **24 actual closed-target directory removals**. The
eight layouts assert 1/4 KiB, legacy block-mapped/checksummed extent-mapped
128/256-byte inodes and direct/journal combinations. They use Linux-created
ordinary block parents/targets; exclusive external attributes have one reference
and shared target/peer attributes actually share a physical block with two.
No driver mutation, native synthetic shape or retained open target handle is
substituted for this evidence.

The observed contract is:

- Successful 255-byte, exclusive and shared-directory removals increase parent
  mtime/ctime, preserve parent atime and decrease the parent link count by one.
  Refusal loops preserve policy-parent/peer metadata; the whole raw policy-parent
  inode matches before and after the combined error/name-boundary phase.
- A real 255-byte directory name succeeds. An ordinary 256-byte final name is
  ENAMETOOLONG. Missing/empty are ENOENT; regular file/symlink and their trailing
  slashes are ENOTDIR; nonempty/dotdot are ENOTEMPTY; dot is EINVAL. Mixed paths
  retain earlier missing/non-directory errors before a later overlong component;
  a valid directory/symlink/dot prefix reaches ENAMETOOLONG. All 23 answers and
  lengths agree across all eight profiles.
- Each closed removal frees one inode and decreases the correct group's used-
  directory count. Boundary removal frees one data block; exclusive removal
  frees its data plus attribute block; shared removal frees its data while the
  peer retains the attribute block at one reference. The final ledger in every
  profile is **four blocks / three inodes reclaimed, used directories minus
  three**, with per-group credits matching actual released block/inode bits.
- Namespace, exact raw times versus kernel/debugfs observations, target inode
  liveness, freed generation/dtime/checksum, peer inode/data/attribute values,
  shared references, clean state and absence of replay-needed flags pass. Raw checks
  include 16 SB, 40 BGD, 220 live-inode, 24 freed-inode, 28 attribute-block and
  60 directory-block checksum validations; these counts include repeated views.

Linux syscall errno numbers are observations, not a C ABI change. In particular,
existing symbolic ENAMETOOLONG/ENOTEMPTY values and counted-name validation
priorities remain; an empty Linux syscall result does not authorize changing the
Rust/C counted-argument contract. No semantic repair is made in this checkpoint.
The dormant 55-control rmdir patch is unchanged and still applies. Local native
units retain all 1,196 names/228 controls; the SHA-256-verified exact-source CI
unit archive retains the same multiset (1,452 lines / 103,329 bytes). Lint passes.

Other focused jobs for unlink emission/policy and setter policy pass; the main
focused fixture job and complete suites remain pending at this checkpoint.
Expected whole-suite retention is 2,033 plus the new reference test, not yet a
completed-result claim. Next: inspect actual remaining outcomes and prepare
advancing-clock native policy reds for parent mtime/ctime and ordinary long-name
behavior, covering all five entries and preserving mixed validation priorities.
Prove composition/reapplication with the existing dormant audit and byte-restore
active source before implementation. This is ordinary Linux reference evidence,
not rmdir driver-emission, inline/indexed/grown-empty, retained-handle, corrupted
map/ownership, recovery or Windows/card qualification. Casefold writes stay
blocked and Stages 4–5 remain not yet quantified.

### Independent Linux rmdir policy reference added — 2026-10-21

At fetched `ce2dc95`, the dormant 55-control rmdir audit stays unchanged.
The new needs-host reference test and guest-only Python observer use Linux
syscalls, the existing pinned harness and e2fsprogs tools; the driver only reads
saved bytes. Eight ordinary layouts cover 1/4 KiB, legacy/checksummed 128/256-byte
inodes and direct/journal operation. Planned evidence is four phases per layout,
23 ordinary/mixed dot/type/path/name refusals, a real 255-byte removal, parent
times/links, exclusive/shared attributes, statvfs and inode/block/used-directory
reclamation, with closed target handles, advancing clocks and peer preservation.
Read-only shape checks, saved phase images/JSON and debugfs/e2fsck reports must
execute in the new exact focused job (floor one). Syntax/lint/native retention
pass; Linux execution is pending and no measured answer is claimed yet.

No public rmdir/guard/semantic change, dormant-patch edit, global budget/floor/
filter, pin/lock, sibling or write-gate change. Inspect actual CI artifacts
before policy reds or repair. This ordinary-layout reference does not qualify
inline/indexed/grown-empty directory emission, retained target handles, arbitrary
corruption/alias/ownership, interruption recovery or Windows/card operation.

### Executed dormant rmdir red/positive controls — 2026-10-21

Prepared at clean fetched `60e25de143f3ad0b273abc5b8424597f07d3ff28`, whose
own full CI `39552501832` now passes. The preceding independently inspected
full run `39547402531` retains 2,033 passes in both whole suites and all 228
controls in five contexts. Fresh local baseline units pass all 1,196 names.

`docs/CASEFOLD-RMDIR-RED.patch` adds only a test module registration and native
`rmdir_calls` module. It is **executed**, reversed, reapplied byte-identically,
executed again, and reversed to restore active source. Both final runs execute
the same **55** names/outcomes: **six passed / 49 failed / zero ignored**, exit
**101**. Each complete log is **962 lines / 71,082 bytes**, within the final
isolated audit ceiling of 1,200 / 100,000. Native task/CI limits, floors, commands
and filters are unchanged. The patch SHA-256 is
`4cf8a2ac2e55d1595530c980cb3bf0adfb399cede3e81d68ee3dfe022bcd5317`.

| Executed failing controls | Count and observed boundary |
|---|---|
| Held shared context | Five public Rust/C forms proceed instead of refusing before callbacks; generic parent conversion also runs while the context is held (six controls). |
| Clean logical refusals | Twenty Rust/C counted-name cases retain expected missing/invalid/stale/file/symlink/nonempty/dot errors but write/flush the marker. Two path controls execute all nine missing/root/empty/relative/trailing-slash/dot cases each; every refusal changes the clean image. |
| Reentry and policy | Seven guarded workers proceed from caller conversion; writability, read and runtime callbacks allow metadata workers; callbacks in all seven guarded families allow rmdir workers. The second path verdict still refuses read-only but retains earlier marker effects (six controls). |
| Rust planning unwinds | Four Rust/C clean/prior-dirty deletion-clock cases plus a generic conversion panic fail untouched sticky retirement. C catches Rust-side planning panic as existing EIO (five controls). |
| Uncertain effects | Six complete sweeps cover Rust/C writes, flush errors and Rust-device flush unwinds. Each Rust/C set reaches all **114 observed writes**, **28 flush errors** and **28 flush unwinds**, across 1/4 KiB direct/journal and clean/prior-dirty profiles. All 340 injections execute before the aggregate assertions; each lacks the required eight-family/flush/clean-close retirement. |
| Proposed staged-SB boundary only | Four constructed 1/4 KiB direct/journal plans preserve synthetic credits, other state, checksums and outside-SB sentinels but restore the clean bit when committed after a marker. No real bitmap/inode reclamation occurs. These are future-order diagnostics, **not current public clean-bit defects**. |

Thus **45** reds exercise current public dispatch/refusal/reentry/unwind/failure
behavior; **four** diagnose the proposed owned-buffer boundary. Preliminary
harness compilation and counted-C-name preparation issues were corrected before
the reported runs and are not credited as driver failures.

The **six passing controls** retain existing volume/FFI and invalid/dot-before-
conversion priorities; all twenty prior-dirty refusal cases preserve errors,
bytes and context; and the positive matrices complete **127 valid removals**:

- **60** ordinary cases: ext2 direct, ext3 journal and checksummed ext4 direct,
  1/4 KiB, all five Rust/C forms and exclusive/shared external attributes.
- **64** checksummed cases: ordinary/inline parent and target combinations,
  1/4 KiB, Rust byte/C parent forms, exclusive/shared attributes and literal/
  DIR_NLINK unknown parent link counts. Native inline construction releases the
  old directory map, adds `system.data`, restamps checksums and persists features;
  it does not merely relabel allocated pointers.
- **Three** Rust string/byte/C path cases retain parent-symlink traversal and
  successful trailing-slash removal, preserving the symlink across reopen.

Positive assertions cover namespace/inode allocation, exact block/inode and
used-directory credits, parent links, freed generation/dtime and checksums,
shared refcounts/peer inode/data/attributes, inline-parent entries, dirty state,
normal clean close and reopen. These remain driver-built native fixtures;
independent Linux statvfs/image/layout qualification is still required. No parent
timestamp or ordinary 256-byte errno correction is inferred or pinned as Linux
policy from these controls.

Full lint passes with the final audit active. After reversal, the native unit
tier retains the exact **1,196**-name baseline and all **228** controls, with no
failed/ignored cases. Active source is byte-restored and the patch applies
cleanly. No guard/semantic repair, workflow, task/budget/floor/filter, pin/lock,
sibling or write-gate change. Future private-boundary foreign-context/malformed-
SB and retained-context publication controls still need the actual rmdir
boundary; they are not claimed executed by this public-entry preparation.

**Next bounded task:** independent reference-only Linux rmdir policy measurement
in the existing CI VM, using distinct advancing timestamps and closed reference
directory handles. Measure successful/refused parent times, 255/256-byte and
mixed dot/type/path errors, reclamation and shared-attribute behavior before
semantic correction. Then execute any new policy reds before a reviewed guarded
repair. Preserve all native/whole-suite baselines and completed unlink evidence.
Casefold writes and broader ownership/mapping/lifecycle/recovery/Windows/card
qualification remain blocked; Stages 4–5 stay not yet quantified.

### Public rmdir staging, marker and dispatch design audit — 2026-10-21

Audited clean fetched `661ba754839d872e86f680e8cf84a59ddda9bdc7` after scoped
unlink closure below. Exact full run `39547402531` passes every job and `ci-ok`;
both whole suites retain 2,033 passes, all three unit contexts retain 1,196,
and all 228 controls occur once in five contexts. This is a source/design audit
only: no rmdir guard, new native red, fault injection, Linux policy measurement
or emitted-image qualification is implemented or executed here.

#### Existing entries and one owned deletion plan

| Entry or phase | Observed behavior to retain |
|---|---|
| Rust string/byte path | `apply_rmdir` delegates to `apply_rmdir_bytes`; marker-writing `refuse_write` precedes `resolve_parent`. The split trims trailing slashes, rejects empty/root/relative paths as currently implemented, follows parent/intermediate symlinks, and looks up the final entry itself. Do not substitute unlink's special trailing-slash branch. |
| Parent-inode form | `apply_rmdir_at` calls `refuse_write`, checks empty/slash/NUL names, refuses `.` with InvalidArgument and `..` with DirectoryNotEmpty, then invokes caller `Into<InodeRef>`, checks the live parent/generation, retains the matched entry and reads the live target. A non-directory target is NotADirectory. |
| Nested public edge | The path entry calls public `apply_rmdir_at`, repeating the write verdict. Two independently acquired nonblocking guards would recurse. Retain the second pure verdict in a private context-taking helper. |
| C forms | `fs_ext4_rmdir` and `fs_ext4_rmdir_at` keep existing null/path-length/counted-pointer validation, `ffi_guard`, success error clearing, byte names and symbolic errno mapping. `ino_call`, `inode_ref` and `name_arg` preserve generation and FFI argument priorities. C memory/handle lifetimes remain the caller's responsibility. |
| Empty target | Inline targets use `refuse_nonempty_inline_dir`; block targets scan every size-covered logical block through the existing map, skip holes, retain role-dependent checksum checks and reject entries other than dot names. This is the existing algorithm, not full corrupt-directory/index admission. |
| Storage and attributes | Extent release collects all data plus external tree nodes, including beyond-size extents, and verifies external-node checksums. Legacy release includes data/indirect blocks and excludes inline payloads via `holds_block_map`. Checked exclusive attributes are freed; shared attributes lose one reference/checksum while the peer stays allocated. |
| Allocation accounting | One `BlockBuffer` stages block/inode bitmaps, BGD free-block/free-inode credits, a separate target-group used-directory decrement, and accumulated primary-SB block/inode credits/checksum. Preserve group-split accounting. |
| Target and parent | Zero the target body while retaining generation and recording dtime/checksum. Remove only stored spelling with expected-inode recheck; a vanished entry maps to the existing Corrupt error. Read the parent through the current buffer before decrementing links, preserving inline edits and same-inode-table-block splices. Keep DIR_NLINK's unknown-count value of one. |
| Durable boundary | There is one terminal `commit_block_buffer`. It includes journal commit/checkpoint or direct flushes, then allocation-view publication. Marker and that complete boundary currently lack the shared family context. |

Source inspected includes `src/fs.rs` public rmdir entries, resolution, inline
emptiness/entry removal, storage release, inode/BGD/SB staging and commit;
`src/fs/guarded_chmod.rs` context/identity/effect/unlink boundary; both C entries
and their argument helpers. Existing read/checksum refusals and logical ordering
must survive. These helpers do not establish complete role/ownership, alias,
mapping-shape, retained-handle or concurrent-lifecycle safety.

#### Reviewed migration contract

1. Acquire the same mount-bound nonblocking context used by the seven guarded
   metadata/unlink families before filesystem/device/runtime callbacks or
   generic parent conversion. Keep contention/reentry an untouched refusal and
   prior failure/poison sticky. Preserve earlier C foreign-memory validation.
2. Replace early marker-writing calls with pure write verdicts. Share one
   private context through path resolution and parent-inode planning; retain
   both verdicts on the ordinary path. Preserve name and dot/dotdot before
   conversion, parent liveness/generation before lookup, final symlink/type and
   trailing-slash behavior. Do not silently impose the creation or repaired
   unlink name-length rule on rmdir.
3. Extract the existing deletion into one private owned-buffer planner: no
   marker, device write, flush, commit or publication until it succeeds. Keep
   all existing empty checks, matched identity, storage/attribute reclamation,
   used-directory accounting, freed-inode state, parent nlink and buffered
   splicing. Logical Err drops staging; read-cache effects are not promised
   invisible. Rust planning unwind conservatively retires the mount.
4. Add a rmdir-specific whole-buffer boundary. Check context identity first;
   validate the staged primary-SB full-block shape/magic and normalize only its
   clean bit/checksum while retaining planned counters, other state and bytes
   outside the SB. Real rmdir plans carry the SB credits even for inline targets.
   Do not overwrite them from a stale mount/disk snapshot. This is a trusted
   planner boundary, not admission of arbitrary buffers. Preserve the qualified
   unlink boundary and raw commit rather than silently generalizing them.
5. Hold the same context through marker-once and complete failure-latching
   commit/checkpoint/flush/publication effects. Returned failures, writes that
   change bytes then fail, Rust unwinds and post-durability publication poisoning
   must retire all guarded families and clean close. No C callback is assumed
   safe to unwind across its ABI. Other unguarded writers still require caller
   exclusion and remain outside this migration.

Moving the marker after planning creates a *future* staged-clean-SB ordering
hazard because rmdir credits the primary SB. The current public code marks
before building that buffer. Constructed clean-SB tests therefore must be labeled
future-boundary diagnostics, not an existing public clean-bit regression.

#### Executable controls and independent evidence required next

Prepare native red/positive controls first using the existing Device/Clock/C
harness. Cover all five Rust/C forms under a held shared context; prompt worker
reentry from writability/read/runtime callbacks; Into ordering/panic; clean and
prior-dirty missing, stale, file/symlink, nonempty, dot/dotdot and invalid-name
refusals; second-verdict writability changes; existing volume and C argument/
errno priorities. Positive cases must perform real empty-directory removal with
ordinary/inline parent/target shapes where the fixture asserts them, literal and
DIR_NLINK parent counts, exclusive/shared attributes and 1/4 KiB direct/journal
layouts. Check parent/target/peer bytes, namespace, data/attributes, inode/block
bitmaps, SB/BGD/statvfs and used-directory counters, checksums and dirty/close/
reopen state. Include full observed write/flush failure sweeps, Rust planning/
effect/publication unwinds, foreign contexts and malformed staged-SB diagnostics.
An unsupported fixture/build/zero-match result is not an executed red.

The source updates parent nlink/checksum but has no explicit parent mtime/ctime
stamp; it stamps final dtime. Ordinary overlong counted names have no explicit
255-byte check here. These are observations, not newly measured Linux answers.
Before semantic correction, obtain independent reference-only Linux results
with distinct advancing clocks and closed reference directory handles: successful
empty removal, refusal timestamps, 255/256-byte names and mixed path/dot/type
priorities. Preserve existing symbolic C errno values rather than copying raw
Linux integers. Execute any resulting policy red before its repair.

After migration, require asserted Linux-created input shapes and saved input,
dirty and normally closed Rust/C images. Independently verify namespace,
parent links/times, complete block/inode/used-directory reclamation, shared-peer
preservation and checksums with judged e2fsck plus cold kernel readback. An empty
directory grown and emptied on Linux retains a distinct multi-block/index shape;
only claim shapes actually asserted and exercised. Existing C errno/liveness,
non-UTF8, parent-symlink, inline, block-mapped/checksummed, DIR_NLINK and journal
tests remain baseline. Existing crash tests' remount/root-link checks are not
new complete recovery qualification.

**Next bounded task:** execute and save a dormant native rmdir red/positive
patch, reapply/rerun it byte-identically, then reverse and verify active source
restoration plus baseline retention. Do not implement a guard or semantic repair
in that preparation. Independent policy/emission and standalone family review
remain later prerequisites. Stages 4–5 are not yet quantified; casefold writes,
ownership/mapping/lifecycle/seal/index-growth/recovery and Windows/card gates stay.

### Standalone public unlink source/evidence review — 2026-10-20

Reviewed at clean fetched `3998daaaf18e74efaec9ac6cfcbc17a5b9951a37`; driver,
test and workflow bytes match independently executed implementation source
`cd15806199a698e3e1a78f6bfdee58a62802f36a`. Runtime policy is current/enforced;
Git/API and actual artifact downloads work without environment changes.

**Source-review finding:** no additional blocker found within the existing
caller-exclusion, closed-reference-handle and healthy-layout contract. This is
not final family closure: exact full-CI retention and remaining focused evidence
are still required until the completed-result addendum below records them.

The walk inspected `src/fs.rs` public entries/planner, retained entry removal,
current buffered inode splicing, extent/indirect storage and external-attribute
release, bitmaps/BGD/SB counters, raw commit/checkpoint/allocation publication,
and failure/close gates; `src/fs/guarded_chmod.rs` context entry/check/drop and
unlink boundary; `src/operation.rs` exclusion/effect invalidation; both C ABI
entries and their argument/error boundary; the 57 native controls and Linux
reference/emission oracles. It establishes:

- All five public Rust/C forms share one nonblocking mount context. Identity,
  prior mount failure and contention precede device/runtime callbacks or generic
  parent conversion. Path dispatch reuses the context, retaining both pure write
  verdicts and its existing parent/trailing-slash logic rather than recursively
  entering a public helper. Logical planning refusals drop only local staging.
- Counted empty/slash/NUL validation precedes generic conversion; parent liveness
  and generation precede the new ordinary-component length verdict. Mixed
  invalid-name/stale-parent/read-only ordering remains covered. C foreign-memory
  argument validation and the existing symbolic errno mapping remain unchanged.
  The deliberately historical ENOENT observation was never an active baseline
  contract; its replacement is supported by independent Linux measurements and
  executed advancing-clock reds, not by silently weakening a passing baseline.
- Entry planning retains stored spelling and expected inode identity. Parent
  timestamps update the current buffered parent after removal, preserving inline
  edits; each inode splice retains other inodes in the same table block. The
  surviving-link branch updates ctime/link count without freeing storage or
  changing atime/mtime. Final deletion retains the existing storage and
  attribute-release helpers, generation/dtime, bitmap/checksum and credit rules.
- The unlink-specific boundary checks owned primary-SB full shape and magic,
  clears only VALID_FS and restamps its checksum, preserving planned credits, other
  state and bytes outside the SB. An SB-free surviving plan stays SB-free. This
  is a trusted planner boundary, not arbitrary buffer admission or a plan seal.
- Marker-once and the complete raw commit/checkpoint/flush/allocation publication
  are inside failure-latching effects while the same context remains held.
  Returned failures and Rust planning/effect/publication unwinds retire later
  writes and clean close, including a caught unwind with a retained context.
  No foreign C callback is assumed safe to unwind across its ABI boundary.

All 57 new controls passed and the 1,139-name local baseline is retained. The
verified exact-source standalone and aarch64 CI unit artifacts each retain all
1,196 occurrences and all 228 controls; aarch64 also retains every one of 527
image-tier occurrences. The measured unit-budget repair passes with intact
commands/filters/floor/output. The source/evidence review rechecks provenance of
the existing SHA-256-matching emission archive and its 16-profile/80-image audit,
including 80 judged e2fsck checks and 48 cold kernel mount/readbacks. Healthy
exclusive/shared attributes, surviving/final links, distinct clocks, legacy/
checksummed 1/4 KiB and direct/journal combinations are represented. Native tests
add refusal/callback/failure/retained-context controls; their constructed SB
credits are still ordering diagnostics, not independent deletion qualification.

Limits remain explicit: the new matrix uses ordinary block parents, small inline
extent roots or direct/single-indirect maps and closed reference file handles.
It does not qualify arbitrary corruption/aliases, open-file deletion ownership,
all timestamp encodings or every mixed trailing-slash/overlong path. The change
preserves existing unsupported-case and logical error behavior outside the
measured correction. Caller-owned C handle/inode/session lifetimes, unguarded
families, complete ownership/mapping/lifecycle/seal/admission, deeper index
writes and interrupted-write recovery remain separate gates. No casefold writes
or Windows/SteamOS/card qualification is enabled by this review.

**Completed focused evidence follow-up:** exact run `39542365213` now passes
every job. Actual SHA-256-verified main saved bytes pass all 15 exact test logs,
16 setter layouts, eight fresh first-attribute cases and 84 retained metadata
profiles. The separate verified policy archive retains all 32 ordinary answers,
16 byte-identical Rust/C emissions and four diagnostic duplicates; the five
symlink tests and saved images also pass. These actual retained-family audits
are distinct from the preceding verified 16-profile/80-image unlink audit.

**Measured native full-suite follow-up:** exact run `39542365226` now retains
all 1,975 preceding successful occurrences plus 57 native controls and the
emitted-image oracle (2,033 passed, zero failed, one unchanged ignored slow fuzz
case). All 228 controls occur once in the whole suite and native units retain
all 1,196 names. The actual verified archived log is 3,433 lines / 192,299 bytes;
the job fails with output-budget exit 65 at 189,000 bytes. Byte-identical saved-log
replay reproduces that failure and passes the measured native-only 195,000-byte
repair. The 6,000-byte adjustment tracks 5,728 bytes of baseline growth and
restores 2,701 bytes of headroom. Line cap, floor, command, filters and all output
remain intact. Guest is still running, so its limits stay unchanged. Native
trailing shell/CLI checks did not run after the failed wrapper. Fresh full CI is
required; no source/test/pin/sibling/write-gate change or family closure is claimed.

**Measured guest full-suite follow-up — 2026-10-21:** original exact-source run
`39542365226` is now complete. Guest retains the same 1,975 preceding successful
occurrences plus 58 additions (2,033 passed, zero failed, one unchanged ignored
fuzz case). Rechecked SHA-256-matching exact-source artifacts retain all 228
controls once in five CI contexts and all 1,196 names in three unit contexts.
The actual guest job and identical saved-log replay fail only the output budget
with exit 65: 3,601 lines / 201,734 bytes exceeds 196,500. The same complete bytes
pass the measured guest-only 203,000-byte repair, retaining the 3,900-line cap,
1,060-test floor, command, filters and all output. The 6,500-byte adjustment tracks
6,093 bytes of growth from the preceding guest baseline, restoring 1,266 bytes
of headroom. Native stays at its separately measured 195,000-byte repair. Lint
and budget-contract checks pass. This does not repair a functional test failure
or establish fresh full-CI success; source/test/workflow/pin/sibling/write gates
remain unchanged. Both-limit repair-source full CI and trailing native checks
are still required before scoped closure.

**Completed scoped closure — 2026-10-21:** repair-source `661ba75` passes every
job and `ci-ok` in exact full run `39547402531`. Actual SHA-256-verified archives
retain all 1,975 preceding successful occurrences plus 58 additions in both
whole suites (2,033 passed, zero failed, one unchanged ignored fuzz case).
All 228 controls occur once in five CI contexts, and three unit contexts retain
all 1,196 names; aarch64 retains all 527 image tests. Native trailing checks pass
all twenty shell checks and 159 CLI checks. Actual native output is 3,433 lines /
192,301 bytes within 3,700 / 195,000; guest is 3,603 / 201,982 within 3,900 /
203,000. All commands/floors/filters/full output are retained. Driver/test/focused
workflow bytes match independently verified `cd15806` evidence. Combined with
the completed source and focused-image reviews above, no further blocker is
found within the documented caller-exclusion, closed-reference-handle and
healthy-layout contract. The scoped unlink review is complete. Broader fuzz,
corruption/alias/ownership/mapping/lifecycle/recovery gates and casefold write
refusal remain. The next operation is the rmdir design audit recorded above.

**Closure requirements, satisfied by the completed-result addendum:** inspect repair-source full-suite logs and
baseline-name multisets (including all 228 controls in five CI contexts), repair
actual failures and complete required trailing native checks. Do not infer green CI from completed
unit/emission jobs while other jobs are running. Do not raise output caps before
an actual measured failure. The existing ignored slow fuzz case remains a broader
qualification gate. The next operation after a completed unlink review is a
bounded public-rmdir staging/marker/dispatch design audit, before new red tests
or another guard migration.

### Guarded public unlink implementation — 2026-10-20

At fetched `223adb3`, the saved 50-test red is reproduced before implementation;
four added write/flush retirement controls also fail at the missing shared latch.
The public unlink family now uses the shared context, private nested dispatch,
verdict-only planning and an unlink-specific owned-buffer boundary. Parent
mtime/ctime and surviving ctime follow measured policy; long-name correction
retains mixed invalid-name/parent/volume priorities and the existing C errno table.
The boundary preserves planned credits/state/outside-SB bytes while normalizing
dirty state/checksum, and latches marker/commit/checkpoint/publication failures.
Raw commit, shared storage helpers, caller-exclusion and write gates are unchanged.

All **57** controls pass, preserving the 27 original contracts and all 22 policy
controls, plus eight failure/priority/boundary checks. Only the documented dormant
ENOENT observation is superseded; historical patches stay unchanged and should
not be reapplied onto promoted source. Constructed SB diagnostics now exercise
the actual new boundary without weakening their assertions. Failure sweeps cover
176 writes / 52 flushes for each Rust/C dispatch set, with all seven families and
clean close retired. Planning/flush unwinds and retained-context publication
panic are checked. Native units retain all 1,139 names and add 57 (1,196 passes,
zero failed/ignored). Full lint passes. The actual 1,444-line / 102,845-byte unit
log fails the old ceiling and passes identical replay at the measured 1,460 /
106,500 repair; no command/filter/floor/output change.

Earlier exact reference-source full/focused runs now both pass; native/guest
retention confirms 1,975 passes and one existing ignored fuzz case. A new focused
16-profile Rust/C unlink-emission job saves Linux-created input and dirty/closed
branch images, requires actual shared-attribute layout and independent Linux/
e2fsck checks. The exact source `cd15806` now passes that job in [run
39542365213](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39542365213):
one passed / zero failed / zero ignored, covering all 16 profiles. Its complete
log is 285 lines / 28,950 bytes; floor and ceiling pass. The advertised SHA-256
matches the actual compressed archive; all **80 images** and relevant reports
are read and audited without local mounts or full extraction. Raw checks cover
all target payload/maps, external attributes and sharing references, generation,
inode/block bits and free counts, checksums, parent/target times and dirty/closed
state. Complete peer inode/data/attributes are retained; final reclamation agrees
with Linux statvfs (two inodes and 37/35/11 blocks for legacy-1K/checksummed-1K/4K).
All 80 judged e2fsck checks and 48 Linux mount/readbacks pass. Verified exact-source
CI units retain all 1,196 passes and 228 controls within the measured ceiling.

Main focused fixture work and full run `39542365226` remain pending, as do complete-
suite retention and the standalone unlink-family review. The new job qualifies
healthy emitted layouts, not caller/raw-handle lifetime, ownership/alias admission,
mapping broadening, crash recovery or the real card. See `extprogress.md` for
commands, artifacts, exact-source results and next prerequisites; Stages 4–5
stay not yet quantified and casefold writes remain blocked. No ownership/lifecycle/index-growth/recovery or card qualification
is claimed from this implementation checkpoint.

### Executed native unlink policy red preparation — 2026-10-20

At fetched `44b30bb`, native baseline and restored units each pass **1,139 / zero
failed / zero ignored**, retaining the exact passing-name multiset and all
**171** controls. The new dormant [CASEFOLD-UNLINK-POLICY-RED.patch](CASEFOLD-UNLINK-POLICY-RED.patch)
adds **22** native controls grounded in the preceding independent Linux oracle:
**two pass / 20 fail / zero ignored**, exit 101. All five Rust/C entries reach
all six native 1/4 KiB ext2/direct, ext3/journal and checksummed ext4/direct
profiles before each red asserts; checksummed-journal native coverage is not
claimed. Distinct clock values separate setup, first unlink and final unlink.

The reds reproduce missing parent mtime/ctime updates after both removals,
missing surviving hard-link ctime updates, and the ENOENT result for 256-byte
names. Existing atime/mtime, identity/generation, link counts, exact payload,
attributes, allocated-sector/free counts and reopen checks pass before timestamp
policy failure; overlong refusals preserve inode bytes and free counts.
Positive controls execute 30 successful 255-byte removals (with C errno clearing)
and 60 ordinary missing/directory refusals preserving complete inode bytes/times.
Native ENAMETOOLONG uses the existing ABI's symbolic value **63**; independent
Linux reports **36**. Do not broaden this repair into a global errno-table change
or replace counted empty/slash/NUL validation with path-component semantics.

Both patches compose in either order and reconstruct identical tested source.
Two actual combined runs retain **50 names: seven passed / 43 failed / zero
ignored**, exit 101; all original 28 outcomes remain identical. First combined
log is **450 lines / 41,930 bytes**, rerun **451 / 41,990**, within an isolated
1,200-line / 100,000-byte audit ceiling. Full lint passes with both audit modules
active. Final reversal restores active source and leaves the original dormant
patch byte-identical; restored units measure **1,387 lines / 96,842 bytes**.
No existing tier cap/floor/filter, active test, production or write gate changes.

One historical expectation must be explicitly superseded when promoting tests:
`oversized_counted_name_retains_existing_block_directory_error` deliberately
pins the old ENOENT observation rather than a Linux contract. Preserve that
historical patch, replace that one dormant behavior-recording test with the new
Linux-grounded long-name expectations, and retain the other **27 original** and
all **22 new** controls without weakening them. This drops no active baseline
test. Guard/failure/constructed-SB assertions remain requirements for repair.

Focused Linux run `39534096165` now passes all three jobs; all 15 main test logs
and their one-test floors are verified from a SHA-256-verified compressed archive.
Main saved images are not re-audited here. Full run `39534096164` still has its
native job pending. Verified guest bytes retain all 1,974 preceding successful
occurrences and add only the unlink oracle (1,975 passed / zero failed / one
unchanged ignored slow fuzz case); unchanged output limits pass. Unit/aarch64
contexts each retain 1,139 passes and all 171 controls. No full-CI success or
unlink-emission qualification is claimed. See `extprogress.md` for commands,
evidence paths, complete measurements and the next bounded guarded/semantic
unlink repair. Stages 4–5 remain not yet quantified and casefold writes blocked.

### Executed reference-only Linux unlink policy — 2026-10-20

Source `30c15f4` passes the new reference job in [focused run
39534096165](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39534096165):
**one passed / zero failed / zero ignored**, covering eight coherent legacy/
checksummed 1/4 KiB direct/journal profiles. The complete 213-line / 17,242-byte
log passes its ceiling and floor. The actual SHA-256-verified archive retains
24 phase images, JSON and independent debugfs parent/target reports; all 24
phases pass judged non-repairing e2fsck checks. All image mutation is in Linux;
driver shape inspection uses a strictly read-only device. All regular-file
handles close before final deletion, and timestamp-sensitive removals cross a
full second even for legacy inodes.

Saved raw bytes and reports confirm parent mtime/ctime updates on both removals,
surviving hard-link ctime updates with unchanged atime/mtime/data/attributes,
2→1 identity/generation preservation, non-final free-count preservation and final
reclamation of the inode and every target data/map/exclusive-attribute block.
Checksums, payload mappings, bitmaps, SB/BGD/statvfs counts and all 24 image states
are audited. All eight 255-byte names succeed; all 40 error observations agree
(256-byte ENAMETOOLONG, missing/empty ENOENT, directory EISDIR, trailing-regular
slash ENOTDIR) without changing parent/target snapshots. The saved native
ordinary-block 256-byte ENOENT control differs from this independent observation.
Separate counted-entry slash/NUL/empty validation priorities remain their own
API contract, not a reason to substitute arbitrary path semantics.

Native baseline/post-change multisets each retain 1,139 passes and all 171
controls; full lint passes. Full CI `39534096164` and the main focused fixture
job remain pending at this checkpoint; the separate policy/symlink job passes
without a fresh saved-byte audit here. Inspect their results, then prepare and actually
execute advancing-clock native policy reds before semantic/guard changes.
The dormant 28-control patch remains unchanged and unenabled. No driver-emitted
unlink qualification or family closure is claimed; production code, existing
caps/floors/filters and casefold write gates remain unchanged. Detailed provenance,
measured results, evidence paths and the next bounded task are in `extprogress.md`.

### Executed dormant unlink red preparation — 2026-10-20

At clean fetched `7ef3929cba92a6f11d610ed6c622953448955368`, the unchanged
native baseline passes 1,139 tests. The reviewed unlink controls are now actually
executed and saved in [CASEFOLD-UNLINK-RED.patch](CASEFOLD-UNLINK-RED.patch).
The patch was reversed, checked, reapplied byte-identically, actually rerun and
reversed again. Both final runs execute the same **28 names: five passed /
23 failed / zero ignored**, exit 101. Each complete log measures **281 lines /
20,608 bytes**, inside an isolated 900-line / 70,000-byte audit ceiling. No
existing tier cap, floor or filter changes. Active production/test source is
byte-restored; the patch is dormant, not a guard implementation. After final
restoration, all 1,139 unit occurrences and 171 existing controls pass with no
additions/losses or failed/ignored tests (1,387 lines / 96,842 bytes).

The 23 failures reach the intended assertions:

- **Five public entries** (Rust string/byte/parent-inode and C path/parent-inode)
  bypass a held shared context and reach device effects instead of the proposed
  prompt pre-callback refusal.
- **Ten clean-mount refusals** retain their expected logical errors but each
  writes and flushes the dirty marker once: Rust/C missing paths, invalid counted
  names, stale parent generation, directory targets and trailing slashes on a
  regular file. The desired pre-effect behavior is red, not their errno checks.
- **Two callback controls** observe missing worker exclusion in both directions:
  generic parent conversion allows all six existing metadata workers to avoid
  Unsupported; callbacks in each of those families allow a non-final unlink
  worker to avoid it. Every family actually reaches its callback. The worker
  target has another hard link, avoiding recursive deletion-time callbacks.
- **One second-verdict control** actually revokes device writability at the
  path-to-parent edge's second query. ReadOnly propagates after exactly two
  queries, but one marker write/flush has already occurred. Preserve that pure
  policy check while removing the early effects.
- **One caught planning unwind** triggers a Rust Runtime panic at final-link
  deletion-time planning. C reports EIO, but `operation_failed` stays false.
  No foreign C callback unwinds; subsequent retirement/close assertions are
  requirements for the later repair.
- **Four constructed SB-order controls**, at 1/4 KiB with direct/journal commit,
  preserve synthetic free-block/inode credits, other state, applicable checksum
  and outside-SB sentinels, but marker-plus-raw-commit restores VALID_FS. The
  context records two effects and the marker is owed. These remain diagnostics
  for the future unlink boundary, **not valid deletions or a current public
  clean-bit regression**; current public unlink marks before planning.

Five passing controls retain useful baseline behavior:

1. Real surviving/final hard-link deletion in sixteen size/journal/interface/
   shared-attribute combinations, **32 successful unlink calls**. Native checks
   cover namespace/inode liveness, target and peer data, inode/block bitmap bits,
   SB/BGD free counts and unchanged used-directory count, link counts, shared
   references/checksums/values, peer raw inode bytes, freed generation/dtime,
   dirty/closed state and reopen. C success clears an explicitly seeded errno.
2. Ten prior-dirty refusal combinations retain their logical errors, complete
   bytes, marker state and control status without another write/flush.
3. Read-only, replay-pending, operation-failed, direct-failed and casefold policy
   refusals preserve their priorities; C null/oversized counted-buffer checks
   remain before filesystem callbacks.
4. Invalid counted names and read-only refusal precede generic parent conversion.
5. A 256-byte counted name in the current ordinary block directory returns native
   NotFound/C ENOENT. This pins current behavior only; it is **not an independent
   Linux errno verdict** and does not cover every directory layout.

Test authoring initially hit a wrong errno constant name, a managed-flag input
and the unsupported legacy pwrite setup. Those compilation/setup attempts are
excluded from the final evidence. The corrected fixtures preserve EXTENTS while
changing NODUMP and use the supported content-replacement path for legacy data;
all sixteen final positive combinations execute. Full lint passes with the
final audit source active, including format, clippy, provenance and shared/CI
contracts. No test assertion or production behavior was weakened to obtain that
result. Temporary logs, source hashes and exact result maps are retained in
ignored `tmp/casefold-unlink-red`; the committed patch recreates the controls.

To reproduce after the handoff's pinned setup, apply the patch, then run:

```sh
scripts/tier.sh unlink:audit-red unlink-audit-red 900 70000 -- scripts/test.sh --locked --lib fs::guarded_chmod::tests::unlink_calls:: -- --show-output
```

Expected outcome is the executed 5/23 result, not a green suite. Reverse the patch
after inspection and check active-source restoration. A missing prerequisite or
zero matched tests is not a substitute. The native positive images are self-
consistency evidence, not new Linux compatibility, ownership or recovery proof.

**Next bounded task:** obtain independent reference-only Linux unlink policy
evidence for parent/surviving-inode timestamps and counted-name errors before
semantic changes or equivalence claims. Use advancing/distinct timestamps,
coherent legacy/checksummed 1/4 KiB direct/journal inputs, closed reference file
handles and saved reports/images. Do not emit through or modify the driver in
that reference checkpoint. Then implement the reviewed shared-context and
unlink-specific owned-buffer boundary against the saved reds, retaining the
positive controls and qualifying actual emissions independently. Any demonstrated
timestamp/name mismatch needs its own red-to-green repair; current native ENOENT
is not permission to preserve it as Linux behavior.

All caller-exclusion/raw-handle/C-lifetime, mixed zero-extra, ignored-fuzz,
insertion-alias, ownership/mapping/lifecycle/seal/admission/index-growth/recovery
gates remain. No host oracle/mount, local VM, forbidden implementation source or
Windows/SteamOS/card test was used. Casefold writes remain blocked; Stages 4–5
are not yet quantified.

### Public unlink staging, marker and dispatch design audit — 2026-10-20

Audited clean fetched `c20dd1b5bbd0800fa97a955e4a38a5991e8771cd`. Driver,
tests and tasks are unchanged from the verified green `2c6ece6` full run
39505996964: both whole suites retain 1,974 passes, the three unit contexts
retain 1,139, and all 171 controls execute in five contexts. The documentation
HEAD's own full CI is still pending at audit start. This checkpoint changes
review/progress records only. **No new red control, fault injection, Linux
reference measurement or emitted-image test was executed here.** Findings below
are source observations and a proposed migration contract, not a completed fix.

#### Concrete entry points and effects

| Entry or phase | Observed current behavior and boundary to retain |
|---|---|
| Rust string/byte path | `apply_unlink` delegates to `apply_unlink_bytes`, which calls marker-writing `refuse_write` before splitting or resolving the parent. It follows intermediate/parent symlinks but removes the final entry itself. |
| Trailing slash | After parent lookup, a live target directory returns IsADirectory and any other live target returns NotADirectory; lookup failures propagate. This branch returns before the parent-inode unlink call. |
| Parent-inode form | `apply_unlink_at` calls `refuse_write`, validates the counted entry name, then runs the caller's `Into<InodeRef>` conversion and `live_dir`. Preserve generation/bitmap/liveness/type checks and that conversion order. |
| Nested public edge | A normal path call invokes public `apply_unlink_at`, so it performs a second write verdict. Acquiring the nonblocking gate independently at both public entries would reject a valid operation recursively. |
| C path | `fs_ext4_unlink` keeps its null and bounded C-path validation, byte paths, error clearing and `ffi_guard`; it delegates to the Rust byte entry without preliminary filesystem lookup. |
| C parent-inode | `fs_ext4_unlink_at` uses `ino_call`, `inode_ref` and `name_arg`; foreign-memory null/length checks precede filesystem dispatch. Keep these priorities and caller-owned handle lifetimes. |
| CLI | Its failed newly-created-file content-write cleanup also calls `apply_unlink`. This is a separate best-effort operation, not an existing held operation context or a reason to extend this migration into content writes. |
| Entry selection | `find_dir_entry` retains stored spelling and inode identity; inline/casefold reads use shared resolution, ordinary block scans retain checksum checks. `buffer_remove_matched_dir_entry` rechecks that saved spelling still names the expected inode in the staged view. |
| Durable boundary | Two terminal `commit_block_buffer` calls cover surviving-link and final-link plans. Marker I/O, both commits, checkpoint/flush and allocation-view publication currently lack the shared family context. |

`refuse_write` is not a pure check: its `mark_not_clean_once` can write and flush
the primary SB before a missing path, invalid name, stale parent, directory target
or later planning refusal. This ordering is directly visible in source; exact
callback counts and error/byte effects still need executable red controls.
`write_refusal` already carries operation/direct-commit failure, device writability,
replay, unsupported RO_COMPAT and write-breaking incompat refusals. Retain those
verdicts and the casefold volume gate before lookup.

#### One owned plan, with two meaningful branches

Both current branches already stage one `BlockBuffer`; their helpers need no
new inner commit. Parent/target inodes can share an inode-table block, so preserve
`buffer_write_inode`'s splicing into the current buffer rather than replacing
that whole block from a stale snapshot.

| Branch | Changes that must stay in the same owned plan |
|---|---|
| Another hard link survives | Remove only the matched parent entry and decrement target links, restamping the target inode checksum. Keep inode allocation, data, external attributes/refcount, sectors and free counters intact. |
| Final link | Remove the matched entry; free target storage and the inode slot; update inode/block bitmaps, BGD and SB counters/checksums; release the external attribute block or decrement its shared reference; zero the target body while retaining generation and recording deletion time; restamp the freed inode checksum. |
| Inline parent | Read and edit its current buffered inode/inline entries; retain expected-inode recheck and directory checksum behavior. A target inode splice must retain the parent's distinct bytes even in the same table block. |
| Extent-mapped target | `extent_tree_runs` includes data runs **and external index/leaf blocks**, including preallocation beyond size; it verifies external-node checksums. Do not replace this with a size-limited data-only walk. |
| Indirect/inline/special target | `holds_block_map` excludes inline data, fast-symlink bytes and device numbers from pointer interpretation, subtracting external-xattr sectors. Real block maps free data and indirect blocks. Preserve that distinction. |
| External attributes | A non-final hard-link removal does not release inode-owned attributes. At final deletion, an exclusive block is freed and credited; a shared block is restamped at a lower reference count, with peer attributes and allocation retained. |

These existing collectors are not the disconnected complete owner/admission
proof. Range, role, allocation and cross-owner alias qualification are not added
by guarding their current use. Keep their current refusals; do not silently claim
arbitrary-corruption safety or broaden accepted layouts.

#### Reviewed migration contract

1. Acquire the same mount-bound, nonblocking context as the six qualified
   metadata families before filesystem/device/runtime callbacks, including
   writability checks and generic inode conversion. Contention/reentry must
   refuse promptly; failure/poison stays sticky. C pointer validation stays at
   its current boundary and gains no handle-lifetime guarantee.
2. Replace early marker-writing refusals with verdict-only checks. Preserve
   volume-before-path/name ordering, parent resolution and trailing-slash errors,
   name-before-Into ordering, parent generation/liveness and target-type checks.
   The path-to-parent-inode edge becomes a private context-taking helper, not
   another public entry. Retain its second pure write verdict rather than
   silently removing an existing post-lookup policy check; cover it with a
   device-writability-change control. The trailing-slash branch keeps its
   existing error path without entering the ordinary unlink planner.
3. Prepare one complete owned buffer without marker, raw commit, device write,
   flush or allocation-view publication. Share this planner between path and
   parent-inode dispatch. Retain matched spelling/expected inode and all current
   read/checksum errors. An ordinary planning Err drops local staging; cache
   reads are not promised invisible. A Rust planning unwind follows the existing
   conservative mount-retirement policy.
4. Add a private **unlink-specific** whole-buffer boundary. Check context
   identity before callbacks and validate a staged primary-SB block's full shape
   and magic before effects. If present, clear only VALID_FS and restamp its
   checksum, preserving planned free-count credits, features, other state and
   all bytes outside the SB. Do not replace it with current disk or mount SB
   bytes. A surviving-link plan with no staged SB must remain SB-free.
5. Keep marker-once and complete commit/checkpoint/flush/allocation publication
   under the same context's failure-latching effects. Retain the context across
   the whole public operation. Leave raw commit, other family boundaries and
   shared deletion/allocation helpers unchanged; avoid a global raw-commit fix.
   Test marker and all observed commit failures, effects that change bytes then
   fail, caught Rust unwinds and post-durability publication poisoning. Later
   guarded calls and clean close must refuse after uncertain effects.
6. Preserve existing error priorities explicitly. `check_entry_name` currently
   rejects empty/slash/NUL names but does not impose the creation helper's
   255-byte limit; C's earlier `name_arg` ceiling is FFI_PATH_MAX. Do not silently
   substitute `check_new_entry_name` or assume every overlong unlink name has
   one established errno. Pin mixed-invalid-input cases first; independently
   measure any intended Linux semantic correction before changing it.

The intended pre-effect change also means an ordinary planning refusal no longer
attempts marker I/O; a hypothetical marker-I/O failure is not preserved as an
earlier error on that refused call. Preserve logical/FFI priorities without
retaining those early physical side effects.

Moving final-link planning before the marker creates the same stale-clean-SB
hazard addressed separately for xattrs: the plan may contain a clean SB with
valid free-count credits, then raw commit could overwrite the dirty marker.
That is a **future-boundary hazard**, not a reproduced current-public dirty-bit
regression: current unlink marks first. The private boundary must preserve the
complete real plan while normalizing only state/checksum.

#### Concrete next red preparation and positive controls

The following are **unexecuted control requirements**. Use the existing native
`Device`/`Clock` and C `with_test_mount` harness; ensure fixtures reach each
intended assertion and save full logs/exact test counts through the tier wrapper.
Do not count a missing fixture, build failure or zero matched tests as a red.

| Control | Required observation before implementation |
|---|---|
| Held context through Rust string/byte/parent-inode and both C entries | Unsupported/established C errno before device properties, reads, writes, flush or runtime callbacks; snapshot bytes and control status. |
| Callback worker and generic Into conversion | Each entry owns the shared context before callbacks; same-mount work promptly refuses in both directions with all six existing families. Invalid counted names precede conversion as before. |
| Missing/invalid/stale/directory/trailing-slash refusals | Retain the exact current error, but no marker write/flush, disk change, allocation publication or epoch advance on a fresh clean mount; also preserve prior-dirty bytes. |
| Volume and FFI priorities | Read-only/replay/feature/failure gates precede filesystem lookup; C null/length checks retain their earlier boundary. Pin overlong names, invalid parent generation and the path edge's second verdict without inventing a new errno. |
| Planning callback unwind caught by C | Trigger a Rust Runtime panic on the final-link deletion-time path; C reports EIO, then the same mount's guarded calls/clean close refuse. Do not unwind a foreign C callback. |
| Whole-plan SB normalization | Four 1/4 KiB direct/journal constructed clean-SB plans with explicit counter credits: old marker-plus-raw-commit order must expose the clean-bit hazard; credits, features, checksum and outside-SB sentinels must survive. Label these constructed diagnostics, not valid deletions; later adapt only them to the private boundary. |
| Real positive deletion controls | Separate surviving-link and final-link cases on fresh images, with actual inode/block bitmap/counter changes, same-table parent/target bytes, shared/exclusive attributes, untouched peers, dirty/close/reopen state and Rust/C parity. These remain self-consistency evidence until independently qualified. |

Save an actually executed, reapplied-and-rerun dormant red patch in the existing
docs area, then reverse it and verify active source is byte-restored. That is the
next bounded checkpoint; do not implement unlink or migrate other families in
that preparation. Promote real controls without weaker assertions only after
the reviewed boundary is ready. Preserve the 1,139-unit/1,974-whole occurrence
baselines and all 171 existing controls. Measure any actual new log failure
before changing an existing cap; no cap/floor/filter changes in this audit.

#### Independent Linux and semantic qualification still required

Existing `capi_unlink`, `capi_unlink_errno`, `capi_ino_api`, `capi_link`,
`extent_block_leaks`, `block_mapped_writes_on_csum`, `shared_xattr_block`,
`kernel_inline_writes`, `inline_dir_writes_oracle`, `htree_dir_writes_oracle`,
`non_utf8_names_oracle` and `symlink_path_kernel` cover useful existing behavior.
Retain them and all current metadata oracles. Their earlier green results do
not execute the new exclusion/failure/marker contract or qualify a migrated
family. The existing unlink crash sweep suppresses writes and checks remount;
its title/comment does not establish complete namespace/allocation equivalence
or a new power-loss recovery guarantee. Do not weaken it or treat it as a
replacement for independent recovery qualification.

Before family closure, add explicit independently built 1/4 KiB,
legacy128/checksummed256, direct/journal, Rust/C path and counted-name coverage,
including surviving/final hard links, inline/block parents, data/extent-node/
indirect storage, zero-size preallocation, symlinks and shared/exclusive external
attributes. Prove each selected shape actually exists in the input; do not
silently substitute a smaller layout. Save before/dirty/closed images, parent
namespace and inode identity, target/peer data and attributes, link/generation/
sector/deletion-time reports, actual allocation bits/refcounts/counter deltas
and applicable checksums. Require judged non-repairing e2fsck and cold Linux
readback, not only a successful tool exit or this driver's reopen.
Close reference file handles before recording final-deletion/free observations;
open-file orphan/lifetime semantics remain a separate qualification obligation.

**Timestamp policy is an open semantic prerequisite.** The current surviving-link
branch changes links/checksum without explicitly stamping target ctime; parent
entry removal does not explicitly stamp parent mtime/ctime. The final-link branch
records dtime after zeroing the inode. These are source observations, not newly
measured Linux answers. Existing namespace/readback tests do not establish that
policy. Independently record Linux surviving-link target/parent timestamp changes
and final-deletion observations with distinct advancing times before asserting
equivalence or silently preserving/fixing them. Promote any demonstrated mismatch
as a separate actual red-to-green semantic repair; fixed clocks cannot prove it.

Keep caller exclusion for unguarded operations/raw handles/maintenance/C lifetimes
and all mixed-layout, ignored-fuzz, insertion-alias, ownership/mapping/lifecycle/
seal/admission/index-growth/recovery gates. Casefold writes stay blocked. No local
VM, host oracle/mount, forbidden implementation source or Windows/SteamOS/card
test was used. Stages 4–5 remain not yet quantified.

### Public setter family review closure — 2026-10-20

**Decision: the standalone public setxattr migration, its same-value repair and
refusal-order repair pass implementation/evidence review within the existing
caller-exclusion and healthy-layout scope.** This does not enable casefold
writes, establish ownership/admission readiness or complete Stage 5.

Budget-repair source `2c6ece6f7789f5ae138d2773147c55b05810936c` passes every job
and `ci-ok` in [full run 39505996964](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39505996964).
All four actual tier archives match GitHub's advertised SHA-256. Complete native
and guest successful-name Counters each retain all 1,974 preceding occurrences,
with zero failed and the same one ignored `deep_walk_byte_flip_fuzz_never_panics`.
All 171 controls execute once in five contexts; each unit context retains all
1,139 occurrences. Actual native output is 3,377 lines / 186,486 bytes within
3,700 / 189,000; guest is 3,541 / 195,440 within unchanged 3,900 / 196,500.
Native trailing checks now actually execute: all twenty shell checks pass
(40 lines / 2,213 bytes), including scratch cleanup, and all 159 CLI checks pass
(29 / 1,097). The missing-fixture local attempt below remains a failed local
attempt; the successful CI harness supplies its required execution evidence.

The driver, tests and focused workflow are unchanged from `4079f36`; the diff
to `2c6ece6` contains only the measured native byte-cap change and review records.
Thus [focused run 39458800523](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39458800523)
remains evidence for the exact unchanged implementation, not a new focused run
at the budget commit. All fifteen named main-job oracles, sixteen setter layouts,
eight first-attribute profiles and 84 retained metadata profiles were audited.
Separate evidence covers 32 ordinary same-value observations, sixteen Rust/C
emission images byte-identical to Linux input and all five symlink tests. Four
duplicate diagnostics remain outside healthy-image qualification.

The final source check found no further blocker in this scope. Public Rust/C
dispatch enters one shared mount context before filesystem callbacks/planning;
volume refusal remains pre-effect. Inline preflight restores name/EA-inode
priority without accepting a match before complete both-location validation.
Unchanged ordinary values avoid clock, marker, writes, flush and publication;
initialization, feature publication, duplicate cleanup and real changes keep
their mutation paths. One owned buffer retains the complete allocation/refcount/
inode/superblock plan; external helpers stage only. The setter boundary checks
context and staged SB shape/magic, clears only clean state and recomputes its
checksum, preserving counters/features and unrelated bytes. Marker plus complete
commit/checkpoint/flush/allocation publication remain guarded. Uncertain effects,
unwinds and poisoned publication retire subsequent guarded mutations and clean
close. The promoted fault, callback, C-error and priority controls are retained.

Caller coordination is still required for other mutators, readers, raw handles,
maintenance and C reference lifetimes. There is no sealed plan, complete owner
proof, Ready state, general malformed-layout admission, concurrent-device safety
or whole-operation crash atomicity. Mixed zero-extra, the ignored slow fuzz case,
three insertion aliases, complete mapping, lifecycle/seal/admission, index
collision/growth and recovery gates remain open. No host oracle/mount, forbidden
implementation source, real card or Windows/SteamOS qualification was used.

**Next bounded task:** audit existing public unlink path/byte/parent-inode and C
dispatch from entry through matched-entry planning and complete durable effects.
Specify a shared-context, marker/whole-buffer and error-priority contract before
implementing that migration; identify concrete red controls and the independent
Linux evidence needed. This advances the namespace-safety prerequisite for
Stages 4–5, not indexed insertion or public casefold-write admission.

### Complete retention and measured native-budget repair — 2026-10-20

Exact `4079f36` full run
[39458801198](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39458801198)
has completed. SHA-256-verified native/guest archives each retain every one of
the preceding 1,972 successful occurrences and add only the two refusal-order
controls: **1,974 passed, zero failed, one unchanged ignored slow fuzz test**.
All 171 controls execute once in five contexts, with 1,139 passes in each of
the three unit contexts. Focused Linux and actual saved-image evidence at this
unchanged implementation source already pass as recorded below.

The native job fails solely at the whole-suite byte budget: the actual log is
3,374 lines / 186,123 bytes against 3,700 / 186,000. Replaying that complete log
through the actual tier wrapper reproduces exit 65. The native-only measured
repair raises the byte cap to 189,000; the same log passes byte-identically and
the unchanged 1,060 floor counts all 1,974 tests. Its 2,523-byte growth since
the earlier 183,600-byte source justifies restoring a similar output cushion.
Guest output is 3,543 / 195,557 within unchanged limits. No code, test, filter,
floor or line cap changes.

The replay is not fresh Linux execution. Native script/CLI checks following
the failed tier did not run in that job, so successful repair-source full CI
and its exact artifacts still gate standalone setter-family closure. Scoped
implementation review found no further blocker under caller exclusion; all
broader ownership, lifecycle, layout and write-qualification limits remain.
Casefold writes stay blocked.

### Current-source focused metadata follow-up — 2026-10-20

Exact refusal-order implementation `4079f36` now passes both jobs in focused
[run 39458800523](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39458800523).
The downloaded main archive matches GitHub's SHA-256 and passes ZIP integrity.
All fifteen named main-job oracles execute once with zero failures/ignored and
complete output within unchanged budgets. Read-only saved-image/report audits
pass all sixteen setter layouts, eight first-attribute profiles and 84 retained
metadata profiles: 192 setter calls (including no-effect calls), 64 preserved
peer records, 280 emitted Linux records and 304 debugfs records. The retained
removal audit also passes 112 calls and 144 Linux/debugfs records. Separate
policy/symlink evidence at this same source was already verified.

This completes the current-source focused evidence prerequisite. Exact full run
39458801198 still executes both complete suites, so complete retention, the
five-context 171-control check and standalone setter-family closure remain
pending. The scoped source-review conclusion and caller-exclusion assumptions
below remain unchanged. No implementation or output budget changes here;
healthy-image evidence does not qualify ownership, faults or recovery, and
casefold writes stay blocked.

### Complete baseline and current-source review follow-up — 2026-10-20

Exact `25a039e` full run
[39452506096](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39452506096)
now passes. SHA-256-verified archives show both whole suites retain all 1,951
preceding successful occurrences and add exactly 21: **1,972 passed, zero failed,
one unchanged ignored slow fuzz test**. All 169 controls execute once in each of
five contexts. Native output is 3,372 lines / 185,898 bytes and guest output is
3,540 / 195,346, within the existing caps. Native headroom is only 102 bytes;
measure the current source's actual output before any budget adjustment.

For refusal-order repair `4079f36`, verified standalone and aarch64 unit archives
each retain all 1,137 previous occurrences and add exactly two (1,139 passed,
171 controls). Its focused policy/symlink job passes: all five symlink tests,
32 ordinary unchanged-ctime observations and sixteen byte-identical Rust/C
policy-emission images were audited. Four duplicate diagnostics remain separate
from healthy-image qualification. The main focused metadata job and exact full
run 39458801198 remain in progress; 1,974 whole-suite passes are expected, not
credited as executed. Prior image audits are not relabeled as current results.

The completed scoped source walk found no further implementation blocker:
Rust/C dispatch preserves pre-callback context/refusal checks; inline preflight
preserves refusal priority without bypassing complete both-location validation;
no-effect success avoids clock and device effects; real changes retain owned
whole-plan staging, marker normalization, guarded durable effects and failure
retirement. This conclusion assumes the existing caller-exclusion contract.
Raw handles, unguarded operations, maintenance and C lifetimes still require
caller coordination. Ownership/seal/admission, mixed zero-extra, insertion-alias,
ignored-fuzz, mapping, index-growth and recovery gates remain open.

Standalone setter-family closure still requires current-source main focused
artifacts and full-suite retention, including all 171 controls in five contexts.
This checkpoint changes review/progress records only; casefold writes stay blocked.

### Setter refusal-order follow-up — 2026-10-20

Exact source `25a039e` passes focused Linux run 39452506142. Its verified actual
artifacts cover all five symlink tests, sixteen setter layouts, eight fresh
first-attribute profiles, sixteen byte-identical policy-emission images and all
84 retained metadata profiles. Saved symlink bytes independently confirm the
1/4 KiB limits, matching Linux/Rust/C namespace trees and the relative casefold
chain. Full run 39452506096 remains in progress; complete-suite retention is
not inferred from those results.

The source walk found a refusal-order regression in the no-effect candidate:
external validation preceded inline name and EA-inode checks. Two controls were
actually red, returning external magic corruption where the inline-capable path
previously returned invalid-name/NameTooLong or Unsupported. Legacy external-only
layouts retain their external-error priority. A complete inline preflight now
restores that ordering before external I/O; its match verdict is ignored until
both locations are checked. No new success bypass, write boundary or disk-format
change is introduced. Rust/C controls cover invalid namespace, empty/long suffix,
inline EA-inode refusal, legacy precedence and byte/clock/write preservation.

Both controls pass after repair; all 1,137 prior native successful occurrences
remain with two additions (1,139 passed, zero failed/ignored, 171 controls).
Lint and repository checks pass. Fresh repaired-source CI, complete native/guest
retention and standalone family closure remain pending. The broader ownership,
lifecycle, raw handles/C lifetimes, mixed zero-extra, ignored-fuzz, admission,
index-growth and recovery limitations remain open; casefold writes stay blocked.

### Same-value setter repair; independent emission review pending — 2026-10-20

Follow-up: exact repaired-source `01f96c8` now passes focused workflow
39391995739. Both SHA-256-verified artifacts pass all sixteen/eight setter
layout/first-attribute and 84 retained-profile audits, plus sixteen byte-identical
policy-emission images. Three CI unit contexts retain every baseline occurrence
and execute all 169 controls. Full run 39391995709 instead fails a separate
casefold-symlink fixture's escaped absolute target and masked guest lookup error.
The relative-target/status-propagation fixture correction and focused five-test
evidence step now need fresh CI. Full-suite retention and standalone family
closure remain pending; earlier repaired-driver emission status below is historical.

The timestamp blocker below now has an implemented repair. The saved two probes
were rerun red, and all eight promoted policy controls now pass: five actual
failures become green while three validation/gate/cleanup controls stay green.
Fresh units retain every one of the 1,129 preceding successful occurrences and
add eight, with 1,137 passed and zero failed/ignored. Full lint passes.

The corrected reference-only job at `aa266f3` in
[39390763789](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39390763789)
passed. Its downloaded archive matches the advertised SHA-256. All 32 ordinary
inline/empty/external/shared observations across eight profiles preserve ctime,
raw inode bytes, values and external blocks. Four separate constructed duplicate
diagnostics preserve both copies and ctime; those inputs remain outside healthy
qualification. The initial reference fixture failure was actual Linux attribute
deduplication before an exclusive-source constructor; distinct source data fixes
that fixture while retaining its strict reference-count assertion.

The private planner may now return no plan only after both locations are decoded,
the external header/checksum and EA-inode refusals are retained, and exactly one
ordinary matching value exists. Current SB magic/checksum/EXT_ATTR checks prevent
an old mount snapshot hiding missing publication. Zero-extra initialization,
zero-reference blocks and cross-location duplicate cleanup retain the mutation
path. Context and volume refusal precede this decision. No-effect success avoids
the clock, marker, commit, flush and publication; real changes retain the guarded
whole-plan boundary. No ownership, lifecycle, raw-handle exclusion, seal or C
lifetime guarantee is added.

The first-attribute emission now advances its clock between identical calls,
and the expanded oracle checks sixteen Rust/C images against Linux-created input
bytes. These checks compile locally; fresh repaired-source Linux execution,
actual artifacts, complete-suite budgets/retention and standalone family closure
remain pending. The sixteen-layout same-value timestamp/epoch expectations now
follow the observed no-effect policy; mutation assertions remain intact. The
handoff records the separate measured older guest-budget failure and repair.
Stage 5 is not yet quantified; casefold writes remain blocked.

### Setter image audit and same-value policy blocker — 2026-10-20

The actual `casefold-fixtures` archive from successful focused run
[39348758499](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39348758499)
was downloaded and its GitHub-advertised SHA-256 verified. Its exact source is
`9035b80616c5bdb37940960e27883463752098d3`; this is earlier setter evidence,
not qualification of the subsequent symlink backports at `b6ccad6`.

Read-only audits of the saved bytes and independent reports cover all sixteen
setter layout profiles and eight separate fresh first-attribute profiles.
The sixteen layouts contain 192 setter operations and 64 preserved peer/untouched
records. The first-attribute profiles contribute 24 emitted and 24 reference
records: together, 280 emitted Linux records and 304 debugfs records agree with
the saved final bytes. The audit checks inode identity, attributes and values,
payload, metadata, allocation/counters, shared references, dirty/closed state,
EXT_ATTR publication and applicable checksums. All 84 retained metadata profiles
also pass the existing read-only audits: 68 chmod/chown/timestamp/flag profiles
and sixteen removal profiles. These healthy-image results do not qualify faults,
power loss, ownership aliases or the mixed zero-extra layout previously excluded.

**Family closure is blocked by a reproduced same-value timestamp mismatch.**
Every one of the eight Linux `first-*.reference.txt` reports records
`same same_value_changed 0`, with identical before/after ctime for the second
identical inline attribute set. First nonempty and first empty creation advance
ctime in all eight profiles. The current planner always calls the clock and
stamps ctime, including an unchanged value. The fixed `SetterClock` used by the
emission matrix makes both sets appear to have the same timestamp and therefore
does not establish semantic equivalence.

Two temporary native probes advanced the clock by ten seconds between identical
inline sets. Rust byte-path and C calls both returned success and retained the
value, but changed ctime from `1700000100` to `1700000110`. Both tests failed
their Linux-derived preservation assertion: zero passed / two failed / zero
ignored, exit 101. The exact executed controls are saved in
[CASEFOLD-SETXATTR-SAME-VALUE-RED.patch](CASEFOLD-SETXATTR-SAME-VALUE-RED.patch).
Active test source was restored byte-for-byte; no failing test or production
repair is active. The probes establish this ordinary inline case only; they do
not establish external/shared/duplicate or repeated-empty policy.

Next, qualify those policy branches independently and repair the demonstrated
case without bypassing context/refusal checks, external validation, duplicate
cleanup, feature publication or failure retirement. Promote the saved probes
with an advancing-clock oracle, preserve all baseline tests, then inspect fresh
exact-source CI and perform the remaining standalone review. Full/focused
backport CI remains pending at this checkpoint; no output cap is changed based
on a prediction. Stage 5 remains not yet quantified and casefold writes stay
blocked.

### Setter CI evidence investigation — 2026-10-19

Exact source `9035b80` passed focused Linux workflow
[39348758499](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39348758499),
including API-reported success for the setter/first-attribute step and all retained
metadata steps. Full workflow
[39348758428](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39348758428)
failed in x86_64 `chore test` with an actual exit-65 check annotation. Whole-guest
and the other jobs passed. These statuses do not identify the failing tier or
supply its measured log size, complete test counts or saved image/policy answers.

Actual download attempts remain denied at the log destination and current blob
hosts. The applied policy still lacks the requested exact destinations despite
its configured blob wildcard. `extprogress.md` records the current artifact IDs,
hosts and authenticated metadata/digest checks. No archive was downloaded or
hash-verified; no numeric cap was raised or output suppressed. Actual image,
multiset, 161-control and standalone family/evidence review remain **blocked**.
The partial source walk reconfirmed private whole-plan staging, pre-callback
entry, planned-byte-preserving dirty normalization and complete guarded effects.
It does not close the caller-exclusion, ownership or qualification limitations.
The driver test's fixed clock cannot distinguish the second same-value set's
ctime policy; inspect the actual independent reference observation before closure.

Next: restore only the missing routes, obtain and verify the source/baseline
artifacts, measure the actual failed tier and make a demonstrated repair. Then
finish exact evidence/multiset/family review. This follow-up changes documentation
only. Stage 5 remains not yet quantified and all casefold write gates stay closed.

### Guarded public setter implementation — 2026-10-19

The reviewed whole-plan migration is now implemented; standalone family review
and independent qualification are **pending**. The unchanged saved patch was
actually rerun (one passed / fourteen failed), then its ten public controls and
positive external control were promoted without weaker assertions. Only the four
constructed SB-order diagnostics use the new private setter-specific boundary.
Twenty-seven new native controls pass; the full native multiset retains every
one of the preceding 1,095 occurrences and adds exactly 27. The existing Rust/C
worker and failure-retirement controls now exercise all six metadata families.

`apply_setxattr_bytes` enters the existing mount context before callbacks, uses
verdict-only refusal and prepares exactly one owned buffer. Its single-caller
external helper stages layout/alloc/ref edits without committing. The caller
stamps ctime and stages the inode, including successful inline edits; first inline
creation now includes EXT_ATTR publication. The separate setter boundary checks
context identity and staged primary-SB shape/magic, normalizes only VALID_FS and
its checksum, and guards marker plus whole commit/checkpoint/flush/allocation
publication. The raw commit primitive, removal-specific boundary and shared
xattr helpers remain unchanged. No seal, complete ownership proof or Ready state
is added; the existing caller-exclusion and C-lifetime contract still applies.

Native controls cover actual layout/counter/shared-peer branches, pre-effect
refusals, observed write/flush faults, Rust planning/effect panics caught by C or
Rust, publication poisoning with retained context, foreign contexts and exact
planned-byte normalization. The legacy-grow native expectation was corrected
because its small input already occupied an external block. That test failure
required no production repair. Native runs do not qualify real-volume bytes or
interrupted writes. The measured unit output-budget repair is in the handoff.

The new exact Linux oracle keeps all six prior metadata oracles and adds sixteen
legacy128/checksummed256, 1/4 KiB, Rust/C, direct/journal profiles plus eight
separate first-attribute profiles whose input must have EXT_ATTR clear. It saves
Linux first/same/empty reference observations independently of actual driver
emission, full phase images and debugfs/Linux metadata/value/data reports. It
compiles locally but requires exact CI execution and actual artifact/policy audit;
same-value semantic equivalence cannot be inferred merely from Linux readback.
All original and new CI outcomes must be checked at their own commits. No host
oracle or mount is used. Full-suite budget growth must be measured before any
further cap change; no existing full-suite cap/floor/filter was lifted here.

Next: finish exact full/focused CI, artifact/multiset and standalone setter-family
review, repairing actual failures first. Keep mixed zero-extra, ignored fuzz,
insertion aliases, ownership/mapping/lifecycle/seal/admission/collision/growth/
recovery and every casefold-write gate open. Stage 5 remains not yet quantified;
no real-card, Windows or SteamOS readiness is claimed.

### Public setxattr staging and marker audit — 2026-10-19

Audited clean fetched `b6190fdb29fdada30b86507bf4f6a90f471a54ae`, whose
[exact full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39196911445)
passes. This checkpoint saves an executed dormant red patch and a design contract;
production, active tests, workflows, pins/locks and write gates are restored
byte-for-byte. No setter dispatch migration or new emitted-format qualification.

**Executed evidence:** [CASEFOLD-SETXATTR-RED.patch](CASEFOLD-SETXATTR-RED.patch)
was saved, applied afresh, actually rerun and reversed. Both runs report **one
passed / fourteen failed / zero ignored**, exit 101, with the same fifteen test
names. All fixtures reached the intended assertions. The final audit tier is
155 lines / 10,568 bytes inside its isolated 800-line / 65,000-byte cap; existing
tier caps/floors/filters are unchanged. Ten failures exercise real public calls:

- Rust string/byte and C calls succeed and write despite a held shared context;
  they should refuse before device/runtime callbacks under the proposed contract.
- Rust/C missing-path and invalid-prefix calls return the existing correct error
  but each performs one marker write and flush before refusal.
- A Rust Runtime panic during external-set planning is caught by the existing C
  boundary as EIO, but does not latch `operation_failed`. No foreign callback
  unwinds; this demonstrates the missing shared-context failure bridge.
- An actual inline-only creation leaves ctime at 1,700,000,000 instead of the
  injected 1,700,000,100 and leaves EXT_ATTR clear although the value is present.
  These are **native emission/desired-contract findings**, not new independent
  Linux corruption or compatibility verdicts. Qualify the intended behavior
  independently, including same-value and empty-value semantics, before closure.

Four other failures deliberately snapshot a clean primary SB, stage EXT_ATTR and
an artificial free-count decrement, then run guarded marker plus raw commit.
Feature/counter/checksum survive, but VALID_FS becomes set in all 1/4 KiB direct/
journal profiles. **These are constructed future-boundary ordering diagnostics,
not complete valid allocations or current public clean-bit regressions.** Current
public set marks before it plans. One passing control retains real external first
allocation and rewrite through Rust/C in all four profiles: eight combinations,
sixteen sets, exact values/ctime/sectors/free counts/feature/dirty state and close/
reopen. It is self-consistency evidence, not a new Linux fixture qualification.

**Current flow and owned changes.** `apply_setxattr_bytes` calls marker-writing
`refuse_write` before resolve and verified-inode reads. It initializes a supported
zero-extra inode's first 32 extra bytes in owned raw storage, tries the inline
layout, and removes any stale copy from the other region in that same operation.
An inline success owns one buffer and commits directly. On overflow the private
`apply_setxattr_external_block` creates its own buffer and has two terminal raw
commit sites, existing-block and first allocation. It has exactly one caller.
There is no whole-operation context around these three terminal commit sites.

| Prepared branch | Owned metadata that must stay together |
|---|---|
| Inline insert/replace | Raw inode, possible zero-extra initialization, ctime; first-attribute feature publication is an open native finding. |
| Growth into external storage | Removed inline copy plus external update/allocation, ACL pointer, sectors, counters and feature bit. |
| Shrink with an unshared external remainder | New inline value, external rewrite with the old copy removed, inode and checksum. |
| Shrink freeing the last unshared block | Inline value, bitmap/BGD/SB credit, cleared pointer and reduced sectors. |
| Shrink with a shared remainder | Inline value, allocated private remainder, old reference decrement, bitmaps/counters/pointer; sectors stay constant. |
| Shrink releasing the last shared attribute | Inline value, old reference decrement, cleared pointer/reduced sectors; no free-count credit. |
| Existing unshared external rewrite | External block plus inode/ctime; the immutable mount feature snapshot can request an already-set EXT_ATTR OR again. |
| Existing shared external rewrite | New private block, old reference decrement, allocation bitmap/BGD/SB debit, pointer; sectors stay constant. |
| First external allocation | Full new block, bitmap/BGD/SB debit, EXT_ATTR OR, pointer and added sectors. |

The inline encoder checks fit before re-encoding, so NoSpace leaves that region
unchanged. Local raw/owned-buffer edits are discarded on later failure; this does
not promise unchanged cache residency or memoized committed allocation views.
Checked external reads retain magic/shape/checksum and EA-inode refusals, even
when an inline edit was locally possible. Shared helpers remain unchanged.
Existing independent one-copy, shared-block, checked-block and zero-extra tests
supply earlier limited evidence; none covers this new guard/first-inline metadata
contract. The removal mixed zero-extra/nonzero-extension shape stays unqualified.

**Reviewed next boundary.** Enter the same mount context before any filesystem
lookup/device/runtime callback; retain verdict-only volume refusal and current
error priorities. Prepare exactly one owned BlockBuffer for the whole setter.
Convert the single-caller external helper into staging over that buffer, without
an internal commit. Finish pointer/sector/layout edits, resolve successful-change
ctime and first-attribute feature publication, then checksum/stage the inode.
The inline ctime/feature red findings must be addressed explicitly and measured
with Linux, rather than silently inherited or described as qualified fixes here.

Validate context identity and any staged primary-SB whole-block shape/magic
before effects. Normalize only its VALID_FS bit and checksum while preserving
planned feature ORs, counter deltas, other state and bytes outside the SB region.
Keep inode-only plans inode-only when no feature change is required. Marker-once
and complete raw commit/checkpoint/flush/post-durability allocation publication
must stay inside failure-latching effects. The raw primitive is unchanged; adapt
the four constructed cases to the new private family boundary, not a global raw
commit fix or routing setters through a removal-named public adapter. Ordinary
buffers/context identity establish neither seals nor ownership/admission/Ready.
Promote the ten public controls without weakening their assertions, retain the
eight-combination legacy positive control, and adapt only the four constructed
cases to the new family boundary. Inline findings still need independent Linux
reference/emission evidence before being called qualified fixes.

Preserve volume-before-lookup and path-before-name behavior. On legacy inodes
without an inline area, first allocation can currently fail before namespace
validation; checked external-block refusal can also precede name validation.
Do not silently move these priorities while extracting the planner. C retains
its existing null/name/value validation, 64 KiB input ceiling, byte paths, errno/
error clearing and caller-owned handle lifetime contract; its external-support
comments are stale and should be corrected with migration.

**Independent/failure qualification still required.** Retain all six exact
metadata oracles and all 134 existing controls. Add real six-family Rust/C worker
exclusion in both directions, actual layout/counter/refcount branches, clean/prior-
dirty refusals, observed write/flush faults and planning/effect/publication unwind
retirement including C close. Preserve the whole baseline occurrence multiset.
A Linux matrix needs legacy128/checksummed256, 1/4 KiB, Rust/C, direct/journal,
inline/grow/shrink/shared/duplicate/external/zero-extra cases and untouched peers.
Use **separate fresh-image first-attribute cases**: existing input attributes
already set the global EXT_ATTR bit and cannot expose this gap. Record independent
Linux first-inline metadata behavior, raw dirty feature/counter/checksum state,
judged e2fsck and exact debugfs/Linux values/data/times before qualification.
No host oracle/mount was run. The full-source ignored fuzz gate, ownership/
lifecycle/seal/admission/mapping/collision/growth/recovery and every casefold-write
gate remain open. This audit does not qualify Windows, SteamOS or the real card.

### Public removal family review closure — 2026-10-18

Repaired source `396488818d774de1852b8872cce5ea0350ec5122` now passes
[exact full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39192028328)
and [exact focused Linux CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39192028338).
The standalone removal-family implementation/evidence review is **complete, with
no blocker within the existing caller-exclusion contract**. This closes the
pending full-suite review below, not the broader ownership/write qualification.
The reviewed `dbfb9f4` checkout differs from repaired source only in these two
handoff/review documents; no production, active test, workflow, pin or cap changes.

SHA-256-verified whole-guest and x86_64 artifacts each report **1,923 passed,
zero failed, one pre-existing ignored fuzz test**. Counter comparison retains all
1,895 preceding qualified whole-suite occurrences, including duplicates and
expected panics, and adds exactly the 25 removal controls, two fixture controls
and one removal Linux oracle. All **134** guard/family/parser/fixture controls
execute in five CI contexts: standalone native, aarch64 and x86_64 debug units,
x86_64 release and whole guest. Native unit Counters match at 1,095. Actual whole
suite logs measure 3,313 lines / 180,692 bytes (x86_64) and
3,477 lines / 189,631 bytes (guest),
inside unchanged limits. Separate oracle/kernel/lwext4 tiers pass 225/28/9 tests;
installed CLI and script checks pass. The existing ignored slow corrupt-input
fuzz case remains a broader qualification gate; it is not credited as passed.

The complete repaired focused image/report audit is recorded immediately below:
sixteen removal profiles / 112 mutations / 144 Linux and debugfs final answers
per stream, plus all 68 retained metadata profiles / 136 final inode records.
Source review confirms pre-callback entry, fallible whole-plan preparation,
planned-byte-preserving SB dirty normalization and complete failure-latching
commit/checkpoint/flush/allocation-publication effects. Counter-order and
publication-poison diagnostics remain synthetic. No new implementation fix or
active test is needed for this closure. The unrepaired parent's full suites
failed the same fixture comparison and aborted later targets; its partial runs
are not whole-suite approval.

The mixed zero-extra/nonzero-extension layout stays unqualified. Other readers,
writers, flush/maintenance/raw handles and C lifetimes need caller coordination.
Buffer/context ownership is not a seal, whole-filesystem ownership proof,
Ready state or admission. Keep ignored fuzz, insertion aliases, complete mapping/
ownership/lifecycle/seal/admission/collision/growth/recovery and casefold-write
gates open. No real-card, Windows, SteamOS or new power-loss claim follows.

**Next:** a bounded public setxattr staging/marker/feature-counter design audit
and actual red preparation, before dispatch migration. Trace inline grow/shrink
and duplicate moves, shared/unshared external updates/allocation, zero-extra
initialization and feature/counter publication. Retain all six exact metadata
oracles and prior controls. Its distinct whole-plan requirements must be reviewed
before generalizing this removal-specific commit boundary.

### Repaired removal independent-evidence review — 2026-10-18

Follow-up: the closure above records passing exact full CI and complete whole-
suite artifact/control review, superseding this earlier checkpoint's pending status.

Reviewed clean fetched `396488818d774de1852b8872cce5ea0350ec5122`.
Its [exact focused Linux CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39192028338)
passes; its [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39192028328)
still has x86_64 and whole-guest suites running. The eight other full-CI jobs
pass. **Standalone removal-family review remains open pending those suites and
actual whole-suite artifact/control review.** Do not migrate another family on
the strength of focused success alone.

The actual SHA-256-verified focused artifact covers all sixteen removal profiles,
64 four-phase images, 112 mutations and 144 final inode answers in each independent
Linux/debugfs stream. Read-only raw bytes agree on data, attribute values/names,
IDs/mode/timestamps, pointer/sector/refcounts, allocation/free counters and dirty/
closed state. Checks include 32 superblock, 80 descriptor, 288 inode and 128
external-xattr checksum observations. All sixteen Linux stderr reports are empty.
The coherent zero-extra fixture passes and stays zero-extra after kernel readback;
the original mixed layout remains unqualified. Deliberately seeded duplicates
are not ordinary Linux-created valid-input claims. The allocator's later COW
reuse of an earlier freed block is accounted to the new owner.

All five retained exact metadata oracles pass, and their 68 profiles / 136 final
inode records agree, including 96 timestamp and 96 flag phase answers per stream.
All preceding exact insertion/inventory/role/FIEMAP/fixed/allocation/ownership
filters pass. The removal test executes one pass, zero failures/ignored, with
401 lines / 52,394 bytes inside unchanged limits/floor. Full local/native-CI/
aarch64-CI unit Counters match at 1,095 occurrences, including all 134 retained
guard/family/parser/fixture controls. Full guest controls remain unverified.

The source review finds no blocker **within the current caller-exclusion
contract**: entry precedes callbacks; whole-plan refusals are untouched; prepared
SB normalization preserves planned counters/unrelated bytes; full commit,
checkpoint, flush and allocation publication stay inside failure-latching effects.
The four constructed counter diagnostics and publication-poison control remain
synthetic ordering/containment evidence. Raw commit/shared helpers and production
timestamp behavior are unchanged by the fixture repair. Ordinary buffers and
context identity do not prove ownership, seals, readiness or admission; other
readers/writers/flush/maintenance/raw handles and C lifetimes need coordination.
No power-loss, arbitrary-map, casefold-write or real-card qualification is added.

See the [independent-evidence checkpoint](extprogress.md) for measurements and
retained gates. Finish exact full-CI/artifact review next, resolving failures,
then close this family review before auditing setxattr's distinct whole-plan,
feature-bit/counter and marker requirements. Keep the existing ignored fuzz,
ownership/lifecycle/seal/admission/mapping/collision/growth/recovery gates open.

### Removal Linux fixture mismatch and repair — 2026-10-18

Source `499c1b4`'s exact focused workflow failed; full CI was still running at
investigation. Its first two legacy profiles completed Linux/e2fsck checking.
The third, Rust 1 KiB/checksummed256/direct, passed emitted metadata/e2fsck/debugfs
checks and failed the exact Linux timestamp tuple on `/zero`. Changing only
`extra_isize` to zero left old extension words in the saved inode. The ordinary
stat failure representation omitted the differing nanoseconds, so the exact
kernel tuple is unknown. The captured input words execute red under the old
fixture constructor; the new cfg(test)-only constructor clears unused storage
while preserving the base inode and refuses erasing inline attributes.

Both native and VM zero-extra fixtures now use that coherent construction, with
normal checksum restamping and independent input checking. All sixteen profiles,
strict exact timestamp/value/data checks, filters, floors, caps and artifacts
remain in place. Failed Linux checks now save partial stdout/stderr and name the
actual/expected timestamp tuples. No production removal or timestamp semantics
change. **The mixed zero-extra/nonzero-extension layout remains unqualified**;
its Linux interpretation requires separate black-box investigation, not a claim
that the narrowed profile repairs or qualifies arbitrary such inodes.

The native/source review found no blocker within the recorded caller-exclusion
contract, and the saved artifact audit verifies all 68 retained metadata profiles
plus partial removal bytes/reports. Independent removal qualification and the
standalone family review **remain open** until repaired exact full/focused CI and
all sixteen profiles/artifacts are reviewed. See the latest
[handoff checkpoint](extprogress.md) for the actual red, repair, fresh validation,
partial evidence and unchanged ownership/lifecycle/admission/write gates.

### Public removal implementation status — 2026-10-18

The bounded migration specified below is implemented locally from `65d99d7`.
Existing Rust string/byte and C path removal now enter the shared metadata
context, prepare the full old removal plan, normalize any already-staged primary
SB's clean bit/checksum without replacing planned bytes, and guard marker plus
complete commit/checkpoint/flush/publication effects. Global raw commit, setxattr,
C lifetimes and casefold gates remain unchanged. Context identity and prepared SB
shape/magic are checked before effects; inode-only plans add no SB block.

The saved five public failures were rerun and promoted unchanged; the four
constructed diagnostics now exercise the new boundary. Native controls add
five-family Rust/C worker exclusion, allocation/refcount/counter preservation,
all observed write/flush faults, planning/effect/publication panics, refusal
priorities and C close. See the latest implementation checkpoint in
[extprogress](extprogress.md) for exact fresh counts and measurements.

**Independent qualification remains pending.** The new sixteen-profile exact
Linux/e2fsprogs removal test and its CI floor/artifacts are added and compile,
but cannot execute here without the harness VM. Git/helper reads verify the
remote audit tip and an explicit Bearer header restores the existing API binding;
its full CI passes. No new access or secret is needed. See the handoff for exact
fresh validation and publication status. No standalone qualified removal-family
review is claimed: publish without forcing, inspect exact full/focused CI and
actual images/reports, and resolve findings before reviewing this family or
migrating another one.
All ownership/lifecycle/seal/admission, ignored-fuzz, index-growth and write gates
listed below remain open.

### Xattr-removal staging and recovery-marker audit — 2026-10-18

Audited clean fetched `a387b90340f3edfeb80ed8d7e9f3be022276008d`, whose
[full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39119479035)
passed. This checkpoint changes documentation and saves an executable red patch;
production, active tests, workflows, dependency pins/locks and every write gate
are restored byte-for-byte. No xattr public dispatch is migrated here.

**Executed evidence:** [CASEFOLD-XATTR-REMOVAL-RED.patch](CASEFOLD-XATTR-REMOVAL-RED.patch)
adds ten temporary native controls. The final actual run reports **one passed,
nine failed, zero ignored**. Five failures cover current public behavior: Rust
string/byte and C path removals bypass a held metadata context; Rust/C absent-
attribute calls return their expected NotFound/ENOENT but still write and flush
one clean-mount marker. Four separate constructed cases (1 KiB/4 KiB, direct
checksummed ext4/journaled legacy ext3) snapshot a clean primary superblock,
stage a synthetic free-count decrement, then execute existing marker and raw
commit routines under the private context. Each first clears the on-disk clean
bit, commits the planned counter edit, then exposes state **1** with marker still
owed. The direct profiles' superblock checksums remain valid. This is a controlled
ordering reproduction, not a complete valid allocation transaction, an oracle
verdict, or the current public planning order.

The passing control executes all four profiles through the current public
last-external-block removal. It frees the block, clears the inode pointer,
reduces sector accounting, credits the free count and keeps the marker dirty
before close. Current public removal marks before planning. Initial 32 MiB
1 KiB fixtures hit the project's multi-group mkfs restriction; the corrected
fixtures use one-group 8 MiB at 1 KiB and 32 MiB at 4 KiB. The final report above
contains no setup failures. No host oracle or mount was used.

The saved patch is self-contained and includes apply/run/reverse commands plus
these scope limits. It was reversed after execution; all source hashes match
the starting commit and the dormant patch passes `git apply --check`. There are
no active failing/ignored tests added. The five public failures should become
green in the next migration. The four raw-buffer diagnostics must be promoted
into green tests of the **new private guarded whole-buffer boundary**, retaining
their original diagnostic meaning. Do not globally alter raw commit semantics
solely to make this constructed call order pass.

#### Concrete current path

Rust string calls the byte path. C performs pointer/path/name conversion and
then calls that same byte path; C reference creation and caller lifetime rules
remain external prerequisites. The byte path currently runs `refuse_write`
(verdict plus marker), resolves, reads/checksums the inode, plans inline removal,
plans external removal even if inline succeeded, stamps ctime, stages the inode,
and commits one BlockBuffer. An error after inline planning drops its local raw
image; a later bad external block never justifies committing the inline edit.

| Removal branch | Staged result before commit | Accounting/publication |
|---|---|---|
| Inline only, or inline copy plus external name absent | Re-encoded inode; existing external block is still checked when referenced | ctime/inode checksum; no external allocation change |
| Unique external block retains other attributes | Re-encoded external block and inode | Block-number-based xattr checksum; pointer and sector count unchanged |
| Shared external block retains other attributes | New private block, old block refcount decremented, inode points to new block | Allocation bitmap/BGD/SB free count decrease; new and old xattr checksums; sectors unchanged |
| Unique external block becomes empty | Inode pointer cleared; bitmap bit freed | Bitmap/BGD/SB checksums and free counts increase; sectors decrease by block-size/512 |
| Shared external block becomes empty for this inode | Old block retained with one fewer reference; target inode pointer cleared | No physical free; sectors decrease; SB counter helper still stages primary superblock with delta zero |
| Same name in both locations | Both copies removed in this same local plan | External branch above decides copy/free/refcount changes; no stale copy is left to reappear |

`buffer_read_xattr_block` checks magic, one-block layout and checksum before
editing. The decoders refuse EA-inode-backed entries, including unrelated ones;
retain that supported subset. `i_extra_isize == 0` and 128-byte inodes have no
inline area for removal; removal does not initialize one. Prefix validation is
currently reached through a planner, after resolution: a missing path wins over
an invalid namespace. With neither inline area nor external block, even an
unknown namespace yields NotFound; on legacy external-only layouts the external
block check can fail before prefix validation. C invalid name encoding/null
arguments retain their existing early EINVAL behavior. Do not silently hoist a
new universal namespace check or change these priorities during routing.

Allocation planning reads the current staged bitmap and staged uninit flags.
`clear_bgd_uninit_flag_if_set` records prospective clears in the buffer only;
bitmap, descriptor and superblock helpers edit local owned vectors. The mount's
allocation-clear map publishes inside `commit_block_buffer`, after direct flushes
or journal checkpoint success. Publication can allocate/lock/panic, so it belongs
inside the entire guarded commit effect. `allocation_groups` may memoize a view
of already-committed clears during planning; that is not publication of proposed
clears. Refusal checks must preserve logical clear state and disk bytes, without
promising unchanged read-cache residency or memoization identity.

#### Reviewed whole-buffer boundary for the next migration

1. Existing Rust string/byte and C path **removexattr only** enter the same mount
   context before volume/device/runtime callbacks or resolution. Reuse that
   context for all helpers. Keep setxattr, shared helper semantics, C reference/
   lifetime rules and every other unguarded operation externally coordinated.
   No new public inode API or family lock is needed.
2. Extract the existing full removal planner into a private prepared BlockBuffer:
   preserve inline/external duplicates, shared/nonshared branches, errors and
   ctime/pointer/sector/checksum semantics. Finish all fallible planning and local
   preparation before marker effects. Drop refused plans without physical writes
   or logical allocation publication, on clean and prior-dirty mounts.
3. A private context-checked whole-buffer commit boundary must normalize the
   **already-staged primary superblock**, when present, before any effects.
   Its block is `1024 / block_size`, region offset `1024 % block_size`. Validate
   the prepared block/region shape and magic, clear only `EXT4_VALID_FS` in its
   staged state, then restamp its superblock checksum when enabled. Preserve
   every planned counter, feature, orphan and unrelated byte, including other
   state bits and bytes outside the 1024-byte region. Do not replace it with a
   fresh device read or mounted snapshot, which would lose planned edits. Do
   not stage a primary superblock just to normalize an inode-only buffer.
4. Retain the marker-once effect, then a second effect covering **all** of the
   existing commit, flush/checkpoint and allocation publication. If marker is
   already owed, only the commit effect advances versions. Any error/unwind
   conservatively retires later write/flush/clean-close reuse. The journal writer
   independently patches RECOVER and its checksum in transaction superblocks;
   that handling does not clear VALID_FS. Keep both protocols and their ordering.
   Do not clear the recovery marker after commit to repair a stale staged image.
5. Keep raw `commit_block_buffer` and unrelated callers' semantics unchanged.
   The new boundary consumes only this operation's private plan under its same
   mount context; ordinary BlockBuffer ownership, counters and pointer identity
   are not a seal or owner proof. No Ready state, owner cache/publication,
   transaction admission or casefold write enablement is introduced. Future
   refactoring of the four reviewed inode-only families must retain their exact
   existing controls and callback/failure behavior.

**Next bounded implementation and evidence:** apply/promote the five public red
controls, implement the private full removal planner and whole-buffer boundary
above, and add green controls for the constructed ordering under that boundary.
Cover clean/prior-dirty normalization, 1 KiB/4 KiB offsets, checksums, preserved
counter edits/other state bits, marker absent/already owed, foreign context and
malformed prepared regions. Exercise all five guarded families' Rust/C worker
reentry, every observed direct/journal marker/commit write/flush fault, planning/
effect/publication unwind and actual C close, with joined worker ownership and
no foreign C callback unwind. Preserve invalid UTF-8 path and C error clearing.

Add an exact independent public Rust/C Linux/e2fsck/debugfs removal oracle with
an explicit supported matrix covering inline, unique external, shared COW,
shared release, duplicate-copy and final-block-free branches. Verify both target
and unaffected sharing inode values/refcounts, allocation bits and free counts,
`i_file_acl`/sector accounting, ctime/generation/metadata/data, dirty state before
release and clean close/reopen. Include legacy/zero-extra and corrupted/unsupported
refusals without broadening formats or permissions. Keep all five existing exact
metadata oracles, floors/caps/artifacts and complete baseline retention; use
measured evidence for any later legitimate output-budget change. Inspect exact
source full/focused CI and actual artifacts, then perform a standalone removal-
family review before another migration. Shared and duplicate cases above are
source-audited here; this checkpoint adds no new independent Linux qualification.

Stage 5 is **not yet quantified**. The pre-existing ignored slow fuzz case,
three insertion aliases, complete mapping/collision/growth and ownership/
lifecycle/seal/admission/recovery work remain open. All casefold write gates and
real-card/Windows/SteamOS qualification remain unchanged.

### Public inode-flag implementation/evidence review — 2026-10-18

Reviewed clean fetched `4d4df814316c1aeae804c23408d0cf2866081d0d`.
Exact [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39099985737)
and [focused Linux evidence](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39099985843)
passed. Downloaded and SHA-256-verified the actual focused artifact and standalone,
x86, aarch64 and guest tier artifacts; this review uses this source commit's
results, not an earlier family's CI.

**Executed controls.** All sixteen flag, 71 retained mount-family, five report-
parser and fifteen operation controls execute in standalone/x86/aarch64 units
and both full-suite library contexts: **107 distinct controls verified in five
contexts**. Each native CI unit Counter equals the fresh local **1,068**-occurrence
baseline, including repeated names and expected panics, with zero failed/ignored.
Both compiled full-suite libraries pass **767**, zero failed/ignored. Complete
x86 and guest suites each pass **1,895**, zero failed, **one existing ignored**
slow deep-fuzz test. That ignored case remains a broader qualification gate.
The guest log is 3,448 lines / 186,153 bytes. Cold standalone unit output is
1,324 lines / 89,387 bytes, within unchanged 1,400 / 90,000 caps.

The exact flag oracle executes all **16 profiles** (Rust/C × 1 KiB/4 KiB ×
128-/256-byte inodes × direct/journal) at **306 lines / 36,281 bytes**, with
224 tool and 64 kernel markers. All four retained exact metadata oracles pass:
24 timestamps, sixteen chown, eight public chmod and four private chmod profiles.
No family control or exact metadata oracle is ignored.

Audited **68 actual emitted images**, **248 debugfs reports** and **124 Linux
reports**. The new sixteen flag profiles supply 96 phase/inode records in each
independent report stream across set/same/restore. Full flag words, identity,
mode/type, owner IDs, unchanged access/modification times and exact ctime agree.
Repeated same-flags calls advance ctime in both legacy and extended layouts;
restores retain the original managed bits. Debugfs verifies generation throughout;
Linux independently observes flags through its public GETFLAGS ioctl and checks
metadata and payload. The asserted 64-bit ioctl ABI is the harness VM's scope.
NODUMP/NOATIME are the successful Linux flag matrix; broader flag-feature semantics
and caller privilege/permission policy remain separate obligations.

The read-only public-format byte audit checks all **136 final inode records**,
allocation and payload, including the 32 new restored flag inodes. All current
and retained checksummed profiles pass **44 superblock / 66 descriptor / 88 inode
checksum** checks. Flag/timestamp O_NOATIME reads preserve exact final times and
checksums; the retained older chmod/chown file-read atime change is accounted
for. Timestamp short-extra-12 layouts and their full Linux values remain covered.
Judged e2fsck accepts inputs, emitted phases and Linux readback closes. No host
oracle, mount, or forbidden implementation source was used.

**Decision: no blocker within the existing public flag caller contract.** Rust
string/byte and existing C path dispatch share the reviewed mount context with
chmod/chown/utimens, before lookup and device/runtime callbacks. One context spans
verified-inode policy/planning, full buffer preparation, guarded clean-marker and
complete commit effects. The exact changed-bit mask permits already-set managed
bits unchanged and refuses managed changes, including CASEFOLD/INDEX/EXTENTS.
Missing-path priority, same-flags ctime/commit, mode/IDs/generation/data, precision
and checksums remain. Planning refusals preserve bytes, marker and versions on
clean and prior-dirty mounts. The corrected NOATIME C comment matches the public
header; public signatures and flag constants/policy are unchanged.

Every reached direct/journal marker/commit write/flush failure and Rust planning/
effect unwind conservatively retires later write/flush/drop reuse. Cross-family
Rust/C callback workers refuse before effects in both directions, own/join handles
before C release and defer timeout assertions until outer guard release. Actual
exported C close preserves clean/owed-dirty state after caught planning panics,
without relying on a pre-existing direct-commit failure or unhealthy journal.
C errors and successful error clearing pass. No foreign C callback unwinds;
arbitrary callback termination is not promised.

C reference creation and caller-owned lifetime/aliasing contracts are unchanged.
Other operations/readers/maintenance/raw handles remain externally coordinated;
argument destruction, swallowed callback errors, foreign unwind/panic-abort and
fairness/duration are outside this ordering claim. No whole-API concurrency,
owner readiness/cache/publication, buffer seal or transaction admission is added.
The ignored fuzz case, insertion aliases, complete mapping/collision/growth and
recovery/admission dependencies stay open. Stage 5 is **not yet quantified**;
casefold writes and real-card/Windows/SteamOS qualification remain gated.

**Next bounded task: multi-block xattr-removal staging/marker design audit and
executed red preparation, before migrating public removal dispatch.**

The existing removal path is larger than the reviewed inode-only families:
`apply_removexattr_bytes` can edit both inline and external copies, copy a shared
external block, decrement a shared reference, or free the last block. Allocation/
free branches stage bitmap, descriptor and primary-superblock counters alongside
the inode; allocation-uninit state publishes inside `commit_block_buffer` after
commit. `buffer_unshare_xattr_block`, `buffer_release_xattr_block` and their
allocation/counter helpers stage into the same private BlockBuffer.

A concrete ordering dependency must be resolved first: the current public path
marks dirty **before** planning, so its buffered primary superblock includes the
dirty state. Moving the marker after preparation can leave a clean-state copy
in that buffer; committing that copy would overwrite the new dirty marker.
`BlockBuffer::get_mut`, `buffer_patch_sb_counters` and `commit_block_buffer` do
not automatically reconcile that state. The flag family's inode-only buffer does
not stage this counter block. This is a dependency of the proposed migration;
no current xattr migration or marker fix is claimed by this review.

1. Audit concrete inline/external/duplicate-copy/shared-copy/shared-release/last-
   block-free paths through all staged counters, checksums, `i_file_acl`, `i_blocks`
   and uninit publication, including errors and callbacks. Record existing
   namespace/missing/error ordering, zero-extra/128-byte behavior and unsupported
   EA-inode/corrupt external-block refusals. Keep setxattr unchanged and external
   caller coordination explicit; correct stale removal-scope comments when that
   family is actually migrated, without changing behavior or ABI.
2. Execute current public held-context and refused-plan marker controls red, and
   a controlled scratch-buffer reproduction of stale clean-state overwrite using
   existing private marker/commit routines. Preserve actual red evidence in an
   ignored log plus a self-contained saved patch in the existing documentation
   area; do not leave failing tests active in the normal suite. Label a constructed
   pre-marker buffer reproduction as such, without claiming the current legacy
   public path uses that proposed ordering. Existing controls must remain green.
3. Review a private whole-buffer commit boundary that acquires once before reads,
   keeps all planning private, preserves the dirty marker in final staged primary
   superblock bytes/checksum and never loses planned allocation/counter updates.
   Specify complete marker/commit/publication effects, sticky failure and clean/
   prior-dirty/refused paths. Do not generalize the inode-only adapter by dropping
   its context or clearing state after a commit. No new lock, seal, Ready state,
   owner cache/publication or casefold admission is introduced.
4. Record the bounded follow-on implementation/test contract: existing Rust string/
   byte and C path removexattr only, no new inode API; five-family callback and
   failure/panic/close controls; exact independent Linux/e2fsck/debugfs removal
   evidence for inline, external, shared, duplicate and last-block cases, with
   allocation/counter/refcount and payload preservation. Retain all five existing
   exact metadata oracles/floors/caps/artifacts. Complete this focused crash/
   staging design review before changing public dispatch, and keep every gate.

### Public inode-flag shared-guard implementation — 2026-10-18

Existing Rust string/byte flag updates and the C path delegate now share the
reviewed chmod/chown/utimens mount context, before lookup/device/runtime callbacks.
The existing verified-inode planner retains the exact user-modifiable changed-bit
policy, already-set managed bits, missing-path priority, ctime/checksum and metadata
semantics. Same-flags calls still perform a ctime update/commit. Planning refusals
now precede clean-marker writes. Prepared buffers use guarded marker and complete
commit effects with the same sticky failure bridge. No new inode API, lock, ABI,
C reference/lifetime rule, owner readiness/cache/publication, builder or seal is
introduced. Other operations/readers/maintenance/raw handles stay externally
coordinated; this is not whole-API concurrency or transaction admission.

Four actual public bypass/refused-marker controls executed red and now pass.
Sixteen new controls retain all 71 prior mount controls; the expanded group passes
87 with zero failed/ignored. Coverage includes same-flags ctime/versions, all
managed-bit refusals, volume/missing priority, foreign context, clean/prior-dirty
untouched refusals, Rust/C worker overlap in all four families/both directions,
every reached direct/journal marker/commit write/flush failure, Rust planning/
effect panics, later sticky refusals and actual exported C close. Workers remain
owned/joined before C release, with timeout assertions after outer guard release.
No foreign C callback unwinds. An incorrect new positive fixture flag was caught
and corrected to declared constants; an existing C comment now states the correct
NOATIME value, with flag policy and public signatures unchanged.

The new exact sixteen-profile public Rust/C oracle checks set/same/restore phases
across block/inode sizes and direct/journal writes, with 96 debugfs and 48 Linux
reports planned. Debugfs checks full flags, generation, metadata and times; Linux
independently checks full flags through public GETFLAGS, identity, mode/IDs, all
times and payload, using O_NOATIME. Every mount is finished before external access
and judged e2fsck checks each phase. All four prior exact metadata steps, floors,
caps and artifact entries remain byte-identical. Fresh local validation is in
extprogress; no local VM run is claimed. Exact source full/focused CI, artifact
audit and standalone family review remain pending before another migration.
Stage 5 stays not yet quantified. Every casefold gate, ignored fuzz qualification,
insertion alias and lifecycle/ownership/admission/seal/mapping/growth/recovery
dependency remains open; no real-card/Windows/SteamOS qualification is claimed.

### Public timestamp implementation/evidence review — 2026-10-17

Reviewed clean fetched `f8649b7df1f1f5f677519e12163e3c2d9ca27204`.
Exact [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39085491295)
and [focused Linux evidence](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39085491294)
passed. The earlier `5dd65f9` short-extra report failure is retained in the repair
checkpoint; this review uses repaired-commit artifacts, not the failing run.

**Executed evidence.** All nineteen timestamp controls, three timestamp-report
controls, sixteen chown/two owner-report controls, 36 chmod controls and fifteen
operation controls execute in standalone/x86/aarch64 units and both full-suite
library contexts: **91 distinct controls verified in five contexts**. All three
native CI unit Counters equal the fresh 1,052-occurrence local baseline, with
zero failures/ignored tests. The guest library passes 750 with zero ignored;
the complete x86/guest suites each pass 1,878 with zero failures and **one existing
ignored** slow deep-fuzz test. The guest log is 3,432 lines / 184,381 bytes.
That ignored case remains a broader qualification gate. Cold standalone unit
output is 1,308 lines / 87,590 bytes within unchanged 1,400 / 90,000 caps.

The exact timestamp test passes all **24 profiles**: Rust/C, 1 KiB/4 KiB,
128-byte legacy / 256-byte extra-isize-12 / 256-byte extra-isize-32, direct/journal.
Its log is 338 lines / 42,417 bytes with 248 tool and 72 kernel markers. The
three prior exact metadata steps also pass unchanged: 16 chown, eight public
chmod and four private chmod profiles. No exact step or family control is ignored.

Downloaded and SHA-256-verified the actual focused artifact and all four relevant
full-CI tier artifacts. Audited **52 emitted images**, 152 debugfs reports and
76 Linux reports: the 24 timestamp profiles add 96 intermediate inode reports
and 48 Linux reports to the 28 retained metadata profiles. All 96 timestamp
phase/inode Linux answers match complete signed seconds and nanoseconds,
identity, mode/type and IDs. Debugfs matches generation and full reported time
words; only extra-isize 12 uses its observed legacy display. Linux still verifies
the full values hidden by that display immediately after explicit updates and
again after sentinel/no-op updates. The read-only public-format audit checks all
104 final inode records, allocation, payload and supported fields, plus **36
superblock / 54 descriptor / 72 inode checksums** where enabled. Timestamp
O_NOATIME reads preserve exact final times/checksums; the retained older chmod/
chown file-read atime update is accounted for. All final short-extra inodes still
have extra-isize 12. Judged e2fsck checks inputs, emitted phases and Linux closes.
No host oracle/mount or forbidden implementation source was used.

**Decision: no blocker within the public timestamp caller contract.** All public
Rust string/byte/inode and existing C path/inode delegates use the same mount
context as chmod/chown. Entry precedes lookup/device/runtime callbacks and
arbitrary inode conversion; volume and pure explicit-time verdicts precede
conversion. Path dispatch retains invalid-time-before-missing-path priority and
never reacquires. Current live/generation checks, single-clock planning, sentinel
precedence, signed explicit range/nanosecond checks, precision/refusals and ctime/
checksum packing remain. This is not a redesign of out-of-range runtime-clock
behavior; independent clock values cover the stated supported profiles.

Both-omitted plans validate volume/identity and preserve bytes, marker and operation
versions on clean/prior-dirty mounts, without a commit. Other plans finish buffer
preparation before guarded marker and complete commit effects. Every propagated
effect error or Rust unwind conservatively retires write/flush/drop reuse, including
caught planning panics on an otherwise clean mount. Cross-family Rust/C worker
callbacks refuse before effects in both directions; handles are owned and joined
before C release and timeout assertions follow outer guard release. C errors and
success clearing, actual exported close and retained guard/failure controls pass.
No foreign C callback unwinds; arbitrary callback termination is not promised.

Public signatures, C reference creation and caller-owned lifetime/aliasing rules
are unchanged. Other operations/readers/maintenance/raw handles remain externally
coordinated. Argument destruction, swallowed callback failures, foreign unwind/
panic-abort and fairness/duration are outside this ordering claim. There is no
whole-API concurrency, owner readiness/cache, builder publication, buffer seal,
transaction admission or casefold write enablement. Stage 5 is not yet quantified;
real-card/Windows/SteamOS and exhaustive recovery qualification remain open.

**Next bounded implementation: existing public inode-flag updates only.**

1. Migrate Rust string/byte `apply_set_flags` and existing C path `fs_ext4_set_flags`
   into the same mount context/failure bridge, before lookup/device/runtime
   callbacks. Reuse one context and the shared metadata commit helper; no second
   lock, new inode API, ABI/lifetime change or unrelated family migration.
2. Extract the current verified-inode planner. Preserve the exact changed-bit
   test against `USER_MODIFIABLE_FLAGS`: managed bits already set may be passed
   back unchanged; INDEX/EXTENTS/INLINE_DATA/CASEFOLD and other managed changes
   stay refused. Preserve missing-path/error order, ctime/checksum semantics,
   mode/IDs/generation/data, and same-flags calls still performing their existing
   ctime update/commit. Only planning refusals move before marker effects.
3. Execute actual public held-context and refused-plan marker controls red before
   routing, then green. Extend all four families' Rust/C callback-worker exclusion,
   direct/journal marker/commit write/flush error and Rust panic controls, untouched
   refusals, sticky later refusal and exported C close. Retain invalid-UTF8 byte
   paths and all prior controls. Join workers before releasing borrowed C handles;
   never allow a foreign C callback to unwind.
4. Add a new exact Rust/C Linux/e2fsck/debugfs flag oracle with metadata, full
   flags, generation and payload evidence over supported block/inode and direct/
   journal layouts. Linux must independently observe emitted flag bits and
   metadata. Include refused managed-bit changes and same-flags ctime behavior;
   retain all four existing exact metadata tests, floors, caps and artifact paths.
5. Preserve the complete native baseline as Counter multisets, lint and appropriate
   build checks; inspect exact-commit full/focused CI and actual artifacts before
   the next standalone family review. Keep all casefold gates, ignored fuzz,
   insertion aliases, owner/lifecycle/admission/seal dependencies and complete
   mapping/collision/growth/recovery work open. No index admission is connected.

### Timestamp oracle display repair — 2026-10-17

Initial timestamp source `5dd65f9` failed exact full/focused CI at the same
extra-isize-12 debugfs report assertion. The two full-suite library contexts
reported 746 passed / one failed and did not finish the suite; timestamp
qualification is incomplete. Verified downloaded artifact digests and inspected
the actual report/image: debugfs omitted timestamp extra words despite the
emitted inode containing the requested full mtime/ctime. The prior `719f66c`
documentation checkpoint now passed full CI.

A captured-report native control executed red before the test-only correction
and now passes. Only the observed short-extra-12 report is compared as legacy
seconds; normal-extra timestamps remain exact. Two further controls reject
missing normal extra data and incorrect legacy/short seconds. Linux now checks
unprojected full timestamps immediately after explicit updates and again after
sentinel/no-op updates, with metadata/data checks and O_NOATIME. The extra phase
closes the coverage gap where a later update could overwrite values hidden by
the short debugfs display. All 24 profiles, 96 debugfs reports, existing final
Linux reports, generation/metadata/no-op/exclusion/e2fsck checks and prior oracles
remain; 24 explicit-phase Linux reports are added under existing artifact paths.
No production write, dependency, workflow cap/floor or gate changed.

Fresh local results are recorded in the matching extprogress checkpoint.
Repaired exact full/focused CI, all-profile artifact audit and standalone family
review remain required before another migration. No local VM, Windows, SteamOS
or card qualification is claimed. Stage 5 stays not yet quantified; casefold
writes, the ignored fuzz qualification, insertion aliases and lifecycle/admission/
mapping/collision/growth/recovery work remain gated/open.

### Public timestamp shared-guard implementation — 2026-10-17

Public Rust string/byte/inode `utimens` and existing C path/inode delegates now
share the same mount control/context/failure bridge as reviewed chmod/chown.
Entry precedes lookup/device/runtime callbacks and conversion; volume and pure
time verdicts precede arbitrary conversion. The shared raw planner retains
sentinel precedence, signed seconds, nanosecond/range checks, one clock value,
current extra-field precision/refusals and ctime packing. Paths keep invalid-
time-before-missing-path priority. Validated both-omitted plans perform no marker,
buffered commit or version advance and preserve inode/clean/dirty state. Other
plans prepare the full buffer before guarded marker and complete commit effects.
Pre-marker refusal/conversion ordering is intentional; no ABI/reference creation,
timestamp format, permission policy, other family or casefold write gate changed.

Four actual bypass/conversion/no-op-marker controls executed red and now pass.
Nineteen new controls retain all 52 prior family and two parser controls; the
expanded 71-control mount group passes locally. Controls cover supported time
and inode-layout boundaries, Rust/C sentinel/no-op/error ordering, cross-family
callbacks in both directions, every observed direct/journal marker/commit write
and flush fault, Rust planning/effect panics, subsequent write/flush refusal and
exported C close with both clean/dirty state. Worker handles remain owned/joined
before C release, with timeout assertions deferred until outer guard release.

The new exact public Rust/C timestamp oracle plans 24 profiles across block
sizes, legacy/normal/short-extra inode layouts and journal/direct writes, saving
intermediate explicit/sentinel debugfs reports and final Linux metadata/data
checks. All three existing exact metadata oracles, floors/artifacts and gates
remain. This is an implementation checkpoint awaiting exact-commit full/focused
CI and standalone family/evidence review; it does not qualify writes or whole-API
concurrency. Other operations/raw handles and C lifetime/aliasing remain externally
coordinated. No owner readiness/cache, builder publication, seal or transaction
admission is added. The ignored fuzz qualification, insertion aliases and full
mapping/collision/growth/recovery/admission work remain open. Stage 5 stays not
yet quantified; casefold writes and actual-card/Windows/SteamOS remain gated.

### Public chown implementation/evidence review — 2026-10-17

Reviewed `198baba2e7b31bdac28e3e814812040f2ced5d5d`, equal to clean fetched
remote tip. Exact-commit [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39005643503)
and [focused Linux evidence](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39005643486)
passed. Downloaded the actual artifacts and verified the focused SHA-256.
All sixteen chown, two parser and 36 retained chmod controls execute in native
standalone/x86/aarch64 contexts (each 1,030 passed, zero failed/ignored, full
Counter multisets matching the fresh baseline) and the compiled guest library
(727 passed, zero failed/ignored). The whole guest suite reports 1,855 passed,
zero failed, one existing ignored deep-fuzz case at 3,408 lines / 181,700 bytes;
that broader qualification gate remains open. These are 54 distinct controls,
not 216 tests. No family control or exact metadata oracle was ignored.

The public chown oracle passes all sixteen profiles (145 lines / 18,870 bytes,
96 tool and 32 kernel markers); retained public/private chmod pass eight/four
profiles at 81/49 lines. All execute as exact one-test focused steps and in the
guest. Audited all 28 images, 56 debugfs and 28 Linux reports. Full IDs, mode/type,
generation, exact ctime, preserved input times and payload agree. All 32 signed
group displays preserve the stored owner bits. Judged e2fsck accepts every input,
emitted close and Linux readback. The read-only byte audit verifies clean state,
allocation, payload, timestamp fields and 20 superblock/30 descriptor/40 inode
checksums where enabled. Legacy 128-byte profiles are not checksummed; the
Linux final file-read atime/checksum update is accounted for. No host oracle,
mount or forbidden source was used. Full scope/evidence is in `extprogress.md`.

**Decision: no blocker within the public chown caller contract.** Entry precedes
lookup/device/runtime callbacks, and verdict-only volume checks precede arbitrary
inode conversion. Shared inode planning precedes guarded marker and complete commit. Full owner halves and each/both
leave-unchanged sentinels preserve current ctime/checksum semantics. Refusals,
sticky uncertainty, Rust panic handling, C error/success clearing and strong/any
generations remain. Actual C close preserves clean/dirty state after caught
planning panics. New worker handles are owned/joined before release, with timeout
checked after outer guard release; successful workers execute and cleanup is
inspected. Arbitrary callback termination is not guaranteed.

Public/C signatures and reference creation are unchanged. Other operations,
readers/maintenance/raw handles and C lifetime/aliasing remain externally
coordinated. Argument destruction, swallowed callback errors, foreign unwind/
panic-abort and fairness/duration remain outside this ordering claim. No owner
readiness/cache, builder publication, buffer seal, transaction admission or
casefold-write enablement is added. Stage 5 remains not yet quantified.

**Next bounded implementation: public timestamp updates (`utimens`) only.**

1. Reuse the same mount control/context/failure bridge for public Rust string,
   byte and generic inode timestamp entries and existing C path/inode delegates.
   No second family lock, ABI/reference/lifetime redesign or unrelated migration.
   Enter before device/runtime callbacks, lookup or arbitrary conversion; check
   volume and explicit time verdicts before conversion. Reuse the context from
   path to inode planning, without calling another public entry recursively.
2. Share existing `TimeUpdate`/raw-inode planning, preserving `TIME_OMIT` and
   nanosecond `UTIME_OMIT`/`UTIME_NOW` precedence, signed seconds, range/nanosecond
   checks, single clock value for NOW/ctime, and current extra-field precision/
   refusal behavior on supported 128-/256-byte and short-extra layouts. Preserve
   invalid-time-before-missing-path priority. Document the intentional pre-marker
   refusal ordering; do not introduce timestamp format or permission policy.
3. Both omitted times still validate the volume and live inode/generation, keep
   inode/ctime unchanged and return a no-effect plan: no marker/commit/version
   advance. Other successful plans prepare the full buffer before guarded marker
   and complete commit, retaining cross-family sticky refusal on uncertainty.
4. Execute public overlap/conversion controls red before routing. Require green
   Rust/C cross-family controls in both directions with chmod/chown, byte paths,
   generation/volume/invalid-time ordering, explicit/sentinel/mixed/both-omit
   behavior, pre-1970 and post-2038 boundaries, nanoseconds and short-extra
   layouts. Exercise actual callback workers, every observed marker/commit write
   and flush fault, Rust planning/effect panics and C errors, later cross-family
   write/flush refusal and exported C close. Retain/join worker handles on every
   timeout/error path before C handle release; no foreign C callback unwind.
5. Add a new exact public Rust/C Linux/e2fsck/debugfs timestamp oracle over the
   supported inode layouts, including no-op/sentinel updates, seconds/precision
   boundaries and untouched mode/owner/generation/data. Retain all three exact
   current metadata oracles, controls, baseline occurrences, floors/artifacts and
   write gates. Review this family before another migration. Actual-card/Windows/
   SteamOS, the ignored fuzz qualification, insertion aliases and full mapping/
   collision/growth/recovery/admission dependencies remain separate.

### Public chown oracle report repair — 2026-10-17

Initial implementation `2cedf28` failed exact full/focused CI in its new oracle:
debugfs printed group `0x87654321` as `-2023406815`, rejected by unsigned parsing.
All three failed contexts stop there on the first profile; later profiles and
Linux readback were not qualified. A read-only audit of that saved image agrees
with its full stored IDs, mode, generation, ctime and payload; this does not
replace independent qualification. The actual reports/runs are in `extprogress.md`.

A new native captured-report control executed red before the test-only parser
correction and now passes. Owner displays accept checked u32 or i32 decimal,
preserving exactly 32 bits; malformed/overflow controls pass and generation
remains unsigned. Production mutation code, public/C ABI, all sixteen profiles,
exact oracles/floors and write gates are unchanged. The unit byte budget grows
only for two added native controls, with the actual prior CI measurement recorded
in `chores.yml`; the line cap and all other budgets remain. This is an oracle
repair awaiting corrected exact full/focused CI and standalone family review.

The older `ab8129a` guest rerun passed at the same SHA; its complete artifact
retains one pre-existing ignored deep-fuzz test. Neither that ignored qualification
nor the cause of its original VM loss is resolved by the passing retry. Check
the corrected owner evidence and complete family review before another migration.
Casefold writes, owner publication and whole-API/actual-card/Windows/SteamOS
qualification remain gated; Stage 5 is not yet quantified.

### Public chown shared-guard implementation — 2026-10-16

Public Rust string/byte/inode chown and its existing C delegates now use chmod's
mount context and sticky failure bridge, with no second gate or C ABI change.
Acquisition and verdict-only volume checks precede arbitrary inode conversion;
lookup/inode planning reuses the context before marker and complete-commit
effects. The existing high/low owner fields, each/both unchanged sentinels,
mode/type/ctime/checksum semantics and all casefold write refusals remain.

Three executed-red public bypass/conversion controls now pass. Sixteen new
controls plus all 36 retained chmod controls pass locally, covering supported
inode layouts, Rust/C cross-family callback workers in both directions, volume/
planning refusals, full marker/commit write/flush faults, caught Rust panics and
subsequent cross-family write/flush/close refusal. New worker handles remain
owned and joined, including timeout/error cleanup before C handle release.
The new exact public Rust/C Linux/e2fsck/debugfs test covers sixteen supported
layout/block-size/journal profiles and all owner sentinel updates; both exact
chmod tests, every existing floor/artifact and write gate are retained.

This is an implementation checkpoint awaiting exact-commit full/focused CI and
standalone emitted-metadata/family review, not a completed migration review.
The preceding documentation HEAD `ab8129a` also needs its failed guest-job rerun:
original run 38991380517 ended on VM/SSH loss, exit 255, with no completed suite
or established cause. Its local 1,012 units and separate x86 oracle suite passed;
the actual logs and current retry status are recorded in `extprogress.md`.
Do not claim the transport interruption fixed, relax a budget or skip a test.
Check both sets of independent evidence before another family. Other API paths,
caller-owned C lifetimes, physical-owner readiness, builder/cache publication,
dependencies/overlays/seals and complete mapping/collision/growth remain open.
Casefold writes stay blocked; Stage 5 remains not yet quantified.

### Public chmod implementation/evidence review — 2026-10-16

Reviewed `c47b328381ea67dceca26fc33c796bd241c7fbac`, equal to fetched
remote tip with a clean tree. Exact-commit [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38983322842)
and [focused Linux evidence](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38983322854)
passed. Downloaded actual artifacts: all eighteen public and eighteen retained
private controls execute in standalone/x86/aarch64 unit contexts (each 1,012
passed, zero failed/ignored, full multisets matching the fresh baseline) and
the compiled guest library (708 passed, zero failed/ignored).

Both exact metadata tests execute in guest and focused steps. The public
Rust/C test covers eight profiles, one passed/zero failed/ignored, 81 lines /
9,553 bytes, with 48 tool and sixteen kernel markers. The retained private test
covers four profiles, one passed/zero failed/ignored, 49 lines / 4,960 bytes,
with 24 tool and eight kernel markers. The whole guest suite separately reports
1,836 passed, zero failed, **one ignored**, at 3,391 lines / 179,772 bytes. The
existing slow `fuzz_smoke::deep_walk_byte_flip_fuzz_never_panics` stays an
unresolved broader qualification gate; no family control or exact oracle was
ignored and workflow success does not waive the missing evidence.

Audited all twelve current images, 24 debugfs reports and twelve Linux reports,
including all eight new public images. All 24 emitted inode records agree on
mode/type, generation/identity, exact extended ctime and preserved input times/
payload. Judged e2fsck accepts input/emitted close/readback. A separate format-
byte audit verifies clean superblocks, descriptor/inode checksums, allocation
bits, ctime epochs, mtime, directory atime and every file payload. The final
Linux data read can update file atime/checksum after the saved debugfs report;
this expected difference is accounted for. No host oracle/mount or forbidden
implementation source was used. The full table/audit scope is in `extprogress.md`.

**Decision: no blocker within the public chmod caller contract.** Entry precedes
conversion and filesystem/runtime callbacks; verdict-only checks precede inode
conversion, planning precedes the marker, and path dispatch reuses its context.
Both effects retain invalidation/error propagation and sticky failure. Rust
panics retire before C error return; ordinary C errors/success clearing and
the any-generation sentinel are preserved. The exported-close control leaves
a prior dirty marker unchanged after a caught planning panic with ordinary
commit flags healthy. Test-only helpers add no ABI; successful worker controls
join before handle release.

This is a bounded public family, not whole-API concurrency or ownership proof.
Other operations/readers/maintenance/raw fields/devices and C handle lifetime/
aliasing remain externally coordinated. Caller argument destruction, swallowed
errors, foreign unwind and callback duration remain outside the ordering claim.
No owner builder/cache, Ready state, buffer seal or casefold admission is added.

**Next bounded implementation: guarded chown family only.**

1. Route public Rust string/byte/inode file owner/group changes and existing
   C chown delegates through the **same** mount control, borrowed context and
   sticky failure bridge as chmod. Do not add another per-family lock. Adapt
   private helper names/visibility only as needed, retaining all current test
   name occurrences and public signatures/error handling. No other family or
   C ABI reference/lifetime redesign is included.
2. Enter before generic inode conversion and filesystem/runtime callbacks.
   Verdict-only volume refusals precede conversion; lookup/generation/raw-inode/
   block planning precedes marker I/O. Path/inode helpers reuse one context.
   Keep the whole early marker and complete direct/journal commit inside effects;
   errors/unwind cannot heal common write/flush/clean-close health.
3. Share the existing chown planner: full 32-bit uid/gid low/high halves, each
   `u32::MAX` leave-unchanged sentinel (including both together), current mode/
   type/ctime/checksum behavior and supported inode sizes. Preserve byte names,
   stale generations, read-only/casefold/unsupported/journal refusals and the
   distinct C `FS_EXT4_GEN_ANY` generation convention. No permission-policy or
   new filesystem-feature expansion is part of this checkpoint.
4. First execute red controls showing unguarded chown bypassing held chmod
   and generic-conversion ordering, then green actual public Rust/C controls.
   Cover both directions of cross-family overlap, callback-worker reentry,
   conversion/unwind, full-width IDs/sentinels, missing/stale pre-I/O refusal,
   marker/commit write/flush faults, caught Rust panic/C errors, subsequent
   cross-family write/flush refusal and exported close without clean restoration.
   Workers must remain joined/owned before handles can be released, including
   timeout/error paths; do not unwind through non-unwinding extern C callbacks.
5. Add a new exact public Rust/C Linux/e2fsck/debugfs oracle over currently
   supported inode layouts, including 128-/256-byte owner fields, high IDs and
   sentinel updates. Check full emitted owner IDs, unchanged mode/type and other
   times, ctime/checksums, generation and payload. Retain both chmod exact VM
   tests, all baseline occurrences, focused floors/artifacts and gates. Review
   this family before another migration. File owner IDs grant no physical
   storage-ownership readiness, cache publication, builder invocation or seal.

The ignored fuzz qualification, three insertion aliases, other families and
transaction dependencies/overlays/seals/publication/map/collision/growth remain
open. All public casefold writes stay blocked; whole-API and actual-card/Windows/
SteamOS qualification remain later. This review changes only the existing
review/handoff documents; fresh validation and unchanged source hashes are in
`extprogress.md`, with no new release build, local VM or environment change.

### Public chmod dispatch implementation checkpoint — 2026-10-16

The reviewed private family now owns public Rust string/byte/inode chmod and
the existing C path/inode delegates. Public signatures and production C ABI
code are unchanged. Generic inode dispatch enters and checks volume verdicts
before invoking `Into<InodeRef>`; conversion is planning before marker I/O.
Path-to-inode helpers reuse one context, with no recursive public entry.
Missing/stale refusals now precede marker writes. Shared metadata semantics,
all write gates and the sticky marker/commit/unwind failure bridge remain.

Three actual public-entry controls first failed with public writes/conversion
accepted while a context was held; the same controls now pass. Expanded Rust/C
controls cover conversion ordering/unwind, callback-worker reentry, byte paths,
generation and metadata, pre-I/O refusal, every observed direct/journal marker/
commit write and flush, caught Rust panics inside the C boundary, error reporting
and subsequent refusal. Initial expanded C test inputs incorrectly used zero
as an any-generation sentinel; they now use unchanged `FS_EXT4_GEN_ANY`, while
strong-generation/stale checks remain. Full counts and validation are recorded
in the matching `extprogress.md` checkpoint.
The exported C unmount is exercised after a successful write and a caught Rust
planning panic, with the dirty marker owed and ordinary commit flags healthy:
no callback or clean restoration occurs. Test-only handle helpers add no ABI.
There are eighteen new public controls, retaining all eighteen private controls;
the complete baseline and final occurrence comparison is in `extprogress.md`.
Full local units pass 1,012, zero failed/ignored, retaining every one of 994
baseline occurrences as a Counter multiset and adding exactly those eighteen.
The log is 1,260 lines / 82,671 bytes under unchanged caps. Fresh lint, locked
release CLI build/install and saved insertion-red patch applicability pass.

New exact oracle
`fs::guarded_chmod::needs_host::linux_checks_public_chmod_metadata` covers eight
Linux-populated checksummed images: public Rust/C separately, 1 KiB/4 KiB,
journal/no-journal. Held-gate refusals preserve bytes, public path/inode success
advances the expected effect versions, and judged e2fsck/debugfs/Linux check the
emitted modes/types/generations/times/checksums/data. The old private oracle and
all existing focused steps/floors remain. New artifacts retain all eight images,
sixteen debugfs reports, eight Linux reports and the exact log. VM execution is
pending; local compilation alone does not qualify this migration.

**Next: standalone public migration/evidence review before another family.**
Inspect exact-commit full/focused logs and every new image/report, retain private
evidence and resolve failures first. Review conversion/callback ordering, one
context per call, Rust panic retirement before the C catcher returns and the
unchanged C caller ownership/lifetime obligations. Other operations/readers/
repair/replay/refresh/raw fields/devices remain externally serialized. No owner
cache/readiness, builder invocation, buffer seal, admission or whole-API
concurrency is established. The ignored deep fuzz gate, three insertion aliases,
dependencies/overlays/seals/publication and mapping/collision/growth remain open;
every public casefold write gate stays blocked.
Refusing generic conversion does not suppress destruction of caller-owned Rust
arguments; caller destructors/raw access remain outside the ordering claim.

### Private mount chmod implementation/evidence review — 2026-10-16

Reviewed `3ee3aa440a962e517092f0935bfe20fed3a068f0`, equal to fetched
remote tip with a clean tree. Exact-commit [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38950053488)
and [focused Linux evidence](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38950053446)
passed. Actual downloaded standalone/x86/aarch64 unit artifacts each execute
all eighteen private controls among 994 passed, zero failed/ignored. Their
complete name-occurrence multisets match the fresh local baseline. The compiled
guest library executes the same eighteen plus the exact new oracle among 689
passed, zero failed/ignored. The focused oracle executes one test over all four
1 KiB/4 KiB journal/no-journal profiles, zero failed/ignored, at 49 lines /
4,960 bytes with 24 tool and eight kernel markers.

The whole guest suite is separately scoped: 1,817 passed, zero failed, **one
ignored**, at 3,370 lines / 177,796 bytes. The existing slow
`fuzz_smoke::deep_walk_byte_flip_fuzz_never_panics` remains a broader
qualification gate; it is not waived or hidden by the workflow's success.

Audited every new image, all eight debugfs reports and all four Linux reports.
Linux/e2fsck/debugfs agree on emitted modes/types, inode identity/generation,
exact post-2038 ctime, preserved input times and file data. A separate byte
audit verifies final superblock, descriptor and inode checksums, clean state,
allocation bits, all eight emitted records, ctime epoch bits, mtime and payload.
Saved debugfs reports precede Linux's final data read; the latter may update
file atime/checksum. That expected difference is accounted for. No host oracle
or mount was used. See `extprogress.md` for the complete evidence table.

**Decision: no blocker within the private contract.** Borrowed mount identity
and nonblocking entry precede callbacks/lookup, nested helpers share a context,
planning refusals precede marker effects, both marker and complete commit stay
inside uncertainty, and unwind/poison/failure cannot restore mount health.
Common write/flush/Drop refusal precedes callbacks after a private fault. No
ownership readiness, builder/cache, buffer seal or admission is established.
External exclusive access remains required across all public operations,
readers, maintenance, raw device/field escapes and C ABI handle lifetimes.
The guard does not repair unsafe reference aliasing or swallowed callback errors.

**Next bounded implementation: public chmod dispatch only.**

1. Route `apply_chmod`, byte-path chmod and public inode chmod through this
   reviewed private family. Existing C path/inode delegates should reach it
   without a second guard. Preserve public signatures and C error/panic handling;
   no C ABI reference/lifetime redesign or other operation migration is included.
2. Enter before **arbitrary generic inode conversion** (`Into<InodeRef>`) and
   filesystem/runtime callbacks. A contended public entry must refuse before
   invoking conversion, lookup, generation checks, marker or other callbacks.
   Check verdict-only volume refusals before conversion; conversion must not
   itself be preceded by a marker write.
   Keep nested path/inode helpers on the same context; never make guarded path
   dispatch call a separately guarded public inode method.
3. Retain byte names, generation/mode/type/time/checksum semantics, all volume
   write refusals and the sticky failure bridge. The private path's pre-marker
   planning ordering now becomes public; retain refusal-byte/flag controls.
   No owner build/readiness/cache, buffer seal or casefold admission is added.
4. Add controls over actual public Rust/C dispatch for overlap, callback-worker
   reentry, generic conversion ordering/unwind, stale generation, marker/commit
   faults, caught ABI panic/error reporting and subsequent write/flush/close
   refusal. Retain all eighteen private and complete baseline occurrences.
   Add exact public-dispatch Linux/e2fsck/debugfs evidence; old public tests or
   the private oracle alone cannot prove the new public routing executed.
5. Review this migration before another family. Other operations, reads,
   repair/replay/refresh, raw handles/fields and unsafe caller aliasing remain
   externally serialized. This limited public guard does not establish whole-
   API concurrency or transaction-current ownership. Keep the ignored fuzz gate,
   three insertion aliases, dependency/overlay/seal/publication and complete
   mapping/collision/growth work open. Every casefold write gate remains blocked.

This review changes only the existing review/handoff documents. Fresh units,
lint and source hashes are recorded in `extprogress.md`; no release rebuild,
local VM run, sibling/pin/lock or environment configuration change is claimed.

### Private mount chmod implementation checkpoint — 2026-10-16

The preceding gate review's bounded integration is implemented: one private
control per mount, a mount-borrowing context checked before any callback, private
path-byte/inode entries reusing that context, shared chmod metadata planning and
separate early-marker/whole-commit effects. Public Rust/C delegates retain legacy
unguarded ordering. The private path does verdict/lookup/generation/raw-block
planning before its marker, so those errors do not become uncertain writes.
The mount failure latch covers effect errors and unwind (including a caught
panic with a still-live context), unhealthy/poisoned entry and context unwind.
Common write verdicts/flush/Drop refuse before callbacks after failure; refresh
or replacing the control cannot recover the mount.

An executed first-marker-flush regression failed with a later legacy write
accepted; the same check is now green. Eighteen focused controls pass on actual
formatted mounts, covering identity/overlap before callbacks, byte paths,
generation/mode/type/checksum/ctime, callback-worker reentry, every observed
direct/journal marker/commit write and flush, caught/uncaught panic, sticky
legacy refusal and no clean restoration on faulted drop. See the matching
`extprogress.md` checkpoint for full scope. Fresh local units pass 994, retaining
all 976 baseline occurrences and adding exactly these 18; zero failed/ignored.
The log is 1,242 lines / 80,883 bytes under unchanged caps. Lint, the locked
release CLI build/install and saved insertion-red patch applicability pass.

New exact test
`fs::guarded_chmod::needs_host::linux_checks_private_chmod_metadata` is compiled
and added to focused CI with a one-test floor and artifact retention. It covers
four Linux-created/populated checksummed 1 KiB/4 KiB journal/no-journal images,
both private entries, judged e2fsck, eight debugfs metadata reports and Linux
mode/type/exact-ctime/preserved-times/data checks. VM execution is pending;
older public chmod evidence does not qualify the new private path.

**Next: standalone family/evidence review before public wiring.** Inspect actual
exact-commit full/focused logs, every new image/report and native/guest control
execution, diagnose failures and keep broader qualification gated. No owner
readiness/cache, builder invocation, buffer seal or transaction admission is
added. Other operations/readers/maintenance/raw fields and device handles/C ABI
aliasing and lifetime still need caller exclusion. All casefold write gates,
the three insertion aliases and the ignored slow-fuzz qualification remain.

### Operation-control implementation/evidence review — 2026-10-16

Reviewed `b6e89b0a8e17d6ece855e61eb44ff47ea52f5f9d`, equal to fetched
remote tip with a clean tree. Its exact-commit [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38865265666)
and [focused Linux workflow](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38865265682)
succeeded. The preceding audit's full CI also succeeded. Downloaded actual full
CI unit, x86, aarch64 and compiled-guest artifacts and compared all operation
test names as Counters with the fresh local baseline.

| Evidence | Gate controls executed | Enclosing result | Lines / bytes |
|---|---|---|---|
| Standalone unit job | 15 | 976 passed, zero failed/ignored | 1,232 / 79,626 |
| x86 host unit tier | 15 | 976 passed, zero failed/ignored | 1,226 / 79,189 |
| aarch64 unit tier | 15 | 976 passed, zero failed/ignored | 1,226 / 79,189 |
| Compiled guest library | 15 | 670 passed, zero failed/ignored | Within the guest log below |

These are the same fifteen controls executed in four contexts, not sixty
distinct tests. The whole guest artifact is 3,353 lines / 176,086 bytes and
reports **1,798 passed, zero failed, one ignored**. The pre-existing slow
`fuzz_smoke::deep_walk_byte_flip_fuzz_never_panics` did not execute. This is
already described in the earlier kernel-budget handoff; the workflow's success
does not turn it into a zero-ignored or exhaustive suite. It remains a broader
qualification gate, not a waived test. None of the gate controls were ignored;
the guest library containing them has zero ignored. No unrelated unbounded fuzz
sweep, ignore removal or output-budget change is introduced by this review.

**Decision:** no blocker found in the disconnected gate under its stated
caller obligations. Private state is behind one mutex; `try_lock` refuses
overlap rather than queueing, and unhealthy/poisoned entries are not recovered.
`Operation` owns the non-Send mutex guard and borrows its control. Every mutable
status method requires the exclusive context, while copied status is diagnostic
only. Nested helpers can borrow that context without relocking. A callback can
wait for a competing worker's prompt refusal while the outer guard stays held.
There is no fairness, throughput or callback-duration guarantee and no hidden
retry loop; a caller that never returns can retain the gate indefinitely.

Borrowed stamp identity cannot outlive its control or permit its address to be
reused while the stamp is subsequently checked. Identity is checked along with
both counters, so equal versions on different controls refuse. This is control
identity only, not an existing association with a filesystem, device or buffer.
Neither fresh stamps nor healthy status imply owner coverage/readiness: the
validity enum has no Ready state and no owner result is stored or published.

Checked counter computation precedes assignment, including the last usable
versions. Overflow invalidates/faults without wrapping or partial counter
advance. `effect` invalidates and marks uncertainty before external work; an
error latches fault and successful return leaves the view invalid. Unwinding
with the guard faults/poisons it. An effect panic caught while the guard remains
alive leaves uncertainty even without mutex poison; health checks refuse reuse,
and normal guard drop faults it. Pure untouched planning abort preserves versions;
a later abort after an earlier effect cannot restore the operation-entry state.

The caller must keep every write/flush and completion check inside the effect,
propagate failures and never swallow a panic/error into success. Deferred work
returned for later execution is outside that contract. Rust panic controls do
not legalize foreign unwinds, C ABI aliasing, freed handles or raw device escape.
There are no explicit heap reservations, growing containers, filesystem reads
or callbacks inside entry itself; platform mutex internals and caller-owned Arc/
effect allocations remain outside that fixed-state observation. No actual-card
memory/latency or Windows behavior is measured here.

**Integration boundary:** the gate currently has no `Filesystem` binding.
Existing `write_refusal`, `flush` and `Drop` consult direct-commit/journal health,
not this control. Simply wrapping chmod would leave an early marker I/O failure
recorded only in the gate while other legacy methods/close could still proceed.
Bridge uncertain/poison/panic health into sticky mount-level write/flush/clean-
release refusal before using a private adapter for real writes. A newly allocated
control must not reset that mount failure. This is an integration prerequisite,
not a defect in the reviewed standalone component or a public fix by this review.

**Next bounded implementation: mount-bound private chmod adapters.**

1. Give each mounted filesystem one private control and a mount-borrowing
   context. Context construction enters before device/runtime callbacks, lookup,
   generation checks or marker writes. Reject a context from another mount
   before any such access. Reuse that context in private path/inode helpers;
   do not reacquire inside inode or commit helpers. Own/borrow the control safely;
   add no self-referential guard, unsafe lifetime extension or global registry.
2. Add crate-private guarded path-byte and inode chmod dispatch only, sharing
   existing mode/type/ctime/checksum semantics. Preserve existing public Rust/C
   delegates as legacy, unguarded paths at this checkpoint. The private entry
   must retain all read-only/casefold/unsupported-feature and stale-generation
   refusals, acquire before resolution and cover the early clean marker as well
   as the entire commit. Do not call a marker-writing `refuse_write` outside an
   effect; verdict-only checks and raw I/O must be distinguished. Planning
   refusals outside an effect need not become uncertain-write faults.
3. Connect a sticky mount failure latch to uncertain effect/poison/unwind and
   existing write, flush and clean-release refusals. Cover marker failure before
   ordinary commit failure flags are set; refresh/new control cannot heal it.
   A private context must latch on unwind before existing ABI panic handling can
   return. No owner cache, Ready state or automatic builder invocation is added.
4. Test actual private adapters on scratch mounts: overlap/foreign context
   refuses before any read/write/runtime callback; path and generation checks,
   mode/type/checksum/ctime preservation, callback-worker reentry, marker and
   commit write/flush failures, caught unwind, legacy write/flush refusal and no
   clean marking on faulted drop. Preserve dirty/device bytes for pre-I/O refusals.
   Add exact private-adapter Linux/e2fsck/debugfs evidence for emitted chmod
   metadata before qualifying it; older public chmod tests alone are insufficient.

This is a limited private integration step, not public concurrency support.
Other mutators, reads, repair/replay/refresh, C ABI reference creation and handle
lifetime remain unguarded and must remain externally serialized. Keep raw public
fields/device handles outside any proof claim. Review this family before public
wrapper migration; all families and maintenance/bypass paths need coverage before
the gate can justify transaction-current ownership. The ignored deep fuzz case
also remains unresolved broader qualification. The three saved insertion aliases,
dependencies/overlays, buffer seals, durable owner publication and complete map/
collision/growth qualification stay open; every public casefold write gate stays.

Fresh local full units pass 976, zero failed/ignored; baseline/final Counter
multisets are identical (1,223 lines / 79,083 bytes, unchanged caps). Lint,
provenance/shared instructions, whitespace and saved ownership-patch applicability
pass. Source/test/workflow hashes and sibling checkouts remain unchanged; no
source fix is required by this review. Ignored evidence is under
`tmp/casefold-gate-review`. No fresh release CLI build or local VM run is claimed.

### Private operation-control implementation — 2026-10-16

Implemented `src/operation.rs` after the concrete audit, starting from
`c57f0d8fdb5ee1161200121719cc0ea1857b9a72`. Builder source `772981a` and its
review `c3557f5` passed full CI; the audit's full CI was still running at entry.
The primitive is a crate-private prerequisite only: no `Filesystem` field,
public Rust/C ABI wrapper, device adapter, buffer, owner builder or commit path
uses it. It does not fix or reproduce public concurrent-operation gaps. No
public error/flush behavior, write gate or ownership profile changed.

`Control` holds one mutex over fixed scalar status. `try_enter` uses `try_lock`
and returns the existing `Unsupported` error on overlap/reentry, including a
callback waiting for a worker on another thread. Poison and unhealthy state
return corruption errors without recovery/unwrapping the poisoned lock. An
`Operation` borrows its control and owns a non-Send guard; nested helpers take
that context. Independent ownership via `Arc` is a caller choice, not an
allocation or lifetime cache built by this module. No explicit heap reservations,
device I/O, retries, sleeps or growing work/output structures are introduced;
standard synchronization internals remain platform-dependent.

Status separates view validity (`Unbuilt`/`Invalid`) from health (`Healthy`,
`Uncertain`, `Faulted`). There is deliberately **no Ready state or proof
publication API**. A borrowed `Stamp` compares control identity plus epoch and
revision; matching counters from another control refuse. The borrow keeps its
owner alive, preventing address reuse while a stamp exists. It is only a
freshness marker, never ownership/admission evidence. Entry, status access and
an untouched guard drop do not change versions or confer readiness.

`revise` invalidates and advances the control-local revision; `invalidate`
invalidates and advances both epoch and revision. This revision is **not yet a
BlockBuffer revision or seal**. Arithmetic is checked as a pair before counters
are assigned: exhaustion invalidates and faults health without partial advance
or wrapping. Previously issued stamps cannot regain validity. There is no
fault/poison reset API and no connection that clears existing filesystem health
flags or replay refusals. Constructing a different control cannot recover an
existing failed filesystem or grant permission to reuse it.

`effect` invalidates and sets `Uncertain` before invoking external work. The
closure must encompass the complete effect/write/flush attempt and propagate
failures. `Ok` restores healthy control state while leaving validity invalid;
it proves neither physical durability nor storage ownership. Every `Err`
conservatively faults reuse, even if the callback reports failure before a known
physical write. On unwind, guard drop faults and the mutex poisons. A panic
caught while the guard itself remains alive does not poison the mutex; health
stays uncertain and all freshness/change/effect methods refuse. Later normal
drop changes that uncertainty to faulted, so catching the panic cannot heal it.
A later planning refusal after an earlier successful effect cannot restore the
old view; returning an error never rolls status back to its operation-entry value.

Fifteen native controls exercise untouched release without readiness, prompt
same-thread reentry refusal, a callback synchronously waiting for its worker,
eight simultaneous contenders with exactly one held winner/effect, borrowed
nested helpers, matching-version foreign-control refusal, stale stamps across
entry and revision-only changes, invalidation/uncertainty before side effects,
sticky effect failure, caught-panic-with-live-guard, mutex poison, exact last
versions/exhaustion without partial advance or extra effects, and a later
planning refusal after an earlier effect. Channels and barriers coordinate the
actual threads; five-second receive bounds report blocking regressions rather
than relying on scheduling sleeps. These are native synchronization/state
controls, not a Linux filesystem or public concurrent-write qualification.

Fresh full local units pass **976**, zero failed/ignored. Counter multisets
retain all **961** baseline occurrences with exactly 15 operation additions.
The log measures **1,224 lines / 79,142 bytes**, inside unchanged caps. The
focused run also passes 15, zero failed/ignored (54 lines / 3,268 bytes).
Lint passes, including formatting, Clippy with warnings denied, Unicode checks
and provenance. The implementation hashes are retained with logs under
`tmp/casefold-operation-gate`; exact-commit CI and implementation review remain
pending. No fresh independent Linux or Windows concurrency run is claimed.
Locked release CLI installation, shared instructions, whitespace and saved
insertion-red patch applicability also pass. Implementation hashes remain
unchanged after validation; dependency pins/locks and sibling checkouts remain
unchanged. No environment draft or publication change was needed.

**Next bounded checkpoint:** inspect exact-commit full/focused CI, then review
the primitive's identity/lifetimes, exclusion/reentry, panic/error transitions,
checked counters and caller obligations before wrapper migration. The audit's
C ABI aliasing/handle-lifetime constraints, raw-field/device escape paths and
concurrent-flush compatibility remain unresolved. Then choose one explicitly
bounded migration family with context-taking helpers; do not add locks to every
public method or put a second lock around existing ABI delegates. Owner epochs,
transaction-private buffer binding, dependencies/overlays, final seals, durable
publication and commit-failure integration still need implementation/review.
The three saved insertion aliases remain unresolved; public writes, deeper
insertion, collisions, splits/growth and index dropping remain blocked. No
card/Windows/SteamOS or transparent concurrent-access readiness follows.

### Concrete operation serialization/invalidation audit — 2026-10-16

Audited `c3557f57088f73bffb18c33f1ccb009235ace4f7`, equal to the fetched
remote tip with a clean tree. Builder source `772981a` now passed both
[focused VM evidence](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38849841814)
and [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38849841824).
The [review commit's full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38853268523)
also passed; downloaded its actual unit artifact and confirmed 961 executed,
zero failed/ignored. The six-image review below remains applicable. This is a
source/design audit, not an executed concurrency reproduction or a guard fix.

#### Concrete boundaries and bypasses

| Existing path | Observed boundary | Required change before owner proof can authorize writes |
|---|---|---|
| `apply_*_bytes` to `resolve`/parent lookup to public `apply_*_at` or `apply_*_ino` | Resolution, duplicate checks, `live_inode` generation checks and allocation precede `commit_block_buffer`'s journal lock. No journal means no such lock. | Acquire once before the first filesystem read/verdict/marker write; delegate to private helpers carrying the same mount-bound operation context. A commit-only lock is insufficient. |
| `apply_truncate_ino` to grow/shrink; `apply_pwrite_ino` to itself for chunks; zero-range to punch then keep-size | Nested public mutators and several commits in one public operation. | Replace internal public calls with context-taking helpers. Keep one outer guard across every chunk/commit, including failure between zero-range's two transactions; do not imply whole-call crash atomicity. |
| C ABI `fs_ext4_truncate`, `fs_ext4_fallocate`, `fs_ext4_write_file` | Truncate/fallocate resolve in the ABI before inode mutation; write-file resolves/type-checks before the filesystem path method resolves again. | Put all state-dependent resolution/type checks under the same operation as mutation. Add guarded path-level dispatch or an internal context-taking bridge; retain ABI errno/type/name behavior. C pointer/length checks may precede entry. |
| Other C ABI path mutators and `ino_call` delegates | Most directly call filesystem APIs; the opaque handle itself has no operation mutex. | Use the same filesystem boundary as Rust callers; do not independently lock the ABI and then call a locking public method. Preserve generation checks inside the operation. |
| `refuse_write` to `mark_not_clean_once` to `write_superblock_state` | A normal write refusal check can already write/flush the primary superblock before planning. Rename uses a verdict-only check to preserve its no-op behavior. | Cover marker I/O and errors, not only final dirty-buffer commits; retain successful no-op rename semantics. |
| `commit_block_buffer`, `commit_inode_write`, `write_direct`, `AllocationState::publish` | Journal lock starts after planning; allocation clears publish after durable commit while journal remains locked. Hash-map publication can allocate. | Require the operation context, preflight controlled publication capacity, then publish ownership/allocation state only after durability. A post-write error/panic must leave proof unusable. |
| `BlockBuffer::dirty`, `get_mut`, `put`, direct table/bitmap/node edits | A mutable vector reference permits later edits without a typed inode writer; cloned/raw buffers have no epoch, mount identity or revision. | Keep staging transaction-private. Invalidate the final seal on every mutable exposure; later final validation must derive dependencies from final bytes, including shared inode-table blocks and external maps changed without inode edits. Do not accept a caller-supplied numeric revision as a seal. |
| `write_inode_raw`, `patch_bgd_counters`, `patch_sb_counters`; `fsck::audit_with_repair` | Raw inode/descriptor writes bypass the main buffer; repair scans, calls user callbacks and commits several buffers. | Guard the whole repair scan/repair/recheck or refuse it in a proof-owning session. Invalidate before raw writes; no retained proof afterward without explicit rebuild. A nominal read-only fsck C call can request replay first. |
| `journal_apply::{apply,replay_if_dirty,replay_into_cache}`, public journal writer | Public lower-level routines can apply disk changes or pin a new logical view without normal mutators. | Separate public invalidating wrappers from private context-taking replay/writer helpers. Internal commit must not reacquire its own guard. Cache replay also invalidates; a clean-log path can still write recovery markers/counters. |
| mount/recovering mount, orphan recovery, `reload_geometry`, `refresh_metadata`, `fresh_read` | Mount may replay/reclaim before returning; refresh changes metadata/allocation overrides; fresh-read flushes/unpins/invalidates. Orphan recovery commits once per member. | Start unbuilt; build only after all recovery/refresh. Invalidate before maintenance/cache changes, retain the guard across multi-commit recovery and require explicit rebuild after success. Never reset sticky failure merely by refreshing/reopening a writer. |
| label updates, `flush`, `finish`, `Drop` | Label commits primary/backups then changes mounted bytes; flush may fail; finish/drop write clean/recovery state. | Include label and release paths. Revoke proof before close; never clean a faulted mount. Preserve current concurrent-flush and exclusive refresh/unmount contracts until public wiring is separately qualified. |
| Public `dev`, `journal`, `sb`, `groups`, `csum`, `flavor`; retained device handles, adapters and formatting | Callers can bypass an internal mutex, directly alter metadata with an exclusive Rust borrow, drive a journal or write through another handle. | No transparent lifetime cache is justified. A future proof-owning session must control filesystem access and discard proof before returning mutable access; external device exclusivity remains a caller prerequisite. Preserve existing APIs without pretending they enforce this capability. Formatting/standalone device APIs must never run against a live proof-owning device. |

The same coverage is needed for chmod/chown/utimens/flags/xattrs, create/mkdir/
mknod/symlink/link/unlink/rmdir/rename, replacement/truncate/preallocation and
ordinary-volume mutations: changing an inode checksum or unsupported storage
role can stale the proof even without changing a namespace. Read consistency is
a separate obligation. High-level lookup/stat/read/directory/xattr/fsck scans
must join the session boundary or remain externally serialized; raw read helpers
are not a coherent snapshot API. An iterator's collected entries do not remain
live ownership/admission evidence.

C ABI lifetime/aliasing is an additional boundary: replay, fresh-read, label and
fsck wrappers construct mutable filesystem references; unmount frees the handle.
A mutex acquired inside a filesystem method cannot legalize references that
already overlap, nor keep a freed handle alive. Preserve the caller's exclusive
maintenance/unmount contract. Any future handle-level concurrency support must
enter its admission/lifetime control before creating filesystem references and
drain users before destruction; it cannot be inferred from the private gate.
Callback reentry refusal must also happen before conflicting reference creation.

#### Lock and callback decisions

Current edges include journal-to-allocation-state during publication, aligned
device write-lock-to-inner-device during read/modify/write, and cache-state-to-
inner-device during invalidation. Normal cache misses release the state lock
before inner I/O; do not generalize that to every cache method. Runtime clock/
generation providers, device read/write/flush/size/writability methods, C device
callbacks and fsck progress/finding callbacks are externally supplied code.
`ffi_guard` catches Rust unwinds but neither serializes operations nor repairs
an uncertain disk/proof; a panic crossing a C callback is not made safe by it.

For the **first disconnected private gate**, choose nonblocking `try_lock`:
overlap/reentry returns an explicit existing `Unsupported` error before any
filesystem access; poison returns an error and permanently refuses proof/write
reuse. This bounds contention and prevents a callback that synchronously waits
for same-mount work on another thread from blocking behind itself. Thread-local
same-thread detection alone would not cover that case. This is a private
prototype decision, not a change to public concurrent-flush behavior, a new ABI
errno promise or a retry loop. Public wrapper migration must review those
compatibility details and callback tests before connecting the gate.

Use a private mount-bound operation context with one owner; nested helpers take
that context rather than acquiring again. A private independently owned control
handle can keep the guard alive without borrowing fields across `&mut self`
refresh/replay; do not work around Rust aliasing with raw pointers. Use checked
mount epoch/revision advancement; overflow, foreign tokens and stale revisions
refuse. No recursive mutex, lock release around callbacks or hidden bootstrap
scan is allowed. Proposed order is operation gate outermost, journal next when
needed, allocation-state/cache/device locks innermost with their existing edges.
Do not hold allocation-state across journal acquisition. None of these locks
may call a public operation that acquires the gate again. Qualify reader behavior
and cross-mount callback ownership separately; raw external access is unsupported.

#### Invalidation, failure and implementation sequence

Keep proof validity separate from write health. Start unbuilt. Under the gate,
an explicit bounded build may eventually establish a ready epoch. Before any
raw write, replay, recovery, cache/geometry refresh or uncontrolled mutable
exposure, revoke ready status and advance the checked epoch. Success leaves
that proof invalid until explicit rebuild; failure/poison with uncertain I/O
also latches faulted health. Existing `direct_commit_failed`, journal health and
`replay_pending` remain authoritative refusals; new state cannot clear them.
All callback-backed write/flush attempts, including the early clean marker,
must enter the uncertainty boundary before invoking external code.

A pure planning refusal before view/device changes drops local work without
publishing. After a successful earlier chunk, a later planning refusal cannot
restore the operation's original proof: retain the last durably published epoch
only if its complete overlay was already validated, otherwise stay invalid.
I/O/flush failure after an attempted write, or unwind after device changes,
discards proof and refuses writes until recovery/remount. A guard's successful
drop is not evidence of durable commit. For future overlays, prepare owner,
reverse-dependency and allocation-publication capacities before device writes;
hold the guard through final seal, durability and publication. Acquire/restore
no proof after failure by rolling back only an in-memory owner map.

**Next bounded implementation:** a private operation-control/gate primitive and
tests only, disconnected from public methods, owner caching and admission.
Prove at-most-one entry, prompt same-thread and callback-worker overlap refusal,
release after untouched abort, mount identity, checked epoch/revision limits,
invalidation before an injected side effect and sticky poison/uncertain-failure
refusal. Use deterministic barriers/channels and bounded waits, not timing-only
race tests. It must confer no ownership token or readiness on construction.
Then review it before migrating wrapper families and maintenance/raw paths under
the table above. Required later integration controls include pre-resolution
serialization, nested chunk/zero-range calls without relocking, C ABI stale
lookup, shared inode-table/raw-node edits, concurrent read/flush, callback
reentry, repair/replay/refresh, partial commits and publication faults.

No production/test/workflow changed in this audit. Current concurrency gaps
above are source observations/design constraints, not newly executed failing
reproductions. The three saved insertion aliases remain unresolved. Public
casefold writes, deeper insertion, collisions, splitting/growth and index
dropping remain blocked; no Windows/SteamOS/card or concurrency readiness claim.

Fresh baseline/final full units pass 961, zero failed/ignored, with identical
test-occurrence Counter multisets (1,208 lines / 77,820 bytes, unchanged caps).
Lint, provenance/shared instructions, whitespace and saved ownership-patch
applicability pass. Source/test/header/workflow hashes and sibling checkouts are
unchanged. Ignored call inventories, CI evidence and verification logs are under
`tmp/casefold-lifecycle-audit`; the existing handoff records this checkpoint.

### Complete ownership implementation/evidence review — 2026-10-16

Reviewed `772981a50a4624b878c4ccd0ea36df5b2fec0233`, equal to the fetched
remote tip with a clean tree. Its exact-commit [focused VM workflow](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38849841814)
passed. Downloaded the actual artifact and inspected all six owner images,
inventory/owner commands and reports, fixed reports and summaries. The exact
whole-result test executed once, zero failed/ignored (53 lines / 6,627 bytes),
with six kernel populations and six judged non-repairing e2fsck checks. All
seven existing focused fixture/insertion/component logs also pass, one test
each with zero failures/ignored. Source [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38849841824)
is still pending; do not treat focused evidence as its conclusion.

| Block size / descriptor / journal | Allocated IDs | Fixed / data / map claims | Total claims | Allocated blocks | Charged reads / tree records |
|---|---|---|---|---|---|
| 1 KiB / 32 / no | 37 | 11 / 24 / 1 | 36 | 10,299 | 48 / 25 |
| 1 KiB / 64 / no | 37 | 11 / 24 / 1 | 36 | 10,299 | 48 / 25 |
| 1 KiB / 64 / yes | 37 | 11 / 25 / 1 | 37 | 11,323 | 48 / 26 |
| 4 KiB / 32 / no | 37 | 10 / 24 / 1 | 35 | 2,599 | 48 / 25 |
| 4 KiB / 64 / no | 37 | 10 / 24 / 1 | 35 | 2,599 | 48 / 25 |
| 4 KiB / 64 / yes | 37 | 10 / 25 / 1 | 36 | 3,623 | 48 / 26 |

The 16 MiB images have two initialized groups, 64 inode slots, 256-byte bodies
and one primary descriptor block. Each has 20 empty files, one inode with two
hard links, three directories, two symlinks (one fast, one extent-backed), one
FIFO and one depth-one file with an external map and unwritten beyond-EOF
storage. No-journal images have nine inactive reserved slots and 22 regular
files; journal images have eight inactive reserved slots and 23 regular files,
including inode 8's depth-zero 1,024-block journal. Other extent roots are depth
zero. The three 1 KiB typed boot claims are outside the bitmap domain; 4 KiB
primary block zero is inside it and set. No journal descriptor-32 combination
or depth-two map is claimed by this six-profile matrix.

A read-only public-format byte audit independently matched every saved `testi`
answer to the inode bitmap, every identity/type/generation/size/link/sector value
and complete data/map interval to inode/tree bytes, and every fixed range to
primary geometry/descriptor addresses. It checked all **222 allocated records**,
**215 claims** (63 fixed, 146 data, six mapping), sector totals and 40,742 used
blocks. Across all twelve groups, the claim union equals the set allocation bits
and the complement of the saved complete free lists; group/global free counts
agree. All 245,376 inode-padding and 184,323 block-padding bits are set. Both
groups in every image have ITABLE_ZEROED only. Audit scripts/results/image hashes
are ignored local evidence under `tmp/casefold-owner-review`; this audit neither
executes a host oracle nor replaces the CI test's independent tool verdicts.

**Implementation decision:** no blocker found within the standalone read-only
constructor's trusted mounted geometry and coherent stable device/buffer view.
Current fixed-layout/inventory checks precede body interpretation; every allocated
ID is read once, even without names, positive links or storage. Role/depth
restrictions and inode/node checksums precede tree claims. Generation, data/map
roles and sector accounting are retained; global sorting rejects all overlaps
before one allocation check. Staged full nodes have exact size/bounds checked
before copy and bypass failing device reads. Failure drops the whole local
result and neither edits caller/device bytes nor publishes reusable proof.

Budget review checked fixed/inventory/allocation group reservations of `3G` and
metadata read reservations of `3(1+D)+2G`, where `D` is primary descriptor
blocks. Body reads debit the actual census and tree calls receive only remaining
reads/records/global claims, with a separate transient claim ceiling. Successful
usage is subtracted before the next inode; a failed call aborts instead of
resetting allowances. In the matrix, `G=2`, `D=1`, so metadata reserves ten reads,
37 bodies consume 37 and one map consumes one: 48 is a conservative logical
fetch charge, not a measured syscall count. Record totals match all declared
root/child entries, including empty roots that contribute zero entries.

Requested-byte review checked `8 * block_size + 256` scratch/checksum bytes,
inventory and retained inode capacities, fixed/global/allocation-conversion
capacities and one transient tree capacity. The sum conservatively includes
phases that do not coexist. Checked arithmetic and fallible exact reservations
precede filling; global pushes are claim-bounded, tree capacity is dropped per
inode and sort is in-place. These bounds do not include allocator overhead,
other scalar stack or shared cache/pinned replay memory. No actual-card RSS,
latency or production defaults are established.

The joined Linux evidence is an initialized ordinary, non-casefold, non-FLEX,
non-sparse layout matrix without attributes. Existing component evidence and
synthetic controls cover other narrow feature/current-view cases; this review
does not newly qualify every allowed feature combination. Staged corruption,
zero-link ordinary records, conflicts and exhaustion remain synthetic controls.
Unclaimed allocation is never permission to acquire storage, even though these
clean images happen to account for every used bit. The constructor carries no
epoch or dependency closure and cannot satisfy its stable-view precondition
against concurrent callers or retained external device handles by itself.

The eighteen constructor controls refuse the known shapes, but all three saved
insertion-level aliases remain unresolved because admission is disconnected.
Keep public casefold writes, deeper insertion, collisions, splits/growth and
index dropping blocked. Source/test/workflow hashes are unchanged by this
review. No real card, Windows or SteamOS qualification follows.

Fresh local full units pass 961, zero failed/ignored, retaining the exact baseline
Counter multiset with no additions/losses (1,208 lines / 77,820 bytes under
unchanged caps). Lint, provenance/shared instructions, whitespace and saved
ownership-patch applicability pass. No source, tests or workflow changed; the
source's release CLI build remains prior evidence, with no new local VM run.
Sibling checkouts remain clean. The existing handoff records the continuation.

**Next bounded checkpoint:** first resolve outstanding exact-commit full CI,
then audit operation serialization and mutation/invalidation paths before adding
the guard or publishing an epoch. Produce a concrete call-path/lock-order plan
for path and inode entry points, direct writes, raw buffer edits, replay/recovery,
refresh, callbacks/reentry and failed commits. Identify guarded internal helpers
so wrappers do not relock; acquire the operation guard before path resolution
and all allocation/cache/journal locks, hold it through durable publication, and
invalidate before bypass/raw mutation. Define stale epoch/buffer revision and
poison/failure refusal rules. Add no mounted-only cache, per-insertion full scan
or insertion admission while the mutation audit/lifecycle is incomplete. The
earlier transaction-current design below remains the contract; this checkpoint
is its concrete serialization/invalidation audit, not permission to skip overlays,
reverse dependencies, final sealing or failure qualification.

### Private complete current-view ownership implementation — 2026-10-15

The bounded constructor in `src/ownership.rs` joins the reviewed fixed layout,
current allocated-inode inventory, strict inode-role reader, complete extent
collector and one final current-allocation verification. It requires one trusted,
coherent stable device/buffer view throughout the call. It returns a complete
local result or an error; it publishes no cache, epoch, dependencies, transaction
overlay or write admission. Retained external device handles are outside Rust's
borrow guarantee, so this precondition still needs the later operation guard.

Every allocated ID is visited once, including inactive reserved, empty and
zero-link records. The result retains inode/generation, role, storage form,
size, sector total and link count. Hard links do not duplicate ownership. Root
and the declared journal must be allocated and classified correctly. The common
profile requires metadata checksums, 1/4 KiB blocks, 32/64-byte descriptors,
256-byte Linux inode bodies, initialized inode and touched block bitmaps, and
extent depths zero/one (journal zero). Unknown flags, attributes, legacy maps,
RESIZE_INODE, unsupported roles/depths and malformed current metadata refuse.

Claims distinguish fixed role/group from inode/generation/role and data versus
mapping storage. Whole trees retain unwritten and beyond-EOF allocation. The sum
of data and mapping blocks must match the inode's sector count under the
non-HUGE profile. One in-place global sort rejects every overlap, including
fixed/journal and same-inode data/map conflicts. One allocation check covers all
claims except the typed 1 KiB boot reservation; 4 KiB block zero is checked
normally. Unclaimed allocated bits do not establish free space or authorize
acquisition. The aligned external-node adapter checks filesystem bounds and
exact staged length before copying; staged nodes bypass underlying disk reads.

Shared limits cover three conservative group-phase reservations, full inode
bitmap probes, allocated IDs, global claims, total tree records, metadata/node
reads, allocation bitmap work and requested memory. Metadata phases reserve
their conservative read maxima up front; body reads debit the actual census
length. The extent collector now exposes successful read/record consumption,
and each next tree receives only the remaining global allowance plus a separate
per-inode transient claim ceiling. Any failure aborts the entire invocation;
there is no retry with a fresh per-inode allowance.

Checked requested-byte accounting sums inventory, retained inode identities,
fixed ranges, global claims, allocation-input conversion, one transient tree
and all component scratch (including the explicit inode checksum stack copy).
This deliberately conservative sum even includes nonoverlapping phases. All
direct vector reservations are fallible and fixed; each transient tree is
consumed/dropped before the next inode. This is not measured RSS or a card-size
default: allocator overhead, other scalar stack use, shared cache and pinned
replay allocations remain outside the bound. Global sorting is bounded by the
explicit claim count; runs are never expanded into per-block owner entries.

Four initial constructor controls used a baseline adapter that collected each
inode separately without global conflict checking: the three known cross-owner
shapes failed on acceptance, while the disjoint control passed. Through the
builder all four pass. Eighteen expanded controls cover complete typed output,
exact/exhausted shared budgets and overflow, zero-link/empty/reserved IDs,
hard-link identity, fixed/same-inode/journal conflicts, missing root/journal,
sector totals, staged child relocation/conflict, allocation/uninitialized flags,
unsupported bodies, malformed staged blocks/checksums, generation and unwritten
beyond-EOF storage, device failure and staged precedence over a failed read.
The first red attempt had a compile-only test helper typo; the recorded red
result is the subsequent executed run, not that compile error.

The final full local tier passes **961**, zero failed/ignored; a Counter multiset
retains all **943** baseline occurrences with exactly 18 builder additions.
The log measures 1,209 lines / 77,879 bytes under unchanged caps. Lint, locked
release CLI installation, provenance/shared instructions, whitespace and saved
insertion-level red patch applicability checks pass. Validated source/guest/
workflow hashes remain unchanged and core/Windows siblings remain clean.

A new VM-only whole-result oracle creates six initialized Linux profiles:
1/4 KiB times 32-byte/no-journal, 64-byte/no-journal and 64-byte/journal. It
populates both groups and includes empty files, a hard link, directories, short
and long symlinks, FIFO, external maps and unwritten beyond-EOF storage. Judged
e2fsck must be clean. Complete independent `debugfs testi` answers determine the
census; `stat` determines each identity/type/size/link/sector value and every
data/map range; `dumpe2fs` determines typed fixed ranges and used-block totals.
The entire sorted owner vector and every allocated inode record are compared,
not merely aggregate bits or our own logical lookup. Missing/duplicate report
answers fail. CI requires exactly this test with an executed-test floor of one
and retains images, commands, raw reports and summaries. It compiles locally;
execution/evidence review is pending the exact new commit's CI. No local VM or
host oracle execution is claimed.

**Next bounded checkpoint:** inspect that exact-commit focused/full CI, diagnose
any failures, download all six owner fixtures/reports and review complete output,
resource accounting and the constructor contract. The allocation source's
`72b6911` focused/full CI passed; this does not qualify the new joined builder.
Only after this review should operation guard, epoch/dependencies, overlay,
sealing/invalidation, publication and commit-failure integration be attempted.
The three saved insertion-level cross-inode failures remain unresolved because
this constructor is disconnected. Public casefold writes, deeper insertion,
collisions, splitting/growth and index dropping remain blocked. No real card,
Windows or SteamOS qualification follows from these constructor controls.

### Allocation verifier implementation and evidence review — 2026-10-15

Reviewed `72b6911d4d265ce1339c56caac1f9330a62099ef`, including the production
verifier, twenty storage controls, two report-reader controls and repaired VM
oracle. Exact-commit [focused VM evidence](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38795846343)
and [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38795846219)
passed. Downloaded the actual artifact and inspected all four allocation images,
command/bit/free-list reports and summaries, plus current insertion, FIEMAP,
inventory, role and fixed-metadata logs. The allocation test executed once,
zero failed/ignored (37 lines / 3,890 bytes). The production verifier remains
byte-identical to `1fcb361`; no source/test/workflow fix is needed by this review.

| Block size / descriptor | Domain `[first, end)` | Allocated / free | Last group's valid blocks | Allocated runs |
|---|---|---|---|---|
| 1 KiB / 32 | `[1, 2044)` | 1,573 / 470 | 1,019 | 2 |
| 1 KiB / 64 | `[1, 2044)` | 1,573 / 470 | 1,019 | 2 |
| 4 KiB / 32 | `[0, 511)` | 401 / 110 | 255 | 2 |
| 4 KiB / 64 | `[0, 511)` | 401 / 110 | 255 | 2 |

These are **5,108 complete domain classifications across eight groups**.
All 5,106 supported nonzero `debugfs testb` answers match complete `dumpe2fs`
free lists; the two 4 KiB zero classifications come from those complete,
count-checked lists. Both zero bits are actually set, and the verifier checks
them normally. Each singleton and all allocated runs together agree; free-range
and 1 KiB boot-domain refusals remain. The raw public-format audit additionally
matched every saved bit/list answer to image bitmap bytes and checked all 158,732
padding bits. Both bitmap groups have ITABLE_ZEROED only in every profile.
Each image has 32 allocated IDs, 256-byte inodes, nine zero-mode reserved records,
two directories and 21 regular files; all 23 extent headers are depth zero.
Those are candidate initialized fixtures for a joined builder, not independent
qualification of the inventory/role/tree join on these images.

**Decision:** no blocker found within the standalone verifier's narrow profile
and trusted coherent stable-view/fixed-layout preconditions. Bounds and geometry
precede interval arithmetic/slicing; ordered disjoint ranges make group progress
monotonic. Partial-byte masks check exactly the proposed bits, including zero.
Current superblock/descriptor identity and checksums, stable pointers, staged
sizes/precedence, known/pending flags and initialized bitmap requirements are
enforced. CRC coverage/width and full-block padding agree with the independent
matrix, including non-byte-aligned final group ends. One shared set of limits
charges groups, fetches and bitmap work before access. Two fallibly reserved
blocks are reused; no payload-block output or growing bitmap cache is produced.
Errors preserve buffered/device bytes and publish no partial proof.

Scope remains important: the independent positive matrix has two groups and two
separate allocated runs each; a single allocated interval crossing a group
boundary remains synthetic (the 69-group control). Staged corruption, exhaustion,
unknown/uninitialized flags and malformed inputs also remain synthetic controls.
Two-block scratch bounds the verifier's own requested allocation, not allocator
overhead, shared device-cache allocations/pinned replay bytes, other components
or card-wide RSS. The qualified fixed layout must come from the same coherent
view; this function does not re-prove every fixed-role conflict. A successful
empty input proves no coverage. Allocation bits establish no exclusive owner,
snapshot epoch, transaction lifecycle or public-write readiness. The three saved
other-inode aliases remain unresolved.

#### Next bounded implementation: complete private current-view ownership builder

Join the already implemented prerequisites into one read-only, bounded complete
owner result, disconnected from admission, caching and public writes. Require a
trusted coherent stable device/buffer view; the later operation guard/epoch/
overlay/invalidation design remains mandatory before any result authorizes writes.

- Enforce the intersection of qualified component profiles: current fixed layout,
  initialized inode and touched block bitmaps, 256-byte Linux inode bodies,
  supported reserved/ordinary roles and qualified extent depths zero/one (journal
  depth zero). RESIZE_INODE, unknown storage/attribute/legacy profiles and deeper
  maps refuse. Run a complete current inventory; visit every allocated ID once,
  including reserved and zero-link records, and require the root/declared journal
  outcomes. Names and link counts never substitute for that census.
- Keep fixed roles and every inode/generation/storage role explicit. Collect each
  whole extent tree, including mapping nodes and unwritten/beyond-EOF storage;
  check sector totals under the supported non-HUGE inode profile. Fast symlinks,
  inactive reserved slots and FIFO/no-storage roles contribute no block claims
  only after their role checks. Hard links never duplicate an inode's ownership.
  Sort one bounded physical claim arena and reject every forbidden overlap,
  including different roles of one inode and conflicts with fixed/journal roles.
- Verify all claimed intervals against current allocation in one shared bounded
  invocation after separation. Keep only the 1 KiB typed boot reservation outside
  that domain; 4 KiB primary block zero must pass the normal bit check. Unclaimed
  allocated blocks are never proof of free space or permission to acquire storage.
  Any unknown owner, missing coverage, malformed body/tree, allocation conflict,
  I/O failure or resource exhaustion discards the entire local result.
- Share budgets across the entire build. The role reader already shares limits
  across records; extent collection currently resets private read/record counters
  per call and does not return usage. Add the minimal private accounting needed
  to debit successful walks from shared remaining budgets; bound each call by
  remaining limits and abort the whole build on failure. Never silently reset a
  full per-tree allowance for every inode. Keep any explicit per-inode ceiling
  separately documented, conservative and tested.
- Charge retained inventory/fixed/owner capacities, transient tree capacity,
  any allocation-input conversion vector and reused role/extent/allocation
  scratch at their peak coexistence. The extent
  collector reserves its full claim cap; copy into the bounded global arena and
  discard transient capacity before the next inode, or account for every retained
  capacity explicitly. Use checked arithmetic/fallible reservations and bound
  global sort/overlap work. Requested-byte limits exclude allocator/device-cache
  overhead; do not present them as measured card defaults or an RSS bound.
- Prove the private builder rejects the three known owner-alias shapes plus
  fixed/journal/same-inode conflicts, including staged-only ownership changes;
  preserve a disjoint control, zero-link/reserved/empty-file coverage, staged
  bitmap/map/body changes and exact/exhausted cumulative limits. These constructor
  checks do not fix the still-disconnected insertion path's saved failures.
  Require independent whole-result evidence on initialized Linux fixtures,
  comparing complete owner ranges/identities/storage accounting, not only total
  used bits or this driver's own lookups. Then review the builder before any
  operation guard, dependencies, overlay, sealing, publication or commit-failure
  integration. No whole-card scan per insertion or mounted-only owner cache.

### Allocation-oracle block-zero repair — 2026-10-15

Follow-up: repair `72b6911` passed focused VM and full CI. The review above
supersedes the pending repair validation and selects the private joined builder.

At source `1fcb361`, focused CI and both full library executions failed at
`debugfs testb 0`: the tool refuses zero even when it belongs to the filesystem's
allocation domain. The verdict reader correctly rejected its zero-status failed
request. Two 1 KiB profiles completed (2,043 answers, 1,573 allocated blocks and
two allocated runs each); 4 KiB/32 formatted/populated and passed judged e2fsck
before the failure, and 4 KiB/64 remained unrun. This is an oracle-query defect;
it does not establish a production allocation defect or qualify all four profiles.
CI links, actual artifacts and scope are recorded in `extprogress.md`.

The test-only repair keeps every nonzero `debugfs testb` answer and compares it
with complete `dumpe2fs` free-block lists. The report reader requires every group
in order with exact filesystem bounds, one explicit list each (including empty
lists), matching group/global free counts and no duplicate/outside addresses.
The complete set also supplies zero's actual allocation classification. All
valid singletons, coalesced allocated runs, the free-range refusal and block-zero
domain checks remain. No zero-is-allocated exception, verdict relaxation, test
skip or production change is introduced. Saved `.free-blocks.txt` reports join
the existing focused artifact; the exact filter and one-test floor remain.

Two local controls use a captured tool report and synthetic alterations to pin
zero's classification and reject incomplete/inconsistent reports, including an
empty list without an explicit zero count. The capture's uninitialized metadata
groups qualify report syntax only, not the allocation verifier's positive profile.
The production verifier and existing twenty controls remain unchanged; local
counts and new independent CI status are maintained in `extprogress.md`.

**Next:** require exact-commit focused/full CI and inspect complete actual
allocation/free-list evidence on all four profiles, then finish the verifier's
implementation/resource review before selecting the complete owner join. The
three saved cross-inode aliases and all aggregate lifecycle/write gates remain.

### Private bounded current block-allocation intervals — 2026-10-15

Follow-up: initial independent CI failed on the oracle's block-zero query; the
test-only repair above supersedes that query. Complete VM qualification and the
implementation/evidence review remain outstanding.

`block_allocation::verify` now checks physically ordered, disjoint, nonempty
intervals directly against current bitmap bytes. It requires a trusted coherent
stable mount/buffer view and a successfully qualified fixed layout on that same
view. It neither re-proves every fixed-role separation nor builds/caches/publishes
an owner snapshot, and remains disconnected from insertion/public write admission.
The existing insertion helper is unchanged. A successful empty input proves no
claim coverage; it still verifies the current superblock.

The new component validates ordinary allocation geometry, group/inode coverage,
the full filesystem extent, 1/4 KiB blocks and 32/64-byte descriptors before
traversal. Unknown feature/geometry profiles refuse. Current primary superblock
CRC and identity are checked, allowing only existing free-counter differences.
Each touched current descriptor has its CRC, all three stable metadata pointers,
known flags, pending initialization agreement and bitmap/table bounds checked.
Non-FLEX placement must remain in its owning group. BLOCK_UNINIT refuses before
bitmap access, even if its staged bytes are present. INODE_UNINIT/ITABLE_ZEROED
are known flags but do not themselves make block bits unspecified.

Every current bitmap fetch has exact-size checks, descriptor-width checksum
truncation over `blocks_per_group / 8` bytes, and full-block padding validation.
This initial profile requires a byte-aligned blocks-per-group count. Every bit
after the group's actual filesystem end, both inside checksum coverage and beyond
it, must be one. Ranges never admit such padding. Boot block zero at 1 KiB is
outside the bitmap domain and remains separately protected; superblock block zero
at 4 KiB is an ordinary proposed bit and is checked without an exception.

One invocation shares range/group/logical-read/bitmap-work/scratch caps. Two
fallibly reserved blocks are reused: one descriptor block and one bitmap.
Ordered ranges retain only the last group bitmap and descriptor block, so nearby
ranges share reads without a growing group cache. No per-data-block vector or
owner entry is generated. Bitmap work charges two full block sizes per touched
group before checksum/padding work, then the bytes covered by each range/group
segment, including repeated edge bytes. Reads debit before staged/device access;
all errors discard the local proof and preserve caller/device inputs. The future
builder must still charge this scratch alongside all coexisting retained capacity
and enforce a whole-view lifecycle; per-component limits do not establish that.

The initial baseline adapter to the old helper accepted a corrupt current
superblock and clear padding beyond checksum coverage; its healthy control
passed. Those same three assertions pass through the new component. Expanded
controls cover current allocation/freeing, both bitmap/descriptor CRC sources,
superblock identity/counters, malformed staged sizes, pointer relocation, flags,
partial-byte edges, block zero, final-group padding, both checksum widths,
unsupported geometry, invalid/overlapping intervals, FLEX placement, metadata
aliases/table bounds, injected reads, empty input and exact/exhausted shared caps.
A full 69-group interval exercises two primary descriptor blocks with no payload
block expansion. Fresh counts/results are maintained in `extprogress.md`.

The new VM oracle is compiled locally but unrun here (no KVM). It formats four
profiles (1/4 KiB, 32/64-byte descriptors), makes Linux initialize both allocation
groups, checks judged non-repairing e2fsck and the fixed-layout prerequisite, then
compares each valid singleton against an explicit `debugfs testb` answer. It also
checks every coalesced allocated run together, free-block refusal, partial final
groups and the block-zero domain. Images, command files, complete reports and
summaries are retained by the focused workflow; its exact filter and one-test
floor prevent a zero-test success. New-source independent CI remains pending.

**Next bounded checkpoint:** inspect exact-commit focused/full CI and actual
allocation reports/images, resolve any failure, then review current-view, byte
masks/padding, bounds, profile intersection and aggregate resource contracts.
Only afterward choose the complete ownership join of fixed roles, every typed
allocated inode's full tree and verified current allocation. Cross-owner
conflicts, complete claim coverage, peak budgets, guard/overlay/dependencies/
sealing/invalidation and commit failures remain separate gates. The three saved
other-inode aliases remain unresolved and every public write gate stays blocked.

### Fixed-metadata implementation and evidence review — 2026-10-15

Reviewed `95cb21810b0e43edfbabcaf1b7d55e5d83d23787`, the complete private
collector, seventeen controls and VM oracle/workflow. Its exact-commit
[focused VM validation](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38771333034)
passed. Downloaded the actual artifact and inspected every saved metadata image/
location report plus metadata/role/inventory/insertion/FIEMAP logs. The new
metadata test executed once, zero failed/ignored (59 lines / 7,385 bytes), across
**14 profiles, 196 groups and 943 protected ranges**. Every format and judged
non-repairing e2fsck check passed, and the complete typed/grouped interval list
matched `dumpe2fs`, not just its size. Its full source CI and the subsequent
documentation review's full CI now passed; links are in `extprogress.md`.

Role, inventory, insertion and FIEMAP tests each executed once, zero failed/
ignored (41 lines / 4,671 bytes; 33 / 3,357; 133 / 7,954; 23 / 1,601).
The source hashes match the final locally validated implementation. No production
fix or new test is introduced by this review.

| Block size / descriptor | Normal-profile groups | Dense ranges | Classic sparse ranges / reserved GDT blocks | Sparse-super2 ranges |
|---|---|---|---|---|
| 1 KiB / 32 | 8 | 41 | 40 / 255 | 31 |
| 1 KiB / 64 | 8 | 41 | 40 / 256 | 31 |
| 4 KiB / 32 | 8 | 40 | 39 / 63 | 30 |
| 4 KiB / 64 | 8 | 40 | 39 / 127 | 30 |

Dense layouts have superblocks in all eight groups, classic sparse in 0/1/3/5/7,
and sparse-super2 in 0/1/7. In each FLEX_BG profile, six of eight inode tables
start before their own group. Two further dense 64-byte-descriptor profiles have
two-block descriptor runs: 32 groups/161 ranges at 1 KiB and 68 groups/340 ranges
at 4 KiB. A public-format raw audit additionally checked group coverage, every
bitmap/table address and full table span, bounds and disjointness against the
saved reports. These are small-image compatibility observations, not card-wide
throughput or memory measurements. Partial final inode-group coverage remains
synthetic; the 1 KiB images do independently exercise a partial final block group.

**Decision:** no implementation blocker found within the standalone fixed-layout
profile and trusted, coherent, stable-view preconditions. The collector is a
private prerequisite for ownership construction; it proves no allocation bits,
inode/extent ownership, epoch or transaction lifecycle. Most saved metadata
profiles have INODE_UNINIT/BLOCK_UNINIT groups, and some have RESIZE_INODE; they
are not automatically valid positive fixtures for the narrower inode inventory/
role reader or a future allocation checker. Keep the qualified profile
intersection explicit. The three saved cross-inode aliases remain unresolved.

The review checked:

- Supported sizes, nonzero bounded geometry, group/inode coverage and filesystem
  extent checks precede slicing/traversal. The u32 group bound and bitmap-sized
  group limit bound multiplication; range ends/group heads use checked bounds.
  The primary geometry determines full backup descriptor/reserved runs and full
  inode tables. It never treats unused slots or initialization flags as missing
  physical reservations. Boot and superblock protection do not duplicate a block.
- Current superblock/descriptor CRCs, identity and pending flags are checked;
  staged copies take precedence and require exact size. Relocations refuse.
  Backups are protected without trusting their stale contents as current state.
  FLEX_BG ranges are checked globally, other placements against their own group;
  physically sorted adjacent intervals detect all fixed-role conflicts.
- One group/read/range/scratch limit set bounds the entire invocation. Fetches
  debit before access, shared descriptor blocks reuse scratch, and both fixed
  reservations are fallible with checked combined byte arithmetic. In-place
  unstable sorting needs no secondary range vector. Limit/read/validation errors
  discard local output without mutating the caller or publishing a partial proof.

#### Next bounded prerequisite: current allocation of storage intervals

Implement a private bounded verifier for proposed physical intervals before the
complete owner join. The existing private insertion helper checks individual
blocks with a growing bitmap cache; it is not an aggregate-bounded interval
verifier. Preserve its existing behavior and keep the new component disconnected
from admission. Require a trusted coherent stable current view and an explicitly
qualified current layout; no mounted-only bitmap cache or reused old census.

Validate input intervals, filesystem/group coverage and ordering, current
superblock/descriptor identity/checksums, initialization/unknown/pending flags,
bitmap bounds/sizes and descriptor-width checksum truncation. Verify every
proposed block's current allocation directly against bitmap bytes/words without
expanding payload runs into owner entries. Define and independently qualify
final-group/padding/checksum coverage. Initially refuse BLOCK_UNINIT before
interpreting unspecified bytes; the fixed collector's acceptance of its physical
location is not permission to invent allocation bits.

Use shared limits for intervals, groups, metadata reads, bit/word work and scratch
across the whole invocation; charge staged copies and failed accesses. Reuse
bounded buffers, refuse unsupported/exhausted states without mutation or partial
publication, and charge coexistence with the other components in the eventual
builder. One-KiB boot block zero has no allocation bit: retain its separate typed
fixed protection, but reject generic proposed intervals below `first_data_block`.
At 4 KiB superblock block zero is in the allocation domain and must be checked.
Do not introduce a generic "zero is allocated" exception for data claims.

Tests must cover staged bit changes/CRCs/flags, malformed sizes/pointers,
cross-group intervals and boundaries, exact/exhausted limits, checksum widths,
unknown initialization and unchanged inputs on failure. Require independent VM
allocation answers on initialized positive profiles; the saved uninitialized
metadata fixtures may support refusal controls but cannot silently become
positive allocation proofs. Then review this verifier before the complete join
of fixed roles, all typed inode/tree claims and current allocation. Cross-owner
conflicts, shared peak budgets, complete-result coverage, operation guard/overlay/
dependencies/sealing and invalidation/commit failure rules remain separate gates.

### Private bounded fixed-metadata interval collector — 2026-10-15

Follow-up: source `95cb218` passed focused VM location and existing Linux tests;
the implementation/evidence review above supersedes the pending focused status
and review task. Full CI status is maintained in `extprogress.md`.

The next component after the role review is implemented in `fixed_metadata`.
It reads a current checksum-valid primary superblock and every current primary
descriptor under one stable trusted device/buffer view. Superblock bytes match
mounted contents except free counters; all three descriptor addresses preserve
mounted locations. Descriptor checksums, unknown flags, pending initialization
flags/group bounds and exact staged sizes are verified. The existing mounted
boolean metadata helper is unchanged and no mutation path calls this collector.

Its explicit feature/block/descriptor profile supports metadata checksums,
1 KiB/4 KiB blocks, 32/64-byte descriptors and no META_BG/bigalloc. Unknown
layout features and non-Linux creator profiles refuse. It enumerates one-block
superblocks, complete primary/backup descriptor runs and declared reserved-GDT
space according to dense, classic sparse or sparse-super2 placement. At 1 KiB
the boot block is separate; at 4 KiB primary block zero protects both boot bytes
and the superblock without a duplicate claim. Invalid sparse-super2 backup IDs,
duplicates and group heads that exceed even a partial final group refuse.

Every group's block/inode bitmap locations and full declared inode-table span
are protected, including unused inode slots in a partial final inode group.
FLEX_BG permits placement outside the owning group, but all ranges must fit the
filesystem; without it each descriptor's ranges must fit its own group. Physical
sorting rejects overlaps between any fixed roles, including cross-group tables,
instead of allowing insertion to replace another role. Backups are protected
from the verified primary geometry; their potentially stale contents are not
used as current authorities or required to match current counters.

Known INODE_UNINIT/BLOCK_UNINIT/ITABLE_ZEROED flags retain all physical
reservations. This collector does not read bitmap contents, inode bodies or
journal storage. Allowing these flags or resize-inode layout reservations here
does not qualify the narrower inode inventory/role profile, allocation policy
or resize inode ownership. A complete builder must use the intersection of its
qualified component profiles and keep the typed journal/extent path counted once.

One invocation-wide group limit bounds traversal; logical fetches (staged copies
and failures included), output ranges and two-block scratch have explicit caps.
Adjacent groups share the cached descriptor block. Checked allocation arithmetic
covers `range_cap * size_of::<Range>() + 2 * block_size`; both vectors reserve
fallibly and do not grow during traversal. Sorting uses bounded in-place
`O(R log R)` work. Retained capacity, allocator overhead, device cache and later
inventory/role/tree/dependency storage still need aggregate builder accounting.
Any error drops the complete local result without changing buffer/device or
publishing an epoch/snapshot; no card-wide resource defaults are qualified.

Before implementation, three controls failed on acceptance through the mounted
boolean helper: corrupt current descriptor, checksum-valid changed superblock,
and overlapping current fixed roles across groups. A healthy table control
passed. The same assertions now pass against the collector; the cross-group
control explicitly permits FLEX_BG before requiring an overlap error. Seventeen
focused controls cover complete typed/disjoint output, cross-group placement,
full partial-group tables, shared/exhausted limits, malformed sizes, all pointer
relocations, superblock CRC/counters, aliases/bounds/overflow, unknown profiles,
initialization flags/pending state, three backup policies/reserved heads,
multiple descriptor blocks, truncated heads and read errors. Synthetic positive
fixtures were corrected when overlap and uncleared descriptor bytes were caught;
they are not independently clean filesystem claims.

A new independent VM oracle prepares 14 Linux-tool-created profiles: both block
sizes/descriptor widths across dense, classic sparse with reserved GDT/FLEX_BG,
and sparse-super2/FLEX_BG layouts, plus larger multi-descriptor profiles at each
block size. It requires judged non-repairing e2fsck cleanliness and compares the
entire typed/grouped interval list with saved `dumpe2fs` location reports. It
requires every group report and the intended reserved, cross-group and extended
descriptor shapes. The focused workflow enforces one executed test and saves
images/reports/log. It is compiled but unrun locally; exact-commit CI is pending.

The source/resource self-review found no blocker in this standalone fixed-layout
contract. See `extprogress.md` for final validation. Current block allocation,
all inode/extent owners, cross-owner conflicts and one coherent epoch remain
unproved; the three saved other-inode alias failures remain unresolved. Inspect
new CI and review this component before complete ownership construction, with
aggregate budgets, dependency/overlay/guard/sealing and commit-failure contracts.
Keep every existing operation/recovery/write gate; do not connect admission yet.

### Inode-role implementation and evidence review — 2026-10-15

Follow-up: the private fixed-metadata collector above implements this review's
next component. Its new source still needs its own independent CI and review;
it does not implement the complete ownership join or lifecycle.

Reviewed `ef1ca24d89a2b38a0e3a035c63466628bf3daceb`, its complete private
reader, seventeen controls, VM fixture and focused workflow. Both exact-commit
[focused Linux validation](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38691730821)
and [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38691730797)
passed, including both VM suites and the aggregate gate. The preceding review
`d650d8e` also passed full CI. Downloaded the actual focused artifact and read
the role/inventory/insertion/FIEMAP logs plus six role images/command files/reports.
The new role test executed once, zero failed/ignored (41 lines / 4,671 bytes).
Inventory, insertion and FIEMAP each executed once, zero failed/ignored (33 lines /
3,357 bytes; 133 lines / 7,954 bytes; 23 lines / 1,601 bytes respectively).

Each role image contains 37 allocated records and exactly one saved type/sector
answer per record: **222 records across six profiles**. A public-format raw audit
also matched command IDs to bitmap allocations and sector fields to the reports.

| Block size | Descriptor bytes | Journal | Inactive reserved | Ordinary regular | Journal records | Directories / symlinks / FIFO |
|---|---|---|---|---|---|---|
| 1 KiB | 32 | No | 9 | 22 | 0 | 3 / 2 / 1 |
| 1 KiB | 64 | No | 9 | 22 | 0 | 3 / 2 / 1 |
| 1 KiB | 64 | Internal | 8 | 22 | 1 | 3 / 2 / 1 |
| 4 KiB | 32 | No | 9 | 22 | 0 | 3 / 2 / 1 |
| 4 KiB | 64 | No | 9 | 22 | 0 | 3 / 2 / 1 |
| 4 KiB | 64 | Internal | 8 | 22 | 1 | 3 / 2 / 1 |

Inactive reserved records have zero extra-section size and low-only checksums;
the remaining records have 32-byte extra sections/full checksums. Short symlinks
contain eight bytes with zero sectors. Long symlinks contain 100 bytes with a
depth-zero map and 2/8 sectors respectively. Each sparse external-map file has
depth one and 34/136 sectors, accounting for 16 data blocks plus one map block.
Internal journals are inode 8 with depth-zero maps: 1 MiB/2,048 sectors at 1 KiB,
and 4 MiB/8,192 sectors at 4 KiB. These are observations of independently checked
small images, not performance measurements or a general journal/layout matrix.

**Decision:** no implementation blocker found within this standalone role
reader's narrow profile and same-view inventory/trusted-mount preconditions.
No production fix or new test is needed for this review. The reader may serve
as a private interpretation component in a later complete builder. It does not
establish allocation, complete map validity, exclusive ownership or an epoch.
The three saved cross-inode alias failures remain unresolved.

The source review checked that validated 256-byte inode geometry bounds body
slices, supported block sizes bound scratch arithmetic, and table addresses are
checked before fetches. Staged blocks must have exactly one block's length.
Parsing follows fixed fields only; the fixed checksum copy clears precisely the
slots present under the admitted extra-section policy and salts with inode and
generation. No checksum-verified pointer is followed by this component. Attribute,
fragment, flag, type and unused-byte refusals precede a storage-free outcome;
reserved/fast-symlink/FIFO records cannot silently discard nonzero storage fields.
Shared record/fetch counters charge before the respective work, including errors,
and only the private scratch/counters change. Returned records allocate no heap.

#### Requirements carried into complete ownership construction

1. **Prove coverage and a single view.** The reader neither validates allocation
   nor prevents reuse of an old census. Establish current inventory/descriptor
   identity in one stable epoch, visit every allocated ID once, retain zero-link
   records and require the root/declared journal outcomes. Unknown roles or any
   partial failure discard the whole construction. Rust borrows alone do not
   exclude external writes through retained device handles or commit/replay.
   Establish cache coherence/exclusive access or an explicit invalidating refresh;
   pausing outside writes afterward does not refresh previously cached blocks.
2. **Complete every storage interpretation.** An admitted extent header is not
   a verified tree. Run the complete collector in the same view for every
   extent-backed role, including unwritten/beyond-EOF ranges and every map node;
   check allocation and sector accounting before publication. Keep journal claims
   typed and counted once, distinct from fixed layout metadata. Preserve depths
   zero/one and the narrower journal profile; do not extend them from synthetic
   depth tests. Attributes, resize/quota/orphan layouts, devices/sockets and
   unknown flags remain refused until their complete interpretation is qualified.
3. **Charge coexistence, not only scratch.** The reader's reusable block plus
   checksum copy is only metadata scratch. Charge retained inventory capacity,
   this buffer, typed owner/claim arenas, transient tree capacity, dependencies,
   sorting/overlap work and device/cache exclusions explicitly. Every primitive's
   limits are local; budget their sums/peaks across the whole builder. Repeated
   table fetches remain charged even when cached. No card-wide defaults or
   per-insertion full scan is qualified by these tests.
4. **Keep independent evidence scoped.** The six profiles qualify these observed
   role/extra/map shapes, not every permitted feature-bit combination. Allocated
   zero-link bodies, malformed/unknown-role refusals and staged edits remain
   synthetic controls. A later snapshot needs its own independent whole-result
   evidence; successful role/type and sector-field comparisons do not prove that
   all returned physical ranges are allocated, disjoint or correctly owned.

The next bounded component is a **private bounded current fixed-metadata interval
collector**, before a complete builder. The existing mounted boolean refusal
helper does not return a complete bounded role inventory. Enumerate all protected
boot/primary/backup superblock and descriptor/reserved-descriptor ranges, every
group's bitmap locations and full inode-table spans using the current validated
layout. Handle FLEX_BG cross-group placement and complete bounds/overlap checks;
use explicit group/read/range/scratch limits and fail without partial publication.
Represent boot/superblock bytes sharing one physical block as one protection
range; do not let interval insertion conceal a conflicting metadata location.
Keep the journal in its separate inode-role/extent path, avoiding duplicate claims.
Require staged checksum/layout/size controls, tiny-limit and unchanged-state
controls, plus independent VM metadata-location reports. Block-allocation policy,
the complete owner join, operation guard/overlay/dependencies, sealing/publication
and invalidation/commit failures remain later gates. Do not connect admission yet.

### Private verified current inode-role reader — 2026-10-15

Follow-up: source `ef1ca24` passed focused Linux and full CI. The implementation/
evidence review above supersedes this section's pending CI status and review task;
it adds no complete snapshot, lifecycle or write activation.

The next private component after the inventory review is now implemented in
`inode_roles`. `Reader` requires a successful current inventory and only its
allocated IDs, on the same trusted mounted geometry and stable device/buffer
view. It does not revalidate the inventory, descriptors or allocation per record,
and carries no epoch token. A future builder must establish that precondition,
visit every allocated ID exactly once, and discard all accumulated results on
any refusal; this reader alone does not provide a complete-result snapshot.

The initial profile uses metadata checksums, 1 KiB/4 KiB blocks, 256-byte Linux
inodes, first ordinary inode 11, and explicit feature allowlists. Resize, quota,
orphan/recovery, external-journal and unknown feature profiles refuse. No journal
requires zero journal fields; an internal journal must be declared as inode 8
with no external UUID/device. Inactive reserved records require mode/links zero
and no size, sectors, flags, block contents or attribute storage. Root inode 2
must be a directory; a declared journal must be a nonempty regular inode.
Ordinary allocated zero-link records are retained and interpreted normally.

Each read returns inode/generation, typed role, size, sector/link counts and the
verified 60-byte storage area. Extent-backed regular files/directories and
block-backed symlinks have an admitted inline header with depth zero or one;
the journal is initially limited to depth zero. Only EXTENTS is allowed for file,
journal and block-backed symlink records; directories additionally allow INDEX
and validated CASEFOLD policy. The complete extent collector must still verify
every record/child and account for all storage; role classification does not
check extent payload records, external nodes, sector totals or global conflicts.

Fast symlinks require zero sectors/flags, a nonempty target of at most 60 bytes,
no embedded NUL and zero unused storage bytes. FIFO records require zero storage.
Character/block devices, sockets, ordinary mode-zero records, legacy/inline maps,
unknown flags, HUGE_FILE storage units and external attributes refuse. Extra
inode sections are limited to observed sizes zero or 32; bytes beyond the extra
section must all be zero, refusing inline attribute references as well. Fragment
addresses refuse. This narrow ownership-interpretation profile does not change
the existing general inode reader or its supported read matrix.

Reads use current staged table bytes when present, requiring exactly one block,
and otherwise the device. Table addresses/body slices are bounded before use.
Inode checksums include inode number and generation, using a fixed 256-byte
stack copy with the checksum slots cleared; the zero-extra profile compares
only the low checksum half. One fallible table-block allocation is reused.
Record attempts and logical fetches, including staged/failing accesses, debit
shared counters across calls. Metadata scratch requires `block_size + 256`
bytes; fixed reader/parsed-record stack state, allocator overhead and device
cache are outside that scratch figure. No result vectors are retained by this
component. The eventual builder must charge this buffer's retained capacity,
inventory capacity, role/claim arenas and transient outputs together.

Four controls first failed on acceptance through the existing verified inode
reader: unknown storage flags, external attributes, inline attributes, and storage
hidden in an inactive reserved record. A healthy file control passed. Those
same refusal assertions pass with the new component. Seventeen focused controls
cover current staged identity, zero-link allocation, both-view checksums, fast/
block symlinks, FIFO, explicit journal role, header/type/flag/attribute profiles,
low-only checksums, exact/exhausted shared limits, malformed tables, bounds and
read faults. See `extprogress.md` for full baseline retention and final checks.

A new independent VM test formats four no-journal block/descriptor profiles and
two 64-bit internal-journal profiles. Linux initializes both groups, creates and
removes files, creates both symlink shapes, a FIFO and a depth-one extent file.
It requires judged non-repairing e2fsck cleanliness, classifies every allocated
ID including reserved slots, and compares its type and sectors with exactly one
saved `debugfs stat` result per ID. It requires both symlink forms, FIFO, the
external-map file and the declared journal outcome. The focused workflow enforces
one executed test and saves images/commands/raw reports/log. This new oracle is
compiled but unrun locally; exact-commit CI remains pending after push.

No production mutation path calls this reader. The three other-inode alias
failures, fixed/block-allocation checks, snapshot construction/overlay, final
sealing, mutation invalidation, operation guard and failure publication rules
remain open. Inspect new CI and review the component before joining it to the
complete collector under aggregate budgets; keep every write gate in place.

### Inode-inventory implementation and evidence review — 2026-10-15

Follow-up: the private role reader above implements this review's next component.
Inventory source `4309ed1` now passed full CI as well as focused Linux validation.
This does not qualify the new role reader or a complete owner snapshot.

Reviewed `4309ed114caca9d13c31e6647276ca883c116781`, the complete private
inventory, its fifteen unit controls, VM fixture and focused workflow. Its
[focused Linux run](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38685893409)
passed. Downloaded and inspected the actual images, allocation commands/reports
and logs. The new inventory test executed once, zero failed/ignored (33 lines /
3,357 bytes). Existing insertion and FIEMAP tests each executed once, zero
failed/ignored (133 lines / 7,953 bytes and 23 lines / 1,601 bytes respectively).
The separate full CI is still running at this review's initial check; its final
status and this checkpoint's local validation are recorded in `extprogress.md`.

| Block size | Descriptor bytes | Groups / inode slots | Allocated per group | Free per group |
|---|---|---|---|---|
| 1 KiB | 32 | 2 / 64 | 22, 11 | 10, 21 |
| 1 KiB | 64 | 2 / 64 | 22, 11 | 10, 21 |
| 4 KiB | 32 | 2 / 64 | 22, 11 | 10, 21 |
| 4 KiB | 64 | 2 / 64 | 22, 11 | 10, 21 |

For each image, Linux population and non-repairing e2fsck passed. Every one of
the 64 IDs has exactly one saved allocation answer. The test compares the
complete result with those answers, including reserved IDs 1–10 and free slots.
A raw public-format audit also checked both bitmaps against the reports, full
padding, initialized flags (ITABLE_ZEROED only), and the shared primary descriptor
block. These are small-image compatibility controls, not large-card performance
or a measured memory bound. Partial final inode groups and allocated zero-link
bodies remain synthetic controls; this Linux fixture does not qualify them
independently. It has no journal, resize inode, quota or attribute-owner matrix.

**Decision:** no implementation blocker found within this standalone allocation
scan's stated supported profile and stable trusted-mount preconditions. No source
fix or new regression test is needed for this review. The scan is suitable as a
private prerequisite for role classification; it establishes neither ownership
nor reusable transaction-current state. Keep it disconnected from admission.

The source review checked:

- Supported block/descriptor sizes and nonzero bounded group geometry precede
  slicing, division and descriptor lookup. Group coverage plus `s_inodes_count`
  bounds every emitted ID; ascending group/bit traversal cannot duplicate IDs.
  The final group's valid count excludes padding from output. Full bitmap-block
  padding is checked even where it lies outside the checksum's covered bytes.
- Current superblock identity/checksum precedes descriptor reads; permitted free
  counters do not change addressing. Each current descriptor verifies in its
  group context, preserves mounted metadata addresses and reconciles pending
  flags. INODE_UNINIT refuses before unspecified bitmap contents are read.
  Bitmap checksum width follows the admitted descriptor width. Current staged
  bytes take precedence and must have exactly one block's length.
- Group/probe work is checked up front, each logical fetch is charged before
  access, and output capacity is checked before pushing. Fixed fallible output
  and two-block scratch reservations use checked requested-byte arithmetic;
  descriptor caching reuses scratch across groups. Any failure drops local
  output, with no device/buffer mutation or publication. The fifteen controls
  include exhausted limits, current staged changes and unchanged-state checks.

#### Contracts for the next role-classification component

1. **Use the same stable current view.** The returned ID vector carries no
   epoch or allocation dependencies and reads no inode bodies. It becomes stale
   if allocation changes. A later component must not combine an earlier census
   with newer inode/table/tree contents or treat a fresh census as operation-wide
   serialization. Bound and verify staged inode-table blocks before interpreting
   complete inode records; preserve the current superblock/descriptor contract.
2. **Give every allocated ID an explicit outcome.** Include reserved and
   zero-link records; never infer allocation or ownership from reachability,
   type, size or link count alone. The saved Linux images have mode-zero reserved
   bodies as well as root inode 2; none of their reserved bodies is all-zero.
   Do not assume every reserved record is an ordinary extent file, or that
   mode zero alone permits ignoring storage. Establish each admitted reserved
   role and its checksum/storage rules using documented fields and independent
   fixtures. Refuse unqualified reserved/journal/resize/quota/orphan profiles.
3. **Make map and attribute policy explicit.** Verify current ordinary inode
   checksums and identity, distinguish supported extent-backed files/directories,
   fast versus block-backed symlinks, and storage-free special records. Refuse
   unknown flag/type combinations, legacy/inline maps and external attributes
   until their full storage interpretation is qualified. Check inline attribute
   layouts or refuse them; they can encode references beyond `i_file_acl`.
   Preserve the collector's independent depth-zero/one matrix rather than
   treating its synthetic maximum depth as independent qualification.
4. **Keep complete-builder gates separate.** This scan does not check block
   allocation or all cross-group metadata roles. Allowing BLOCK_UNINIT here
   does not qualify a block-allocation policy. Before publishing ownership,
   combine fixed roles, every admitted inode's complete storage claims and
   allocation checks; reject all conflicts under shared read/work/claim and
   peak-capacity budgets. Charge retained inventory capacity together with
   role/tree output and scratch, not just populated IDs. No partial snapshot
   may escape on an unknown role, limit, read or validation failure.

The three saved other-inode alias failures remain unresolved. Complete snapshot
construction, transaction overlay/invalidation, operation guard, commit failure
handling and public write activation remain separate checkpoints. The next
bounded implementation is a private verified current inode-role classifier,
with independent role controls; do not wire it into insertion admission yet.

### Private bounded allocated-inode inventory — 2026-10-14

Follow-up: source `4309ed1` passed focused Linux allocation/insertion/FIEMAP;
the implementation/evidence review above supersedes this section's pending
focused status and review task. Full CI status is maintained in `extprogress.md`.

Following the collector/resource review at `7825062`, the new private
`inode_inventory::collect` enumerates all current set inode bits, including
reserved and allocated zero-link slots. It reads no inode bodies and does not
infer owners from directory reachability. It is disconnected from admission;
the existing mounted allocation helper and ordinary mutation paths are unchanged.

Before implementation, the existing mounted helper reproduced three acceptance
failures (bad staged descriptor CRC, bad staged bitmap CRC, initialized-policy
change) and one missed current allocation. The healthy reserved-ID control
passed. The same assertions pass against the new inventory. Expanded local
controls cover exact/exhausted aggregate limits, partial final-group coverage,
shared descriptors, malformed block sizes, checksum-valid bad padding inside
and outside CRC coverage, pending/unknown flags, relocation/bounds, superblock
policy changes, freed bits, allocated zero-link bodies and read failures.

The initial profile requires metadata checksums, 1 KiB/4 KiB blocks, 32/64-byte
descriptors, valid mounted geometry and stable device/staged contents. META_BG,
bigalloc and INODE_UNINIT are explicitly unsupported; unspecified bitmap bytes
are never interpreted as live allocation. Known BLOCK_UNINIT/ITABLE_ZEROED flags
do not themselves invalidate an initialized inode bitmap. Pending initialization
flags must agree with current descriptor flags and stay within the group table.

The current superblock must verify and match mounted bytes except the existing
free-counter fields. Every descriptor verifies with its group context; metadata
pointers remain mounted and bounded. Bitmap CRC width follows descriptor size.
Coverage must account for every inode group, including a partial last group;
all bits beyond valid IDs, including full-block padding beyond checksum coverage,
must be set and are excluded from output. Output IDs are ascending and unique.
Basic within-group metadata aliases refuse; complete cross-group metadata roles
and allocation/ownership conflicts remain snapshot-building requirements.

One limit set covers the entire invocation: groups, full bitmap-bit probes,
logical fetches (including staged copies), output IDs and scratch. Group/bit
work is checked before scanning; reads and output pushes cannot exceed caps.
The descriptor block is cached across neighboring groups, so the two-group
control needs four logical reads: superblock, one descriptor block, two bitmaps.
Two fixed buffers are reused; fallible reservations request at most
`4 * inode_cap + 2 * block_size` bytes, with checked arithmetic. Retained vector
capacity, allocator overhead/device cache and later tree/role storage must still
be accounted for by a complete builder; this is not a measured card/RSS bound.
Staged sizes are checked before copying, so malformed dirty blocks are not cloned
outside the scratch bound. All errors drop the local output and preserve the
full dirty map/pending flags/device; no cache/snapshot is published.

New independent CI controls format four Linux profiles: both block sizes and
both descriptor widths, each with two groups sharing a primary descriptor block.
Linux creates/removes files across the group boundary, directories, a symlink
and a hard link, leaving both inode bitmaps initialized. Non-repairing e2fsck
must pass; the inventory must equal every valid ID's `debugfs testi` answer,
including reserved IDs and free slots. Missing/duplicate oracle answers fail.
The workflow requires one executed test and retains images/command files/raw
allocation reports/logs alongside insertion and FIEMAP evidence. This oracle
is compiled but unrun locally; exact-commit Linux/full CI remains required.

See `extprogress.md` for final counts/status. Partial-last-group and allocated
zero-link cases are synthetic unit fixtures, not Linux-clean image claims.
The inventory proves no inode-body checksum, reserved role, mapping/attribute
ownership or transaction epoch. The three saved cross-inode refusals remain
unresolved. Next inspect CI and review this component before verified current
inode-role classification and joining roles with complete extent claims under
aggregate snapshot budgets. Keep every write gate and lifecycle requirement.

### Collector implementation and resource review — 2026-10-14

Follow-up: the private inventory above implements its next component; source
qualification, typed roles, complete snapshots and admission remain distinct.

Reviewed `7bdfe0cf83d3d65f4068225707c379f3fad9e6aa`, its complete private
collector/tests, the real staged-view control and the saved Linux evidence.
Both [focused Linux validation](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38675564104)
and [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38675564205)
passed at that exact commit. Full CI includes both VM suites, wasm, semver,
MSRV, unit/CLI/provenance jobs and the aggregate gate. The preceding review's
full CI at `80caea6` also passed.

Downloaded the actual focused artifact. The new FIEMAP test executed exactly
once, zero failed/ignored (23 lines / 1,601 bytes); its log records successful
Linux population and non-repairing e2fsck for both block sizes. The existing
insertion test also executed once, zero failed/ignored, across its four profiles
(133 lines / 7,954 bytes). This is independent source qualification, not only
a successful workflow badge or a compilation check.

Saved Linux FIEMAP reports and public-format raw image observations give:

| Block size | File | Extent depth | Data records | Data blocks | Map blocks | Returned claims |
|---|---|---|---|---|---|---|
| 1 KiB | Inline | 0 | 1 | 1 | 0 | 1 |
| 1 KiB | External | 1 | 17 | 19 | 1 | 18 |
| 4 KiB | Inline | 0 | 1 | 1 | 0 | 1 |
| 4 KiB | External | 1 | 17 | 19 | 1 | 18 |

Each external file has 16 initialized one-block ranges and one three-block
unwritten range at logical blocks 1024–1026, past visible EOF at block 497.
Linux reports 40 allocated sectors at 1 KiB and 160 at 4 KiB: 19 data plus
one mapping block. The test compares all data locations/lengths/unwritten flags
with FIEMAP, and the total including map nodes with the inode sector count.
The raw observations are a profile audit of these already independently checked
images, not another host oracle or performance measurement.

**Decision:** no implementation blocker found for this standalone read-only
primitive within its stated profile/preconditions. It is ready to be used by
the next private snapshot-building components. It remains disconnected from
admission. This decision does not certify a mounted/current owner cache, enable
writes or close findings 2 and 3. No production fix or new test is introduced
by this review; `extprogress.md` records fresh baseline retention.

The source review checked these invariants:

- Exact 60-byte root, supported checksum/block profile and depth at most five
  precede traversal. Node count/capacity checks and the existing header parser
  bound all record slices; exact decreasing depth bounds scratch slicing and
  recursion. The caller must still verify the inode and stable view: inline
  data has no external checksum that could establish that precondition.
- Children are filesystem/device-bounded and checksum-verified before their
  headers/descendants are interpreted. Every declared branch is visited;
  first-key agreement and parent intervals exclude shadowed logical records,
  while sparse gaps are allowed. Physical sorting checks adjacent disjoint
  intervals and catches cross-branch role aliases; no payload is read.
- Reads, declared-record visits and pushes cannot exceed their explicit caps.
  Output and per-level scratch reservations are fallible and their combined
  requested byte arithmetic is checked. Siblings reuse scratch; sorting is
  in-place with bounded `O(C log C)` work. Read/validation/limit errors discard
  local results without editing device or buffer; no shared state is published.

#### Requirements carried into the snapshot builder

1. **One aggregate budget.** Per-tree limits reset on each call; they do not
   bound a volume-wide scan. Charge all descriptors/bitmap/inode probes, metadata
   reads, tree records, claims, overlap work and allocation requests against
   shared remaining limits. Bound retained index capacity, transient tree output,
   scratch and reverse dependencies together, including their peak coexistence.
2. **Count capacity, not only populated ranges.** The collector reserves the
   full caller-supplied claim cap and returns that vector. The current Linux
   control requests 1,024 slots even for its one-claim inline file. Do not retain
   one such vector per inode and charge only `len()`. Either charge retained
   capacities or fallibly copy populated claims into one bounded arena and drop
   transient output before processing the next inode. Copying must include peak
   old/new allocations in the budget; shrinking is not a substitute for that
   proof. Allocator overhead/device caches remain outside requested-byte bounds.
3. **Respect the independent depth matrix.** Linux evidence here covers depths
   zero and one. Depth two through five have synthetic controls, including the
   maximum-depth chain, but no new independent matrix in this checkpoint. The
   initial snapshot profile must refuse those depths or add independent deeper
   controls before admitting them; do not infer full qualification from the cap.
4. **Establish every owner before publication.** Collect every allocated inode,
   not only reachable names or positive-link-count files. Interpret reserved
   roles explicitly, validate current allocation/fixed roles, and refuse unknown
   inode/attribute/legacy profiles. A successfully collected tree proves neither
   inode allocation nor exclusive ownership against other trees. A partial scan
   or exhausted budget cannot produce an apparently complete snapshot.
5. **Keep transaction gates.** This review adds no operation guard, epoch,
   dependency closure, current overlay, final seal, failure handling or device
   exclusivity. A newly built snapshot is not a cache safe for later admission
   until those reviewed lifecycle requirements exist. Never rebuild it by
   scanning the card on every insertion as an implicit fallback.

**Next bounded checkpoint:** implement the snapshot builder's private bounded
allocated-inode inventory, not admission. Verify current descriptor/bitmap
checksums, bounds, initialization/unknown flags and final-group coverage before
returning every set inode bit, including reserved and unlinked allocated slots.
Keep role interpretation and complete cross-owner snapshots as later components;
an inventory alone must not claim ownership. Use shared limits for group/bit
probes, reads, output and scratch, require no partial result on errors, and test
disk/staged bitmap changes, shared descriptor blocks, invalid tails/checksums,
initialization state, reserved IDs and exact/exhausted budgets. Add independent
Linux bitmap/inventory controls before qualifying that component. Preserve all
872 passing unit occurrences, the existing Linux controls and every write gate.

### Private bounded extent-ownership collector — 2026-10-14

Follow-up: this landed at `7bdfe0c`. Its exact-commit Linux FIEMAP/insertion
and full CI passed; the implementation/resource review above supersedes its
pending status and next task without widening the collector or admission profile.

Following the design review at `80caea6`, `extent::ownership::collect` now
provides a crate-private read-only primitive. It is deliberately disconnected
from insertion admission and does not build a volume owner index. The shared
public lookup/collection APIs and all write gates are unchanged.

Before implementation, seven malformed-tree cases were tested against the
existing `collect_all_with_nodes` adapter. All seven failed specifically on
acceptance, while a healthy disjoint control passed: unselected-child checksum,
wrong child depth, shadowed/parent-key coverage, capacity, duplicate child,
data/map role overlap and zero-length extent. The same refusal assertions now
pass using the new collector. This demonstrates why the existing public helper
cannot itself serve as an ownership proof; it does not change that helper's API.

The collector requires an already verified 60-byte inode root and its inode/
generation checksum context, a stable device or staged view, filesystem block
bounds and explicit limits. The initial supported profile is metadata checksums,
1 KiB/4 KiB blocks, root capacity at most four and full-capacity external nodes.
Other block sizes, disabled checksums and smaller external capacities are
unsupported. The capacity restriction keeps the tail at the location supported
by the existing checksum verifier; broader profiles need independent evidence.

It walks every declared child/record, including unwritten and beyond-EOF
storage, with no payload reads or expansion into per-data-block claims. Before
following a child it bounds the address and verifies its checksum; exact depth,
capacities/counts, nonempty children/indexes, strictly increasing index keys,
parent-first-key agreement, logical ordering/parent intervals and physical
bounds are required. Sparse leading/interior/trailing holes are allowed.
Physical sorting rejects overlap between any pair of claims, including
repeated nodes and data/map aliases across branches. Cycles cannot evade exact
decreasing depth; all traversal is also bounded by read/record/claim limits.

Results identify the inode/generation and complete disjoint physical intervals
with data logical offsets/unwritten flags or mapping-node roles. Any error
returns no result. It never changes its input bytes, device or transaction
buffer. Current allocation, fixed metadata, other inode/attribute roles and
concurrent/external mutation remain separate prerequisites.

Limits are caller-supplied, with no production defaults: node reads, examined
records, output claim count and scratch bytes. Before walking, fallible fixed
reservations request at most `claims * size_of::<Claim>()` output bytes and
`depth * block_size` scratch bytes, with checked combined arithmetic. The same
per-depth buffers are reused across siblings; maximum depth five means at most
six recursive frames. Record/claim/read limits are enforced before excess work,
with in-place physical sorting bounded by `O(C log C)` for claim cap `C`.
Allocator overhead and the supplied device's own cache are outside these
allocation-request bounds; this is not a measured full-card memory/latency claim.
No partial list, unbounded fallback scan or growing visited-node set is used.

Expanded local controls cover exact/short/zero budgets and size overflow,
empty inline roots, maximum depth, cycles, logical/end/address boundaries,
high physical addresses, 32,768-block initialized extents, unwritten ranges,
read errors, checksum identity and unsupported profiles. A filesystem-level
test uses the real `StagedTreeNodes` overlay: changing only an external child
changes the returned data claim while the disk claim stays separate; a staged
checksum error preserves the whole buffer, pending flags and device bytes.

A new independent `needs_host` control creates 1 KiB/4 KiB Linux files with
inline and external trees, sparse leading holes and KEEP_SIZE preallocation
beyond EOF. Its public FIEMAP ioctl report is compared with every returned data
interval/unwritten flag; Linux's allocated-sector count must equal all data plus
mapping claims. Non-repairing e2fsck checks the image. The focused workflow runs
this test with an execution floor of one and saves its log, Linux reports and
images, alongside the existing four-profile insertion evidence. This new oracle
has not run on this cloud host, which lacks KVM; exact-commit CI is required.

See `extprogress.md` for final local counts and CI status. Findings 2 and 3,
the three saved cross-inode aliases, snapshot/overlay integration, operation-wide
serialization and failure/invalidation handling remain open. After inspecting
the new independent results, review this primitive and its resource profile
before starting the bounded complete snapshot builder. Public writes, deeper
insertion, collisions, splitting/growth and index dropping remain blocked.

### Transaction-current ownership design review — 2026-10-14

Follow-up: the collector checkpoint above implements the first private
primitive. The design's snapshot, serialization, overlay and integration gates
remain unimplemented; its other-owner tests still require the eventual fix.

Reviewed own MIT source at `f9a0df0544072b0e252d5d4ff2912c70f1838a0b`.
Its [focused Linux run](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38666477495)
and [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38666477535)
passed. The downloaded insertion log executes one test across four profiles,
zero failed/ignored (133 lines / 7,953 bytes). This qualifies the scoped journal
checkpoint, not the owner-index proposal below.

The saved other-owner patch applies to this source and still executes four
tests: three fail specifically on `Ok(())` at blocks 517, 521 and 549, while
the disjoint control passes (37 lines / 1,893 bytes). The patch was reversed
after this observation; the unchanged production baseline is revalidated in
`extprogress.md`. No owner refusal is implemented by this review.

**Decision:** use a complete, bounded ownership snapshot per mount epoch plus
a transaction overlay, only after operation-wide serialization and write-path
coverage exist. Do not integrate a mounted-only cache or scan the whole volume
on each insertion. A snapshot that cannot complete within its supported profile
and resource limits refuses write admission; it never publishes partial proof.
This is a design selected for incremental implementation, not an enablement
review. Keep findings 2 and 3 and every existing write gate open.

#### Evidence that constrains integration

| Existing path | Consequence for the design |
|---|---|
| `buffer_add_dir_entry_indexed` maps referenced logical blocks through the current inode root in two device views | The visited-node set is not the whole tree. Unused/shadowed references need a complete ownership walk; do not call the disk view a separate disk-inode-root walk. |
| `extent::collect_all_with_nodes` recursively gathers records without an ownership work budget or full structural proof | Do not turn it directly into admission. `extent_tree_runs` checks node tails after collection but uses the device view and is also not a current ownership proof. |
| `BlockBuffer::dirty`, `put` and `get_mut` permit direct staging | Tracking only `buffer_write_inode` misses changed external map nodes, bitmap bits and raw inode-table patches. Derive changed dependencies from final dirty bytes as well. |
| `commit_block_buffer` locks the journal only after the caller finishes planning | This lock cannot serialize duplicate checks, allocation or ownership admission. No-journal commits lack even that journal lock. |
| `write_inode_raw`, direct descriptor writes, replay, recovery and `fresh_read` exist outside the normal buffer lifecycle | Audit them before trusting an epoch. Each must join the operation guard and update or invalidate ownership before mutation; otherwise retain write refusal. |
| `Filesystem::dev` is public and callers can retain their own device handle | An internal mutex cannot stop external changes. Exclusive device ownership is a prerequisite; explicit refresh discards proof. Arbitrary concurrent device mutation remains unsupported. |
| External attributes may legally share a block | Ownership needs roles and multiple references; a single block-to-inode map is insufficient. Attribute sharing never licenses directory storage. |

#### Completeness and supported roles

Bootstrap after journal replay/orphan recovery and under exclusive device access.
Read current verified descriptors and inode bitmaps, including initialization
state, checksum width, final-group coverage and reserved-inode rules. Do not use
`inode_bit_is_set` alone: its mounted allocation view and unchecked bitmap are
not the required proof. Enumerate all allocated inodes, including unlinked
allocated inodes; directory reachability and positive link counts are not the
enumeration criterion. Interpret reserved slots by their documented role, not
as ordinary files merely because their allocation bits are set. Unsupported
active reserved roles or unqualified layouts refuse the initial profile.

Each claim records a checked physical half-open interval, inode number and
generation, storage role, logical interval where applicable, and view/epoch.
Keep data extents as intervals, never one owner entry per payload block. Walk
every declared map record, including unwritten/preallocated storage beyond
`i_size`, and every external mapping node. Check bounds before reads, node
capacity, exact decreasing depth, checksums before following children, logical
ordering/parent key ranges, duplicate children/cycles, and physical overlaps.
The tree's own data and mapping roles must be disjoint. A selected lookup is
not a substitute for that walk. Budget exhaustion and unsupported shapes return
errors, with no truncated claim list.

Initial collectors cover extent-mapped ordinary files/directories and the
already restricted journal profile. Legacy indirect maps and special reserved
layouts remain unsupported until completely walked and independently qualified.
Classify fast symlinks, inline data and device payloads before looking at
`i_block`; never interpret their bytes as block addresses. Unsupported flags,
attribute-value inodes, inline layouts or special inode profiles refuse the
snapshot, rather than disappearing from it. An initial narrow implementation
may refuse all external attributes. Later attribute support must record every
reference, verify its block and sharing policy, and include attribute-value
storage before admitting that profile. Unknown ownership cannot mean unowned.

Keep fixed metadata and journal roles protected separately. Require current
allocation for recorded claims. Detect illegal overlapping claims, including
two roles of one inode; do not let later interval insertion overwrite an earlier
owner. Legal shared attribute claims need explicit representation. A hard link
is another name for the same inode, not another storage owner.

#### Transaction overlay and revalidation

Retain reverse dependencies from inode-table blocks to inode slices, bitmap
blocks to allocation groups, and external map/attribute blocks to every owning
inode. A changed external child can relocate storage without changing its inode
root; rebuild affected owners through the staged view, discovering new children
while walking. Dirty inode-table blocks require comparing all contained slices,
not invalidating or approving every inode merely because it shares the block.
Allocation bitmap differences include newly allocated/freed/reused inode slots;
inode identity includes generation. Reconcile descriptor initialization changes.

Use the final dirty map, not a helper-maintained touched list alone, to determine
the dependency closure. Dirty superblock/layout/descriptor changes retain the
existing narrow refusals. Unexpected bitmap/table/map writes, missing dependency
coverage or ambiguous staging refuse admission. Never assume a staged payload
is a map node just because its bytes happen to parse as one.

Build an isolated overlay of affected owners without mutating the committed
index. Keep committed claims protective until commit succeeds. In the initial
profile, any storage transfer from another inode in the same transaction is
unsupported even if its bit has been cleared; retaining old claims conservatively
prevents an early reuse from erasing the original owner. New current claims
protect newly allocated/remapped storage, which catches the buffered-only owner
at block 549. Directory data and map roles must agree with the complete current
parent walk and remain disjoint from all conflicting claims in both views.

Any ownership-relevant edit after admission invalidates that admission. Seal
and validate the final candidate before device writes; prove the selected leaf
edit leaves owner mappings unchanged. Revalidate after later helpers have
composed changes (rename continues planning after insertion). Pending claims,
bitmaps and dirty bytes are discarded together on precommit failure. No shared
snapshot publication occurs during planning.

#### Serialization, commit and failure

Acquire a mount operation guard before path resolution/duplicate checks and
retain it through allocation, planning, final ownership checks, durable commit
and index publication. Path and inode entry points must share guarded internal
helpers, avoiding recursive locking when wrappers call one another. Define one
lock order: operation guard before allocation-state/cache/journal locks; no
commit-held callback may reacquire the operation guard. Review callback/reentry
behavior and reader consistency separately before enabling public writes.

An overlay carries its base epoch and buffer revision; a stale epoch/revision
cannot authorize a write. Prepare every allocation needed to publish the next
index before the first device write, then publish without fallible allocation
only after commit reports durable success. A journal/device failure can leave
replayable or partly written bytes: invalidate ownership and refuse subsequent
writes until recovery/remount. Do not roll back only the in-memory index and
continue as though the old disk state were known. Poisoning has the same refusal.
Explicit refresh, replay, geometry changes and raw mutation discard the proof
before work starts; rebuilding is explicit, not an automatic scan per insertion.

#### Resource contract and acceptance gates

An interval index avoids a per-block owner array for the 1.5 TB goal, but a
highly fragmented card can still exceed memory/work limits. Account for peak
committed index plus reverse dependencies, staged replacements, sorting scratch,
visited sets and buffers, not only the final claim vector. No eviction may
convert unknown coverage into apparent freedom. Use fallible growth and checked
arithmetic. A builder/overlay must charge each inode/bitmap probe, record/edge,
metadata read, range query and byte allocation before doing that work.

For the next private collector, propose explicit caller-supplied limits for
node reads, examined records, output claims and scratch bytes, with tiny-limit
boundary tests. Enforce a depth cap of five from the public extent format.
Production scan/transaction limits need measured profiles and fragmentation
data before integration; no defaults, latency or real-card capacity are qualified
by this review. Bootstrap is bounded and performed once for a valid epoch;
ordinary overlays revisit changed dependencies only. Exceeding a limit gives
an explicit unsupported/resource refusal before mutation, never a partial proof
or a fallback to an unbounded scan.

Implementation gates, in dependency order:

1. Add a private, read-only complete extent-ownership collector with explicit
   budgets. Reproduce unused/shadowed child, bad-depth/key/capacity/checksum,
   duplicate/cyclic child and unwritten/beyond-EOF cases before their fixes.
   Require both device and staged-view tests, exact role/range output, immutable
   inputs on failure, and healthy inline/external Linux reference controls.
   This is the **next bounded checkpoint**; do not wire it into admission yet.
2. Build a complete bounded snapshot for an explicit inode/feature profile,
   including bitmap/current-allocation validation, reserved roles and conflicts.
   Test refusal on incomplete scans/limits; measure retained and peak resources.
3. Add operation-wide serialization and the full mutation/invalidation audit,
   then transaction overlays/final sealing and commit-failure handling. Test
   changed child without inode edit, shared inode-table blocks, slot reuse,
   later-helper edits, abort, durable publication, refresh and failed commits.
4. Integrate admission only when these prerequisites hold. Activate all three
   saved other-owner refusals and the disjoint control together; require their
   full unchanged-buffer/flag/device assertions. Preserve every existing test
   occurrence, lint/build checks and four-profile Linux insertion evidence;
   add independent profile validation for each newly admitted map/role.

The collector alone does not solve the three current owner aliases, establish
operation concurrency safety, close complete directory-map qualification or
authorize public writes. These remain explicit later integration gates.

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
exact-commit focused Linux/full CI subsequently passed at `f9a0df0`, as recorded
in the ownership design review above. `extprogress.md` records the measured logs.

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
matrix; exact-commit focused/full CI passed at `f9a0df0`. The
synthetic multi-record control does not extend the independent profile matrix.
JBD2 contents/lifecycle, concurrent/external mutation, other inode/attribute
ownership and complete directory map shape remain unqualified. In particular,
the three other-owner counterexamples below are **not fixed** by journal-only
protection. The design review above now records the bounded transaction-current
owner-map strategy and next collector checkpoint; preserve those tests and their
disjoint control before the eventual refusal implementation.
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
