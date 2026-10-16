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
- Latest public chmod integration: public Rust string/byte/inode chmod and its
  existing C delegates now share the reviewed mount guard. Acquisition and volume
  verdicts precede arbitrary inode conversion; path dispatch never enters twice.
  Public signatures/C error handling, metadata semantics, sticky failure and all
  casefold write refusals remain. Three public bypass/conversion controls executed
  red before routing and now pass. All 1,012 local units pass, retaining all 994
  baseline occurrences and adding eighteen public controls; lint and the locked
  release CLI build/install pass. New eight-profile exact public Rust/C VM
  evidence is recorded in the checkpoint below; CI execution
  and standalone migration review are next before another operation family.
  This is not whole-API concurrency or ownership readiness. Stage 5 remains
  not yet quantified and normal casefold writes stay blocked.
- Latest private mount review: `3ee3aa4` passed exact-commit full/focused CI.
  Downloaded the actual artifacts: all 18 controls executed in standalone,
  x86, aarch64 and guest-library contexts. All four new Linux images, eight
  debugfs reports and four Linux reports agree with the emitted metadata audit.
  No blocker found within the private family's externally exclusive contract.
  This is not whole-API concurrency, ownership readiness or casefold admission.
  The whole guest suite still ignored the pre-existing deep fuzz case; broader
  qualification stays gated. That review selected public chmod migration,
  implemented in the newest checkpoint above with acquisition before conversion
  and new public-entry controls/VM evidence. Its prior bounded contract remains
  recorded below; review the new public migration before another family.
- Latest private mount integration: private path-byte/inode chmod now borrows
  one mount context before lookup/callbacks, separates planning from marker and
  complete-commit effects, and latches uncertain/poison/unwind failure into common
  write/flush/faulted-drop refusal. All 18 focused scratch-mount controls pass.
  Four-profile exact-adapter Linux/e2fsck/debugfs evidence and the standalone
  private-family review now pass within the stated scope.
  Public delegates were legacy/unguarded at that checkpoint and now use this
  family in the new public migration above. Casefold writes remain blocked.
  See the private checkpoint/review below; Stage 5 is not yet quantified.
- Latest operation-control review: `b6e89b0` passed exact-commit full/focused
  CI. Actual artifacts show all 15 controls executed in standalone unit, x86,
  aarch64 and guest-library contexts, with no gate control failed/ignored.
  No blocker found within the private primitive's caller contract; no source
  change was needed in that review. The whole guest suite retains one pre-existing
  ignored deep fuzz case, so broader qualification remains gated. Its selected
  private chmod/health bridge is implemented in the newest checkpoint above;
  public delegates remain unguarded and writes blocked. See the 2026-10-16
  review for the bounded integration contract.
- Prior standalone operation-control implementation (`b6e89b0`): a disconnected
  crate-private gate provided nonblocking exclusion, control-bound freshness and
  checked invalidation/fault transitions. All 15 focused controls and 976 local units passed, including
  real thread/callback-worker contention and caught/uncaught panic handling.
  It had no Ready state, owner publication, Filesystem binding or BlockBuffer
  seal and changed no public entry point. Exact-commit CI and standalone review
  completed within that contract. The private mount adapter is now implemented
  above and needs independent evidence/family review before public migration;
  write gates and saved insertion aliases remain unchanged.
- Latest serialization/invalidation audit: builder source `772981a` and review
  `c3557f5` now pass full CI; the source's focused VM evidence also passed.
  Concrete Rust/C ABI, nested multi-commit, raw write/repair/replay, refresh,
  callback and public-field escape paths are mapped in the 2026-10-16 audit.
  That audit added no operation guard or reusable proof. The subsequent private
  gate is now implemented with 15 controls; review it before wrapper migration.
  Public writes and the three saved insertion aliases remain blocked/unresolved.
- Latest complete-owner review: `772981a` passed exact-commit focused VM
  evidence. Downloaded all six images/reports: every one of 222 allocated records
  and 215 typed claims agrees with independent tools and the public-format byte
  audit. No blocker found in the private constructor under its trusted coherent
  stable-view contract. Its source and review full CI now pass. The concrete
  serialization/mutation/invalidation audit is recorded above and below before
  implementing the guard or publishing an epoch.
  This review changes no source/test/workflow or write gate; saved insertion
  aliases remain unresolved. See the 2026-10-16 review below.
- Latest implementation: a private complete current-view ownership builder now
  joins the reviewed components under shared read/work/claim/memory budgets.
  It retains all allocated inode identities and typed fixed/data/map claims,
  checks whole-inode sector accounting, rejects global overlaps and checks
  current allocation once. Four initial controls showed three alias acceptances
  before the join and all four pass afterward; expanded controls pass 18 and
  the full local tier passes 961, retaining all 943 baseline occurrences.
  Its six-profile whole-result Linux oracle passed focused CI and the complete
  standalone implementation/evidence review is recorded above; source and review
  full CI now pass. The concrete serialization/invalidation audit is complete. The
  constructor is disconnected, public writes remain blocked, and the three saved
  insertion aliases remain unresolved. See the implementation checkpoint below.
- Latest allocation review: `72b6911` passed exact-commit focused VM and full CI.
  Inspected all four allocation images/reports: 5,108 complete domain answers
  across eight initialized groups, including both 4 KiB block-zero bits, agree
  with the bit/free-list reports and raw public-format audit. No blocker found
  within the standalone verifier's narrow coherent-view/fixed-layout contract.
  This documentation review selects a complete bounded private current-view
  ownership builder, with shared accounting and whole-result Linux evidence,
  before lifecycle/admission integration. It changes no source/test/workflow or
  gate; the three saved other-inode aliases remain unresolved.
- Latest allocation-oracle repair: `1fcb361` failed focused/full CI because
  `debugfs testb` rejects block zero on a 4 KiB filesystem. The test-only repair
  uses complete count-checked `dumpe2fs` free-block lists for the whole domain and
  retains explicit `debugfs` cross-checks for every nonzero block. Two local
  report-reader controls cover zero and incomplete reports. The production
  verifier, insertion helper and gates are unchanged. Repair `72b6911` passed
  VM/full CI; its evidence/implementation review is recorded above and below.
- Latest allocation-interval checkpoint: a private bounded checker now verifies
  physically sorted ranges against current initialized block bitmaps, including
  staged allocation/freeing, checksums, full padding and explicit shared caps.
  It expands no data run into owner entries and keeps two reusable scratch blocks.
  Local validation is recorded below. Its initial independent CI failed on an
  invalid block-zero oracle query; the test-only repair passed VM/full CI and
  the standalone evidence/contract review is complete within its stated scope.
  Admission and public casefold writes remain disconnected/blocked; the three
  saved other-inode aliases remain unresolved.
- Latest fixed-metadata review: `95cb218` passed exact-commit focused VM location
  and existing insertion/inventory/role/FIEMAP tests. Downloaded all 14 metadata
  images/reports covering 196 groups and 943 protected ranges. No blocker was
  found within the standalone profile and trusted coherent stable-view contract.
  Source full CI and review `67d96f6`'s full CI now passed. The review below
  selected the private bounded current block-allocation interval verifier
  implemented in the following checkpoint before the complete owner join.
  It changes no source or gate; the three other-inode aliases remain unresolved.
- Latest fixed-metadata checkpoint: a private bounded collector now validates
  current primary superblock/descriptors and returns physically sorted disjoint
  boot, superblock/descriptor/reserved-GDT, bitmap and full inode-table ranges.
  It covers backups and FLEX_BG placement, preserves uninitialized reservations,
  refuses relocations/unknown layouts/overlaps, and has explicit aggregate limits.
  All 17 focused controls and 921 full unit tests pass; independent VM location
  and existing Linux checks and full CI now passed.
  No admission or public write gate is connected or relaxed.
- Latest role review: `ef1ca24` passed exact-commit focused Linux and full CI.
  Downloaded and inspected six role images/reports (222 records), plus current
  insertion/inventory/FIEMAP logs. No blocker was found within the narrow private
  reader profile/preconditions. The review below records coverage/resource/view
  requirements and selects a private bounded fixed-metadata interval collector
  as the next component before a complete owner snapshot. It changes no source,
  admission or gate; the three other-inode alias failures remain unresolved.
- Latest inode-role checkpoint: a private bounded reader now verifies current
  inode bodies and returns explicit reserved/root/journal/ordinary/symlink/FIFO
  roles and qualified storage-header interpretations. It requires a successful
  same-view inventory and allocated IDs, refuses unknown profiles, preserves
  zero-link allocation, and shares record/read limits across calls. All 17
  focused controls and 904 full unit tests pass; exact-commit independent Linux
  and full CI now passed.
  It is disconnected from admission and does not build a complete snapshot.
- Latest inventory review: source `4309ed1` passed exact-commit focused Linux
  allocation/insertion/FIEMAP. Actual logs, four allocation images and all 256
  per-ID answers were inspected. No blocker was found within the standalone
  scan's stated profile/preconditions. The review below records current-view,
  coverage/resource limits and the next private inode-role classifier contract.
  Inventory source and its documentation review's full CI now passed. This
  review changes no gate.
- Latest inode-inventory checkpoint: a private bounded scan now validates
  current superblock/descriptor/initialized inode bitmap contents and returns
  every allocated ID, including reserved and zero-link records. It reads current
  staged bits, checks final-group/full-block padding and shares limits across
  groups. It interprets no inode bodies/roles and is disconnected from admission.
  Its independent Linux allocation controls now passed; all write gates remain.
- Latest collector review: the implementation/resource review at `7bdfe0c`
  found no blocker within the standalone collector's stated profile and caller
  preconditions. Exact-commit focused Linux FIEMAP/insertion and full CI passed;
  the actual artifact/logs/reports were inspected. The next private snapshot
  component is the bounded allocated-inode inventory implemented below. Its
  per-scan limits do not replace complete snapshot budgets, retained-capacity
  accounting or complete-result requirements.
  The depth-zero/one independent matrix is not qualification of deeper maps.
  This review changes no production source, owner admission or write gate.
- Latest collector checkpoint: a private bounded complete extent-ownership
  walker now reports every data extent (including unwritten/beyond-EOF storage)
  and external mapping node, verifies complete tree shape/checksums/bounds and
  rejects internal physical role overlap. It accepts the real staged-node view
  and has explicit read/record/claim/scratch limits. It is not connected to
  admission; the three cross-inode aliases and every public write gate remain.
  Its independent Linux/full CI now passed; see the review/checkpoint below.
- Latest review checkpoint: the journal guard's exact-commit focused Linux and
  full CI passed at `f9a0df0`. A focused transaction-current ownership design
  review selects a complete bounded epoch snapshot plus transaction overlay,
  conditional on operation-wide serialization and mutation/invalidation coverage.
  The three saved other-owner aliases were rerun and still fail on acceptance;
  their disjoint control passes. This review changes no production source or
  gate. The following collector checkpoint implements that private primitive,
  without connecting it to admission. See the review checkpoint below
  and `docs/CASEFOLD-INDEX-REVIEW.md` for contracts and implementation gates.
- Latest journal-admission checkpoint: private indexed insertion now refuses
  overlap with the declared internal journal's complete initialized inline
  extent map. It verifies a stable journal inode, current inode/block allocation,
  full mapping shape and disjoint directory/map roles. External or unqualified
  journal maps and staged journal inode changes remain unsupported. See the
  scoped journal checkpoint below. Its exact-commit independent CI now passed;
  other-inode ownership and the volume write gate are unchanged.
- Ownership-review checkpoint: current journal/other ownership was reviewed at
  `96b3b72`. Three allocated cross-inode aliases reproduce acceptance: a leaf
  owned by `lost+found`, a map node owned by a normal file, and map-node ownership
  introduced only in the current buffer. A disjoint-storage control passes.
  The self-contained tests are saved in `docs/CASEFOLD-OWNERSHIP-RED.patch`;
  production source and write gates are unchanged. This establishes the gaps,
  not their fix. See the ownership review below for journal profile observations,
  scope, validation and next task.
- Latest admission checkpoint: private insertion now checks current descriptor
  flags/checksums and allocation bitmap checksums/bits for every admitted
  directory block and mapping node visited in either view. The three saved
  allocation regressions are active; positive planner fixtures explicitly stage
  their allocations. See the current-allocation checkpoint below for scope,
  validation and next task. Its exact-commit focused Linux and full CI now pass.
  This does not prove exclusive storage ownership.
