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
