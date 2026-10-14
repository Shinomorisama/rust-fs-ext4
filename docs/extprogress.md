# extprogress

## What we are doing

We are extending `rust-fs-ext4` so a normal ext4 removable-storage card can eventually be read and written safely from Windows through `ext4-win-driver`, then used natively by Linux and SteamOS. The intended use is a 1.5 TB microSD card shared between these systems.

The main compatibility gap is **ext4 casefold**: selected directories treat different spellings as the same name according to the filesystem's declared Unicode rules. Correct support must cover finding names, preventing equivalent duplicates, changing directory entries, maintaining directory indexes, and recovering interrupted writes. Stored filenames must retain their original bytes.

The goal is a working implementation. Documentation, unit tests, and independent Linux/e2fsprogs checks support that goal. Passing this driver's own tests alone is not enough to establish compatibility.

**Casefold writes are still deliberately blocked. This is not yet a Windows-ready release for the card.** No actual SteamOS hardware or Windows/card round trip has been qualified.

## Required chat opening and staged progress reports

When the user links the repository in a new chat, read the initial `AGENTS.md`,
follow its pointer to the latest `casefold-rw`, read that branch's `AGENTS.md`,
then read this handoff. Verify the current commit, local work and available CI
evidence. Resume the latest bounded task rather than restarting completed work.

Before implementation, explain the project to a noncoder: the aim is one ext4
microSD card usable across Windows, Linux and eventually SteamOS; casefold makes
different capitalizations refer to the same name while preserving saved spelling.
State the present qualification limits and whether normal casefold writes remain
blocked. Describe the approach: one bounded checkpoint at a time, reproduce a
defect before fixing it, preserve passing baseline tests, perform focused reviews,
validate independently with Linux/e2fsprogs in CI, and record the checkpoint
before pausing. Scratch-image results do not qualify a real card or Windows use.

At both the start and conclusion of **each staged run**, post a self-contained
report containing:

1. The complete Stage 0–7 table from **Planned stages and our position**, with
   plain-language purposes and current progress for every stage. Repost the full
   list even when only one stage changed; retain the original numbering.
2. The active stage (or overlapping stages), its completed and remaining work,
   and the bounded checkpoint being attempted or just completed.
3. Progress on the active stage. Give a percentage only when it has an explicit
   scope and evidence-based completion criteria; label estimates and explain
   their basis. Otherwise say **not yet quantified**, give the concrete completed
   and remaining pieces, and do not invent a percentage from test counts or the
   number of stages. Checkpoint completion does not establish stage completion.
4. At checkpoint conclusion, the work attempted, resulting changes, validation
   outcomes and relevant commit/CI links. Distinguish fresh results from earlier
   evidence and passed, failed, pending, blocked and unrun checks. Documentation
   changes alone do not advance implementation or qualify writes.
5. The next bounded task and any remaining prerequisites, with the write gates
   and Windows/SteamOS qualification limits stated accurately.

Keep the explanation concise, understandable and complete in chat, rather than
requiring the user to open a log or remember an earlier report. At checkpoint
completion update this existing handoff's current checkpoint, affected stage
statuses, evidence and next task. Preserve historical evidence and restrictions;
do not create a competing progress file. These reporting instructions add no
permission to relax write gates, modify siblings or push to `main`.

## Current checkpoint

