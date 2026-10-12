# extprogress

## What we are doing

We are extending `rust-fs-ext4` so a normal ext4 removable-storage card can eventually be read and written safely from Windows through `ext4-win-driver`, then used natively by Linux and SteamOS. The intended use is a 1.5 TB microSD card shared between these systems.

The main compatibility gap is **ext4 casefold**: selected directories treat different spellings as the same name according to the filesystem's declared Unicode rules. Correct support must cover finding names, preventing equivalent duplicates, changing directory entries, maintaining directory indexes, and recovering interrupted writes. Stored filenames must retain their original bytes.

The goal is a working implementation. Documentation, unit tests, and independent Linux/e2fsprogs checks support that goal. Passing this driver's own tests alone is not enough to establish compatibility.

**Casefold writes are still deliberately blocked. This is not yet a Windows-ready release for the card.** No actual SteamOS hardware or Windows/card round trip has been qualified.

## Current checkpoint

- Repository: [Shinomorisama/rust-fs-ext4](https://github.com/Shinomorisama/rust-fs-ext4), branch `casefold-rw`.
- Latest implementation work: root-only insertion now verifies external extent checksums in both disk and buffered views, refuses differing mappings, and reuses the physical leaf mapping it admitted, committed alongside this handoff update. Use `git log` to identify the current checkpoint.
- Mapping fix: `6402d5d82dcf5768105964b985cb1d914d351d80`. Previous checkpoint: `a7db0769d85a1cbea189a860d9a46fa1d293d3d9` (focused Linux insertion CI). The original insertion primitive landed in `248e879636c4e03f476a440dac48d316b42c0fe8`; this checkpoint hardens its mapping admission.
- Original baseline: `v0.8.0`, commit `204d8f6518ca5d66de07ab8e84c81501909b5d09`.
- Local validation: **796 unit tests pass; lint and CLI build pass**. All preceding baseline tests remain present and passing.
- Previous checkpoint `a7db076`: [focused Linux insertion and fixture evidence](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38131444906) **passed**. Its insertion step executed exactly one test covering four Linux profiles (133 log lines, 7,954 bytes). [Full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38131444887) also **passed**, confirmed in the final pre-push check. The mapping fix subsequently passed the focused Stage 0 workflow (38134122031). Full CI at the mapping fix (38134122021) and this handoff commit (38134210737) failed; the latest unit artifact confirms all 796 tests passed and the log exceeded its line budget by two lines. See the access follow-up below.
- Working tree clean at verification; the checkpoint is pushed to the fork.
- `rust-fs-core` remains at `v0.3.0` (`9e900154a7dff2a6c8f136db31def33bed9f09cc`). Neither it nor `fs-linux-test-harness` is to be modified.

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

**Next bounded task: use Sol 6.1 to review root-only index-write admission**,
including current buffered inode/extent state, mapping/metadata role validation,
existing-entry route checks, collision refusals, and unchanged-buffer behavior.
Inspect the retained focused Linux insertion log/images if needed. Resolve any
finding before extending admission to one intermediate level, and keep all
casefold write gates blocked. Pause after that review checkpoint.

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
| **5 — Implement and qualify directory-index writes** | Use frozen Unicode keys for hashing, insertion, collisions, leaf splits, and growth. Preserve Linux-visible routing. | **Hashing foundation, sampled read routing, and root-only in-place insertion implemented behind the volume write gate.** The new insertion primitive validates all referenced entries before changing one leaf. The original insertion bytes passed four Linux profiles; new mapping hardening is locally tested and needs fresh CI. Intermediate-level insertion, collisions, splitting/growth, and complete write qualification remain required; index dropping stays refused. |
| **6 — Prove recovery and enable qualified writes** | Test interrupted operations, journal recovery, errors, and refusals; enable only the combinations that pass. | Not yet reached. Keep casefold repair and unsupported feature combinations blocked. |
| **7 — Integrate and qualify Windows** | Audit the Windows adapter's name conversion, caching, create/rename semantics, serialization, device ownership, flushing, and eject. Test alternating Windows/Linux use. | Not yet reached. Pin and audit an actual `ext4-win-driver` revision; later qualify specific SteamOS versions/devices. |

## What comes next

1. Check current full CI and fresh Linux fixture results, then implement a bounded supported indexed insertion path behind the volume write gate. Direct routing probes and explicit fallback refusals are in place; complete write admission and collision/growth qualification remain.
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

**Next bounded task (Sol 6.1 review):** inspect the latest focused Stage 0 workflow and full CI, explicitly checking that `casefold_inplace_index_insert_is_readable_by_linux_and_e2fsck_clean` executed and passed. Inspect its retained insertion log/images if needed; compilation and the older fixture step alone do not qualify insertion. Resolve any failure before extending support. Review the remaining root-only admission design using Sol 6.1, including buffered inode/extent state after this fix, mapping/metadata role validation, route validation, collision refusals, and the unchanged-buffer guarantee. Then extend in-place admission/insertion to one intermediate level while retaining whole-directory uniqueness, complete existing-entry routing checks, current buffered state, and unchanged-buffer refusal. Include independent Linux cold lookup and e2fsck for those writes. Keep collision chains, full-leaf splitting/growth, and index dropping refused until separately implemented and qualified. Perform a focused Sol 6.1 review of the index-write admission/design before progressing to broader index mutations or enabling writes.

This continuation stops after a tested mapping-safety fix. Root-only insertion has independent Linux evidence at the previous commit, while the hardening now has a successful focused workflow and a measured unit output-budget failure in full CI. No deeper-index admission has been enabled. A focused Sol 6.1 review is still required before the deeper-index implementation; do not interpret this targeted fix as completion of that broader review. The focused workflow retains its exact filter, one-test floor, saved images/logs, pinned siblings, and harness-only oracle/mount rules.

Pause after that tested checkpoint. Complete namespace operations, serialization, crash recovery, write-gate activation, and Windows integration remain later work.

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