- Latest integration checkpoint: upstream ordinary-volume recovery recount fix
  `d7519c7f90ef9ec2c538949459c0d4a68655d145` (#508) is backported with its Linux
  crash-image test and a new local regression. Local units, lint and CLI build
  pass. Its independent Linux crash-image test now passed in both kernel and
  guest tiers; full CI at `0896577` failed because the kernel log exceeded its
  line budget. The measured repair's full CI now passed at `643fd2a`; subsequent
  full runs at `1ee1fed` and `d6ae8d8` also passed.
- Prior staged-superblock implementation (`b7b0325`) admits only checksum-valid
  free-counter edits to mounted contents. Other edits remain unsupported. Its
  exact-commit focused Linux and full CI now passed; the subsequent allocation
  review (`065f9a8`) passed full CI too. Their bounds, fixed metadata, mapping
  overlap, layout/checksum and disk/staged mapping checks remain. New-source
  independent CI is separate from those preceding successful runs.
- Mapping fix: `6402d5d82dcf5768105964b985cb1d914d351d80`. Previous checkpoint: `a7db0769d85a1cbea189a860d9a46fa1d293d3d9` (focused Linux insertion CI). The original insertion primitive landed in `248e879636c4e03f476a440dac48d316b42c0fe8`; this checkpoint hardens its mapping admission.
- Original baseline: `v0.8.0`, commit `204d8f6518ca5d66de07ab8e84c81501909b5d09`.
- Current local review: **943 units pass**, zero failed/ignored, with identical
  fresh baseline/final occurrence multisets and no additions/losses. The twenty
  allocation and two oracle-reader controls remain retained. Fresh lint and
  documentation checks pass. Source `72b6911` passed focused VM/full CI and its
  standalone review is complete; this review changes only documentation.
  The preceding locked release CLI build and CI CLI job passed.
- Previous local validation: the fixed-metadata documentation review passed **921 unit tests**,
  with identical baseline/final test-occurrence multisets, zero failed/ignored
  and no additions or losses. Fresh lint and documentation checks pass.
  The preceding implementation retained all 904 baseline occurrences and added
  exactly 17 fixed-metadata controls; its focused controls and locked release
  CLI build/install passed. Previous role/inventory/collector/staged controls,
  casefold tests and journal refusals remain retained.
  Source `4309ed1` passed focused Linux allocation/insertion/FIEMAP/full CI.
  Role source `ef1ca24` also passed focused Linux/full CI; its implementation
  review is complete within the standalone profile. This review's fresh local
  baseline retention and documentation checks are recorded below.
  The separate other-inode ownership patch still records three unresolved
  refusals and its passing disjoint control.
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
  now **passed**, including both VM suites and the aggregate gate.
- Mapping-overlap checkpoint `8f94cf8` passed its exact-commit
  [focused Linux workflow](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38547095289)
  and [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38547095190).
  Downloaded its insertion log: exactly one passed, zero failed or ignored,
  covering four profiles (133 lines / 7,954 bytes). These results qualify the
  preceding source; its then-pending mapping-metadata successor also passed below.
- Mapping-metadata checkpoint `986808c` passed its exact-commit
  [focused Linux workflow](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38604667122)
  and [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38604667297),
  including both VM suites and the aggregate gate. Downloaded its insertion
  log: exactly one passed, zero failed or ignored, covering the four existing
  profiles (133 lines / 7,954 bytes). These results qualify the preceding source.
- Staged-descriptor checkpoint `1a6e8d1` passed its exact-commit
  [focused Linux workflow](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38620539030)
  and [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38620539258),
  including both VM suites and the aggregate gate. Downloaded its insertion
  log: exactly one passed, zero failed or ignored, covering four existing
  profiles (133 lines / 7,953 bytes). These results qualify the preceding source.
- Staged-superblock checkpoint `b7b0325` passed its exact-commit
  [focused Linux workflow](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38642467164).
  Downloaded and inspected the insertion log: exactly one passed, zero failed
  or ignored, across four existing profiles (133 lines / 7,954 bytes). Its
  [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38642467254)
  now **passed**, including both VM suites and the aggregate gate.
- Allocation-review checkpoint `065f9a8` passed exact-commit
  [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38646510239),
  including both VM suites and the aggregate gate. Its source matched `b7b0325`;
  no focused workflow was triggered by the documentation-only change. These
  results do not qualify this new allocation guard.
- Allocation guard `96b3b72` passed exact-commit
  [focused Linux insertion](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38656514666)
  and [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38656514635),
  including both VM suites and the aggregate gate. Downloaded and inspected its
  insertion log: one passed, zero failed or ignored, across all four existing
  profiles (133 lines / 7,953 bytes). This ownership review changes no production
  source; it does not add independent qualification of ownership refusals.
- Ownership-review full CI at `da2ffcd`
  ([run](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38664567403))
  now **passed**. That checkpoint changed no production source. Its result is
  separate from the journal guard's own successful validation below.
- Journal guard `f9a0df0544072b0e252d5d4ff2912c70f1838a0b` passed
  [focused Linux validation](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38666477495)
  and [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38666477535),
  including both VM suites and the aggregate gate. Its downloaded insertion
  log confirms exactly one passed test, zero failed/ignored, across four existing
  profiles (133 lines / 7,953 bytes). This is emitted-byte qualification for the
  existing private insertion path, not a global ownership or public-write proof.
- Collector `7bdfe0cf83d3d65f4068225707c379f3fad9e6aa` passed
  [focused Linux validation](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38675564104)
  and [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38675564205),
  including both VM suites and the aggregate gate. Downloaded the actual focused
  artifact: insertion executes once across four existing profiles (133 lines /
  7,954 bytes); the new FIEMAP test executes once across two block sizes with
  inline/external files (23 lines / 1,601 bytes). Both have zero failed/ignored.
  Saved FIEMAP reports and images confirm unwritten allocation beyond EOF and
  Linux's total data/map sector counts. The owner-design review's full CI at
  `80caea6` ([run](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38673684102))
  also passed. These results do not qualify global ownership or public writes.
- Inventory `4309ed114caca9d13c31e6647276ca883c116781` passed
  [focused Linux validation](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38685893409).
  Downloaded and inspected its actual artifact: inventory executes once across
  four block/descriptor profiles (33 lines / 3,357 bytes), insertion once across
  four existing profiles (133 lines / 7,953 bytes), and FIEMAP once across both
  block sizes (23 lines / 1,601 bytes). All have zero failed/ignored. Its
  [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38685893425)
  now **passed**, including both VM suites and the aggregate gate. The
  documentation review `d650d8e` passed its own
  [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38687585020),
  including the aggregate gate. The preceding collector review `7825062` passed
  [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38682775335).
- Role reader `ef1ca24d89a2b38a0e3a035c63466628bf3daceb` passed
  [focused Linux validation](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38691730821)
  and [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38691730797),
  including both VM suites and the aggregate gate. Downloaded its actual focused
  artifact: roles execute once across six profiles (41 lines / 4,671 bytes),
  inventory once across four profiles (33 lines / 3,357 bytes), insertion once
  across four profiles (133 lines / 7,954 bytes), and FIEMAP once across both
  block sizes (23 lines / 1,601 bytes). All have zero failed/ignored. Saved
  type/sector reports cover all 37 allocated records in each role image,
  including reserved slots and the declared journal: 222 records in total.
- Role review `c1afa4771014233363a0978ddeb572f894c6a4f5` passed exact-commit
  [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38705064937),
  including both VM suites and the aggregate gate. That documentation-only
  review did not qualify the new fixed-metadata source implemented below.
- Fixed metadata `95cb21810b0e43edfbabcaf1b7d55e5d83d23787` passed
  [focused VM/Linux validation](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38771333034).
  Downloaded its actual artifact: metadata executes once across 14 profiles
  (59 lines / 7,385 bytes), roles once across six profiles (41 / 4,671), inventory
  once across four profiles (33 / 3,357), insertion once across four profiles
  (133 / 7,954), and FIEMAP once across both block sizes (23 / 1,601). All have
  zero failed/ignored. All 943 metadata ranges across 196 groups agree with saved
  VM `dumpe2fs` reports. Its [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38771333399)
  now passed, including both VM suites and the aggregate gate. Documentation
  review `67d96f6088bbc9545763b0dbc55f4de3878ea551` also passed exact-commit
  [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38774824704).
  These results do not qualify the new allocation verifier or a complete snapshot.
- Previous checkpoint `a7db076`: [focused Linux insertion and fixture evidence](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38131444906) **passed**. Its insertion step executed exactly one test covering four Linux profiles (133 log lines, 7,954 bytes). [Full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38131444887) also **passed**, confirmed in the final pre-push check. The mapping fix subsequently passed the focused Stage 0 workflow (38134122031). Full CI at the mapping fix (38134122021) and the earlier handoff commit (38134210737) failed; that unit artifact confirms all 796 tests passed and the log exceeded its line budget by two lines. See the access follow-up below.
- Working tree clean at verification; the checkpoint is pushed to the fork.
- The required core pin remains `v0.3.0`
  (`9e900154a7dff2a6c8f136db31def33bed9f09cc`). This cloud instance uses the
  immutable pinned archive through the documented source view; its actual
  sibling checkout stays clean at onboarding `v0.3.6`. Neither sibling nor
  dependency pins/locks are to be modified.

## Public chmod dispatch checkpoint — 2026-10-16

Started from `e16a0528fb921dc6344a7ceaf115a2498239829d`, equal to fetched
`origin/casefold-rw` with a clean tree. Its [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38970737249)
passed; the preceding private implementation's exact full/focused Linux CI and
four-profile image/evidence review remain recorded below. Existing Git/API
pull/push access and the prepared pinned Rust/core source view were reused.

**Implemented scope:** public Rust string/byte-path chmod and generic inode
chmod now delegate into the reviewed mount-bound family. The concrete private
inode entry stays available for its original controls. A shared generic helper
enters once and checks verdict-only volume refusals before calling arbitrary
`Into<InodeRef>` code. Conversion happens before a marker write and remains
inside the context, so overlap cannot execute it and unwind cannot heal the
mount. Path dispatch reuses its context in the private inode helper rather than
calling a separately guarded public inode entry. Both existing C delegates
inherit this routing without production C ABI/signature/reference changes.

Mode masking, file type, generation checks, ctime/checksum updates and volume
refusals stay shared. Public missing/stale planning refusals now follow the
reviewed private ordering before any marker write. Marker and complete-commit
effects, sticky failure, common write/flush/Drop refusal and checked invalidation
remain intact. No owner builder/cache, Ready state, buffer seal or casefold
write admission is introduced. All other mutators/readers/maintenance/raw device
or field access and C ABI caller lifetime/reference validity still require
external exclusion; this partial public guard does not establish whole-API
concurrency or transaction-current ownership.

**Executed red/green:** three actual public-entry controls failed before routing:
public Rust path and C path dispatch accepted a write while the gate was held,
and the generic inode conversion ran while it was held. The same controls now
pass. Expanded public controls cover Rust/C overlap, conversion ordering and
panic, real callback-worker reentry, pre-marker planning refusals, byte names,
generation/mode/type/ctime/checksum semantics, every observed direct/journal
write and flush through path/inode dispatch, C error clearing/reporting and
sticky refusal after caught planning/effect panics. Panics are injected in Rust
inside `ffi_guard`, never through non-unwinding extern C callbacks. An initial
expanded test run used generation zero as an any-generation sentinel; correcting
the fixture to the unchanged `FS_EXT4_GEN_ANY` constant removed those test-input
failures. No generation or error semantics were changed to make tests pass.
The final C close control first completes a public write, then catches a Rust
planning panic with the dirty marker owed and ordinary commit flags still healthy.
It releases the handle through exported `fs_ext4_umount`: zero device callbacks
occur and every byte, including the dirty marker, remains unchanged. Test-only
opaque-handle helpers own their boxes and add no production ABI.

**Independent evidence added, pending execution:** new exact test
`fs::guarded_chmod::needs_host::linux_checks_public_chmod_metadata` exercises
public Rust and C dispatch separately over eight Linux-populated checksummed
images (1 KiB/4 KiB, journal/no-journal for each interface). It proves held-gate
public refusals preserve image bytes, then calls public path/inode success,
requires the expected effect versions, and verifies metadata with judged e2fsck,
debugfs and Linux. The old private four-profile exact test remains selected in
its existing step, using the same shared fixture/report checks. Focused CI adds
the new exact one-test floor and saves eight images, sixteen debugfs reports,
eight Linux reports and its log alongside the retained private evidence. Local
compilation does not claim VM execution; this host lacks KVM and no host oracle
tool or mount was used.

Fresh full local units pass **1,012**, zero failed/ignored, retaining every
**994** baseline occurrence as a Counter multiset and adding exactly **18**
public controls. All eighteen private controls remain retained. The log is
**1,260 lines / 82,671 bytes**, within unchanged 1,400-line / 85,000-byte caps.
Fresh lint (formatting, all-target clippy, shared/provenance instructions and
frozen Unicode checks), the locked release CLI build/install, whitespace and
saved insertion-red patch applicability pass. Exact-commit full/focused CI and
the new VM execution are pending at push, not inferred from prior
private-entry results. The pre-existing ignored deep-fuzz guest qualification
remains unresolved; this local tier does not execute it.
Focused validation passes the retained eighteen private controls plus seventeen
public controls in one 35-test run (140 lines / 9,720 bytes), and the final
exported-close control separately (one passed, zero failed/ignored). That is
eighteen new public controls in total, not thirty-six new tests.
Ignored evidence under `tmp/casefold-public-chmod` retains the executed red,
expanded controls, baseline retention and validation logs. No sibling, dependency
pin/lock, environment draft or publication is changed.

**Next bounded task:** review this public chmod migration and its actual exact-
commit full/focused CI artifacts, including all eight new Rust/C images/reports
and the retained private oracle. Resolve any failure before another operation
family. Public C panic handling, generic conversion ordering, context reuse,
failure retirement and caller alias/lifetime limits require explicit review.
Caller-owned argument destructors/raw access are outside the conversion/callback
ordering claim; refusing conversion does not suppress normal Rust argument drop.
The three insertion aliases, ignored deep-fuzz qualification, transaction
dependencies/overlays/seals/publication and full map/collision/growth work remain
open. Stage 5 is **not yet quantified**; casefold writes, deeper insertion,
growth/index dropping and Windows/SteamOS/actual-card qualification stay gated.

## Private mount chmod implementation/evidence review — 2026-10-16

Reviewed `3ee3aa440a962e517092f0935bfe20fed3a068f0`, equal to fetched
`origin/casefold-rw` with a clean tree. Exact-commit [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38950053488)
and [focused Linux evidence](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38950053446)
passed. Downloaded actual artifacts, rather than inferring execution from job
conclusions. All eighteen scratch-mount control names match the fresh baseline
as Counter multisets in each context:

| Evidence | Private controls executed | Enclosing result | Lines / bytes |
|---|---|---|---|
| Standalone unit | 18 | 994 passed, zero failed/ignored | 1,250 / 81,367 |
| x86 host unit | 18 | 994 passed, zero failed/ignored | 1,244 / 80,930 |
| aarch64 unit | 18 | 994 passed, zero failed/ignored | 1,244 / 80,930 |
| Compiled guest library | 18 | 689 passed, zero failed/ignored | Within whole guest log |

These are eighteen controls in four contexts, not seventy-two different tests.
The exact new private-adapter oracle also executed in the guest library and
as the focused one-test step: **1 passed, zero failed/ignored**, **49 lines /
4,960 bytes**, with 24 oracle and eight kernel invocation markers. All four
1 KiB/4 KiB journal/no-journal profiles completed. The **whole guest suite**
reports **1,817 passed, zero failed, one ignored** (3,370 lines / 177,796 bytes).
The ignored `fuzz_smoke::deep_walk_byte_flip_fuzz_never_panics` is the existing
slow corrupt-input case. It remains an unresolved broader qualification gate;
none of the private controls or the new VM oracle was ignored. This review
does not waive it, remove the ignore or launch an unrelated fuzz sweep.

Inspected all four saved images, eight pre-Linux-readback debugfs reports and
four Linux reports. Each emitted file/directory record has the expected type,
mode 06751/02750, inode identity/generation and exact post-2038 ctime. Linux
asserted preserved atime/mtime and original file data; judged e2fsck passed at
input, after driver close and after Linux readback. A separate public-format
byte audit checked final superblock/descriptor/inode checksums, clean state,
allocation bits, both inode records per profile, ctime epoch bits, mtime and
file payload. The kernel's final file read can update atime and its checksum
after the saved debugfs report; that expected difference is not a checksum
failure. Directory atime remains the pinned input value. No host mount or host
oracle tool was used; raw image inspection does not import implementation code.

**Decision:** no blocker found within this private family's stated contract.
One borrowed non-Send context enters before filesystem callbacks/lookup and
foreign context checks precede even device properties. Nested dispatch reuses
the context. Verdict/generation/raw-block planning has no marker write; marker
and whole-commit effects invalidate before I/O and propagate failures through
all completion checks. The unwind latch runs before an outer catcher can return,
including a caught effect panic with a live context. Common write verdicts,
flush and Drop consult sticky failure before callbacks; refresh/new control
cannot heal it. The public delegates still use their legacy unguarded path.
No owner cache/Ready state, buffer seal, builder invocation or admission is
provided. Callback duration, swallowed callback errors, panic-abort/foreign
unwind, raw handles/fields and caller device ownership are outside this claim.
Other public operations/readers/maintenance and C ABI lifetime/reference creation
remain externally serialized; this guard cannot repair unsafe caller aliasing.

Fresh baseline and final local units both pass **994**, zero failed/ignored,
with identical Counter multisets and no additions/losses. The final log is
**1,241 lines / 80,823 bytes**, within unchanged caps (baseline: 80,824 bytes).
Source/test/workflow hashes remain unchanged during this review; only the
existing handoff and review documents change. Fresh lint, shared/provenance
instructions, whitespace and saved insertion-red patch applicability pass.
The source's exact-commit release CLI CI passed; no fresh release rebuild or
local VM run is claimed. Evidence is under ignored `tmp/casefold-chmod-review`,
including downloaded artifacts, scoped CI comparisons, image audit and hashes.
No sibling, dependency pin/lock, environment draft or publication changed.

**Next bounded implementation: public chmod dispatch only.** Use the detailed
contract in `docs/CASEFOLD-INDEX-REVIEW.md`: route public Rust string/byte and
inode chmod through the reviewed private family, so existing C chmod delegates
inherit the same guard without recursively entering through a public inode call.
Enter before arbitrary `Into<InodeRef>` conversion as well as filesystem/runtime
callbacks. Preserve signatures, byte paths, generation checks, metadata and
read-only/unsupported/casefold refusals; preserve the sticky failure bridge.
Require actual public Rust/C callback/failure/panic controls and exact public-
dispatch Linux evidence. Review that migration before adding another family.
This partial migration is not whole-API concurrency safety. Keep all casefold
write gates, the three insertion aliases, ignored-fuzz qualification and pending
ownership/overlay/seal/publication/map/collision/growth work open. Stage 5 remains
**not yet quantified**; Windows/SteamOS/actual-card qualification remains later.

## Private mount chmod and failure-bridge checkpoint — 2026-10-16

Started at `1b754fa1c24d7d3eaae59f053ab678ab24179294`, equal to fetched
`origin/casefold-rw` with a clean tree. That review's [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38937506330)
passed. Reused the prepared pinned Rust/core source view and existing Git/API
pull/push access; no environment draft, dependency pin/lock or sibling changed.

**Implemented scope:** each mount owns one private control and sticky failure
flag. Only the new crate-private path-byte/inode chmod entries acquire it. Their
context borrows the mount; identity checks precede even device property calls.
Path dispatch reuses that context for inode dispatch. Shared planning retains
the public chmod's low twelve permission bits, file type, generation, ctime and
checksum semantics. The new private path completes verdict/lookup/generation/raw
inode/block planning before writing the early marker, so ordinary planning
refusals preserve device bytes. Public Rust/C delegates retain their original
unguarded dispatch and marker ordering; external exclusive access is required.

The marker's complete write/flush and the entire existing direct/journal commit
(including completion/allocation-state publication) run in separate effects.
Invalidation precedes each effect; errors or panics cannot publish Ready or
restore freshness. A small unwind guard latches failure even if a caller catches
an effect panic while retaining the context. Context destruction also latches
unwind/unhealthy state before an outer ABI catcher returns. Unhealthy/poisoned
entry latches failure; ordinary overlap does not. The sticky flag is checked by
common write verdicts, flush and Drop before callbacks. Refresh or a replacement
control cannot clear it. No owner builder/cache, buffer seal, Ready state or
public wrapper migration is introduced.

**Executed red/green:** the new marker-flush regression first delegated through
the old chmod path. It failed with `legacy write reused failed mount` after the
first marker flush fault, before an ordinary commit failure flag existed. The
same check passes with the bridge. The expanded **18 controls pass**, covering
actual formatted scratch mounts, invalid-UTF-8 byte paths, inode generation,
mode/type/checksum/ctime, untouched refusal bytes/allocation flags, same-thread
and worker overlap, runtime callback-worker reentry, every observed write and
flush in direct and journaled marker/commit sequences, caught/uncaught effect
panics, planning callback panic, legacy write/flush refusal, no faulted-drop
callbacks/clean restoration, and sticky failure across refresh/control replacement.
A later planning refusal retains earlier effect versions and a healthy close
restores the clean marker. Unit controls alone establish no format qualification.

**Independent evidence added, pending execution:** exact test
`fs::guarded_chmod::needs_host::linux_checks_private_chmod_metadata` creates four
Linux-populated checksummed 1 KiB/4 KiB images, with/without an internal journal.
It calls both exact private entries, requires judged e2fsck before/after, debugfs
file/directory mode/type/generation/checksum and post-2038 ctime reports, then a
Linux mount verifies modes, exact ctime, preserved atime/mtime and original data.
The existing focused workflow retains its steps/floors and adds a one-test floor
and saved images/eight debugfs reports/four Linux reports in `casefold-fixtures`.
The local host lacks KVM; no local oracle execution or host mount is claimed.

Fresh full local units pass **994**, zero failed/ignored, retaining all **976**
baseline occurrences as a Counter multiset and adding exactly these **18**
controls. The full log is **1,242 lines / 80,883 bytes**, within unchanged
1,400-line / 85,000-byte caps. Lint (formatting, all-target clippy, provenance,
shared instructions and frozen Unicode checks), whitespace and applicability of
the saved insertion-alias red patch pass. The locked release CLI build/install
also passes. The new VM test compiles but is unrun
locally. Exact-commit full/focused CI is pending at push and must be reviewed.
The earlier ignored deep-fuzz guest qualification remains unresolved.
Ignored local evidence is under `tmp/casefold-private-chmod`, including the
executed red and passing focused controls; CI supplies durable VM artifacts.

**Next bounded task:** review this private family's implementation and actual
exact-commit full/focused CI artifacts, including all four images and reports.
Resolve any failure before broader/public wiring. Existing public mutators,
readers, repair/replay/refresh, raw fields/devices and C ABI reference/lifetime
paths remain outside exclusion; this does not establish concurrent API safety.
The ignored deep fuzz qualification, three saved insertion aliases, transaction
dependencies/overlays/seals/publication and full map/collision/growth work remain
open. Stage 5 remains **not yet quantified**; casefold writes, deeper insertion,
growth/index dropping and Windows/SteamOS/actual-card qualification stay gated.

## Operation-control implementation/evidence review — 2026-10-16

Reviewed `b6e89b0a8e17d6ece855e61eb44ff47ea52f5f9d`, equal to fetched
`origin/casefold-rw` with a clean tree. Exact-commit [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38865265666)
and [focused Linux CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38865265682)
succeeded; the preceding audit's full CI also succeeded. Downloaded actual full
CI artifacts and compared all fifteen gate test names as Counter multisets with
the fresh local baseline. Results are scoped explicitly:

| Evidence | Gate controls executed | Enclosing result |
|---|---|---|
| Standalone unit job | 15 | 976 passed, zero failed/ignored |
| x86 host unit tier | 15 | 976 passed, zero failed/ignored |
| aarch64 unit tier | 15 | 976 passed, zero failed/ignored |
| Compiled guest library | 15 | 670 passed, zero failed/ignored |

These are the same fifteen tests in four contexts, not sixty distinct controls.
The **whole guest suite** reports **1,798 passed, zero failed, one ignored**
(3,353 lines / 176,086 bytes). The pre-existing slow
`fuzz_smoke::deep_walk_byte_flip_fuzz_never_panics` did not execute, as recorded
in the historical kernel-budget handoff. CI workflow success is not a
zero-ignored/exhaustive-suite qualification. That missing evidence remains a
broader qualification gate; this review neither waives it nor runs an unrelated
unbounded sweep. No gate control was ignored. Keep write qualification blocked
until the missing evidence is explicitly resolved.

**Decision:** no blocker found within the disconnected control's caller
contract. The mutex provides exclusive non-Send contexts, try-entry refuses
same-thread/callback-worker overlap, and neither poison nor unhealthy state is
recovered. Nested helpers reuse a borrowed context. Control-borrowed stamps
prevent address reuse while live and compare identity plus both counters;
matching versions do not equate different controls. These are not actual mount,
device or buffer identities yet and grant no ownership/admission rights.

Checked epoch/revision arithmetic cannot wrap or partly advance on exhaustion.
Invalidity/uncertainty precede the effect; errors and caught/uncaught effect
panics prevent reuse, even when a caught panic did not poison the mutex. Pure
untouched abort changes no versions; an earlier effect cannot be rolled back
by a later planning error. There is no Ready state, publication, heap reservation,
growing queue or filesystem callback inside entry. Platform synchronization,
caller Arc/effect allocations and callback duration are outside this observation;
there is no fairness/latency or real-card memory guarantee. Callers must include
all writes/flushes and completion checks inside effects and propagate failures;
deferred work, swallowed errors, foreign unwinds and C ABI aliasing are not
made safe by this gate.

**Required integration bridge:** `Filesystem::write_refusal`, `flush` and `Drop`
currently do not know gate health. Merely wrapping chmod could record an early
marker I/O failure in the private gate while legacy writes/close still proceed.
Bind a context to its mount and latch uncertain/poison/unwind failure into common
write/flush/clean-release refusal before a private adapter writes. A replacement
control/refresh cannot recover a faulted mount. This is a next-step obligation;
no standalone primitive or public concurrency fix is claimed by this review.

Fresh full local units pass **976**, zero failed/ignored, with identical
baseline/final Counter multisets and no additions/losses. The log is **1,223
lines / 79,083 bytes**, within unchanged caps. Lint, provenance/shared
instructions, whitespace and saved insertion-red patch applicability pass.
Source/test/workflow hashes remain unchanged; only the two existing handoff/
review documents change, and core/Windows siblings remain clean. Ignored
evidence under `tmp/casefold-gate-review` includes actual downloaded native/guest
logs, scoped CI comparisons, source hashes, unit retention and lint. The source's
release CLI/full CI results remain prior evidence; no fresh CLI rebuild or local
VM run is claimed. Pins, locks, credentials and environment draft/publication
are unchanged.

**Next bounded implementation:** mount-bound **private path-byte and inode chmod
adapters**, with the detailed contract in `docs/CASEFOLD-INDEX-REVIEW.md`:

- One private control per mount and a context borrowing the mount; enter before
  callbacks, lookup, generation validation or early marker writes. Reject foreign
  contexts before any read/write/runtime callback. Reuse the context internally.
- Share existing mode/type/ctime/checksum semantics and retain all read-only,
  casefold/unsupported and stale-generation refusals. Cover both the early
  `mark_not_clean_once` I/O and entire commit with effect/failure handling;
  verdict-only checks and planning refusals need separate treatment.
- Add a sticky mount failure bridge for uncertain/poison/unwind failure, including
  write, flush and faulted-drop clean-marker refusal. No owner readiness,
  automatic builder, buffer seal or reset of existing health flags is permitted.
- Keep public Rust/C ABI delegates as legacy unguarded methods in this bounded
  step. Test actual adapters on scratch mounts, including pre-I/O refusal,
  identity, callback-worker reentry, marker/commit write/flush failure, panic,
  stale generations and legacy refusal after private failure. Require new exact
  private-adapter Linux/e2fsck/debugfs output evidence before qualification.

Public wrappers, other operations/readers, repair/replay/refresh and raw fields/
device handles remain outside serialization; external exclusive access remains
required. Review this private family before public wiring. The ignored slow fuzz
case remains broader qualification work. The three saved insertion aliases,
transaction-private buffer/dependency/overlay/seal/publication, complete map
shape and collision/growth work remain open. Stage 5 remains **not yet quantified**;
public casefold writes, deeper insertion, splits/growth and index dropping stay
blocked, with Windows/SteamOS/actual-card qualification still later.

## Private operation-control checkpoint — 2026-10-16

Started from `c57f0d8fdb5ee1161200121719cc0ea1857b9a72`, equal to fetched
`origin/casefold-rw` with a clean tree. The builder source and ownership review
full CI passed. The audit's [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38860003615)
was still running on the x86 VM job at this checkpoint's initial check; no failed
job was observed. Existing six-profile Linux owner evidence remains prior
source evidence, not qualification of the new operation gate or public writes.

Implemented private `src/operation.rs` plus fifteen native controls, declared
as a private module only. No `Filesystem` field, Rust/C ABI wrapper, device,
buffer, ownership builder or commit routine calls it. The original concurrency
issues were source-audit observations and remain unresolved in those paths;
this is a new private prerequisite, not a claimed public bug fix.

- One mutex holds fixed scalar state. `try_enter` immediately refuses overlap/
  reentry using existing `Unsupported`, while poison or unhealthy state refuses
  reuse. One non-Send context is passed to nested helpers rather than relocking.
  No explicit heap reservation, device access, retries or growing output is
  introduced; synchronization internals depend on the platform. Caller-owned
  `Arc` is optional and creates no device exclusivity or session guarantee.
- View validity is only `Unbuilt` or `Invalid`; health is separately healthy,
  uncertain or faulted. There is **no Ready state or ownership publication API**.
  A borrowed freshness stamp checks control identity plus epoch and revision;
  the borrow prevents its owner's address from being reused while it exists.
  A matching stamp grants no admission, durable-commit or coverage proof.
- `revise` invalidates and advances the control-local revision; `invalidate`
  advances epoch and revision together. This counter is not yet an actual
  `BlockBuffer` revision or seal. Checked overflow faults reuse without wrapping
  or partially advancing counters. The control has no fault/poison reset API.
- `effect` invalidates and marks uncertainty before calling external work. Its
  caller must include the entire write/flush effect and propagate failures.
  Successful return restores healthy state while leaving the view invalid;
  every error conservatively faults reuse, even before a known physical write.
  An uncaught panic faults/poisons the guard. A panic caught while the guard
  remains alive leaves uncertainty and refuses further effects/stamps; normal
  guard drop then latches fault. Catching that panic cannot restore usability.
- Untouched entry/drop changes no version. After a successful earlier effect,
  a later planning refusal never restores the original view. No automatic
  rebuild, reset of existing filesystem health flags, buffer rollback or owner
  cache publication is performed.

All **15 focused controls passed**, zero failed/ignored. They cover same-thread
nested refusal, a synchronous callback waiting for its worker, eight simultaneous
contenders with exactly one held winner/effect, nested borrowed helpers, foreign
control identity with matching counters, stale stamps across invalidation and
revision-only changes, invalidation/uncertainty before the side effect, sticky
error and caught/uncaught panic handling, last-version/exhaustion boundaries,
untouched release and a later planning refusal after an earlier effect. Real
threads use channels/barriers and five-second receive bounds, with no sleeps.
Injected panic/error messages are expected control inputs; the tests passed.
These validate native exclusion/state behavior, not concurrent filesystem
operations, actual card ownership or Linux on-disk changes.

Fresh full units pass **976**, zero failed/ignored. A Counter multiset retains
all **961** baseline occurrences with exactly fifteen operation additions.
The final log is **1,224 lines / 79,142 bytes**, under unchanged 1,400-line /
85,000-byte caps. Focused controls also pass 15, zero failed/ignored (54 lines /
3,268 bytes). Lint passes, including formatting, Clippy with warnings denied,
Unicode checks and provenance. Exact-commit CI and standalone implementation
review are pending; no new independent Linux/Windows concurrency qualification
or local VM execution is claimed.

Locked release CLI installation, shared-instruction/provenance, whitespace and
saved insertion-red patch applicability checks also pass. Exact implementation
hashes remain unchanged after validation. The core/Windows siblings remain
clean; builds reuse the immutable pinned core source view and four-job cache.
No dependency pins, locks, siblings, credentials or environment draft/publication
changed. Ignored evidence under `tmp/casefold-operation-gate` retains baseline,
focused/full logs, Counter retention, lint/CLI results and source hashes.

**Next bounded task:** inspect this implementation's exact-commit full/focused CI
and resolve any failure, then review exclusion/reentry, borrowed identity,
checked versions, panic/error/drop transitions and callback obligations before
wrapper migration. The concrete audit still governs C ABI reference creation/
handle lifetime, public raw-device/metadata escape paths, concurrent flush,
read consistency and nested multi-commit operations. Choose a bounded migration
family only after review; preserve guarded internal helpers and no recursive
public relocking. Buffer/session binding, dependencies/overlays, final sealing,
durable publication and commit-failure qualification remain separate work.

Stage 5 remains **not yet quantified**. The private gate is implemented but
unintegrated. All three saved insertion aliases remain unresolved. Public writes,
deeper insertion, collisions, splitting/growth and index dropping remain
blocked; namespace/recovery and Windows/SteamOS/actual-card qualification remain.

## Concrete operation serialization/invalidation audit — 2026-10-16

Started at `c3557f57088f73bffb18c33f1ccb009235ace4f7`, equal to fetched
`origin/casefold-rw` with a clean tree. Builder source `772981a` now passed
[full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38849841824)
as well as its already reviewed [focused VM evidence](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38849841814).
The [review commit's full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38853268523)
also passed. Downloaded its actual unit artifact: 961 executed, zero failed/
ignored. The six-image ownership evidence and standalone builder decision remain
applicable; neither successful CI nor this audit enables public casefold writes.

Completed a concrete source/call-path/lock-order audit in
`docs/CASEFOLD-INDEX-REVIEW.md`. This is a design checkpoint with no production,
test or workflow change, not an executed concurrency reproduction or guard fix.
The principal findings and chosen boundaries are:

- Normal path resolution, generation/duplicate checks and allocation precede the
  journal lock in `commit_block_buffer`. No-journal commits have no equivalent
  lock. Acquire one outer operation context before the first filesystem read,
  write verdict or early `mark_not_clean_once` superblock write, and retain it
  through all commits and durable publication.
- Wrappers call other public mutators; large `apply_pwrite_ino` recursively
  commits chunks, truncate calls grow/shrink and zero-range calls punch then
  keep-size in two transactions. Move these edges to private context-taking
  helpers instead of locking each public method recursively. Preserve the
  outer guard across commits without claiming whole-call crash atomicity.
- C ABI truncate/fallocate resolve paths before inode mutators; write-file does
  preliminary resolution/type checks before resolving again in the filesystem.
  Those checks belong inside one operation too. Use guarded path dispatch or
  an internal context-taking bridge, preserving errno/type/generation behavior.
  Do not independently lock both ABI and filesystem wrappers.
- Raw inode/descriptor writes, fsck repair, public journal application/writer
  paths and cache replay bypass normal namespace methods. Cover the complete
  scan/repair/recheck or refuse it while proof is owned; invalidate before any
  raw write or logical-view change. Even clean-log recovery can write markers
  and counts. Mount starts unbuilt and cannot build until recovery finishes.
- Refresh, replay/orphan recovery, label changes, flush and finish/drop need
  explicit state transitions. Keep existing direct-commit/journal failures and
  replay-pending refusals sticky. Cache refresh/reopening a writer cannot clear
  failed health. Allocation-state publication also needs preflight capacity
  before future owner publication can be allocation-free after durability.
- Direct dirty-map edits and mutable block vectors defeat typed-writer-only
  tracking. Later transaction-private staging must invalidate seals on mutable
  exposure and validate final bytes/dependencies, including shared inode-table
  blocks and external nodes changed without an inode edit.
- Public `dev`, `journal`, mounted metadata fields and retained device handles
  escape an internal mutex. A future proof-owning session must control access
  and discard proof before returning mutable/raw access; external device
  exclusivity remains a caller obligation. A mutex is not a transparent cache
  guarantee for arbitrary existing `Filesystem` users.

C ABI reference lifetime remains a separate obligation: mutable maintenance
calls and unmount require exclusive caller access. An internal gate cannot fix
aliasing that occurred before entry or keep a freed handle alive. Future handle
admission must precede creating filesystem references and drain users before
release; callback refusal must precede conflicting reference creation too.

The first private gate will use **nonblocking entry with explicit refusal** on
contention/reentry, so synchronous callbacks waiting for same-mount worker work
cannot wait behind themselves. Poison refuses reuse. This is not a public ABI
or concurrent-flush behavior change. Runtime providers, device methods and fsck
callbacks are external code; a thread-local same-thread check alone is
insufficient. Proposed lock order is operation outermost, journal when needed,
then allocation/cache/device locks under their existing edges. Private nested
helpers reuse the same mount-bound context; no recursive mutex or unlock-around-
callback shortcut. Exclusive maintenance can own the control handle separately
without unsafe aliasing across mutable filesystem refresh.

Separate proof validity from health: unbuilt/invalid state grants no admission;
raw changes revoke proof and advance a checked epoch before side effects.
Uncertain write/flush failure or poison makes health faulted until recovery/
remount. Untouched planning abort discards only local work. If an earlier chunk
already committed, a later abort cannot resurrect the original proof. Future
publication must preflight controlled capacities, verify final buffer/epoch/
revision and publish only after durability. High-level readers must join the
qualified session or remain externally serialized; raw reads are no snapshot.

**Next bounded implementation:** add only a private disconnected operation gate/
control primitive and deterministic tests: at-most-one entry, prompt same-thread
and callback-worker overlap refusal, untouched abort/release, mount identity,
checked epoch/revision exhaustion, invalidation before injected side effects,
and sticky poison/uncertain-failure refusal. It must create no owner token or
readiness. Review it before wiring wrapper families, repair/replay/raw/refresh
paths or public APIs. Later qualification includes nested multi-commit calls,
C ABI pre-resolution, read/flush compatibility, raw dependency edits, callback
reentry and durable publication/failure injection. No mounted-only cache or
per-insertion scan is allowed.

Fresh baseline/final units both pass **961**, zero failed/ignored; Counter
multisets are identical, with no additions/losses (1,208 lines / 77,820 bytes,
under unchanged caps). Lint, provenance/shared instructions, diff whitespace and
saved insertion-red patch applicability pass. Source/test/header/workflow hashes
are unchanged and core/Windows siblings remain clean. Ignored evidence is under
`tmp/casefold-lifecycle-audit`: actual CI unit log, source CI status, source call/
lock inventories, hashes, local logs and retention. The source's locked release
CLI build/full CI remain prior evidence; no new CLI rebuild or local VM run is
claimed for this documentation-only audit. Pins, locks, siblings, credentials
and environment draft/publication are unchanged.

Stage 5 remains **not yet quantified**. The private builder and ownership
prerequisites are implemented/reviewed; serialization/lifecycle integration,
overlays, reverse dependencies, final sealing, publication/commit failures,
complete mapping shape and collision/growth qualification remain. All three
saved insertion aliases remain unresolved. Public casefold writes, deeper
insertion, collisions, splits/growth and index dropping remain blocked.
Windows/SteamOS/actual-card qualification is still later work.

## Complete ownership implementation/evidence review — 2026-10-16

Reviewed `772981a50a4624b878c4ccd0ea36df5b2fec0233`, equal to fetched
`origin/casefold-rw` with a clean tree. Its [focused VM evidence](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38849841814)
passed. Downloaded the actual artifact, all six owner images and their inventory/
owner commands, raw reports, fixed ranges and summaries. The exact whole-result
oracle executed **1**, zero failed/ignored, across six profiles (53 lines /
6,627 bytes). All seven other focused fixture/insertion/component logs also
pass one test each, zero failed/ignored. Source [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38849841824)
remains pending with x86 VM work outstanding. The preceding documentation
review's [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38848087843)
passed at the final pre-push check. That earlier result does not qualify the new
builder or establish the new source's full-CI conclusion.

| Block size / descriptor / journal | Allocated IDs | Fixed / data / map claims | Total claims | Used blocks | Charged reads / tree records |
|---|---|---|---|---|---|
| 1 KiB / 32 / no | 37 | 11 / 24 / 1 | 36 | 10,299 | 48 / 25 |
| 1 KiB / 64 / no | 37 | 11 / 24 / 1 | 36 | 10,299 | 48 / 25 |
| 1 KiB / 64 / yes | 37 | 11 / 25 / 1 | 37 | 11,323 | 48 / 26 |
| 4 KiB / 32 / no | 37 | 10 / 24 / 1 | 35 | 2,599 | 48 / 25 |
| 4 KiB / 64 / no | 37 | 10 / 24 / 1 | 35 | 2,599 | 48 / 25 |
| 4 KiB / 64 / yes | 37 | 10 / 25 / 1 | 36 | 3,623 | 48 / 26 |

Each 16 MiB image has two initialized groups, 64 slots, 256-byte inodes and
one primary descriptor block. Its 37 allocated records include 20 empty files,
one hard-linked inode, three directories, fast and extent-backed symlinks, FIFO,
one external map and unwritten beyond-EOF storage. Journal profiles include
inode 8's depth-zero 1,024-block journal. No-journal profiles have nine inactive
reserved slots; journal profiles have eight. Every image has one depth-one file;
other extent roots are depth zero. The 1 KiB boot reservation is outside the
bitmap domain; each 4 KiB primary block-zero bit is inside it and set.

A read-only public-format audit matched all **222 records** and **215 claims**
(63 fixed, 146 data, six mapping) to saved image bytes, including identity,
generation, type, size, links, sectors and logical/unwritten map values. The
claim union equals every set allocation bit and the complement of the complete
saved free lists: **40,742 used blocks** across twelve groups, with group/global
free counts matching. All **245,376 inode-padding** and **184,323 block-padding**
bits are set, and both groups have ITABLE_ZEROED only. Commands exactly cover
all 64 slots and every reported allocated ID. The audit supplements actual
VM-only tool/kernel checks; it is not a host oracle invocation or mount.

**Decision:** no blocker found within the standalone private constructor's narrow
profile and trusted coherent stable-view preconditions. Current fixed layout and
complete inventory precede body interpretation; every allocated ID and complete
qualified tree is checked, including empty/zero-link/reserved records. Typed
identities/roles, generation and sector totals survive the join; global overlaps
refuse before one allocation check. Staged node size/bounds precede copying and
staged bytes bypass underlying read errors. Failures drop the entire local
result, change no caller/device bytes and publish no reusable proof.

Reviewed aggregate group/read/probe/record/claim/bitmap budgets and checked,
fallible retained/transient requested-capacity accounting. The read charge is
conservative: fixed/inventory/allocation reserve `3(1+D)+2G`; body reads debit
the actual census, and complete trees debit returned consumption from remaining
global allowances. Here ten metadata reservations plus 37 bodies plus one map
are 48 logical reads, not a measured syscall count. The requested-byte bound
includes all vector capacities and scratch, even nonoverlapping phases; it
excludes allocator overhead, other scalar stack and shared cache/pinned replay
memory. No card-scale defaults, latency or RSS are qualified.

The joined Linux matrix has ordinary non-casefold, non-FLEX, non-sparse layouts
without attributes; it does not independently qualify every allowed feature
combination. Corruption, zero-link ordinary bodies, alias conflicts and limit
refusals remain synthetic controls. Unclaimed used bits never mean free storage.
The stable-view precondition is still a caller obligation: there is no operation
guard, epoch, dependency closure, overlay, seal, publication or commit-failure
handling. All three saved insertion-level aliases remain unresolved because the
insertion path does not use the builder. No source/test/workflow fix is needed by
this review; all hashes remain unchanged.

Fresh full units pass **961**, zero failed/ignored, with identical baseline/final
test-occurrence Counter multisets and no additions/losses. The final log measures
**1,208 lines / 77,820 bytes**, within unchanged caps. Lint, provenance/shared
instructions, diff whitespace and saved ownership-patch applicability pass.
Validated source/test/workflow hashes remain unchanged; core and Windows sibling
checkouts remain clean. Local tools reuse the pinned source view and existing
four-job build cache. Ignored evidence is under `tmp/casefold-owner-review`,
including downloaded artifacts, raw audit/results/image hashes, unit logs and
retention. The source's locked release CLI build already passed; no fresh CLI
rebuild or local VM run is claimed for this documentation-only review. No pins,
locks, siblings, credentials or environment draft/publication changed.

**Next bounded task:** resolve the source's outstanding full CI first, then audit
the concrete operation serialization/mutation/invalidation call paths under the
existing transaction-current design. Cover path and inode entry points, direct
writes/raw staging, replay/recovery, refresh, callbacks/reentry, lock order and
failed commits. Specify guarded internal helpers, epoch/buffer revision and
poison/invalidation transitions before implementing the operation guard. Keep
admission/cache publication disconnected; no per-insertion full scan or mounted-
only cache. Later overlays, reverse dependencies, final sealing and durable
publication/failure qualification remain required. Stage 5 is **not yet
quantified**. Public writes, deeper insertion, collisions, splits/growth and index
dropping remain blocked; Windows/SteamOS/actual-card qualification remains later.

## Private complete current-view ownership checkpoint — 2026-10-15

Started from `824a98edf52916f94389af34fbddbb8edf5f1d50`, equal to fetched
`origin/casefold-rw` with a clean tree. Its documentation-review
[full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38848087843)
was still running during this checkpoint (unit, provenance, CLI, semver, MSRV,
Wasm, fixtures and aarch64 jobs passed; x86 VM and guest-suite jobs pending).
The allocation source's `72b6911` exact-commit focused/full CI already passed;
neither result establishes the new builder's compatibility.

Implemented private `src/ownership.rs`, successful extent-walk usage accounting,
18 constructor controls, a six-profile VM oracle and its focused CI step with
an exact-test floor and retained artifacts. No public API or write gate changed.
The builder visits every current allocated ID once, including reserved, empty
and zero-link records, and retains generation/type/storage/size/sectors/links.
Root and a declared journal must be present and correctly typed. It collects
complete data/map storage, including unwritten and beyond-EOF allocation, and
checks whole-inode sector totals. Typed fixed and inode/generation/role claims
are sorted together; any cross-owner, fixed, journal or same-inode role overlap
aborts the result. One current-allocation verification follows, excluding only
the 1 KiB boot reservation. Unclaimed used bits never imply free storage.

The supported intersection remains narrow: checksummed 1/4 KiB layouts,
32/64-byte descriptors, initialized inode/touched block bitmaps, 256-byte Linux
inode bodies, known no-storage/extent roles and depth zero/one (journal zero).
Unknown attributes/storage/flags, RESIZE_INODE and deeper maps refuse. The node
view checks aligned full-block bounds and staged size; staged bytes take
precedence without a preliminary device read. Caller/device bytes are unchanged.

Whole-build budgets charge conservative fixed/inventory/allocation group and
read reservations, actual inode-body reads, cumulative extent reads/records,
inventory probes, global claims, a separate transient tree ceiling and allocation
bitmap work. Checked, fallible requested-capacity accounting sums retained and
transient vectors plus scratch, even phases that do not coexist. A tree's
capacity is dropped before the next inode; no per-block owner output or growing
bitmap cache is created. Any error discards the entire local result. These are
explicit caller limits, not card-size defaults or measured RSS bounds; shared
cache/pinned replay memory and allocator overhead remain outside them.

**Local evidence:** preserved a fresh 943-pass baseline before edits. The first
red attempt had a compile-only helper typo; after correction, four executed
controls produced three acceptance failures and one passing disjoint control.
Through the builder the same four pass. All 18 expanded controls pass, including
exact/exhausted cumulative budgets, overflow, typed census/storage, zero links,
hard links, fixed/journal/same-inode conflicts, missing root/journal, sectors,
staged body/bitmap/map state, malformed inputs, unsupported profiles, generation,
unwritten storage, I/O failure and staged precedence. The final full unit tier
passes **961**, zero failed/ignored. A Counter multiset confirms all **943**
baseline occurrences retained with exactly 18 builder additions. Its log is
**1,209 lines / 77,879 bytes**, within unchanged 1,400-line / 85,000-byte caps.
Lint (including Clippy with warnings denied, formatting, Unicode tables and
provenance), locked release CLI installation, shared-instruction, diff whitespace
and saved ownership-patch applicability checks pass. Exact source, guest-script
and workflow hashes remain unchanged after validation. The actual core sibling
is clean at onboarding v0.3.6; builds use the immutable pinned v0.3.0 source view.
`ext4-win-driver` also remains clean. No pins, locks, siblings, credentials or
environment configuration draft/publication changed. Ignored baseline, executed
red/green, focused/full logs, Counter retention and source hashes are retained
under `tmp/casefold-owner-builder`.

**Independent evidence pending:** the new VM-only oracle creates six initialized
Linux images (1/4 KiB, 32-byte/no-journal, 64-byte/no-journal and 64-byte/journal).
Both groups are populated with empty files, hard links, directories, short/long
symlinks, FIFO, external maps and unwritten beyond-EOF storage. Judged e2fsck
must be clean. Complete `debugfs testi` answers establish the inode census;
`stat` supplies identities/types/storage ranges and accounting; `dumpe2fs`
supplies every fixed role/range and used totals. Compare the complete typed
sorted claim vector and every inode record, with missing/duplicate answers
refused. Commands, reports, images and summaries are CI artifacts. This test
compiles locally but has not run here: the cloud host has no KVM or local
harness checkout. No host oracle or mount is used.

The constructor requires a trusted coherent stable device/buffer view for the
whole invocation. It publishes no cache, epoch, dependencies, transaction
overlay or admission token. The three constructor alias shapes now refuse,
but the three saved insertion-level cross-inode failures remain unresolved:
the insertion path does not call this builder. Operation guard, epoch and
dependency tracking, overlays, sealing/invalidation, publication and commit
failure handling still need implementation and independent qualification.

**Next bounded task:** inspect the new exact-commit focused/full CI and diagnose
any failure; download all six owner images/reports, review whole-result agreement,
shared resource accounting and constructor preconditions. Only after that review
start lifecycle/admission integration. Do not add per-insertion card scans or a
mounted-only cache. Preserve sibling pins, baseline occurrences and write gates.
Stage 5 completion remains **not yet quantified**. Deeper insertion, collisions,
splits/growth, namespace/recovery and Windows/SteamOS/card qualification remain.

## Allocation verifier implementation/evidence review checkpoint — 2026-10-15

Reviewed `72b6911d4d265ce1339c56caac1f9330a62099ef`, equal to the fetched
remote tip with a clean tree. Its [focused VM evidence](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38795846343)
and [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38795846219)
passed, including both full VM suites and the aggregate gate. Existing Git/API
pull/push, logs and actual artifact downloads work. Reused the tested onboarding
setup without a draft/publication, credentials, sibling or dependency/pin/lock
change. No source/test/workflow fix or gate change is justified by this review.

Downloaded the actual focused artifact and inspected all four allocation images,
commands, complete nonzero bit answers, free-block lists and summaries. The
allocation oracle executed once, zero failed/ignored (37 lines / 3,890 bytes).
Each 1 KiB/32- or 64-byte descriptor profile has domain `[1, 2044)`, 1,573 used
and 470 free blocks, two allocated runs and 1,019 valid blocks in its last group.
Each 4 KiB/32- or 64-byte profile has `[0, 511)`, 401 used/110 free, two runs and
255 valid last-group blocks. Across eight groups there are **5,108 complete
domain classifications**: 5,106 explicit nonzero `debugfs` answers cross-checked
against complete `dumpe2fs` free lists, plus the two zero classifications from
those count-checked lists. Both 4 KiB zero bits are set and checked normally.
The 1 KiB boot block remains outside the allocation domain, separately protected.

A raw public-format audit matched every reported answer to current image bitmap
bytes and all 158,732 padding bits are set. Both groups in every profile have only
ITABLE_ZEROED, with no block/inode uninitialized flags. Raw inode observations
show 32 allocated IDs each: nine zero-mode reserved slots, two directories and
21 regular files, with 256-byte records and 23 depth-zero extent headers. These
are candidate initialized builder fixtures; these observations do not qualify
the complete inventory/role/tree ownership join on those images.

Existing focused tests each executed once, zero failed/ignored: fixed metadata
across 14 profiles (59 lines / 7,385 bytes), roles across six (41 / 4,671), inventory
across four (33 / 3,357), insertion across four (133 / 7,954), FIEMAP across both
block sizes (23 / 1,601). These results qualify their scoped existing behavior,
not a complete owner snapshot or public writes.

The implementation/resource review in `docs/CASEFOLD-INDEX-REVIEW.md` checked
pre-slice geometry/coverage, interval bounds/order, group progress, byte masks,
current CRC/identity/pointer/flag/size checks, staged precedence, full padding,
fallible two-block scratch reuse and shared error-inclusive caps. **No blocker
found within the standalone verifier's narrow profile and trusted coherent
stable-view/fixed-layout preconditions.** The production verifier is unchanged
from `1fcb361`. The independent matrix covers every bit and multiple groups, but
one allocated interval spanning a group boundary remains a synthetic control,
as do staged corruption, uninitialized/unknown profiles and limit refusals.

The scratch limit bounds this component's own requested bytes, not allocator
overhead, shared cache/pinned replay memory or other live components. It proves
neither exclusive ownership nor complete census, epoch or transaction lifecycle;
empty proposed input proves no coverage. No card-scale RSS/performance bound,
real-card/Windows/SteamOS qualification or write activation follows from this
review. The three saved other-inode aliases remain unresolved.

Fresh full units pass **943**, zero failed/ignored, with identical baseline/final
test-occurrence multisets and no additions/losses. The final log is 1,190 lines /
76,251 bytes, within unchanged caps. Lint, provenance/shared instructions, diff
whitespace and saved ownership patch applicability checks pass. Exact source,
test and workflow hashes remain unchanged and the sibling stays clean.
Ignored evidence is under `tmp/casefold-allocation-review` and
`tmp/casefold-allocation-review-baseline`, including actual downloaded reports/
images, complete raw audit, unit logs/comparison and exact source hashes. The
source's locked release CLI build/CI CLI job already passed; no fresh CLI rebuild
or new local VM run is claimed for this documentation-only review.

**Next bounded implementation:** a private complete current-view ownership builder
joining fixed roles, the current allocated-inode census, every typed inode's
complete tree and verified current allocation. Follow the detailed contract in
the review: supported-profile intersection, root/declared-journal coverage,
unwritten/beyond-EOF and mapping storage, sector accounting, all cross-owner/
fixed/journal conflicts, one bounded typed arena and shared whole-build budgets.
The extent collector currently hides per-call read/record usage; add minimal
private consumption accounting as needed, never reset a full per-inode budget.
Charge retained/transient capacities and scratch at their peak coexistence;
abort the entire result on unsupported state, missing coverage or any failure.
Require the three alias shapes and disjoint control in builder tests, plus
independent whole-result Linux evidence. Keep this constructor disconnected from
admission/cache/epoch publication. Review it before later guard/dependency/overlay/
sealing/invalidation/commit-failure integration; no per-insertion full scan.
Preserve all baseline occurrences, sibling pins and every write gate.

Stage 5 completion is **not yet quantified**. Casefold writes, deeper insertion,
collisions, splitting/growth and index dropping remain blocked; full namespace,
recovery and Windows/SteamOS/card qualification remain later work.

## Allocation-oracle block-zero repair checkpoint — 2026-10-15

Follow-up: `72b6911` passed exact-commit focused VM and full CI; the complete
standalone allocation evidence/implementation review is recorded above.

Started at `1fcb3610b5c00e09511648827d62fd2e9ca8268f`, equal to the fetched
remote tip with a clean tree. The fresh baseline passes **941 unit tests**,
zero failed/ignored. Git/API pull/push, job logs and artifact downloads work.
Reused the existing onboarding setup without an environment draft/publication,
credential, sibling, pin/lock, production-verifier or public-write change.

Inspected [focused CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38787960237)
and [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38787960217)
at that exact commit. The focused allocation test failed once (39 lines), and
both full library executions report **633 passed, one failed, zero ignored**
(669-line native and 832-line guest logs). All three fail at the same oracle
request: `testb: Invalid block number 0`, despite exit zero. The existing verdict
reader correctly refused to treat that failed request as an allocation answer.
No verdict or skip gate is weakened. All other CI jobs succeeded, and the focused
insertion/fixture/FIEMAP/inventory/role/fixed-metadata steps passed.

The two 1 KiB profiles completed before the failure: each has 2,043 explicit
allocation answers, 1,573 used blocks and two allocated runs. The 4 KiB/32-byte
image formatted, mounted/populated and passed judged e2fsck before the invalid
query. The 4 KiB/64-byte profile was unrun. Those partial results do not qualify
the complete allocation matrix or complete this component's implementation review.
Downloaded and inspected the actual focused images/reports and both full logs.

The repair changes only test code/data and documentation. `debugfs testb` now
queries every valid nonzero block. An additional `dumpe2fs` report must enumerate
every group in order with exact bounds, one free-block list per group, matching
per-group and global free counts, no duplicate/out-of-group entries and no
missing/truncated list. Its complete free set supplies an explicit used/free
classification for the entire allocation domain, including block zero, and every
nonzero `debugfs` answer must agree. Each singleton and all coalesced allocated
runs are still checked by the verifier; the whole-range free-bit refusal and
1 KiB boot-domain/4 KiB zero-bit checks remain. No generic block-zero allocation
exception is added. New `.free-blocks.txt` reports are retained by the existing
artifact directory; the exact test filter and one-test floor are unchanged.

Two local controls use captured black-box `dumpe2fs` output from the preceding
focused metadata test (8 groups, 4,051 free blocks; trailing whitespace trimmed).
That report is a text-reader fixture, not qualification of uninitialized groups
for the allocation verifier. Controls recognize zero as allocated in the capture
and explicitly free in a count-preserving synthetic alteration, reject missing,
duplicate, out-of-group, wrong-count and truncated lists, and require an explicit
zero count for an empty group list. The initial focused run passes both controls,
zero failed/ignored (16 lines / 795 bytes). No kernel/e2fsprogs source was read.

Fresh full units pass **943**, zero failed/ignored, retaining all **941** baseline
occurrences and adding exactly the two oracle-reader controls (multiset comparison).
The final log is 1,191 lines / 76,311 bytes, within unchanged caps. Fresh lint,
including formatting, Clippy, provenance/shared instructions and Unicode checks,
passes. Diff whitespace and saved ownership patch applicability checks pass;
validated source/data hashes match and the sibling remains clean.
Evidence is under ignored `tmp/casefold-allocation-ci-repair`, including the fresh
baseline/final logs and occurrence comparison, source CI failures, downloaded
artifacts, raw image/report audit and parser results.
The production verifier is byte-identical to `1fcb361`; the existing controls,
module/gate wiring, guest script and workflow are unchanged. Its preceding locked
release CLI build and exact-commit CI CLI job passed; no fresh CLI rebuild or
new local VM run is claimed for this test-only repair. Repaired independent
VM/full CI remains pending after push.

**Next bounded task:** inspect the repair's exact-commit focused/full CI and all
four profiles' actual allocation/free-list reports and images; resolve failures
before completing the allocation verifier's current-view, mask/padding, profile
and resource review. Only then select complete ownership construction under
aggregate budgets and a whole-view lifecycle. The three saved other-inode aliases
remain unresolved. Complete claim coverage, conflicts, guard/overlay/dependencies/
sealing/invalidation and commit failures remain gates. Stage 5 completion is
**not yet quantified**; casefold writes stay blocked and Windows/SteamOS/card use
and card-scale resource/performance bounds remain unqualified.

## Private bounded current block-allocation interval checkpoint — 2026-10-15

Follow-up: initial independent CI at `1fcb361` failed on an unsupported `testb 0`
request. The test-only repair and current validation status are recorded above;
the complete four-profile evidence/implementation review remains outstanding.

Started at `67d96f6088bbc9545763b0dbc55f4de3878ea551`, equal to the fetched
remote tip with a clean tree. The fresh baseline passed **921 unit tests**, zero
failed/ignored. Fixed-metadata source `95cb218` and review `67d96f6` both passed
full CI; the source's focused VM tests also passed. Git/API pull/push and log
downloads work. Reused the existing onboarding setup with no configuration draft,
publication, credential, sibling, dependency/pin/lock or write-gate change.

The baseline adapter to the existing per-block insertion helper accepted both a
bad current superblock CRC and a clear padding bit beyond checksum coverage; a
healthy superblock-bit control passed (three executed: two failed, one passed,
zero ignored; 35 lines / 1,464 bytes). Those same assertions pass through the
new `block_allocation::verify` (three passed, zero failed/ignored; 18 / 791).
The old helper remains unchanged. An oracle test initially failed to compile
because `FileDevice::open` requires a string path; its call was corrected before
validation. This was a test-build error, not a test execution or oracle result.
The first expanded execution passed 19 controls and failed one raw-device CRC
control because the fixture retained a cached pre-mutation block. The fixture
now invalidates that cache after raw mutation before checking the new coherent
view; verifier behavior and its coherent-view precondition are unchanged.

The new private verifier checks physically sorted, disjoint, nonempty intervals
without expanding runs into block/owner entries. It requires a trusted coherent
stable current view and a successfully qualified current fixed layout. It checks
supported geometry/coverage, current superblock CRC/identity, touched current
descriptor CRC/stable metadata pointers/known/pending flags, bitmap/table bounds,
non-FLEX own-group placement, bitmap CRC and every proposed allocation bit.
The initial profile is ordinary metadata-checksummed allocation, 1/4 KiB,
32/64-byte descriptors, byte-aligned blocks-per-group and initialized touched
block bitmaps. BLOCK_UNINIT refuses before reading unspecified bytes. CRC covers
`blocks_per_group / 8` bytes with the descriptor-width mask. Every padding bit
through the entire bitmap must be one, including partial final groups and bytes
outside the CRC. At 1 KiB, boot block zero is outside this generic domain; at
4 KiB, superblock block zero has a bit and is checked normally.

One invocation shares range/group/read/bitmap-work/scratch caps. Fallible two-block
scratch is reused for the current descriptor and bitmap; ordered ranges keep only
the last group, with no growing bitmap cache. Work charges two full blocks per
touched group before checksum/padding and each range/group segment's covered
bytes before checking its bits. Logical reads debit before staged/device access.
Failure publishes no partial proof and changes no caller/device state. Empty
input still verifies the current superblock but establishes no claim coverage.

Twenty focused controls cover shared exact/exhausted limits, a complete 69-group
interval with two descriptor blocks, byte edges, block zero, staged allocation/
freeing, disk/staged bitmap/descriptor CRCs, malformed sizes, relocated pointers,
superblock identity/counters, initialization/pending/unknown flags, final padding,
checksum widths, unsupported geometry, malformed intervals, FLEX placement,
metadata aliases/table bounds, injected reads and empty input. Synthetic layout
fixtures isolate these conditions and do not claim Linux-clean images.

Fresh full units pass **941**, zero failed/ignored, retaining every one of the
921 baseline occurrences and adding exactly 20 controls (multiset comparison).
The focused expanded run passes all 20 (52 lines / 3,661 bytes). The full unit
log is 1,189 lines / 76,089 bytes, within unchanged 1,400 / 85,000 caps. Lint
(including formatting, Clippy, provenance, shared instructions and Unicode checks)
and the locked release CLI build/install pass. Saved ownership patch applicability,
guest Python syntax and diff whitespace checks pass. Source/workflow/guest hashes
match the validated bytes; no sibling, pin/lock or gate changes occurred.
Ignored evidence is under
`tmp/casefold-block-allocation-evidence`, including baseline/red/green/focused/
unit logs, test-occurrence comparison, prior CI log and source hashes.

The new VM test formats four small profiles (both block sizes and descriptor
widths), asks Linux to allocate across both groups and retain free blocks, checks
judged non-repairing e2fsck and the current fixed layout, then compares every
valid singleton with one explicit `debugfs testb` answer. It checks coalesced
allocated runs together, full-range refusal on free bits, partial final groups
and the block-zero domain. Its guest script also initializes both inode groups.
The focused workflow retains images, commands, full allocation reports, summaries
and the log, with an exact test filter and one-test floor; existing focused steps
remain. The oracle is compiled but unrun locally because there is no KVM; new-
source independent VM/full CI remains pending. This is not a complete owner proof.

**Next bounded task:** inspect the new checkpoint's exact-commit focused/full CI
and actual allocation reports/images, resolve failures, then review current-view,
bit masks/padding, bounds, profile intersection and aggregate resource contracts.
Only after that review select the complete ownership join of fixed roles, every
typed allocated inode's full tree and verified current allocation. Cross-owner
conflicts, whole-result coverage, shared peak budgets and guard/overlay/dependency/
sealing/invalidation/commit failure rules remain separate prerequisites. Preserve
all baseline occurrences, the three saved other-inode failures and disjoint
control, pinned siblings and every write gate. Stage 5 completion is **not yet
quantified**; Windows/SteamOS/card use and card-scale performance are unqualified.

## Fixed-metadata implementation/evidence review checkpoint — 2026-10-15

Follow-up: fixed source `95cb218` and review `67d96f6` now passed full CI above.
The following implementation supersedes this historical section's next task.

Reviewed `95cb21810b0e43edfbabcaf1b7d55e5d83d23787`, equal to the fetched
remote tip with a clean tree. Its exact-commit focused workflow passed; full CI
is still running at the initial check. Existing Git/API pull/push and artifact
download access work. The existing onboarding setup is reused without a draft
save/publication, credential, sibling or dependency/pin/lock change. This review
changes documentation only; no production fix, test or workflow change is needed.

Downloaded the actual focused artifact and inspected all 14 metadata images/
location reports and current metadata/role/inventory/insertion/FIEMAP logs.
The metadata oracle executed once, zero failed/ignored, with successful VM
formats, judged non-repairing e2fsck and complete typed/grouped range equality
against `dumpe2fs`: **196 groups and 943 protected intervals**. Dense eight-group
profiles have backups in all groups, classic sparse in 0/1/3/5/7, and sparse-super2
in 0/1/7. Classic reserved-GDT counts are 255/256 blocks at 1 KiB and 63/127 at
4 KiB for 32/64-byte descriptors. Every FLEX_BG profile places six of eight
tables before their own group. Two larger dense profiles have two-block
descriptor runs: 32 groups/161 ranges at 1 KiB and 68 groups/340 ranges at 4 KiB.
A raw public-format audit also checked every bitmap/table address/full span,
group coverage, range bounds and disjointness against the reports. Actual log
measurements and CI links are recorded above. These are small-image observations,
not real-card performance or memory measurements.

The 1 KiB profiles independently cover partial final block groups; partial final
inode-group coverage and malformed/staged refusals remain synthetic controls.
Many saved metadata groups have INODE_UNINIT/BLOCK_UNINIT, and the reserved-GDT
profiles have RESIZE_INODE. Their physical protection does not qualify allocation
bits, the narrower inode inventory/role profile or resize inode ownership. They
must not silently become positive fixtures for a full owner snapshot. Enforce
the intersection of supported and independently qualified component profiles.

The source/test/resource review in `docs/CASEFOLD-INDEX-REVIEW.md` checked bounded
geometry/slicing/arithmetic, complete backup/table coverage, current identities,
CRC/pending flags/staged precedence, global fixed-role separation, fallible fixed
reservations and error-inclusive shared caps. **No implementation blocker was
found within the standalone collector's narrow profile and trusted coherent
stable-view preconditions.** It remains disconnected from mutation/admission
and proves neither allocation nor complete ownership/epoch/lifecycle. No fix or
test is introduced by this review. The three saved other-inode aliases remain
unresolved; no implementation progress is inferred from documentation changes.

Fresh validation passes **921 unit tests**, zero failed/ignored, with identical
baseline/final test-occurrence multisets and no additions or losses. The final
unit log is 1,168 lines / 74,303 bytes, within unchanged caps. Lint, provenance,
shared-agent instructions, saved ownership patch applicability and diff whitespace
checks pass. Ignored evidence is under
`tmp/casefold-fixed-review-evidence`, including the downloaded artifact, complete
profile audit, baseline/final unit logs and test-occurrence comparison. Source/
test/workflow hashes still match the preceding validated implementation. Its
locked release CLI build and exact-commit CI CLI job already passed; no release
CLI rebuild or new local VM run is claimed for this documentation-only review.

**Next bounded task:** check outstanding exact-commit full CI and resolve any
failure first. Then implement a private bounded current block-allocation interval
verifier before the complete owner join. Verify proposed physical intervals
directly against current bitmap bytes/words without expanding data runs into
owner entries. Validate interval/group/filesystem coverage, current superblock/
descriptor identity/checksums, bitmap sizes/bounds/checksum widths and pending/
unknown flags; define and independently qualify final-group/padding rules.
Initially refuse BLOCK_UNINIT before reading unspecified allocation bytes.
Use shared interval/group/read/work/scratch limits across the invocation and
fail without mutation or partial publication. Do not reuse the insertion helper's
growing per-block bitmap cache as a global bounded proof. At 1 KiB, boot block
zero remains separately typed fixed protection outside the allocation domain;
generic proposed ranges below `first_data_block` refuse. At 4 KiB, superblock
block zero belongs to the bitmap domain and must be checked. Require staged,
cross-group, checksum/size/flag/boundary/resource/unchanged-state controls and
independent VM allocation answers on initialized positive profiles. Then review
the verifier before joining fixed metadata, every typed inode/tree owner and
current allocation under aggregate peak budgets. Cross-owner conflicts, complete
coverage and guard/overlay/dependency/sealing/invalidation/commit failures remain
separate prerequisites. Preserve all 921 baseline occurrences, saved owner
failures/control, pinned siblings and every write gate; do not connect admission.

Stage 5 completion is **not yet quantified**. Public casefold writes, deeper
insertion, collisions, splitting/growth and index dropping remain blocked.
Windows/SteamOS/real-card use and large-card performance remain unqualified.

## Private bounded current fixed-metadata checkpoint — 2026-10-15

Follow-up: source `95cb218` passed its exact-commit focused VM location and
existing Linux tests. The review above supersedes this historical section's
pending focused status and next review task; full CI status is maintained above.

Started at `c1afa4771014233363a0978ddeb572f894c6a4f5`, equal to the fetched
remote tip with a clean tree. The fresh baseline passed **904 unit tests**, zero
failed/ignored. The preceding role source and its review passed exact-commit
focused Linux/full CI as recorded above. Existing Git/API bindings report pull/
push access. The existing onboarding setup is reused without a configuration
draft/publication, credential, sibling, dependency/pin/lock or write-gate change.

Three refusal controls first failed on acceptance through the existing mounted
boolean metadata helper: bad current descriptor checksum, checksum-valid changed
superblock and cross-group fixed-role overlap. A healthy inode-table control
passed (four executed, three failed, one passed, zero ignored). The same original
assertions now pass through `fixed_metadata::collect`; the existing boolean helper
is unchanged. The expanded cross-group case permits FLEX_BG before requiring an
explicit overlap error, rather than passing on a same-group placement refusal.

The red log is 37 lines / 1,638 bytes. The original four pass after the new
component; the expanded focused run passes 17 (25 lines / 1,734 bytes).

The new private collector verifies the current primary superblock and all
primary descriptors under a trusted mount and stable device/buffer view. It
preserves the existing narrow identity rules: free-counter-only superblock
changes and descriptor counters/checksums/known flags may change, while metadata
addresses stay mounted. Bad CRCs, malformed staged block sizes, pending-flag/group
disagreements, unknown flags and unqualified layout features refuse. Its initial
profile supports metadata checksums, 1 KiB/4 KiB blocks, 32/64-byte descriptors,
Linux creator fields and explicit feature allowlists; META_BG/bigalloc refuse.

Output ranges protect boot space, primary/backup superblocks, all primary/backup
descriptor blocks, declared reserved-GDT blocks, every group's block/inode bitmap
locations and its full declared inode table. Dense, classic sparse and sparse-
super2 backup selection is explicit. Primary block zero at 4 KiB includes boot
bytes in its superblock protection range; 1 KiB has a separate boot block. Bad
backup IDs/duplicates, truncated group heads, out-of-range metadata and overlapping
fixed roles across any groups refuse. FLEX_BG may place metadata outside its own
group, still within the filesystem; without it, own-group spans must fit. Partial
final inode groups retain their entire declared table span, not only valid IDs.

Backups are protected from verified primary geometry without trusting their
possibly stale contents as current authorities. Known initialization flags never
remove physical reservations. This collector reads no allocation bitmap contents,
inode bodies or journal maps. Admitting a resize reservation or uninitialized
location here does not qualify resize-inode ownership or the narrower inventory/
role profile. The eventual builder must take the intersection of qualified
profiles and keep journal storage on its typed inode/complete extent path once.

One invocation-wide limit set bounds groups, logical fetches (staged/failing reads
included), ranges and two-block scratch. Shared descriptor blocks are cached.
Checked requested allocation bytes are `range_cap * size_of::<Range>() + 2 * bs`;
fallible fixed reservations and in-place sorting prevent hidden vector growth.
Retained capacity, allocator overhead/device cache and later tree/role/dependency
storage still need aggregate peak accounting. Any error drops the whole local
result and preserves dirty blocks/pending flags/device bytes; no epoch or owner
snapshot is published, and no real-card resource default is qualified.

Seventeen focused controls pass: the original four plus complete typed output,
cross-group placement/full partial-group tables, exact/exhausted limits, malformed
sizes, all address relocations, CRC/counter rules, aliases/bounds/overflow,
unsupported profiles/geometry, known/unknown initialization flags and pending
state, dense/sparse/sparse2/reserved heads, shared multi-block descriptors,
invalid backups/truncated heads and read faults. Expanded synthetic positive
fixtures initially exposed an accidental bitmap/table overlap and uncleared
descriptor bytes; those fixtures were corrected. They remain synthetic controls,
not e2fsck-clean filesystem claims. The source/resource self-review found no
blocker within the standalone fixed-layout contract; follow-up review and new
independent source qualification remain required.

A new VM oracle prepares **14 profiles**: both block sizes/descriptor widths
across dense, classic sparse with reserved GDT/FLEX_BG and sparse-super2/FLEX_BG,
plus larger multi-descriptor profiles at both block sizes. It requires judged
non-repairing e2fsck cleanliness and exact equality of all typed/grouped protected
intervals with complete saved `dumpe2fs` location reports. It requires every group
report and intended reserved/cross-group/multi-descriptor shapes. The focused
workflow adds an exact test filter, one-test floor and retained images/reports/log.
It is compiled but unrun locally; there is no local KVM, host mount or host
e2fsprogs invocation. Exact-commit VM metadata/insertion/inventory/role/FIEMAP
and full CI remain pending after push; preceding successes do not qualify it.

Final-source full units pass **921 tests**, zero failed/ignored (1,169 lines /
74,363 bytes within the unchanged 1,400-line/85,000-byte budget). Multiset
comparison retains all 904 fresh baseline occurrences and adds exactly the 17
fixed-metadata controls. Lint (fmt/clippy, provenance and Unicode checks),
shared-agent and whitespace checks pass; the saved other-owner patch remains
applicable. The locked release CLI build/install passes and stages all four tools
under `tmp/cli/bin`. Ignored evidence is under
`tmp/casefold-fixed-evidence`, including fresh baseline/red/focused/full logs,
fixture-failure diagnosis, multiset comparison and final validated source hashes.
The three saved other-inode ownership aliases remain unresolved; this collector
does not join ordinary storage claims or validate current allocation. Production
insertion, journal guard, inode inventory/roles, complete extent collector and
feature gates are unchanged; only private preparation and its tests are added.

**Next bounded task:** inspect this checkpoint's exact-commit VM location,
insertion/inventory/role/FIEMAP and full CI first, resolving failures before
broadening source. Review this collector's coverage, current-view, profile and
resource contract before complete ownership construction. Combine fixed ranges,
typed inode roles and complete tree claims only with verified current allocation,
all cross-owner conflicts and shared read/work/peak-capacity budgets. Preserve
all baseline occurrences, saved owner failures/control and operation-wide guard,
overlay/dependency/sealing/invalidation/commit-failure requirements. Do not wire
admission or scan every file on each insertion. Keep every write gate intact.

Stage 5 completion remains **not yet quantified**. Public casefold writes, deeper
insertion, collisions, splitting/growth and index dropping remain blocked.
Windows/SteamOS/real-card use and large-card performance remain unqualified.

## Inode-role implementation/evidence review checkpoint — 2026-10-15

Follow-up: the review landed at `c1afa47` and its exact-commit full CI passed.
The private fixed-metadata component above supersedes its next implementation
task; new-source independent CI and a complete ownership snapshot remain separate.

Reviewed `ef1ca24d89a2b38a0e3a035c63466628bf3daceb`, equal to the fetched
remote tip with a clean tree. Its exact-commit focused Linux and full CI passed;
the preceding documentation review `d650d8e` also passed full CI. Existing Git/
API pull/push and artifact downloads work. The current onboarding setup is reused
without a configuration draft/publication, credential, sibling or dependency/
pin/lock change. No production/test/workflow change or new test is needed here.

Downloaded the actual focused artifact and read all six role images, allocation-
derived command lists and raw type/sector reports, plus the role/inventory/
insertion/FIEMAP logs. Each role image has 37 allocated records with exactly one
oracle type/sector answer per ID. Four no-journal profiles have nine inactive
reserved records, 22 ordinary regular files, three directories (root included),
two symlinks and one FIFO. Two 64-bit journal profiles replace one inactive
record with the explicit journal role, preserving 37 total. A public-format raw
audit also matched the command IDs to both allocation bitmaps and sector fields
to every saved answer. All six Linux populations and judged non-repairing e2fsck
checks passed. Actual counts/log measurements and CI links are recorded above.

The saved inactive records exercise zero extra sections and low-only checksums;
other records exercise 32-byte sections/full checksums. Eight-byte symlinks have
zero sectors; 100-byte symlinks have depth-zero maps and 2/8 sectors respectively.
Depth-one sparse files have 34/136 sectors (16 data plus one map block). Journal
inode 8 has depth-zero storage, 1 MiB/2,048 sectors at 1 KiB and 4 MiB/8,192
sectors at 4 KiB. These are small-image observations, not a large-card benchmark,
full journal/recovery matrix or independent qualification of every feature bit.
Allocated zero-link bodies, staged edits and malformed/unknown-role refusals
remain synthetic controls rather than new Linux fixture claims.

The source/test/resource review in `docs/CASEFOLD-INDEX-REVIEW.md` checked body
bounds, checksum identity/width, type/flag/attribute/storage-free policies, stable
current-view preconditions, fixed allocation and shared error-inclusive budgets.
**No implementation blocker was found within the standalone reader's stated
narrow profile and trusted same-view inventory preconditions.** No fix or test
was added by this review. The reader does not verify allocation, complete extent
trees/sector accounting, fixed roles, cross-owner conflicts or an epoch, and it
does not enforce whole-result publication. Each of those remains a builder gate.

Fresh baseline and final unit runs each pass **904 tests**, zero failed/ignored.
Multiset comparison retains every occurrence with no additions or losses. The
final log is 1,151 lines / 72,914 bytes, within the unchanged tier budget. Lint
(fmt/clippy, provenance and Unicode checks), shared-agent and whitespace checks
pass. The saved other-owner patch remains applicable. This documentation
checkpoint's own full CI is pending after push; it adds no source qualification.
Ignored evidence is in
`tmp/casefold-role-review-evidence`, including the downloaded artifact, per-profile
raw audit, baseline/final unit logs and test-occurrence comparison. The source/
test/workflow hashes still match the preceding validated implementation. Its
locked release CLI and exact-commit CI already passed; no CLI rebuild or new
local VM run is claimed for this documentation-only checkpoint.

**Next bounded task:** implement a private bounded current fixed-metadata
interval collector before joining all owners. Under a validated current primary
layout, enumerate boot/primary/backup superblock and descriptor/reserved-GDT
ranges, every group's bitmap locations and full inode-table spans. Cover FLEX_BG
placement, complete bounds/overlaps and explicit group/read/range/scratch limits;
reject unsupported layouts and errors without partial publication. Preserve the
inventory's current superblock/descriptor identity and narrow relocation policy.
Keep journal storage on the typed inode/complete extent path, counted once.
Add staged checksum/layout/size, exact/exhausted limit and unchanged-state controls,
plus independent VM metadata-location reports. Then review it before combining
roles/trees, current allocation and interval conflicts under shared peak budgets.
This is not a mounted boolean cache or a whole-card scan per insertion. Preserve
all 904 baseline occurrences, saved owner failures/control, operation guard/
overlay/dependency/sealing/commit-failure requirements and every write gate.

Stage 5 completion remains **not yet quantified**. This review advances evidence
and the implementation contract, not production functionality. The three saved
cross-inode aliases remain unresolved. Public casefold writes, deeper insertion,
collisions, splitting/growth and index dropping remain blocked. Windows/SteamOS/
real-card use and large-card performance remain unqualified.

## Private verified current inode-role checkpoint — 2026-10-15

Follow-up: source `ef1ca24` passed exact-commit focused Linux/full CI. The review
above supersedes this historical section's pending status and next review task;
a complete owner snapshot, transaction lifecycle and admission remain separate.

Started at `d650d8efd8ec1ad4fe18b5d3721e1ae91ca894cb`, equal to the fetched
remote tip, with a clean tree. A fresh baseline passed **887 unit tests**, zero
failed/ignored. Inventory source `4309ed1` passed focused Linux allocation,
insertion/FIEMAP and now full CI; the documentation review's full CI is still
pending at the initial check. Existing Git/API bindings report pull/push access.
The existing onboarding setup is reused without an environment draft change,
credential, sibling, dependency/pin/lock or public write-gate change.

Four role-refusal assertions first failed on acceptance through the existing
`buffered_inode_verified` helper: unknown storage flags, external attributes,
inline attributes and storage hidden in an inactive reserved record. A healthy
extent-file control passed. Five tests executed, four failed, one passed, zero
ignored. The existing reader's checksum behavior is unchanged; this demonstrates
why it alone is not a qualified ownership-role interpreter. The same assertions
now pass through the new private `inode_roles::Reader`.

The red log is 44 lines / 1,896 bytes. All five original controls passed after
the new component; the expanded focused run passes 17 (25 lines / 1,663 bytes).

The reader requires a successful current inventory on the same trusted geometry
and stable device/staged view, and only allocated IDs from it. It deliberately
does not rescan allocation/descriptors for every inode or carry an epoch token.
The caller must enforce that precondition, enumerate every ID exactly once and
discard accumulated output on any error before publishing a future snapshot.
No mutation path calls it. Per-record success is not complete owner coverage.

The initial profile requires metadata checksums, 1 KiB/4 KiB blocks, 256-byte
Linux inodes, first ordinary inode 11 and explicit feature allowlists. Unknown
features, resize/quota/orphan/recovery and external-journal profiles refuse.
An internal journal must be declared as inode 8, a nonempty regular inode with
depth-zero extents; the no-journal policy requires zero journal fields. Root
inode 2 must be a directory. Other reserved slots require mode/links zero and
no storage/attributes; mode zero alone never discards a possible owner.

Verified records retain inode/generation, role, size, sectors, links and the
60-byte storage area. Regular files/directories and block-backed symlinks require
qualified extent headers at depth zero or one. Only the storage-defining flags
for that role are allowed; directories additionally admit INDEX and validated
CASEFOLD. Fast symlinks require bounded nonempty non-NUL bytes, zero sectors/
flags and zero unused storage bytes. FIFO records must have zero storage.
Sockets/devices, unknown flags, HUGE_FILE units, legacy/inline maps, external
attributes and nonzero bytes after the admitted extra-inode section refuse.
Extra sections zero/32 and low-only/full inode checksum widths are explicit.
The complete extent collector must still verify all declared records/children;
the classifier does not validate full maps, sector totals or exclusive ownership.

One fallibly reserved table-block buffer is reused; staged blocks require exact
size before copying. Table/body bounds and inode-number/generation checksums
are verified. The checksum copy is a fixed 256-byte stack array, with no per-read
heap clone. Metadata scratch is bounded by `block_size + 256`; fixed reader/
parsed-record stack state, allocator overhead/device cache are outside that
figure. Record attempts and logical fetches, including staged/failing reads,
share limits across reader calls. No result vector/cache is retained. The future
builder must charge retained capacity together with inventory, role/tree output
and peak coexistence, and must establish operation-wide stability/invalidation.

Seventeen focused controls cover the original five plus typed root/reserved/
ordinary/journal roles, staged identity and zero-link records, checksum identity
in disk/staged views, fast/block symlinks/FIFO, invalid types/flags/headers,
low-only checksums, extra/fragment policy, exact/exhausted aggregate limits,
malformed table sizes, address/ID bounds, volume/journal policies and read faults.
The source/resource self-review found no blocker within this deliberately narrow
interpretation contract. Independent validation and a follow-up focused review
remain required before using it in a complete builder.

A new VM oracle formats four no-journal block/descriptor profiles plus two
64-bit internal-journal profiles. Linux initializes both inode groups, creates/
removes files and makes both symlink forms, a FIFO and a depth-one extent file.
It requires judged non-repairing e2fsck cleanliness and exactly one saved
`debugfs stat` type/sector result for every allocated ID, reserved records
included. It requires the fast/long symlink, external-map, FIFO and declared
journal outcomes. The focused workflow adds an exact test filter/one-test floor
and saves images/commands/raw reports/log. It is compiled but unrun locally;
there is no local KVM, host mount or host e2fsprogs invocation. New-source
independent Linux/full CI remains pending after push.

Final-source full unit validation passes **904 tests**, zero failed/ignored
(1,152 lines / 72,974 bytes, within the unchanged 1,400-line/85,000-byte budget).
Multiset comparison retains all 887 fresh baseline occurrences and adds exactly
the 17 role controls. Lint, including fmt/clippy, provenance and Unicode checks,
passes. Guest Python syntax, shared-agent and whitespace checks pass; the saved
other-owner patch remains applicable. The locked release CLI build/install
passes and stages all four tools under `tmp/cli/bin`. Validated source, test and
workflow hashes are saved with the logs. Ignored evidence lives
under `tmp/casefold-role-evidence`; the existing saved three cross-inode alias
failures remain unresolved and their disjoint control remains separate. No
namespace, collector, journal guard, shared mapper or feature gate is changed.

**Next bounded task:** inspect this checkpoint's exact-commit role/inventory/
insertion/FIEMAP and full CI, resolving failures first. Review the new reader's
role coverage, current-view and resource contracts before combining typed roles,
complete extent claims, fixed roles and current allocation under aggregate
snapshot budgets. Complete-result publication, interval conflicts, operation
guard/overlay/invalidation and commit failure rules remain prerequisites for
admission. Do not treat per-record classification as a complete ownership proof
or wire it into insertion yet. Preserve every baseline occurrence, the saved
owner failures/control, pinned siblings and all operation/recovery/write gates.

Stage 5 completion is **not yet quantified**. Public casefold writes, deeper
insertion, collisions, splitting/growth and index dropping remain blocked.
Windows/SteamOS/card use and large-card performance remain unqualified.

## Inode-inventory implementation/evidence review checkpoint — 2026-10-15

Follow-up: inventory source `4309ed1` now passed full CI as well as focused
Linux checks. The private role component above supersedes this review's next
implementation task; it does not implement a complete snapshot or admission.

Reviewed `4309ed114caca9d13c31e6647276ca883c116781`, equal to the fetched
remote tip with a clean tree. Existing Git/API pull/push and artifact-download
access work. The existing onboarding configuration supports this workflow;
no environment draft save/publication or credential change is needed. No source,
test, workflow, sibling, dependency/pin/lock or write-gate change is made here.

The focused Linux workflow passed at that exact source commit. Downloaded and
read the new inventory log, every saved command/allocation report and all four
raw images. One inventory test executed, zero failed/ignored, with successful
Linux population and judged non-repairing e2fsck for both block sizes and both
descriptor widths. Each image has two groups of 32 inode slots, 22/11 allocated
and 10/21 free respectively. All 64 IDs per image have exactly one oracle answer,
including reserved IDs 1–10. The test compares the whole inventory, not just
its count. A public-format audit also compared bitmap bytes with every answer,
checked full-block padding/initialized flags and shared descriptor placement.
Existing insertion and FIEMAP tests passed with their saved logs inspected;
exact measurements and CI links are recorded above. Full source CI is pending
at the initial check. The previous collector review's full CI now passed.

The review in `docs/CASEFOLD-INDEX-REVIEW.md` checked bounded slicing/arithmetic,
complete valid-ID coverage, padding/CRC widths, current superblock/descriptor
identity, staged precedence, flag reconciliation, resource accounting and
failure immutability. **No implementation blocker was found within this private
scan's stated supported profile and stable trusted-mount preconditions.** It
is suitable as a prerequisite for role classification. It returns no epoch or
dependencies, verifies no inode body and proves no exclusive storage ownership.
No production fix or new test is introduced by this review.

The next component must classify every current allocated reserved/ordinary
record, including zero-link records, with explicit verified checksum/storage
policies. These Linux fixtures include mode-zero reserved bodies, none all-zero;
their allocation cannot be ignored based on type or links. They do not qualify
journal/resize/quota/attribute roles or a partial final inode group. Unknown
roles/maps/attributes must refuse rather than disappear from coverage. Use a
stable shared view, bounded current inode reads and the independently qualified
extent depth-zero/one matrix. Inventory capacity, later role/tree output, scratch
and transient coexistence need aggregate snapshot budgets. Full fixed-role/block
allocation validation, cross-owner conflicts, lifecycle/invalidation and sealing
remain later work. BLOCK_UNINIT being allowed by the inode scan does not qualify
block allocation. No census may be reused across allocation changes without
the future epoch/dependency contract.

Fresh baseline and final unit runs each pass **887 tests**, zero failed/ignored.
Multiset comparison retains every baseline occurrence with no additions/losses.
The final log is 1,134 lines / 71,596 bytes within the unchanged tier budget.
Lint (fmt/clippy, provenance and Unicode verification), shared-agent and
whitespace checks pass; the saved other-owner patch remains applicable. Verified
the production/test/workflow hashes still match the preceding validated source.
The CLI is not rebuilt for this documentation-only review; its identical source
already passed the locked release CLI build locally and the exact-commit CI CLI
job. No new local VM run or new-source Linux qualification is claimed. This
documentation checkpoint's own full CI is pending after push. Ignored evidence lives under
`tmp/casefold-inventory-review-evidence`, including the downloaded artifact,
per-profile audit, fresh baseline/final logs and test-occurrence comparison.

**Next bounded task:** verify outstanding exact-commit full CI first, resolving
failures before broadening the source. Then implement a private bounded verified
current inode-role classifier, disconnected from admission, following the review
contract. Establish explicit reserved/root/journal/ordinary and symlink/storage-
free policies for its initial profile, refusing unqualified ones. Add staged
body/identity/checksum/flag/attribute/limit/unchanged-state controls and independent
Linux role evidence. Do not treat census/roles as a complete snapshot or enable
deeper insertion. Preserve all 887 baseline occurrences, the three saved owner
refusal failures and disjoint control, operation/recovery gates and sibling pins.

Stage 5 completion is **not yet quantified**. This documentation review does not
advance implementation. Public casefold writes, deeper insertion, collisions,
splitting/growth and index dropping remain blocked; Windows/SteamOS/card use and
large-card performance remain unqualified.

## Private bounded inode-inventory checkpoint — 2026-10-14

Follow-up: this source landed at `4309ed1`; exact-commit focused Linux allocation,
insertion and FIEMAP passed. The review above supersedes this historical
section's pending focused status and review task; full CI status is recorded above.

Started at `7825062eb7631ba623fc6984abb52caa8fa127fd`, equal to the fetched
remote tip with a clean tree. A fresh baseline passed **872 unit tests**, zero
failed/ignored. The preceding collector source already passed focused Linux and
full CI; the documentation review's own full CI was running at the initial check.
Existing Git/API bindings report pull/push access. No environment configuration,
sibling, dependency/pin/lock or public write-gate change is made here.

The existing mounted `inode_bit_is_set` helper was first used as a test adapter.
Three malformed-current-state controls failed on acceptance: bad staged
descriptor checksum, bad staged bitmap checksum and INODE_UNINIT policy change.
A fourth failed because newly staged inode 42 was omitted. A healthy reserved-ID
control passed. The red run executed five tests: four failures, one passed,
zero ignored (44 lines / 2,116 bytes). These show why the mounted helper cannot
serve as a complete current inventory; its existing behavior is unchanged.
The same refusal/current-ID assertions now pass using the new private inventory.

`inode_inventory::collect` accepts a trusted mounted filesystem, stable current
device/buffer view and explicit aggregate limits. Its initial profile supports
metadata checksums, 1 KiB/4 KiB blocks and 32/64-byte descriptors. META_BG,
bigalloc, INODE_UNINIT and unknown allocation flags refuse. Known unrelated
BLOCK_UNINIT/ITABLE_ZEROED flags remain allowed. Current superblock bytes verify
and match mounted contents except free-counter fields; current descriptors
verify per-group, preserve mounted addresses and agree with pending flags.
Metadata bounds and basic bitmap aliases are checked. Each current inode bitmap
verifies with the correct CRC width, and full group/final partial-group coverage
is checked. All padding bits through the whole bitmap block must be set, even
outside checksum coverage. Every in-range set bit returns an ascending ID;
reserved IDs and allocated zero-link slots are not filtered by inode contents.

One invocation-wide budget covers group count, every full-block bitmap-bit probe,
logical fetches (staged copies included), output IDs and scratch. Fallible fixed
reservations request at most `4 * inode_cap + 2 * bs` bytes, with checked combined
arithmetic; two buffers are reused. Descriptor-block caching lets two neighboring
groups share one read. Reads and pushes check their caps before excess work;
exhaustion/faults discard the complete local result and publish nothing. Malformed
staged block sizes refuse before copying rather than cloning arbitrary vectors.
Allocator overhead, device cache, retained capacity and future tree/role buffers
are separate full-builder accounting requirements; no real-card bound is claimed.

Fifteen local controls now pass, including the original five plus shared primary
descriptors, partial final groups, exact/exhausted limits, padding within/outside
CRC coverage, size/geometry/bounds/relocation errors, unknown/pending flags,
current counter-only superblock edits, current frees and allocated zero-link
records, read errors and complete dirty-map/flag/device immutability. The
two-group partial fixture is synthetic, not an independent Linux-clean image.

The new Linux oracle formats four profiles (1 KiB/4 KiB, 32/64-byte descriptors),
with two groups and 64 inode slots. Linux creates enough files to initialize both
groups, removes alternating files and adds a directory, symlink and hard link.
The test requires non-repairing e2fsck cleanliness, then compares the entire
inventory with every valid inode's `debugfs testi` used/free response; missing or
duplicate responses fail. It verifies reserved coverage and exact four-read scan
limits. Focused CI triggers on the new files, requires one executed test, and
saves images, commands, raw allocation reports and logs alongside the existing
insertion/FIEMAP artifact. This Linux test is compiled but unrun on this host;
there is no local KVM, host mount or host oracle invocation. Public format docs,
own MIT code and black-box VM oracles are the only sources used.

Final-source local validation: **887 unit tests pass**, zero failed/ignored,
in 1,135 lines / 71,656 bytes within the unchanged budget. Multiset comparison
retains all 872 fresh baseline occurrences and adds exactly 15 inventory controls.
The expanded focused run passes 15 tests (23 lines / 1,551 bytes). Lint, including
fmt/clippy, provenance and Unicode verification, guest Python syntax, shared-agent
and whitespace checks pass. The locked release CLI build/install passes and stages
all four tools under `tmp/cli/bin`. The saved other-owner patch remains applicable.
Ignored evidence is under
`tmp/casefold-inventory-evidence`, including baseline/red/focused/full logs,
multiset comparison and validated source/workflow hashes. New-source focused
Linux allocation/insertion/FIEMAP and full CI remain pending after push; the new
oracle is compiled but unrun locally, not claimed passed.

This is an allocation inventory, not a complete owner snapshot. Inode-body
checksums and role interpretation, reserved/journal/symlink/attribute profiles,
complete cross-group fixed-role and allocation checks, interval conflicts,
snapshot retention, operation-wide serialization, current overlays, final
sealing/publication and invalidation/commit failures remain later requirements.
The three saved cross-inode refusal failures remain unresolved. Stage 5 progress
is **not yet quantified**; public writes, deeper insertion, collisions,
splitting/growth and index dropping remain blocked. Windows/SteamOS and real-card
performance remain unqualified.

**Next bounded task:** inspect this inventory's exact-commit Linux allocation,
insertion/FIEMAP logs and full CI, resolving failures first. Review the inventory's
coverage/resource/current-state contract before implementing verified current
inode-role classification. Classify every enumerated reserved/ordinary inode
explicitly, refuse unknown map/attribute profiles and preserve the collector's
independent depth matrix. Combine roles and tree claims only under shared snapshot
budgets; do not wire admission or treat inventory completion as ownership proof.
Preserve the saved three owner refusal cases, their disjoint control, every
baseline occurrence and all operation/recovery/write gates.

## Collector implementation/resource review checkpoint — 2026-10-14

Follow-up: the inode-inventory checkpoint above implements this review's next
private component and supersedes its continuation task; it does not add a full
snapshot, transaction lifecycle or write admission.

Reviewed `7bdfe0cf83d3d65f4068225707c379f3fad9e6aa`, equal to the fetched
remote tip with a clean tree. Git/API pull/push and artifact downloads work
through existing bindings. No environment configuration or credential change,
sibling/dependency/pin/lock change, host oracle/mount, source fix or new test is
needed for this review. The complete standalone collector and tests, staged
view control, caller contract and Linux evidence were inspected. The preceding
source's full CI and focused workflows now passed, as recorded above.

The downloaded new Linux test log confirms one executed test, zero failed/
ignored, and successful Linux/e2fsck checks at both 1 KiB and 4 KiB. The saved
reports and raw public-format image observations agree for four files: each
inline file has depth zero, one data range/block and no map block; each external
file has depth one, 17 data records covering 19 data blocks plus one map block,
for 18 returned claims. Each has three unwritten blocks at logical 1024–1026
beyond visible EOF at block 497. Linux sector totals are respectively 40/160,
including mapping storage. These are profile observations, not a throughput,
peak-RSS or real-card benchmark. Independent deeper-tree evidence was not added.

The review in `docs/CASEFOLD-INDEX-REVIEW.md` records the checked slicing/depth,
key/range, bounds/checksum, complete traversal, overlap, error immutability and
resource invariants. **No implementation blocker was found within this private
primitive's stated supported profile/preconditions.** The caller must still
verify its inode/root and stable view. This review does not certify an owner
index, current allocation, other-inode/attribute separation or admission.

Important requirements for the snapshot builder are now explicit:

- Per-tree read/record/claim/scratch limits reset on each call; aggregate group,
  bitmap/inode probes, reads, records, overlap work, output and allocation limits
  must be shared across the entire scan and remain bounded on errors.
- Output reserves the full claim cap. The current Linux control requests 1,024
  slots even for a one-range inline file. Retaining each vector while charging
  only its length would undercount memory. Charge actual retained capacities or
  fallibly compact into a bounded arena while accounting for peak old/new/tree/
  scratch/dependency allocations. No production defaults are qualified here.
- The independent matrix covers extent depths zero and one. The initial
  snapshot profile must refuse deeper maps or qualify them independently first;
  synthetic depth-two-through-five controls alone do not widen that matrix.
- Enumerate all allocated inode IDs, including reserved and unlinked slots;
  reachable names are insufficient. Unknown roles/profiles and partial scans
  cannot disappear from the proof. Allocation/fixed roles, complete snapshots,
  transaction overlays, operation serialization, final sealing, publication,
  invalidation/failure handling and exclusive device use remain later gates.

Fresh local validation: **872 unit tests pass**, zero failed/ignored, both
before and after the documentation update. The final log is 1,119 lines /
70,390 bytes within the unchanged budget. Multiset comparison retains every
baseline occurrence; no active test is added or removed. Lint, including
fmt/clippy, provenance and Unicode verification, shared-agent and whitespace
checks pass. The saved other-owner patch still applies. Source, tests, workflow,
dependency files and shared instructions are unchanged. The release CLI is not
rebuilt for this documentation-only review; the identical source's locked
release/CLI and exact-commit full CI already passed. Ignored evidence is under
`tmp/casefold-collector-review-evidence`, including the downloaded artifact,
profile observations, fresh logs and multiset comparison. This review commit's
own full CI is pending after push; no new-source Linux run is claimed here.

**Next bounded task:** implement a private bounded allocated-inode inventory,
the first snapshot-building component, disconnected from insertion admission.
Verify current descriptor and inode bitmap checksums/bounds, initialization and
unknown flags, final-group coverage and disk/staged views before returning all
set inode IDs. Define the initial initialization-state policy explicitly; refuse
unqualified states rather than reading unspecified bitmap bytes. Test reserved
IDs, allocated unlinked slots, staged bit changes, shared descriptor blocks,
invalid checksum/size/tail/group geometry, exact/exhausted aggregate limits and
unchanged inputs on refusal. Require independent Linux inventory controls for
the admitted profile. Do not parse every reserved slot as a normal file or call
an inventory a complete owner snapshot; typed roles, tree claims, allocation and
cross-owner conflicts are subsequent components. Preserve all 872 baseline test
occurrences, Linux insertion/FIEMAP controls and every existing write gate.

Stage 5 progress remains **not yet quantified**. The three saved cross-inode
refusal failures remain unresolved, and no implementation progress is inferred
from this documentation-only checkpoint. Public writes, deeper insertion,
collisions, splitting/growth and index dropping remain blocked. Windows/SteamOS
and real-card performance remain unqualified.

## Private bounded extent-ownership collector checkpoint — 2026-10-14

Follow-up: this source landed at `7bdfe0c`, with exact-commit focused Linux
FIEMAP/insertion and full CI now passed. The implementation/resource review
above supersedes this historical section's pending status and next task.

Started at `80caea699f4c66b6b1a5d7e4a3eb716b79a9e7a5`, equal to the fetched
remote tip, with a clean tree. The fresh unit baseline passed **852 tests**,
zero failed/ignored. That documentation checkpoint's full CI is still running
at the initial check; the actual preceding journal source already passed focused
Linux and full CI at `f9a0df0`. Existing Git/API bindings report pull/push access.
No environment configuration, sibling, dependency/pin/lock or write-gate change
is made in this checkpoint.

Seven negative whole-tree cases against the old `collect_all_with_nodes` helper
failed on acceptance and a disjoint control passed (eight executed, zero ignored).
The cases concern an unselected child's checksum, exact depth, parent key/
shadowed mapping, external capacity, duplicate child, data/map overlap and zero
length. These demonstrate why an unchecked public helper is not an ownership
proof; its public behavior is preserved. The same assertions now pass against
the new private `extent::ownership::collect` primitive.

The collector requires a verified inline inode root, checksum identity and stable
device or staged view, plus explicit filesystem bounds and caller limits. It
supports checksummed 1 KiB/4 KiB full-capacity external extent nodes, validates
all declared records and children, ordered logical/parent key intervals, exact
depth/capacity and physical ranges. Every child checksum is checked before
following its pointers. It returns physically sorted disjoint data/map intervals
and inode/generation identity, preserving unwritten and beyond-EOF extents without
reading payload or expanding data runs. Any failure discards the whole local
result. Smaller external capacities, other block sizes or disabled checksums are
unsupported; legacy maps and attribute roles are outside this primitive.

Read, examined-record, output-claim and scratch limits are explicit; there are
no proposed production defaults here. Output capacity and per-depth scratch
are reserved fallibly with checked byte arithmetic before traversal. Owned
allocation requests are bounded by `claims * size_of::<Claim>() + depth * bs`,
plus fixed-size state and bounded traversal/sort stack; allocator overhead and
device caches are separate.
At most five external levels are allowed. Sibling visits reuse the same buffers;
in-place physical sorting has bounded `O(C log C)` work. Exhaustion returns an
explicit resource refusal, never a partial owner list or unbounded scan.

The new filesystem-level control uses the actual `StagedTreeNodes`: a staged
child edit changes a data claim without editing the inode, while the disk view
retains its original claim. A bad staged checksum refuses with the full dirty
map, pending flags and device bytes unchanged. Expanded collector tests cover
budget boundaries, cycles, maximum depth, sparse/unwritten/beyond-EOF storage,
logical/address limits, inode checksum identity, read errors and profile refusals.

The new Linux oracle test creates two block-size profiles, each with inline and
external files, and records kernel FIEMAP intervals/unwritten flags and allocated
sectors. It compares all data claims and the combined data/map block count,
including KEEP_SIZE allocation beyond visible EOF, and requires non-repairing
e2fsck cleanliness. The focused workflow now triggers on extent changes and runs
this test with an execution floor of one. It preserves reports/images/logs with
the existing artifact. The test is compiled locally; no KVM/host mount/oracle is
used here, so its independent result remains pending until exact-commit CI runs.
Only published format/FIEMAP documentation and own MIT sources were consulted.

Local validation on the final source: **872 unit tests passed**, zero failed or
ignored (1,120 lines / 70,450 bytes), within the unchanged 1,400/85,000 budget.
Multiset comparison retains all 852 fresh baseline occurrences and adds exactly
20 local tests: 19 collector tests and one real staged-buffer control. The
expanded focused run passed 19 tests (27 lines / 2,039 bytes); the staged control
passed one (8 lines / 377 bytes). The initial red run executed eight tests:
seven acceptance failures, one passing control, zero ignored (65 lines / 3,458
bytes). A test-fixture trait error was corrected before the actual red run;
compile failures are not counted as that behavioral evidence. The cycle fixture
was corrected to clear obsolete pointer bytes before the expanded green run.
Lint, including fmt/clippy, provenance and Unicode verification, passes; Python
guest-script syntax, shared-agent and whitespace checks pass. The locked release
CLI build/install also passed, staging all four tools under `tmp/cli/bin`.
The saved other-owner patch remains applicable without modification. No test is
disabled or budget increased.
Ignored evidence is under `tmp/casefold-extent-owner-evidence`, including baseline,
red, focused/staged/full green logs, multiset comparison and final source hashes.
New-source focused Linux insertion/FIEMAP and full CI remain pending after push;
the compiled Linux control is unrun locally, not claimed passed.

This completes the first private primitive from the reviewed ownership design,
not global ownership or a casefold write path. The three cross-inode failures
in `docs/CASEFOLD-OWNERSHIP-RED.patch` remain unresolved. Current inode bitmap/
reserved-role enumeration, allocation/fixed-role checks, full snapshot and
transaction overlays, operation-wide serialization, final sealing/publication,
invalidation/commit failure and independent recovery qualification remain open.
Stage 5 completion is **not yet quantified**. Public writes, deeper insertion,
collisions, splitting/growth and index dropping remain blocked; Windows/SteamOS
and real-card capacity/performance remain unqualified.

**Next bounded task:** inspect this collector's exact-commit focused Linux
insertion/FIEMAP logs and full CI, resolving failures first. Review the complete
collector and its explicit resource profile before implementing the bounded
snapshot builder for an explicit inode/feature profile. Reuse the design's
bitmap/reserved/attribute and failure contracts; do not wire admission or relax
gates until snapshot, serialization, current overlays and final commit validation
are implemented. The saved three owner refusals plus disjoint control are later
integration acceptance criteria, not claimed fixed by this collector.

## Transaction-current ownership design checkpoint — 2026-10-14

Follow-up: the collector checkpoint above implements its first private primitive
and supersedes this historical section's next task; no owner-index integration
or write-gate change follows from that implementation alone.

Reviewed at `f9a0df0544072b0e252d5d4ff2912c70f1838a0b`, with the source and
remote tip equal and a clean tree. Git/API/artifact access and pull/push
permissions work through the existing bindings; no environment configuration
change, new credential, sibling edit, dependency/pin/lock change or host oracle
was needed. Both exact-commit journal CI runs above passed, as did the preceding
ownership review's full CI. The focused artifact was downloaded and its actual
insertion log inspected, including all four 1 KiB/4 KiB strict/non-strict profiles.

A fresh unit baseline passed **852 tests**, zero failed/ignored. Applied the
unchanged saved `docs/CASEFOLD-OWNERSHIP-RED.patch` temporarily: its four tests
execute, with **three acceptance failures** at blocks 517, 521 and 549 and
**one passing disjoint control**, zero ignored (37 lines / 1,893 bytes). Reversed
the patch after preserving the log. These are still unresolved refusal tests;
their unchanged-state assertions have not passed. Production source is restored
byte-for-byte, not left with expected-failing tests in the active suite.

The focused design review in `docs/CASEFOLD-INDEX-REVIEW.md` records concrete
source constraints: current admission's two mapping views share the current
inode root; visited nodes are not a complete tree walk; raw dirty-map edits can
change owners without `buffer_write_inode`; the journal mutex starts at commit;
direct writes/replay/refresh and public device access prevent assuming a stable
mount epoch. A shared attribute block needs roles/multiple references.

The selected design is a complete bounded snapshot once per valid epoch, with
interval claims and reverse dependencies, plus an isolated transaction-current
overlay. Unsupported inode/map/reserved/attribute profiles or exhausted limits
refuse completion; partial coverage never means unowned. Include allocated
unlinked inodes, unwritten/beyond-EOF extents, every map node, current allocation
and complete tree structure. Derive dependencies from final dirty bytes; compare
individual inode slices; handle newly allocated/freed/reused slots and child-map
edits without inode edits. Retain committed claims until success, initially
refusing cross-inode reuse within one transaction. Final edits invalidate prior
admission. Publish a prepared overlay only after durable success; partial or
uncertain commit invalidates proof and stops writes. An operation guard must
span resolution/duplicate checks through commit/publication; explicit refresh,
raw mutation and recovery invalidate the epoch. External mutation remains
unsupported. The review defines budgets for all work/peak memory, resource-limit
failure behavior, lock-order requirements and staged acceptance tests. Production
limits and real-card performance remain unmeasured; this is not implementation
or write qualification.

After reversing the patch, the full unit tier again passed **852 tests**, zero
failed/ignored, within its unchanged output budget (1,100 lines / 68,661 bytes).
Multiset comparison retains every one of the 852 fresh baseline test occurrences,
with none added or removed. Fresh lint (fmt, clippy, provenance and Unicode
verification), shared-agent and whitespace checks pass. The release CLI is not
rebuilt for this documentation-only review; its implementation checkpoint build
and exact-source full CI passed. Production source is unchanged. Ignored evidence is under
`tmp/casefold-owner-design-evidence`; the original four counterexamples stay in
the existing committed patch. No new independent oracle is claimed for the
proposed owner index. Stage 5 progress remains **not yet quantified**, and the
three cross-inode aliases, mapping-shape integration, operation concurrency,
crash qualification, deeper insertion and collision/splitting/growth remain open.
Public casefold writes, index dropping and Windows/SteamOS use stay unqualified.

**Next bounded task:** add a private read-only complete extent-ownership
collector with explicit caller-supplied node-read, record, claim and scratch
budgets. Reproduce its missing whole-tree/shape checks before fixes; verify
inline/external initialized and unwritten/beyond-EOF claims, unused/shadowed
children, checksums/depth/key/capacity, duplicates/cycles, bounds and limit
failures in device and staged views. Require exact role/range output, unchanged
inputs on failure and independent Linux healthy-image controls. Keep it
disconnected from insertion admission until snapshot, serialization, overlays,
invalidation and final commit validation are implemented and reviewed; those
later gates must make the saved three owner refusals and disjoint control green.

## Scoped inline-journal role refusal — 2026-10-14

Follow-up: this source landed at `f9a0df0` and its exact-commit focused Linux
and full CI now passed, as recorded above. The design checkpoint supersedes
this historical section's next task; the guard's implementation scope is unchanged.

At source `da2ffcd321c67093573bd83d4bd1cc77b749cd5d`, a fresh local baseline
passed **839 unit tests**, zero failed or ignored. The preceding allocation
implementation already passed exact-commit focused Linux/full CI; the review's
own full CI is still pending as recorded above. This checkpoint keeps all public
casefold writes blocked and introduces no sibling, dependency/lock or pin edits.

Three negatives reproduced `Ok(())` when the staged directory root, selected
leaf or external directory-map node occupies **journal data block 522** (journal
logical block 1). Journal block 0 remains intact and parses as clean. A positive
disjoint-storage control passed. The red run executed four tests: one passed,
three acceptance failures, zero ignored (37 lines / 1,824 bytes). The same
negative tests now pass their exact journal-alias refusal and full dirty-map,
pending-flag and device-byte assertions.

The self-contained fixture allocates eight blocks through ordinary file APIs,
transfers that storage to checksummed reserved journal inode 8, releases the
temporary name/inode, and declares journal/casefold policy before the read-only
mount. It verifies reserved-inode allocation and a clean, parseable journal.
This is a synthetic planner fixture, not an independent Linux-clean image claim.
The existing four Linux-created profiles remain the independent healthy-image
matrix (1 KiB/4 KiB, strict/non-strict); no host oracle/mount is used locally.

The private guard runs after existing read-only directory/mapping/layout/
allocation checks and before insertion edits. It reads journal declarations,
not the absent live writer. The initial internal profile requires consistent
presence/inode fields, no external-device/UUID declaration, a stable checksummed
regular inode with only the extent flag and no external attributes, and current
initialized/checksummed inode allocation. It inspects every declared inline
extent record, requiring initialized nonzero lengths, complete ordered logical
coverage equal to the file size, valid capacity/bounds, no physical overlap and
no fixed filesystem metadata. Every journal block needs current allocation.
External/legacy journal maps, unwritten/preallocation outside this size profile
and staged journal inode edits remain unsupported.

All admitted directory blocks and mapping nodes visited in either view must be
disjoint from the journal ranges, including unselected leaves and staged-only
nodes. A different inode sharing the journal's inode-table block remains allowed;
the guard compares the journal slice itself. The descriptor checks are shared
with existing allocation admission. Fixed metadata now has an interval helper,
with the previous point check delegating to it, so journal ranges are checked
against descriptor roles without a block-times-group nested scan. The inline
profile bounds journal allocation probes to four records / 131,072 blocks.
This introduces no full-volume inode scan or ownership cache.

The expanded focused run passed **68 tests**, zero failed or ignored (76 lines /
5,969 bytes). New controls cover current/disk inode checksums and truncation,
checksum-valid staged inode edits, whole inline mapping shape, allocation bitmap
checksums/sizes/flags/bits, unallocated journal tails, external/inconsistent
declarations, shared inode-table blocks, two-record maps, unselected leaves and
staged-only map nodes. The full unit tier passed **852 tests**, zero failed or
ignored, in 1,100 lines / 68,661 bytes within its unchanged budget. Multiset
comparison retains all 839 baseline occurrences and adds exactly 13 journal
tests. Lint (including provenance/Unicode tables), shared-agent/whitespace checks
and locked release CLI build/install pass. Fresh evidence is under ignored
`tmp/casefold-journal-role-evidence` and `tmp/logs/casefold-journal-{red,green}.log`.
Its new-source exact-commit focused Linux and full CI subsequently passed at
`f9a0df0`; see the current evidence above.

Stage 5 remains **not yet quantified**. This addresses journal roles within the
restricted private profile, not general inode/attribute ownership, JBD2 lifecycle,
whole-operation serialization, concurrent/external mutation or complete directory
map shape. The three other-owner counterexamples in
`docs/CASEFOLD-OWNERSHIP-RED.patch` remain unresolved; a journal-only guard cannot
establish a global owner proof. Public writes, deeper insertion, collisions,
splitting/growth and index dropping remain blocked. Windows/SteamOS is unqualified.

**Next bounded task:** inspect this journal guard's exact-commit focused Linux
insertion log and full CI; resolve failures before broadening support. Then
perform a focused design review of bounded transaction-current owner tracking:
define supported inode/map/attribute roles, memory/work bounds, updates and
invalidation/rollback under operation-wide serialization. Use the three saved
other-owner reproductions and disjoint control as acceptance criteria before
implementation. Do not substitute a scan of every file on each insertion or a
mounted-only cache for a current ownership proof. Keep finding 3 and write gates.

## Current ownership review and regression preparation — 2026-10-14

Follow-up: the scoped journal checkpoint above reproduces and fixes its journal
aliases. The three other-inode counterexamples in this historical review remain
open; the new checkpoint supersedes this section's next task.

At source `96b3b72b7a51ba49f5918044038551942893eafa`, a fresh local baseline
passed **839 unit tests**, zero failed or ignored. Its exact-commit focused
Linux and full CI passed; the retained insertion log was downloaded and inspected
as recorded above. No production, dependency/lock, sibling or write-gate edits
are introduced by this review checkpoint.

Three self-contained synthetic planner counterexamples demonstrate cross-inode
ownership acceptance after the production current-allocation guard succeeds:

- The default staged directory leaf at block 517 is mapped by `lost+found` on
  disk and in its current checksummed inode.
- A directory-map node at block 521 is the allocated payload of a normal file
  created and written before enabling the casefold mount.
- A directory-map node at block 549 is owned by that file only in its checksummed
  buffered inode. The file's disk inode still points at block 521, so a mounted
  owner snapshot would miss this case.

For the map-node cases, directory data is moved to explicitly allocated blocks
8190/8191 to isolate the node alias from `lost+found`. All relevant bitmap and
inode/extent checksums verify; the original allocation guard passes before
insertion. The file payload deliberately contains valid extent-node bytes;
the planner still must respect its other owner. These are staged corruption
counterexamples, not Linux-clean committed operations or exposed public writes.

The completed focused run executes four tests: **three fail on `Ok(())`, one
disjoint-storage control passes, zero ignored** (37 lines / 1,890 bytes). The
pending-owner case is a separate test, so the earlier disk-owner refusal failure
does not prevent it from executing. The three post-refusal dirty-map, allocation-
flag and device-byte assertions have not passed: execution stops at acceptance.
The positive control verifies stored spelling/inode and change to only one leaf;
its named file and `lost+found` do not overlap admitted directory/map storage.
It is not a whole-volume owner audit. No ownership fix is claimed.

The tests are retained in
[CASEFOLD-OWNERSHIP-RED.patch](CASEFOLD-OWNERSHIP-RED.patch), outside active
source pending the ownership design/fix. No tests are ignored, weakened or
relabeled passing. The source is restored byte-for-byte and the patch applies
cleanly to this reviewed source. Tested reproduction source blob:
`d084a7b4c0c4811a88d5680466bb716bf4426fb7`; retained `src/fs.rs` remains
`31613a61b4b3fedfc9564f8e0c23ac1d8e4a13dc`.

```sh
# Preserve unrelated local work; use the reviewed source or an identical fs.rs.
git apply --check docs/CASEFOLD-OWNERSHIP-RED.patch
git apply docs/CASEFOLD-OWNERSHIP-RED.patch
/workspace/.cloud-setup/run-casefold.sh scripts/tier.sh casefold-ownership-red casefold-ownership-red 230 26000 -- scripts/test.sh --locked --lib fs::tests::casefold_ownership_
# Expected before an ownership fix: exit 101; three acceptance failures, one pass.
# Preserve the log. Reverse only this patch if no subsequent source edits exist:
git apply --reverse docs/CASEFOLD-OWNERSHIP-RED.patch
```

The helper path is specific to this cloud instance; use equivalent pinned setup
in a new environment. These reproductions require no VM, artifact download or
host oracle. Local evidence is under ignored
`tmp/casefold-ownership-review-evidence` and `tmp/logs/casefold-ownership-red.log`.

After saving/reversing the reproduction patch, the full retained local unit tier
passed **839 tests**, zero failed or ignored, in 1,087 lines / 67,530 bytes within
its unchanged budget. Multiset comparison retains exactly all 839 fresh baseline
occurrences, none added or lost. These passes concern the retained implementation,
not the three unresolved ownership refusals. Fresh lint (including provenance
and Unicode tables) and locked release CLI build/install pass. The new patch
reconstructs the exact tested source blob and applies cleanly; shared-agent,
tracked provenance and whitespace checks also pass.

The [focused review](CASEFOLD-INDEX-REVIEW.md) also establishes journal requirements.
A casefold mount's absent live `JournalWriter` does not establish no journal.
The existing four Linux insertion images declare journal inode 8, 4 MiB,
depth-zero single inline extent: physical 16899 + 4096 blocks at 1 KiB, or
1033 + 1024 blocks at 4 KiB. These are raw format observations of already
validated images, not a new malformed-journal test. A first scoped journal guard
can qualify that complete inline shape, explicitly refuse unqualified maps/
external journals and staged journal relocation, and preserve every directory/
map role before insertion. Other-owner admission still needs a bounded current
owner map, current inode bitmap/flag/checksum handling and operation-wide
serialization. A full-volume scan on every insertion is not a practical plan
for the intended 1.5 TB card.

Stage 5 remains **not yet quantified**. This checkpoint establishes ownership
gaps and the next contract; implementation support does not increase. Public
casefold writes, deeper insertion, collisions, splitting/growth and index dropping
remain blocked; Windows/SteamOS qualification remains outstanding.

**Next bounded task:** inspect this review checkpoint's exact-commit full CI,
then reproduce a journal-storage alias (including a nonzero journal data block)
before implementing the bounded journal-role refusal under the review contract.
Preserve the four existing Linux profiles, all 839 baseline occurrences and the
disjoint allocated-storage control. A journal-only guard leaves the three
cross-inode aliases open: define a bounded transaction-current owner-map strategy
before claiming general ownership admission. Keep complete map shape and
serialization open; do not enable writes or deeper insertion.

## Current-allocation admission refusal — 2026-10-14

Follow-up: exact-commit focused Linux and full CI at `96b3b72` now passed;
its insertion log was downloaded and inspected. The ownership review above
supersedes this section's next task. Its new cross-inode aliases are not fixed.

At source `065f9a8ef994d848f4f41575f1546cf425a264f5`, a fresh local baseline
passed **825 unit tests**. Both preceding full CI runs now passed, and the
earlier focused insertion artifact was already downloaded and inspected.
Applying the saved allocation patch and rerunning its exact prefix reproduced
three failures on `Ok(())`, zero passed or ignored. The same tests are now active
in `src/fs.rs`; the historical patch must not be reapplied to this source.

After the existing read-only directory/mapping admission and staged layout
checks, before insertion edits, the new private guard requires allocation of
every admitted directory block and mapping node visited in either disk/staged
view. It reads full current descriptors/bitmaps through the buffer with device
fallback, verifies checksums and stable metadata pointers, reconciles pending
initialization flags, refuses unknown or uninitialized flags, and checks actual
group-relative used bits. It refuses bigalloc rather than applying ordinary
block indexing to clusters. Per-group bitmap reuse is limited to the call.
Missing bits return corruption; no growth/index-drop signal is introduced.

The original three tests now pass their refusal and unchanged dirty-map,
allocation-flag and device-byte assertions. Additional controls cover current
disk/staged bitmap checksums, current unstaged descriptor checksums, bitmap
size, descriptor/pending flags, bigalloc, new staged allocations, initialized
allocation clearing, an unselected free leaf and a free mapping node reachable
only in the staged view while data mappings agree. The focused run passed
**54 tests**, zero failed or ignored; the subsequent full tier passes all
**55 fs casefold tests**, including the added 32-byte descriptor control and
its truncated bitmap checksum format.

Five positive tests initially failed because their old synthetic storage was
free. Their fixtures now explicitly stage real bitmap/checksum/free-counter
updates; all prior name/inode, checksum, boundary and unchanged-state assertions
remain. The previous superblock free-counter test still checks the same one-count
delta, now from current staged counters after fixture allocation. No ignored
tests, production bypasses, sibling edits or lock/pin changes are introduced.
Planner fixtures still do not establish exclusive inode ownership or Linux-clean
committed images. Independent validation uses the existing Linux-created profiles.

Local evidence is under ignored `tmp/casefold-allocation-fix-evidence` and
`tmp/logs/casefold-allocation-{red,green}.log`. The full local unit tier passed
**839 tests**, zero failed or ignored, in 1,087 lines / 67,531 bytes within its
unchanged budget. Multiset comparison retains all 825 baseline occurrences
and adds exactly these 14 allocation tests. Fresh lint (including provenance
and Unicode tables), whitespace/shared-agent checks and locked release CLI
build/install pass. New-source exact-commit focused Linux and full CI remain
pending after push. No host oracle/mount or forbidden source was used.

This is a bounded allocation guard, not a counter-accounting audit, complete
owner map or serialization proof. Journal/other inode ownership, unused/shadowed
map branches and complete map shape remain open. Stage 5 remains **not yet
quantified**. Public casefold writes, deeper insertion, collisions, splitting/
growth and index dropping remain blocked; Windows/SteamOS use is unqualified.

**Next bounded task:** inspect this guard's exact-commit focused Linux insertion
log and full CI; resolve failures before broadening support. Then review current
journal/other ownership and reproduce one bounded allocated-storage alias before
implementing its refusal. A used bitmap bit alone must not be treated as proof
of directory ownership. Keep finding 3's complete-map and serialization work open.

## Allocation/ownership review and regression preparation — 2026-10-14

Historical preparation: the current-allocation checkpoint above incorporates
the saved tests and supersedes this section's next task. Both preceding full
CI runs now passed. Do not reapply its historical patch to the new active tests.

At source `b7b0325eeaa6d8e2f46f1e346c3ea680e875b3c3`, a fresh local baseline
passed **825 unit tests**, zero failed or ignored. The focused Linux insertion
log was downloaded and inspected as recorded above; full CI remains pending.
No production source, dependency/lock, sibling or write gate changes here.

Review finding 2 now has three demonstrated missing-allocation cases:

- A valid external extent node at block 549 is marked free on disk.
- Valid staged directory root/leaf blocks at 8190/8191 are within the filesystem,
  outside fixed metadata and marked free.
- Directory root block 516 is allocated on disk but free in the current staged
  bitmap, with refreshed bitmap and descriptor checksums.

Each new test verifies current initialized descriptor flags, descriptor and
bitmap checksums and actual allocation bits before insertion. The external
extent tail verifies too. **All three fail because insertion returns `Ok(())`,
not because setup, checksums or fixtures are missing.** These are synthetic
private-planner counterexamples, not public volume writes or Linux-clean images.
The post-refusal dirty-map/allocation-flag/device assertions have not yet passed:
the red run stops at each acceptance assertion. There is no green fix claim.

The failing tests are retained in
[CASEFOLD-ALLOCATION-RED.patch](CASEFOLD-ALLOCATION-RED.patch), outside active
test source while the bounded allocation contract and fixture updates are
prepared. No tests are ignored, bypassed or relabeled passing. The active source
was restored byte-for-byte, and `git apply --check` confirms the patch applies.
The red tier executed exactly three tests: zero passed, three failed, zero
ignored (36 lines / 1,628 bytes). Local evidence is under ignored
`tmp/casefold-allocation-review-evidence` and `tmp/logs/casefold-allocation-red.log`.
The saved patch reconstructs tested source blob
`ad76a284fbeba3f71e76915b786f9d4a0098cf62`; retained `src/fs.rs` is unchanged
at blob `07691ec3c6f5c28c5657f81db1f34ffc14b1b0b2`.

After saving/reversing the reproduction patch, the active full local unit tier
again passed **825 tests**, zero failed or ignored, in 1,073 lines / 66,364 bytes
within its unchanged budget. Multiset comparison retains exactly all 825 fresh
baseline occurrences, with none added or lost. These passes concern the retained
suite, not the three intentionally failing reproductions.
Fresh lint (including provenance and Unicode-table checks), shared-agent and
whitespace checks pass, as does the locked release CLI build/install. The newly
tracked patch also passes provenance/whitespace checks and applies cleanly.

To reproduce against the reviewed source or a later checkpoint with identical
`src/fs.rs`, first preserve unrelated local changes and verify patch applicability:

```sh
git apply --check docs/CASEFOLD-ALLOCATION-RED.patch
git apply docs/CASEFOLD-ALLOCATION-RED.patch
/workspace/.cloud-setup/run-casefold.sh scripts/tier.sh casefold-allocation-red casefold-allocation-red 180 20000 -- scripts/test.sh --locked --lib fs::tests::casefold_allocation_refuses
# Expected before the fix: exit 101; all three fail on Ok(()).
# Preserve the log. Reverse only this patch if no subsequent source edits exist:
git apply --reverse docs/CASEFOLD-ALLOCATION-RED.patch
```

The helper path above belongs to this cloud instance; fresh environments should
use the equivalent pinned setup documented below. These tests need no VM or
host oracle. A later fix must incorporate and retain the same refusal assertions
in active source, prove them green, and run the full baseline and independent CI.

The [focused review](CASEFOLD-INDEX-REVIEW.md) specifies the next guard: all
admitted directory blocks and mapping nodes visited in either view must have
current initialized/checksum-valid bitmap evidence; use transaction-current
descriptors/flags, not only mounted caches. Initially refuse uninitialized-group
and bigalloc profiles rather than guessing allocation. Correct newly staged
allocations need positive controls, and every existing planner fixture that
relies on free storage needs an explicit allocation update with its original
behavioral assertions retained. A used bit still does **not** establish which
inode owns storage. Journal/other ownership, unused/shadowed map branches,
complete map shape and operation-wide serialization remain open.

Stage 5 remains **not yet quantified**. This checkpoint establishes the defect
and fix contract; it does not advance write support. Public casefold writes,
deeper insertion, collisions, splitting/growth and index dropping remain blocked.

**Next bounded task:** inspect the outstanding full-CI outcome at `b7b0325` and
the review checkpoint's own runs; resolve failures before broadening support.
Apply the saved allocation regressions, implement the scoped current-allocation
refusal under the review contract, and explicitly update planner fixtures.
Prove the same three tests green, add allocated/currently-staged positive and
malformed bitmap/flag/profile controls, retain all 825 baseline occurrences,
then run lint, release CLI and exact-commit focused Linux/full CI. Do not claim
ownership qualified or broaden index depth from a bitmap-only fix.

## Staged superblock counter-only contract — 2026-10-14

Follow-up: its focused Linux workflow now passed and the retained insertion log
was inspected. Full CI remains pending. The allocation-review checkpoint above
supersedes this section's next task; production source is unchanged by that review.

At source `1a6e8d1da3ddad40f91eb2ea0972c13a9019d690`, a fresh local baseline
passed **820 unit tests**. Its exact-commit focused Linux insertion and full CI
completed successfully. The retained insertion log was downloaded and inspected:
one executed test, zero failed or ignored, with independent Linux/e2fsck checks
across all four existing profiles. New-source evidence remains separate.

Four new negative tests reproduced `Ok(())` despite staged superblocks with
changed inode counts, feature bits, write time or a broken checksum. The tests
also cover block/group/inode geometry, descriptor layout, backup locations,
UUID/checksum/hash/Unicode policy, other non-counter bytes and truncation.
Initial red runs stop at the first failed refusal; later loop cases complete
their unchanged-state assertions in the green run. The free-counter control
passed before and after the fix, for inline and external maps. These are
synthetic planner fixtures, not newly qualified filesystem profiles.

After read-only directory/mapping admission, before the descriptor check and
before editing/staging insertion bytes, the new helper locates the primary
superblock using mounted block size and byte offset 1024. It requires a full
staged block and complete 1,024-byte superblock region, verifies the checksum
using the mounted checksum policy, then compares every byte before the checksum
with the mounted raw superblock. Only free-block low/high counters and the free
inode counter are exempt (0x0C–0x13 and 0x158–0x15B); their checksum must still
verify. It does not parse an unqualified new geometry or infer permission from
unknown/reserved fields. Other staged superblock edits return `Unsupported`;
malformed size/checksum returns the corresponding corruption/checksum error.

All five focused tests pass. Each refusal compares the entire dirty map,
allocation flags and device bytes. The control uses the real buffered counter
patch, parses/verifies the staged superblock, checks expected free counts and
the inserted name/inode, and requires only the selected leaf to change. The
superblock buffer itself, allocation flags and device bytes remain unchanged
by insertion. Informational, orphan-list and other non-counter edits also remain
unsupported until separately qualified. This is a stability/refusal contract,
not a free-space accounting audit or a general superblock-mutation feature.

Together with the descriptor guard, this keeps staged layout/policy edits from
silently invalidating mounted-role assumptions at the insertion write boundary.
It does not qualify external device changes, allocation/other ownership,
journal storage, unused mapping branches, complete mapping shape or whole-operation
serialization. Findings 2 and 3 stay open. Stage 5 completion remains **not yet
quantified**. Public casefold writes, deeper insertion, collisions,
splitting/growth and index dropping remain blocked.

The full local unit tier passed **825 tests**, zero failed or ignored, in
1,073 lines / 66,364 bytes within its unchanged budget. Multiset comparison
retains all 820 baseline occurrences and adds exactly these five tests.
Lint, provenance, whitespace and shared-agent checks pass. The locked release
CLI build/install passes. Fresh evidence is
under ignored `tmp/casefold-staged-superblock-evidence` and
`tmp/logs/casefold-staged-superblock-{red,green}.log`. New-source exact-commit
focused Linux and full CI remain pending after push.

**Next bounded task:** inspect this fix's exact-commit focused insertion log and
full CI outcomes. Resolve failures before broadening support. Then perform a
focused review of remaining allocation/ownership admission: define how current
bitmap/descriptor flags and journal/other ownership must be proved, and reproduce
one bounded missing-allocation counterexample before implementing its refusal.
Existing synthetic planner blocks are deliberately not allocation/ownership
evidence; preserve all baseline test occurrences and update their fixtures
explicitly if a new allocation precondition requires it. Complete mapping shape
in finding 3 and serialization remain open. Do not enable writes or deeper insertion.

## Staged descriptor relocation refusal — 2026-10-14

Follow-up: exact-commit focused Linux and full CI at `1a6e8d1` now passed; its
retained insertion log was inspected. The staged-superblock checkpoint above
supersedes this section's pending status and next task.

At source `986808c9cbdcd3ab360f88272d3181ff4ed5aae8`, a fresh local baseline
passed **815 unit tests**. Its exact-commit focused Linux insertion and full CI
completed successfully. The retained insertion log was downloaded and inspected:
one executed test, zero failed or ignored, with independent Linux/e2fsck checks
across all four existing profiles. New-source evidence remains separate.

Four new negative tests reproduced `Ok(())` despite staged descriptors that
relocated a bitmap or inode table into directory data, changed high pointer
halves, or carried a broken checksum. The same tests also require refusal
for relocation into visited mapping nodes and for truncated descriptor blocks;
those later loop cases first complete their refusal assertions in the green run.
The positive counter-only control passed before and after the fix, for inline
and external maps. These are synthetic planner fixtures, not qualified allocated
images or legitimate metadata-relocation operations.

After the existing read-only directory admission and mapping-role checks, but
before editing the copied leaf or staging insertion bytes, the private helper
checks every mounted group's primary descriptor that appears in the dirty map.
It requires a full-sized block, checks the descriptor slice and checksum using
the existing checksum policy, parses the existing descriptor format and compares
all three complete bitmap/inode-table addresses with mounted values. A changed
address returns `Unsupported("casefold insertion after staged metadata relocation")`;
bad checksums and truncated blocks return the corresponding checksum/corruption
errors. Placing the check after read-only admission retains the existing
fixed-metadata/root/leaf corruption refusals without trusting relocated roles
for an actual insertion write.

All five focused tests pass. Each refusal compares every dirty block, allocation
flag and device byte. Counter-only cases verify the inserted name/inode, unchanged
descriptor bytes and all other nonselected blocks, and unchanged allocation/device
state. Low and high address halves are checked; checksum-valid counter updates
are not treated as relocations. Non-address fields are not a full flags/allocation
audit. Descriptors are located using mounted geometry: staged superblock layout
or policy changes, allocation, journal/other ownership, unused mapping branches,
complete mapping shape and operation-wide serialization remain unqualified.
Findings 2 and 3 stay open; Stage 5 completion is **not yet quantified**.
Public casefold writes, deeper insertion, collisions, splitting/growth and index
dropping remain blocked.

The full local unit tier passed **820 tests**, zero failed or ignored, in
1,068 lines / 65,960 bytes within its unchanged budget. Multiset comparison
retains all 815 baseline occurrences and adds exactly these five tests.
Lint, provenance, whitespace and shared-agent checks pass. The locked release
CLI build/install passes. Fresh evidence
is under ignored `tmp/casefold-staged-descriptor-evidence` and
`tmp/logs/casefold-staged-descriptor-{red,green}.log`. Exact-commit independent
focused Linux and full CI for this new source are pending after push.

**Next bounded task:** inspect this fix's exact-commit focused insertion log and
full CI results. Resolve failures before broadening support. Then reproduce
staged superblock layout/policy changes that invalidate mounted metadata roles
or comparison/routing policy, and define a bounded refusal while preserving
valid counter-only superblock updates. Keep allocation, journal/other ownership,
complete mapping shape and serialization open. Do not enable writes or deeper
index insertion yet.

## Fixed-metadata refusal for mapping-node reads — 2026-10-14

Follow-up: exact-commit focused Linux and full CI at `986808c` now passed; its
retained insertion log was inspected. The staged-descriptor checkpoint above
supersedes this section's pending status and next task.

At source `8f94cf83dedc783c67bc58b853cafb2924ab5630`, a fresh local baseline
passed **810 unit tests**. Both preceding full CI runs (`3be872e`, `8f94cf8`)
completed successfully, including their VM suites and aggregate gates. The
latest focused Linux workflow also passed; its downloaded insertion log
confirms one executed test, zero failed or ignored, and independent checking
across the four existing 1 KiB/4 KiB, strict/non-strict profiles.

Three new negative planner tests reproduced `Ok(())` for checksummed extent
nodes placed in primary superblock block **0**, bitmap block **2** and
inode-table block **5**. The same ordinary directory-data mappings and valid
checksums are retained. Additional cases cover descriptor block 1, the inode
bitmap and the final inode-table block, with disk-only and identical-staged-node
variants. All five focused tests now pass, including the positive boundary
test described below. A fifth test isolates a metadata node reachable only
through a changed staged extent index, while the disk view uses ordinary
storage and both children return identical data mappings. This directly
exercises the staged-view guard. Every refusal compares the complete dirty
map, allocation flags and device bytes. These are synthetic malformed planner fixtures, not
Linux-clean volumes or permission to write metadata.

The fixture directly changes its backing device after mounting, so it discards
clean cached reads before testing and verifies the relocated extent checksum.
The first fixture attempt exposed stale cached superblock/root bytes; correcting
that fixture yielded the acceptance reds above. It does not disable checksums
or alter production cache behavior. A positive test places the map node exactly
after the inode table and moves directory data clear of it. Both disk-only and
staged cases insert successfully, retain the stored name/inode, change only the
selected leaf and preserve device/allocation state.

`CasefoldMapDevice` now checks every block covered by a nonempty map read
against the existing mounted fixed-metadata classifier before consulting the
underlying disk or staged device. This covers known group-head metadata and all
descriptors' bitmap/inode-table ranges without changing layout calculations.
Filesystem bounds remain the first check; successful safe reads still populate
the shared visited-node set for the later directory-data overlap refusal.

This closes the demonstrated fixed-location gap for visited mapping nodes.
Transaction-current descriptor/allocation state, journal and other ownership,
unused/shadowed mapping branches, complete mapping shape and whole-operation
serialization remain unqualified. Findings 2 and 3 stay open; Stage 5 completion
is **not yet quantified**. Public casefold writes, deeper index insertion,
collisions, splitting/growth and index dropping remain blocked.

The final full unit tier passed **815 tests**, zero failed or ignored, in
1,063 lines / 65,558 bytes within its unchanged budget. Multiset comparison
retains every one of the 810 baseline test occurrences and adds exactly these
five tests. Lint, provenance, whitespace and shared-agent checks pass. The
locked release CLI build/install passed for the production fix; the later
staged-only enhancement changed test code only. Fresh evidence
is under ignored `tmp/casefold-map-metadata-evidence` and
`tmp/logs/casefold-map-metadata-{red,green}.log`. Independent focused Linux and
full CI for this new source are pending after push.

**Next bounded task:** inspect this new fix's exact-commit focused insertion
log and full CI outcomes. Resolve failures before broadening support. Then
reproduce a staged descriptor relocation that makes otherwise ordinary
directory/map storage metadata in the current transaction, and define a safe
refusal contract without rejecting valid counter-only descriptor updates.
Keep allocation, journal/other ownership and complete map shape explicitly open.
Do not enable writes or intermediate-level insertion yet.

## Visited mapping-block/data overlap refusal — 2026-10-14

Follow-up: exact-commit focused Linux and full CI at `8f94cf8` have now passed;
the retained insertion log was inspected, as recorded above. The new
mapping-metadata checkpoint supersedes this section's pending status/next task.

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
| **5 — Implement and qualify directory-index writes** | Use frozen Unicode keys for hashing, insertion, collisions, leaf splits, and growth. Preserve Linux-visible routing. | **Hashing foundation, sampled read routing, and root-only in-place insertion implemented behind the volume write gate.** The new insertion primitive validates all referenced entries before changing one leaf. The preceding insertion/mapping source passed independent Linux and full CI. Preceding admission checkpoints through `1a6e8d1` passed exact-commit focused Linux and full CI. The staged superblock contract and allocation-review checkpoints passed full CI; focused Linux insertion passed at `b7b0325`. The current-allocation guard at `96b3b72` passed 839 local units, lint, the CLI build, focused Linux insertion and full CI. The scoped journal guard at `f9a0df0` refuses all three reproduced journal aliases and passed 852 local units, lint, the CLI build and exact-commit focused Linux/full CI. The private bounded complete extent-ownership collector at `7bdfe0c` passes 872 local units and exact-commit Linux FIEMAP/insertion/full CI. Its implementation/resource review found no blocker within the standalone scope and records aggregate-budget, capacity and depth-matrix requirements. The private current inode inventory at `4309ed1` has 15 local controls and passed exact-commit Linux allocation/insertion/FIEMAP/full CI; its implementation review is complete. The private current inode-role reader at `ef1ca24` has 17 controls and passed exact-commit Linux role/inventory/insertion/FIEMAP/full CI; its implementation review is complete. The private fixed-metadata collector at `95cb218` adds 17 controls and passed exact-commit VM location/insertion/inventory/role/FIEMAP/full CI; its implementation/evidence review found no blocker within the standalone scope. The private bounded current block-allocation interval verifier is implemented with 20 focused controls, padding/current-view checks and shared resource caps. Initial independent CI hit an unsupported block-zero oracle query; repair `72b6911` passed exact-commit focused VM/full CI. Its complete standalone allocation review found no blocker within the stated scope. A private complete current-view ownership builder is implemented with shared accounting and 18 focused controls; 961 local units pass, retaining all 943 baseline occurrences. Its six-profile whole-result Linux oracle passed exact-commit focused VM evidence; all 222 allocated records and 215 claims were audited with no blocker within the standalone stable-view contract. Source and review full CI now pass. The concrete operation serialization/mutation/invalidation audit is complete. A disconnected private gate is now implemented with 15 native exclusion/freshness/failure controls; 976 full local units and lint pass, retaining all 961 baseline occurrences. Exact-commit full/focused CI and standalone primitive review are complete; all 15 controls executed in four native/guest contexts. The whole guest suite still has one pre-existing ignored slow fuzz test, an unresolved broader qualification gate. Mount-bound private path/inode chmod adapters and sticky mount-failure bridging now pass exact-commit full/focused CI and standalone family/evidence review, including all four images and reports. Public chmod dispatch now uses the guard with new Rust/C controls; its exact-commit public-entry VM evidence and standalone migration review are next. Other families and lifecycle remain unguarded. Stage progress remains not yet quantified. Admission remains disconnected. The three other-inode ownership aliases were rerun and remain unresolved. Exclusive ownership and complete mapping shape remain open. Intermediate-level insertion, collisions, splitting/growth, and complete write qualification remain required; index dropping stays refused. |
| **6 — Prove recovery and enable qualified writes** | Test interrupted operations, journal recovery, errors, and refusals; enable only the combinations that pass. | Not yet reached. The ordinary-volume recovery recount regression passed independent Linux checking and readback; its full-CI output-budget repair and subsequent prerequisite checkpoints passed. Casefold operation/recovery qualification remains incomplete; keep repair and unsupported combinations blocked. |
| **7 — Integrate and qualify Windows** | Audit the Windows adapter's name conversion, caching, create/rename semantics, serialization, device ownership, flushing, and eject. Test alternating Windows/Linux use. | Not yet reached. Pin and audit an actual `ext4-win-driver` revision; later qualify specific SteamOS versions/devices. |

## What comes next

1. Review the public chmod migration and its actual exact-commit full/focused CI artifacts, including all eight new Rust/C Linux images, sixteen debugfs reports, eight Linux reports and the retained private oracle. Check generic conversion/callback ordering, context reuse, failure retirement, C error/panic handling and unchanged caller lifetime/exclusion obligations; resolve any failure before another family. No ownership readiness/cache, builder or buffer seal is included. Other operations/readers/maintenance/raw handles and unsafe C caller aliasing remain externally serialized. Keep every casefold write gate and the ignored deep fuzz qualification, insertion aliases, dependencies/overlays/seals/publication and map/collision/growth work open.
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

**Next bounded task:** use the latest 2026-10-16 public chmod checkpoint above; review its exact-commit full/focused CI and all new public-entry images/reports before another operation family.
The journal guard's exact-commit focused Linux and full CI passed. The bounded
transaction-current ownership design review and first private complete collector
are recorded above. The collector's exact-commit insertion/FIEMAP and full CI
passed, and its implementation/resource review is complete. The private bounded
allocated-inode inventory passed focused Linux/full CI and its implementation
review above. The private role reader passed exact-commit Linux/full CI and its
implementation review. The private fixed-metadata interval component is now
implemented and passed focused VM/Linux/full CI and review. The bounded current
allocation interval verifier and block-zero oracle repair passed exact-commit
VM/full CI, and the standalone implementation review is complete. Implement the
private complete current-view ownership builder under its aggregate-budget and
whole-result evidence contract. Keep it disconnected from
insertion admission and retain all baseline occurrences/unchanged-state checks.
The three other-inode aliases and disjoint control remain in the saved patch,
unresolved. Complete directory mapping shape, snapshot/overlay integration,
mutation invalidation and operation-wide serialization remain open. Do not use
a mounted-only owner cache or rescan every file per insertion.
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

Some saved Linux images and local logs live under the checkout's ignored `tmp/` directory. They are useful when present but are not guaranteed in a fresh cloud environment; use committed tests and CI artifacts to reproduce or retrieve evidence. The previous index-routing/refusal checkpoint passed full CI and fresh Stage 0 evidence; the subsequent root-only insertion passed focused Linux evidence at `a7db076`, with full CI also passed. The early mapping fix had successful focused evidence but hit the measured unit output budget in full CI. The repairs and later full CI at `3be872e` and `8f94cf8` passed; the mapping-metadata checkpoint at `986808c` also passed. The staged-descriptor checkpoint at `1a6e8d1` also passed. The staged-superblock checkpoint at `b7b0325` passed focused Linux insertion and full CI; the allocation review at `065f9a8` passed full CI. The current-allocation guard at `96b3b72` passed focused Linux and full CI; its insertion log was inspected. The ownership review retains three unresolved other-inode aliases in a self-contained patch. The scoped journal guard at `f9a0df0` passed exact-commit focused Linux/full CI; its actual insertion log was inspected. The owner-design review reran the three other-inode failures. The subsequent private bounded complete extent-ownership collector at `7bdfe0c` passed exact-commit Linux FIEMAP/insertion/full CI. Its implementation/resource review records the independent depth-zero/one matrix and aggregate-budget/capacity requirements. The private inode inventory at `4309ed1` adds 15 local controls without admission integration and passed exact-commit focused Linux allocation/insertion/FIEMAP/full CI; its implementation review is complete. The private role reader at `ef1ca24` has 17 focused controls and passed exact-commit Linux role/inventory/insertion/FIEMAP/full CI; its implementation review is complete within the narrow profile. The private fixed-metadata collector at `95cb218` adds 17 controls and passed exact-commit focused VM location and existing Linux/full CI; its implementation review and documentation review full CI are complete. The private bounded current allocation interval verifier adds 20 focused controls. Initial CI failed on an unsupported block-zero oracle query; repair `72b6911` passed exact-commit focused VM/full CI and its standalone implementation review is complete. The private complete current-view owner builder now joins those prerequisites under shared accounting, with 18 controls and 961 passing local units. Its six-profile whole-result Linux oracle passed exact-commit focused VM evidence; all six complete reports/images were audited and the standalone constructor review found no blocker within its stable-view contract. Source and review full CI passed. The concrete serialization/mutation/invalidation audit is complete, and the disconnected private operation gate now has 15 native controls. Its exact-commit full/focused CI and standalone review are complete; the private chmod/context/health bridge now passes exact-commit full/focused CI and standalone implementation/evidence review, including all four Linux images and reports. Public chmod dispatch is now guarded, with exact-commit public-entry Linux evidence and migration review pending; other operation/lifecycle/admission integration remains separate. The broader guest suite retains one pre-existing ignored slow fuzz case, so exhaustive write qualification remains gated. The insertion-level aliases and public write gates remain unchanged. No claim is made here that a separate coverage workflow ran at the latest commit.

This note is committed as `docs/extprogress.md` on `casefold-rw`. A new Codex session should open that branch, read this file, and verify the actual checkout and CI before continuing. Verify the selected commit against its own CI; earlier successful runs are evidence for those earlier checkpoints. Update this handoff as implementation checkpoints are completed.