- Repository: [Shinomorisama/rust-fs-ext4](https://github.com/Shinomorisama/rust-fs-ext4), branch `casefold-rw`.
- Latest integration checkpoint: upstream ordinary-volume recovery recount fix
  `d7519c7f90ef9ec2c538949459c0d4a68655d145` (#508) is backported with its Linux
  crash-image test and a new local regression. Local units, lint and CLI build
  pass. Its independent Linux crash-image test now passed in both kernel and
  guest tiers; full CI at `0896577` failed because the kernel log exceeded its
  line budget. The measured repair's full CI now passed at `643fd2a`; subsequent
  full runs at `1ee1fed` and `d6ae8d8` also passed.
- Latest implementation work: private casefold index admission now also refuses
  overlap between directory data and visited external mapping blocks, including
  mapping blocks discovered after their directory-data role was admitted. The
  prior fixed-metadata, bounds, checksummed disk/staged mapping agreement and
  admitted-leaf checks remain. See the mapping-role checkpoint below and use
  `git log` to identify its commit. Independent CI for this new fix is pending.
- Mapping fix: `6402d5d82dcf5768105964b985cb1d914d351d80`. Previous checkpoint: `a7db0769d85a1cbea189a860d9a46fa1d293d3d9` (focused Linux insertion CI). The original insertion primitive landed in `248e879636c4e03f476a440dac48d316b42c0fe8`; this checkpoint hardens its mapping admission.
- Original baseline: `v0.8.0`, commit `204d8f6518ca5d66de07ab8e84c81501909b5d09`.
- Local validation: **810 unit tests pass**, retaining all 807 fresh baseline
  test occurrences (multiset comparison), and lint passes. The three new focused
  mapping-role tests pass, including two reproduced red refusals. The locked
  release CLI build/install also passes.
- The separately selected slow deep-walk fuzz test passed at source commit
  `643fd2a26d50e24b4d06d2b2872bfdd538cfbb22`: one executed, zero failed or
  ignored. It remains ignored in the ordinary suite; see the explicit run below.
- Metadata-role regressions are now active in `src/fs.rs`. Four negative tests
  reproduced metadata root/leaf acceptance before the scoped fix; those same
  tests and a positive inode-table boundary test now pass. This addresses only
  the fixed-location subset of review finding 2, not complete ownership checks.
- Fixed-metadata checkpoint `3be872e` passed its exact-commit
  [focused Linux workflow](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38542952071).
  Downloaded its insertion log: exactly one passed, zero failed or ignored,
  covering four profiles (133 lines / 7,954 bytes). Its
  [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38542952151)
  still has two VM suites running, with no reported failures. Do not call it passed.
- Previous checkpoint `a7db076`: [focused Linux insertion and fixture evidence](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38131444906) **passed**. Its insertion step executed exactly one test covering four Linux profiles (133 log lines, 7,954 bytes). [Full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38131444887) also **passed**, confirmed in the final pre-push check. The mapping fix subsequently passed the focused Stage 0 workflow (38134122031). Full CI at the mapping fix (38134122021) and the earlier handoff commit (38134210737) failed; that unit artifact confirms all 796 tests passed and the log exceeded its line budget by two lines. See the access follow-up below.
- Working tree clean at verification; the checkpoint is pushed to the fork.
- The required core pin remains `v0.3.0`
  (`9e900154a7dff2a6c8f136db31def33bed9f09cc`). This cloud instance uses the
  immutable pinned archive through the documented source view; its actual
  sibling checkout stays clean at onboarding `v0.3.6`. Neither sibling nor
  dependency pins/locks are to be modified.

## Visited mapping-block/data overlap refusal — 2026-10-14

At source `3be872e8a8794239182918ffc4b21121d97ac29e`, a fresh local baseline
passed **807 unit tests**. The preceding fixed-metadata change's exact-commit
focused Linux insertion workflow passed; downloaded and inspected the retained
insertion log, confirming one executed test, zero failed or ignored, and its
independent Linux/e2fsck checks across four profiles. Its full CI is still
pending in two VM suites, with all eight completed jobs successful. This new
change only narrows admission and does not extend write support.

Two new refusal regressions were red: the private planner returned `Ok(())`
when physical block **517** was both directory leaf 1 and the legacy
single-indirect mapping block. A thirteen-block directory reads that map block
only when reaching logical block 12, after earlier directory leaves have been
admitted. One test selects the aliased leaf for insertion; the other selects
the final ordinary leaf, proving that unselected leaves also need role checks.
A positive test with a separate mapping block passed before and after the fix.
These are synthetic, deliberately unallocated planner fixtures with unique
directory addresses and valid name routing/checksums, not Linux-clean images
or qualification of legacy-map writes.

`CasefoldMapDevice` now records physical blocks successfully read by the
existing extent/legacy mappers in a per-call set shared by disk and staged
views. After the complete directory admission walk, insertion refuses any
intersection with its directory-data addresses, before editing or staging the
copied leaf. Repeated reads of a shared mapping block remain allowed. Tracking
read locations in both views does not assume that identical returned mappings
necessarily traverse identical nodes. The mapper's filesystem bounds and
existing extent checksums and disk/staged mapping agreement are unchanged.

All three focused tests now pass. Their disk-only and identical-staged-node
cases execute; each refusal verifies unchanged dirty blocks, allocation flags
and device bytes. The positive cases verify the inserted name/inode, require
only the selected leaf to change and preserve device/allocation state. This
records visited mapping nodes only: unused/shadowed branches, complete map
shape, fixed metadata roles of mapping nodes, transaction-current descriptors
and allocation, journal/other ownership and operation-wide serialization are
still unqualified. Findings 2 and 3 remain open. Stage 5 completion remains
**not yet quantified**; public writes, deeper insertion and growth stay blocked.

The full local unit tier passed **810 tests**, zero failed or ignored, in
1,058 lines / 65,157 bytes within its unchanged budget. Multiset comparison
retains all 807 baseline occurrences and adds exactly the three new tests.
Lint, provenance, whitespace and shared-agent checks pass. The locked release
CLI build/install passes. Fresh logs are under ignored `tmp/casefold-map-role-evidence` and
`tmp/logs/casefold-map-role-{red,green}.log`. Independent focused Linux/full CI
for this new source is pending after push. Do not substitute the previous
checkpoint's successful focused run for new-source evidence.

**Next bounded task:** inspect both pending full-CI results and this new
checkpoint's focused insertion log. Resolve any failure before broadening
support. Then reproduce and qualify fixed-metadata aliases of mapping nodes
(the existing fixed-role check covers directory data only), retaining the
visited-node/data refusal and unchanged-state assertions. Keep transaction-current
allocation/descriptor state, journal ownership and complete mapping shape
explicitly open. Do not enable writes or intermediate-level insertion yet.

## Scoped fixed-metadata role refusal — 2026-10-14

Follow-up: its exact-commit focused insertion workflow at `3be872e` has now
passed and its retained log was inspected, as recorded above. Full CI remains
pending; the mapping-role checkpoint supersedes this section's next task.

The prerequisite full CI runs completed successfully:
[643fd2a / 38507056428](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38507056428),
[1ee1fed / 38509239857](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38509239857)
and [d6ae8d8 / 38510435499](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38510435499).
Downloaded the last run's `tier-logs-x86_64` artifact and confirmed the exact
healthy-image casefold insertion test passed in `suite.log`, and the upstream
crash-image recovery test passed in both `suite.log` and `kernel.log`. These
results qualify the preceding source, not the new refusal code.

A fresh **802-test** local baseline passed before applying the saved red patch.
Its three refusal tests failed again with `Ok(())` at metadata blocks 0, 2 and 5.
The expanded run had four expected negative-test failures and one positive pass:
it also reproduced acceptance of a metadata index root while its leaf remained
in ordinary data storage. The negative tests now cover primary superblock and
descriptor blocks, both bitmaps, interior/last inode-table blocks and metadata
roots. All five focused tests now pass, with unchanged dirty maps, pending
allocation flags and device bytes on every refusal. The positive test inserts
immediately after the inode table, verifies the stored name, and requires only
the selected leaf to change.

Admission checks each mapped directory block before reading its staged bytes.
The fixed-role check uses the existing, independently exercised group-head
layout helper for superblock/descriptor copies and reserved GDT regions, and
all mounted group descriptors for both bitmaps and complete inode-table ranges.
Checking every descriptor avoids assuming that FLEX_BG metadata lives in its
own group. Range comparisons avoid overflowing an inode-table end address;
group zero retains the special primary-superblock location at byte 1024.
This reuses established geometry and does not qualify new filesystem profiles.

This is deliberately a **mounted fixed-location refusal**, not proof that a
directory block is allocated or exclusively owned. Transaction-current
descriptor/allocation state, journal ownership, external map-node/data aliases
and other owners remain open in finding 2. Finding 3's complete mapping-shape
contract also remains open. Public casefold writes, intermediate-level
insertion, collisions, splitting/growth and index dropping remain blocked.
Stage 5 completion is **not yet quantified**.

The full local unit tier passed **807 tests**, zero failed or ignored, in
1,055 lines / 64,893 bytes within its unchanged budget. Multiset comparison
retains all 802 baseline test occurrences and adds exactly these five tests.
Lint, provenance, whitespace and shared-agent checks pass. The locked release
CLI build/install passes. Evidence is saved
under ignored `tmp/casefold-known-role-evidence` and
`tmp/logs/casefold-role-{red,expanded-red,green}.log`.
Independent focused Linux and full CI for the new fix are pending after push.
The old red patch is retained as historical reproduction evidence for `1ee1fed`;
its tests are already active now, so do not reapply it to the new source.

**Next bounded task:** inspect this fix's exact-commit focused insertion and
full CI outcomes, including retained independent insertion results. Resolve
any failure before extending admission. Then reproduce and qualify the next
remaining role counterexample (external map-node/data aliasing) with unchanged
buffer/device assertions. Retain staged-role/allocation, journal and mapping-shape
gaps explicitly; do not enable writes or deeper insertion at this checkpoint.

## Metadata-role regression preparation — 2026-10-13

Historical preparation: the 2026-10-14 checkpoint above supersedes its pending
CI and unapplied-test status. The patch is now incorporated and extended in
active tests; the commands below apply only to the recorded old source.

At source `1ee1fedf90f1056cc9f14d5550ecea4cbf460030`, a fresh local baseline
passed **802 unit tests**. Three new synthetic planner regressions were then
compiled and executed through the real tier/test runner. All three failed with
the private insertion helper returning `Ok(())` for a checksummed staged leaf
mapped to primary superblock block **0**, block bitmap block **2**, or inode-table
block **5**. The index root remains in its ordinary data block; a valid two-extent
inode map supplies the leaf. These reproduce part of review finding 2; they are
not Linux-clean images or evidence of an exposed public write path.

The test patch is [CASEFOLD-METADATA-ROLE-RED.patch](CASEFOLD-METADATA-ROLE-RED.patch).
It also specifies descriptor/inode-bitmap cases for the corrected implementation
and requires unchanged dirty blocks, pending allocation flags and device bytes
on refusal. The initial red runs stop at their failed refusal assertions, so
those later unchanged-state assertions are **not yet green evidence**.

The temporary edits were removed from `src/fs.rs` after capturing the red:
its bytes match the pre-test commit exactly. The patch remains applicable, but
is deliberately not active in the ordinary suite while waiting for the existing
CI prerequisite. No tests or assertions were disabled, and production code,
dependency pins, sibling checkouts and write gates are unchanged. Local baseline
and red logs are under ignored `tmp/casefold-role-evidence` and
`tmp/logs/casefold-role-red.log`. Later chats can reproduce the red by applying
the committed patch to the unchanged source and running:

```sh
cd /workspace/rust-fs-ext4
test "$(git hash-object src/fs.rs)" = "$(git rev-parse 1ee1fed:src/fs.rs)"
git apply --check --unidiff-zero docs/CASEFOLD-METADATA-ROLE-RED.patch
git apply --unidiff-zero docs/CASEFOLD-METADATA-ROLE-RED.patch
/workspace/.cloud-setup/run-casefold.sh scripts/tier.sh casefold-role-red casefold-role-red 100 12000 -- scripts/test.sh --locked --lib casefold_role_
```

The patch has zero context so embedded blank context lines do not violate the
repository's whitespace check. Verify the source identity as above before using
it; if newer implementation commits changed the file, adapt the regressions to
the new source rather than forcing the saved patch onto it.

At this pause point, budget-repair CI **38507056428** and documentation-checkpoint
CI **38509239857** still have VM suites in progress, with no reported job failures.
Their full results remain pending. No production admission fix has been applied.
Avoid extra status-only pushes while these runs finish.

**Next bounded task:** verify the completed CI outcomes and inspect any failures.
Once the prerequisite passes, apply and reproduce this patch, then implement a
scoped metadata-role refusal contract and prove these same tests green. Preserve
all baseline tests, buffered-state behavior and independent healthy-image Linux
evidence. Do not claim all of finding 2 closed: staged descriptor/allocation
state, journal ownership and map-node/data aliases still require qualification;
mapping-shape finding 3 also remains open. Do not enable writes or deeper insertion.

## Explicit slow-fuzz qualification — 2026-10-13

The pre-existing ignored `deep_walk_byte_flip_fuzz_never_panics` test was run
explicitly at source commit `643fd2a26d50e24b4d06d2b2872bfdd538cfbb22`, using
the `fixtures` artifact from that commit's CI run
[38507056428](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38507056428).
The four Linux-created htree, xattr, checksum-seed and inline images are
16/8/16/8 MiB respectively. No image was generated with host oracle tools or
mounted on this host. The test uses read-only in-memory copies.

The release run passed **one test, zero failed or ignored**, in 10.04 seconds.
It exercised 18,432 byte-corruption cases and 36 device truncations, requiring
the mount/audit/extent walks to return results without panicking. This establishes
the test's no-panic property on those images, not correctness of every successful
parse, writable casefold operations, concurrent mutation or crash recovery.
The tier log is 8 lines / 327 bytes; the existing floor reader confirms one
executed test and refuses ignored results for this explicitly named run.

The pinned core 0.3.0 and unchanged lock were verified through locked metadata
in the isolated source view. Four symlinks in ignored `test-disks/*.img` point to
the downloaded CI images; no existing files were overwritten. This avoids a
fixture-directory binding problem in the initial runtime attempt. The actual
core checkout remains clean at onboarding v0.3.6. Test code, its assertions and
the ordinary suite's selection were unchanged; the default suite's ignored
result remains distinct from this separate successful execution.

When these generated images and local helper are available, the tested commands
are:

```sh
cd /workspace/rust-fs-ext4
/workspace/.cloud-setup/run-casefold.sh scripts/tier.sh slow-fuzz slow-fuzz 100 10000 -- scripts/test.sh --locked --release --test fuzz_smoke deep_walk_byte_flip_fuzz_never_panics -- --exact --ignored
/workspace/.cloud-setup/run-casefold.sh bash scripts/core.sh test-floor --refuse-ignored slow-fuzz 1
```

The one-test floor belongs only to this exact-name ad hoc run; existing standard
tier floors are unchanged. Fresh local evidence is in ignored `tmp/logs/slow-fuzz.log`
and `tmp/slow-fuzz-evidence`, including fixture SHA-256 digests, source commit,
artifact run and locked metadata. Restore the fixtures safely from CI if absent;
do not substitute self-generated images or silently decline to run the test.

At this checkpoint, all eight completed jobs of CI run 38507056428 succeeded;
the x86_64 host/VM suite and the separately compiled guest suite were still
running. **Full CI remains pending**, not passed. This documentation-only
checkpoint preserves the tested source and all write gates.

**Next bounded task:** verify the budget-repair run's completed result and
inspect any failed tier. Once the CI prerequisite passes, reproduce the scoped
metadata/map-node role-validation counterexample from review finding 2, then
implement its admission/refusal contract with unchanged-buffer tests. Mapping
shape finding 3 remains open. Keep writes, collisions/growth and deeper insertion
blocked. Do not rerun completed research or report ordinary-suite ignored tests
as if they executed; use separate exact-source evidence where applicable.

## Kernel output-budget repair and CI evidence — 2026-10-13

Inspected exact-commit CI run
[38489085592](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38489085592)
at recovery checkpoint `0896577084ec20d8168d9720cfaf60ad27f3e31d`. Its retained
`tier-logs-x86_64` artifact contains **28 kernel tests passed, zero failed or
ignored**, in **464 lines / 40,920 bytes**. This exceeds the old 460-line cap by
four; replaying the captured output through the actual tier wrapper reproduces
exit **65**. The byte cap was 42,000, leaving little headroom.

The new recovery regression
`a_lazy_mount_writes_into_an_uninit_group_only_after_replaying_the_kernels_log`
executed and passed in both that kernel tier and the separately compiled guest
suite. Its kernel log records filesystem checking and cold kernel readback.
The guest job succeeded with **1,618 passed, zero failed and one ignored**:
the pre-existing slow `deep_walk_byte_flip_fuzz_never_panics` test was not run.
Do not describe that result as a zero-ignored or exhaustive suite pass. The
x86_64 job stopped at the kernel budget, so its later tiers were unrun.

`chores.yml` now caps kernel output at **620 lines / 55,000 bytes**, restoring
approximately **34% headroom in both dimensions** against the observed log.
The complete captured output passes these revised limits, with a byte comparison
confirming no output changed. The floor reader counts all 28 tests against the
unchanged floor of 3. Both existing output-budget/floor shell guards and all
**87 CI-configuration tests** pass; provenance, whitespace and shared-agent
checks pass. Production code, tests, selection, assertions, dependencies and
write gates are unchanged. This replay validates the configuration change;
it is not a fresh Linux test execution or proof that the new commit's CI passed.

The preceding boundary fix `fe7e088a3a115bee68d1ccab4d7ee0bfc8c37484` has successful
full CI (38331234132) and focused Stage 0 CI (38331234257). The focused artifact
was downloaded and its exact insertion log inspected: **one passed, zero failed
or ignored**, with the existing independent Linux/e2fsck assertions completed.

GitHub source, API status/permissions and real artifact downloads now work with
existing credentials; official Unicode and published ext4 references return
HTTP 200. Individual job-log retrieval additionally needs
`results-receiver.actions.githubusercontent.com`. That exact destination was
added to the environment draft while preserving prior rules and presets; apply
and publish that network change before claiming individual job-log access works.
Retained local evidence is under ignored `tmp/kernel-budget-evidence` and
`/workspace/.cloud-setup/ci-0896577`; later chats should use the CI artifacts.

**Next bounded task:** check this budget-repair commit's exact-commit CI and
resolve any remaining failures or ignored-test qualification before extending
index admission. Then reproduce the metadata/map-node role-validation gap in
review finding 2 and implement its scoped refusal contract. Finding 3 remains
open. Keep casefold write gates, collision/growth refusals and public APIs intact;
do not add intermediate-level insertion yet. Pause after this repair checkpoint.

## Upstream recovery integration — 2026-10-13

Backported upstream `antimatter-studios/rust-fs-ext4` commit
`d7519c7f90ef9ec2c538949459c0d4a68655d145` (#508). Finishing writable journal
recovery now restores the primary superblock's free block and inode totals from
the recovered group descriptors, including a clean journal with a leftover
recovery marker. The branch's earlier CASEFOLD/MMP/ENCRYPT refusal still executes
before either kind of recovery write. Casefold support and write gates are unchanged.

The upstream `tests/lazy_write_over_kernel_log.rs` is retained byte-for-byte.
It checks Linux crash recovery followed by writes into an uninitialised group,
then independent filesystem checking and kernel readback. It compiles locally
but **has not executed here**: this host lacks KVM, and the GitHub API route is
currently denied. Do not describe its assertions as passed at this checkpoint.

A new local test in `clean_journal_write_refusal.rs` reproduced recovery leaving
the free-block total at 0 instead of 6,902; it failed before the fix and passes
afterward. All four tests in that binary pass, including unchanged-byte/no-write
refusals for protected features. The full unit tier passes **802 tests, zero
failed or ignored**, preserving all 801 baseline passing occurrences as a multiset.
`chore lint`, `chore cli:install`, provenance and shared-agent checks pass.
No upstream dependency, CLI packaging, CI-policy or shared-agent upgrade was
merged. Source compatibility alone does not qualify the independent crash test.

Validation used the existing pinned-core archive through an isolated PRoot view,
restored under `/workspace/.cloud-setup`, with the GNU linker option described
below. Signed Debian package indexes and package checksums were retained. The
real core checkout remains clean at onboarding v0.3.6; locks and dependency
declarations are unchanged. Fresh local evidence is under ignored
`tmp/upstream-recovery-evidence` and `tmp/logs/recovery-recount-{red,green}.log`.

**Next bounded task:** verify this checkpoint's exact-commit full CI, explicitly
confirming the new Linux crash-image test executed and passed, and check the
boundary fix's focused Linux evidence. Resolve failures before implementing the
remaining metadata/map-node role-validation gap (review finding 2) and map-shape
qualification (finding 3). Do not enable writes or add intermediate insertion.

## Physical admission boundary fix — 2026-10-12

The review's physical-boundary finding is now reproduced and fixed in the
private `buffer_add_dir_entry_indexed` casefold path. `CasefoldMapDevice` bounds
external map reads in both disk and staged views to the mounted filesystem,
including when the backing device is larger. Every mapped directory data block
is checked before a staged byte vector can bypass device I/O. The existing
checksum, mapping-agreement and selected-leaf reuse checks remain unchanged.

Five new tests cover staged root/leaf addresses at and beyond `blocks_count`,
far-out addresses, valid outside bytes on a larger backing device, external
map nodes in both views, a staged-only out-of-range child returning the same
data mappings as disk, and acceptance at the last in-bounds directory block.
The four negative tests were reproduced red before the fix; all five now pass.
Each refusal verifies unchanged dirty blocks, pending allocation flags and
underlying device bytes. These are planner fixtures, not e2fsck-clean volumes.

Local validation completed in this sequence:

- Baseline unit tier: **796 passed, 0 failed, 0 ignored**.
- Before fix: **4 new refusal tests failed**, while the positive boundary test
  passed; the runner returned 101.
- After fix: **all 5 focused tests passed**.
- Full unit tier: **801 passed, 0 failed, 0 ignored**, with all 796 baseline
  passing test occurrences retained (multiset comparison).
- `chore lint` and `chore cli:install` passed, including frozen Unicode checks,
  formatting, Clippy and the release CLI build.

The preserved logs are under ignored `tmp/casefold-bounds-evidence` and
`tmp/logs/casefold-bounds-{red,green}.log` in this instance. They are not assumed
to sync between chats. Committed tests and fresh GitHub CI reproduce the result.

### Testing this cloud instance without moving the sibling core

The actual core checkout remains clean at onboarding v0.3.6. Tests used an
immutable-source archive of the prescribed v0.3.0 commit through a temporary
PRoot filesystem view. Cargo's locked metadata confirmed core 0.3.0, and the
actual sibling source/HEAD and repository locks were left unchanged. PRoot and
libtalloc were downloaded through signed Debian package indexes and extracted
outside the repositories. The view's default LLVM linker crashed; the supported
`-C linker-features=-lld` option selected the installed GNU linker. No compiler
assertions, integrity checks or test requirements were disabled. PyYAML 6.0.3
was installed in the workspace venv for core's existing CI-gate check.

When this runtime setup is retained, the tested commands are:

```sh
cd /workspace/rust-fs-ext4
/workspace/.cloud-setup/run-casefold.sh chore test:unit
/workspace/.cloud-setup/run-casefold.sh chore lint
/workspace/.cloud-setup/run-casefold.sh chore cli:install
```

The helper activates `/workspace/.cloud-setup/env.sh`, sets `FS_CORE_ROOT` to
`/workspace/.cloud-setup/pinned-core-0.3.0`, uses `target/casefold-baseline`, and
executes PRoot binding that archive at `/workspace/rust-fs-core` only for the
child command. Those local helper files are not guaranteed in a fresh task.
A correctly pinned ordinary workspace can use the standard handoff commands
without this source view. Verify the actual dependency version before testing;
do not silently update a lock or move a protected sibling as a workaround.

**Next bounded task using Sol 6.1:** check fresh focused Linux insertion and
full CI for this boundary-fix commit. Resolve any failures before extending
support. Then reproduce the metadata/map-node role-validation gap from review
finding 2 and implement a scoped admission contract; mapping-shape finding 3
also remains open. Keep casefold write gates, collision/growth refusals and
public API compatibility intact. Do not add intermediate-level insertion yet.
Pause after this tested boundary checkpoint.

## Index admission review checkpoint — 2026-10-12

The bounded Sol 6.1 review is complete at source commit
`1596988860edf2de9f0da02d8e0472d4adfddc89`. Findings and evidence are recorded in
[CASEFOLD-INDEX-REVIEW.md](CASEFOLD-INDEX-REVIEW.md).

The exact insertion test's retained focused artifact was downloaded from run
38134122031 and inspected: **1 passed, 0 failed, 0 ignored**, with four saved
1 KiB/4 KiB strict/non-strict images, and the Linux/e2fsck assertions completed.
The reviewed code/tests/focused workflow match that mapping-fix evidence commit.
The log measures 133 lines / 7,953 bytes. This is healthy-image evidence for the
private root-only primitive, not public writes, concurrency or crash recovery.

Admission validates routes/checksums and stages only a copied selected leaf
on success, but lacks an explicit physical filesystem-bound check. Its physical
uniqueness set also does not establish metadata/map-node roles, and checksummed
lookup does not establish complete map-shape validity. These are source review
findings; no new malformed-image regression has been executed or fixed here.
Public casefold writes remain blocked, so this does not claim an exposed write
regression or invalidate the healthy-image test result.

The boundary regression/fix is now complete in the follow-up above. Check its
fresh CI, then resolve the remaining role and map-shape qualification before
deeper index insertion. Keep every volume write gate intact.

This checkpoint changes review documentation only. No production code,
dependency declaration, lockfile, sibling source, or write gate was changed.
Pause after recording this review; do not start the implementation fix in the
same open-ended session.

## Model preference and confirmed CI — 2026-10-12

The user has chosen **Sol 6.1 for all implementation, testing, and review**.
This replaces the earlier recommendations to switch models. Keep the required
focused reviews of hashing/index design, serialization, crash consistency, and
write activation; perform them using Sol 6.1. Do not propose a model switch
unless the user changes this preference. The root `AGENTS.md` records the same
instruction for future chats.

The unit-budget repair commit `34e6a85a4911b294cb12321b26dcec725ec5c740` now has
**successful full CI**, verified directly through GitHub:
[run 38317933016](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38317933016).
This confirms the fresh CI gate after the measured budget adjustment; no code
or write-protection change accompanied that adjustment.

The planned root-only admission review is now complete; see the review
checkpoint above. Its findings must be reproduced and resolved before extending
admission to one intermediate level or relaxing any casefold write gate.

## Unit output-budget repair — 2026-10-12

`chores.yml` now budgets the unit tier at **1,400 lines and 85,000 bytes**.
The latest available unit artifact was retrieved directly from **CI run
38315213313**, commit `47e0d9a922fe75c7c190e447b6bcd38643e21d69`, artifact
`tier-logs-unit`. It again measures **1,052 lines / 64,504 bytes**, with
**796 passed, 0 failed, 0 ignored**. Its passed-test name multiset, retaining
repeated names, matches the artifact from run 38134210737.

The old line cap was exceeded by two; its byte cap had only 496 bytes of
headroom. The new limits restore approximately **33% line / 32% byte headroom**,
consistent with the measured headroom used by the surrounding tiers. Tests,
selection, assertions, the executed-test floor of 440, and log output are unchanged.

Validation exercised the repository's actual tier wrapper using the pinned core
v0.3.0 helper source from its recorded commit, without changing the sibling
checkout:

- Replaying the captured CI log under the old 1,050/65,000 limits reproduced
  exit **65**.
- Replaying the complete latest CI log under the limits parsed from the revised
  unit task passed. A byte comparison confirms the retained output is unchanged.
- The existing floor reader counted **796** tests against floor **440**, with
  ignored tests refused.
- Existing output-budget and tier-floor shell guards passed through the tier
  runner. They retain coverage that breached caps and empty test selections fail.
- Provenance, the protected shared agent block, and whitespace checks passed.

These are checks of the budget configuration using retained GitHub test results;
no fresh local Rust suite was run, and the core/harness sibling checkouts were
not changed. The pushed repair requires its own CI outcome. Check the remote
branch tip and that exact commit's CI rather than treating the replay as a fresh
full-suite pass. If CI is still running at the pause point, report it as pending.

**Next bounded task:** check the repair commit's CI and investigate any failure.
Once CI is green, use Sol 6.1 to review root-only index-write admission before
extending insertion to one intermediate level. Keep all casefold write gates
and unsupported-shape refusals intact. Do not broaden implementation at this
budget-repair checkpoint.

## Discovery and access follow-up — 2026-10-12

The repository-root `AGENTS.md` now points to this committed handoff and lists
initial access checks. Keep `casefold-rw` selected when resuming; `main` has not
been changed. Do not create a competing root-level progress file. Future chats
should fetch the latest remote branch, read its committed instructions, and
preserve newer or uncommitted work before making changes.

GitHub source/API access and reported push permissions are verified. A real
Actions artifact download also succeeded with the existing authentication.
Retained evidence comes from **CI run 38134210737**, commit
`25211664aaa3d77a3a07ecc32b1caa6f1b618a44`, artifact **tier-logs-unit**:

- **796 passed, 0 failed, 0 ignored** in the unit log.
- The log measures **1,052 lines and 64,504 bytes**.
- At that run the task allowed **1,050 lines and 65,000 bytes**. The line budget explains
  exit 65; the byte budget was not exceeded.
- The focused mapping-fix run **38134122031** reports success for both the
  driver-planned insertion step and the Linux fixture step. Its retained
  insertion log/images should still be inspected when reviewing index admission.

The environment draft preserves the `package_managers` network preset and adds
`api.github.com`, `*.blob.core.windows.net`, `unicode.org`, `www.unicode.org`, and
`docs.kernel.org`. The existing exact artifact hostname remains preserved.
The wildcard accounts for Actions storage hosts changing between runs. Review
and save the draft, then publish if required; saving alone does not apply these
additional routes. The observed repository/API/artifact operations already work.
No new secret or repository admin permission is needed for the current task.

The measured unit budget is now repaired as recorded above. Check its fresh CI
outcome before reviewing index-admission design and deeper support using Sol 6.1.
Casefold writes remain blocked. This follow-up changes documentation only and
uses GitHub test evidence; it does not claim a new local casefold suite ran.
The local sibling core is still onboarding's v0.3.6, whereas the recorded
casefold baseline used v0.3.0; neither sibling source has been changed.

## Planned stages and our position

These retain the original plan's Stage 0–7 numbering. Completed foundations cover the qualified subset, not every possible ext4 combination.

| Stage | Work | Current position |
|---|---|---|
| **0 — Establish Linux reference behavior** | Build Linux-created images, record encoding/settings, measure uncertain behavior, and validate with Linux and e2fsprogs. | Established for the current test profiles; extend the matrix when new write behavior needs qualification. |
| **1 — Recognize and validate casefold policy** | Read filesystem encoding and directory flags; reject unknown or inconsistent settings. | Implemented. Write restrictions remain in place. |
| **2 — Implement frozen Unicode comparison** | Use filesystem-compatible Unicode 12.1 rules, including normalization and full case folding, rather than current platform Unicode tables. | Implemented for the supported valid-UTF-8 subset. Malformed casefold names and normalized dot aliases remain refused. |
| **3 — Resolve names and retain actual entries** | Share comparison rules across lookup and mutation planning; return stored spelling and inode identity. Validate indexed lookup. | Implemented for supported read shapes, including linear directories and indexes with one intermediate level. Casefold indexed reads scan validated referenced leaves; this does not establish correct write routing. |
| **4 — Implement namespace mutations** | Make create, mkdir, links, symlinks, unlink, rmdir, and rename obey casefold identity and inheritance. Preserve transaction safety. | **Current stage: partially implemented.** Removal and rename retain/recheck matched entries. Duplicate checks use shared comparison. New-directory inode construction now inherits casefold correctly. Actual casefold writes remain blocked. |
| **5 — Implement and qualify directory-index writes** | Use frozen Unicode keys for hashing, insertion, collisions, leaf splits, and growth. Preserve Linux-visible routing. | **Hashing foundation, sampled read routing, and root-only in-place insertion implemented behind the volume write gate.** The new insertion primitive validates all referenced entries before changing one leaf. The preceding insertion/mapping source passed independent Linux and full CI. Fixed-metadata refusal passed 807 local units and exact-commit focused Linux validation, with full CI pending. The new visited mapping-node/data refusal passed 810 local units, lint and the CLI build; its exact-commit independent CI remains pending. Intermediate-level insertion, collisions, splitting/growth, and complete write qualification remain required; index dropping stays refused. |
| **6 — Prove recovery and enable qualified writes** | Test interrupted operations, journal recovery, errors, and refusals; enable only the combinations that pass. | Not yet reached. The ordinary-volume recovery recount regression passed independent Linux checking and readback; its full-CI output-budget repair and subsequent prerequisite checkpoints passed. Casefold operation/recovery qualification remains incomplete; keep repair and unsupported combinations blocked. |
| **7 — Integrate and qualify Windows** | Audit the Windows adapter's name conversion, caching, create/rename semantics, serialization, device ownership, flushing, and eject. Test alternating Windows/Linux use. | Not yet reached. Pin and audit an actual `ext4-win-driver` revision; later qualify specific SteamOS versions/devices. |

## What comes next

1. Check current full CI and exact-commit focused Linux insertion evidence, then continue the bounded role-validation task recorded in the current checkpoint. Root-only insertion and explicit fallback refusals are in place; complete ownership, mapping-shape and collision/growth qualification remain.
2. Finish creation and mutation planning, including serialization from duplicate checks through commit, stale-entry handling, and replacement semantics.
3. Implement and independently verify indexed insertion, collisions, splits, and supported growth. Keep unsupported growth as a safe refusal; do not silently drop the index.
4. Exercise complete write sequences against Linux/e2fsprogs, including interrupted writes and recovery, before changing the write gate.
5. Integrate the qualified core into a pinned Windows build and test real cross-OS workflows. Core casefold support alone does not establish Windows or SteamOS readiness.

## Rules for continuing

- Work only on `casefold-rw`; do not change `main` or the sibling repositories.
- Keep casefold writes blocked until the supported operation and recovery matrix passes independent validation.
- Preserve original filename bytes. Never treat an unsupported lookup or corruption error as proof that a name is absent.
- Keep encryption, unknown encoding settings, unqualified directory shapes, malformed casefold names, generic casefold-flag changes, and casefold repair refused until separately qualified.
- Follow the repository's clean-room rules: published specifications, official Unicode data, and black-box Linux/e2fsprogs observations are allowed; kernel/e2fsprogs implementation source is not.
- This workspace has no local KVM. Use the existing CI harness for Linux/e2fsprogs checks; do not run the oracle tools or mount test images on the host.
- Check the actual branch, commit, working tree, and CI at the beginning of each session. Give brief plain-language notes in chat, complete one useful checkpoint, and pause before usage becomes excessive.
- Use Sol 6.1 for all implementation, testing, and review. Perform focused reviews for hashing/index design, concurrency and crash consistency, and the decision to enable writes; do not recommend a model switch unless the user changes this preference.

## Instructions for a new chat

Treat this note as a handoff for ongoing implementation, not a request to restart the research or redo completed stages. The user authorizes useful implementation changes and checkpoint commits/pushes to `origin/casefold-rw`. Do not merge into or push to `main`. Do not change the sibling repositories. Keep explanations brief and understandable, and stop after a tested checkpoint rather than starting an open-ended run.

The next chat should first verify that this snapshot is still current. If later commits exist, inspect them and their CI results before choosing the next task; do not reset the branch to the commit recorded here. Preserve any uncommitted work. Read the applicable `AGENTS.md` and task instructions before editing.

### Workspace and commands

The current checkout is `/workspace/rust-fs-ext4`. Its origin is `https://github.com/Shinomorisama/rust-fs-ext4.git`. The sibling harness is pinned at `v0.4.0`, commit `7e0a84291c76fe4c762143f6f6d5787cc11749e7`.

Start with read-only state checks:

```sh
cd /workspace/rust-fs-ext4
git branch --show-current
git status --short
git log -5 --oneline
git rev-parse HEAD
git -C ../rust-fs-core status --short
git -C ../rust-fs-core describe --tags --exact-match
git -C ../fs-linux-test-harness status --short
git -C ../fs-linux-test-harness rev-parse HEAD
gh run list --branch casefold-rw --limit 5
```

The existing local toolchain uses:

```sh
export CARGO_HOME=/workspace/.cargo
export RUSTUP_HOME=/workspace/.rustup
export PATH=/workspace/bin:/workspace/.cargo/bin:$PATH
export CARGO_BUILD_JOBS=4
export CARGO_TARGET_DIR=/workspace/rust-fs-ext4/target/casefold-baseline
export XDG_CACHE_HOME=/workspace/.cache
export XDG_STATE_HOME=/workspace/.local/state
```

Use the repository's tasks, preserve command exit statuses, and inspect failure logs:

```sh
chore test:unit
chore lint
chore cli:install
```

Run them sequentially with failure handling; do not let a later successful command hide an earlier failure. Focused tests also run through `scripts/tier.sh` and `scripts/test.sh`. The full Linux/e2fsprogs/kernel suite runs in GitHub CI using the existing VM harness. GitHub CLI and HTTPS push worked in this workspace; reuse existing access before asking for credentials. A fresh environment may need the onboarding setup restored.

Before editing, preserve the baseline unit log. Afterward confirm all baseline tests still pass, counting repeated test names as a multiset rather than a set. Stage only intended files. Commit with the repository's conventional declarative subject and flat bullet body, then push with `git push origin HEAD:casefold-rw`. Check that CI starts for that exact commit. If it is still running at the pause point, report it as pending; do not claim it passed or spend an hour polling.

### Code map and completed behavior

- `src/superblock.rs` and `src/inode.rs`: validated encoding metadata and `directory_casefold_encoding`. Known encoding is UTF-8/Unicode 12.1; known flags are strict or non-strict. The volume feature alone does not make an ordinary directory case-insensitive.
- `src/casefold/unicode_12_1.rs` and `data/unicode/12.1.0/`: frozen Unicode engine and generated tables. `src/casefold/lookup.rs` contains `LookupName`, including fallible name preparation, comparison, duplicate detection, `hash_bytes()`, and the shared prepared-name `hash()` method.
- `src/casefold.rs`: legacy public `fold_name` and `casefold_name_hash` remain for API compatibility. Their newer Unicode dependencies and malformed-byte fallback must not govern filesystem casefold writes. Preserve public API compatibility while introducing the correct internal path.
- `src/path.rs`: casefold lookup validates supported directory/index structures, scans referenced leaves, and refuses duplicate equivalent entries. Literal `.` and `..` are handled separately. Unsupported depth, inline casefold directories, encryption, malformed casefold names, and normalized dot aliases remain refused. Supported indexed reads do not prove that insertion or split routing is safe.
- `src/fs.rs`: `lookup_entry_at` returns stored name bytes, inode, and recorded type. `find_dir_entry` supplies mutation preflight. `buffer_remove_matched_dir_entry` checks the stored spelling and expected inode against the current buffered transaction. Unlink, rmdir, and rename use it. This is not a general concurrency or inode-generation guarantee.
- Rename rereads its source parent through the transaction buffer after destination insertion/growth. Preserve that behavior: insertion can move the source entry or convert the parent layout. Existing same-inode hard-link/no-replace behavior must not be casually changed.
- Creation's shared planner rejects equivalent existing names. `build_directory_inode` now receives the parent, validates its policy, and inherits CASEFOLD before computing the inode checksum. Ordinary children, regular files, and symlinks do not acquire CASEFOLD merely because the volume supports it. This preparation is implemented behind the write gate; end-to-end casefold mkdir is not yet enabled.
- `src/htree.rs` now has internal `lookup_leaf_prepared`; `src/htree_mut.rs` has internal `plan_leaf_split_with_encoding`. Filesystem index routing and splitting call these prepared-policy paths. Public raw-name helper signatures and behavior remain compatible. `src/path.rs::casefold_insert_leaf` now admits private root-only in-place insertion after checking the whole current directory. `src/fs.rs::buffer_add_dir_entry_indexed` reads the buffered inode/blocks, verifies external extent nodes while mapping through both the device and the existing `StagedTreeNodes` overlay, and refuses when a referenced logical block has different physical locations in the two views. It retains the admitted logical-to-physical locations, inserts into a copied selected leaf, preserves requested spelling, restamps its checksum, and stages only that leaf after success. It does not reread the final location through the unchecked device-only mapper. Full-leaf refusal returns `Unsupported`, never the `OutOfBounds` growth signal. Intermediate insertion, collision boundaries, and splitting remain refused. `buffer_drop_htree_index` still refuses casefold before editing blocks.
- `src/features.rs` (`WRITE_BREAKING_INCOMPAT`) and mount/mutation checks in `src/fs.rs` keep casefold volumes write-protected. Preserve these protections during the next checkpoint.

### Completed insertion/mapping checkpoint and bounded next task

This checkpoint implements the private buffered insertion path for **root-only indexes, 1 KiB/4 KiB blocks, FILETYPE, metadata checksums, and known Unicode 12.1 policy**. Public writable mounts and namespace operations on casefold volumes remain refused. The primitive is preparation for later activation, not a completed writable filesystem.

Admission validates the current buffered inode, root layout/checksum/capacity, unique bounded logical and physical block references, and every referenced leaf's checksum and record layout. It requires every logical directory block to be referenced and every existing live name to route to its actual leaf using frozen comparison keys and effective hash version/seed. Equivalent existing names are corruption; an existing equivalent query returns `AlreadyExists`. Reads can still use their previously supported broader shapes.

Odd collision boundaries, repeated boundaries, intermediate levels, unknown profiles, malformed names, and unreferenced directory blocks are refused. These are deliberate restrictions on this first insertion primitive, not claims that Linux itself refuses those structures. A full leaf refuses without staging changes or attempting growth/index dropping. The copied leaf is staged only after successful insertion and checksum restamping, so failures do not alter the caller's buffer or device. Reads consult buffered removals and prior insertions. External extent checksums are now checked in both views, and differing disk/buffer mappings for referenced blocks are refused before staging. Identical staged nodes remain accepted. This is not concurrency protection, arbitrary map-change support, or qualification of index growth.

Nine added unit tests bring the tier from 783 to **792**, with no lost baseline tests. They cover prepared-key routing, stored spelling/inode/type, staged duplicate detection, checksum-valid misrouting, equivalent existing names, corruption in an unselected leaf, unsupported collision/depth shapes, full-leaf refusal, and corrupt-leaf refusal. A checksum-valid entry that absorbed the checksum tail was reproduced red, then refused by explicit record-span validation. Local full units, lint, and CLI build pass.

The safety review found that the previous insertion primitive consulted buffered directory bytes but used an unchecked device-only reader for external block maps. Three regression checks reproduced acceptance of a changed staged mapping, a corrupt extent checksum on disk, and a corrupt checksum in a staged node without changing its routes. All three were red before the fix, then green. A fourth test confirms an unchanged staged external map remains accepted and only the selected leaf changes. These are synthetic planner fixtures, not e2fsck-clean volumes. The full unit tier increased from 792 to **796**, with no baseline losses; lint and CLI build also pass.

The fix uses the existing checksummed extent mapper with inode/generation context in both views, compares each referenced logical block's mapping, and refuses disagreement. Legacy maps are also read through both views; their pointer blocks have no extent-tail checksum. It reuses the admitted selected-leaf physical address. The shared public mapper, ordinary-directory insertion, sibling crates, and all volume write gates remain unchanged. General serialization, metadata/data ownership and role validation, full mapping-shape qualification, and crash recovery remain subjects for the next index safety review and later stages.

The `src/fs.rs::tests::needs_host::casefold_inplace_index_insert_is_readable_by_linux_and_e2fsck_clean` test **passed at the previous checkpoint `a7db076`**. It creates four Linux-populated profiles (1 KiB/4 KiB, strict/non-strict), invokes the actual private buffered insertion primitive, completes hard-link accounting, and writes the staged bytes directly to a scratch image. It then requires non-repairing e2fsck cleanliness and cold Linux lookup of the new Unicode spelling and canonical/case alias, inode/payload agreement, Linux rejection of an equivalent duplicate, and readability of all 1,000 original indexed files. This tests the emitted bytes, **not** the public namespace API, journal commit, or crash recovery. It runs in full CI's whole-suite tier and now also runs explicitly as the first test step in `.github/workflows/casefold-stage0.yml`. That focused step uses the exact test name, requires one executed test, and keeps its log plus four scratch images in the existing `casefold-fixtures` artifact. Relevant casefold/index source changes on this branch trigger the focused workflow automatically. The existing fixture/routing tests remain after it. That previous run passed both the one-test insertion floor and the later fixture/routing step. The mapping-hardening focused workflow subsequently passed; the unit output-budget failure and measured artifact are recorded in the 2026-10-12 follow-up. No host mount or host e2fsprogs invocation was used.

The preceding routing checkpoint's full CI **38100380202** and Stage 0 **38100380219** both passed at `e4c3e85`. Its 52 direct selected-leaf probes across eight saved Linux images remain useful evidence for root-only and one-intermediate-level reads, distinct from qualification of the new insertion writes.

**Next bounded task:** use the latest 2026-10-14 current checkpoint above.
Inspect exact-commit focused Linux insertion and both pending full CI results.
Then reproduce and qualify fixed-metadata aliases of mapping nodes, retaining
visited mapping-node/data refusals and unchanged-buffer assertions. Transaction-current descriptor and
allocation state, journal/other ownership and complete mapping shape remain open.
Do not proceed to intermediate-level insertion until the remaining admission
review findings are resolved and independently qualified. The eventual deeper
insertion checkpoint must retain whole-directory uniqueness, complete entry
routing, current buffered state and Linux cold lookup/e2fsck evidence. Collision
chains, splitting/growth and index dropping remain refused.

Pause after a tested bounded checkpoint. Full namespace operations,
serialization, crash recovery, write activation and Windows integration remain
later work. Existing independent evidence qualifies the preceding source;
follow-up hardening requires its own exact-commit results. Preserve the focused
workflow's exact filter, one-test floor, saved images/logs, pinned siblings and
harness-only oracle/mount rules.

### Conditions still required before enabling writes

Supported namespace operations must agree with Linux on stored names, inode identity, duplicates, link counts, directory inheritance, and rename outcomes. Creation and removal must remain correct across directory growth and moves between ordinary and casefold parents. Resolve serialization of the complete operation, from absence checks through commit, across Rust and C APIs.

Index tests must prove Linux can find entries after driver insertion, collisions, splits, and supported depth transitions, using cold mounts and non-repairing e2fsck checks. Unimplemented growth must refuse safely. A successful full-leaf scan by this driver is insufficient evidence of correct Linux index routing.

Crash/error tests must establish acceptable pre- or post-operation states, safe recovery in both directions, and correct checksums and allocation accounting. Audit mount, deferred replay, writability transitions, mutation APIs, and repair so relaxing one guard does not expose an unqualified path. Qualify journal/geometry combinations explicitly rather than assuming all ext4 features work.

Windows integration still needs a selected repository revision and an audit of its adapter and mount lifecycle. The user has no Steam hardware or known final filesystem profile. Do not require that information to continue independent core work, and do not claim SteamOS-device compatibility until tested. Ensure Unicode licensing accompanies future redistributed binaries/tables.

## Where the evidence lives

Committed documentation: `docs/CASEFOLD-BEHAVIOR.md`, `docs/CASEFOLD-STAGE0.md`, and `docs/CASEFOLD-STAGE1.md`. Reference observations live in `test-disks/casefold-*-reference.json`; Linux fixture tests and guest scripts live under `tests/`. The older stage documents describe earlier checkpoints and are not a current completion checklist.

The original detailed plan is `/workspace/casefold-implementation-plan.md`, outside the repository. Its baseline findings and future-tense descriptions are historical; use this note and verified code/CI state for current progress. In particular, do not infer a safe malformed-name fallback from that old plan: later observations led to the current refusal policy. This note contains the continuation essentials even if that original file is unavailable.

Some saved Linux images and local logs live under the checkout's ignored `tmp/` directory. They are useful when present but are not guaranteed in a fresh cloud environment; use committed tests and CI artifacts to reproduce or retrieve evidence. The previous index-routing/refusal checkpoint passed full CI and fresh Stage 0 evidence; the subsequent root-only insertion passed focused Linux evidence at `a7db076`, with full CI also passed. The mapping fix now has a successful focused workflow; full CI is blocked by the measured unit output budget, as recorded in the follow-up above. No claim is made here that a separate coverage workflow ran at the latest commit.

This note is committed as `docs/extprogress.md` on `casefold-rw`. A new Codex session should open that branch, read this file, and verify the actual checkout and CI before continuing. Verify the selected commit against its own CI; earlier successful runs are evidence for those earlier checkpoints. Update this handoff as implementation checkpoints are completed.
