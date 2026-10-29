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

## Fresh environment handoff — 2026-10-28

Start with `AGENTS.md` and `docs/CLOUD-STARTUP.md`: request/verify the complete
known access bundle upfront, then perform the required second-message recheck
and wait for implementation confirmation. The setup script now generates a
native runner using tracked `scripts/cloud-casefold-run.py`; no ignored helper,
PRoot/ptrace grant, host oracle or sibling rewrite is needed. The core/chore/
Rust pins and archive checks remain. Source `/workspace/.cloud-setup/env.sh`
and use that runner for Cargo/chore checks; its temporary manifest selection
is restored and the lock is checked. The tracked runner passes the full native
unit tier with every 1,639 occurrence / 662 controls retained, plus full lint.
Offline dependency selection and manifest/lock restoration are verified on both
successful and failed child commands. Setup-shell/Python syntax checks pass;
the complete download/bootstrap has not been rerun in an empty machine here.
Do not reuse an old generated PRoot wrapper on a fresh setup.

The project checkpoint below remains authoritative. The subsequent saved-byte
audit now confirms corrected growth source `09cf8dc` passed focused
**40744071010** (growth **123013076874**, allocation-free **123013076829**)
and full **40744071049**. Setup-only tip `aaf5e2a` also passed full
**40748257814**. The current checkpoint records the bounded linear-growth
evidence and limitations; setup-only changes do not themselves qualify growth.
Fresh bootstrap network access and permission persistence
must be verified in the new chat; the repository cannot guarantee the platform
will honor/retain Full access or session approvals. All changes are intended
for `casefold-rw`, never main or siblings.

## Current checkpoint

- **The complete inline-source guest suite retains every test; its measured byte-only wrapper failure is repaired locally.**
  Exact source **`6c71cbe`**, full run **40785791674**, guest job
  **123159536037**, passes **2,497 tests**, retaining every previous occurrence
  and adding exactly eight native controls plus the inline Linux test. All
  **670 controls occur once**; the same ignored slow fuzz case remains a gate.
  The verified complete log is **4,095 lines / 248,844 bytes**, so the old
  **248,000-byte** ceiling returns **65** despite the successful underlying
  suite. Byte-identical replay reproduces that failure and passes at
  **251,000 bytes**, with the unchanged 4,120-line ceiling and local floor
  1,060. Only the measured guest byte limit changes. The actual failed CI
  invocation never reaches its floor; fresh repair-source CI remains required.
  Native whole job **123159535998** is still running at this observation;
  its complete retention, wrapper and trailing checks are not yet credited.
  **Next:** inspect that actual native result, repair only demonstrated failures,
  then close repair-source full retention and all five control contexts.
  The fourteen focused jobs and all three link image audits remain passed at
  `6c71cbe`; no full green gate or broader growth qualification is inferred.
  Stages 4–5 remain not quantified, casefold writes blocked and platform gates open.

- **Exact inline-conversion Linux emission and all three link saved-byte matrices pass; whole-suite retention is still pending.**
  Source **`6c71cbe`** passes all fourteen focused jobs in **40785791695**.
  Its new inline archive is API-source/digest/all-CRC/24-file-hash verified:
  **sixteen layouts / 48 links / sixteen conversions / 160 images**, **608
  debugfs / 112 judged non-repairing e2fsck reports**, **540 cold Linux rows /
  twelve C snapshots** are audited against raw bytes. Real parents start inline;
  fit stays inline, conversion debits exactly one block, and postfit allocates
  nothing more. Complete table/bitmap/metadata/EA/checksum and dirty/closed/cold
  checks pass. Both repeated linear archives also pass their full **sixteen
  layout / 112 link / 352 image** audits. No new source defect or cap failure
  was observed in these focused runs; no production/test/workflow change is made.
  Exact full **40785791674** currently retains all **1,647 unit occurrences /
  670 controls** in standalone/ARM unit contexts, **527 ARM image / five wasm /
  159 standalone CLI** successes; its two whole-suite jobs are still running.
  **Next:** close this source's actual full native/guest retention (expected
  **2,497 whole**, all 670 controls once in five contexts, actual floors and
  trailing checks), repairing only demonstrated failures. Full qualification
  is not inferred from the focused matrix. The existing ignored slow fuzz case,
  exhaustion error classification, broader growth/counter-wrap/scoped review
  and all ownership/recovery/write/platform gates remain. Stages 4–5 stay not
  quantified; casefold writes remain blocked.


- **Inline-parent conversion native controls and the independent Linux test/job are implemented; exact Linux execution is pending.**
  Eight new native controls pass: 40 same-table Rust/C API fit/convert/postfit
  layouts, 52 measured planning-read faults, 176 write faults, 56 flush faults,
  56 flush unwinds, four C readback errors and four publication unwinds. The
  synthetic journal exercises the transaction engine, not Linux-clean images.
  The separate exact host test plans **sixteen Linux-created layouts / 48 driver
  links / 160 input/dirty/closed/cold images**, with exact block/SB/group/bitmap,
  inode/EA/metadata/entry/type/time/version and cold Linux evidence. The new
  pinned-sibling `link-inline-conversion` job requires one actual test and retains
  24 source hashes; inline/helper trigger paths are included. Existing linear
  job bodies and production writers/shared helpers/pins/write gates are unchanged.
  Previous plan commit `f08db64` passes all eleven jobs/ci-ok in **40773436382**.
  **Next:** inspect this new source's focused inline and repeated linear jobs,
  fix only demonstrated failures, audit complete source/digest/CRC/raw saved-byte
  evidence, then close exact full retention (expected 1,647 unit / 2,497 whole
  successes, all 662 old controls plus eight new). Planned image counts are not
  results; inline conversion stays independently unqualified. The shared
  allocator's exhausted-search corruption classification is retained, not
  presented as Linux-compatible ENOSPC. Broader growth/counter-wrap/review and
  all ownership/recovery/fuzz/write/platform gates remain; stages 4–5 stay
  not quantified and casefold writes remain blocked.


- **Inline-parent hard-link conversion contract and independent qualification plan are prepared.**
  Previous checkpoint `43280ec` now passes all eleven jobs/ci-ok in exact full
  **40761780868**; seven verified tier archives retain **1,639 unit / 2,488
  whole** successes, **662 controls once in five contexts**, actual floors and
  trailing checks. Four existing exact native tests freshly pass through their
  wrappers/floor-one checks. The source review distinguishes head fit,
  continuation fit/expansion and actual one-block conversion; existing Linux
  inline coverage does not independently qualify hard-link-triggered conversion.
  A **planned**, not executed, sixteen-layout Rust/C × 1/4 KiB × direct/journal
  × head-only/populated-continuation matrix uses 256-byte checksummed inodes.
  Fit/convert/post-conversion steps plan **48 driver links / 160 snapshots**,
  with exact allocation, `system.data` removal, EA/peer/shared-table and cold
  Linux evidence. No new test, fixture, driver change or Linux dispatch occurs.
  **Next:** implement this bounded independent test/profile and pinned focused
  CI job, first adding/executing the native fit/conversion/same-table/error
  contract controls; fix only reproduced defects. Preserve all prior suites
  and exact source-bound evidence. Inline conversion remains unqualified;
  broader growth/counter-wrap/review and all write/recovery/platform gates remain.
  Stages 4–5 stay not quantified; casefold writes remain blocked.


- **Corrected public hard-link linear growth now has exact Linux saved-byte and full-suite retention evidence.**
  Corrected source `09cf8dc` passes all thirteen focused jobs in **40744071010**
  and all eleven full jobs/ci-ok in **40744071049**; setup-only `aaf5e2a`
  also passes full **40748257814**. The growth and repeated allocation-free
  archives are source/digest/CRC/thirteen-file-hash verified and audited across
  **sixteen layouts / 352 images / 112 links each**. Growth allocates exactly
  one data block on the first link in each layout, with matching primary-SB,
  group and one-bit bitmap debit, unchanged inode allocation/EA/peer bytes,
  valid applicable checksums, closed journals and cold Linux readback. Each
  archive retains **1,536 debugfs / 240 judged e2fsck** reports; growth retains
  1,932 cold metadata rows and 28 C snapshots, allocation-free 1,928 and 24.
  Both exact full runs retain every previous occurrence, **1,639 unit / 2,488
  whole successes**, all **662 controls once in five contexts**, actual floors
  and trailing checks. The existing ignored slow fuzz case remains a gate.
  This closes the corrected linear-growth execution/evidence checkpoint within
  its healthy ordinary-directory/caller-exclusion profile, not stages 4–5.
  **Next:** prepare bounded inline-directory conversion growth qualification,
  beginning with its current implementation/refusal contract and independent
  allocation/metadata evidence plan. Indexed/deeper mapping growth, counter wrap,
  scoped implementation review and broader ownership/recovery gates remain.
  Casefold writes remain blocked; Windows/card/SteamOS remain unqualified.

Earlier checkpoint entries below retain their original observations; the latest
closure above supersedes their pending corrected-linear-growth status.

- **The first linear-growth CI failure is a reproduced test expectation error; its EA-sector accounting is corrected, with rerun pending.**
  Source **`3fe3f92`**, focused job **`122955259496`** in **`40727685681`**,
  fails before the first driver link: the Linux-created one-block directory
  reports **four 512-byte sectors**, including its separate external-attribute
  block; the new test expected two. Artifact **`12833757215`** is source/
  advertised-digest/all-CRC/thirteen-source-hash verified. Its raw inode and
  Linux stat agree on one directory-data block plus one EA block. The exact
  parent-shape assertion reproduces red on those saved bytes, then passes
  after explicitly counting both blocks. Temporary replay code is restored.
  No driver change, assertion removal or growth-debit relaxation is made.
  Fresh native units retain **1,639 occurrences / 662 controls**, and full lint
  passes. All twelve other Stage 0 jobs, including the original allocation-free
  focused job, pass at `3fe3f92`; their new saved-byte audit remains separate. Both full
  suites fail only this new library test (**1,343 passed / one failed**), so
  remaining integration and native trailing checks are unrun, not credited.
  **Next:** inspect corrected-source focused growth/repeated allocation-free
  CI, audit all saved shapes/allocator/metadata/checksums and complete exact
  full-suite retention (expected 2,488 whole successes). This correction proves
  the first input-shape check only; allocating growth is still unqualified.
  Inline/index/deeper growth, counter wrap and scoped review remain later work.
  Casefold writes stay blocked, Windows/card/SteamOS remain unqualified, and
  stages 4–5 remain not yet quantified.

- **A focused independent full-linear-parent hard-link growth test is added; Linux execution and saved-byte qualification are pending.**
  The new `link-linear-growth` job plans **sixteen Rust/C layouts / sixteen
  first-block allocations / 112 links / 448 refusal observations / 352 images**.
  Linux fills the original one-block parent; raw records prove no room for the
  first 255-byte alias. The driver must add exactly one block, debit primary-SB
  and group free space by one, preserve every inode bitmap and unrelated block
  bit, source identity/data/EA and protected peer metadata, then pass cold
  Linux/debugfs/judged e2fsck checks. All five link forms cycle through real growth across layouts;
  this is not their full Cartesian product. Planned counts are **not results**.
  All **1,639 native occurrences / 662 controls** remain, with full lint,
  compile, exact one-test selection and guest Python syntax passed. Writers,
  shared helpers, manifest/lock and write gates are unchanged. Preceding cap
  commit **`2137e23`** now passes all eleven jobs/ci-ok in **`40649608984`**;
  source/digest/all-CRC-verified archives retain 2,487 whole successes, actual
  floors/wrappers and every trailing check. **Next:** inspect the exact new
  growth and repeat allocation-free jobs, fix only demonstrated failures and
  audit their source-bound reports/images, then close new-source full retention
  (expected **2,488** whole successes). Inline/index/deeper mapping growth,
  counter wrap, scoped review and broader ownership/recovery/fuzz gates remain.
  Casefold writes and Windows/card/SteamOS use stay unqualified. Stages 4–5
  remain not yet quantified; no new growth compatibility is claimed yet.

- **The new public-link host test is now retained in complete green full CI; a measured native log-cap failure is repaired.**
  Exact docs tip **`741381c`** passes all eleven jobs and ci-ok in
  [full run 40640607199](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40640607199).
  Its source/test/workflow/manifest/lock bytes are identical to emission source
  **`e6db258`**. Seven source/digest/all-CRC-verified archives retain every
  **1,639 unit / 2,487 whole** success and **662 controls once in five contexts**.
  All actual wrappers/floors and trailing **twenty native shell / 159 native
  installed-CLI / 159 standalone CLI** checks pass, with 527 ARM images,
  225 oracle, 36 kernel and nine third-implementation successes. One unchanged
  ignored slow fuzz case remains a qualification gate. Earlier exact-source
  **`e6db258`** full run **`40637075795`** passes all 2,487 tests in both suites
  but fails only the native whole log cap (**3,916 lines / 238,064 bytes**, old
  byte limit 238,000); its later native floor/shell/CLI steps are unrun.
  The identical complete log reproduces exit 65 at that old cap and passes at
  **3,950 / 239,000**, retaining the floor and all output. Only that byte limit
  changes; the later green run's 237,948-byte log confirms ordinary variation.
  **Next:** check exact budget-repair CI, then independently qualify real
  full-linear-parent hard-link growth with exact block/SB debits, preserved
  inode/peer/EA bytes and cold Linux/e2fsck evidence. Inline/index/legacy/
  extent/deep/uninit growth, counter wrap and scoped review follow. No production
  code changes or new driver-test execution here; stages 4–5 stay not quantified
  and casefold writes/Windows/card/SteamOS remain gated.

- **Previous checkpoint: allocation-free public Rust/C hard links passed focused Linux execution and the complete saved-byte audit.**
  Exact source **`e6db258`** passes all twelve jobs in
  [Stage 0 run 40637075787](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40637075787).
  Focused job **`122646396891`** passes all **sixteen layouts / 112 links /
  448 refusal observations / 352
  images / 1,536 debugfs reports / 240 judged e2fsck checks / 128 guest mounts**.
  Source-bound artifact **`12793499218`** has verified advertised SHA-256 and
  all **4,275 ZIP CRCs**. The saved-byte audit matches real Linux metadata,
  aliases/content/EA/free counts and all 24 requested C target snapshots; checks
  directory versions, maps, bitmap/inode/EA/journal checksums, and preserves
  unrelated allocated inode bytes throughout dirty/closed/Linux-copy states.
  The actual one-test floor/wrapper passes at **2,141 lines / 287,619 bytes**;
  no new cap repair is needed. All 1,639 native occurrences/662 controls remain.
  Implementation **`afd24b8`** now passes all eleven full-CI jobs and ci-ok in
  [run 40633679897](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40633679897),
  with 2,486 whole successes in both suites, all 662 controls in five contexts
  and actual floors/trailing checks retained. These protected writer bytes are
  unchanged in `e6db258`; its newly added full-suite host test still needs its
  own **2,487-whole** CI/retention result. **Next bounded task:** verify that
  exact full CI, then extend independent qualification to real parent growth/
  conversion/index/mapping shapes and counter-wrap boundaries before the scoped
  implementation/evidence review. This healthy ordinary, non-growing matrix
  does not qualify casefold writes, growth, corruption admission, crash recovery
  or Windows/card/SteamOS. Those gates remain; stages 4–5 are not quantified.

- **Previous checkpoint: allocation-free public Rust/C hard-link emission checks were added pending Linux execution.**
  Production bytes remain unchanged from **`afd24b8`**. The new focused
  `link-emission` job pins the existing core/harness/Rust versions, requires the
  exact new test and floor one, and retains source hashes, complete logs and
  driver-dirty/driver-closed/Linux-copy images. The planned matrix is **sixteen
  layouts / 112 links / 448 refusal observations / 352 images / 1,536 debugfs
  reports / 240 judged e2fsck calls / 128 guest mounts**; these are test-plan
  counts, **not credited results**. All five existing source kinds, all five
  Rust/C forms, raw/255-byte names, exact metadata/directory-version changes,
  optional C existing-target attrs and complete allocation/EA/peer preservation
  are asserted. Cold Linux reads use copies, preserving actual driver bytes
  across phases even when symlink reads update observer atime. Full native
  validation retains all **1,639 occurrences / 662 controls**, zero failed/ignored;
  full lint and guest Python syntax pass. Existing implementation full CI
  `40633679897` is still running; all eleven Stage 0 jobs and the compatibility
  run already pass at `afd24b8`. **Next:** inspect/fix actual focused Linux
  outcomes, audit all saved bytes/reports and source/digest/CRC bindings, and
  close exact-source full-CI retention when complete. No emitted-link or growth
  qualification is yet claimed. Growth, counter wrap, scoped source/evidence
  review and all broader write gates remain open; stages 4–5 are not quantified.

- **Guarded public hard-link migration and measured metadata repair pass native checks.**
  All five Rust/C forms now use the existing shared operation context through
  lookup, both arbitrary conversions, completed `LinkPlan`, marker, entire
  commit/publication and optional C owned target readback. Source ctime and
  destination-parent mtime/ctime/version now follow the measured Linux policy;
  source identity/data/EA/owners and explicit path/name/Into/FFI priorities remain.
  Both saved patches are **historical/promoted: do not reapply them**. Fresh reds
  reproduced **32 passes / 91 failures** before repair; all **123 original
  controls plus thirteen real boundary/Runtime diagnostics now pass**. Six
  sweeps remeasure **216 actual I/O injections** (formerly 200), zero retirement
  violations; all four real post-commit publication profiles remain. The full
  native unit suite passes **1,639 occurrences / 662 controls**, retaining every
  prior 1,503 occurrence as a multiset, zero failed/ignored. Full lint passes.
  A promoted diagnostic label tripped the unchanged no-skip scanner; its callback
  counter was renamed without removing a check. A new reverse-Runtime diagnostic
  was corrected to count every actual clock call rather than assuming one in
  older families. The actual all-green unit output exceeds the old wrapper cap;
  only its measured caps are repaired below. No full-suite budget is guessed.
  **Next bounded task:** verify exact-source full CI and complete occurrence/
  five-context/floor/trailing-check retention, repairing only measured failures;
  then add focused independent Rust/C emitted-link/growth qualification and audit
  its saved images/reports before the scoped implementation/evidence review.
  Fresh full/focused Linux results for these changed driver bytes are **pending**;
  earlier reference images do not qualify driver-emitted bytes. Casefold writes
  stay blocked. Stages 4–5 remain **not yet quantified**; broader ownership,
  lifecycle/recovery, ignored-fuzz and Windows/card/SteamOS gates remain open.

- **Measured native hard-link policy reds and both patch compositions are executed.**
  The new [policy patch](CASEFOLD-LINK-POLICY-RED.patch) executes **39 controls:
  12 pass / 27 current semantic failures / zero ignored**. Source ctime,
  destination-parent mtime/ctime/version and optional C target attrs are checked
  across all five Rust/C forms, five existing source kinds and six native
  ext2/3/4 × 1/4 KiB profiles. All **1,554 successful links / 516 refusals** run;
  the 1,290 semantic link calls all reach their measured timestamp/counter
  expectations. Explicit API versus syscall error/path/Into contracts remain.
  Policy-then-safety and safety-then-policy produce **identical source bytes**;
  both complete **123-control** runs repeat **32 passes / 91 failures**, with
  every original **84 safety outcome / 200 I/O injection / four real publication
  unwind** retained. Original positive/readback profiles remain; no fake helper
  or unreached Runtime hook counts as a red. Both patches stay **dormant**.
  Active source/test/workflow/manifest/lock bytes are restored; fresh native
  validation retains every **1,503 occurrence / 526 controls once**, zero
  failed/ignored, with active full lint and dormant standalone/composed clippy
  passed. The existing Linux eight-layout/80-image reference remains unchanged.
  Exact reference source **`9fd59cf`** now passes all eleven full-CI jobs and
  ci-ok in **`40615435814`**. Five source/digest/all-CRC-verified archives retain
  **2,350 successes** in each whole suite and all **526 controls in five contexts**;
  actual wrappers/1,060 floors, twenty native shell/159 native CLI checks,
  159 standalone CLI checks and 527 ARM image successes pass. One unchanged
  ignored whole/image slow fuzz case remains a qualification gate. Docs tip
  **`8f0006a`** also passes exact full CI **`40616704908`**. No CI budget changes.
  **Next bounded task:** implement guarded public hard-link entry/owned LinkPlan,
  optional-SB/complete-effect/publication and optional C owned target-readback
  handling, plus the measured metadata repairs, preserving all explicit API
  priorities. Promote repaired controls and add real helper-boundary/runtime/
  fault diagnostics; independent Rust/C emission/growth qualification follows.
  No link guard or semantic repair is implemented here. Casefold writes stay
  blocked; stages 4–5 remain not yet quantified and broader gates remain open.

- **Independent Linux hard-link policy collection and saved-byte audit pass.**
  Corrected source **`9fd59cf`** passes exact focused job **`122576714109`** in
  [run 40615435825](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40615435825).
  Source/digest/all-CRC-verified artifact **`12785027612`** retains all **eight
  layouts / eighty images / 56 links / 560 consistent syscall errors / 960
  debugfs reports**. Actual log/floor one pass with zero failed/ignored. Linux
  alone creates the bytes; driver closed-image reads leave them unchanged.
  The tested link/linkat(0) profiles preserve regular/fast/slow/dangling/FIFO source
  identity and data/EA/owners/generation, raw/255-byte spelling and parent link
  counts; source nlink/ctime and destination-parent mtime/ctime update. Every
  successful alias increments the destination directory version once. Exact
  group/bitmap/itable-unused accounting proves no inode/block allocation for
  the asserted non-growing parents; old parent/peers and cold namespace remain.
  The first run's overly strict directory-version expectation is resolved using
  its retained Linux bytes and the public format, without changing the driver.
  Fresh native validation and exact CI units retain **all 1,503 occurrences /
  526 controls once**, zero failed/ignored, with full lint and **159 CI CLI
  checks** passed. Preceding `8791f06` exact full CI `40610284789` now succeeds.
  Corrected-source full CI **`40615435814`** is still running; whole/architecture/
  five-context/floor/trailing retention is pending, not credited from that prior
  source. The dormant 84-control safety patch remains unchanged and inactive.
  **Next bounded task:** inspect corrected-source full CI, repair actual failures
  and verify expected **2,350 whole-suite passes / 526 controls in five contexts**
  plus actual floors/trailing checks. Then execute measured native link policy
  reds and compose them with the dormant safety patch before guarded migration.
  No link guard, semantic repair, emitted-image or casefold write qualification
  is claimed. Casefold writes stay blocked; stages 4–5 remain not yet quantified.

- **The first link reference run exposed a test expectation error; its saved bytes establish the correction.**
  Exact source `0183060`, focused run `40614604473`, job `122574039759`,
  reaches the first successful Linux link and fails the reference test's
  byte-preservation assertion. Verified artifact `12784752592` retains the
  input/names/first images and complete failure log. The destination directory
  version changes **6→7 at raw offset 0x24**; the regular source version stays
  5, source ctime/nlink update, old parent/peer remain unchanged. The published
  ext4 inode format identifies this counter separately from `i_generation`.
  The test now explicitly requires one destination-directory version increment
  per link, preserving source identity and all other field checks. Production
  code is unchanged. Repeat exact Linux CI and audit the full matrix; this
  partial archive does not qualify all eight layouts or close reference work.

- **Reference-only Linux hard-link policy collection is added; actual VM evidence is pending.**
  The new `link_policy::linux_records_link_policy` test uses eight 1/4 KiB,
  128/256-byte inode and direct/journal layouts. Ten phases per layout retain
  **80 planned images**, seven successful links per layout (**56 aliases**),
  **35 error cases through both link/linkat(0)** (**560 observations**) and
  **960 debugfs inode reports**, followed by a separate cold namespace check.
  Linux creates every reference byte; the driver only reads closed images and
  proves those reads preserve image bytes. Source identity/data/owners/EA,
  source ctime/nlink, destination-parent mtime/ctime, old parent/peers, exact
  group/bitmap accounting and final-symlink/raw/255-byte behavior are covered.
  A floor-one pinned Stage 0 job saves all images/reports, the complete log,
  exact source commit and nine source hashes. New Rust code compiles; Python
  is syntax checked without host execution. Fresh native validation retains
  **1,503 successes / 526 controls once**, zero failed/ignored, and full lint
  passes. The dormant 84-control safety patch is unchanged and inactive.
  **Next:** inspect this exact source's focused CI, repair actual failures and
  audit the saved reference bytes/reports before measured native policy reds,
  patch composition or guard migration. Counts above are planned until that
  execution/audit completes; no fresh Linux result or semantic repair is claimed.
  Casefold writes stay blocked; stages 4–5 remain not yet quantified.

- **Dormant public hard-link safety/positive preparation is executed and reproducible.**
  At fetched **`90acfc1`**, whose exact full CI `40575650714` now passes, the
  saved [native link patch](CASEFOLD-LINK-RED.patch) executes **84 controls**:
  **20 pass / 64 current public safety failures / zero ignored**, identically
  in two complete runs after actual reverse/reapply to byte-identical source.
  All **200 measured write/flush/error/unwind injections**, **four actual
  post-commit publication unwinds** and **54 primary positive profiles** run.
  Existing-inode identity, payload/EA/owners/generation, raw/255-byte names,
  parent counts/no inode allocation, same-table splices, inline/conversion,
  ordinary index splitting, legacy indirect/fragmented depth-one extent growth
  and two-group uninit accounting pass. Both Rust Into hooks, eight device/
  reentry phases, all ten qualified reverse families and optional C target
  readback expose the expected current exclusion/retirement gaps. Ordinary
  clean refusals currently dirty the marker; prior-dirty controls pass.
  C readback error controls prove exactly one committed entry/count increment
  with output sentinels retained. No fake private helper or unreached link
  Runtime clock/generation hook is counted as a red.
  The patch remains **dormant**: active source/tests/manifest/lock are restored
  byte-for-byte, every **1,503 native occurrence / 526 controls** is retained
  with zero failed/ignored, and active full lint plus dormant clippy pass.
  This adds evidence, not a link guard, measured time repair, new Linux result
  or write activation. The detailed checkpoint below records scope, hashes,
  replay commands and retained local logs. **Next bounded task:** add/execute
  independent **reference-only Linux link/linkat-zero-flags policy** collection
  in the pinned CI VM, then audit its saved bytes before native semantic reds,
  patch composition and guard/owned-plan/optional-SB/readback migration.
  Stages 4–5 remain **not yet quantified**. Casefold writes stay blocked;
  broader ownership/admission, growth, recovery and real-platform gates stay open.

- **The three upstream backports and measured kernel-budget repair are verified; scoped review is closed.**
  This supersedes the pending CI/audit status below. Budget repair **`3e3d922`**
  passes all eleven jobs and ci-ok in exact run
  [40550417452](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40550417452).
  The prior cancelled unlink-reference job is resolved: retained Stage 0
  [40502467008](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40502467008)
  attempt two passes all ten jobs. The latest pre-review docs tip **`ceb3232`**
  also passes full CI `40554014520`. Driver/test/workflow/dependency bytes remain
  identical to independently checked backport implementation **`fce2afb`**;
  only budget configuration and documentation have changed since that source.
  Five complete source/digest/all-CRC-verified tier archives retain all **1,503
  unit occurrences** and **526 controls once in all five contexts**. Both whole
  suites retain every earlier **2,337** occurrence plus twelve additions:
  **2,349 passes**, zero failed, one unchanged ignored slow fuzz test. Actual
  whole wrappers and 1,060 floors pass, as do the repaired **36-pass kernel**
  wrapper/floor. Its actual log is **631 lines / 55,214 bytes** within 700/62,000;
  native whole is **3,776 / 225,190**, guest whole **3,943 / 234,396**, within
  unchanged caps. All **twenty native shell and 159 native installed-CLI checks
  execute and pass**; standalone CLI retains 159, and ARM images retain all
  527 successes and the same existing ignored case.
  All eleven retained Stage 0 archives verify source/digest/every CRC. Completed
  metadata/setter/removal/policy/unlink/rmdir/symlink/mkdir-reference audits were
  preserved through the workspace outage. The final three audits now actually
  complete: mkdir emission **16 layouts / 112 images / 48 creations**, Linux
  create reference **8 layouts / 48 images / 24 creations / 216 error records**,
  and regular-file create emission **16 layouts / 112 images / 48 creations**.
  Each emission checks 544 debugfs reports; all retained audits pass. The focused
  compatibility run `40502467167` and its fourteen verified images/reports remain
  applicable to the unchanged implementation. The interrupted local audit is
  now closed, not inferred from successful jobs alone.
  Local receipts/logs are under ignored `tmp/kernel-budget-repair/`, including
  `retention.json`, `image-audit-summary.json` and the three resumed receipts;
  `tmp/logs/backport-retention.log` and `backport-resumed-image-audits.log` pass
  their tiers. No new source, dependency, test, workflow, write-gate or budget
  change is made by this review; no fresh Linux dispatch is claimed.
  **Next bounded task:** actually execute/save/reverse/reapply/repeat a dormant
  native public hard-link safety/positive patch under the existing design;
  restore active source and retain **1,503 occurrences / 526 controls** with
  lint before reference-only Linux link policy work, semantic reds or migration.
  Preserve the now-qualified source trailing-slash directory requirement,
  counted-name/dual-Into priorities, optional-SB allocation-free plans and C
  target-readback contract. No link guard or executed link red is claimed yet.
  Stages 4–5 remain **not yet quantified**. Casefold writes remain blocked;
  ownership aliases/admission, complete mapping/index growth, seals/lifecycle,
  recovery/ignored fuzz and Windows/SteamOS/real-card qualification stay open.

- **Measured kernel log-budget repair passes local checks; fresh CI is next.**
  Full source run `40502466987` at `fce2afb` and docs-source run `40537693019`
  at `07e86b3` both fail only the actual kernel output wrapper: **36 successful
  kernel tests**, zero failed/ignored, **630 lines / 55,111 bytes**, exceeding
  620 / 55,000 and returning exit 65. Native whole-suite execution/floor and
  trailing shell/installed-CLI checks were **unrun**, not passing. Downloaded
  five source/digest/all-CRC-verified archives retain **1,503 unit occurrences**
  in unit/native/ARM contexts and all **526 controls** there and in the guest;
  the guest whole suite exactly retains every **2,337** previous occurrence
  plus twelve additions (**2,349 passes**, zero failed, one unchanged ignored
  slow fuzz case). ARM image names exactly retain **527 passes** and that same
  existing ignored case; standalone CLI retains **159** checks. Four of five
  control contexts are verified; native whole remains outstanding.
  Only the `test:kernel` caps in `chores.yml` change to **700 / 62,000**.
  Actual identical-log replay fails old limits with 65 and passes new with 0,
  preserving all 55,111 bytes; headroom is 70 lines / 6,889 bytes. Test selection,
  output, floor 33, other caps, driver/test/workflow/dependency bytes and write
  gates are unchanged. All **87 CI-profile controls**, the output-wrapper shell
  checks and full lint pass locally. This configuration repair does not claim
  a fresh Linux execution or successful actual wrapper/native whole suite.
  Stage 0 run `40502467008` has nine successful jobs and a cancelled unlink
  reference job, whose log reports cancellation rather than a test verdict.
  An authorized targeted rerun of job `122183292746` is accepted (HTTP 201);
  its new outcome and evidence remain pending. Session network permission
  restored actual API access after this runtime's sandbox could not reach its
  configured proxy; the granted network scope is this session.
  **Next:** inspect the budget-repair exact-commit CI, require the actual kernel
  wrapper/floor, native whole/floor, trailing twenty shell/159 CLI checks and
  ci-ok, and verify five-context retention and remaining saved Stage 0 evidence.
  Then resume dormant hard-link safety preparation against the current **1,503
  native occurrences / 526 controls**, incorporating the new source trailing-
  slash contract. No link guard or executed link red is introduced here.
  Casefold writes remain blocked; stages 4–5 remain not yet quantified;
  Windows/SteamOS/card and broader ownership/growth/recovery work stay open.
- **All three upstream backports pass focused independent Linux validation.**
  Tested/pushed implementation is **`fce2afb64232d1264f1504f18d2ae5dfa642b081`**,
  following truncate `3d98e5e` and raw-xattr `4428a10`. Exact focused run
  [40502467167](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40502467167)
  succeeds: all **three tests**, no failures/ignored, all **fourteen images and
  fourteen reference reports**. Downloaded artifact `12732749462` matches its
  advertised SHA-256 `2fecb640d0e785c6011248acaa833b9c63b1d6550450c4dd37626fe0c9ce835e`;
  every ZIP CRC, recorded source commit and all eight source/test hashes verify.
  Linux independently bisects all eight file-size boundaries; Rust and C
  accept the measured maximum and refuse its next byte without device changes.
  Both 1/4 KiB raw-name images retain inline/external payloads and both distinct
  raw attributes across ordinary-neighbor metadata writes. All four indexed-root
  path reports agree on directory-link targets, non-directory/missing/loop
  refusals and casefold-only aliases; all 84 Linux rows agree through Rust and C.
  Native validation retains **1,503 passes / all 1,494 preceding occurrences /
  all 526 controls**, with full lint passed. The downloaded exact-CI unit archive
  independently retains the same occurrences; its source/digest/all CRCs verify.
  The verified standalone CLI archive retains **159 successful checks**.
  Full source CI [40502466987](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40502466987)
  and retained Stage 0 [40502467008](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40502467008)
  are **still running at inspection**; completed jobs are successful. Native/
  guest whole-suite retention, architecture evidence, remaining family archives,
  actual wrapper/floor/trailing-check outcomes and ci-ok are not yet verified.
  **Next:** inspect those exact runs, repair actual failures, verify their saved
  evidence/retention, then resume dormant native hard-link safety preparation.
  Casefold writes remain blocked; stages 4–5 remain not yet quantified and real
  Windows/SteamOS/card qualification remains open. The new raw-name read API is
  Rust-side and C listing returns exact bytes; C get/set/remove name arguments
  retain their existing UTF-8 contract. No raw-name C mutation expansion is claimed.
- **All three approved upstream adaptations pass local validation (2026-10-27).**
  Truncate limits/EFBIG from `3322f7b`, raw xattr names/read/C-list behavior from
  `ea72663`, and trailing-slash path semantics from `77f8df2` are adapted without
  newer core/CLI/dependency imports. Actual pre-repair native evidence is one
  oversized-growth failure, four xattr parser/read failures, one final-slash
  identity failure and one ordinary-index-dot checksum-refusal failure. Their
  corresponding repaired controls pass. Literal indexed dots now read and
  verify their root directly for ordinary as well as casefold directories;
  casefold encoding/depth/inline refusals, checksums and prepared-name lookup
  remain. Final links without trailing slashes retain no-follow behavior.
  Full native validation passes **1,503 occurrences**, zero failed/ignored;
  multiset comparison retains every **1,494** baseline occurrence and all
  **526** context/family controls, with nine new controls. Full lint passes.
  The complete unit log is 1,760 lines / 133,531 bytes, within existing caps;
  no output limit or selection changed. All three independent Linux tests
  compile: eight mapping/feature/block-size truncate boundaries with Rust/C
  refusal-byte checks, 1/4 KiB raw-name inline/external retained-byte checks,
  and four indexed-root/casefold path profiles with Linux lstat/Rust/C outcomes.
  A focused pinned-harness workflow saves reports/images/source hashes and
  requires all three tests with no ignored cases. Its actual CI, saved-byte
  audit and exact-source full-CI retention are **pending**, not passed locally;
  this cloud has no local KVM. Verify those next and repair any actual failure
  before resuming dormant hard-link red preparation. Casefold writes stay
  blocked; stages 4–5 remain not yet quantified; Windows/SteamOS/card and
  broader ownership/index-growth/lifecycle/recovery qualification stay open.
  Local direct builds select the immutable core through a temporary relative
  sibling path and restore the manifest afterward. An initial absolute-path
  selection failed the workspace-member guard; the relative selection fixes
  that local setup error and the complete native rerun passes. Generated debug
  caches/old binaries from this checkout alone were cleared after disk exhaustion;
  source, sibling checkouts and retained external evidence were preserved.
- **Raw-byte xattr read compatibility is adapted locally (2026-10-27).**
  Upstream `ea72663` (#590) core/parser and C names-only listing changes now
  preserve exact `raw_name` bytes, provide raw listing/resolved lookup, and
  prevent lossy display spelling from becoming lookup identity. Four actual
  native failures precede repair (both parsers, neighboring lookup and inline
  overflow content); all twenty xattr unit controls now pass, including exact
  parsed bytes and a display-collision setter/remover preservation control.
  Our guarded setters/removers and their byte encoders are retained. The
  newer CLI and dependency changes are excluded. A new independent Linux
  test builds inline/external raw-name cases at 1/4 KiB, compares Rust values and C names, mutates an ordinary neighbor and checks Linux retained bytes;
  it compiles but has not executed here. Full retention/lint and independent
  CI remain pending. The approved trailing-slash adaptation is next; this
  update does not qualify casefold writes or advance stages 4–5 percentages.
- **Selective upstream compatibility backports are in progress (2026-10-27).**
  The user approved three changes after a public upstream/seven-fork survey:
  truncate file-size limits (#587), exact raw xattr names (#590), and the
  trailing-slash walker portion of #589. The initial native baseline retains
  **1,494 successful occurrences / 526 controls**. The truncate regression
  actually accepted an oversized Ext2 sparse length before repair; it now
  passes for Ext2 and Ext4, including exact accepted boundaries, EFBIG on
  boundary+1/u64::MAX and unchanged device bytes on refusal. Adapted core
  `3322f7b` adds mapping/feature-specific bounds and EFBIG; newer CLI/core,
  workflow and dependency upgrades are excluded. An eight-layout Linux
  boundary-bisection/Rust+C readback test is added, not yet executed here.
  Full retention/lint and exact independent CI remain pending. The xattr/path
  fixes are next within this approved run; the hard-link design remains
  complete but native link red preparation has not run. Casefold writes stay
  blocked; stages 4–5 remain not yet quantified. Local PRoot test retries were
  interrupted at sandbox approval. Direct native checks use the identical
  immutable core via a temporary manifest path selection, restored afterward;
  the committed manifest/lock and sibling files remain unchanged.
- **Public hard-link staging/marker/dispatch design audit is complete.**
  At clean fetched `c15e588`, all five Rust/C forms, two-Into/name/source/parent
  priorities, existing target identity/nlink and shared growth/commit/readback
  paths are traced. The bounded design uses one context, completed LinkPlan,
  **optional** full-primary-SB handling and complete effects/C owned readback.
  Non-growing links can legitimately omit SB staging; growth must retain its
  actual allocation debits. Source/parent timestamp and path-policy candidates
  await Linux measurement; current link has no direct Runtime clock/generation
  hook, so unreached copied hooks must not be counted as reds. Concrete native
  safety/positive/reentry/uncertainty and later reference/emission requirements
  are recorded. No guard, executed red, semantic repair or new Linux result is
  claimed. Next actually execute/save/reapply/repeat a dormant native link safety
  patch, restore source and retain **1,494 occurrences / 526 controls** before
  reference-only Linux policy work. Exact c15e588 full CI is still in progress
  at inspection; prior verified `38295af` source/evidence is unchanged. Stages
  4–5 remain not yet quantified; casefold writes and broader qualification gates
  stay blocked/open as recorded below.
- **Scoped regular-file creation review is closed after verified green full CI.**
  Budget repair `38295af` passes every job and ci-ok in exact run `40080977121`.
  Five source/digest/all-CRC-verified tier archives retain all **2,337 whole-suite
  successes**, zero failed and one unchanged ignored slow fuzz test; all **526
  controls once in five contexts**, every 1,494-unit occurrence and 527 architecture
  image passes remain. Both actual whole wrappers and 1,060 floors now pass.
  All **twenty native shell and 159 native installed-CLI checks execute and pass**;
  standalone CLI also passes 159. Implementation/test/workflow bytes still match
  independently audited `f436f4e`, so its sixteen-layout/112-image creation and
  retained-family evidence remains applicable. No further blocker is found within
  caller exclusion, healthy layouts and closed external reference handles.
  Create and mkdir patches remain historical/promoted; do not reapply. Next:
  audit public hard-link (`link`) Rust/C staging/marker/dispatch and source/
  destination identity before new native reds or migration. Stages 4–5 remain
  not yet quantified; ownership/mapping/index growth/lifecycle/recovery/ignored-
  fuzz/Windows/SteamOS/card qualification stay open. Casefold writes stay blocked.
- **Create independent emission/full-retention review passes; measured CI budget repair awaits qualification.**
  Exact `f436f4e` passes every focused job and the complete sixteen-layout /
  **112-image / 48-creation** saved-byte audit, all 544 debugfs/432 Linux records,
  allocation/checksum/parent/peer/journal checks and all retained-family audits.
  Verified full-CI archives retain every **2,228** earlier whole-suite success
  plus 108 controls/one emission: **2,337 passes**, zero failed, one unchanged
  ignored slow fuzz case; all **526 controls in five contexts**, 1,494 units and
  527 architecture image passes. Both actual native/guest wrappers fail only
  measured output caps, exit 65. Identical complete logs fail old and pass new
  **3,950 / 238,000** and **4,120 / 248,000** limits with output/selection/floors
  retained. All source/evidence audits find no further scoped blocker under
  caller exclusion, healthy layouts and closed reference handles. Full ci-ok,
  actual whole-suite floors and twenty trailing shell/159 native installed-CLI
  checks still require the budget-repair CI; those trailing checks were unrun.
  Next inspect that exact CI and verify retention/floors/trailing checks before
  scoped create closure. No driver/test/write-gate change; stages 4–5 remain not
  yet quantified, casefold writes and Windows/SteamOS/card qualification blocked.
- **Guarded public regular-file creation and measured policy repair pass native validation.**
  All five Rust/C forms now share one mount context through complete owned
  staging, marker, commit/allocation publication and optional C readback.
  Parent timestamps and create-local mixed-prefix ordering are repaired while
  preserving Into-before-name, modes/names/FFI and existing path syntax. All
  **99 promoted controls plus nine actual private-boundary controls pass**;
  all **304 measured I/O injections and four public publication unwinds** retire
  the mount. Normal native retention is **1,494 passes**, zero failed/ignored,
  all 1,386 prior occurrences and **526 controls once**; full lint passes.
  The actual unit-log overflow is repaired by identical old-red/new-green
  replay at **1,860 lines / 143,000 bytes**, preserving selection and floor.
  Both create patches are now historical promoted artifacts; **do not reapply**.
  A separate sixteen-layout/112-image Rust/C Linux emission test and floor-one
  focused job are added. Actual CI, saved-byte audit, complete-suite retention
  and standalone family review remain pending. Next inspect exact focused/full
  CI, repair real failures, then audit emitted/retained images and close the
  scoped review only after evidence passes. Stages 4–5 remain not yet quantified;
  casefold writes and Windows/SteamOS/card qualification remain blocked.
- **Measured native create policy reds are executed; guarded repair is next.**
  The new dormant 27-control policy patch runs **fourteen passes / thirteen
  failures**, every red reaching all six native profiles. Ten controls reproduce
  parent mtime/ctime for first/back-to-back creation through all five Rust/C
  forms; three reproduce regular/missing-prefix versus final-length errors.
  All 270 successful creations and 714 refusal/priority/syntax observations
  execute, preserving explicit modes/names/Into/FFI/rooted-path contracts.
  Both patch orders recreate identical combined source; both actual **99-control
  runs repeat 28 passes / 71 failures**, retaining all original 72 safety outcomes,
  **288 I/O injections and four public publication unwinds**. Full lint passes
  with both audits active; source is restored and native units retain all
  **1,386 occurrences / 418 controls**, zero failed/ignored. No production repair
  or limit/write-gate change. Next inspect this checkpoint's CI, then implement
  the reviewed create shared-context/owned-plan/full-SB/complete-effect boundary
  and measured timestamp/path repair, preserve optional C readback, and qualify
  actual emitted images independently. Stages 4–5 remain not yet quantified;
  casefold writes and Windows/SteamOS/card qualification remain blocked.
- **Independent Linux create reference passes and all saved bytes are audited.**
  Exact `e384ed9` passes all nine focused jobs and every full CI job/ci-ok.
  Its source/digest/CRC-verified creation archive passes audit of all **eight
  layouts / 48 images / 24 creations / 216 consistent error observations** and
  **408 debugfs reports**, including final cold namespace/empty-file records,
  exact bitmaps/counters/itable-unused/CRCs, protected peers/EA and clean journal
  state. Linux advances parent mtime/ctime, preserves atime/links/storage and
  consumes only an inode. Requested 02750/01770/0750 survive; regular/missing
  parent errors precede final-name length. Keep syscall facts distinct from
  explicit API/Into/FFI/rooted-path/trailing contracts. Verified full-CI archives
  retain all **2,227 previous successes plus one reference (2,228 passes)**,
  all **418 controls in five contexts**, 1,386 unit occurrences, 527 architecture
  image passes and trailing twenty shell/159 installed-CLI checks. One unchanged
  ignored fuzz case remains a broader gate. Next prepare and actually execute
  native measured create policy reds, compose with the dormant 72-control safety
  patch and restore the baseline before guarded repair. No production repair or
  write-gate change. Stages 4–5 remain not yet quantified; casefold writes and
  actual Windows/SteamOS/card qualification stay blocked.
- **Independent reference-only Linux create policy test is added; actual CI evidence is pending.**
  One exact VM test covers eight 1/4 KiB, legacy 128-byte/checksummed 256-byte,
  direct/journal layouts. Linux exclusively creates the files; the driver only
  reads closed images. Six phases plan **48 saved images / 24 creations / 216
  actual error observations**, plus final cold namespace/empty-file verification,
  parent/child timestamps, requested/actual modes, byte names, peers/EA and exact
  per-group inode debits with unchanged blocks/directory counts. A separate
  focused job enforces floor one and retains complete logs/reports/images.
  Python syntax, Rust compilation, all 94 CI profile/contract checks, full lint
  and native **1,386-occurrence / 418-control** retention pass locally. No Linux
  answer or independent qualification is claimed before actual VM execution and
  saved-byte audit. The dormant 72-control safety patch and production creation
  remain unchanged. Next inspect exact focused/full CI and audit actual reference
  errors/modes/times/images, repairing real test or driver failures before native
  semantic reds or guarded repair. Stages 4–5 remain not yet quantified; casefold
  writes and Windows/SteamOS/card qualification remain blocked.
- **Native public-create safety/positive preparation is executed and saved.**
  The dormant `CASEFOLD-CREATE-RED.patch` recreates identical source and repeats
  all **72 actual test names/outcomes: 14 passed / 58 failed / zero ignored**,
  exit 101, both complete logs 908 lines / 58,680 bytes. These are current public
  exclusion/refusal/reentry/unwind/C-readback/retirement reds, not future private-
  SB diagnostics. All **288** measured I/O fault injections and **four** actual
  post-commit publication unwinds execute before aggregate assertions. All 54
  primary storage profiles plus same-table/uninit/parent-link/path/FFI/readback
  positives execute. Full lint passes with the audit active; source is restored
  and native units retain all **1,386 occurrences / 418 controls**, zero failed/
  ignored. No production repair, semantic red or new Linux result is claimed.
  The preceding audit's exact CI has all other jobs passed and the native job
  still running at inspection. Next: independently measure reference-only Linux
  exclusive regular-file creation, audit its saved bytes, then prepare measured
  semantic reds before guarded migration. Stages 4–5 remain not yet quantified;
  casefold writes and ownership/mapping/lifecycle/recovery/ignored-fuzz/actual
  Windows/SteamOS/card qualification remain blocked/open as previously recorded.
- **Public regular-file create design audit is complete; executed native safety preparation is next.**
  At clean fetched `1248f71`, its exact full CI `40013659564` passes every job
  and ci-ok. The audit traces all five Rust/C forms, marker-before-validation,
  shared inode setup, flavor-specific empty-file construction, parent inline/
  indexed/grown/same-table staging and complete commit/allocation publication.
  The design uses one existing mount context, private nested dispatch, one
  completed owned create result and a create-only full-SB/effect boundary;
  optional counted C attributes remain under that context. Counted create's
  **Into-before-name** priority and actual separator/root-walk syntax remain
  distinct from mkdir and Linux syscall policy. Concrete safety/positive and
  later independent-reference/emission requirements are recorded. No guard,
  executed red, semantic repair or new Linux result is claimed. Next actually
  execute/save/reapply/rerun the dormant native create audit, restore source
  and retain all 1,386 units/418 controls before independent policy work and
  migration. Stages 4–5 remain not yet quantified; casefold writes and broader
  ownership/mapping/lifecycle/recovery/Windows/SteamOS/card gates stay blocked.
- **Scoped mkdir review is closed after verified green full CI.** Budget repair
  `7f9617d` passes every job and ci-ok in exact full run `39997261948`. Five
  source/digest/CRC-verified tier archives retain all **2,227 whole-suite passes**,
  zero failed and one unchanged ignored slow fuzz test, all **418 controls once
  in five contexts**, all 1,386 unit occurrences and 527 architecture image
  passes. Actual native/guest wrappers and floors now pass; all **twenty native
  shell checks and 159 installed-CLI checks** execute and pass. Existing focused
  16-layout/112-image mkdir and retained-family/source audits remain applicable
  because implementation/test/workflow bytes are unchanged. No further scoped
  blocker is found under caller exclusion, healthy layouts and closed reference
  handles. Next: audit public regular-file `create` Rust/C staging/marker/
  dispatch, including shared-helper users and optional C readback, before new
  native reds or migration. Stages 4–5 remain not yet quantified; ownership,
  mapping/growth, lifecycle, recovery, ignored-fuzz and actual Windows/SteamOS/
  card qualification stay open. Casefold writes remain blocked.
- **Mkdir full-suite retention and retained-family review pass; CI budget repair is next to qualify.**
  Implementation `97356c9` and evidence `31e64be` each retain **2,227 whole-suite
  successful occurrences**, zero failed, one unchanged ignored slow fuzz test,
  all 418 controls once in five CI contexts, all 1,386 unit occurrences and 527
  aarch64 image passes. All eight focused jobs now pass, and fresh retained
  metadata/setter/removal/unlink/rmdir/reference/symlink image audits pass at
  the implementation source. Scoped source/evidence review finds no further
  blocker within caller exclusion, healthy layouts and closed reference handles.
  Actual full native/guest wrappers fail only on measured byte overflows, exit
  65; identical complete logs fail old and pass new **220,000 / 230,000** byte
  caps. Line limits, floors, selection and output are unchanged. The 94 CI
  profile/contract checks and full lint pass. Native trailing shell/installed-CLI
  checks were **unrun**, so full CI/ci-ok and scoped mkdir closure remain pending
  on the new budget-repair commit. Next: inspect that exact CI and verify complete
  retention, actual floors and trailing checks before closing mkdir review.
  Stages 4–5 remain not yet quantified; casefold writes and actual Windows/
  SteamOS/card qualification remain blocked.
- **Independent mkdir Linux emission now passes and its saved bytes are audited.**
  Exact repair `97356c9` passes the new focused job with floor one. Its source/
  advertised-digest-verified archive retains all sixteen layouts / 112 images /
  48 creations. All 544 debugfs reports, 432 cold kernel metadata records,
  per-group allocation/bitmap/CRC accounting, protected peer contents/attributes
  and dirty/closed/journal state checks agree with saved raw bytes. Explicit
  modes and byte/255-byte identities remain intact. Exact CI units retain all
  1,386 successful occurrences and 418 controls once; standalone CLI and the
  other separate focused jobs pass. Main focused fixtures and full runner/
  fixture preparation remain pending; whole-suite/aarch64 retention, ci-ok,
  retained-family audit and standalone review closure are not yet established.
  No driver, test, workflow, global limit or write-gate change. Next: inspect
  remaining exact CI, repair real failures, retain complete-suite evidence and
  perform the scoped source/evidence review. Stages 4–5 remain not yet
  quantified; casefold writes and Windows/SteamOS/card qualification stay blocked.
- **Guarded public mkdir and measured policy repair pass native validation.**
  All five Rust/C forms now share one mount context and completed owned plan;
  the mkdir-only full-SB boundary preserves debits/dirty state through complete
  effects and allocation publication. Parent timestamps and mkdir-local mixed
  pathname precedence are repaired without changing counted name/generation/
  mode/FFI contracts; optional C readback stays under the same context. All 100
  promoted controls plus four real private-boundary controls pass, with every
  one of 352 measured I/O uncertainties and four public publication unwinds now
  retiring the mount. Normal native retention is exactly **1,386 passes**, no
  failed/ignored, all 1,282 prior occurrences and 418 controls once. Full lint
  passes. An actual unit-only output overflow is repaired by byte-identical
  old-red/new-green replay at 1,740 / 131,000, preserving selection/floor/output.
  Both mkdir patches are now historical promoted artifacts; do not reapply.
  A sixteen-layout/112-image Rust/C Linux emission test and floor-one focused
  job are added; actual CI, artifact audit, complete-suite retention and scoped
  review closure remain pending. Next: inspect exact CI and saved evidence,
  repairing actual failures before closure. Stages 4–5 remain not yet quantified;
  casefold writes and Windows/SteamOS/card qualification stay blocked.
- **Measured native mkdir policy reds are executed; guarded repair is next.**
  The new dormant 27-control policy patch reaches fourteen passes / thirteen
  failures across all six native profiles: parent timestamps and mixed pathname
  precedence, with explicit mode/name/FFI/generation contracts preserved. Both
  actual combined 100-control runs repeat **25 passes / 75 failures**, retaining
  every original safety outcome, all 352 measured I/O injections and four public
  publication unwinds. Both patch orders recreate identical source; active lint
  passes and source is restored with all 1,282 units / 314 controls retained.
  Prior exact full/focused CI now passes; digest-verified logs retain all 2,122
  whole-suite passes, 314 controls in five contexts, 527 architecture image
  passes and trailing shell/CLI checks. Four failures remain future SB
  diagnostics, not current public clean-bit bugs. No production repair or global
  limit/write-gate change. Next: inspect this checkpoint's CI, implement the
  reviewed mkdir shared-context/owned-plan and measured policy repair, then
  independently qualify actual driver emissions. Stages 4–5 remain not yet
  quantified; casefold writes and Windows/SteamOS/card qualification stay blocked.
- **Independent Linux mkdir policy evidence now passes and is audited.** At
  `79ed456`, exact focused job `119992355646` in run `39826871672` executes one
  successful test, zero failed/ignored, with floor one. The advertised-digest/
  source-verified archive passes read-only audit of all eight layouts / forty
  images / 216 consistent error observations / 24 creations and all 328 debugfs
  reports. Parent mtime/ctime advances, atime and peers remain intact, and each
  creation consumes one block/inode with exact per-group used-directory credits.
  Actual Linux mode/path answers are recorded without silently changing the
  existing Rust/C contract. Full-source CI units/CLI/fixtures/MSRV/semver/wasm/
  provenance/aarch64 pass; both whole suites remain pending at inspection.
  No driver repair or write-gate change. Next: inspect remaining exact CI, then
  prepare measured native mkdir policy reds, keeping explicit validation/mode
  contracts distinct from Linux path behavior. Stages 4–5 remain not yet
  quantified; casefold writes and Windows/SteamOS/card qualification stay blocked.
- **Independent reference-only Linux mkdir test is added; VM results are pending.**
  One exact test covers eight 1/4 KiB, 128-byte legacy/256-byte checksummed and
  direct/journal layouts. Linux creates all bytes; the driver only mounts the
  closed images read-only. Five saved phases per layout plan 216 actual error
  observations, 24 successful creations and 40 images, preserving parent/child
  times, requested/actual modes, raw-byte/255-byte names, namespace, peers,
  attributes and exact per-group allocation accounting. A separate focused CI
  job has floor one and retains complete logs, JSON and images. Python syntax,
  Rust compilation, full lint and native baseline retention pass locally; no
  Linux outcome is claimed before the VM job and saved-byte audit execute.
  The dormant 73-control safety patch, public mkdir and global limits remain
  unchanged. Next: inspect this exact source's focused/full CI and audit its
  actual observations/images before native policy reds or guarded repair.
  Stages 4–5 remain not yet quantified and casefold writes remain blocked.
- **Native mkdir safety red/positive preparation is executed and saved.** The
  dormant `CASEFOLD-MKDIR-RED.patch` repeats the same 73 executed names after
  reversal/reapplication: **11 passed / 62 failed / zero ignored**, exit 101,
  both complete logs 931 lines / 59,622 bytes. Fifty-eight failures exercise
  current public exclusion, untouched refusal, reentry, unwind, C readback and
  mount-retirement gaps; four are future pre-marker SB ordering diagnostics,
  not current public clean-bit regressions. All 352 measured write/flush fault
  injections and four actual post-commit publication unwinds reach their
  checks before aggregate assertions. Fifty-four primary creation profiles and
  same-table, constructed multi-group/uninit, symlink and post-commit C-read
  compatibility controls execute. Full lint passes with the audit active.
  Restored native units retain all 1,282 names and 314 controls, zero failed/
  ignored. No production repair or Linux policy result is claimed. Next:
  independent reference-only Linux mkdir timestamp/name/error/accounting
  measurement before semantic reds or guarded repair. Stages 4–5 remain not yet
  quantified; casefold writes and Windows/SteamOS/card qualification stay blocked.
- **Public-mkdir staging/marker/dispatch design audit is complete.** At clean
  fetched `a98fcdc`, its exact full CI `39711385558` passes every job and ci-ok.
  The audit traces all five Rust/C forms, current refusal/name/generation
  priorities, inode/data allocation, inline/indexed/grown-parent helpers,
  same-table splicing, inheritance/counters/checksums and complete commit/
  allocation-publication effects. The chosen design uses one shared context,
  private nested dispatch with both pure verdicts, one owned inode/buffer result
  and a mkdir-only full-SB/marker/commit boundary. Counted C optional attributes
  require readback under that same context, preserving possible read errors
  after successful creation. Concrete native red/positive groups and later
  independent Linux policy/emission requirements are recorded in the existing
  review. No guard, semantic fix, new test execution or Linux measurement is
  claimed. Next: actually execute/save/reapply/rerun native mkdir safety and
  positive controls, restore source and retain the 1,282-unit/314-control
  baseline before implementation. Stages 4–5 remain not yet quantified;
  casefold writes and Windows/SteamOS/card qualification remain blocked.
- **Scoped rmdir review is closed after verified full-CI retention.** Budget
  repair `0da4221` and evidence checkpoint `49d3484` both pass every job and
  `ci-ok` in full runs `39702689248` and `39703344673`. Ten downloaded tier-log
  archives match advertised SHA-256 and exact workflow source. Both whole
  suites in both runs retain all 2,121 successful occurrences, zero failed and
  one unchanged ignored slow fuzz test. All 314 controls occur once in five
  contexts; all three unit contexts retain 1,282 names, aarch64 retains 527
  image passes, and native trailing checks pass twenty shell checks and 159
  installed-CLI checks. Actual wrappers and floors pass with complete output.
  The completed source and 16-layout/112-image rmdir review remain applicable
  because implementation/test/workflow bytes are unchanged. No further scoped
  blocker is found under caller exclusion, healthy layouts and closed reference
  handles. Next: the public-mkdir staging/marker/dispatch design audit, before
  any new implementation or executed-red claim. Stages 4–5 remain not yet
  quantified; other families, ownership/mapping/lifecycle/recovery and actual
  Windows/SteamOS/card qualification remain open. Casefold writes stay blocked.
- **Rmdir source/saved-evidence review is complete; full-CI closure is pending.**
  Exact `fbc334e` focused CI and read-only audit pass all 16 layouts / 112 images /
  48 removals, with exact group accounting, shared-peer preservation, timestamps,
  checksums and dirty/closed state. Every retained metadata/setter/removal,
  unlink-emission/policy, rmdir-reference and symlink saved-byte audit now passes
  at that exact source. Both complete suites retain all 2,034 previous passes
  and add 87 (2,121 passed, zero failed, one unchanged ignored), with all 314
  controls in five contexts. Actual byte-only overflows are repaired and pushed
  in `0da4221`: native 205,000 / guest 213,000 pass identical full-log replay;
  line limits, floors, filters and output stay intact. Full repair CI
  `39702689248` is running; trailing native shell/installed-CLI checks still gate
  scoped closure. API credential requests returned 401, but public REST status/
  artifact downloads and default Git fetch/push work; no reconnect is needed for
  this public read workflow. Source and image audits need not be repeated solely
  for these configuration/docs commits. Next: inspect exact repair-source full
  CI, retention and trailing checks, then close the scoped rmdir review before
  another family. Stages 4–5 remain not yet quantified; casefold writes and
  broader Windows/SteamOS/card qualification stay blocked.
- **Guarded public rmdir implementation now passes native controls.** All five
  Rust/C entries share the mount guard, pure verdicts and one completed owned
  deletion plan; the rmdir-only boundary preserves SB credits/dirty state and
  latches complete effects. Measured parent timestamps, long-name errors and
  path-parent priorities are repaired. All 82 promoted controls and four new
  private/precedence controls pass; all 340 original uncertainty injections now
  retire eight guarded families and clean close. Native retention is exactly
  1,196 + 86 = 1,282 passes, zero failed/ignored, all 314 controls once. Full lint
  passes. Actual unit-only output overflow is repaired by byte-identical old-red/
  new-green log replay at 1,560 / 115,500; whole caps/floors/filters stay unchanged.
  A separate 16-profile/112-image ordinary Rust/C Linux emission matrix and exact
  focused job are added, but independent execution/artifact audit/full retention
  and standalone family review are pending. Both rmdir patches are historical
  and must not be reapplied onto their promoted source. Next: inspect actual
  exact-source focused/full CI, audit saved output and repair genuine failures
  before scoped review closure. Stages 4–5 remain not yet quantified; casefold
  writes remain blocked and Windows/SteamOS/card qualification is not reached.
- **Native measured rmdir policy reds are executed; repair remains next.** The
  new dormant 27-control policy patch runs nine passes / 18 failures, every red
  reaching all six native profiles. It reproduces parent timestamps, ordinary
  long-name errors and mixed path precedence, with 120 valid removals and 564
  refusal observations. Both patch orders recreate identical source; both actual
  combined 82-control runs report 15 passes / 67 failures, retaining every
  original 55-control outcome and all 340 uncertainty injections. Four failures
  remain future staged-SB diagnostics, not current public defects. Full lint
  passes with both audits active; byte-restored units retain all 1,196 names
  and 228 controls. All preceding focused/full jobs and ci-ok now pass at
  `ccef341`; verified native/guest evidence retains all 2,033 earlier passes
  plus one oracle (2,034 passed, one unchanged ignored), with all 228 controls
  in five CI contexts and every trailing native check. No production, workflow,
  global cap/floor/filter, pin/lock/sibling or write-gate change. Next: inspect
  this checkpoint's CI, then perform the reviewed rmdir guard/semantic repair
  and independent driver-emission qualification. Stages 4–5 remain not yet
  quantified; casefold writes remain blocked.
- **Independent Linux rmdir policy is executed and its saved evidence audited.**
  Exact source `ccef341` passes the new reference job in focused run
  `39644593294`. The SHA-256-verified archive retains all eight coherent profiles
  and **32 phase images**, with **184 consistent error observations and 24 valid
  removals**. Raw-byte/kernel/debugfs checks confirm parent mtime/ctime updates,
  unchanged refusal metadata, 255-byte success/ordinary 256-byte ENAMETOOLONG,
  parent link changes, exact per-group inode/block/used-directory credits,
  exclusive/shared attributes and peer preservation. All 32 e2fsck checks and
  32 mounts pass; the final ledger per profile frees four blocks and three
  inodes, decreasing used directories by three. Native local/exact-CI units
  retain all 1,196 names and 228 controls; lint passes. Other focused unlink/
  setter-policy jobs pass, while the main fixture job and full suites remain
  pending. No public-rmdir/guard/semantic change, dormant-patch edit, global
  cap/floor/filter, pin/sibling or write-gate change. Next: inspect actual
  remaining CI outcomes, then execute measured native policy reds with advancing
  clocks and long-name priorities before guarded implementation. Stages 4–5
  remain not yet quantified; casefold writes remain blocked.
- **Independent reference-only Linux rmdir oracle is added; exact CI execution
  is pending.** One exact needs-host test covers eight 1/4 KiB, 128/256-byte
  inode and direct/journal layouts, with Linux-created ordinary directories,
  exclusive/shared external attributes, 23 error observations and a real
  255-byte removal per layout. Four saved phases per layout retain parent/name
  metadata, closed-handle exclusive/shared removal, statvfs, inode/block and
  used-directory credits, read-only input-shape checks, peer preservation,
  judged e2fsck and debugfs reports. A separate focused job has floor one and
  keeps complete logs/images. Python syntax, full lint and native units pass;
  all 1,196 baseline occurrences and 228 controls are retained. The test has
  not run in this cloud host, which has no KVM; no Linux outcome or driver
  repair is claimed before actual CI evidence. Public rmdir, the dormant
  55-control patch, global caps/floors/filters, pins/siblings and write gates
  are unchanged. Next: inspect exact reference CI and its actual saved bytes,
  repair any measurement failure, then prepare measured native policy reds.
  Stages 4–5 remain not yet quantified; casefold writes remain blocked.
- **Native rmdir red/positive preparation is executed and saved; active source
  is restored.** The dormant `CASEFOLD-RMDIR-RED.patch` is reversed, reapplied
  byte-identically and rerun. Both final runs execute the same 55 names:
  **six passed / 49 failed / zero ignored**, exit 101, each 962 lines /
  71,082 bytes. Forty-five failures exercise actual public exclusion, untouched
  refusal, callback, unwind and uncertainty retirement; four are future staged-SB
  ordering diagnostics, not current public clean-bit regressions. Positive
  controls execute 127 ordinary/inline/shared-attribute/DIR_NLINK/path removals;
  all 340 observed write/error/unwind fault injections execute before aggregate
  assertions. Full lint passes with the audit active. Restored native units
  retain all 1,196 baseline names and 228 controls, zero failed/ignored. No guard,
  semantic, workflow, cap/floor/filter, pin/lock, sibling or write-gate changes.
  Next: independent reference-only Linux rmdir timestamp/name/error/reclamation
  policy measurement before semantic reds or guarded repair. Stages 4–5 remain
  not yet quantified and casefold writes remain blocked.
- **Scoped unlink review is closed; public-rmdir design audit is complete.**
  Repair source `661ba75` passes every job and `ci-ok` in full run `39547402531`.
  SHA-256-verified native/guest artifacts each retain all 1,975 preceding passes
  and add 58 (2,033 passed, zero failed, one unchanged ignored slow fuzz case).
  All 228 controls occur once in five CI contexts; all three unit contexts
  retain 1,196 names and aarch64 retains 527 image tests. Native trailing checks
  pass twenty shell checks and 159 CLI checks. Actual native 3,433 lines /
  192,301 bytes and guest 3,603 / 201,982 fit the measured limits. Combined with
  completed source and focused-image reviews, no further scoped unlink blocker
  is found under caller exclusion, closed reference handles and healthy layouts.
  The subsequent rmdir source/design audit traces all five Rust/C forms, empty
  checks, storage/attribute/counter/parent-link staging and complete effects.
  It records one shared context, private nested dispatch, a completed owned plan
  before marker/commit and concrete next controls. Parent timestamp/long-name
  policy still needs independent Linux measurement before semantic changes.
  No implementation/test/workflow/cap/floor/filter/pin/sibling/write-gate changes
  in this review checkpoint. Next: actually execute and save dormant native
  rmdir red/positive controls, reapply/rerun, then restore active source before
  implementation. Stages 4–5 remain not yet quantified; casefold writes and
  broader ownership/mapping/lifecycle/recovery/Windows/card gates stay blocked.
- **Both unlink whole suites retain the baseline; the measured guest output
  limit is now repaired too.** Original exact-source run `39542365226` is
  complete: native and guest each retain all 1,975 preceding successful
  occurrences and add 58 (2,033 passed, zero failed, one unchanged ignored slow
  fuzz case). SHA-256-verified artifacts confirm all 228 controls once in each
  of five CI contexts, with all three unit contexts retaining 1,196 names.
  Guest fails only its output budget: 3,601 lines / 201,734 bytes exceeds
  196,500; the actual job and identical saved-log replay return 65. The same
  complete bytes pass at 203,000 with the unchanged 3,900-line limit and
  1,060-test floor. The 6,500-byte adjustment tracks 6,093 bytes of growth from
  the preceding baseline and restores 1,266 bytes of headroom. Native remains
  at the separately measured 195,000-byte repair. Lint and budget-contract
  checks pass. No implementation/test/workflow/filter/floor/pin/sibling/write-gate
  change. Native-only repair run `39546367987` is still in progress; fresh full
  CI with both measured limits, including native trailing checks, remains
  required before scoped unlink closure and the public-rmdir design audit.
  Completed source/focused-image reviews stay applicable. Stages 4–5 remain
  not yet quantified; casefold writes stay blocked.
- **Unlink full-native retention passes; its measured output-budget failure is
  repaired.** Exact source `cd15806` in run `39542365226` retains all 1,975
  preceding successful occurrences and adds 58 (2,033 passed, zero failed, one
  unchanged ignored slow fuzz case). All 228 controls occur once in the complete
  suite; native units retain all 1,196 names and controls. The verified archived
  suite log is 3,433 lines / 192,299 bytes, exceeding the 189,000-byte cap;
  the actual job reports exit 65. Identical complete-log replay reproduces that
  failure and passes at the measured native-only 195,000-byte ceiling, retaining
  the 3,700-line cap, 1,060-test floor, command, filters and all output. The
  6,000-byte adjustment tracks 5,728 bytes of growth from the preceding baseline
  and restores 2,701 bytes of headroom. Guest CI is still running; its ceiling
  stays unchanged. Native checks after the failed wrapper did not run. Fresh
  repair-source full CI, guest retention and trailing native checks remain
  prerequisites to scoped unlink closure and the subsequent public-rmdir audit.
  Source/focused-image review is already complete; do not repeat it merely for
  this configuration repair. Stages 4–5 remain not yet quantified and casefold
  writes stay blocked.
- **Standalone unlink source/saved-evidence review is complete; full-CI closure
  is pending.** At byte-identical implementation source `cd15806`, the source
  walk found no additional blocker within the caller-exclusion, closed-reference-
  handle and healthy-layout contract. Exact focused run `39542365213` now passes
  every job. Its SHA-256-verified main archive passes all 15 exact logs and the
  saved-byte audits for 16 setter layouts, eight fresh first-attribute cases and
  84 retained metadata profiles. The separate policy artifact retains all 32
  ordinary observations, 16 byte-identical Rust/C images and four explicitly
  diagnostic duplicates; all five symlink tests and their saved images pass.
  The earlier 16-profile/80-image unlink evidence remains verified. Exact CI
  standalone/aarch64 units retain all 1,196 names and 228 controls, and aarch64
  retains all 527 image tests. The two full native/guest jobs in `39542365226`
  are still running, so no family closure or complete-suite retention is claimed.
  No implementation/test/cap/floor/filter/pin/sibling/write-gate changes. Next:
  inspect those completed full logs, repair actual failures, then close this
  scoped review before the public-rmdir design audit. Stages 4–5 remain not yet
  quantified; casefold writes stay blocked.
- **Guarded public unlink repair now passes independent emitted-image evidence.**
  Source `cd15806` passes the focused `unlink-emission` job in run `39542365213`:
  one test covers all 16 Rust/C, 1/4 KiB, legacy/checksummed, direct/journal
  combinations, each with exclusive/shared attributes and surviving/final links.
  The SHA-256-verified compressed archive retains **80 images**; raw-byte/Linux
  report audit confirms timestamps, data/attribute preservation, checksums,
  bitmap/free-count reclamation, shared-peer preservation and dirty/closed state.
  All **80 e2fsck checks and 48 kernel mounts/readbacks** pass. Native units retain
  every 1,139 baseline occurrence and add 57 (1,196 passed, zero failed/ignored);
  all **228** controls pass locally and in verified exact-source CI units. Lint
  passes. The unit ceiling is repaired only from actual measured growth; whole-
  suite caps/floors/filters stay unchanged. Main focused fixture work and full
  CI `39542365226` remain pending. Standalone unlink-family review/complete-suite
  retention are prerequisites to closure. Both historical patches stay unchanged
  and must not be reapplied onto promoted source. Stages 4–5 stay not yet
  quantified; casefold writes remain blocked.
- **Linux-measured native unlink policy reds are executed and saved; source
  is restored.** The new dormant `CASEFOLD-UNLINK-POLICY-RED.patch` adds 22
  controls: **two pass / 20 fail / zero ignored**, with every red reaching all
  six native profiles through one of five Rust/C entries. Parent times after
  both removals, surviving ctime and 256-byte name errors disagree with the
  independent Linux observations. Positive controls execute 30 successful
  255-byte removals and 60 timestamp-preserving ordinary refusals. Both patches
  compose in either order and recreate identical tested source; two actual
  combined runs retain **seven passes / 43 failures / zero ignored** across all
  50 names, preserving every original 28-control outcome. Full lint passes with
  both audits active. Restored units retain all 1,139 passes and 171 controls
  with zero failed/ignored (1,387 lines / 96,842 bytes). Focused Linux
  CI now passes all jobs; verified guest evidence retains all 1,974 passes and
  adds the unlink oracle (1,975 passed, one unchanged ignored fuzz case). Exact
  full CI's native job remains pending. Next is the reviewed unlink repair,
  explicitly superseding only the historical dormant ENOENT behavior pin with
  the measured long-name contract. Stages 4–5 remain not yet quantified;
  casefold writes stay blocked.
- **Independent Linux unlink policy evidence is executed and inspected.**
  Source `30c15f4` passes the new reference job in focused run `39534096165`.
  The SHA-256-verified archive contains all eight coherent profiles and 24 saved
  phase images. A read-only raw-byte/report audit confirms parent mtime/ctime
  updates, surviving-inode ctime updates with unchanged atime/mtime/data/attrs,
  non-final free-count preservation, and closed-handle final reclamation of one
  inode plus every target data/map/attribute block. All 40 error observations
  match Linux, including 256-byte ENAMETOOLONG; all eight 255-byte removals succeed.
  Native baseline/post-change runs retain exactly 1,139 passes and 171 controls;
  full lint passes. Full CI `39534096164` and the main focused fixture job remain
  pending at this evidence checkpoint; the separate policy/symlink job passes. Production unlink and the dormant 28-red
  patch are unchanged. Next: inspect those outcomes, then prepare and actually
  execute advancing-clock native policy reds before a semantic/guard repair.
  Stages 4–5 remain not yet quantified and casefold writes stay blocked.
- **Unlink native red preparation is executed and saved, with active source
  restored.** `docs/CASEFOLD-UNLINK-RED.patch` was applied, reversed, reapplied
  byte-identically, rerun and reversed again. Both final runs execute the same
  28 names: **five passed / 23 failed / zero ignored**, exit 101; each complete
  log is 281 lines / 20,608 bytes under an isolated audit ceiling. Nineteen
  failures exercise actual public exclusion/refusal/callback/unwind boundaries;
  four are explicitly constructed future SB-order diagnostics, not current public
  clean-bit regressions. The real positive control executes 32 unlink calls in
  sixteen size/journal/Rust-C/sharing combinations with bitmap, SB/BGD, data,
  attributes, identity, dirty/close/reopen checks. Other positives retain prior-
  dirty refusals and logical/FFI priorities. Full lint passes with final audit
  source active. Restored native units retain all 1,139 baseline occurrences
  and all 171 controls (zero failed/ignored; 1,387 lines / 96,842 bytes).
  No guard, active test, workflow, budget/floor/filter, pin/lock,
  sibling or write gate is changed. Next is independent reference-only Linux
  timestamp/counted-name policy measurement before semantic changes and guarded
  implementation. Stages 4–5 remain not yet quantified; casefold writes stay blocked.
- **Public-unlink staging/marker/dispatch design audit is complete; migration
  and red execution are not yet done.** At unchanged implementation source
  `c20dd1b`, the source walk traces Rust string/byte/parent-inode calls, both C
  entries, parent symlink/trailing-slash and generation/error ordering, retained
  entry identity, surviving/final hard-link plans, data/extent/indirect storage,
  shared attributes and complete commit/publication. The reviewed next contract
  uses one shared context, private nested dispatch, verdict-only planning and an
  unlink-specific owned-buffer boundary that preserves planned counter credits
  while normalizing dirty state/checksum. Concrete red/positive controls and
  independent Linux evidence requirements are recorded in the existing review.
  Parent/surviving-inode timestamp policy needs a new independent measurement;
  no Linux mismatch or executed red is claimed from this source observation.
  Next is actual dormant native red preparation and reapplication/rerun, then
  byte-restore active source before any implementation. No code/tests/workflows,
  caps/floors/filters, siblings/pins/locks or write gates change. Stages 4–5 stay
  not yet quantified and casefold writes remain blocked.
- **Standalone public setter review is complete within its existing caller-
  exclusion/healthy-layout scope.** Budget-repair source `2c6ece6` passes every
  job and `ci-ok` in full run `39505996964`. Actual SHA-256-verified native and
  guest archives each retain all 1,974 preceding successful occurrences, with
  zero failed and one unchanged ignored slow fuzz test. All 171 controls execute
  once in five contexts; all three unit contexts retain 1,139 occurrences.
  Native output is 3,377 lines / 186,486 bytes within 3,700 / 189,000; guest is
  3,541 / 195,440 within unchanged 3,900 / 196,500. Native trailing checks now
  pass all twenty shell checks and 159 CLI checks. Exact unchanged implementation
  focused Linux evidence at `4079f36` already passes every metadata/policy/symlink
  audit. The final source check found no further blocker under the stated
  contract. Next is a bounded public-unlink staging/marker/dispatch design audit,
  before implementing another guard migration. Stages 4–5 remain not yet
  quantified; casefold writes and all broader qualification gates stay blocked.
- Exact `4079f36` full run `39458801198` has completed: both whole suites pass
  1,974 tests, retaining all 1,972 preceding occurrences and adding only the two
  refusal-order controls; zero failures and one unchanged ignored slow fuzz case.
  All 171 controls execute once in five CI contexts. The native job nevertheless
  fails at its output budget: 3,374 lines / 186,123 bytes exceeds 186,000 by 123.
  The actual complete archived log reproduces exit 65 at that cap and passes
  byte-identically at the measured native-only 189,000-byte repair. The 3,000-byte
  increase tracks 2,523 bytes of observed growth since the earlier 183,600-byte
  source and restores 2,877 bytes of headroom. Guest passes at 3,543 / 195,557
  under unchanged 3,900 / 196,500 limits. Floors, filters, line limits, test output
  and production/test code remain unchanged. Fresh repair-source full CI and
  native checks after the whole suite remain prerequisites to family closure.
- Exact refusal-order source `4079f36` now passes the complete focused Linux
  workflow `39458800523`. Its main archive matches GitHub's advertised SHA-256;
  read-only audits pass all sixteen setter layouts, eight first-attribute
  profiles and 84 retained metadata profiles. All fifteen named main-job
  oracles execute exactly once, with zero failed/ignored and complete output
  within unchanged budgets. The separately verified policy/symlink artifacts
  also pass. Full run `39458801198` is still executing both complete suites;
  neither the expected 1,974 whole-suite count nor standalone setter-family
  closure is credited yet. No code, cap, floor, pin, sibling or write gate is
  changed by this focused-evidence checkpoint.
- Full CI for `25a039e` now passes (`39452506096`). Actual SHA-256-verified
  native/guest archives each retain all 1,951 earlier occurrences and add the
  expected 21: 1,972 passed / zero failed / one unchanged ignored slow fuzz test.
  All 169 controls execute across both full suites and all three unit contexts.
  The native log is 3,372 lines / 185,898 bytes; guest is 3,540 / 195,346, both
  within unchanged caps. Current refusal-order source `4079f36` has a verified
  standalone and aarch64 CI unit artifacts with 1,139 tests, retaining all 1,137
  previous names and executing all 171 controls in each. Its policy/symlink job
  passes with all five symlink tests,
  32 ordinary policy observations and sixteen byte-identical emission images.
  The implementation source walk found no further blocker within the existing
  caller-exclusion contract. Its main focused metadata job and exact full CI
  remain running, so standalone setter-family closure is still pending. No
  implementation, test, cap, floor, dependency, sibling or write gate changes
  are made by this evidence checkpoint.
- Exact correction source `25a039e` passes focused Linux run `39452506142`.
  All five symlink tests actually execute and their verified saved images pass
  the boundary/tree/casefold audit. The same exact-source setter, first-attribute,
  sixteen policy-emission and 84 retained metadata-profile audits pass. Full run
  `39452506096` remains in progress at this checkpoint. The source review found
  a separate refusal-order regression in the unchanged-value path: external
  corruption masked invalid inline names and inline EA-inode refusal. Two controls
  executed red and now pass after restoring complete inline validation before
  external I/O. Legacy external-only precedence stays intact. Fresh native units
  retain all 1,137 occurrences and add two (1,139 passed, zero failed/ignored),
  with all 171 controls; lint passes. New exact-source full/focused results and
  complete-suite retention remain prerequisites to setter-family closure.
- Exact setter-repair source `01f96c8` now passes focused workflow
  `39391995739`. Actual SHA-256-verified artifacts pass all sixteen setter
  layout and eight first-attribute audits, all 84 retained metadata profiles,
  and all sixteen Rust/C policy-emission images against their Linux-created
  input bytes. All 32 ordinary policy observations preserve ctime. Three fresh
  CI unit contexts retain all 1,129 prior occurrences and add eight (1,137
  passed), executing all 169 controls. Exact full CI `39391995709` fails in
  both native kernel and whole guest on the backported casefold-symlink fixture:
  its absolute target escapes the guest mount, and failed `stat` output was
  masked by `printf`. Both logs show the same empty-inode parse failure; this
  is not an output-budget failure. The fixture now uses a relative target and
  Python stat with propagated failure. A focused five-test symlink step keeps
  its own evidence artifact. It compiles locally; fresh exact-source execution
  and full-suite retention remain required before setter-family closure.
- Same-value setter repair now passes 1,137 native units, retaining all 1,129
  preceding successful occurrences and adding eight policy controls. Five of
  those controls failed before repair; the three guard/cleanup controls passed.
  The independent reference-only job at `aa266f3` passed and its verified saved
  bytes confirm all 32 ordinary inline/empty/external/shared timestamp answers.
  Four separate constructed duplicate diagnostics retain two copies with
  unchanged inode bytes; they are not healthy-image qualification. The private
  planner now returns no plan only for one validated identical ordinary value,
  retaining operation/refusal gates, both-location validation, cross-location
  duplicate cleanup, missing-feature publication and zero-extra initialization.
  First-attribute emission now advances the clock between identical sets, and
  the expanded matrix requires unchanged Rust/C image bytes. Fresh exact repair
  full/focused CI and standalone family review are pending. Casefold writes
  remain blocked; Stage 5 remains not yet quantified. See the repair section.
- The preceding native-budget repair's exact full run `39384659463` now
  completes: x86_64 passes, and all 1,951 guest tests pass with the one existing
  ignored fuzz gate, but its wrapper fails at 193,017 bytes against 193,000.
  SHA-256-verified native/guest archives retain all 1,951 preceding occurrences.
  Replaying the unchanged 3,507-line guest log reproduces exit 65 at 193,000 and
  passes at the measured 196,500-byte repair. The 3,500-byte increase tracks
  3,381 bytes of growth from the pre-setter baseline, restoring 3,483 bytes of
  headroom (baseline: 3,364). Line cap, command, 1,060 floor, filters and full
  output remain unchanged. This replay is not fresh Linux execution; later
  backport/policy source needs its own results and budget measurements.
- The initial reference-only job at `290d6db` failed before recording policy:
  Linux naturally deduplicated three identical external values, while the
  retained explicit-share constructor requires an exclusive source (actual
  refcount 3, expected 1). The new shared-source value is distinct from the
  other inputs, retaining that strict constructor and every validation check.
  Fresh reference execution remains required; no policy answer is inferred
  from the failed fixture setup.
- Same-value policy reference extension is prepared independently of production
  repair. A dedicated focused Linux job records eight block-size/inode-size/
  journal profiles for inline, empty, external and shared values, plus four
  explicitly constructed duplicate-name diagnostics. It saves images, reports
  and the tier log in `setxattr-policy-reference`, using the existing setter
  output envelope and an unchanged test floor of one. The reference test compiles
  locally; this host cannot execute its Linux VM. Actual answers and the measured
  output must be inspected before extending the repair beyond the already
  reproduced inline case. No production setter change or family closure is
  credited at this reference-only checkpoint; all write gates remain closed.
- Setter evidence follow-up at `b6ccad6`: the SHA-256-verified focused artifact
  for earlier setter source `9035b80` passes read-only audits of sixteen setter
  layouts, eight fresh first-attribute profiles and all 84 retained metadata
  profiles. However, all eight Linux same-value observations preserve ctime,
  while two executed advancing-clock Rust/C probes show this driver changes it.
  Family closure is blocked. The exact red controls are saved in
  `docs/CASEFOLD-SETXATTR-SAME-VALUE-RED.patch`; active source is restored exactly.
  Fresh backport standalone-unit CI retains all 1,122 preceding occurrences,
  adds seven (1,129 passed, zero failed/ignored) and executes all 161 controls.
  Backport full/focused and preceding budget-repair full CI remain pending.
  Next: inspect their actual outcomes, qualify same-value policy branches and
  repair the reproduced mismatch before closing setter review. Stage 5 remains
  not yet quantified; all casefold write gates remain closed. See the evidence
  follow-up and review below; earlier-source artifacts do not qualify backports.
- Upstream compatibility backports: `4122d20` reserves the symlink target's
  terminating byte (upstream `45bed01` / #561); `d38cb90` follows intermediate
  and parent-directory symlinks while retaining this branch's frozen casefold
  matcher (upstream `4db7413` / #558). CI chore downloads now retry transient
  failures with the existing checksum verification (upstream `7bb33f6` / #500).
  Native validation passes 1,129 tests, retaining all 1,122 preceding occurrences
  and all 161 guard controls, with seven added tests and zero failed/ignored.
  Full lint passes. Five new independent Linux reference tests compile but have
  not executed on this no-KVM host; fresh exact-source full/focused CI is pending.
  This supports Stages 3–4 and shared Stage 4–5 prerequisites; it does not qualify
  new casefold writes or close the setter-family review. See the backport section
  below before continuing. Dependency pins, siblings, locks and write gates stay
  unchanged. Next: inspect fresh CI and actual budgets, then resume the outstanding
  setter Linux image/metadata-policy audit and standalone review.
- Setter CI budget repair: actual SHA-256-verified source/baseline artifacts
  retain all 1,923 preceding full-suite occurrences and add 28 in both native
  and guest suites (1,951 passed / zero failed / one existing ignored). Three
  unit contexts retain all 1,095 and add 27; all 161 guard/operation/parser/
  fixture controls execute in all five contexts. The exact native log reproduces
  exit 65 at 183,600 bytes under 183,000 and passes unchanged under the measured
  186,000-byte repair. Line cap, floor, filters, application/tests/pins/locks and
  write gates are unchanged. Fresh full CI for the repair is pending; Linux
  image/metadata-policy audit and standalone setter-family closure remain open.
  See the repair/evidence section below. Stage 5 stays not yet quantified.
- Cloud startup/access follow-up: the user reports three consecutive regular
  chats receiving `403 Cloudflare egress requires HTTP inspection` while using
  this selected environment. The editor runtime can download SHA-256-verified
  artifacts from the three reported storage hosts and individual Actions logs;
  **regular-chat recovery is not verified**. `AGENTS.md` now requires each chat's
  own two-message access/permission preflight and links [CLOUD-STARTUP.md](CLOUD-STARTUP.md)
  plus a reproducible pinned setup script. Editor access is not a repair claim.
  The first message requests missing access and reads the current handoff; the
  user's second message triggers fresh verification, a readiness/next-step report
  and a wait for explicit implementation confirmation. The user prefers a
  separate editable environment per chat; unavailable editing must be reported.
  The complete known cloud access inventory is now listed upfront in `AGENTS.md`,
  including the preset/custom routes, all source repositories, fork workflow
  capabilities, writable roots and setup tools. Complete and verify that bundle
  in the first setup; the second message rechecks it, with no known access deferred
  to a later stage. This documents requirements; it does not repair the cloud proxy.
  The downloaded x86_64 log measures the existing full-CI failure at 183,600
  bytes against a 183,000-byte native-suite budget; that access follow-up made
  no repair. The budget repair above now completes both occurrence comparisons;
  complete image/metadata-policy and family review remain open.
  The new repeatable setup script passes locked CLI build and the existing
  1,122-test unit tier at source `eafd3b1` (1,370 lines / 95,221 bytes), with
  provenance/shared-agent/whitespace checks passing. This validates the editor's
  local setup, not regular-chat access, a fresh snapshot restore or setter-family
  qualification. Application source, dependencies, locks and write gates are unchanged.
- Latest setter CI investigation: source `9035b80` passed focused Linux CI
  `39348758499`, but full CI `39348758428` failed in x86_64 `chore test` with
  an actual exit-65 annotation. The whole-guest suite and other jobs passed.
  Actual logs/artifacts were denied in that investigation; the editor follow-up
  above now measures its failing tier; the budget repair completes both full-suite
  occurrence comparisons. No image/policy audit or family closure is credited.
  At that investigation no cap or
  implementation was changed without the missing evidence. Establish access in
  the current chat using the preflight above, then make only a demonstrated repair.
  See the investigation section below. Stage 5 remains not yet quantified and
  all casefold write gates remain closed.
- Latest guarded public setter implementation: the saved patch was actually
  rerun (one pass / fourteen failures), then its fifteen controls were promoted
  against a setter-specific whole-plan boundary. Twenty-seven new native
  controls pass; the complete unit comparison retains all 1,095 baseline
  occurrences and adds exactly 27. Rust/C exclusion and failure retirement now
  cover all six metadata families. A new exact Linux setter matrix and separate
  fresh-image first-attribute reference/emission cases are committed for CI;
  exact full/focused results, actual artifact/policy audit and standalone family
  review remain pending. The prior audit's migration task is implemented locally,
  not independently qualified. Stage 5 remains not yet quantified and casefold
  writes remain blocked. See the implementation section below for measured
  output-budget handling and current access/setup limits.
- Repository: [Shinomorisama/rust-fs-ext4](https://github.com/Shinomorisama/rust-fs-ext4), branch `casefold-rw`.
- Latest setter design audit: `b6190fd` passes exact full CI. Public setxattr
  staging/marker review and actual red preparation are complete. The saved
  `docs/CASEFOLD-SETXATTR-RED.patch` reproduces one pass / fourteen failures:
  ten public guard/refusal/panic/inline-metadata findings and four explicitly
  constructed feature/counter/marker ordering diagnostics. At that audit all
  active source was restored and no setter was migrated. The implementation
  above now supersedes its migration task; independent Linux first-inline
  metadata/feature qualification and family review remain pending. Stage 5 remains not yet quantified; all
  casefold writes remain blocked.
- Latest removal-family review: repaired source
  `396488818d774de1852b8872cce5ea0350ec5122` passes exact full/focused CI and
  standalone implementation/evidence review within the existing caller-exclusion
  contract. Both whole-suite paths retain all 1,895 preceding test occurrences
  and add exactly 28, reporting 1,923 passed / zero failed / one pre-existing
  ignored slow fuzz case. All 134 guard/family/parser/fixture controls execute
  across five CI contexts. All sixteen removal and 68 retained metadata profiles
  were audited. The mixed zero-extra layout and broader ignored-fuzz gate remain
  unqualified. The setter design audit and executed red preparation above now
  supersede that review's next task. Stage 5 remains not yet quantified;
  casefold writes stay blocked.
- Latest removal evidence repair: source `499c1b4`'s focused Linux workflow
  failed on the constructed zero-extra inode's timestamp comparison after two
  legacy profiles completed. Its undeclared extension storage still contained
  old timestamp words. A cfg(test)-only fixture constructor now clears that
  storage, refuses inline-attribute erasure and restamps the inode checksum via
  the caller. Strict Linux checks and all sixteen profiles remain; failed
  readback now retains exact timestamp diagnostics and partial observations.
  The original mixed-layout case is unqualified. Fresh local validation is
  recorded below; repaired exact full/focused CI, artifact review and standalone
  removal-family review now pass as recorded above. Stage 5 is not yet quantified
  and casefold writes stay gated.
- Latest public removal implementation: existing Rust string/byte and C
  path removexattr now share the metadata guard and a private whole-buffer
  boundary that preserves planned counters and the dirty marker. The saved
  failures were rerun before implementation and promoted into passing controls.
  Native validation and a sixteen-profile Linux/e2fsprogs matrix are recorded
  below. Git/helper read access verifies remote freshness and the existing API
  binding works with an explicit Bearer header. Exact repaired-source full/focused
  CI and standalone removal-family review now pass within the caller contract.
  Stage 5 remains not yet quantified; all casefold write gates stay closed.
- Latest xattr-removal design audit: traced inline/external/duplicate/shared/
  free branches and post-commit allocation publication. Five public guard/marker
  controls and four explicitly constructed pre-marker-buffer controls executed
  red; one positive legacy control passed all four block-size/journal profiles.
  Saved self-contained `docs/CASEFOLD-XATTR-REMOVAL-RED.patch`; no failing tests
  remain active and all source hashes are restored. The reviewed next boundary
  preserves planned primary-superblock edits while clearing its clean bit before
  guarded marker/complete-commit effects. Public removal migration is now
  implemented locally above; independent evidence and review remain pending. Stage 5 remains not yet
  quantified; casefold writes remain blocked.
- Latest public flag review: source `4d4df81` passed exact full/focused CI.
  Audited all sixteen flag and 52 retained metadata profiles: 96 flag phase/inode
  answers per independent report stream and all 136 final inode records agree.
  All 107 distinct guard/family/parser controls execute in native and guest
  contexts. No blocker found within the existing caller-exclusion contract;
  the ignored slow fuzz test remains a broader qualification gate. The selected
  multi-block xattr-removal staging/marker design audit and executed red
  preparation are now complete; public removal is implemented locally above,
  pending publication/independent CI/review. Stage 5 stays not yet quantified and casefold writes remain blocked.
- Latest public flag implementation: existing Rust string/byte and C path
  inode-flag updates now share the reviewed metadata mount context and failure
  bridge. Four actual bypass/refused-marker controls executed red and now pass;
  sixteen new controls retain prior families. A new exact sixteen-profile Linux
  flag oracle now passes exact source CI and standalone family/evidence review
  within the caller contract recorded below.
  Stage 5 stays not yet quantified and every casefold write gate remains blocked.
- Latest public timestamp review: repaired source `f8649b7` passed exact full/
  focused CI. All 24 timestamp profiles and 28 retained metadata profiles were
  audited against actual artifacts; 96 timestamp phase/inode Linux answers and
  104 final inode records agree. All 91 distinct guard/family/parser controls
  executed across native and guest contexts. No blocker found within the stated
  caller-exclusion contract; the pre-existing ignored slow fuzz case remains a
  broader qualification gate. Its selected public inode-flag family is now
  implemented and reviewed above; the original bounded contract remains below. Stage 5 remains
  not yet quantified and every casefold write gate remains blocked.
- Latest timestamp CI repair: source `5dd65f9` failed exact full/focused CI
  on the short-extra-12 debugfs timestamp display. The captured VM report now
  has a native red-to-green control; normal/legacy display controls retain
  strict checks. Linux now checks full explicit timestamps before sentinel
  updates, as well as final values. Production writes and all gates are
  unchanged. Repaired exact-commit full/focused CI and standalone family/evidence
  review now pass as recorded above and below.
- Latest public timestamp implementation: Rust string/byte/inode `utimens` and
  existing C path/inode delegates now share the reviewed chmod/chown mount
  context. Four controls executed red before the change and pass afterward,
  including the both-omitted call that previously wrote a clean-mount marker.
  Nineteen new controls preserve sentinel, precision, validation and cross-family
  failure behavior. A new exact 24-profile Linux timestamp oracle retains all
  three existing metadata oracles and every write gate. Local validation and
  independent CI and standalone timestamp-family review now pass within the
  caller contract recorded below. Stage 5 remains not yet quantified.
- Latest public chown review: repaired source `198baba` passed exact full/focused
  CI. Downloaded and verified the actual artifact; all sixteen owner profiles
  plus twelve retained chmod profiles (56 inode records) agree with debugfs,
  Linux and the read-only byte audit. All sixteen chown, two parser and 36 chmod
  controls execute in native and guest-library contexts. No blocker found within
  the stated caller-exclusion contract; the existing ignored deep-fuzz test
  remains a broader qualification gate. Its selected timestamp family is now
  implemented above with actual red-to-green controls and a new exact Linux
  oracle; independent timestamp evidence and standalone review now pass above. See
  the bounded contract below. Stage 5 is not yet quantified and
  every public casefold write gate remains blocked.
- Latest public chown oracle repair: initial source `2cedf28` failed full/focused
  CI because the new report parser rejected debugfs's signed decimal display of
  a high-bit group ID. A native control reproduced that failure before the
  test-only correction; signed/unsigned and malformed/overflow controls now pass.
  Generation remains unsigned and production writes are unchanged. Fresh local
  validation and the remaining independent checks are recorded below;
  the standalone family review is complete below within its stated scope.
- Latest public chown implementation: Rust string/byte/inode owner/group changes
  and the existing C delegates now share chmod's mount context and sticky failure
  bridge. Three controls executed red before routing and green afterward; sixteen
  new controls cover both families, metadata, callbacks, refusals and uncertainty.
  Local/full validation, sixteen-profile exact Linux metadata evidence and the
  standalone family review are complete below within the stated scope; casefold writes remain
  blocked and Stage 5 is not yet quantified.
- Public chmod review HEAD `ab8129a` now passed its CI follow-up: original full run
  [38991380517](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38991380517)
  failed on guest VM/SSH loss (exit 255), with the cause of that loss unknown.
  It did not finish the suite. Other required jobs passed, including the x86
  oracle suite. Attempt 2 at the same commit passed, including the complete guest
  suite (1,836 passed, zero failed, one existing ignored fuzz test). The cause of
  the original VM loss remains unknown; no transport fix is claimed. See below.
- Latest public chmod review: `c47b328` passed exact-commit full/focused CI.
  All eighteen public and eighteen retained private controls executed in native
  and guest-library contexts. Audited all eight public and four retained private
  Linux images/reports (24 emitted inode records); no blocker found within the
  stated caller-exclusion contract. Public chmod has no whole-API concurrency
  or ownership-readiness claim. The whole guest suite still ignored the existing
  slow deep fuzz case; broader qualification remains gated. Its selected owner/
  group migration and review are now complete above, with cross-family controls
  and exact Linux metadata evidence. Timestamp dispatch is now implemented above
  with independent evidence and standalone review pending. Stage 5 is
  not yet quantified; every casefold write gate remains blocked.
- Latest public chmod integration: public Rust string/byte/inode chmod and its
  existing C delegates now share the reviewed mount guard. Acquisition and volume
  verdicts precede arbitrary inode conversion; path dispatch never enters twice.
  Public signatures/C error handling, metadata semantics, sticky failure and all
  casefold write refusals remain. Three public bypass/conversion controls executed
  red before routing and now pass. All 1,012 local units pass, retaining all 994
  baseline occurrences and adding eighteen public controls; lint and the locked
  release CLI build/install pass. New eight-profile exact public Rust/C VM
  evidence and standalone migration review now pass within the stated scope.
  The subsequent chown migration and review are complete above.
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

## Inline-source guest whole-suite retention and measured byte-limit repair — 2026-10-28

Exact [full run 40785791674](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40785791674)
at **`6c71cbea0edeae6e854e692ebbb345d4a12d048b`** completes guest job
**123159536037** with an underlying successful suite and wrapper exit **65**.
API artifact **12862020202**, `tier-logs-suite-in-vm`, is bound to this run/head;
its advertised SHA-256 is
`221c375e32cc69fe0b995ecd01658e6bd4d034517c2091451d44b021dec02ced`.
The archive digest and every ZIP CRC pass. Its complete `vm.log` measures
**4,095 lines / 248,844 bytes**, exceeding only the existing 248,000-byte ceiling.
The actual job log explicitly reports the successful wrapped command and byte
overflow; this is not a driver-test failure, timeout or missing fixture.

The full success-name **multiset**, compared with exact baseline `43280ec`
full **40761780868**, retains all **2,488** occurrences and adds exactly the
eight inline native controls and one exact inline Linux test: **2,497 passed**,
zero failed, all **670 controls once**. The unchanged ignored-test multiset
contains the pre-existing slow fuzz case; it is not counted as passed or qualified.
Together with completed standalone/ARM unit evidence, three of the five control
contexts are now observed; native unit/whole retention is still pending.

The actual complete log is replayed through `scripts/tier.sh` with no filtering
or truncation. The old **4,120 / 248,000** limits reproduce **exit 65**; identical
bytes at **4,120 / 251,000** pass with **exit 0**. SHA-256 and full byte equality
are checked for both replay logs. The existing executed-test floor **1,060**
passes on the successful local replay with all 2,497 names. This does **not**
credit the failed CI invocation's actual floor: its wrapper exits before that
step. The byte-only guest cap adjustment provides 2,156 bytes of measured
headroom; line cap, floor, filters, entire output, test names and ignored policy
remain unchanged. No native limit is changed on prediction alone.

Local receipts are ignored `tmp/inline-link-audit/guest-retention-and-cap-replay.json`
and `guest-actual.log`, with both exact replay logs in `tmp/logs/`. Source-bound
API metadata, the complete verified archive and actual job log are retained in
`tmp/growth-audit/40785791674/`. The existing scoped inline source review finds
no new blocker in the tested healthy ordinary profile; it does not close the
broader public-link review or qualify allocator exhaustion as Linux ENOSPC.

Fresh local full units retain the exact **1,647-occurrence / 670-control**
baseline with no regressions, and full lint passes. The existing output-budget
shell contract passes separately through the native runner and tier wrapper.
A broader local `test:scripts` attempt first fails because its scratch-cleanup
test lacks two generated fixtures. Both are restored from the exact-source,
digest/CRC-verified CI fixture artifact, and that check then passes. The rerun
passes fourteen shell files before failing the staticlib-freshness first build:
the native runner's temporary core path is copied into that test's isolated
tree, where `../.cloud-setup/pinned-core-0.3.0` does not exist. Its five later
files are unrun. This is a local runner/copy-layout limitation, not a passing
shell tier or an observed product regression; no test or dependency is changed
to conceal it. CI's ordinary pinned-sibling layout must provide the complete
shell-tier and installed-CLI result. The failure logs are retained alongside
the local repair receipts. The unavailable `dev-loop` skill is not claimed as
executed; the baseline/reproduction/replay/lint review steps are recorded here.

**Next bounded task:** inspect the still-running native job **123159535998**
and audit its complete occurrence/control/tier/floor/trailing evidence; reproduce
any actual wrapper failure before changing that limit. Then verify the resulting
repair source's complete green CI and all five contexts, preserving exact
baseline multisets. The previous fourteen focused jobs and 864-image link audit
remain evidence for `6c71cbe`, not a substitute for the new full gate. Broader
growth/counter-wrap/scoped review, ownership/admission/seal/lifecycle/recovery/
fuzz/write and Windows/card/SteamOS gates remain. Stages 4–5 remain **not yet
quantified**, and casefold writes remain blocked.

## Inline-parent exact Linux saved-byte audit and pending full retention — 2026-10-28

Audited immutable source **`6c71cbea0edeae6e854e692ebbb345d4a12d048b`**.
Exact [focused run 40785791695](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40785791695)
passes **all fourteen jobs**, including inline conversion **123158524255**,
linear growth **123158524525** and repeated allocation-free **123158524667**.
All three actual individual logs show wrapper success and **one executed test
against floor one**. Inline output is **957 lines / 129,769 bytes** within the
existing new-job ceiling; growth is **2,157 / 295,955**, allocation-free
**2,141 / 288,067**. No budget, filter, floor, test or production change is needed.
The other eleven focused jobs are green; their saved-image audits are not
repeated or newly credited by this link-only audit.

| Exact-source archive | Artifact ID | Advertised SHA-256 |
|---|---|---|
| `link-inline-conversion` | `12858898992` | `111e88308cda4bb2fe38cf845bf7e85989957153c6598ed9a8bc308bead38665` |
| `link-linear-growth` | `12859028963` | `1dfed617155ae434781421a16160462080ce4c8c90cf3540147ac30a8c564f25` |
| `link-emission` | `12859428161` | `9e82a69cdeb0e59709e0418803f7fdf388ba2c04f0cdb9c2104fe462813ce95c` |

Complete archives pass advertised SHA-256 and every ZIP CRC. API workflow/run/
head associations and retained source-commit files match the exact source.
All **24 inline** and **thirteen files in each linear** source-hash inventories
match that committed tree and the inspected current bytes. Audit tools stream
all **864 images** from the archives; no image is mounted and no host oracle is
invoked. Saved Python literals are decoded through a restricted AST, never run.

### Actual inline observations

Every **Rust/C × 1/4 KiB × direct/journal × head-only/populated-continuation**
layout has a Linux-created 256-byte checksummed input. Independent raw decoding
proves the INLINE_DATA/no-extent parent, inode-body header and `system.data`,
actual continuation size, `user.keep` and distinct external EA, implicit parent
number and physical same-table source/parent locations. Input Linux stat rows
match the actual saved inode fields. The fitting record has room in the intended
area; a 264-byte record cannot fit either the 56-byte head or any possible
continuation in this inode profile. Inline bytes are never decoded as a block map.

All **48 driver links / sixteen conversions / 160 snapshots** pass. The first
link stays inline; for continuation inputs its head bytes remain unchanged.
Conversion removes only `system.data`, retains the user attribute, preserves
other inode flags while setting the extent map, maps exactly logical block zero
and changes size to one block. It adds exactly block-size/512 sectors over the
EA-inclusive initial count. Post-conversion fit retains the same mapping.
Primary SB and exactly one group's free count debit by one, exactly the new
block's zero bit is set, inode bitmaps remain identical, and complete descriptors
and all unrelated bitmap bits remain stable. Full inode tables are compared with
only the independently audited source/parent records permitted to change.

The source retains identity, owner/mode, generation, payload and all metadata
except one link/ctime/checksum update per successful call. Parent mtime/ctime
match the selected Runtime stamp, change version advances once, and unrelated
parent fields, protected inodes and complete external-EA bytes remain unchanged.
All existing directory records retain name/inode/type, new aliases record the
regular-file type, and dot/parent identity survives conversion. Raw payloads
match known input bytes for every regular file and alias. Applicable SB, group,
initialized-bitmap, inode, directory-tail and EA CRC32C checks pass; uninitialized
bitmap flags/bytes are preserved rather than assigned a fictitious checksum.

Dirty and closed inode records match. Closed images have clean state, no replay
flag and clean journal start when journaled. Every cold-copy inode matches its
driver counterpart byte-for-byte; only documented superblock mount telemetry
changes. The complete raw inventory, directory membership, free counters and
all expected literals agree with **540 actual Linux JSON rows**, including
**twelve requested C snapshots**. All **608 debugfs metadata reports** and
**112 judged five-pass non-repairing e2fsck reports** are checked. This validates
the declared healthy ordinary-inline profile; no malformed-map, concurrency or
power-loss qualification is inferred.

### Repeated linear evidence and incomplete full gate

Both independent repeated matrices retain their actual **sixteen layouts / 112
links / 352 images / 1,536 debugfs / 240 e2fsck** results each. Raw allocation,
full table/bitmap/EA/checksum/metadata and dirty/closed/cold checks pass. Growth
retains **1,932 Linux rows / 28 C snapshots**; allocation-free retains **1,928 /
24**. These are this source's repetitions, not borrowed older archives.

Exact [full run 40785791674](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40785791674)
is still running at this checkpoint's observation: native whole job
**123159535998** is in `chore test`, guest whole **123159536037** in `chore
test:vm`. Eight other jobs have passed; ci-ok and complete whole-suite retention
are **not credited**. Available source/digest/CRC-verified tier archives retain
every earlier unit occurrence plus exactly the eight new names in standalone
and ARM unit contexts (**1,647**), all **670 controls once**, zero unit ignored,
**527 ARM image / five wasm / 159 standalone CLI** successes. ARM retains the
same one ignored slow fuzz case as the baseline; it remains unqualified.
Actual completed job logs are retained alongside the archives. The expected
**2,497** whole successes, remaining three control contexts, whole wrappers/
floors and native trailing shell/installed-CLI checks remain **pending**, not
inferred from early artifacts. Repair any actual failure only after inspecting
its underlying command and complete output.

Complete local receipts and tools are ignored `tmp/inline-link-audit/`, including
`inline-images.json`, both repeated audit receipts, `partial-retention.json`,
`images.log`, `repeated-linear.log` and `tools/`. API metadata, actual job logs
and complete archives live in `tmp/growth-audit/<run>/`. The source-bound Actions
runs/artifact identifiers above permit independent retrieval. This is an audit/
ledger checkpoint: production writers, test bytes, workflows, budgets, pins,
manifest/lock and sibling contents are unchanged. Fresh full local lint passes,
including formatting, workspace/all-targets/CLI clippy, provenance/core/family/
CI-gate and eight frozen-table checks. No new local driver/oracle run or
independent dispatch is substituted for the recorded CI evidence.

**Next bounded task:** finish source `6c71cbe` full native/guest CI and exact
multiset/control/wrapper/floor/trailing retention, repairing only reproduced
failures; do not start broader mapping growth or declare the full inline gate
closed while that result is outstanding. Scoped implementation/evidence review,
allocator exhaustion error classification, indexed/deeper/uninitialized-group/
counter-wrap growth and ownership/admission/seal/lifecycle/recovery/fuzz remain
open. Stages 4–5 stay **not yet quantified**; casefold writes stay blocked and
Windows/card/SteamOS qualification remains later work.

## Inline-parent public-link native controls and independent test preparation — 2026-10-28

The planned conversion tests are now implemented. Production `src/fs.rs`,
`src/capi.rs`, `src/inline_mut.rs`, `src/inline_data.rs`, `src/xattr.rs`, all
shared commit/allocation helpers, manifest/lock, sibling pins and write gates
are unchanged. Changes are the new native/host/guest tests, two test-module
registrations, focused-job/trigger/source-retention configuration and these ledgers.

### Native evidence actually executed

`src/fs/guarded_chmod/tests/link_inline.rs`, under `link_calls::inline_conversion`,
adds **eight native tests**. The synthetic fixture explicitly finds source and
parent in the same inode-table block, preserves ordinary peers and external EAs,
and supplies either head-only entries or a populated continuation. A declared
plain journal transfers allocated storage to reserved inode 8 using the existing
native-fixture pattern; it exercises real transaction code without claiming an
e2fsck-clean/Linux-created input.

The positive test executes **forty layouts × three links (120 successes)**:
all five Rust/C forms × 1/4 KiB × direct/journal × head/continuation. It keeps
inline state on the fitting link, then requires exact one-block conversion and
no additional post-conversion allocation. It checks source identity/count/
payload/metadata, parent flag/size/sector/version changes, inode bitmap and
single block-bitmap bit, exact SB/group debit, old entries/types through lookup,
user attribute/EA values and bytes, and unrelated bytes in the shared table.

The error controls execute **52 measured planning-read errors**, **176 write
errors**, **56 flush errors**, **56 flush unwinds**, **four C attribute-readback
errors**, and **four post-commit publication unwinds**. The effect sweeps cover
Rust/C handle forms, both block sizes and direct/journal, on clean/already-dirty
mounts; they measure the converting operation first and prove every injected
position was reached. All uncertain effects retire the mount. C readback errors
preserve exactly one committed alias/debit and leave the caller's attribute
sentinels untouched. Publication unwinds require the committed mapped directory,
source count and debit in raw bytes, then retirement with zero later callbacks.
Planning errors and eight malformed/exhausted-input cases require unchanged
bytes/health/epoch, no writes/flushes and zero Runtime calls.

Three test-fixture corrections are recorded without claiming a production fix:

- The first C readback probe used a cached mount, so its injected device-read
  position was never reached (the call returned success). The probe now uses
  the existing readback harness's zero-cache mount consistently; the actual
  post-commit read fails, the output stays untouched and the alias remains.
- A strict ENOSPC assertion failed with the shared allocator's existing
  `Corrupt("no group has a contiguous free run of this size")`. The full-input
  fixture now also zeros the SB/group free counters and refreshes bitmap/BGD
  checksums, rather than supplying inconsistent counters. It asserts that exact
  existing conservative refusal and unchanged state. Linux-compatible exhaustion
  error classification is **not qualified**, and no global allocator policy is
  changed to fit this test.
- The first publication probe correctly poisoned publication and reached its
  unwind, then tried a lookup through the deliberately poisoned retired mount.
  That inspection could itself panic. The corrected inspector decodes actual
  source/parent/directory bytes and validates their checksums without using that
  mount's metadata APIs, then verifies retirement. The emitted-alias/debit and
  no-later-callback assertions remain; no production assertion is disabled.

Complete receipts, including those earlier diagnostic failures, are ignored
`tmp/inline-link-native/` and `tmp/logs/inline-native.log`. The fresh focused
run reports **eight passed / zero failed or ignored**, 440 lines / 28,871 bytes,
with the executed-test floor eight also checked explicitly.
Fresh full units pass **1,647 tests**, with the full 1,639-occurrence baseline
retained and exactly eight new names; all **670 controls occur once**. No unit
fails or is ignored. Full lint passes (formatting, workspace/all-targets/CLI
clippy, core/family/CI-gate/provenance and eight frozen-table checks). Targeted
workflow pin/VM-order/no-root checks pass. The exact host-test listing contains
one test, not an executed oracle result. Manifest, lock and siblings remain
unchanged; no old failing verdict substitutes for these final outcomes.

### Independent test and focused workflow

`needs_host/link_inline.rs` is registered as
`link_emission::inline_conversion::linux_checks_public_link_inline_conversion`;
its separate `link_inline_guest.py` creates inputs and checks disposable cold
copies solely through the prescribed guest. It does not modify the existing
linear runner or builder: only a new test-module registration is added there.

The **planned, not credited** matrix is Rust/C × 1/4 KiB × direct/journal ×
head-only/populated continuation, with 256-byte checksummed, FILETYPE, extent/
inline_data inputs. Linux creates a same-table parent/source, known data/owner/
times and inode-body/external attributes. Continuation population is bounded;
the test fails if EA setup or Linux filling has already converted the parent.
Raw input checks require inline flags/size, valid inode-body attrs/system.data,
actual shared-table positions, a fitting short alias, and refusal of a 264-byte
record even after maximum possible continuation expansion.

Each input executes fit, 255-byte-name conversion and post-conversion fit:
**48 planned driver links / sixteen conversions / 160 snapshots**. Every phase
first checks four pre-effect refusals (duplicate, dot, overlength and directory
source) with unchanged device bytes and no clock call. Actual first-converting
forms cycle all three Rust/two C APIs across layouts, not their full Cartesian
product. Old namespace entries and relevant metadata/EA bytes must remain;
exact target count/ctime and parent mtime/ctime/version changes use one Runtime
call. Only conversion may debit one primary-SB/group block and flip its one
previously-free bitmap bit; inode allocation/accounting remains unchanged.
Closed checks require clean state, no recovery flag and a clean journal where
present. Complete debugfs, judged non-repairing e2fsck, source literals,
allocator identities and actual dirty/closed/cold bytes are saved. The guest
verifies all recorded metadata, complete directory membership, payload/EA values,
source/alias identity, requested C attributes and free-space counts. Linux copies
are disposable; driver images must remain unchanged by their inspection.

The new `link-inline-conversion` job pins Rust 1.95.0 and both sibling commits,
uses the existing harness, selects the exact test and requires **floor one**.
It retains **24 source hashes**, including both new test files, native controls,
inline/xattr/directory/checksum/allocation dependencies and workflow/manifest/lock.
The push-trigger list now includes the previously omitted top-level inline and
related helper modules. All thirteen prior focused jobs, including both link
matrices, remain; the two existing linear job bodies are byte-identical.
The initial 4,200-line / 340,000-byte cap is the existing larger linear matrix's
ceiling (2,157 lines / 295,955 bytes observed). Actual new-job output must still
be measured. No existing tier cap, floor, ignored-case policy or filter changes.

Compile and a one-name listing are local preparation, not oracle execution.
The cloud host never mounts images or invokes e2fsprogs. Focused Linux and full
CI at the resulting source are pending until actually observed; source-bound
archive and complete saved-byte/retention audits remain the next checkpoint.
The new host test plus eight native tests should add nine whole-suite occurrences
(expected **2,497**, versus 2,488); fresh units should be **1,647**, with all
**662 original controls plus eight new** retained in five contexts once CI runs.
Report actual results, including any output-budget failure, rather than assuming
those counts or borrowing a preceding source's green result.

Previous plan source `f08db64b9333deb6c685dba0013c858d8458229d` now passes all
eleven jobs/ci-ok in [full run 40773436382](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40773436382).
This observation confirms that preceding documentation checkpoint only. It does
not qualify any new native/host test bytes or their planned Linux emissions.

**Next bounded task:** inspect the new exact inline-conversion and repeated
linear jobs, repair only demonstrated failures, then verify API source/digest/
CRC/all retained source hashes and audit all actual shapes, allocation, complete
metadata/EA/checksum and dirty/closed/cold evidence. Close exact full-suite
multiset/control/floor/trailing retention before crediting inline conversion.
The conservative exhaustion error classification, broader indexed/deeper/
uninitialized-group/counter-wrap growth, scoped review, ownership/admission/
seal/lifecycle/recovery/fuzz and Windows/card/SteamOS remain open. Keep casefold
writes blocked; stages 4–5 are **not yet quantified**. Both earlier link patches
remain historical/promoted and are not reapplied.

## Inline-parent public-link conversion contract and qualification plan — 2026-10-28

Reviewed `43280ec97c5bf40c567c36a69827f29849a4c9ae` without changing production,
tests, fixtures, dependency pins, workflows or write gates. This is a source
contract/test-plan checkpoint, not implemented inline-conversion qualification.

### Current path and evidence gap

The Rust string/byte/handle and C path/handle forms reach the guarded link
context in `src/fs/guarded_chmod.rs`, then `plan_link_at` in `src/fs.rs`.
That planner stages the source count, destination insertion, and final
source/parent metadata together. It rereads both inodes through the buffer
after insertion; source and parent can share an inode-table block. One Runtime
reading supplies source ctime and parent mtime/ctime; only the parent change
version advances. Optional C attributes describe the existing target under the
same context; a readback failure can follow one committed alias, not undo it.

`buffer_add_dir_entry_inplace` recognizes a buffered inline parent and calls
`buffer_add_inline_dir_entry`. `InlineDir::add` tries the 56-byte head and then
the `system.data` continuation. With no continuation, `expand` tries the maximum
space available alongside existing inode attributes. Only when neither can
hold the new entry does `buffer_convert_inline_dir` build a new directory block:

- Explicit `.`/`..` plus every old entry and the requested spelling/type must
  fit in one block; overflow returns the existing corruption error.
- `buffer_alloc_dir_block` plans exactly one block from the buffered allocation
  view, staging one bitmap bit plus BGD/SB debit in the same buffer.
- `inline_mut::strip` removes only inode-body `system.data`, clears INLINE_DATA
  and zeros the old inline bytes. `map_one_block` installs the flavor's block
  map. The selected first profile must prove an extent-capable input and a
  single depth-zero extent after conversion; it does not qualify other maps.
- Parent size becomes one block; sector count increases by block-size/512 over
  its actual input count, including any external EA. Existing inode-body user
  attributes and separate EA storage must remain intact. Checksums are restamped.
- The guarded whole-plan boundary preserves a present primary-SB debit and
  clears clean state before effect. Existing direct ordered flushes are not
  power-loss atomicity; journal success is not crash-recovery qualification.

Current native `link_calls::positive_inline_and_conversion` tests eight
synthetic parameter cases (two block sizes, Rust/C handle forms, fit/convert).
`real_inline_link_plan_distinguishes_fit_and_conversion` tests four parameter
cases, including absent-versus-present staged SB and exact block debit.
`inline_mut` tests head/continuation filling and stripping. They use self-built
inodes, not Linux-made conversion inputs. Existing generic write/flush sweeps
use a plain parent and the short name `new`; they do not measure the conversion
write set. `tests/kernel_inline_writes.rs` independently links a small name into
an inline directory and converts another directory by **create**, not by link.
It neither proves that this hard link caused conversion nor supplies this
profile's allocator/shared-table/C-readback matrix. Do not credit it as such.

### Bounded first independent profile

Use **Rust/C × 1/4 KiB × direct/journal × head-only/populated-continuation**,
with **256-byte inodes, FILETYPE, extents, inline_data and metadata_csum**:
sixteen proposed layouts. The inode-body attribute requirement excludes the
prior 128-byte-inode matrix; do not manufacture an impossible continuation or
silently substitute a block directory. Non-checksummed/other inode profiles,
source-kind growth, indexed/deeper maps, uninitialized allocation and counter
wrap stay separate. Check actual guest support/settings; never quietly skip.

Build the initial image and all directory/source data exclusively through the
prescribed guest. Add a separate opt-in builder; do not enable inline_data in
the existing shared setter/linear builder, whose four parent directories are
asserted to occupy one data block. Require feature bits, ordinary parent flags,
`i_extra_isize`, inode-body xattr header, `system.data`, inline size, saved `..`
parent number, raw record geometry and **no directory data block** immediately
before each driver sequence. Never interpret inline bytes as block addresses.

Linux should create the regular source, known payload/owner/mode/times and
protected peers. Preserve both small inode-body user attributes and a controlled
external EA where present. Any EA setup must be followed by an explicit inline
shape check. The continuation variant must have real Linux-created entries in
both areas and enough measured space for its fit control; do not hard-code a
filler count from the earlier linear fixture. Keep a bounded population loop
that fails if Linux converts too early or the desired shape is unavailable.

For each independent input, execute:

1. **Fit:** a small alias proven to fit the intended current inline area. Keep
   INLINE_DATA, `system.data` and every existing entry; allocate no block/inode,
   keep primary-SB/group/bitmap accounting unchanged and preserve peer/EAs.
2. **Convert:** a 255-byte alias needs a 264-byte record. Prove it fits neither
   current area nor the maximum possible continuation alongside retained attrs,
   while all old entries plus the alias and dot/checksum records fit one block.
   Require INLINE_DATA cleared, `system.data` absent, exactly logical block zero
   mapped once, no second block, size one block and exact sector increment.
   Preserve other flags/attributes and parent identity/generation/links.
   Debit exactly one free block in primary SB/group and set exactly its one
   previously clear bitmap bit; no inode/used-directory delta or unrelated
   descriptor, bitmap or inode-table change. Include actual same-table
   source/parent coverage, proven from raw locations rather than assumed.
3. **Post-conversion fit:** another alias measured to fit that block, with no
   further allocation and all prior aliases still resolving to the source.

Cycle all three Rust and both C entry forms through actual conversion across
the layouts; retain that this is not every API in every layout. Include the C
attribute and null-output alternatives without changing their existing contracts.
Use deterministic Runtime timestamps, one call per successful link, exact
source count/ctime and parent mtime/ctime/version changes. Keep source content,
identity, owner/mode/size/generation, unrelated inodes and user EA values/bytes.
Before each success, exercise applicable duplicate/dot/overlength/directory-source
refusals with unchanged device bytes, health/epoch and zero clock/effect calls.
Reuse measured path/handle error priorities, not a newly inferred POSIX policy.

The three phases plan **48 driver links / sixteen conversions** and input plus
three dirty/closed/cold snapshots per layout (**160 images**). These are planned
counts only. Retain per-phase raw shape/allocator/source identities and literals,
complete debugfs metadata and judged non-repairing e2fsck output. Require cold
Linux directory membership, dot/parent identity, alias inode/payload/EA agreement,
free-space counters and requested C attributes; keep Linux copies disposable.
Use direct-read `O_NOATIME` for metadata checks, with symlink handling unchanged.
Closed images must be clean, have no recovery flag and have an empty journal
start where journaled. Never mount or run e2fsprogs on this host.

### Native preparation, failure handling and evidence retention

Before the independent job, add focused native contract controls for fitting
without allocation, populated-continuation conversion, same-table merging,
complete source/parent/EA preservation and exact one-block accounting. Measure
conversion-specific write/flush positions before injecting faults; generic
plain-link positions are insufficient. Cover Rust/C, direct/journal, clean/
already-dirty mounts, post-effect retirement and C readback sentinels. Distinguish
pre-effect planning refusal (no device/clock/marker/publication effects) from
uncertain writes/flushes (retire the mount). Include malformed inline records,
missing/invalid `system.data`, genuine no-space and read failures at the relevant
planning boundaries. Reproduce any defect red, preserve the diagnostic/unchanged
state, then make only the needed correction. No such new red/fix is claimed now.

Add a separate exact host test, for example
`link_emission::inline_conversion::linux_checks_public_link_inline_conversion`,
under `src/fs/guarded_chmod/needs_host/link_inline.rs`, with a guest helper and
pinned `link-inline-conversion` job. Its selection must execute exactly one test,
with floor one and retained logs/images. Add the new Rust/guest files and
`src/inline_mut.rs`, `src/inline_data.rs` and `src/xattr.rs` to the retained
source-hash inventory alongside the current thirteen files. Include the shared
directory/checksum/allocation modules used by the final test in its inventory.
The present focused workflow's push paths omit the top-level inline modules;
add those conversion-specific trigger paths when adding the job so a later
helper fix actually runs the independent test. Keep source identity bound to
the complete commit, not only this selected file inventory. Any shared
runner/readback refactor must retain and rerun both existing linear matrices;
current `GROW` literals and one-block parent/listing assumptions are not inline
shape evidence. Measure complete output before selecting a justified budget;
keep original filters, floors and caps unchanged unless a real overflow is
reproduced from the complete log. A listing or compile is not execution.

Whole-suite retention must compare occurrence multisets and all 662 existing
controls, including duplicate names. Native counts remain 1,639 only until new
native tests are added; the new host test alone would add one to each 2,488
whole suite. Report actual additions and failures, not those planning counts.
Verify the exact emitted source, all retained source hashes, API artifact source,
advertised SHA-256 and every ZIP CRC. Then independently audit raw input/mapped
bytes, allocator/bitmap/table/EA/checksums, every dirty/closed/cold report and
complete full-suite/trailing retention before crediting inline conversion.

### Checks actually completed here

Previous [full run 40761780868](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40761780868)
at exact `43280ec` passes all eleven jobs and ci-ok. Its seven tier archives
are verified against API source, advertised digests and all CRCs. The same
multiset auditor retains **1,639 unit / 2,488 whole** successes, all **662
controls once in five contexts**, actual wrappers/floors, twenty trailing shell
checks and both 159-check installed-CLI contexts. The existing ignored slow fuzz
case remains unchanged and remains a qualification gate. This confirms the
preceding evidence commit; it does not supply the planned inline images.

Four existing native tests named above execute freshly through `scripts/tier.sh`
and `scripts/test.sh`, each with exact selection and **one executed test against
floor one**. Focused logs and receipts are ignored `tmp/inline-link-preflight/`
and `tmp/logs/inline-link-preflight-{0,1,2,3}.log`. Fresh full native units retain
every **1,639 occurrence / 662 controls**, zero failed/ignored, with the exact
pre-edit occurrence multiset preserved. Full lint passes, including formatting,
workspace/all-targets/CLI clippy, core/family/CI-gate/provenance checks and eight
frozen-table checks; manifest, lock and siblings are unchanged. No new test or production
change, fixture, independent dispatch, permission round or environment publication
is performed. The `dev-loop` skill package is unavailable in this environment;
no unavailable skill execution is claimed for this documentation/source review.

**Next bounded task:** implement the proposed profile/test/job after the native
contract preparation, run its exact independent Linux evidence, then audit its
saved bytes and full retention. Fix only reproduced defects, preserve both
linear matrices and do not expand to indexed/deeper/counter-wrap growth yet.
Stages 4–5 remain **not yet quantified**; this plan does not advance implementation
or qualify writes. All ownership/admission/seal/mapping/lifecycle/recovery/fuzz
and Windows/card/SteamOS limits remain; casefold writes stay blocked.

## Corrected linear-growth saved-byte and full-retention closure — 2026-10-28

Audited current `aaf5e2ad8924e80eaafac92eedc0bbd28f373d7a` against corrected
source `09cf8dcf61c63c72b4ae211624738e597eb8bd1c`. The thirteen recorded
production/test/workflow/manifest/lock hashes match both trees. No new driver,
test, dependency, workflow, budget or write-gate change is introduced here.
The earlier EA-sector failure/correction remains historical evidence below.

Exact [focused run 40744071010](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40744071010)
passes all thirteen jobs. Actual individual logs verify both exact test
selections, wrapper success and **one executed test against floor one**:
growth job **123013076874** and allocation-free job **123013076829**. The
CLI's job-log view returned empty output, so the individual REST log downloads
were used; a zero-byte CLI result was not credited as evidence.

| Saved archive | Artifact ID | Advertised SHA-256 |
|---|---|---|
| `link-linear-growth` | `12840692865` | `f77ddc443dfb52201a1b252a195fd38a9173a51bdc4dedb0d3fb5833fdeea798` |
| `link-emission` | `12840778248` | `780af18e75347cf75cb61aad3bf2967fa7faddf42660f90938bc2e909be06155` |

API run/source identity, complete archive SHA-256, every ZIP CRC and all
thirteen retained source hashes are verified. Read-only decoding streams every
image from the archives, without extracting 22 GiB of image bytes, mounting
anything or invoking host e2fsprogs. Saved Python literals are parsed as a
restricted AST, never executed. A standalone public-format decoder compares
raw superblock/group/inode/directory/EA bytes with debugfs, Linux JSON records
and the retained literals. It checks applicable superblock, descriptor,
initialized-bitmap, inode, directory-tail and external-EA CRC32C values.
Uninitialized bitmap flags/bytes remain preserved; no checksum is invented
for a bitmap whose descriptor marks it uninitialized. Legacy journal maps are
read only for the journal's first block; this is not deeper directory growth.

Growth passes **sixteen Rust/C × 1/4 KiB × 128/256-byte × direct/journal
layouts**, **sixteen first-link block allocations / 112 total links / 352
images**, **1,536 debugfs reports / 240 non-repairing five-pass e2fsck reports**,
**1,932 cold Linux rows / 28 requested C snapshots**. The source-bound passed
test also executes all **448 pre-effect refusal checks** with unchanged device
bytes and no clock call. All five public forms cycle through real first growth;
this is not the full API/layout Cartesian product, nor all-source-kind growth.

The Linux-filled parent has exactly one data block and a distinct external EA,
three/fifteen 255-byte fillers, no available 264-byte slot and the correct
EA-inclusive sector count. Its first link adds exactly one mapping, preserving
the first mapping and forbidding a third. Primary-SB/group free blocks debit by
one; all inode bitmaps remain identical and exactly the new block's bit changes.
Complete descriptors, later non-growing allocations and complete inode tables
are compared, with only audited source/parent inode records allowed to differ.
Source identity, payload, links, timestamps and parent version changes match
all retained metadata; unrelated peer/source fields and external EA bytes stay
unchanged. Directory membership, filler identities, raw/255-byte names, final
symlink targets and FIFO kind are retained. Dirty and closed inode records
match; closed images have valid clean state, no recovery flag and empty journal
start. Disposable Linux copies retain allocation/inode evidence while allowing
only documented mount telemetry and symlink read-atime changes. Their saved
cold metadata rows agree with actual driver images and requested C snapshots.
This is healthy-image evidence, not arbitrary corruption or crash qualification.

The repeated allocation-free archive independently retains its own **sixteen
layouts / 112 links / 352 images / 1,536 debugfs / 240 e2fsck** reports,
**1,928 cold rows / 24 C snapshots**, with no block or inode allocation.
No old source result substitutes for this corrected-source repetition.

Exact [full run 40744071049](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40744071049)
and setup-tip [full run 40748257814](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40748257814)
pass all eleven jobs and ci-ok. Their seven tier archives each are bound by API
source, advertised digest and every CRC. Compared as **multisets** against green
`2137e23` full **40649608984**, all old **1,639 unit / 2,487 whole** successes
remain, with exactly the new growth host test added once per whole suite
(**2,488**). All **662 operation/guard controls occur once in five contexts**.
Actual job logs retain all wrappers/floors and trailing **twenty native shell /
159 native installed-CLI / 159 standalone CLI** checks. ARM images retain 527,
oracles 225, kernel 36, third implementation nine and wasm five successes.
Static-library freshness includes its intentional stale-lock failure and passes
its enclosing negative test; that diagnostic is not a new repository failure.
The one existing ignored slow fuzz case remains unchanged and unqualified.
Raw native whole logs are **3,916 / 238,055** and **3,917 / 238,177** lines/bytes;
both guest logs are **4,083 / 247,379**, within unchanged caps. ANSI bytes count
in budgets; normalized text is used only for parsing test names.

Complete local receipts and read-only audit tools are ignored
`tmp/growth-audit/`: per-run API/artifact/job metadata, downloaded archives,
`retention.json`, `link-{linear-growth,emission}-audit.json` and `tools/`.
The Actions runs and artifacts above provide the source-bound retrievable evidence.
Fresh local units pass with **1,639 successes / 662 controls**, zero failed or
ignored; their full occurrence multiset equals the preserved pre-edit baseline.
Full lint passes, including formatting, workspace/all-targets/CLI clippy,
provenance/core/family/CI-gate checks and eight frozen-table checks. The native
runner restores the original manifest and leaves the lock and siblings unchanged.
No source or sibling edit is needed for this evidence closure. The documentation
commit's later CI must still be checked separately; its success is not inferred
from either earlier full run.

**Next bounded task:** prepare independent inline-directory-to-block conversion
growth qualification, first reviewing the actual supported/refused path and
its exact allocator, metadata, checksum and error contract. Do not infer inline,
indexed, indirect/extent/deep/uninitialized-group or counter-wrap growth from
this linear result. Scoped implementation/evidence review and all broader
ownership/alias/seal/admission/lifecycle/recovery/fuzz gates remain open.
Stages 4–5 stay **not yet quantified**. Casefold writes remain blocked and
Windows/card/SteamOS qualification remains later work.

## Linear-growth external-attribute sector expectation correction — 2026-10-28

The first exact new-source focused
[Stage 0 run 40727685681](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40727685681)
fails only `link-linear-growth` job **122955259496** at source
**`3fe3f92189ed0b877c2b3946c24affba5362d949`**. The other twelve jobs pass,
including the repeated allocation-free emission test. Failed artifact
**12833757215** has advertised SHA-256
`1d24a451b5267f9483aebbf0368b3d30ae76ba10df2b4dc172a6679db42fabad`;
all **eleven ZIP CRCs**, recorded source and **thirteen source-file hashes**
are verified against that committed tree. This is a partial first-input archive,
not the planned 352-image matrix or a successful emitted-link qualification.

The complete focused log shows `full_parent` failing **4 != 2** in the first
Rust/1 KiB/128-byte/direct profile, before any driver link is invoked. Linux's
saved fill report gives size **1,024**, sector count **four**, and three long
fillers; judged non-repairing e2fsck output contains all five passes. Raw
public-format decoding of the retained inode **2049** confirms one direct
directory-data block **8453**, one separate external-attribute block **8456**,
no other data/indirect pointers, a one-reference EA header and four sectors.
The test input intentionally has a large parent EA, so both blocks contribute
to `i_blocks` / Linux `stat.st_blocks`. The old shape assertion counted only
the data block; the driver has not produced incorrect bytes at this failure.

A temporary native diagnostic reads the **actual saved Linux image**, calls
the unchanged `full_parent` assertion and reproduces **one failure, 4 != 2**.
After the test-only correction, the same diagnostic passes **one test, zero
failed/ignored**, including all original record/slack/mapping checks. The
corrected assertion requires a nonzero EA address distinct from the directory
block and exactly two blocks' worth of 512-byte sectors for this controlled
input. The actual growth assertion remains **old sectors plus one data block**,
with the same one-bit allocation/SB/group debit, no inode allocation, metadata/
EA preservation, independent reports and cold readback. No test is removed,
weakened to mere success, skipped or replaced by self-built image bytes.
The temporary probe is restored and is not committed as a host-specific test;
the exact corrected focused test remains the durable reproduction.

The exact source's
[full run 40727685619](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40727685619)
fails native job **122956663190** and guest job **122956663257** at that same
assertion. Each reports **1,343 library passes / one failure / zero ignored**;
later whole-suite integration tests do not run, and native whole floor/shell/
installed-CLI steps are unrun. ci-ok correctly fails; the other eight jobs pass.
Seven source/digest/all-CRC-verified tier archives preserve all three **1,639**
unit-occurrence multisets. Do not describe this as 2,488 passing whole tests
or borrow the preceding `2137e23` green result for the corrected source.

Fresh corrected-source native units retain every **1,639 occurrence / 662
controls once**, zero failed/ignored. Full lint passes, including formatting,
workspace/all-targets/CLI clippy, provenance/core/family/CI-gate checks and
eight frozen-table checks. Production writers, shared commit helpers, manifest/
lock, workflow selection/floors/budgets and every write gate remain unchanged.
Local complete receipts are ignored `tmp/casefold-link-growth/input-witness.json`,
`ea-correction.json`, `replay_input.py`, `tmp/logs/link-growth-input-{red,green}.log`
and `tmp/casefold-link-ci/407276856{19,81}/`.

Corrected-source Linux/full CI is **pending** until actually observed. First
inspect its focused linear-growth and repeated allocation-free results, repair
only demonstrated failures, then audit the complete saved directory/data/EA/
allocator/SB/group/bitmap/checksum/journal/cold metadata evidence and full
retention (expected 1,639 unit / 2,488 whole successes, 662 controls in five
contexts and actual wrappers/floors/trailing checks). The same ignored slow
fuzz case remains a qualification gate once reached. This fixes the first
input-shape expectation only; no first growth, all-layout growth, broader
mapping/ownership/admission/seal/lifecycle/recovery, casefold write activation,
or Windows/card/SteamOS qualification is inferred. Inline/index/indirect/
extent/deep/uninit and counter-wrap evidence remain separate bounded work
before the scoped implementation/saved-evidence review. Stages 4–5 are still
**not yet quantified**; both link patches remain historical/promoted.

## Full-linear-parent public hard-link growth test preparation — 2026-10-28

Production `src/fs.rs`, `src/capi.rs` and `src/fs/guarded_chmod.rs` are
unchanged. The shared independent link-emission test runner now has separate
allocation-free and full-linear-parent profiles. Its original test/name and
all prior phase/error/metadata/readback assertions remain. The new exact test is
`fs::guarded_chmod::needs_host::link_emission::growth::linux_checks_public_link_growth`.
The pinned `link-linear-growth` job selects it with `--exact --show-output`,
requires floor one, initially budgets 4,200 lines / 340,000 bytes and retains
complete logs, thirteen source hashes, input/dirty/closed/Linux-copy images,
allocator identity, debugfs/e2fsck reports and cold Linux metadata literals.
The repeat allocation-free job also retains all thirteen source hashes now.
No existing tier budget, floor, selection or ignored test is changed here.

The **planned**, not yet credited matrix is Rust/C × 1/4 KiB × 128/256-byte
inode × direct/journal: **sixteen layouts, sixteen growing first links and
96 later non-growing links (112 total), 448 pre-effect refusals, 352 images,
1,536 debugfs reports, 240 judged non-repairing e2fsck checks, 144 guest mounts,
28 requested C target snapshots and 1,932 cold Linux JSON rows**. The first
growing operation cycles all three Rust forms and both C forms across layouts;
this does not cover every form in every layout. Regular-file sources perform
the first growth; later phases retain the fast/slow/dangling-symlink and FIFO
coverage and raw/255-byte names. No general all-source-kind growth claim is made.

Linux alone creates the input and fills `/policy` with three/fifteen 255-byte
hard links to an existing peer, without allocating new inodes or a second
directory block. This preserves all filler identities in the existing peer
record. The driver-side raw input check walks every directory record, excludes
the checksum tail and proves no live/free record has the 264-byte space needed
for the first alias. The parent must be ordinary, non-inline, non-indexed,
one data block with matching sector count and no second logical mapping.

After the first driver link, the directory must have exactly two data blocks
and the old first mapping must remain. No third mapping is allowed. The primary
SB and affected group lose exactly one free block, free-inode count stays fixed,
every inode bitmap stays byte-identical and every block bitmap equals the input
plus exactly the newly mapped block's zero-to-one bit. Complete descriptor-block
comparisons allow only that group's free-block words and applicable block-bitmap/
descriptor checksums; inode accounting, flags, unused counts and other group
metadata must remain. Later links and closed snapshots must preserve the same
allocation bytes. Existing source/parent time/version/identity/readback checks
remain, with only first-parent size/sector/mapping fields permitted to change.
Cold Linux checks every filler alias against the protected peer and requires
exact directory membership, content/EA values and independent free-space counts.
Dirty/closed/journal state, checksum checks, all debugfs metadata fields and
non-repairing e2fsck verdicts remain required; Linux observes disposable copies.

Fresh native validation retains **all 1,639 occurrences / 662 controls once**,
zero failed/ignored, with exact before/after multisets. Full lint passes,
including formatting, workspace/all-targets/CLI clippy, shared-core/family/
CI-gate/provenance checks and eight frozen-table checks. The focused name lists
exactly one real test; compilation and guest Python syntax pass. A selection
listing is not execution. The cloud host never mounts images or runs oracle
tools; independent execution is pending in the pinned GitHub harness. Local
receipts are ignored `tmp/casefold-link-growth/` and `tmp/logs/link-growth-*.log`.

The preceding budget-repair source **`2137e234a17fb3486a7f5741671ddd6cc87d1b91`**
now passes all eleven jobs/ci-ok in
[full run 40649608984](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40649608984).
Seven API-source/advertised-SHA-256/all-CRC-verified tier archives retain all
**1,639 unit / 2,487 whole** successes and **662 controls in five contexts**.
Actual wrappers/floors and trailing twenty native shell/159 installed-CLI checks
pass, with standalone CLI 159, ARM images 527, oracle 225, kernel 36,
third-implementation nine and wasm five. Its native whole log **3,916 lines /
238,065 bytes** passes the repaired 239,000-byte cap (it exceeds the prior cap);
guest whole **4,081 / 247,156** passes unchanged caps. The same one ignored
whole/image slow fuzz case remains a qualification gate. Native artifact
**12802095565** has SHA-256
`7e689b807daee11e27ed33999bc7f0ffb9e87bd0d2f51a7f995c94aa565421b8`;
guest artifact **12801261663** has
`bda34fe8f107ce7b2828eece9c5fbb33e1a768b5876f26decf1e849533aa0dad`.
Complete audit receipts remain under ignored `tmp/casefold-link-ci/40649608984/`.
These preceding results do not qualify the new test source or growing bytes.

Next inspect exact new-source `link-linear-growth` and repeated `link-emission`
jobs and all retained source/digest/CRC/image/report evidence, adapting the local
auditor for thirteen source hashes and `GROW` literals. Audit primary-SB/group/
bitmap accounting, inode/data/EA/checksum preservation, actual input/mapping
shape and each dirty/closed/Linux-copy state. Fix only actual demonstrated
failures, preserving the full baseline. Close exact full-CI occurrence retention
(expected **1,639 unit / 2,488 whole**, 662 controls in five contexts and actual
wrappers/floors/trailing checks) when complete; do not wait an hour polling.
Only after these results may the tested linear shape be credited. Inline
conversion, ordinary index, legacy indirect/extent/deep/uninit growth and
counter-wrap qualification remain separate bounded work before scoped source/
saved-evidence review. The ownership/alias/seal/admission/mapping/lifecycle/
power-loss/fuzz gates and casefold write protection remain. Both link patches
stay promoted/historical. Windows/card/SteamOS are unqualified and stages 4–5
remain **not yet quantified**.

## Public-link full-suite retention and measured native budget repair — 2026-10-28

Exact emission source **`e6db2588cbd913db4357afa43eee8c6ef80f3eda`** full
[run 40637075795](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40637075795)
is complete. All 2,487 whole-suite occurrences pass in both contexts, with zero
failed tests and the same one ignored slow fuzz case; all 1,639 units and 662
controls are retained. Native job **122647162297** then returns **65** because
its complete log is **3,916 lines / 238,064 bytes**, 64 bytes above the unchanged
238,000-byte cap. The subsequent actual whole-suite floor, twenty native shell
checks and native installed-CLI checks are **unrun**, not inferred passes.
Its other nine jobs pass; ci-ok correctly fails with the native job.

The docs-only successor **`741381c87de299ec3fa021fbdcaa639bff70faf0`** differs
only in this handoff and the index review. Its exact
[full run 40640607199](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40640607199)
passes all **eleven jobs and ci-ok**. Both runs' seven complete tier archives
are bound to their respective source by API metadata, advertised SHA-256 and
every ZIP CRC; extracted bytes are checked against each archive. Full test-name
**multisets**, not just totals or sets, retain every prior 1,639 unit/2,486
whole success and add exactly the new public-link emission host test once in
each whole suite. The 662 guarded-operation controls occur once in each of
the five unit/whole contexts. Newer full native/guest logs measure
**3,915 / 237,948** and **4,084 / 247,513** lines/bytes. All five actual wrappers
and floors pass; the actual trailing twenty shell and 159 installed-CLI checks
now execute successfully. Standalone CLI retains 159, ARM image tier 527,
oracle tier 225, kernel tier 36, third-implementation tier nine and wasm five
successes. The same ignored whole/image slow fuzz case remains a gate.

The failure is measured and reproducible despite the later green run: elapsed
times and runner output make this near-limit log vary. Replaying the **complete
unaltered** failing log through the pinned wrapper returns 65 at **3,950 /
238,000**, then succeeds at **3,950 / 239,000**. SHA-256 is identical for the
download, failing replay and passing replay:
`fe5bb7c54fb9c2c09fe907e138a8f36edff05c81aa244c28affcc9a344e313b8`.
The real floor parser counts **2,487** successes against floor **1,060** in
both that repaired replay and the later green log. Only the native whole byte
cap changes by 1,000 bytes; its line cap, guest/unit/other caps, filters, test
selection, floors, ignored-case policy and complete output are unchanged.
No new production code or driver tests are introduced or executed locally by
this checkpoint; these are retained actual CI executions plus wrapper replays.
Full local lint passes, including formatting, workspace/all-targets/CLI clippy,
shared-agent/family/CI-gate/provenance checks and eight frozen-table checks.
The task list parses successfully; the temporary pinned-core source selection
restores the manifest and leaves the lockfile unchanged. No sibling is changed.

For independent retrieval, the green full-run archives are:

| Artifact | ID | Advertised SHA-256 |
|---|---|---|
| `tier-logs-suite-in-vm` | `12797901557` | `3cd414e2059f4e7c02087e9a3a6b7dc70be7440df8b8c73083fddde56b7b260a` |
| `tier-logs-x86_64` | `12796964018` | `bf0f5d952ee36ef50a48a51aa217909882d3f689b83a5a67e1b70957428fd6ae` |
| `tier-logs-semver` | `12796010189` | `dedf25fa9af802bd7b3233ad086851af9ce0c5795a4982e8dc08d0891f2e458c` |
| `tier-logs-aarch64` | `12795817775` | `498b2272db03d1280f2c7c0452e87a98ee50b5199208265668250b0ba07c759d` |
| `tier-logs-cli` | `12795282333` | `39521667ac026ac807226cef1e4ad11cbb9d4f62d27483d91250786c341c5398` |
| `tier-logs-wasm` | `12795157408` | `7745b54d5b45472d65df0524acb55c66c2fc40b31f301fb6f5d8405d357ba0cc` |
| `tier-logs-unit` | `12795113053` | `ac415885752efbe012b6ed5a1a1443e0da261a769ae038ec9747491d0bc6d6c8` |

The earlier overflowing native archive is **12796183385**, advertised SHA-256
`dda91b002bf359d88c80fab6f9c921e914ebe2ff3230fa5bc203553976af8504`.
Local complete receipts are ignored `tmp/casefold-link-ci/<run>/retention.json`,
`full-closure-retention.json` and `tmp/logs/link-full-closure-retention.log`;
the linked Actions runs retain the original source-bound logs. This audit does
not repeat the previous 352-image saved-byte audit or infer new growth evidence.

The new cap commit's exact CI remains **pending** until actually observed;
the earlier green run establishes unchanged source/test behavior, not an
unseen later-tip conclusion. After checking it, the next implementation-sized
qualification is **a real full linear parent which must allocate on hard-link
insertion**, with Rust/C, 1/4 KiB and direct/journal profiles as supported.
Assert the actual input/growth shape, exact primary-SB/group/bitmap block debit,
no inode allocation, source/shared-table/peer/EA preservation, dirty/closed
journal state and independent cold Linux/e2fsck readback. Inline conversion,
ordinary indexes and legacy/extent/deep/uninit mapping, independent counter-wrap
qualification and scoped implementation/saved-evidence review remain later
bounded work. No casefold write, corruption-admission, ownership/seal/mapping,
lifecycle/recovery/fuzz or Windows/card/SteamOS gate is relaxed. Stages 4–5
remain **not yet quantified**. Both link patches stay historical/promoted.

## Allocation-free public hard-link exact Linux emission and saved-byte closure — 2026-10-28

Exact test source **`e6db2588cbd913db4357afa43eee8c6ef80f3eda`** passes
focused job **`122646396891`** in
[Stage 0 40637075787](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40637075787).
Artifact **`12793499218`** has advertised SHA-256
`1e6cf605d9b700a6dbbd8fe457694c3ad3eb6b0943a66ac64cbd1e624f78c917`;
all **4,275** ZIP members pass CRC validation. The retained commit and all eleven
source hashes match the actual git tree. Protected `src/fs.rs`, `src/capi.rs`
and `src/fs/guarded_chmod.rs` are byte-identical to implementation `afd24b8`.
All twelve Stage 0 jobs also pass for this exact source; only the dedicated
link-emission saved images are freshly audited here. No production fix,
missing fixture, ignored case or new budget repair was needed.

The exact test executes once, zero failed/ignored. Its real job-log floor one
and wrapper pass at **2,141 lines / 287,619 bytes**, inside initial 4,200/320,000
caps. Actual oracle markers, saved images/reports and source-bound code match
all **sixteen profiles / 112 successful links / 448 pre-effect refusal
observations / 352 images / 1,536 debugfs reports / 240 judged e2fsck calls /
128 guest mounts**. This is observed evidence, replacing the preparation's
planned counts. Rust/C × 1/4 KiB × 128/256-byte inode × direct/journal layouts
cover regular, fast/slow/dangling-symlink and FIFO sources, with all five public
forms cycled through seven alias phases. It is not every possible cross-product
of source kind and entry form. Three C target snapshots per C layout (regular,
slow symlink and FIFO: **24** total) agree with independent Linux metadata;
C counted NULL readback is also exercised. The existing native policy controls
retain the wider C snapshot source-kind coverage.

The complete saved-byte audit independently decodes all 352 images and compares
actual Linux/debugfs metadata and namespace reports. Every expected source
inode/count/ctime, parent mtime/ctime/directory-version change, generation,
mode/owners, size/sectors, mapping, payload, symlink target and user EA matches.
All group descriptors/bitmap bytes, global/group free counts and exclusive EA
bytes/refcounts remain unchanged. Every unrelated allocated inode, including
reserved/protected records, retains its original bytes in dirty, closed and
Linux-copy states. Inode/SB/BGD/bitmap/EA/JBD2 checksums and clean closed journal
headers are checked where enabled. Original root/old-parent/peer namespaces
and payload remain; driver snapshots agree across controlled close. Linux reads
copies, permitting expected symlink-atime observer changes while the actual
driver bytes remain pristine between phases. All **1,928** cold Linux JSON rows
match saved driver bytes and all **240** saved e2fsck outputs contain judged
non-repairing full-pass reports. This is normal completed-operation evidence,
not a crash-recovery or corrupted-layout admission qualification.

The original implementation's exact
[full CI 40633679897](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40633679897)
now passes all eleven jobs and ci-ok at `afd24b8`. Seven tier archives are bound
to that source by API metadata, advertised SHA-256 and every ZIP CRC. Complete
occurrence multisets retain every old 1,503 unit/2,350 whole result plus 136 new
controls, giving **1,639** unit successes in three contexts and **2,486** whole
successes in both suites, all **662 controls once in five contexts**. Actual
wrappers/floors and trailing twenty native shell/159 native installed-CLI checks
pass; standalone CLI retains 159 and ARM images 527, with 36 kernel checks.
One unchanged ignored whole/image slow fuzz case remains a qualification gate.
Native whole measures **3,914 lines / 237,852 bytes** and guest
whole **4,081 / 247,052**; existing caps fit, so none is raised.
All eleven earlier Stage 0 jobs and the compatibility job also pass at `afd24b8`;
no fresh audit of all their image archives is claimed here.

Fresh native validation at the emission checkpoint retains all **1,639 / 662**
occurrences/controls with full workspace/all-targets/CLI lint, provenance,
formatting and Python syntax passed. The exact focused filter lists one real
test. New source `e6db258` adds one host test; its separate full CI
**`40637075795`** and expected **2,487** whole successes remain **pending** at
this checkpoint, not credited from the earlier source. This closing commit
changes documentation only and does not claim an unobserved later-tip CI run.
Local complete receipts/logs are under ignored `tmp/casefold-link-emission/`
and `tmp/casefold-link-ci/40633679897/`; committed test/job and archived evidence
provide reproduction without relying on those local caches surviving.

Next inspect exact `e6db258` full CI and occurrence/floor/trailing retention,
repairing only measured failures. Then qualify actual full-parent growth,
inline conversion, ordinary-index and legacy/extent/deeper/uninit mapping
shapes with allocator SB/debits, source/shared-table/peer preservation and cold
Linux/e2fsck evidence. Cover directory-counter wrap independently. The scoped
source/saved-evidence review follows those missing shapes. This checkpoint
qualifies healthy ordinary **allocation-free** tested layouts only; it does
not enable casefold writes or resolve ownership aliases, admission/seals,
complete index/mapping, lifecycle, power-loss/recovery, ignored fuzz or real
Windows/card/SteamOS use. Both link patches remain historical/promoted and
must not be reapplied; stages 4–5 remain not quantified.

## Allocation-free public hard-link emission test preparation — 2026-10-28

Production writers are unchanged from `afd24b8`. The new
`needs_host::link_emission::linux_checks_public_link_metadata` test and focused
Stage 0 job prepare independent qualification for **allocation-free ordinary
parents only**, across Rust/C × 1/4 KiB × 128/256-byte inode × direct/journal.
They reuse the existing Linux-only reference input script, not a self-built
image. The driver then creates seven aliases per layout, retaining regular,
fast/slow/dangling symlink and FIFO source identity, mapping/data/EA/owners/
generation; source nlink/ctime and parent mtime/ctime/version must change once
per successful link. The test covers all three Rust and both C forms, raw and
255-byte spellings, final source symlinks, intermediate parent symlinks, optional
C target snapshots and NULL readback. Four ordinary pre-effect refusals per
phase leave every byte unchanged and do not call Runtime. Exactly one reached
link clock call and no new inode generation are required on success.

Saved descriptors and both bitmaps in every group, all free counters, inode
bytes and exclusive EA blocks enforce zero allocation and protected peer state.
The independently parsed debugfs output checks inode identity, mode/owners,
generation/flags, size/sectors/EA, link count, times and persisted directory
version. The cold Linux script checks every original and alias inode/metadata,
namespace, payload, symlink targets, attrs and free counts. It reads disposable
copies; original driver images remain byte-identical across observer checks.
Judged non-repairing e2fsck reports and source hashes are retained explicitly.
Dirty and clean driver snapshots and post-Linux observer copies remain separate.

The **planned**, not yet executed, matrix is sixteen profiles, 112 successful
links, 448 refusal observations, 352 saved images (one input and seven sets of
three per profile), 1,536 debugfs reports, 240 e2fsck checks and 128 guest mounts.
The new focused budget 4,200 lines/320,000 bytes is an initial cap awaiting actual
measurement; no existing tier cap is changed. Compilation/full workspace lint,
guest Python syntax and the full native 1,639-occurrence/662-control multiset pass.
Actual Linux execution, floor-one evidence, source/digest/all-CRC binding and
saved-image/report audit are **pending**. Existing `afd24b8` full CI is still
running; passed earlier Stage 0 badges do not substitute for this new matrix.
Next execute/repair/audit this focused matrix and close pending full retention.
Qualify all claimed growth shapes and counter wrap independently afterward,
then perform the scoped source/evidence review. This is not crash recovery,
corrupt-layout admission, casefold write activation or Windows/card qualification.
Both link patches stay historical/promoted; stages 4–5 remain not quantified.

## Guarded public hard-link native migration — 2026-10-27

Implemented the bounded public hard-link migration from `fa3949a`. The
fresh native pre-fix run retains all 123 names with 32 passes / 91 failures.
The repaired safety and measured-policy modules are active in the normal unit
and full tiers; the two committed red patches stay unchanged as historical
artifacts and **must not be reapplied**. Their original bodies remain, except
for the safety callback counter/printed label `skip` → `after` required by the
existing no-skip scanner and the updated module comments. No scanner rule,
selection, floor, oracle, manifest, lock or shared nlink/allocator helper changes.

The link-only trusted completed plan owns the existing target and all staged
source/parent edits. The public context precedes device/runtime/Into callbacks,
retains the second path volume verdict, counted name-before-both-Into order,
source-before-parent validation and existing syscall/API differences. Only
OutOfBounds requests parent growth. Latest buffered inode slices retain source
count and shared-table edits, inline content and grown mappings. One reached
Runtime clock read supplies source ctime and destination-parent mtime/ctime;
the parent's persisted directory version increments, separately from inode
generation and the operation epoch. The low counter carries only into a covered
high word. Counter wrap boundaries still need independent qualification; the
reference currently measures ordinary increments. No generation is allocated.

Genuine allocation-free/inline-fit plans may omit the primary SB. Actual full
parent/inline-conversion plans retain SB presence and exact block debits. A
present block must have the complete mounted block size and valid SB offset/
magic before effects; preparation clears only VALID_FS, retaining counters,
other state and outside-region bytes and restamping CRC where enabled. The
one-time marker and **entire** raw commit, including empty-override publication,
run inside the existing latching effects. Returned errors and flush/publication
unwinds retire the mount before a retained context drops. Ordinary planning
refusals remain healthy; public planning/Into/Runtime unwinds retire on context
unwind. C optional target reads hold the same context and return an owned inode
snapshot for pure attr formatting. NULL skips readback; a normal read error may
follow exactly one committed alias/count increment, leaves output sentinels and
does not roll back or retry. This does not qualify crash atomicity for direct
writes, arbitrary caller races, corrupted layouts or lifecycle admission.

Validation:

- Original 123 focused controls turn green. The final focused run passes all
  **136**, zero failed/ignored: original 84 safety + 39 measured-policy controls
  plus thirteen real-plan/foreign-context/malformed-SB/constructed-SB/effect/
  retained-publication/Runtime diagnostics. Source/header-only and callback-
  counter changes do not drop the original assertions. All positive inline,
  ordinary-index, legacy/extent/deep growth, shared-table and uninit profiles
  remain. Six fault sweeps now reach **216 measured I/O injections** (60 writes
  + 24 flush errors + 24 flush unwinds in each Rust/C route), zero retirement
  violations, plus four original actual post-commit publication profiles.
- Full native units pass **1,639**, zero failed/ignored, retaining all 1,503
  baseline occurrences as a multiset. **662** controls execute once (formerly
  526). The first full attempt finds the promoted callback diagnostic's `skip`
  label through the unchanged scanner; renaming it fixes that contract failure.
  The initial new reverse-Runtime diagnostic assumed one clock call in all old
  families; its correction asserts an actual nonzero invocation count and
  checks exclusion on every clock callback. Neither issue suppresses a test.
- All-green unit log measures **1887 lines / 145,690 bytes**
  against old 1,860/143,000 caps; actual wrapper exit 65 and identical-log old-red/
  new-green replay justify **2,000 lines / 155,000 bytes**, floor 440 unchanged.
  No native/guest whole, kernel, fixture or CI workflow cap is raised from a
  projection. Full native lint (shared core, family/gate, provenance, Unicode,
  formatting, workspace/all-targets/CLI clippy) passes. Final local receipts and
  complete logs are under ignored `tmp/casefold-link-repair` and `tmp/logs`.
- Earlier exact reference `9fd59cf` full/focused archives remain evidence for
  those bytes. Fetched pre-repair `fa3949a` full CI `40629901027` had passing
  unit/CLI/MSRV/semver/provenance/wasm/fixture/ARM jobs at this inspection, with
  native/guest whole still pending. Fresh full CI and Linux emitted-byte evidence
  for this repair remain **pending**, not credited from older reference CI.

Next verify exact-source full CI/retention, fixing only actual measured failures;
then independently qualify Rust/C emitted hard links, retained source metadata,
optional readback and all supported growth shapes in the pinned guest. Save and
audit dirty/closed/journal images, exact allocation accounting, e2fsck verdicts
and cold Linux alias/payload/EA/peer results. Scoped source/evidence review follows.
The Linux reference-only eight-layout/80-image collection stays unchanged.
Both red patches are promoted historical artifacts, not fresh-chat setup steps.
Casefold writes, broad ownership/alias/seal/mapping/lifecycle/recovery and real
Windows/card/SteamOS qualification remain gated; stages 4–5 are not quantified.

## Measured native hard-link policy reds and composition — 2026-10-27

Started from clean fetched `8f0006a509cdccf4838d4cde9a02225225d64bb9` after
checking [reference-source full CI 40615435814](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40615435814)
and its retained evidence. All eleven jobs and ci-ok succeed. Docs HEAD also
passes exact full run **40616704908**; source/test/workflow/dependency bytes
match the verified reference source. No protected production writer, sibling,
pin, lock, normal task/cap/floor or write gate changes in this checkpoint.

### Full reference-CI retention gate is closed

All five tier archives verify their source, advertised digest and every CRC.
Complete **multisets**, not sets or totals alone, retain the old 2,349 whole-suite
successes plus the one executed Linux link reference (**2,350**). Each context
retains all **526 controls once** and every earlier native unit occurrence.

| Context | Successes | Failed / ignored | Actual lines / bytes |
|---|---|---|---|
| unit | 1,503 | 0 / 0 | 1,759 / 133,707 |
| ARM-unit | 1,503 | 0 / 0 | 1,753 / 133,270 |
| native-unit | 1,503 | 0 / 0 | 1,753 / 133,270 |
| native-whole | 2,350 | 0 / 1 | 3,778 / 225,383 |
| guest-whole | 2,350 | 0 / 1 | 3,945 / 234,723 |

The unchanged native/guest caps are 3,950/238,000 and 4,120/248,000. Actual job
logs prove both whole wrappers and **2,350 executed / 1,060 floors** pass.
The native **36-pass kernel** log is 631 lines / 55,214 bytes within 700/62,000;
all **twenty shell and 159 native installed-CLI checks actually execute and pass**.
Standalone CLI retains 159; ARM images retain all 527 successes and the same
existing ignored slow fuzz case. That ignored case is not new reference coverage
or permission to enable writes. No budget or test-selection repair is needed.
Local receipts are `tmp/casefold-link-reference/full/retention.json` and
`tmp/logs/link-policy-full-retention.log`, plus the verified archives/job logs.
The independent reference's eight layouts/80 images/56 aliases/560 errors/960
reports and source/digest/full byte audit remain applicable; no fresh dispatch
or new Rust/C emitted-image result is inferred from this native preparation.

### Standalone measured policy

The saved [CASEFOLD-LINK-POLICY-RED.patch](CASEFOLD-LINK-POLICY-RED.patch) has SHA-256
`c84914f0e105bd4b1a9eb74560f197de9e8346d41786a6417c92d84ea789b924`.
Its temporary test module hashes to
`53f03a0fd20dfd5d844eafd3f99631db02ca7d8e80456341095cee7ebe74849b`.
It is executed before implementation: **39 controls / 12 passes / 27 semantic
failures / zero ignored**. All six native profiles run (ext2 legacy/direct,
ext3 legacy/journal, ext4 checksummed/direct, each at 1/4 KiB), using measured
Linux rules from the separate eight-layout reference. Five source kinds are
regular, fast/slow/dangling symlink and a constructed FIFO. Native fixtures
prove self-consistency against the measured rules, not independent emissions.

| Group | Controls and actually executed coverage |
|---|---|
| Source ctime | Ten failing controls: five Rust/C forms × one/repeated alias, five kinds × six profiles. **450 links** reach expected Runtime-second ctime while identity, source atime/mtime, payload/EA/owners/generation/version and nlink +1 remain. |
| Destination times | Ten failing controls, same forms/kinds/profiles and **450 links**: parent mtime/ctime expected from the same stamp; atime/nlink/maps/EA/mode/owners/generation and unrelated source/old-parent/peer bytes remain. |
| Directory version | Five failing controls, one per entry form, repeated aliases for every kind/profile: **300 links** require the independently measured +1 directory change counter, distinct from source generation/version. |
| Optional C attrs | Two failing one/repeated controls: **90 links** read existing target identity/mode/owners/nlink/size/blocks correctly, but its returned ctime remains old. All kinds/profiles execute before aggregate failure. |
| Existing identity/types | One passing aggregate: **150 links**, all five forms/types × six profiles; exact data/EA/raw source fields and peer/parent/counter invariants remain. |
| Names | Two passing aggregates: **30** 255-byte and **24** raw source/destination name links, preserving identity with no allocation. |
| Explicit path contracts | Three passing syntax controls execute **54 links** through intermediate parent symlinks, trimmed destination trailing slashes, root-based relative parents, repeated separators and distinct letter case. Three refusal controls execute **342 errors** preserving metadata and prior-dirty bytes. |
| Counted/FFI priorities | Two passing controls execute **144 refusals**: invalid names before source/parent checks, source before parent, generations/types/duplicates/dots, and C pointer/ceiling priorities. One passing dual-Into control executes **30 refusals and six successes**, invalid names before both conversions and stale source before parent Into. |

Totals per executed policy run are **1,554 successful links / 516 refusals**;
**1,290** of those successful links reach the semantic expectations and all
currently mismatch the requested timestamp or directory-counter field. Source
and parent edits reopen consistently; checksum-verified reads and unchanged
allocation counters/other inode bytes are asserted. Legacy Runtime timestamps
have second precision; the test expects zero subsecond fields rather than
claiming nanosecond Runtime support. No hypothetical clock/generation unwind,
missing-fixture/compile failure or fake future LinkPlan helper counts as a red.

Measured Linux syscall errors do not silently replace the driver's explicit
APIs: source-directory EISDIR, counted name-before-two-Into/source-before-parent
order, path destination syntax/length before source lookup, root-based parent
resolution and trimmed destination trailing behavior remain. C pointer/path/
ceiling/generation, permission-only attr mode, existing-target readback and
zero/-1 conventions stay. The earlier source-trailing-slash directory requirement
and no-follow final-symlink identity are retained. Ordinary refusal observations
use a prior-dirty mount so these semantic controls do not duplicate the saved
clean-marker safety failures; no new refusal-marking exemption is introduced.

### Both patch orders and complete repetitions

The original safety patch remains unchanged at SHA-256
`87d359242643c487141bb250a304a8c109f22eb6ec599419c206d948c1ee1e63`.
Policy-only executes first, is saved/reversed to original bytes, then
policy→safety is applied and executed. Both patches are reversed to baseline;
safety→policy produces the exact same three source-file hashes and executes
again. Both actual combined runs repeat **123 names / 32 passes / 91 failures /
zero ignored**, with every standalone policy outcome and original **84 safety
outcome** retained. Each retains all **200 measured I/O injections / four actual
post-commit publication unwinds / 54 primary safety-positive profiles** and all
original optional C readback observations. Full name/verdict maps and profile/
call/injection counts are compared, not only aggregate totals. Compile errors
from an initially mistyped existing helper name were corrected before these
executions and are not counted as semantic failures.

The separate diagnostic tiers keep complete output: standalone **362
lines / 188,529 bytes**, combined A **1292 / 252,299**,
combined B **1292 / 252,299**, within 6,000/800,000. Normal CI caps
are unchanged. Dormant standalone/composed all-target/CLI clippy passes. The
patch declarations are in different parts of the existing test module, so both
orders apply without editing either patch or dropping a test. Earlier promoted
family patches remain historical and must not be reapplied.

### Restoration, receipts and next bounded task

Both patches are reversed at the end; active source/test/workflow/manifest/lock
bytes match the baseline hash manifest and both temporary modules are absent.
Fresh `chore test:unit` retains every **1,503 baseline successful occurrence / all
526 controls exactly once**, zero failed or ignored; full `chore lint` passes.
The immutable core v0.3.0 and temporary coding source view are preserved and
Cargo.toml is restored with unchanged Cargo.lock. The saved patches stay dormant,
so normal CI does not execute deliberately failing controls.

Ignored local receipts under `tmp/casefold-link-policy/` include baseline/source
hashes, `composition-a.json`, `composition-b.json`, standalone/combined outcome
maps and `retention.json`; complete logs are `tmp/logs/link-policy-red.log`,
`link-policy-combined-a.log`, `link-policy-combined-b.log`, `link-policy-outcomes.log`,
dormant lint logs and the saved native baseline/restored logs. Fresh environments
can reproduce from the two committed patches and prescribed pinned source view.
A combined diagnostic command, after applying both patches, is:

```sh
scripts/tier.sh link-policy:combined link-policy-combined 6000 800000 -- cargo test --locked --lib link_calls:: -- --skip unlink_calls:: --nocapture --test-threads=1
# Expected current source: exit 101; 32 passes / 91 failures / zero ignored.
# The filter selects the two link modules; the explicit skip excludes the
# unrelated existing unlink modules, which remain in the full native baseline.
```

**Next bounded task:** implement public hard-link shared-context entry and a
completed owned LinkPlan, preserving path volume verdicts, counted name-before-
both-Into/source-before-parent priorities, optional-SB allocation-free plans,
full growth debits, source/parent inode-table splices, complete marker/raw commit/
checkpoint/flush/allocation publication effects and optional C owned target
readback under that same context. Repair measured source/parent times and parent
version behavior; preserve generation/data/EA/owners and all explicit API/FFI
contracts. Reapply/promote both patches only with their failures repaired, then
add diagnostics at the actual helper boundary (including legal Runtime hooks
once they really execute), foreign context, malformed present SB, valid no-SB,
real growth debits and retained-context returned-error/unwind retirement. Re-
measure actual I/O positions after the plan changes; do not freeze the pre-repair
200 count or weaken the controls. Independent Rust/C emitted-image, growth and
full-CI/standalone source/evidence qualification follow before scoped link closure.

No guard, metadata repair, fresh Linux dispatch or new emitted-image qualification
is implemented here. Stages 4–5 remain **not yet quantified**; casefold writes
stay blocked. Broader ownership/alias/admission, complete mapping/index growth,
seals/lifecycle/recovery/ignored fuzz and Windows/SteamOS/real-card gates remain.

## Independent Linux hard-link reference and saved-byte audit — 2026-10-27

The reference-only test is added at `01830609ddd2026fa0b197eadfc5b3ff15fcc773`;
corrected source **`9fd59cf5b5d3c1f00846c9015233594f06adb70e`** passes exact focused job **122576714109** in
[run 40615435825](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40615435825).
The full driver, its public APIs, dependency pins/lock, normal budgets/floors and
write gates are unchanged. Only a new `needs_host::link_policy` reference test,
its guest script/declaration, a pinned Stage 0 job and handoff are added. The
[design review](CASEFOLD-INDEX-REVIEW.md#independent-linux-hard-link-reference-policy--2026-10-27)
records the measured behavior and migration limits. No host kernel/oracle, GPL
implementation source or historical non-clean implementation is used.

### First executed failure and evidence-based test correction

Source `0183060` fails job `122574039759` in run `40614604473` after Linux's first
successful link. Verified partial artifact **12784752592**, SHA-256
`0c06a37fe645230f5e9f6d33107625243d8eaad588a125d7f7aef38474f58de7`,
retains the actual input/names/first images, live image and failure log. Nine
source hashes and all 61 ZIP member CRCs verify. The complete 70 error rows in
that first layout execute. Raw bytes show destination-directory version **6→7**
at offset **0x24**, while source version stays **5**, source nlink/ctime update,
and old parent/peer bytes stay unchanged. The test's blanket preservation
assertion was too strict; this is not a driver mutation failure.

The published [ext4 inode format](https://docs.kernel.org/filesystems/ext4/inodes.html)
identifies Linux `l_i_version` at 0x24 and the covered high word at 0x98, separately
from `i_generation`. Corrected source reads that counter and requires exactly
one destination increment per successful link, zero on refusal/other-parent/cold
phases. Source raw-field preservation still includes its generation and version.
This is a measured reference expectation, not an implementation copied from the
kernel or a silent relaxation to obtain a pass. All eight corrected layouts now
actually execute and pass; no partial first-layout result substitutes for them.

### Complete independent matrix and retained-byte audit

Artifact **12785027612** is **3,776,321 bytes**. Its advertised SHA-256
`78f9944e0e46f2a3ff4ac15fe188a600ae47b750f881e399ee5d77a25ff6ecff`
matches the downloaded ZIP. All **2,163 members** pass CRC verification;
recorded source commit and all **nine source/test/workflow/dependency hashes**
match exact committed bytes. Actual complete focused log is **1,317
lines / 162,155 bytes**, within its separate 3,200/200,000 caps; actual wrapper
and test floor one pass. Normal CI budgets/floors remain unchanged.

| Coverage | Executed and audited result |
|---|---|
| Layouts | Eight: 1/4 KiB × 128/256-byte inode × direct/journal; legacy and extents/checksummed features are asserted. |
| Saved phases | Input, refusal names, three regular aliases, fast/slow/dangling/FIFO aliases, and a separate cold close: **80 images**, all checked read-only. |
| Linux operations | Explicit libc link and linkat with zero flags; absolute paths/intermediate directory symlink and relative parent descriptors. **56 successful aliases**, seven per layout. Driver reads only closed images and verifies full image bytes unchanged. |
| Refusals | **35 source/destination/path cases through both forms × eight layouts = 560 errors**, all actual symbolic errno observations consistent across layouts. Includes empty, missing, 256-byte, duplicate/same-name, dots, directory sources, trailing slashes and source-versus-destination priorities. |
| Independent tools | **8 mke2fs / 80 judged non-repairing e2fsck / 960 complete debugfs stat reports / 80 kernel mounts**, all actually executed. |
| Existing identity | Regular payload/600-byte EA, explicit owners/mode/generation, fast/slow/dangling symlink identity and target bytes, FIFO, raw-byte and 255-byte names remain. No new inode or second EA reference; source nlink increments once. |
| Times/counters | Source ctime advances while source atime/mtime stay; destination-parent mtime/ctime and version advance; parent atime/nlink/data/maps/EA stay. Other parent/peer/raw inode fields remain. Failed final-symlink walks may change symlink atime, which is measured separately. |
| Allocation | Exact primary/per-group free-block/free-inode/used-dir counters, inode/block bitmaps and itable-unused/flags remain. Parents are asserted non-growing linear one-block directories. No arbitrary growth/index/ownership admission qualification is inferred. |
| Format integrity | Every applicable SB/BGD/inode/directory/EA/bitmap and journal-header checksum is checked from saved bytes, including clean journal state. Complete report fields, times and mapped data blocks agree with the decoded bytes. |
| Cold verification | All seven aliases per layout retain existing source identity/counts and exact stored names after a fresh mount; source/peer payload and EAs remain. |

Both syscall forms report EPERM for a directory source, ENOENT for a missing
source paired with an overlong destination, ENOENT for a new destination with
trailing slash, ENOTDIR for a regular/fast-file-symlink source with trailing
slash, and ENOENT for a dangling source with trailing slash. **These measured
syscall priorities do not silently override the existing API**: counted name/
Into/source-before-parent ordering, symbolic errno conventions, root-based
paths, destination splitting/trailing handling, C pointer/ceiling/generation
checks and optional committed-target readback remain explicit contracts.

### Retention and next bounded task

Fresh native units retain every **1,503 baseline occurrence / all 526 controls
exactly once**, zero failed/ignored, and full lint passes. Corrected-source
GitHub unit artifact `12784354765` and CLI artifact `12784863198` verify source,
digest and every CRC: same 1,503/526 multiset, **159 installed-CLI successes**.
Preceding safety-doc source `8791f06` now completes exact full CI `40610284789`
successfully. Corrected-source full run **40615435814 remains in progress**;
its complete new whole-suite name multiset (expected **2,350**), all five control
contexts, architecture images, actual wrapper/floor and native trailing checks
remain pending until their actual archives/job logs are verified. One existing
ignored whole-suite slow fuzz case remains a separate gate, not native unit
coverage or a newly ignored reference case.

Local ignored receipts under `tmp/casefold-link-reference/` include source/run/job
metadata, advertised artifact metadata and ZIP, `image-audit.json`, the first
failure's preserved archive/`failure-audit.json`, `local-validation.json` and
`full/unit-cli-retention.json`. Actual logs are `tmp/logs/link-reference-compile.log`,
`link-reference-local.log` and `link-reference-image-audit.log`. These local files
may not survive a fresh environment; committed tests and retained Actions
artifacts provide the reproducible evidence. The dormant safety patch hash stays
`87d359242643c487141bb250a304a8c109f22eb6ec599419c206d948c1ee1e63`.
Do not reapply earlier promoted family patches.

**Next bounded task:** inspect the corrected-source exact full CI and verify
complete **2,350-pass** whole-suite/526-control retention, floors and trailing
checks; repair actual failures before further migration. Then prepare/actually
execute measured native hard-link policy reds and compose them in both orders
with the dormant safety patch, restoring active source afterward. The shared
context/LinkPlan/optional-SB/effect/C owned-readback repair and real helper
boundaries remain later implementation, followed by independent Rust/C emission,
growth and standalone scoped source/evidence review. No link guard or native
semantic repair has been implemented. Casefold writes remain blocked; stages
4–5 remain not yet quantified; ownership/alias/admission, complete mapping/index
growth, seals/lifecycle, recovery/fuzz and Windows/SteamOS/real-card gates stay open.

## Executed dormant public hard-link safety preparation — 2026-10-27

Started at clean fetched `90acfc1bf25093ed61a6c99d1a79e7c8ef248a19` in the
existing checkout. Exact full run [40575650714](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40575650714)
now completes successfully. No sibling, dependency, workflow, cap, floor, active
source or write-gate change is made. The native baseline is freshly executed:
**1,503 successful occurrences / 526 controls**, zero failed/ignored. Earlier
independent backport/retained-family evidence remains applicable to unchanged
implementation; no fresh whole suite or Linux execution is inferred here.

### Executed patch and repeated outcomes

[CASEFOLD-LINK-RED.patch](CASEFOLD-LINK-RED.patch) adds only a temporary test
module and its declaration. It is intentionally failing, **not active CI**.
SHA-256: `87d359242643c487141bb250a304a8c109f22eb6ec599419c206d948c1ee1e63`.
Applied test-module SHA-256:
`521efef4b213d6e1e627e8dbc6822828c51e5e6eeaca3bcd19afb17b68541ff4`.
Actual reverse verifies original source bytes and removes the added module;
reapply verifies the exact exercised bytes before the second run. Complete
**84-name/verdict maps repeat: 20 passes / 64 driver safety reds / zero ignored**.
A rejected overlapping lint attempt made no changes; all completed temporary
manifest source-view runs were sequential.
A C attr assertion initially compared full inode mode with permission-only C
mode; that fixture expectation was corrected before both final executions.
Earlier compile/fixture/trial failures are not counted as driver reds.

| Group | Actually executed observations |
|---|---|
| Entry and Into priorities | Five Rust/C forms bypass a held context today; both Into conversions also lack entry exclusion. Separate release positives pass all five forms. Invalid counted names precede both conversions; stale/directory/MAX_LINKS source precedes parent Into; volume and C null/ceiling/bounded-path priorities pass. |
| Ordinary refusals | Sixteen counted cases × Rust/C × clean/dirty: **64 observations**, plus **18 pathname refusals**. Expected errors execute first. All 32 clean counted cases and both path groups currently change marker bytes; all 32 prior-dirty cases pass byte/health/pending-state preservation. Includes source/parent generations/types, duplicate/dots, 256-byte names, maximum links, corrupt source/directory, no-block parent growth and unsupported directory policy. Unsupported policy is an early refusal, **not executed casefold growth**. Separate refusal/reuse and 64,999→65,000 boundary positives pass. |
| Reentry | Eight real callback profiles: read, size, writability, marker/commit writes and flushes, and same-thread read. Each reaches **eleven callback-free refusal expectations** (ten qualified families plus link). Both source/parent Into hooks attempt all ten qualified workers. All ten existing qualified-family clock callbacks actually attempt link in the reverse direction. Current link has no clock/generation call; none is fabricated. |
| Measured uncertainty | **48 successful-call measurement rows** across six sweeps × eight direct/journal × 1/4 KiB × clean/dirty profiles. Actual Rust and C calls measure **52 writes + 24 flushes** per API; 24 legal Rust flush unwinds per API are also injected. **200 reached positions**, all expose missing complete mount retirement. Checks attempt all eleven families, flush and close after each injection; write errors occur after changing bytes. No create injection count is reused. |
| Publication | Four direct/journal × 1/4 KiB public calls poison publication only at the first commit flush, after planning-state reads. Each proves the **entry and source nlink are already committed** before asserting retirement; all four currently fail retirement. An early poisoned planning lock is not substituted. |
| Positive storage and identity | **54 primary profiles**: 30 plain five-form × ext2/3/4 × 1/4 KiB, 12 actual full-parent growths, eight inline/conversion and four unknown-parent-link profiles. Existing source raw fields (allowing later separately measured ctime), data/EA/owners/generation, parent counts, peer bytes, inode bitmap and exact block/inode/group accounting are checked and reopened. |
| Additional positives | Existing regular/fast/slow/dangling symlinks and four constructed supported special types; raw source/destination bytes and 255-byte names; path/final-symlink/trailing-directory requirements; real shared-table source/growing-parent slices; legacy single-indirect and extent growth past twelve blocks; ordinary htree leaf splits; fragmented depth-one extent promotion **and subsequent append** at both block sizes; two-group BLOCK_UNINIT growth with unchanged inode bitmaps/counts/itable-unused/used-dir state and retained INODE_UNINIT, block CRC/publication/reopen. |
| Optional C readback | All four direct/journal × 1/4 KiB profiles execute normal readback, returned read error and legal Rust device-read unwind. Normal attrs identify the existing target with nlink +1; a NULL-attr control measures the avoided reads. First-read failure returns EIO after exactly one committed alias/count increment with output sentinels unchanged and no allocation. Held readback and unwind retirement expose current safety gaps; no panic crosses an extern-C callback or aliased mutable handle. |
| Allocation-free staging | Both 1/4 KiB actual insertion/nlink staging helpers produce no staged primary SB or allocation overrides; the public call succeeds without allocation. This is **not a newly invented LinkPlan helper**. Optional-SB/growth-debit/private-boundary diagnostics wait for the real migration helper. |

Both final focused tiers return their expected **101** for the saved driver reds;
output remains complete within the separate 1,600-line/170,000-byte diagnostic
caps. These do not modify normal CI budgets. Run A is **944 lines /
64,263 bytes**, run B **944 / 64,263**. The verifier compares
complete outcomes and measurement identities/counts, not just totals. All
compatibility/positive cases must actually pass, all callbacks must be reached,
and every sweep/publication profile must finish. Self-built fixtures prove
self-consistency only; they do not independently qualify emitted bytes or a card.

### Restoration, checks and replay

The final patch is reversed after the repeated run. Active source/test/workflow/
manifest/lock bytes match the baseline hash manifest; the temporary module is
absent. Fresh restored `chore test:unit` retains **all 1,503 baseline successful
occurrences** as a multiset, all **526 controls exactly once**, zero failed or
ignored. Full `chore lint` passes; dormant all-target/CLI clippy also passes.
The pinned immutable core remains v0.3.0; the already documented relative-path
source view restores Cargo.toml afterward and asserts unchanged Cargo.lock.
No PRoot/ptrace permission is needed by this native adapter.

Local ignored receipts are `tmp/casefold-link-red/{source-manifest,outcomes-a,
outcomes-b,retention}.json`; complete logs are `tmp/logs/link-red-final-a.log`,
`link-red-final-b.log`, `link-red-outcomes.log`, `link-red-lint-final.log`,
and the saved baseline/restored unit logs under that receipt directory. They
are not guaranteed in a fresh workspace; the committed patch is reproducible.
With the prescribed pinned source view and tools, apply the patch and run:

```sh
git apply --check docs/CASEFOLD-LINK-RED.patch
git apply docs/CASEFOLD-LINK-RED.patch
scripts/tier.sh link:red link-red 1600 170000 -- cargo test --locked --lib fs::guarded_chmod::tests::link_calls:: -- --nocapture --test-threads=1
# Expected: exit 101; 20 pass / 64 fail / zero ignored.
# Reverse/reapply, verify bytes and repeat before restoring:
git apply --reverse docs/CASEFOLD-LINK-RED.patch
chore test:unit
chore lint
```

### Next bounded task and limits

Add and execute independent **reference-only Linux hard-link/linkat-zero-flags
policy** measurement inside the existing pinned CI VM. Linux creates all bytes;
the driver initially reads only closed images. Measure source ctime, destination
parent mtime/ctime, old-parent/peer preservation, existing source identity/data/EA,
counts/accounting and source versus destination/symlink/trailing/name/error
priorities. Retain before/after images, complete reports, judged non-repairing
e2fsck and cold kernel records; audit saved bytes before measured native policy
reds and patch composition. Keep explicit API/FFI contracts separate from syscall
observations. Guard/owned-plan/optional-SB/complete effect/C readback migration,
real helper diagnostics and independent Rust/C emission qualification follow.

No link guard or time repair is implemented; no new Linux reference/emission,
full-suite, crash/power-loss or Windows/card outcome is claimed. The dormant
patch must stay out of normal CI until repaired/promoted. Unlink/rmdir/mkdir/
create patches remain historical/promoted and must not be reapplied. Stages 4–5
remain **not yet quantified**. Casefold writes stay blocked; broader ownership/
alias/admission, complete mapping/index growth, seals/lifecycle, recovery/ignored
fuzz and Windows/SteamOS/real-card qualification remain open.

## Public hard-link staging, marker and dispatch design audit — 2026-10-27

Started clean at fetched `c15e588e721ee6300c6dee86214d3a04b0939ec6` using Sol 6.1
and the existing checkout. Exact full CI **40476503989** is in progress at
inspection; no completion is inferred. Production/test/workflow/manifest/lock/
agent bytes match the independently qualified and fully verified `38295af`
source. Its full CI `40080977121`, 2,337-pass complete-suite/526-control retention
and actual floors/twenty shell/159 installed-CLI checks remain applicable.
This checkpoint changes only this existing handoff and the casefold review.
No fresh native test suite or oracle execution is claimed for unchanged code.

The [focused design review](CASEFOLD-INDEX-REVIEW.md#public-hard-link-staging-marker-and-dispatch-design-audit--2026-10-27)
traces all five Rust/C entry forms, C pointer/path/name/generation contracts,
two independent Rust Into hooks, source/destination identity, nlink checks,
shared helpers, ordinary insertion/growth and complete commit/publication.
Important observed distinctions from regular-file creation:

- Counted link validates names **before both conversions**; target conversion,
  source liveness/type/MAX_LINKS precede destination conversion/liveness/type
  and duplicate checks. Path destination syntax/final-length precedes source
  lookup, while source-directory refusal precedes destination walking. Both
  volume verdicts and explicit path/root-walk/trailing/final-symlink rules stay.
- Link reuses a live non-directory inode. Source generation, payload/maps,
  owners/mode and EA reference ownership stay; only source nlink is currently
  patched with its covering CRC. No new inode or source data/EA block is needed.
  Parent link count stays. Current source/parent timestamp behavior is an audit
  candidate for advancing Linux measurement, not a measured semantic red.
- The existing body commits a complete buffer in both insertion/growth branches,
  despite its stale “Not journaled” comment. Inline conversion, index/legacy/
  extent growth and allocator publication must stay in that same transaction.
  Same-table source/parent splices must not overwrite one another.
- **An allocation-free link can omit the primary SB from its buffer.** The future
  link-only boundary validates/dirties/restamps a full SB only when present;
  create's mandatory-SB check cannot be copied. Real growth/inline-conversion
  fixtures must prove their SB/debits are retained. This is trusted internal
  completed staging, not an ownership seal.
- Optional counted C attrs refer to the **existing target** after success, and
  can fail after one actual link/count increment. The same context must cover
  requested owned readback; NULL avoids reads, errors preserve output sentinels,
  and legal unwinds retire without retry or rollback claims.
- Existing link code has no direct clock/generation call. Current safety controls
  must use reached conversion/device/effect hooks; new clock controls belong
  after actual measured timestamp work. Future private-SB diagnostics wait for
  the real helper and must not count legitimate no-SB plans as public failures.

The chosen migration keeps shared nlink/allocator/insertion/path/raw-commit and
existing ten qualified families unchanged. All five link forms borrow one mount
context through completed LinkPlan and optional-SB preparation, marker, whole
raw commit/checkpoint/direct flush/allocation publication and requested C owned
inode snapshot. Direct ordered writes do not establish power-loss atomicity.
Casefold insertion/index-dropping/write refusal gates remain intact.

**Next bounded task:** actually execute a native dormant safety/positive patch
using asserted ext2/ext3/ext4 × 1/4 KiB profiles, every entry/priority/dual-Into,
clean/dirty byte-preserving refusal, ten-family callback directions, real no-SB/
same-table/inline/conversion/growth/uninit identities/accounting and optional C
readback. Measure successful link writes/flushes before sweeping every actual
position; do not import create's 304 injection count. Real public publication
unwinds must prove committed entry/count before retirement. Save complete logs,
names/outcomes and actual counts; reverse/reapply to identical source and repeat,
then restore production/tests and retain all **1,494 native occurrences / 526
controls**, zero failed/ignored, with lint passed. Keep intentional reds dormant.
Compile/fixture/unreached-hook failures are not link defects; no fake commit
helper may be added just to compile future diagnostics.

After that preparation, independently measure Linux-only hard-link policy and
audit saved bytes, then execute measured semantic reds/patch compositions before
bounded guard/policy repair. Actual Rust/C emitted-image qualification, full CI
retention and standalone source/evidence review follow. The design specifies
these obligations without claiming any of them executed here. Source-census and
shared-agent/provenance/whitespace checks validate this docs-only checkpoint;
no production, tests, workflows, task/cap/floor, pin, lock, sibling or agent-guide
change occurs. All create/mkdir patches remain historical/promoted and must not
be reapplied. Stages 4–5 remain not yet quantified; ownership/alias/mapping/index
growth/seal/lifecycle/recovery/ignored-fuzz and actual Windows/SteamOS/card gates
remain open. Casefold writes stay blocked.

Local convenience records are `tmp/casefold-link-audit/{baseline-ci.json,
source-audit.json,check_source.py}` and `tmp/logs/link-{source-census,
audit-docs,audit-final}.log`. The census checks actual dispatch order, shared
commit branches, current absence of link guard/plan/runtime/inode-allocation
calls, unchanged implementation and the docs-only change set. It is not a native
safety/semantic red or a new Linux result. Scratch files may not survive a new
runtime; current committed source and exact CI metadata are authoritative.

## Scoped create full-CI review closure — 2026-10-27

Started clean at fetched `38295af4cc23ec93253c9edd963cffde73bbd692` using Sol 6.1
and the existing checkout. Its exact [full CI 40080977121](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40080977121)
now passes **all eleven jobs and ci-ok**. The actual completed native job is
**120834735462**, guest **120834735439**, architecture **120834735417** and aggregate
**120846275433**. This checkpoint verifies the repaired wrappers, complete test
retention and previously unrun trailing checks, then closes the scoped review.
No production/test/workflow/task/limit/floor/pin/lock/sibling or write-gate change.

### Exact-source complete archives and retained occurrences

All five downloaded tier archives verify source `38295af`, run `40080977121`,
non-expiry, advertised compressed SHA-256, safe member paths and every member CRC:

| Artifact | ID | Compressed bytes |
|---|---:|---:|
| `tier-logs-x86_64` | 12556797255 | 89,876 |
| `tier-logs-suite-in-vm` | 12556410910 | 42,824 |
| `tier-logs-cli` | 12555710043 | 413 |
| `tier-logs-aarch64` | 12555681426 | 33,753 |
| `tier-logs-unit` | 12555361568 | 23,700 |

Complete successful-name Counter/multiset comparisons retain **all 2,337 previous
whole-suite successes** in both native and guest, zero failed and **one unchanged
ignored slow fuzz case**, `deep_walk_byte_flip_fuzz_never_panics`. No test is
replaced by a larger aggregate count. All **526 operation/guard controls occur
once in five contexts**: unit, aarch64 unit, native unit, native whole and guest
whole. All three unit contexts retain **1,494 successful occurrences**, zero
failed/ignored; architecture images retain all **527** passes and the same
pre-existing ignored fuzz case. This closes the scoped CI prerequisite, not the
broader ignored-fuzz or exhaustive filesystem qualification gate.

Actual native whole output is **3,746 lines / 223,509 bytes**, within unchanged
**3,950 / 238,000** repaired caps. Complete-log SHA-256:
`bad8905e99adddfde4fb63f4222051f7beca005725adbbbf0ad15e358fa6dc74`.
Actual guest output is **3,915 lines / 232,808 bytes**, within unchanged
**4,120 / 248,000** caps. Complete-log SHA-256:
`086b47bcab3963e2e63b4309f2afe4dd5fa6db5bff6fddf2c1983626094cdf7f`.
Both actual wrapper success lines and **2,337 executed / floor 1,060** are verified
in the downloaded completed job logs; these are new successful wrapper executions,
not only the preceding identical-log replay.

### Actual trailing checks now execute and pass

The native log now contains `scripts.log` and `cli.log`, previously absent after
the output gate stopped the job. **All twenty shell checks pass**, retaining the
exact prior script names/verdicts. Native installed CLI executes **159 checks
across six files**, zero failed, with its actual **159 executed / floor 150**.
Standalone installed CLI independently retains the same six-file **159 passes**.
The unit/Unicode/oracle/kernel/lwext4 execution floors also pass with actual
**1,494 / 8 / 225 / 33 / 9** respectively. These results establish that the new
log budgets let the entire task finish; no selected test, floor or output was
removed to achieve a green aggregate.

### Scoped source/evidence closure and remaining gates

A source comparison proves `src`, `tests`, workflow files, dependency manifests/
lock and agent guide match independently audited `f436f4e`; only the previously
reviewed measured whole-suite cap/description and handoff changes distinguish
`38295af`. The existing **sixteen-layout / 112-image / 48-creation** Rust/C emitted-
image audit, all 544 debugfs/432 cold Linux records, exact allocation/checksum/
parent/peer/journal checks and every retained-family audit remain applicable.
No unchanged multi-gigabyte image audit is rerun merely for docs/cap differences.
The preceding completed source walk and real-boundary/native/fault controls also
remain applicable. No host oracle, mount or KVM is used in this checkpoint.

No further blocker is found within **caller exclusion, healthy layouts and
closed external reference handles**. **The scoped regular-file create source/
evidence/full-CI review is closed.** All five forms use the existing context,
completed owned staging, create-only full-SB boundary, complete effect/publication
latch and optional C owned readback under the reviewed contract. Ordered direct
writes are not a power-loss atomicity claim; normal optional readback errors may
follow exactly one committed creation. Both create patches and both mkdir
patches remain **historical/promoted; do not reapply** to current source.

This closure does not finish Stage 4 or Stage 5. Casefold writes remain blocked.
Ownership/aliases, complete mapping/index growth/seals, unguarded writers,
retained handles, lifecycle/maintenance/C ownership, recovery, ignored fuzz and
actual Windows/SteamOS/card qualification remain open; stages 4–5 are not yet
quantified. The qualified emission matrix uses asserted non-growing parents;
its conclusions do not admit arbitrary corrupt/mutating ownership or all growth
shapes. Existing broader tests remain evidence for their explicit tested cases.

**Next bounded task:** audit public hard-link (`link`) creation in Rust/C, tracing
source and destination identity/generation/priorities, staging and shared-helper
callers, marker and complete commit/allocation publication, optional C readback
and current growth/refusal contracts. Record the bounded migration design and
required executed controls/independent evidence before new native reds or guard
migration. Do not claim that audit already complete or relax write gates.

Local convenience evidence: `tmp/casefold-create-closure/{run.json,jobs.json,
artifacts.json,tier-logs-*.zip,tier-logs-*/,native-job.log,guest-job.log,
check_retention.py,retention.json}` and `tmp/logs/create-closure-retention.log`.
Complete archive digests remain in the saved exact-source API metadata. Scratch
copies may not survive a new runtime; committed tests and immutable exact-source
CI artifacts allow retrieval/reproduction. Shared-agent/provenance and whitespace
checks validate the documentation checkpoint before committing.

## Independent regular-file create emission evidence and saved-byte audit — 2026-10-24

Started clean at fetched `f436f4e0bb3a220f3f25dfb62cfebd2fc8041bff` using Sol 6.1,
the existing checkout, pinned tooling and current chat's verified cloud route.
Only the measured whole-suite output caps and handoff/review change in this
checkpoint. No environment configuration, credential, driver, test, workflow,
pin, lock, sibling, unit cap, selection, floor or write gate changes.
Both create patches and both mkdir patches remain historical/promoted: **do not
reapply them to current source**. The prior native red-to-green and 304-fault/
four-publication-unwind evidence remains applicable to these unchanged bytes.

### Exact independent job and artifact provenance

[Focused run 40075365381](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40075365381)
executes `needs_host::create::linux_checks_public_create_metadata` successfully
in job **120817437070**, at exactly `f436f4e`. All its steps pass. The completed
job log proves **one executed test / floor one**, zero failed/ignored, and a
passing actual wrapper at **893 lines / 113,247 bytes**, below unchanged
3,200 / 200,000 caps. This is actual driver-emitted Linux evidence, distinct
from the earlier reference-only Linux create observations.

Artifact **12553351249**, `create-emission`, is **4,626,763 compressed bytes**,
**3,758,629,943 uncompressed bytes**, **1,329 ZIP members**, SHA-256
`4d74027244a5836504286224bc1e2e11ddb3f01cae08017dc6c92484e0be02e7`.
Exact source/run metadata, GitHub's advertised digest, member path safety and
**every member's CRC** are verified. Images stay compressed during the read-only
cloud audit; no host oracle, mount or KVM is used. Complete reports, JSON records,
images and logs are retained in the actual CI artifact.

### All sixteen emission profiles and 112 saved images pass

The full **Rust/C × 1/4 KiB × 128-byte legacy or 256-byte checksummed extent
inodes × direct/journal** matrix executes, with seven saved phases per layout:
Linux-created input and first/second/255-byte creation, each dirty and clean-
closed. The actual totals are **112 images / 48 creations / 496 refusal-policy
observations**, plus C pointer checks. The complete test log proves **sixteen
mke2fs, 112 judged e2fsck, 544 debugfs and 64 guest mount invocations**. All
**544 complete inode reports and 432 cold Linux metadata records** match raw
saved inode identity/type/mode/owner/link/size/sectors/times and mapped blocks.

Every actual new file is regular, one link, zero size/sectors/data/EA, owner/group
zero and generation 17. Exact requested **02750 / 01770 / 0750** survive all
sixteen layouts. Byte-name and fresh-parent 255-byte identities are preserved;
all back-to-back inode identities are distinct, initially free and subsequently
allocated. Legacy direct/indirect pointers are zero; extent profiles have the
expected empty depth-zero root. Child timestamp/checksum bytes survive dirty to
clean close and cold lookup/readback. Independent Linux reads each empty file.

The audit proves **280 exact block-bitmap byte-preservation checks** and **280
exact inode-bitmap/itable-unused delta checks**, including unaffected groups and
stable bitmap/table addresses. Only the actual new inode bits change; per-group/
SB inode debits are exact, while free blocks and used-directory counts stay
unchanged. This matrix deliberately uses asserted non-growing parents; it does
not independently qualify every parent-growth/inline/index shape or casefold
insertion. The existing native growth/inline/same-table/uninit controls remain,
and broader full-suite independent retention is still pending.

Parent mtime/ctime follow each advancing creation stamp; atime, link counts,
permissions/owners/generation/flags, block mappings, storage and EA remain intact.
A raw-byte comparison permits only parent mtime/ctime and their covering checksum
fields. Protected peer and pre-existing inode bytes, payloads, EA blocks, symlink
contents and namespaces remain intact. Dirty/closed inode bytes agree; cold Linux
reports match canonical names, raw-byte names and exact saved metadata. Empty-file
and peer reads and directory listings use O_NOATIME to avoid observer changes.

Checksummed profiles pass **56 primary-SB, 140 descriptor, 2,084 inode-view,
336 directory-block, 240 EA-block, 140 block-bitmap and 126 inode-bitmap checksum
checks**. All **56 journal headers** are clean/checkpointed where present,
including **28 journal-SB checksum** checks. Dirty/clean primary state changes
as required; RECOVER stays clear after completed checkpoint, all independent
non-repairing e2fsck verdicts are clean, and free counts agree with Linux.

### All retained-family image audits pass at the exact implementation source

All ten jobs in [focused run 40075365381](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40075365381)
now pass, including every one of the **fifteen exact main focused test logs**
with one executed pass / zero failed or ignored. Main artifact **12553098141**,
`casefold-fixtures`, has verified exact source/advertised digest and all **1,823
member CRCs**, covering **9,663,047,962 uncompressed bytes**. All image audits stay
read-only and compressed in this bounded workspace.

Fresh audits at `f436f4e` pass for every retained family: four private/eight public
chmod, sixteen chown, twenty-four timestamp and sixteen flag profiles; sixteen
setter emission profiles/192 operations/64 preserved peers and eight first-
attribute profiles; sixteen xattr-removal profiles/112 removals; public unlink/
rmdir/mkdir emission; independent Linux create/mkdir/unlink/rmdir references;
setter-policy reference; and all five symlink boundary/traversal cases with
saved-image comparisons. These are actual current-source byte/report audits,
not results inferred from the preceding implementation. Existing timestamp
observer/no-op/legacy-display qualifications remain explicit in their audits.

### Complete native/guest/architecture retention passes; actual wrappers fail caps

[Full CI 40075365376](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40075365376)
passes minimum-version build, standalone CLI, fixture preparation, public API,
provenance, browser, units and aarch64. Actual native job **120817935325** and
guest job **120817935286** fail with **exit 65 solely on measured output caps**;
ci-ok therefore fails. Source/digest/all-CRC-verified unit, aarch64, native,
guest and CLI archives are downloaded and compared against complete prior
successful-name multisets, not just aggregate counts.

Both whole suites retain every previous **2,228** successful occurrence and add
exactly **108** promoted/new native controls plus **one** independent create
emission: **2,337 passes / zero failed / one unchanged ignored slow fuzz test**.
All **526 operation/guard controls occur once in five contexts**: unit, aarch64
unit, native unit, native whole and guest whole. All three unit contexts retain
**1,494** successes, zero failed/ignored; aarch64 images retain all **527** passes
and the same pre-existing ignored fuzz case. Standalone CLI retains **159 checks
across six files**, zero failed. The native actual unit/Unicode/oracle/kernel/
lwext4 floors pass with **1,494 / 8 / 225 / 33 / 9** executed respectively.

The whole-suite wrappers stop execution before their 1,060 floors, native twenty
shell checks and native installed-CLI phase. Those checks are **unrun**, not
passed or test regressions. The separate installed-CLI job passes 159 checks,
but does not establish native trailing execution. Corrected full CI/ci-ok and
actual whole-suite floor/trailing-check qualification remain pending.

### Measured whole-suite output repair and identical replays

| Context | Actual complete output | Old caps (lines / bytes) | New caps (lines / bytes) |
|---|---:|---:|---:|
| Native | 3,745 lines / 223,395 bytes | 3,700 / 220,000 | 3,950 / 238,000 |
| Guest | 3,913 lines / 232,693 bytes | 3,900 / 230,000 | 4,120 / 248,000 |

Native complete-log SHA-256:
`068e07844e898badff695f428e59122e3862bcf703869ee5ebdae462f619dee8`.
Guest complete-log SHA-256:
`9155f84915b098430cc457cc37d5d83a7c7a4f19bf3a2035165ab2fc888e67c5`.
Both actual jobs and byte-identical old-cap replays fail **65**. Replaying the
same complete bytes with the new measured limits passes; all four replay files
compare byte-for-byte to their original CI logs. Only these two cap calls and
their measured table/explanation change in `chores.yml`; floors, selection,
commands, output, unit caps and independent-test budgets are retained. This
repairs the output gate, not a driver test or independent filesystem finding.

Fresh **94 CI-profile/test-contract checks pass** after the cap edits (87 profile
and seven contract cases), with zero failed/ignored. Shared-agent/provenance and
whitespace checks pass. A normalized comparison of `chores.yml` proves its only
changes are the two measured cap calls and their explanation; all command text,
selection, output and floors are retained. No host oracle or VM is used for this
replay/configuration validation.

### Scoped source/evidence review and next bounded task

The fresh source walk traces all five entry forms, context identity/health,
Into/name/FFI/path priorities, shared partial allocator callers, completed create
buffer, insertion/growth and latest parent inode splice, full primary-SB dirty
preparation, complete marker/commit/publication effects and optional C owned inode
readback. Existing guarded-family/effect and raw commit code is retained. Actual
emission, retained-family and full-CI occurrence evidence now passes. No further
scoped source/evidence blocker is found under caller exclusion, healthy layouts
and closed external reference handles. **Final scoped create closure remains
pending** on the corrected full CI and actual floors/native trailing checks.
The plan remains trusted internal staging rather than a generic ownership seal;
ordered direct writes do not establish power-loss atomicity. Ordinary C readback
errors may follow exactly one committed creation; legal unwinds retire the mount.

**Next bounded task:** inspect this budget-repair commit's exact full CI, repair
actual failures, retain every 2,337 whole-suite success and all 526 controls in
five contexts, prove both actual whole-suite floors and the twenty shell/159
native CLI trailing checks ran, and close the scoped create review only after
ci-ok and these checks pass. Driver/test/workflow bytes are unchanged from the
independently qualified `f436f4e`; no unchanged large image audit needs repeating
solely for documentation/cap changes. Do not start another public mutation family
or relax the write gate before this review closure. Ownership/aliases, mapping/
index growth/seals, unguarded writers, retained handles, lifecycle/maintenance/C
ownership, recovery, ignored fuzz and Windows/SteamOS/card remain open. Stages
4–5 are not yet quantified; casefold writes remain blocked.

Local convenience evidence: `tmp/casefold-create-emission/{artifact.json,
create-job-final.json,create-job.log,image-audit.json,checkpoint-evidence.json,
full-retention.json,40075365376/,focused/,audit_emission.py,
check_full_retention.py,audit_available.py,audit_remaining_available.py,
audit_main_retained.py,available-summaries.json,remaining-summaries.json}`;
`tmp/logs/create-{emission-audit,retained-available,retained-remaining,
main-retained,full-retention,native-old,native-new,guest-old,guest-new,
budget-contracts,budget-doc-checks}.log`.
Scratch copies are not guaranteed in a fresh runtime; use committed tests and
exact-source CI artifacts to retrieve/reproduce the evidence.

## Guarded public regular-file creation and measured policy repair — 2026-10-24

Started clean at fetched `e2d1cc0e8217b4c77cb5e1949184e699860659d7` using
Sol 6.1, the pinned Rust/core source view and the existing isolated checkout.
Preceding [full CI 40053837295](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40053837295)
and [40051856797](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40051856797)
both report completed success at their exact source commits. A fresh normal
native baseline retains **1,386 successful occurrences / 418 controls once**,
zero failed/ignored. The eight-layout Linux create reference and audited raw
bytes at `e384ed9` remain the independent policy basis; no unchanged reference
archive is reaudited in this checkpoint.

### Reproduced reds and bounded production repair

Both compatible dormant create patches were applied before the fix. The actual
combined run repeats **all 99 saved names and outcomes: 28 passed / 71 failed /
zero ignored**, exit 101, complete log **1,234 lines / 92,526 bytes**. A checker
compares the complete outcome map to the preceding saved composition. No compile
or fixture failure is counted as a driver red. The first repair run turns the
same 99 controls green before adding real boundary diagnostics.

Regular-file create now follows the reviewed bounded design:

- Rust string/byte/counted and C path/counted forms enter one existing mount
  context before device/runtime/arbitrary conversion hooks. Nested helpers
  borrow it and retain both pure volume verdicts. Counted create keeps **Into
  before name validation**, then name before live parent identity/type. FFI
  pointer/ceiling priorities, byte/255-byte names, low-twelve explicit modes and
  existing separator/root-walk/trailing behavior stay intact.
- `NewInodePlan` remains shared partial inode setup; `mknod`, symlink and generic
  path helpers are unchanged. New private `CreatePlan` owns the completed child,
  entry, any supported parent growth and all allocator edits. Only OutOfBounds
  requests growth. After insertion/growth, parent mtime/ctime are patched through
  the latest verified staged inode; its atime, links, mappings/inline edits,
  attributes and sibling child-table slices are retained.
- The create-only commit boundary checks context identity/health and a full
  staged primary-SB block with valid offset/shape/magic before effects. It clears
  only the clean bit, preserving allocation debits and other bytes/state, and
  refreshes the checksum. One-time marking and the entire raw commit/checkpoint/
  direct flush/allocation-publication call use the existing failure-latching
  effects. Shared commit/effect machinery and qualified families are unchanged.
  Ordered direct writes remain distinct from power-loss atomicity.
- The counted C adapter retains that context through requested owned inode
  readback, then formats fields without filesystem access. NULL avoids the read;
  a normal read error may follow exactly one successful creation and leaves the
  attr sentinel intact. A legal readback unwind retires the mount. Signatures,
  generation/errno mappings and return conventions are unchanged.

### Native safety, policy, new real-boundary controls and retention

All **108 controls pass**, zero failed/ignored: every promoted 72-control safety
and 27-control policy name plus **nine new actual-boundary controls**. The latter
cover four constructed full-SB direct/journal × 1/4 KiB debit/state/checksum/
outside-byte cases; foreign contexts and missing/malformed staging; returned
marker/commit errors while the context is retained; publication unwind before
retained-context destruction; second pure verdict before regular-prefix/length
policy; and a flush unwind caught while retaining the context. These constructed
SB cases diagnose the real new private boundary, not an existing pre-fix public
clean-bit bug. No fake helper is introduced.

The actual successful repaired call is remeasured for each Rust/C direct/journal,
1/4 KiB, clean/already-dirty profile before injecting every observed position.
Each interface now exercises **96 returned write faults, 28 returned flush faults
and 28 flush unwinds**, **304 total** across both interfaces, zero retirement
violations. The old 288 count is not copied over the added parent-inode staging.
All **four public post-commit allocation-publication unwinds** still verify the
committed namespace before proving mount retirement. Qualified metadata/removal/
mkdir/create families, flush and close remain excluded after uncertainty.

All **54 original positive primary storage profiles** and same-table, inline,
full-parent, constructed 2 KiB/two-group/uninit and C-readback controls remain.
Every measured policy profile completes: **270 successful creations / 714
refusal/priority/syntax observations**, with parent times, empty regular inode
identity, peers/EA, explicit modes/names and API priorities preserved. Complete
focused log: **969 lines / 67,158 bytes**, SHA-256
`a442d86fde7d938147fc69900942c7ac38105d3a4bf2f8b3ec0812fd3f33c267`.

Normal units execute **1,494 passes**, zero failed/ignored. Counter/multiset
comparison retains every **1,386** previous successful occurrence and adds exactly
108 create controls. All **526 operation/guard controls occur once**, preserving
all 418 previous controls. Full lint passes, including tracked new files,
shared-agent/provenance, Unicode, formatting and all-target clippy; all **94
CI-profile/test-contract checks** also pass. Guest Python syntax passes. The
new independent emission test compiles as part of these native all-target runs;
its VM behavior is not credited before actual CI execution.

### Measured unit-only output budget repair

The actual normal unit suite passes all tests but its wrapper exits **65**:
**1,742 lines / 132,491 bytes** exceed **1,740 / 131,000**. Its complete retained
log has SHA-256
`7225429a3144e3b1f407fc90a45fc55507e0b631f2cb2e1f502692a037eb1063`.
Replaying exactly those bytes through the old caps repeats 65; replay through
**1,860 / 143,000** passes. Both replay logs compare byte-for-byte to the original.
Only the unit row/call and its measured explanation change. Test selection,
unchanged 440 floor, logs and output remain intact; no main/guest output cap is
raised speculatively. The normal task is then rerun with the repaired wrapper:
**1,494 executed / floor 440**, 1,742 lines / 132,491 bytes, wrapper passed;
its complete successful-name multiset matches the over-budget run. An exact
source comparison also confirms the previous guarded-family/effect code and
all original 99 control bodies remain unchanged.

### Independent driver-emission test added; evidence is pending

`needs_host::create::linux_checks_public_create_metadata` uses only the existing
CI VM/harness for independent tools and cold Linux readback. It plans **sixteen
layouts**: Rust/C × 1/4 KiB × 128-byte legacy or 256-byte checksummed extent
inodes × direct/journal. Each layout gets Linux-created input with protected
peers/EA, then three actual public driver creations: explicit 02750/01770/0750,
parent-symlink/trailing contract, counted raw-byte name and a fresh-parent
255-byte boundary name. Across the matrix this plans **48 creations / 112 saved
input, dirty and clean-closed images / 544 complete debugfs inode reports / 432
cold Linux inode records**, plus complete per-phase logs/events.

The test checks untouched duplicate/dot/overlong/mixed-prefix refusals, exact
regular-file kind/identity/mode/owner/link/zero-size/zero-sector/generation/times,
no child block, per-group/SB inode debits with unchanged block/used-directory
counts, parent atime/links/mapping/flags/EA and peer bytes. Clean close checks
primary clean/RECOVER/journal state and cold driver identity; judged non-repairing
e2fsck and Linux independently read saved outputs. Kernel opens use O_NOATIME
for directory listings and empty-file/peer reads. A separate focused job has the
exact filter, **one-test floor**, unchanged pins and complete artifact retention.
No host oracle, mount or KVM is attempted.

**Both `CASEFOLD-CREATE-RED.patch` and `CASEFOLD-CREATE-POLICY-RED.patch` are now
historical promoted artifacts; do not reapply to current source.** Their bytes
remain unchanged for compatible historical reproduction, like the mkdir patches.

Exact implementation focused/full CI, actual emission results and source/digest/
CRC/raw-byte audit, complete-suite occurrence retention, retained-family evidence
and standalone family review remain **pending**. Local passing self-built controls
alone do not qualify writes. Next inspect those exact runs, repair actual failures
or measured output overflows without suppressing coverage, audit every saved
profile and retained family, and review the source before scoped create closure.
Arbitrary ownership/aliases, mapping/index growth/seals, unguarded writers/retained
handles, lifecycle/maintenance/C ownership, recovery, ignored slow fuzz and actual
Windows/SteamOS/card qualification remain open. Stages 4–5 are not yet quantified;
casefold writes remain blocked. No sibling, pin, lock or write-gate change.

Local evidence: `tmp/casefold-create-repair/{baseline-unit.log,red-outcomes.json,
native-results.json,unit-over-budget.log,unit-retention.json,check_results.py}`;
`tmp/logs/create-repair-{red,green,native,results,lint,profile}.log`,
`tmp/logs/create-unit-replay-{old,new}.log` and the normal `unit.log`.
Scratch files are convenience copies, not guaranteed in a fresh environment;
committed controls and exact CI artifacts are the reproducible source.

## Executed native create policy red preparation — 2026-10-23

Started clean at fetched `365ffea346a381c2f600e0c1237031f40202717b` using Sol 6.1,
the pinned Rust/core source view and the existing isolated checkout. A fresh
normal unit baseline executes **1,386 successful occurrences / all 418 controls
once**, zero failed/ignored. Preceding exact full run `40051856797` remains
in progress at inspection. The already verified `e384ed9` focused/full CI,
eight-layout/48-image Linux create reference audit and 2,228-pass whole-suite
retention remain applicable; no unchanged large image audit is rerun here.

### New executed policy patch

The dormant [measured policy patch](CASEFOLD-CREATE-POLICY-RED.patch) adds only
`policy_create_calls` and its test registration. It uses actual public APIs and
the existing native Device/Clock/C machinery; no production correction or fake
private boundary is introduced. Its actual isolated run executes **27 names:
fourteen passes / thirteen failures / zero ignored**, exit 101, complete log
**120 lines / 21,877 bytes**.

All thirteen reds collect mismatches over **ext2/ext3/ext4 × 1/4 KiB**, completing
every profile before the aggregate assertion:

- Ten first/back-to-back parent timestamp controls cover Rust string/bytes/
  counted and C path/counted forms. All **ninety successful creations** complete,
  with child regular/empty fields, correct one-inode/no-child-block accounting,
  parent links/storage/atime/EA and peer metadata/payload retained. Parent
  mtime/ctime remain old instead of following each advancing creation clock.
- Three Rust string/bytes/C path controls execute **108 mixed-path observations**.
  A regular-file or missing prefix currently loses to the final-name-length
  check; independent Linux requires ENOTDIR or ENOENT first. Valid directory/
  symlink/dot/dotdot prefixes retain their length refusal. Use existing symbolic
  errno mappings; the Linux numeric values do not redefine this API's constants.

Across the complete policy module, **270 successful creations** and **714
refusal/priority/syntax observations** execute. Positive coverage retains:

- All thirty fresh-parent, non-growing 255-byte names and 24 raw-byte identities
  cold, with one inode/no child data block consumed; all ninety explicit
  02750/01770/0750 mode cases survive across all five forms/six profiles.
- All 240 ordinary existing/dot/long-name refusal observations preserve allocated
  metadata and counters. Safety's earlier-marker contract remains tested by the
  separate original patch; policy checks do not mistake SB marker bytes for
  a new timestamp/namespace semantic defect.
- Counted names precede stale-parent checks, while **caller Into conversion
  precedes counted-name validation**. The 24 conversion observations preserve
  that create-specific order; volume/FFI priorities and symbolic errors remain.
- All 234 path-form refusal/syntax observations and **36 successful** parent-
  symlink/trailing and non-leading-slash rooted-parent creations preserve the
  explicit current API. Empty/separator-free paths stay EINVAL. Linux empty/
  trailing results remain recorded reference facts rather than a silent rewrite
  of these contracts. Generic `resolve_new_parent`, `mknod` and symlink remain
  unchanged; the measured future prefix repair is local to regular create.

These are executed native semantic reds grounded in the already verified Linux
reference, not a claim of independent driver-emission qualification. No fixture
or compilation failure is counted as a policy red.

### Both patch orders, complete safety retention and restored baseline

The new policy artifact is **30,406 bytes / 812 lines**, SHA-256
`36d6d325594c8548a2ee6b7b7a4b1a9712fc2628f41e7894733f27b567b9ab32`.
The original safety patch remains byte-identical at
`c5078050e8cfd3aaca199f9743265763422030e34d7dc73021a3791ac8b9a77b`.
Policy→safety and safety→policy both apply on the recorded compatible source
and recreate **identical complete source**, verified by hashes of both modules
and their registration. Each actual combined run executes the same **99 names/
outcomes: 28 passed / 71 failed / zero ignored**, exit 101, complete logs
**1,014 lines / 80,065 bytes** each. Both successful/failing name maps match;
thread identifiers/timing need not make complete log bytes identical.

The combined checker retains every original **72 safety outcome**, all **288
actually measured I/O fault injections**, all **four real public post-commit
allocation-publication unwinds**, all 54 safety-positive primary storage profiles
and every new policy outcome/observation. The 71 failures are current public
safety/policy assertions. Foreign/missing/malformed-SB and retained-context helper
controls still wait for the real promotion boundary; there is no constructed
future diagnostic counted as an existing public clean-bit bug.

Full `chore lint` passes **with both audits active**, including formatting,
Unicode, shared-agent/provenance and all-target clippy with warnings denied.
Both patches are reversed again; production/test/workflow/task/lock bytes are
restored. The normal native unit task reruns and retains every preceding
**1,386 successful occurrence / 418 controls once**, zero failed/ignored.
Dormant-patch applicability, shared-agent integrity, provenance and whitespace
checks pass. No production repair, global cap/floor/filter/output, pin/lock,
sibling or write-gate change remains active. Ordinary CI does not acquire the
intentionally failing audit modules. Both promoted mkdir patches remain historical.

Scratch evidence under `tmp/casefold-create-policy/` includes baseline/restored
unit logs, isolated policy and both complete combined logs, source/patch hashes,
active lint logs, `check_runs.py`, three per-name outcome JSONs and retention
results. To reproduce on compatible source, apply both dormant create patches,
wrap `cargo test --locked --lib create_calls:: -- --nocapture --test-threads=1`
in `scripts/tier.sh` (expect 101), lint, reverse both, then run normal units.
The common suffix selects exactly the two registered modules; the checker
asserts all 99 names. Never apply an already promoted historical patch or push
these audit modules active to ordinary CI.

### Next bounded task using Sol 6.1

Inspect this checkpoint's exact CI and repair actual failures before promotion.
Then implement the reviewed regular-create migration and measured policy repair:

1. Enter the existing shared context before device/runtime/arbitrary conversion
   callbacks; preserve both pure volume verdicts with private nested dispatch,
   counted **Into-before-name** and current mode/generation/FFI/path contracts.
2. Complete one owned inode/buffer plan with all allocations/insertion/growth;
   converge before parent mtime/ctime updates, reading the current buffered
   parent so child/shared-table/inline/growth edits and atime/links/EA survive.
   Keep other creation helpers unchanged and correct only the measured create-
   local regular/missing-prefix versus final-length priority.
3. Use a create-only complete staged-SB/context boundary preserving debits/state/
   outside bytes, then latch marker and the **entire** commit/checkpoint/direct-
   flush/allocation-publication call. Bind future diagnostics to that real helper.
4. Keep optional counted C inode readback in the same context and fill attrs
   from an owned snapshot; preserve NULL/no-read and possible error after one
   successful creation, with no retry/pretend rollback.

Reproduce saved reds before fixing; all original controls must become green,
new real private-boundary controls must execute, and normal native retention/lint
must pass. Raise a tier cap only after a measured actual overflow and identical
complete-log old-red/new-green replay, preserving floors/selection/output.
Then add/execute and audit ordinary Rust/C emitted-image Linux evidence before
standalone review/full-CI closure. Native positives/reference-only bytes do not
replace actual driver emission. Ownership/aliases/mapping/index growth/seals,
other writers/handles/lifecycle/recovery/ignored-fuzz and actual Windows/SteamOS/
card qualification remain open. Stages 4–5 stay not yet quantified; **casefold
writes remain blocked**. No host oracle/mount/local KVM or forbidden source is
used in this native checkpoint.

## Verified independent Linux create policy evidence — 2026-10-23

At clean fetched `e384ed99f1e71643ce1caa0c94511fa8707c9539`, exact focused
[run 40033593125](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40033593125)
passes all nine jobs. `create-policy-reference` job `120687552982` actually
executes one successful test, zero failed/ignored, **669 lines / 80,503 bytes**,
with floor **one** passed. The source-verified archive `12536511380` matches
advertised SHA-256
`d202498a3e751014d440c5b039b3c1f0792293d1e4177bf796d2ea3d5d958ec7`:
**2,017,293 compressed bytes / 1,611,015,769 uncompressed bytes / 553 members**.
Every member CRC passes; images are audited directly from the compressed ZIP,
without extracting the multi-gigabyte matrix or invoking a host oracle.

### Actual independent observations and saved-byte audit

All **eight layouts / 48 saved phases / 24 creations / 216 error observations**
pass. Real calls are **eight mke2fs, 48 kernel mounts, 48 judged non-repairing
e2fsck checks and 408 debugfs reports**. All 408 complete reports agree with
raw identities, type/mode/owner, flags/generation, times/crtime, size/sectors,
EA pointers and mappings. Final cold Linux metadata, exact byte namespaces
and empty contents agree with saved bytes for all three creations per profile.

The raw audit verifies 28 checksummed superblocks, 70 descriptors, 636 inode
views, 144 directory blocks, 104 EA checks, sixty block-bitmap and 54 inode-
bitmap CRC checks. Beyond counts/CRCs, all **120 group/phase block-bitmap
comparisons are byte-identical**, and all **120 inode-bitmap/itable-unused
comparisons** match only the actual newly allocated identities. New bits were
free in the input and are set in the output; bitmap addresses remain fixed.
Parent maps/flags/identity/owner, links/storage/atime and protected peers/EA
remain intact. Existing raw regular/directory peers, payloads and all original
namespace entries remain. Only symlink observer atime/covering CRC differences
are allowed outside those protected inode records. Dirty/recovery flags and
closed journal state are checked; all saved images are clean Linux references.

Each successful creation advances parent mtime/ctime while preserving atime,
parent links/size/storage and free blocks. Every file is regular, one link,
zero size/sectors/data allocation, owner/group zero under the recorded root/
zero-umask/non-setgid/no-default-ACL inputs. Per-group and SB counts charge
exactly one inode and no blocks/used-directory increment, using **actual inode
groups**, not assumed physical allocation choices. Final per-layout change is
**zero blocks / -3 inodes / zero used directories**, with unchanged parent links.

| Requested permission/special bits | Actual Linux regular-file bits | Profiles |
|---|---|---:|
| `02750` | `02750` | 8 |
| `01770` | `01770` | 8 |
| `0750` | `0750` | 8 |

These are the explicit exclusive-file observations; preserve the earlier mkdir
measurement as its own evidence. No mode stripping, credential/default-ACL
inheritance or umask policy is inferred for the Rust API.

All 27 errno observations repeat identically across the eight layouts. Names
and lengths are retained in JSON; grouped Linux results are:

| Actual Linux result | Recorded cases |
|---|---|
| `ENOENT` (2) | Empty name; missing parent before new/overlong name; missing parent before dot |
| `ENOTDIR` (20) | Regular-file parent before new/overlong name or dot |
| `ENAMETOOLONG` (36) | 256-byte name; existing directory/symlink-directory/dot/dotdot before overlong name |
| `EEXIST` (17) | Existing directory/file/255-byte file; symlink-directory/file/dangling; dot/dotdot with or without trailing slash; symlink-directory followed by dot |
| `EISDIR` (21) | Existing directory/file/symlink-directory/symlink-file with trailing slash |

In particular, regular/missing prefixes precede final component length. These
are measured exclusive-open facts under the recorded flags, not permission to
rewrite generic counted-name, FFI, stale-generation, empty-path or basename-
trimming/root-walk contracts. A Linux fd is not a Rust `Into<InodeRef>`, and
numeric errno values above are Linux observations; native/C tests must use
existing symbolic platform mappings. Keep accepted API shortcuts explicit.

### Exact full CI and complete baseline retention

Source [full run 40033593162](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40033593162)
passes all eleven jobs and ci-ok. Earlier design `0d38ff1` / native audit
`cdba11d` full runs `40025606526` / `40031383369` also now pass. Five current
full-CI log ZIPs match exact workflow source, advertised digest and every CRC.
Fresh successful-name **multiset** comparison retains repeated names and every
prior occurrence:

- Native/guest complete suites each retain all **2,227 preceding successes**
  plus exactly one new create reference: **2,228 passed / zero failed / one
  unchanged ignored** `deep_walk_byte_flip_fuzz_never_panics`. That ignored
  case remains a broader qualification gate.
- All three unit contexts retain **1,386 passes**, zero failed/ignored; all
  **418** guard/operation controls appear once in standalone/aarch64/native
  units and both whole suites: five contexts. Aarch64 retains all **527 image
  passes** and its same previously recorded ignored fuzz case.
- Native trailing **twenty shell checks** retain exactly the prior names and
  verdicts. All six installed-CLI files / **159 checks** pass in native and
  standalone contexts. Actual whole floors each execute 2,228 against 1,060;
  the native CLI floor executes 159 against 150. Other actual native floors
  retain unit/Unicode/oracle/kernel/lwext4 counts 1,386/8/225/33/9.

| Actual complete log | Lines | Bytes | Unchanged cap |
|---|---:|---:|---|
| Native whole | 3,635 | 212,417 | 3,700 / 220,000 |
| Guest whole | 3,802 | 221,719 | 3,900 / 230,000 |

Complete-log SHA-256 is
`feb9bca91e367800be0ca2aa9b28fe477fa874f9ae8cac530bb67cc3a6e75915`
(native) and
`b8c0ef7731441e5369309c077bdb21fc0e729e5e9c12f59455ec0047b9823a52`
(guest). Actual native/guest job logs `120688366237` / `120688366262` confirm
wrapper/floor success and trailing checks. No output cap/floor/filter or test
selection change is needed.

Verified advertised current tier archive identities:

| Archive / artifact ID | SHA-256 |
|---|---|
| `tier-logs-x86_64` / `12538503700` | `0eb76531b673e88724816f515f1d446a5e5e4940e8d49d083f237b7c9be1821d` |
| `tier-logs-suite-in-vm` / `12537727961` | `0d85585bcde7f026da78086649645df7a7180308472825a6a97a61fa6559d6fd` |
| `tier-logs-aarch64` / `12536716092` | `7254a7b40974e117418ea75af13bd18642a0a60ce428d97e3ccd1eef82fd441c` |
| `tier-logs-unit` / `12536705262` | `2977d4ad5a9017798e9034cd4b4c80aa10e936c98c60ed303854b309998f3089` |
| `tier-logs-cli` / `12535594216` | `0fdfedf83536a05ab183a63a4cb7b0b35093886e0f711fadebed1d1880b9acb7` |

### Scope, validation and next bounded task

The ordinary healthy reference audit is complete. It qualifies **Linux-created
reference bytes**, not the driver's write emission or a guarded create migration.
The dormant safety patch remains unchanged/inactive; all 58 native public safety
reds still require repair. Only this existing handoff and casefold review change
in the evidence checkpoint. Shared-agent, provenance and whitespace checks pass;
no production/test/workflow/task/limit/filter/pin/lock/sibling/write-gate change,
new native source test run, host oracle/mount/local KVM or forbidden source read
is claimed. Retained-family source/image audits need not be rerun for unchanged
implementation merely to document these new reference results.

Scratch evidence under `tmp/casefold-create-reference/` includes exact CI/job/
artifact metadata, verified reference/full-tier archives, complete job logs,
`audit_reference.py`, `image-audit.json`, `check_ci_retention.py`,
`ci-retention.json` and wrapped audit logs. The exact linked runs/archive digest
are authoritative; a new workspace can download them without inheriting local
scratch paths. Current managed policy is enforced; existing Git/public API and
redirected artifact reads work, without environment edits or new credentials.

**Next bounded task using Sol 6.1:** prepare and actually execute native measured
create policy reds for advancing parent mtime/ctime and regular/missing-prefix
versus final-length priority. Retain parent atime/links/storage/EA and completed
shared-table/inline/growth edits, explicit low-twelve-bit modes, generation/FFI,
counted **Into-before-name** and rooted/basename-trimming contracts. Keep generic
`resolve_new_parent` and shared `mknod`/symlink behavior unchanged; this is a
create-local semantic correction. Record syscall-only empty/trailing differences
rather than silently importing them into an existing explicit API contract.

Compose the new policy patch with the unchanged dormant 72-control safety patch,
verify both application orders recreate identical source, actually rerun complete
combined outcomes/fault coverage, lint with audits active, then restore source
and retain all **1,386 unit occurrences / 418 controls** before guarded repair.
Do not invent red/pass totals before execution or apply promoted mkdir patches.
Stages 4–5 remain not yet quantified; casefold writes, ownership/aliases/mapping/
index growth/seals/lifecycle/recovery/ignored fuzz and actual Windows/SteamOS/card
qualification remain blocked/open as previously recorded.

## Independent Linux create policy oracle added — 2026-10-23

Started from clean fetched `cdba11de85e69e7652e1358d995e515e60057bcb` on
`casefold-rw`, using Sol 6.1 and the existing pinned source view. Its preceding
full run `40031383369` and design run `40025606526` were still running at initial
inspection; no completed result is inferred. The executed dormant native safety
patch remains byte-identical/inactive: 72 controls, fourteen passes / 58 current
public safety reds, all 288 I/O injections and four publication unwinds reached.
This checkpoint adds **reference-only independent evidence collection**, not
that production repair or a claim that Linux observations already exist.

### New reference-only test and retained evidence contract

`fs::guarded_chmod::needs_host::create_policy::linux_records_create_policy`
uses the existing VM-only mke2fs/kernel/oracle helpers. The new guest Python
script is embedded in the test; no oracle, mount or local KVM is used here.
The input images use eight 1/4 KiB × 128-byte legacy/256-byte extent+metadata-
checksum × direct/journal profiles, with eager inode/journal initialization.
Only Linux mutates these images. Every Rust mount is explicitly read-only.

Each profile saves **input, names, first, second, boundary and closed** phases:
**48 planned 32 MiB images**, with complete JSON stdout/stderr and debugfs
reports. The final phase is a fresh kernel mount after all creation handles
and earlier mounts are closed, verifying the three empty files and exact raw-
byte namespaces with O_NOATIME observation. Expected calls are eight mke2fs,
48 kernel mounts, 48 judged non-repairing e2fsck checks and 408 debugfs inode
reports. These counts are planned test coverage, **not executed Linux results**.

- Linux `open` uses explicit `O_WRONLY|O_CREAT|O_EXCL|O_NOATIME` with controlled
  root credentials/zero umask, recorded in JSON. Non-setgid parents have no
  default ACL. Three creations per profile use an intermediate parent symlink,
  a counted raw-byte name and a 255-byte boundary name: **24 planned creations**.
- Each creation advances the clock across legacy timestamp resolution. The
  test checks empty regular type, one link, zero size/sectors/data allocation,
  root owner/group and ordinary permission bits; **full requested and actual
  modes are recorded**, without guessing special-bit or Rust-API equivalence.
  Child timestamps, parent advancing mtime/ctime and preserved atime, directory
  links/storage and exact per-group free-inode changes are checked. Initial
  non-growing/non-indexed/non-inline parent shape is asserted rather than
  assuming every regular create avoids parent allocation.
- Each profile records all **27 actual errno answers** for empty/256-byte,
  existing file/directory/symlink/dangling/dot/trailing, 255-byte existing name
  and mixed regular/missing/symlink/long-prefix cases: **216 planned observations**.
  Exact uncertain errors are recorded for audit; only the unambiguous 256-byte
  length error is asserted in advance. Parent/name/peer metadata and space
  must remain unchanged for these failed creations; symlink traversal atime is
  outside the protected regular/directory raw-byte set.
- Read-only Rust checks validate profile/clean journal state, existing peer
  identities/raw inodes/attributes, empty-file flags/mapping/CRC/bitmap identity,
  exact raw-byte lookup and all per-group/SB counters. Previously created raw
  inodes remain stable across later closed phases. Parent links/used-directory
  counts and free blocks stay unchanged; only one inode is charged per creation.
  Final Linux listing records exact names and empty data after a cold remount.

This is ordinary exclusive-file reference behavior under the stated inputs,
not driver emission, arbitrary ownership/corruption or casefold mutation
qualification. Exclusive open is intentionally distinct from truncating an
existing file. Native Into/name/generation/FFI/rooted-path and explicit mode
contracts remain separate from syscall observations; do not silently apply
mkdir answers or alter shared `mknod`/symlink helpers.

### CI, local checks and remaining gate

The new separate `create-policy-reference` job in `casefold-stage0.yml` uses the
existing pinned Rust/core/harness setup. Its exact test is wrapped with the
existing focused 3,200-line/200,000-byte pattern and floor **one**. It always
retains the complete `create-policy-reference` artifact for thirty days and
fails if no evidence is saved. Existing focused jobs, full-CI selection, global
caps/floors/filters and output remain unchanged. This adds one host-only test;
normal unit selection continues to exclude VM tests.

Local Python syntax checking does **not execute** the guest script. Rust test
compilation, **87 CI profile + 7 test-contract tests**, full `chore lint` and
normal native units pass. Unit occurrence multisets retain every **1,386**
baseline success and all **418 controls once**, zero failed/ignored. The dormant
safety patch, production driver/C ABI, pins, lock and siblings are unchanged.
Shared-agent/provenance/whitespace checks pass. No new semantic red or independent
Linux pass is claimed before actual CI runs.

Scratch compile/contract/lint/unit logs, baseline retention, CI status and a
prepared compressed-image auditor live under `tmp/casefold-create-reference/`.
That auditor is not an executed image audit before actual source/digest-verified
CI artifacts exist. A fresh workspace can retrieve the authoritative artifact
and complete logs from the exact pushed source's focused run.

**Next bounded task:** inspect this exact source's focused and full CI. Download
and verify exact workflow SHA/advertised artifact digest, then audit all saved
images/JSON/debugfs/kernel records: error consistency, special modes, byte names,
times, peer/EA preservation, per-group bitmap/free/used-dir/CRC accounting and
clean journal state. Repair actual fixture/test/driver failures first, without
weakening expectations. Only measured reference answers justify native semantic
reds and the reviewed guarded repair. Stages 4–5 remain not yet quantified;
casefold writes and broader ownership/mapping/growth/lifecycle/recovery/ignored-
fuzz/Windows/SteamOS/card gates remain blocked/open as previously recorded.

## Executed native public-create safety preparation — 2026-10-23

Started clean at fetched `0d38ff143231e0055e59c40ec5b58cfeb959f2a5` on
`casefold-rw`, using Sol 6.1 and the existing pinned Rust/core source view.
The normal unit baseline actually executes and retains **1,386 successful
occurrences / zero failed/ignored / all 418 controls once** before test edits.
The preceding design audit is unchanged. Exact [full run 40025606526](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40025606526)
is still running its native x86_64 job at inspection; all other jobs have passed.
No full-CI/ci-ok completion is inferred from those partial results.

### Executed audit and reproducibility

The new dormant [create safety patch](CASEFOLD-CREATE-RED.patch) adds only the
native `create_calls` module and its registration. It uses the existing MIT
Device/Clock/C fixture machinery, actual public APIs and checked native images;
no production helper or fake future commit boundary is introduced. Both full
actual runs through `scripts/tier.sh` execute **72 identical names/outcomes**:
**14 passed / 58 failed / zero ignored**, exit 101, **908 lines / 58,680 bytes**
each. The patch is saved, reversed, checked against original source, reapplied,
verified to recreate identical complete source and rerun. Final per-name verdict
maps match exactly; elapsed time/thread identifiers need not make log bytes equal.
The patch SHA-256 is
`c5078050e8cfd3aaca199f9743265763422030e34d7dc73021a3791ac8b9a77b`
(69,937 bytes). Apply only to its recorded compatible source, not over promoted
future tests, and keep it inactive in normal CI until repaired.

All **58 failures reach actual current public safety assertions**: held entry,
marker-before-refusal, second verdict, legal callback/worker reentry, planning/
conversion unwinds, optional C readback exclusion and the required sticky mount
retirement contract. No unmeasured Linux timestamp/error/mode assertion, compile
failure or future constructed-SB diagnostic is counted as a public driver red.
Missing/foreign/malformed-SB helper controls wait for the real create boundary
at promotion, as the design requires.

### Actual fault coverage and compatibility controls

- Each Rust/C sweep measures that create path's actual writes/flushes across
  ext4 direct/ext3 journal × 1/4 KiB × clean/already-dirty profiles. Per interface,
  **88 returned write errors + 28 returned flush errors + 28 flush unwinds**
  execute, **288 injections total**. Every requested callback index is reached
  before the aggregate assertion. All violate the required shared-latch/ten-
  family/flush/close retirement contract; existing partial direct/journal
  protections are not represented as absent or as complete mount retirement.
- **Four actual public post-commit allocation-publication unwinds** execute on
  direct/journal 1/4 KiB layouts. The namespace is already committed and the
  source asserts that fact before collecting latch/ten-family/close outcomes.
  All four are collected before the final failing assertion.
- Conversion callbacks collect all nine qualified-family worker outcomes;
  incoming guarded-family callbacks also reach create. Writability/read/time/
  generation/marker/commit/flush, same-thread and self-worker callbacks execute
  without panicking across extern C or fabricating mutable C aliases.
- Twenty-four clean counted refusals and all twenty-four already-dirty counted
  refusals reach expected errors. The twelve Rust/C pathname observations all
  execute before their aggregate untouched-byte assertion. Clean logical
  refusals expose a marker write; already-dirty refusal controls pass unchanged
  byte/error/context checks. Exhausted-block fixtures assert actual parent
  saturation for a 255-byte name: empty files need blocks only for parent growth.
- **54 primary successful profiles** cover all five forms over ext2/ext3/ext4 ×
  1/4 KiB, Rust/C full-parent growth, inline edit/conversion and DIR_NLINK unknown
  parent counts. Each verifies a regular empty inode, explicit mode, zero owner/
  group, one file link, generation/times, flavor mapping/flags/CRC, inode bitmap,
  unchanged parent/used-directory link counts, exact SB/group debits, peers/EA,
  actual parent growth charges, distinct next inode and healthy close/reopen.
- Additional positives assert both shared inode-table slices, maximum parent
  link count without a mkdir-style refusal, exact separator/root-walk/parent-
  symlink/trailing/case behavior, null/ceiling FFI priority and healthy reuse
  after refusal. A constructed **2 KiB/two-group uninit** profile explicitly
  reserves group-0 blocks and inodes, then reaches two distinct file inodes
  and one group-1 parent growth block. Bitmap padding, itable-unused, descriptor/
  bitmap/inode CRCs, counters/zero used-directory delta, allocation publication
  and reopen agree. This is a native allocation profile, not arbitrary ownership
  or independent Linux qualification.
- All four C optional readback callbacks execute before their exclusion
  aggregate fails. Four separate post-commit read failures return EIO while
  exactly one file inode/no child data block remains; output inode/generation
  sentinels stay intact. NULL avoids that read. Counted **Into-before-name**,
  volume-before-conversion, name-before-stale-generation and FFI priorities pass.

The initial discovery run's two positive checks used the existing directory-
listing helper on legacy layouts, which that helper refuses. The test harness
is corrected to parse the actual verified mapped directory blocks (retaining
inline listing only for inline fixtures). Those discovery failures are **not**
counted as create defects; all final positive storage checks pass. No production
listing behavior is changed or newly qualified by this fixture correction.

### Restored source, checks and bounded next task

Full `chore lint` passes **with the complete native audit active**, including
provenance over its registered file, Unicode checks, formatting and all-target
clippy with warnings denied. After the second red run the patch is reversed
again. Production/test/workflow/task/lock bytes are restored; the native unit
task again retains **every 1,386 baseline occurrence / 418 controls once**, zero
failed/ignored. The new artifact is dormant; ordinary CI does not gain the 58
intentionally failing active tests. No global limit/floor/filter/output, pin,
lock, sibling, production repair or casefold write-gate change is made.
Shared-agent/provenance/whitespace and dormant-patch applicability checks pass.

Local evidence under `tmp/casefold-create-red/` includes baseline/restored unit
logs, initial discovery/final complete logs, wrapped command/lint logs, source/
patch hashes, `check_runs.py`, both outcome JSONs and retention comparison.
To reproduce through the pinned wrapper, apply the patch on compatible source,
run `scripts/tier.sh` around
`cargo test --locked --lib fs::guarded_chmod::tests::create_calls:: -- --nocapture --test-threads=1`
(expect 101), lint, reverse the patch, then run the normal unit task. Do not push
an active intentionally failing module to ordinary CI. Existing mkdir patches
are historical/promoted and must not be reapplied.

**Next bounded task using Sol 6.1:** add and actually execute independent
**reference-only Linux exclusive regular-file creation** in the existing CI VM:
advancing parent/child times, ordinary existing/dot/trailing/raw/255/256-byte and
mixed-prefix errors, controlled mode/ownership/umask expectations, exact inode/
parent-storage accounting and peer preservation. Assert fresh non-growing input
shape where zero child blocks are expected; save input/output images and complete
judged e2fsck/debugfs/cold kernel records, then audit actual bytes before native
semantic reds or migration. Keep explicit Rust/C modes/Into/FFI/rooted-path
contracts distinct from measured syscall behavior. Casefold writes stay blocked;
Stages 4–5 remain not yet quantified. Ownership/mapping/growth/seals/lifecycle/
recovery/ignored-fuzz/Windows/SteamOS/card gates remain separate. No host oracle,
mount, local KVM or forbidden implementation source is used.

## Public-create staging, marker and dispatch design audit — 2026-10-23

Started from clean fetched `1248f71f1cb6032d1ebb1d7c0c9cd25e9d8f70b3` on
`casefold-rw`, using Sol 6.1. Its exact
[full CI 40013659564](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40013659564)
now passes all eleven jobs and ci-ok. This is a fresh completed status check;
the downloaded repair-source `39997261948` archives remain the occurrence/log
baseline (2,227 complete-suite passes, 1,386 unit occurrences, 418 controls in
five contexts, 527 architecture image passes and trailing twenty shell/159 CLI
checks). Source/test/workflow bytes are unchanged, so prior independent mkdir/
retained-family audits remain applicable. No unchanged image archive audit,
fresh full native suite or Linux oracle execution is claimed here.

The detailed audit is in the existing
[casefold review](CASEFOLD-INDEX-REVIEW.md#public-create-staging-marker-and-dispatch-design-audit--2026-10-23).
It traces Rust string/bytes/counted and C path/counted forms, pointer/error/return
conventions, both current marker-writing verdicts, caller conversion and name/
live-parent/generation/duplicate priorities, common inode allocation and bitmap/
uninit/itable-unused/counter/CRC helpers, empty regular inode construction,
parent inline/indexed/in-place/growth handling and complete effects/publication.
Shared `plan_new_inode_in_dir` callers (`mknod`/symlink) are explicitly outside
the create migration. Old “Not journaled”/early-commit comments do not override
the current bodies: both create branches commit the complete buffer once.

### Design and compatibility decisions

- Acquire one existing nonblocking mount context before device/runtime/caller
  callbacks; use private nested dispatch with both pure volume verdicts and
  preserve the nine already qualified families. External exclusion remains
  required for other writers, handles, maintenance and lifecycle.
- Counted create currently runs **Into before counted-name validation**, unlike
  mkdir. Preserve volume-before-conversion, conversion-before-name, name-before-
  liveness and FFI-null/slice-before-entry ordering. Name validation remains
  empty/slash/NUL/length; no new dot special case or stale-generation masking.
- Preserve exact current path syntax and basename-length-before-parent ordering.
  The splitter trims trailing slashes and requires a separator; the parent walk
  starts at root even for `p/new`, while separator-free `new` is refused. Do
  not silently impose an absolute-only rule, cwd semantics or mkdir's measured
  policy corrections. Parent mtime/ctime and mixed errors are future independent
  measurement candidates, not executed semantic reds in this audit.
- Keep the shared partial `NewInodePlan` setup and all allocators/helpers intact;
  extract a separate private completed create result (inode plus buffer).
  Preserve one non-directory inode debit, no child data allocation, zero used-
  directory delta, one file link, explicit low-twelve mode bits, zero initial
  owner/group, flavor-specific empty mapping/flags, runtime generation/times
  and CRC. Parent conversion/growth/map allocations must be fully staged and
  charged. Future parent metadata changes must read the current buffered inode
  after insertion/growth to preserve inline/mapping/shared-table edits.
- Converge both terminal branches into one create-only full-SB boundary: require
  identity/health and complete valid primary-SB staging before effects, preserve
  all debits/state/outside bytes, clear only clean state and refresh CRC. Keep
  future constructed-SB diagnostics separate from current public regressions;
  current create marks before building its buffer.
- Latch the marker and **entire** raw commit/checkpoint/direct-flush/allocation-
  publication call. Scope includes clone/lock/publication unwinds; do not add a
  second partial commit, refactor shared effects or claim power-loss atomicity
  for ordered direct writes. Keep one context through optional C inode readback,
  then fill attrs from owned bytes. NULL avoids reading; a later read error
  leaves exactly one creation and must not trigger retry or pretend rollback.

### Next native controls and independent evidence

Concrete next groups cover held entry across all five forms; legal same-thread/
worker/device/runtime reentry; clean/already-dirty untouched refusals; exact
Into/name/generation/volume/FFI/path priorities; successful empty-file fields,
byte names and exact per-group accounting; actual inline/full-parent/same-table/
multi-group/uninit fixtures; every measured write/flush uncertainty and legal
runtime/Into/publication unwind; and C readback/null/error behavior. Use ext2/
ext3/ext4 × 1/4 KiB base profiles and assert actual shape. Do not inherit mkdir's
352 fault count or predict which current protections already pass. Real private
boundary diagnostics wait for the real helper, not a fake implementation.

**Next bounded task:** actually execute and save a dormant native create safety/
positive patch, reverse/reapply/rerun with identical source/name outcomes, lint
with the audit active, restore source and retain all **1,386 units / 418
controls**, zero failed/ignored. Every intended fault must execute before aggregate
assertions. Compile or malformed-fixture failures are not driver reds. Keep the
patch inactive in ordinary CI and both mkdir patches historical/promoted.

Then independently measure **exclusive regular-file creation** with Linux-only
reference writes in the existing VM before semantic reds/repair: advancing times,
name/error ordering, controlled mode/owner/umask expectations, unchanged peers
and exact accounting. Save images, judged e2fsck/debugfs/cold kernel records and
audit actual bytes. A later bounded repair still requires independently audited
Rust/C emitted images, complete baseline retention and standalone review. No
self-built positive fixture or earlier mkdir result substitutes for create policy
or emission qualification.

Only this handoff and existing review change. Shared-agent integrity, provenance
and whitespace checks pass under the pinned source view/output wrapper. No
production/test/workflow/task/cap/floor/filter/pin/lock/sibling/write-gate change.
Current policy is enforced and existing Git/public API reads work; no environment
edit/publication is needed. Scratch status/source manifests are under
`tmp/casefold-create-review/`; the authoritative current-CI link is above.
Stages 4–5 remain not yet quantified. Casefold writes, arbitrary ownership/alias/
mapping/index growth/seals, retained handles, lifecycle/recovery/ignored fuzz and
actual Windows/SteamOS/card qualification remain open/blocked as applicable.

## Scoped mkdir full-CI review closure — 2026-10-23

Started from clean fetched budget-repair
`7f9617dc3b4c4d2e3663f382d7a644bd681fd62e` on `casefold-rw`, using Sol 6.1.
Exact [full CI 39997261948](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39997261948)
now completes **all eleven jobs successfully, including ci-ok**. The two
whole-byte repairs are verified by actual CI, not just saved-log replay.
The managed policy is currently enforced and Git/public API/redirected archive
and job-log reads work; no environment edit/publication or new secret is needed.

### Exact-source retention and actual trailing checks

Five tier-log ZIPs match workflow source, advertised SHA-256 and every member
CRC. A fresh read-only checker compares successful names as **multisets** with
the prior baseline, preserving repeated names and every previous occurrence:

- Native and guest complete suites each retain **2,227 passed, zero failed,
  one unchanged ignored** `deep_walk_byte_flip_fuzz_never_panics`. This includes
  all 2,122 preceding successes, 104 new native controls and the mkdir oracle;
  the existing ignored case remains a broader qualification gate.
- Standalone, aarch64 and native unit contexts each retain **1,386 passes**, zero
  failed/ignored, including all preceding 1,282 occurrences. All **418** guard/
  operation controls appear once in each of these three contexts and both whole
  suites: five contexts total.
- Architecture image output retains **527 passes**, zero failed and the same
  existing ignored fuzz test. Standalone installed CLI retains six files /
  **159 passes**.
- Native trailing checks now actually execute: **twenty shell checks** retain
  exactly the preceding names/verdicts, then all six installed-CLI files /
  **159 checks** pass. Separate CLI results no longer stand in for unrun steps.

Downloaded native job `120565123869` and guest job `120565124120` confirm:

| Actual tier | Complete log | Unchanged floor / observed |
|---|---|---|
| Native whole | 3,633 lines / 212,215 bytes | 1,060 / 2,227 |
| Guest whole | 3,800 lines / 221,518 bytes | 1,060 / 2,227 |
| Native shell | 40 lines / 2,213 bytes | All twenty named checks pass |
| Native installed CLI | 29 lines / 1,097 bytes | 150 / 159 |

Whole output fits **3,700 / 220,000 native** and **3,900 / 230,000 guest**.
Actual native unit/Unicode/oracle/kernel/lwext4 floors also pass at 1,386/8/225/
33/9 observed tests, respectively. The complete whole-log SHA-256 values are
`267aae1a0b2453951475b8aa2bc4f32bfdc02cd696f6228bcc46a1e51097f492`
(native) and
`8c1bff7a9fa3be245ff32b1248a06db831c6763b52092b4ece7a434ff881903c`
(guest). No line limit, floor, filter, output or test selection is changed.

Verified advertised archive identities:

| Archive / artifact ID | SHA-256 |
|---|---|
| `tier-logs-x86_64` / `12524351976` | `7434981ea6e0de929a9613a89e82ff23b1e0e50fba959baf499e614db7877c8a` |
| `tier-logs-suite-in-vm` / `12522593927` | `2e5be5ad340aa8153a5bb94dfb09c24264ec15c2457e35042d8ebf999d65dedd` |
| `tier-logs-aarch64` / `12520952712` | `9a631c27a7b73dd3fa440b59ed0f2b5eb92343f31850000cc3669a6e5226095a` |
| `tier-logs-unit` / `12520945974` | `f7988493fbd45a21e2e1557470d7af778a5a984549cde9a06c36da0ca9d8ac2b` |
| `tier-logs-cli` / `12520246743` | `023cb990237e91569a386721a15f426d48f47e603ec5f37d0a2b7adc94dbdd48` |

Local scratch evidence under `tmp/casefold-mkdir-closure/` includes exact run/
job/artifact metadata, `39997261948/` archives/extracted logs/job logs,
`check_green.py`, `green-retention.json` and wrapped `mkdir-green-retention.log`.
The linked exact CI run is authoritative; a future workspace need not inherit
these local files to retrieve the evidence.

### Closure, restrictions and next bounded task

Compared with independently tested `97356c9`, implementation, tests, workflows,
scripts and dependency/tool pins are unchanged; only measured budget configuration
and review/handoff text differ. The completed scoped source review, independently
audited sixteen-layout/112-image/48-creation mkdir emission and every retained
family audit recorded above remain applicable. Unchanged multi-gigabyte matrices
are not rerun or represented as fresh image qualification for this docs checkpoint.

The scoped mkdir review is **closed within caller exclusion, healthy layouts
and closed reference handles**. No further scoped blocker is found. This is not
arbitrary alias/ownership/corruption admission, retained target-handle safety,
complete mapping/plan seals, index growth, all-writer serialization, lifecycle,
power-loss/recovery or Windows/SteamOS/card qualification. The ignored fuzz gate
remains open; Stages 4–5 remain not yet quantified and **casefold writes stay
blocked**. Both mkdir patches remain historical promoted artifacts.

Only this existing handoff and `CASEFOLD-INDEX-REVIEW.md` change in this closure
checkpoint. Shared-agent integrity, provenance and whitespace checks pass under
the pinned source view. No production/test/task/workflow/cap/floor/filter/pin/
lock/sibling/write-gate change is made. No host oracle, mount, local KVM or
forbidden implementation source is used.

**Next bounded task using Sol 6.1:** audit public regular-file `create` string/
byte/counted Rust and C dispatch, volume/name/Into/generation/duplicate priorities,
new-inode allocation and flavor-specific construction, parent inline/in-place/
index/growth and same-table staging, counters/checksums, timestamps, and complete
marker/commit/allocation-publication effects. Inspect the shared
`plan_new_inode_in_dir` callers (`mknod`/symlink) so a future create-local repair
preserves their contracts. Cover optional counted C attributes after commit.
Record one shared-context/owned-plan design and concrete native safety/positive
controls **before** red execution or migration. This checkpoint selects that
audit; it does not claim the audit, new red tests or independent create policy
measurements are complete. Keep all qualified families and write gates intact;
semantic changes need independent Linux policy evidence and executed reds first.

## Mkdir full-suite budget repair and scoped evidence review — 2026-10-23

### Completed exact-source CI and retention

Implementation `97356c928fa9a6d231355f254dc08aa69d540138` now completes all eight
focused jobs successfully in [run 39912448406](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39912448406),
including main fixtures. Full implementation [run 39912448414](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39912448414)
and documentation-source `31e64be639c5b744f99c6968c8c15fa92810574b`
[run 39913996529](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39913996529)
finish with only the native/guest whole-wrapper byte-budget failures and their
aggregate ci-ok failure. Unit, CLI, fixtures, MSRV, wasm, provenance, semver and
aarch64 jobs pass in both full runs. No failing driver test is demonstrated.

Ten downloaded unit/aarch64/native/guest/CLI log ZIPs match exact workflow source,
advertised SHA-256 and all member CRCs. Successful names are compared as
**multisets**, retaining repeated names rather than silently deduplicating them:

- Both whole suites in both runs retain all **2,122** preceding successful
  occurrences plus **104** native controls and **one** new mkdir emission oracle:
  **2,227 passed, zero failed, one unchanged ignored**
  `deep_walk_byte_flip_fuzz_never_panics`. That ignored case remains a broader
  qualification gate; it is not a newly passing or removed test.
- All **418** guard/operation controls occur exactly once in each standalone,
  architecture and native unit context and each native/guest whole suite: five
  contexts per run. Each unit context retains all **1,386** occurrences, zero
  failed/ignored, including every preceding 1,282 occurrence.
- Architecture images retain all **527** passes and the same existing ignored
  fuzz case. Standalone installed-CLI output retains **159** passes per run.
- Native trailing twenty shell checks and 159 installed-CLI checks are **unrun**
  because the preceding whole wrapper returns 65. Standalone CLI success does
  not substitute for those trailing checks. Actual whole-suite floor commands
  after the failed wrapper are also unrun; replay floors below are separate.

### Measured wrapper red and identical-log green

| Exact run / complete log | Lines | Bytes | Old cap / result | New cap / replay result |
|---|---:|---:|---|---|
| `39912448414` native | 3,633 | 212,216 | 205,000 / 65 | 220,000 / 0 |
| `39913996529` native | 3,634 | 212,331 | 205,000 / 65 | 220,000 / 0 |
| `39912448414` guest | 3,801 | 221,651 | 213,000 / 65 | 230,000 / 0 |
| `39913996529` guest | 3,803 | 221,630 | 213,000 / 65 | 230,000 / 0 |

Actual implementation job logs `120283614200` / `120283614100` explicitly report
passed test commands followed by byte-budget exit 65. Both documentation jobs
repeat that wrapper-only failure. Each of the four **complete unchanged logs**
is replayed through `scripts/tier.sh` under old and new byte limits; all four
old replays return 65 and all four new replays return 0. Manual replay floor
checks each observe 2,227 executed against the unchanged floor 1,060. SHA-256
is identical before/after each replay. No output filtering, test selection,
line-limit (3,700 native / 3,900 guest), floor or skip policy changes.

Complete native source/docs log SHA-256 values respectively:
`9e917a4dc4831ddf0bcb369e070be4781d762d26f171e5053584c536953608e2`,
`fe98c765f2386b34305640474b3bfe6498b1b2318fb201ef4fa88e50926ab3b2`;
complete guest source/docs values respectively:
`f67364e6553d9362e2793e4094ffa49233c4df4ada32ac72520aed5a7e1aebcc`,
`3ed610f49fdad379b9ecb90791532c9f6c7ed7b701bfd0ae2598d3db30106afb`.

Only the two exceeded whole-tier byte caps and their measured table change in
`chores.yml`; no driver, test, workflow, lock, dependency pin or sibling changes.
Native unit/Unicode/oracle/kernel/lwext4 tiers already pass in the failed job.
Fresh **87 CI-profile + 7 test-contract tests pass** through the tier wrapper;
full lint passes. A fresh exact-source full CI run must still verify actual
wrapper/floor success, trailing native checks and ci-ok before scoped closure.

### Retained families and source/evidence review

Eight focused archives freshly match implementation source and advertised digest:
`casefold-fixtures` `12484631280`, `rmdir-policy-reference` `12483850748`,
`symlink-backport-evidence` `12483627259`, `mkdir-policy-reference` `12483597217`,
`rmdir-emission` `12483478286`, `unlink-emission` `12483472060`,
`setxattr-policy-reference` `12483358898`, `unlink-policy-reference` `12483207745`.
All fifteen main focused log files show one pass / zero failed/ignored each.
Fresh read-only compressed-image audits pass retained chmod/chown/timestamp/
flags, setter/first-attribute, removal, setter policy, unlink emission/reference,
rmdir emission/reference, mkdir reference and five-test symlink evidence.
This includes the 68 retained metadata profiles, sixteen setter/eight first-
attribute profiles, sixteen removal profiles, 16-layout/112-image rmdir emission,
eight-layout/32-image rmdir reference and eight-layout/40-image mkdir reference.
The prior independently audited **16-layout/112-image mkdir emission** remains
applicable: no implementation/test bytes changed. Its audit is not reexecuted
merely for this configuration/documentation checkpoint.

Scoped review traces all five Rust/C mkdir entries, shared mount context/pure
verdicts, private nested dispatch, generic counted-name validation precedence,
owned inode/buffer staging, same-table/inline/growth preservation, parent
link/mtime/ctime updates with atime retained, mkdir-only full-SB marker boundary,
and complete commit/checkpoint/direct-flush/allocation-state publication effects.
The optional C attribute read remains inside the context: a post-commit read
error leaves one creation, and NULL avoids that read. Review retains explicit
mode/name/generation/FFI contracts and mkdir-local pathname priority; other
creation helpers and guarded families are unchanged. Native red/green, all 352
actual I/O uncertainties/four public publication unwinds, independent saved
bytes and retained-family evidence agree within the stated scope. No further
scoped source/evidence blocker is found; **full-CI review closure is pending**.

These are healthy ordinary-volume results under caller exclusion and closed
reference handles, not arbitrary alias/ownership admission, mapping/index growth,
lifecycle/seal, power-loss/recovery or actual Windows/SteamOS/card qualification.
Both mkdir patches remain historical promoted artifacts. Casefold writes stay
blocked; Stages 4–5 remain not yet quantified.

Local audit scripts/results are under `tmp/casefold-mkdir-closure/`:
`check_retention.py`, `retention.json`, `replay_budgets.py`, `budget-replay.json`,
`audit_retained.py`, `retained-audit-command.log`, and `focused/*audit*.json`.
They are scratch evidence, not a requirement that a fresh workspace inherit
these paths; authoritative archives/logs are the exact linked CI runs above.
Next inspect the budget-repair commit's exact full CI, retain all occurrences/
controls and verify trailing shell/CLI/floors/ci-ok, then close the scoped mkdir
review before selecting another mutation family.

## Verified mkdir Linux emission and saved-byte audit — 2026-10-23

Repair source `97356c928fa9a6d231355f254dc08aa69d540138` passes exact focused
`mkdir-emission` job `120278298194` in
[run 39912448406](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39912448406).
The actual completed job log records **one executed test, floor one**, zero
failed/ignored, **893 lines / 111,735 bytes**, within its isolated 3,200 /
200,000 limits. Invocation markers confirm **sixteen mke2fs, 112 judged e2fsck
checks, 544 debugfs calls and 64 kernel mounts**. These are actual CI outcomes,
not the planned matrix or a native self-consistency claim. No host oracle or
mount is used.

### Source/digest-verified archive and complete raw-byte audit

Artifact `mkdir-emission` (`12483279278`) matches its exact workflow source and
advertised SHA-256:
`4f1b9fcd9b3773ffa48f7cd54b1c1ebac8cb2437d07497ece0a7d3087efc4914`.
It is **4,627,024 compressed bytes / 3,758,624,323 uncompressed bytes / 1,329
members**, retaining all **112** 32 MiB images. The audit reads directly from
the compressed archive; every member is read and ZIP-CRC checked without
extracting several gigabytes into the bounded workspace.

All **sixteen Rust/C × 1/4 KiB × 128-byte legacy/256-byte checksummed ×
direct/journal layouts** pass input plus first/second/boundary dirty and closed
phase inspection. All **48 creations** match saved raw inode/directory bytes,
complete debugfs reports and **432** cold Linux metadata records; initial
Linux parent/name/peer reports match their input bytes too. The single complete
successful test also executes every **496** source-required unchanged-byte
refusal observation and the C errno-seeding calls. Shape assertions exclude
inline/indexed/growing parents and check actual journal/extent/inode/checksum
features rather than trusting profile labels.

The audit independently checks:

- Exact stored namespace and inode identities, dots/dotdot, new raw non-UTF-8
  and 255-byte names, and all existing directory/file/symlink/dangling names.
  Each new child is an empty one-block/two-link directory, generation 17, with
  the expected direct/extent mapping, current timestamps, no attribute block
  and no index/inline flag. Earlier child bytes remain identical in later
  saved phases and dirty/closed inode views agree.
- Parent mtime/ctime advances for both policy-parent creations, then stays at
  the second stamp when the names parent receives the boundary creation.
  The names parent's timestamp changes only for that creation. Parent atime,
  storage, identity, mode/owners/flags/attribute pointers remain intact, and
  link increments are exactly two for the policy parent and one for names.
- All protected peer/existing inode bytes, peer/regular payloads, parent/peer
  external attribute contents and complete EA block bytes remain intact.
  Surviving symlink inode bytes/payloads and root namespace remain unchanged.
  Raw data mappings, owners, modes, links, sectors, generation, access/change/
  modification times and recorded birth-time fields agree with debugfs.
- Actual new inode/block numbers determine per-group inode/block bitmap,
  free-count and used-directory changes; no physical allocation match with
  Linux is assumed. Every child inode and data block is newly allocated and
  distinct, and bitmap counts/CRCs plus descriptor/SB totals agree. Per layout,
  the final ledger is **-3 free blocks, -3 free inodes, +3 used directories**,
  with no parent storage growth.
- Dirty-phase VALID_FS is down and closed-phase VALID_FS is set, preserving
  unrelated state; RECOVER is clear after checkpoint. Every saved journal
  header has zero log start/error and consistent layout, and every checksummed
  journal superblock CRC is valid. This verifies saved checkpoint state, not
  interrupted-write recovery or crash atomicity.

Repeated checksum views include **56 primary superblocks, 140 descriptors,
2,084 allocated inode views, 488 directory blocks, 240 EA views, 140 block
bitmaps, 138 inode bitmaps and 28 journal superblocks**. All **56 journal
headers**, **544 debugfs reports** and **432 kernel records** pass. These are
repeated checks over phase views, not that many distinct structures.

All sixteen profiles retain the explicit driver API modes **02750, 01770 and
0750** in both saved bytes and Linux stat output. Linux's reference syscall
02750-to-0750 observation remains a separate API/credential fact; no mode
contract is silently changed. The reference-only policy measurement remains
applicable and its fresh job also passes. Healthy caller-excluded ordinary
images are qualified here; casefold writes and broader shapes remain gated.

### Exact CI retention and remaining work

The downloaded `tier-logs-unit` archive matches source, advertised digest and
ZIP CRC. Its successful occurrence multiset exactly equals the local repair
baseline: **1,386 passed / zero failed/ignored**, all **418** guard/operation
controls exactly once, **1,642 lines / 122,202 bytes**, within the unchanged
1,740 / 131,000 unit limit. Standalone CLI also passes; its log archive is
source/digest/CRC verified. No driver/test/workflow/task/cap/filter/pin/lock or
sibling change is made in this audit checkpoint.

At inspection, all seven separate focused reference/emission jobs pass;
the main focused fixtures job remains running. In
[full CI 39912448414](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39912448414),
unit, CLI, MSRV, wasm, provenance and semver pass. The fixture job is still in
runner/harness preparation; aarch64, native/guest whole-suite retention and
`ci-ok` are not established yet. No failure is reported in the pending job at
this inspection. Do not infer complete-suite closure from the passing focused
matrix, or change a whole-suite cap before its actual complete log fails.

Evidence is under `tmp/casefold-mkdir-emission/`: exact job/artifact metadata,
verified compressed ZIP, complete job log proving the floor, the read-only
`audit_emission.py` and `image-audit.json`, all 112 image hashes/group ledgers,
unit archive/retention and source manifest. Helpers use prior own MIT raw-format
code and the documented ext4/JBD2 formats; no kernel/e2fsprogs implementation
source was read. No new power-loss, ownership, alias, lifecycle or Windows/card
qualification is implied.

**Next bounded task:** inspect remaining exact full/focused CI when it finishes,
repair actual failures first, and retain every preceding whole-suite occurrence
plus the promoted native controls/new oracle, all 418 controls in the relevant
CI contexts, aarch64 image checks and trailing shell/CLI checks. Verify retained
family evidence and perform the standalone mkdir source/saved-evidence review
before scoped closure. Reuse these verified emission bytes while source stays
unchanged. Stages 4–5 remain **not yet quantified**; broader ownership/mapping,
index growth, lifecycle/seals, ignored fuzz, recovery and Windows/SteamOS/card
gates remain open, and casefold writes stay blocked.

## Guarded public mkdir implementation — 2026-10-23

Started from clean fetched `ea2b41b3629f80fa2a7862ca633bea233cf106dd` on
`casefold-rw`, using Sol 6.1 and the prescribed immutable core source view.
Its exact [full CI 39841568572](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39841568572)
passes every job and `ci-ok`. A fresh execution of both saved patches reproduces
all 100 names/outcomes: **25 passes / 75 failures**, diagnostic exit 101,
**1,037 lines / 80,845 bytes**, with all 352 I/O injections and four actual
publication unwinds retained. This repeats the actual defects before repair.

### Shared context, completed staging and complete effects

All five public Rust/C entries now enter the existing shared nonblocking mount
operation before filesystem callbacks, name/path reads or arbitrary generic
parent conversion. Private `mkdir_bytes_in`, `mkdir_path_in` and `mkdir_from_in`
reuse that context and preserve both pure pathname volume verdicts. Logical,
name, generation, duplicate, link-limit, checksum and allocation refusals occur
before a dirty-marker write. Counted names retain empty/slash/NUL/length before
`Into<InodeRef>` and live-parent validation. Pointer/slice checks still precede
filesystem dispatch at the C boundary; platform errno mapping is unchanged.

`plan_mkdir_at` returns one owned `MkdirPlan { ino, buf }`. It preserves the
existing inode/block allocation, per-group and SB debits, used-directory count,
bitmap/checksum/uninit/itable-unused handling, child mode/generation/times,
dots, ordinary parent insertion/growth and staged same-table inode splicing.
Both parent-storage branches now converge before link/timestamp staging, with
parent mtime/ctime stamped through the current buffer and CRC recomputed;
parent atime and other staged inode bytes survive. Inline conversion, parent
growth, DIR_NLINK and back-to-back distinct allocations retain native controls.

The mkdir-only `commit_mkdir_in` checks context identity/health, requires a full
correctly sized primary-SB block with valid magic, clears only VALID_FS and
refreshes its checksum while preserving every staged debit, other state and
outside-SB byte. The dirty marker and the complete existing commit path run
inside effects, including journal checkpoint/direct flush and allocation-flag
publication. Errors or unwinds latch retirement before returning or an outer
catcher can reuse a retained context. Nothing broadens generic write admission
or refactors the already-qualified rmdir/unlink commit helpers.

Counted C optional attributes are read under that same context, then copied
from an owned inode snapshot without further filesystem reads. NULL attributes
still avoid that read. A post-commit read error may still return EIO after
exactly one successful creation; no rollback is implied. Actual readback worker
and injected first-extra-read controls retain those two distinct cases.

### Measured semantics and executed native evidence

The Linux-grounded timestamp controls now pass for both first and second
creations across all five Rust/C forms and all six native profiles. Pathname
regular/missing parent errors now precede final-name length. The pathname
adapter checks parent type after its second pure volume verdict, before the
length verdict; the counted adapter keeps its earlier validation order.
`resolve_new_parent` is unchanged for other creation families. Existing dots,
trailing forms, raw-byte and 255-byte identities, generation/FFI priorities,
absolute-path syntax and explicit low-twelve-mode-bit contracts are preserved.
No credential/umask/default-ACL or syscall-mode rewrite is included.

An intermediate repaired run reaches **93 passes / seven failures**: the four
not-yet-rebound constructed SB probes and three remaining regular-parent
precedence probes. The latter demonstrate why pathname parent-type validation
must precede length while counted names keep their explicit earlier validation.
After that local correction and rebinding, all **104 controls pass / zero
failed/ignored**, complete diagnostic log **644 lines / 45,594 bytes**.

All original 100 names are promoted without dropping any control. The four
historically named `constructed_future_*` cases now exercise the actual private
mkdir boundary with constructed trusted buffers; they verify dirty state,
credits, CRC, unrelated state and outside-SB sentinels. They do not claim
arbitrary allocation/ownership validation. Four new controls exercise foreign
contexts and missing/malformed primary SBs before callbacks, marker/commit
errors with retained contexts, actual publication unwind retirement before
context drop, and second-verdict precedence over regular-parent/long-name
policy. Existing successful storage profiles gain parent-time/atime assertions.

Every measured **352** I/O failure/unwind now retires the mount and nine guarded
families/flush/clean-close path, with zero retirement violations. All **four**
actual public post-commit publication unwinds now latch and retire correctly.
All 54 primary positive profiles plus same-table, constructed multi-group/
uninit, C readback, symlink and allocation compatibility controls remain.
The policy subset still reaches 252 creations and 714 refusal/precedence/syntax
observations; all ten timestamp and three mixed-path mismatch lists are empty.

Both [safety](CASEFOLD-MKDIR-RED.patch) and
[policy](CASEFOLD-MKDIR-POLICY-RED.patch) patches are now **historical promoted
artifacts; do not apply them on this source**. Their committed bytes remain
unchanged. New native modules carry the active controls.

### Native retention and actual output-budget repair

The first normal unit execution passes **1,386 tests / zero failed/ignored**,
retaining every earlier 1,282 successful occurrence plus exactly 104 controls.
All **418** guard/operation controls occur once. Its complete log is **1,634
lines / 121,718 bytes**; the wrapper actually fails with status 65 at the old
1,560 / 115,500 ceiling. The unchanged complete bytes are replayed through the
wrapper: old limits return 65, new **1,740 / 131,000** limits return zero with
identical SHA-256. Only the measured unit row/task cap changes; selection,
floor 440, output, other tier caps and filters remain unchanged. Final native
retention and full lint pass, including the existing task/profile contracts,
formatter, all-target/CLI clippy with warnings denied, provenance, shared agent/
family/aggregate and all eight Unicode table checks.

### Independent driver-emission test added; CI remains pending

One new needs-host exact test and `mkdir-emission` focused job cover **sixteen**
Rust/C × 1/4 KiB × 128-byte legacy/256-byte checksummed × direct/journal layouts.
The existing pinned harness builds ordinary Linux inputs. Each layout plans
three driver creations: parent-symlink/trailing pathname with mode 02750,
counted raw-byte mode 01770, and new 255-byte name mode 0750. The matrix retains
**112 input/dirty/closed images**, plans **48 creations**, and checks 496
explicit unchanged-byte refusal observations plus C errno seeding. Shapes and
journal/extent/inode/checksum labels are asserted. Parent storage stays fixed.

The test checks per-group inode/block/used-directory and SB accounting against
actual allocated numbers, parent links/times/atime, child identities/modes/
times/mapping/dots, prior children, protected inode bytes and external attributes.
Judged e2fsck and complete debugfs reports accompany saved phases. Cold Linux
readback requires exact metadata, namespace/byte identities, parent/dot links,
empty child directories, protected payload/attributes and statvfs credits.
Readback uses O_NOATIME where enumeration/content reads would otherwise alter
the observed bytes. The separate job pins both siblings, uses exact match and
floor one, and retains all logs/reports/images even on failure.

Python syntax, Rust compilation and lint pass locally. **No Linux execution,
artifact audit, full-CI retention or scoped mkdir closure is claimed yet.**
This host has no KVM; no host oracle tools or mounts are used. The first actual
CI run must establish and inspect those results, including any measured whole-
suite output failure; do not preemptively raise unmeasured caps or skip tests.

Local evidence is under `tmp/casefold-mkdir-repair/`: prior exact CI metadata,
fresh red/intermediate/green logs, original/restored normal unit logs, native
occurrence/control retention, identical budget replay, lint logs and source
hashes. These support the checkpoint; committed tests and CI are the portable
handoff rather than assumed persistence of temporary paths.

**Next bounded task:** inspect this repair's exact focused/full CI, investigate
actual failures first, download source/digest-verified mkdir artifacts and audit
their saved raw bytes against Linux/debugfs and exact per-group accounting.
Retain all older unit/whole occurrences and all 418 controls in the CI contexts;
verify trailing shell/CLI checks and retained family evidence before standalone
source/evidence review closure. Full expected totals must come from actual logs,
not this planned matrix. Stages 4–5 remain **not yet quantified**; ownership/
mapping/index growth, aliases, lifecycle/seals, ignored fuzz, recovery and
Windows/SteamOS/real-card gates remain open. Casefold writes stay blocked.

## Executed native mkdir policy red preparation — 2026-10-22

Started from clean fetched `9937200dd46f0cc33e6ec030c1ae28aff7330d7c` on
`casefold-rw`, using Sol 6.1 and the prescribed immutable core source view.
No production, workflow, pin/lock, sibling, global output cap/floor/filter or
write-gate change is included. Both failing audits remain dormant patches.

### Exact preceding CI and complete retained-test audit

Reference implementation `79ed4560371ae89ecd3e2c95581e5f899e0f7594` now passes
all eleven jobs and `ci-ok` in
[full CI 39826871657](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39826871657).
All seven jobs in
[focused run 39826871672](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39826871672)
also pass, including the previously pending fixtures job. The preceding docs
checkpoint `9937200` passes
[full CI 39828986312](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39828986312).
Its production/test/workflow/task/pin bytes are identical to `79ed456`; unchanged
reference images are reused rather than needlessly regenerated for docs.

Five small tier-log archives match advertised SHA-256, exact workflow source,
nonexpired metadata, safe paths and complete ZIP CRCs. Successful occurrences
are compared as multisets, not inferred from totals: both whole suites retain
every preceding 2,121 successful occurrence and add exactly the one mkdir oracle
(**2,122 passed / zero failed / one unchanged ignored slow fuzz test**).
The ignored test remains a broader qualification gate. All **314** guard/
operation controls occur exactly once in standalone unit, aarch64 unit, native
unit, native whole and guest whole contexts. All three unit contexts retain
**1,282** passes, zero failed/ignored; aarch64 retains **527** image passes and
its same existing ignored fuzz test. Native trailing checks execute all twenty
shell checks and 159 installed-CLI checks; standalone CLI also executes 159.

Actual completed job logs confirm floors: units 1,282/440, whole suites
2,122/1,060, aarch64 images 527/450, Unicode 8/8, native oracle 225/140,
kernel 33/33, lwext4 9/7 and installed CLI 159/150. Complete native whole output
is **3,527 lines / 201,757 bytes**, guest whole **3,695 / 211,191**, within the
unchanged 3,700/205,000 and 3,900/213,000 ceilings. No cap repair is needed.
This closes preceding CI retention; it does not qualify a future mkdir repair.

### Executed policy patch and explicit API decisions

The new dormant [CASEFOLD-MKDIR-POLICY-RED.patch](CASEFOLD-MKDIR-POLICY-RED.patch)
adds only a native test module and its cfg(test) registration. It is **29,847
bytes / 794 lines**, SHA-256
`a90024d81eedffe73e09ba3628bf053e27d2a27463f74562406fc2297a573a6a`.
The complete formatted module SHA-256 is
`3edad2ec4f14daea61ced1ce5923ab07b6cfe5d1ae036e2178e8957cdc22c503`.
All expectations use the independently executed eight-layout Linux reference,
with explicit API boundaries kept separate:

- **Ten timestamp reds:** first and back-to-back successful directory creations
  through all five Rust/C entry forms. Every test reaches all six native
  ext2/ext3/ext4 × 1/4 KiB profiles before its aggregate assertion. All ninety
  creations succeed and persist, with correct child times/type/mode/links,
  dots, exact one-block/one-inode debits, parent link increments, unchanged
  parent storage/atime/identity, external attributes and protected peer bytes.
  Parent mtime and ctime remain their old values instead of advancing to the
  controlled first/second time; both creations are checked in the two-step test.
- **Three pathname-priority reds:** string, byte and C paths each execute 36
  unchanged-inode/allocation observations over all six profiles. Regular and
  missing intermediate components currently return final-name ENAMETOOLONG
  rather than the measured ENOTDIR/ENOENT; twelve mismatches per test. Valid
  directory/symlink/dot/dotdot prefixes retain ENAMETOOLONG. This is a deliberate
  mkdir-local semantic correction after reviewing `resolve_new_parent` and
  existing long-name/C/path tests, not permission to change that shared helper's
  pre-resolution length contract for create/mknod/symlink or other families.
- **Fourteen positive controls:** five existing-name/dot/ordinary-long-name
  refusal groups (240 observations), two counted name/generation/FFI groups
  (108), invalid-name-before-conversion checks (24), three path-form/syntax
  groups (234 observations plus eighteen successful symlink/trailing creations),
  thirty fresh-parent 255-byte creations, 24 raw non-UTF-8 byte/C creations,
  and ninety explicit permission-mode creations. New byte names and the
  255-byte identities persist on cold reopen. Ordinary refusals preserve the snapshotted
  fixture inode/namespace records and allocation credits; premature marker
  writes are diagnosed separately by the safety patch, not excused as safe.

In all, each policy execution reaches **252 successful creations and 714
refusal/precedence/syntax observations**, with **14 passed / 13 failed / zero
ignored**. These are native self-built fixtures, not independent emission
qualification. Ext3 is journaled; ext2/ext4 exercise direct commits. Legacy
128-byte and checksummed 256-byte inodes are asserted, not inferred from labels.

Counted names still reject empty/slash/NUL and then overlength before caller
`Into<InodeRef>` or stale/non-directory parent validation. C null-name and
oversized-slice checks retain EINVAL/ENAMETOOLONG. Ordinary valid names still
check live generations/types; no rmdir-specific stale/long priority is copied.
Path APIs retain their absolute-path syntax contract (empty/relative EINVAL),
separate from Linux's cwd-relative empty-path syscall ENOENT. Symbolic errno
mapping is preserved; raw Linux numeric codes do not replace the C ABI table.

The explicit Rust/C low-twelve-bit mode contract retains requested 02750,
01770 and 0750 across every native entry/profile. Linux's controlled syscall
02750-to-0750 observation is recorded separately, not treated as a defect in
this API. No credentials, umask, SGID inheritance or default-ACL rewrite is
included. Path dots/existing directory/file/symlink/dangling names and measured
trailing forms retain EEXIST; regular/missing prefixes and parent symlink
traversal keep their existing qualified meanings outside the long-name defect.

Discovery found one harness error: five simultaneous 255-byte names grew the
1 KiB parent, invalidating the test's assumed single-block debit. The final
boundary test uses a fresh parent per entry and asserts unchanged parent
storage, matching the independently measured non-growing scope. The initial
positive-test failure is not counted as a driver red. No driver was changed
between discovery and the final repeated defect runs.

### Composition, repetition, lint and restored baseline

The original 73-control safety patch remains byte-identical, SHA-256
`575d683b4e47d1028d426b4497632fb0b963fae2d1d298126f3043875bf88932`.
Both patch orders actually pass apply checks and recreate identical complete
module and registration bytes. The final policy serialization is reversed and
reapplied before the runs; both actual combined executions use:

```sh
/workspace/.cloud-setup/run-casefold.sh bash scripts/tier.sh \
  mkdir-combined-first mkdir-combined-first 2200 200000 -- \
  cargo test --lib mkdir_calls:: -- --nocapture --test-threads=1
# Repeat with mkdir-combined-second after reversing/reapplying both patches.
```

Both runs return expected diagnostic **101**, execute exactly the same **100
names/outcomes: 25 passed / 75 failed / zero ignored**, and each complete log is
**1,037 lines / 80,845 bytes**. The local combined ceiling sums the two isolated
diagnostic allowances; no ordinary task/workflow output budget is modified.
All original 73 outcomes remain: eleven passes, 58 current public safety reds
and four constructed future pre-marker SB diagnostics. Those four remain
future-boundary diagnostics, not current public clean-bit regressions. Every
one of the **352** measured write/flush errors or flush unwinds and all **four**
actual post-commit publication unwinds reach their retirement checks. All 54
original primary creation profiles, same-table/uninit and post-commit C-read
compatibility cases remain executed. Together there are **71 current public
safety/policy reds plus four future diagnostics**.

Full lint passes with both failing audits active: formatter, all-target/CLI
clippy with warnings denied, provenance, agent/family/aggregate and Unicode
checks, including all eight table controls. Both patches are then reversed;
production, tests, workflow, tasks and locks are byte-restored to HEAD. The
normal native tier again retains exactly **1,282 successful names / 314
controls**, zero failed/ignored, compared as a multiset against the fresh clean
baseline. The final patch is dormant; ordinary unit CI is not intentionally red.

Evidence is under `tmp/casefold-mkdir-policy/`: preceding job/artifact metadata,
five digest-verified log ZIPs, completed-job floor reports, `ci-retention.json`,
`baseline-unit.log`, discovery/final combined logs, outcome parser/JSON, both
composition source-hash manifests, active/restored lint logs and restored-unit
retention. Regenerate the tests from the committed patches in a new workspace;
temporary evidence paths are local, not a substitute for those patches or CI.

### Next bounded task and remaining gates

Inspect this checkpoint's exact CI, then implement the reviewed shared-context
mkdir migration and measured semantic repair against both patches. Use one
owned inode/buffer result and a mkdir-only full-primary-SB/dirty-marker/complete-
effect boundary; preserve staged allocation flags, parent growth/inline/shared-
table edits, counted validation priorities, mode/errno contracts and optional
C attribute readback under the same context. Stamp parent mtime/ctime in both
in-place and grown-parent paths. Resolve mixed pathname parents before the
final length verdict only in the mkdir path; do not globally rewrite other
creation families. Rebind the four future SB diagnostics to the actual new
boundary and add its real foreign/malformed/missing-SB/context controls when
that boundary exists. Retain all native occurrences and independently qualify
actual driver-emitted Rust/C images with Linux/e2fsck/debugfs before review
closure. Do not claim a semantic/safety repair from these reds alone.

Stages 4–5 remain **not yet quantified**. Broader ownership/mapping/index growth,
aliases, lifecycle/seals, interrupted recovery, ignored fuzz and Windows/
SteamOS/real-card gates remain open. Casefold writes remain blocked.

## Verified independent Linux mkdir policy evidence — 2026-10-22

Reference source `79ed4560371ae89ecd3e2c95581e5f899e0f7594` passes exact
`mkdir-policy-reference` job `119992355646` in
[focused run 39826871672](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39826871672).
The archived exact test executes **one passed / zero failed/ignored**, **565
lines / 64,658 bytes**, within its isolated 3,200 / 200,000 ceiling. The actual
completed job log confirms **one executed test with floor one**, not a zero-
match success. Invocation markers retain **eight mke2fs, forty kernel mounts,
forty judged e2fsck checks and 328 debugfs calls**. No host oracle is used.

### Verified artifact and actual saved-byte audit

Artifact `mkdir-policy-reference` (`12446349094`) is **1,675,744 compressed
bytes / 1,342,500,801 uncompressed bytes / 449 entries**, including forty
32 MiB images. Its exact workflow SHA and advertised SHA-256 both match:
`2f642a1c548844fad8ed4878613ae58a353ee065df073f798cc54f81c4a54195`.
The audit reads images directly from the compressed archive, preserving disk
space; every image/JSON/debugfs/stderr/log member is read and ZIP-CRC checked.

All eight 1/4 KiB × 128-byte legacy/256-byte checksummed × direct/journal
profiles pass **input/names/first/second/boundary** inspection. All **216**
error observations agree across layouts and all **24** successful creations
match their kernel JSON, actual stored inode/directory bytes and debugfs.
The audit checks all **328** reports' inode identities, mode/owner/group,
generation, size/links/sectors/attribute pointers, timestamps and complete
healthy direct/extent mappings against saved bytes. Repeated checks cover
24 primary SB CRC views, sixty descriptors, 512 allocated inode CRC views,
148 directory blocks, 88 external-attribute views and fifty block/45 inode
bitmap CRC views. These are checks over repeated views, not distinct structures.

Every saved phase preserves root and protected peer namespaces, full peer
payloads, raw peer/existing inode records and parent/peer external attributes.
Each new directory has one data block, two links and correct `.`/`..`, with
unchanged raw child bytes in later phases. Actual inode/block groups determine
the exact bitmap/free-inode/free-block/used-directory deltas; no assumption
that Linux and a driver choose the same physical numbers is used. Per layout,
the final ledger is **-3 free blocks, -3 free inodes, +3 used directories**,
policy-parent links +2 and names-parent links +1. Parent storage never grows.
All images are clean, RECOVER is clear and judged e2fsck accepts them.

### Measured Linux answers and API boundaries

Every successful creation advances its parent's **mtime and ctime**, preserves
parent atime, and preserves unrelated entries/data/attributes. Successes cover
parent-symlink/trailing-slash traversal, a raw non-UTF-8 name and an actual new
255-byte basename. Legacy whole-second and checksummed nanosecond timestamps
agree with their independent reports; exact observations are retained.

| Linux path observation | Same result in all eight layouts |
|---|---|
| A single 256-byte component | `ENAMETOOLONG`; new 255-byte creation succeeds |
| Existing directory/file/symlink/dangling name, dots and their measured trailing forms | `EEXIST` |
| A regular intermediate component, including before an overlong final name | `ENOTDIR` |
| A missing intermediate component, including before an overlong final name | `ENOENT` |
| Valid directory/symlink/dot prefixes followed by a 256-byte component | `ENAMETOOLONG` |
| Empty syscall path | `ENOENT` |

These are Linux errno symbols and path semantics. The existing counted-name
API rejects empty/slash/NUL names and validates names before generic parent
conversion; FFI slice/null priorities and symbolic platform errno conversion
remain separate. A Linux observation is not permission to discard those
explicit contracts. The measured mixed-prefix precedence should be reviewed
when preparing pathname policy reds, with baseline priorities accounted for.

Under root uid/gid, zero umask, a non-setgid parent and no default ACL, the
kernel stores requested `02750` as **0750**, requested **01770** as **01770**,
and **0750** as **0750**, consistently in all eight layouts. These facts are
recorded separately: the current explicit Rust/C mode contract and native
controls retain low permission/special bits. Do not silently rewrite that API
or add credential/default-ACL/umask inheritance on the basis of this reference.

Path traversal may update surviving symlink atime. Saved-byte comparison
allows only that atime field and its covering inode CRC to differ, while
preserving every other field and symlink payload. That is an observed Linux
read effect, not a driver defect or weakened peer comparison.

### Remaining gates and next bounded task

The standalone reference measurement/saved-byte audit is complete for these
healthy ordinary, non-inline/non-indexed, non-growing layouts. It is not driver
emission evidence, guarded mkdir qualification, arbitrary ownership/corruption,
inline/index growth, lifecycle, recovery or Windows/SteamOS/card qualification.
The complete reference source is unchanged after the successful VM run; this
follow-up changes only the existing review and handoff. The dormant 73-control
safety patch remains byte-identical and unapplied. Local native baseline and
post-change runs retain all **1,282 names / 314 controls**, and full lint passes.

At inspection, source
[full CI 39826871657](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39826871657)
has successful unit, CLI, fixtures, MSRV, semver, wasm, provenance and aarch64
jobs; native/guest whole suites remain running. Other focused policy/
emission jobs pass; the main focused fixtures job is still running. No complete-
suite retention or overall CI closure is claimed yet. Reuse the verified mkdir
reference bytes while source stays unchanged rather than rerunning the matrix
solely for documentation commits.

Local material is under `tmp/casefold-mkdir-reference/`: exact metadata/job log,
verified ZIP, complete tier log, read-only auditor and `image-audit.json`, source
hash manifest and native/lint retention logs. **Next bounded task using Sol
6.1:** inspect remaining exact full/focused CI, then prepare and actually run
native policy red/positive controls grounded in these measurements, beginning
with advancing parent timestamps. Audit mixed-prefix priorities and special-mode
contract differences explicitly; preserve qualified name/generation/FFI rules
and avoid an unrelated mode/credential rewrite. Compose with the saved safety
audit, verify repeated actual outcomes, lint and restore active source before
the bounded shared-context/owned-plan repair. Stages 4–5 remain not yet
quantified and casefold writes remain blocked.

## Independent Linux mkdir policy oracle added — 2026-10-22

Started from clean fetched `cd22d9c5d22de37a72e85add017b46c613a29a06` on
`casefold-rw`, using Sol 6.1. Its exact full CI `39813130020` now reports
completed success. The prescribed local baseline has **1,282 passes / zero
failed/ignored**, with all **314** guard/operation controls once.

The new exact needs-host test is
`fs::guarded_chmod::needs_host::mkdir_policy::linux_records_mkdir_policy`.
It uses the existing VM-only mke2fs/kernel/e2fsck/debugfs interfaces; no driver
write method is called. Its separate `mkdir-policy-reference` focused job
uses immutable sibling pins, a release exact-name run, floor one, the existing
reference-tier-sized output ceiling, and always-uploaded complete evidence.
Existing jobs, filters, floors and caps are unchanged.

The matrix has eight block-size/inode-size/journal combinations. Each layout
retains **input, names, first, second and boundary** images, JSONL/stderr and
debugfs inode reports. The planned totals are **40 saved images, 216 error
observations and 24 successful directory creations**. These are coverage
requirements, not reported Linux results before actual execution.

- The input is Linux-created ordinary non-inline/non-indexed directories,
  protected peer data and parent/peer external attributes. Legacy/checksummed
  mapping, block size, inode size and journal state are asserted read-only.
- Twenty-seven actual mkdir-at path cases per layout record errno symbols,
  including existing types/symlinks, empty/dot/dotdot/trailing names, 256-byte
  refusal, existing 255-byte names and mixed missing/regular/symlink prefixes.
  Failed attempts must preserve parent/peer fields and free counts. A counted
  filesystem name API is not silently equated with every Linux path syntax.
- Three successful calls cover parent symlink/trailing traversal, raw-byte
  naming and a new 255-byte basename. Advancing time avoids whole-second legacy
  ambiguity; parent mtime/ctime must advance and atime remain unchanged. Exact
  child times and requested/actual special mode bits are recorded rather than
  assuming the driver's current interpretation.
- Controlled root credentials, zero umask and no parent setgid/default ACL
  make ownership/mode observations explicit. This does not introduce a new
  credential/umask/default-ACL API contract or authorize such a repair.
- Kernel snapshots/statvfs and closed-image read-only inspection check child
  type/links/one-block mapping/dot identity, parent links/no growth, unchanged
  peer inode/payload/attributes, every affected group's bitmap/inode/block/
  used-directory debits and matching SB totals. Each phase requires judged
  e2fsck and debugfs reports; optional journal evidence must be clean.

Local Python syntax and Rust test compilation pass without invoking host
oracles. Full lint passes, including all-target clippy; the normal native unit
run retains the entire **1,282-name baseline and all 314 controls**, zero
failed/ignored. Local checks and source manifests are under
`tmp/casefold-mkdir-reference/`. The existing dormant safety patch remains
unapplied and byte-identical; no production code, dependency/pin/lock, sibling
or write gate changes. Shared-agent/provenance/whitespace checks pass.

This cloud host has no local KVM. **Independent VM execution and saved-image
audit remain pending** until the focused CI job actually finishes. Do not
report any Linux mkdir answer, derive policy reds from predictions or enable
casefold writes based on compilation and native self-consistency alone.

**Next bounded task:** inspect the exact focused `mkdir-policy-reference` job
and full CI; download the advertised-digest/workflow-SHA-verified archive and
audit all actual JSON observations and forty saved images, including byte
identities, times, per-group allocation/bitmap/CRC accounting and protected
peers. Reproduce and repair genuine measurement failures first. Then prepare
native semantic reds grounded in the measured policies before implementing
the reviewed shared-context/owned-plan mkdir migration. Stages 4–5 remain not
yet quantified; broader ownership/mapping/lifecycle/recovery/ignored-fuzz and
Windows/SteamOS/card qualification stay open, with casefold writes blocked.

## Executed native mkdir safety red preparation — 2026-10-22

Started from clean fetched `166e43f48921cfdb9f49444877cc9b41754e7ab7` on
`casefold-rw`, using Sol 6.1. Its exact
[full CI 39717854554](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39717854554)
now reports completed success. The prescribed local baseline executes **1,282
unit passes / zero failed/ignored**, with every **314** guard/operation control
once. Existing pins, immutable core source view and caller-exclusion restrictions
remain in effect. Current runtime policy is enforced; public API reads and
default Git work without environment or credential changes.

### Executed patch, repetitions and classified results

The new dormant [CASEFOLD-MKDIR-RED.patch](CASEFOLD-MKDIR-RED.patch) adds only a
test module and its cfg(test) registration. SHA-256:
`575d683b4e47d1028d426b4497632fb0b963fae2d1d298126f3043875bf88932`
(66,491 bytes). It is saved after actual execution, reversed, checked/reapplied
with byte-identical source, and rerun. Both final runs execute exactly the same
**73 names and outcomes**: **11 passed / 62 failed / zero ignored**, exit 101.
Each complete log is **931 lines / 59,622 bytes**; the replay fits the measured
local diagnostic ceiling **1,100 / 100,000**. No global task cap/floor/filter or
output change is included. Final runs use one test thread to preserve readable
complete diagnostics; earlier concurrent discovery output was interleaved.

After execution, the patch's serialized context is narrowed to one line so a
blank context prefix does not fail the repository whitespace check. The final
serialization is reversed/reapplied and its complete module/audit source hashes
match the tested bytes, then active source is restored again. Only diff context
changes; no test or production source changes after the two final runs.

The failures are separated by meaning:

- **58 current public safety reds:** all five entry forms bypass a held
  operation context; clean logical/path refusals write the marker; generic
  conversion and callback/worker reentry escape exclusion; the second path
  verdict can refuse after marker effects; planning/Into and complete-effect
  failures do not consistently retire the mount. Counted C attribute-readback
  workers are admitted in all four direct/journal × 1/4 KiB profiles.
- **Four constructed future ordering diagnostics:** manually staged SB debits
  and unrelated state/sentinels survive marker/raw commit, but their pre-marker
  clean state overwrites the marker. These exercise a future completed-buffer
  ordering requirement, **not** a current public mkdir clean-bit regression.
  There is no new private mkdir boundary yet; bind these to the real boundary
  after it exists, without treating this raw-plan construction as admission.
- **11 passing controls:** prior-dirty refusals, invalid/long/volume/FFI/live-
  generation priorities, ordinary/full/inline/DIR_NLINK creation, parent
  symlink/trailing/case distinctions, asserted same-table splicing, constructed
  multi-group/uninit allocation and post-commit C-read failures retain the
  behavior described below. Passing native controls establish self-consistency,
  not independent Linux compatibility.

### Actual observations, injections and retained behavior

All **352** original observed I/O faults execute before aggregate retirement
assertions: Rust/C each sweep **120 writes, 28 flush errors and 28 flush
unwinds**, covering clean/prior-dirty ext4-direct and ext3-journal mounts at
1/4 KiB. Each observed write/flush position is measured from a successful call
and its injection is asserted reached. All **352** observations violate the
required sticky mount/nine-family/flush/clean-close retirement contract; this
does not mean every later family writes after every individual failure.

A separate actual-public test poisons allocation publication from the child
runtime callback **after allocation planning**, then reaches commit and the
publication unwind in all **four** direct/journal × 1/4 KiB profiles. Raw
verified parent-entry reads confirm the new namespace is already committed.
The initial sticky mount latch is false in each profile; retirement violations
are aggregated only after all four execute. This is not merely a constructed
raw-commit failure or an assumed reachable fault site.

Positive coverage executes **54 primary creation profiles**: thirty all-entry
ext2/ext3/ext4 × 1/4 KiB cases, twelve Rust/C full-parent growth cases, eight
inline-parent edit/conversion cases and four DIR_NLINK unknown-count cases.
Each checks child mapping/size/mode/generation/times/dot identity, parent
links/edits, complete SB/BGD debits/used-directory credits, peer payload/inode
and parent external-attribute preservation, dirty/healthy-close/reopen state,
plus a distinct next child/data allocation. Two additional same-table controls
assert the shared physical inode-table block and both retained inode slices.

One additional **2 KiB / two-group** constructed allocation profile forces
child and parent growth into an unused group with both uninit flags and
unspecified bitmap bytes. It asserts two distinct inodes, three distinct child/
parent blocks, exact SB/per-group debits and used-directory count, bitmap CRCs/
padding, itable-unused, flag publication, second allocation and reopen. Group 0
is artificially reserved to exercise allocation; this is **not** an independent
ownership-qualified filesystem or Linux fixture.

The counted-C readback controls use a zero-capacity read cache and measure the
null-attr call first, so the next actual read is asserted to be post-commit
attribute readback. Four requested-attribute cases observe admitted guarded
workers (red). Four injected first-read failures preserve **EIO after exactly
one committed inode/data allocation**, while a null attr needs no readback
(passing compatibility controls). The synchronous callback joins its shared-
reference worker before the enclosing borrow ends; it never constructs a
conflicting mutable C handle or extends that reference beyond the callback.

Three harness issues are corrected before the final repetitions and are not
driver reds: the first compile used u32 expectations for i64 time fields;
the unused-group fixture initially expected an unused-tail reduction without
setting its initial truthful count (the formatter starts at zero); and the
publication probe first used live lookup, which touched the deliberately
poisoned allocation state before the remaining profiles. It now uses verified
raw parent-entry readback and reaches every publication/retirement observation.
The log parser also accounts for test-status prefixes on nocapture output.

Future private-boundary foreign/missing/malformed-SB and retained-context helper
controls remain promotion-time obligations: the mkdir helper does not yet
exist. Do not introduce a fake boundary to make those tests compile. Runtime/
name/Into and C-interface safety controls here use the actual public entries;
unmeasured parent timestamp and uncertain Linux error policies remain separate.

### Restored source, checks and next task

Full `chore lint` passes with the complete audit active, including all-target
clippy with warnings denied. The dormant patch is reversed again and the active
source is byte-restored. Normal `chore test:unit` retains exactly the baseline's
**1,282 successful-name occurrences / zero failed/ignored**, with all **314**
controls once. The retained `outcomes.json` compares both 73-name red/positive
runs; `retention.json` compares restored units as a multiset. Existing promoted
rmdir/unlink tests and their historical patches are unchanged and must not be
reapplied. No production/test-body/task/workflow/cap/floor/filter/pin/lock/sibling
change remains active; only this handoff, existing review and dormant patch are
part of the checkpoint. Shared-agent, provenance and whitespace checks pass.

Local evidence is under `tmp/casefold-mkdir-red/`: baseline/restored unit logs,
discovery and final complete red logs, command/lint logs, source/patch hashes,
outcome checker/results and baseline-retention data. Reproduce through the
existing pinned wrapper: apply the patch only on its recorded compatible
source, run `scripts/tier.sh` around
`cargo test --lib fs::guarded_chmod::tests::mkdir_calls:: -- --nocapture --test-threads=1`
(expect 101), lint, reverse the patch, then run the normal native unit task.
Do not push an active intentionally failing module to normal CI.

**Next bounded task using Sol 6.1:** add and actually execute independent
reference-only Linux mkdir policy evidence through the existing CI harness:
advancing-clock parent/child times, ordinary dot/trailing/255/256-byte and mixed
path/counted-name errors, controlled mode/ownership expectations, storage and
per-group accounting, plus saved input/output phases and peer preservation.
Assert fixture shape; use judged e2fsck/debugfs/kernel readback and audit actual
saved bytes before policy reds or semantic corrections. This cloud host still
has no local KVM; no host oracle/mount or forbidden implementation source is
used. Then prepare measured native semantic reds before the bounded shared-
context/owned-plan migration. Casefold writes remain blocked; Stages 4–5 remain
not yet quantified, and ownership/mapping/seals/lifecycle/recovery/ignored-fuzz/
Windows/SteamOS/card qualification stay separate gates.

## Public-mkdir staging, marker and dispatch design audit — 2026-10-21

Started from clean, fetched `a98fcdc3669b795b3ad8799cad4e3e395d57e4cf` on
`casefold-rw`, using Sol 6.1. Its exact
[full CI 39711385558](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39711385558)
now passes every job and `ci-ok`. That is a fresh completed status check;
the preceding two fully audited rmdir CI archives remain the occurrence/image
baseline. No unchanged large archive audit or native full-suite rerun is
claimed in this source/design-only checkpoint. Current runtime policy is
enforced, Git/public API reads work, and no environment setting changes.

The detailed audit is in the existing
[casefold review](CASEFOLD-INDEX-REVIEW.md#public-mkdir-staging-marker-and-dispatch-design-audit--2026-10-21).
It traces the Rust string/bytes/counted and C path/counted forms, FFI checks,
both nested volume verdicts, basename/name/Into/live-generation/duplicate/link-
limit ordering, existing allocators, bitmap initialization/itable-unused,
flavor-specific child mapping/times/generation, dot entries, casefold
inheritance, parent inline/conversion/index/growth paths and same-table splicing.
The final branches already commit through the configured journal or direct
writer despite an outdated “Not journaled” comment. Complete effects include
checkpoint/flush and allocation publication; the journal mutex starts after
planning and does not replace a whole-operation guard.

The existing public entries use marker-writing `refuse_write` before planning
and have no shared operation entry. The bounded migration design acquires one
nonblocking context before callbacks, preserves both pure volume verdicts with
private dispatch, completes one owned new-inode/buffer result and uses a
mkdir-only full-SB normalization/marker/complete-commit boundary. Every staged
allocation/counter/checksum, inline/growth edit and DIR_NLINK rule must survive.
Do not weaken qualified families or turn the internal plan into generic
ownership admission. A constructed pre-marker clean-SB ordering check will be
a future-boundary diagnostic, not a reproduced current public clean-bit bug.

The counted C form needs special treatment: optional `attr_out` currently
reads the newly committed inode after the filesystem call. Its new adapter
must keep the same context through an owned inode snapshot, then format the
C attributes without further filesystem reads. Preserve null-attr behavior,
FFI validation priorities, inode/zero versus zero/-1 return conventions and
the documented possibility of an error after creation succeeded. Do not retry
the allocation or claim rollback on a post-commit readback error.

Concrete proposed controls cover held contexts across all five forms, legal
same-thread/worker/runtime/generation/device callback reentry, clean/already-
dirty untouched refusals, error/Into priorities, six native flavor/block-size
profiles plus asserted inline/full/same-table/multi-group-uninit shapes,
distinct child/parent/map allocations, complete per-group credits, private
foreign/malformed-SB diagnostics, every actual write/flush fault position and
retained-context/publication unwinds. Counted C attribute callbacks and
post-commit read failures need their own observations. These groups have not
yet been executed; source inspection alone is not a failing regression test.

Parent mtime/ctime is not explicitly stamped by the traced parent-link and
directory-inode helpers. That and uncertain dot/trailing/long-name, mixed-error,
mode/ownership and resource policy require independent reference-only Linux
measurements before semantic corrections. Keep controlled credentials/API
expectations explicit; do not silently add umask/setgid/default-ACL behavior.
Existing child casefold inheritance is preserved, with public writes blocked.
Later emitted-image qualification must audit every saved dirty/closed phase
with Linux/e2fsprogs and raw counters/bitmaps/checksums/peer preservation,
including actual parent-growth allocations. Existing crash/inline/index-growth/
unique-inode/DIR_NLINK tests remain; a remount-only mkdir crash check is not
complete ownership/accounting or arbitrary power-loss qualification.

Only this existing handoff and review change. Shared-agent integrity,
provenance and whitespace checks pass. No production/test/task/workflow,
cap/floor/filter, sibling/pin/lock or write-gate change; no host oracle/mount,
local VM or forbidden implementation source. Local audit material is under
`tmp/casefold-mkdir-design/`. Stages 4–5 remain not yet quantified.

**Next bounded task using Sol 6.1:** prepare and actually execute the reviewed
native mkdir safety red/positive controls using the existing fixture machinery.
Save a dormant patch only after execution, reapply/rerun with identical source
and successful/failed name occurrences, then restore active source and retain
all 1,282 unit names/314 controls with lint passed. Expose every intended
injection before aggregate assertions; distinguish real public failures from
future constructed-boundary diagnostics and fixture/compile failures. Then
measure independent Linux policy before policy reds or semantic repairs.
Do not implement the guard yet, reapply promoted rmdir/unlink patches, migrate
another family or enable casefold writes. Ownership aliases, mapping/seals,
lifecycle/recovery, ignored fuzz and Windows/SteamOS/card gates remain open.

## Scoped rmdir full-CI review closure — 2026-10-22

Started from clean, fetched `49d34846debdb1a6804df0d6755dcbd50efff945` on
`casefold-rw`. Current runtime policy is enforced; default Git and public REST
status, completed job-log and redirected artifact reads work through the
inherited proxy/CA. No environment or credential configuration changes.

### Completed exact-source CI and retention

Both the [budget-repair full run 39702689248](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39702689248)
at `0da4221863f14d2283cf2658242a1bdd38c33698` and the
[evidence-checkpoint full run 39703344673](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39703344673)
at `49d34846debdb1a6804df0d6755dcbd50efff945` now **pass all eleven jobs,
including `ci-ok`**. Five tier archives per run are downloaded with exact
workflow-SHA, advertised SHA-256, ZIP CRC and extraction-path checks.

For each run, standalone, aarch64 and native units retain exactly **1,282**
successful occurrences with zero failed/ignored. Native and guest whole suites
each retain exactly **2,121** successful occurrences, equal as multisets to the
complete implementation-source logs: every 2,034 preceding pass plus 86 native
controls and one rmdir-emission oracle. All **314** guard/operation controls
occur exactly once in each of those **five contexts**. Aarch64 retains every
**527** successful image-test occurrence. Both whole suites and aarch64 images
retain the same single ignored `deep_walk_byte_flip_fuzz_never_panics`; it is
not a new skip and remains a broader qualification gate.

| Exact run | Native whole log (lines / bytes) | Guest whole log (lines / bytes) | Trailing native checks |
|---|---|---|---|
| `39702689248` | 3,526 / 201,672 | 3,693 / 210,992 | 20 shell / 159 installed CLI passed |
| `39703344673` | 3,526 / 201,673 | 3,693 / 210,993 | 20 shell / 159 installed CLI passed |

These actual complete logs fit the repaired **3,700 / 205,000** native and
**3,900 / 213,000** guest limits. All unit logs fit **1,560 / 115,500**.
The latest actual native/guest job logs confirm normal wrappers and unchanged
floors: units 1,282/440, oracle 225/140, kernel 33/33, lwext4 9/7, whole suites
2,121/1,060 and installed CLI 159/150. Native shell logs name every existing
`tests/scripts/*.sh` exactly once and record twenty passes. Both native and
standalone installed-CLI archives retain all six files / 159 checks.

Latest-head artifact SHA-256 values, verified against the API metadata:

| Artifact | SHA-256 |
|---|---|
| `tier-logs-unit` | `d1c2f754e7069d230a423af45003f6b90ea5316d8b82702b5508f69989844b4c` |
| `tier-logs-aarch64` | `6e31bacd082fc892a9b07b51f9a566e16b13df282c54344247988264d67cd00f` |
| `tier-logs-x86_64` | `77b450be9ee6d65c43cfe6ed8c97c16d34245c3b0af650591fa391d7989e7ea6` |
| `tier-logs-suite-in-vm` | `51fd3e89d6986a3ddab5f9823995442f92367c8da766c0457fb6c051b87f65b6` |
| `tier-logs-cli` | `6c97b71443b12dc8396f2c72b81d2817dfe03862a40168f10d935f4a75d73a77` |

Local evidence is under `tmp/casefold-rmdir-closure/`: exact run/job/artifact
metadata, ten verified ZIPs and extracted logs, both completed latest-head
native/guest job logs, `check_retention.py` and `retention.json`. The read-only
checker initially assumed zero ignored aarch64 image tests; comparison with the
preceding archive confirms the same pre-existing slow fuzz ignore there too.
The corrected check preserves it explicitly; no driver or test change is made.

### Closure, unchanged restrictions and next bounded task

Compared with independently tested `fbc334e`, source, active/dormant tests,
scripts, focused/full workflow bodies and dependency/tool pins are unchanged.
Only the previously measured budget configuration and review/handoff text
differ. The completed standalone source walk and every new/retained focused
saved-byte audit in the preceding ledger remain applicable. No image audit or
native unit/lint rerun is presented as fresh evidence for unchanged code.

The standalone rmdir review is **closed within the caller-exclusion,
healthy-layout and closed-reference-handle contract**. No further scoped
blocker is found. This does not establish arbitrary ownership/corruption,
retained target handles, complete mapping/plan seals, unguarded writers,
lifecycle, recovery or Windows/SteamOS/card qualification. The ignored fuzz gate
remains open and **casefold writes stay blocked**. Stages 4–5 are still not yet
quantified; closing one operation family is not stage completion.

Only this existing handoff and `CASEFOLD-INDEX-REVIEW.md` change in the closure
checkpoint. Shared-agent integrity, provenance and whitespace checks pass under
the pinned source view and output wrapper. No new production/test/task/workflow,
cap/floor/filter, pin/lock, sibling or write-gate change is made. No host oracle,
mount, local VM or forbidden implementation source is used.

**Next bounded task using Sol 6.1:** audit public `mkdir` string/byte/counted
Rust and C dispatch, refusal/name/generation ordering, inode/data allocation,
inline/in-place/grown-parent splicing, inheritance, counters/checksums and the
complete marker/commit/publication boundary. Directory creation is the bounded
counterpart to the now-reviewed deletion path and adds allocation-side staging
obligations. Record a single-context owned-plan design and concrete native
red/positive controls before migration. This checkpoint chooses that audit;
it does not claim the audit, new reds or independent mkdir policy measurements
have executed. Preserve every qualified family and write gate; semantic changes
need independent Linux policy evidence and executed reds first.

## Verified rmdir Linux emission and whole-suite budget repair — 2026-10-21

At clean fetched `fbc334ecebb5523a20891ca0248172f0c6158fbd`, exact focused
[run 39684661329](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39684661329)
passes every job, including new `rmdir-emission` job `119515552349`. The complete
exact test runs once: **one passed / zero failed / zero ignored**, **669 lines /
80,886 bytes**, with floor one and isolated cap passed. Actual invocation records
retain **16 mke2fs, 64 kernel mounts, 112 judged e2fsck checks and 320 debugfs
reports**. These are executed existing-CI VM results, not local oracle calls.

### Saved-byte and independent readback audit

The rmdir archive's advertised SHA-256 matches:
`18d05ba6ab52464581c3971ff65de51e641811e660e89821f738750d8493018e`.
Its **4,335,231 compressed bytes / 3,758,365,415 uncompressed bytes / 561 entries**
remain compressed; read-only audit checks all **16 profiles and 112 actual images**.
The matrix covers Rust/C, 1/4 KiB, legacy 128-byte/checksummed extent-mapped
256-byte inodes and direct/journal layouts. Input shapes are actual ordinary
Linux-created, non-inline/non-indexed one-block targets and asserted parent shape.
All **48 closed-target removals** and saved input/dirty/closed phases are audited.

Raw bytes and independent kernel/debugfs reports agree on:

- Exact live names and parent links/times: parent atime is preserved, mtime/ctime
  advance on each removal, and links fall from seven to four. Real 255-byte
  directory removal succeeds; ordinary refusal byte preservation is executed by
  the exact test. Native policy controls remain the separate error-priority gate.
- Exclusive data/EA release, shared data release with the peer's EA retained at
  one reference, and boundary data release. Final accounting per profile is
  **four blocks / three inodes freed / used directories minus three**. Every
  per-group credit matches the corresponding released bitmap bits; SB/BGD
  totals reconcile. Cleared target slots retain generation and phase-correct
  dtime, with valid freed-inode checksums.
- Peer inode, payload (including legacy single-indirect storage), directory
  entries and attributes remain preserved. Regular/nonempty siblings retain
  inode/content bytes. Linux `os.readlink` readback can advance the surviving
  symlink's atime between phases: the audit permits that observed atime/CRC
  change while requiring all other fields/content and each phase's dirty/closed
  symlink inode to agree. It does not misattribute observer reads to the writer.
- Dirty snapshots retain the non-clean bit; normal closes restore clean state
  with no replay-needed flag. Raw CRC checks include **56 SB, 140 BGD, 656 live
  inode, 96 freed inode, 140 initialized block bitmap, 84 initialized inode
  bitmap, 80 EA and 344 directory-block validations**, including repeated views.
  All 112 judged e2fsck checks and all 64 mounts/readbacks pass independently.

Source review traces all five entries, private same-context path/count dispatch,
name/parent/generation/volume/FFI ordering, untouched staging refusals and planning
unwinds, inline/block empty checks, matched entry removal, storage/attribute/group
credits and buffered same-table parent splicing. The actual rmdir-only SB boundary
preserves credits/state/outside bytes, and effects cover marker, journal/direct
commit, flush and post-durability publication. Sticky errors/unwinds prevent
subsequent guarded writes, flush and clean close. No additional blocker is found
in this source walk within caller exclusion, healthy asserted layouts and closed
reference target handles. It does not close broader ownership/mapping/lifecycle,
arbitrary-plan admission, inline/indexed/grown-empty Linux or recovery gates.

### Complete-suite retention and actual measured failures

Exact full [run 39684661421](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39684661421)
fails only the two whole-suite output wrappers and their aggregate. Both actual
complete logs have **2,121 passed / zero failed / one unchanged ignored slow fuzz
case**. SHA-256-verified archives retain **every one of the 2,034 prior passing
occurrences and add exactly 87** (86 native controls plus the rmdir emission
oracle). All **314 controls occur once in five CI contexts**; all three unit
contexts retain exactly **1,282** names, and aarch64 retains all **527** image
tests. Standalone CLI, unit, aarch64, fixtures, wasm, MSRV, semver and provenance
jobs pass. Native checks after the failed whole-suite wrapper, including twenty
shell/159 installed-CLI checks, did **not execute** in that job and remain gates.

| Complete log | Actual output | Old byte limit / real job and identical replay | Repaired byte limit / identical replay |
|---|---|---|---|
| Native | **3,525 lines / 201,559 bytes** | 195,000 / exit 65 | **205,000 / pass** |
| Guest | **3,695 lines / 210,972 bytes** | 203,000 / exit 65 | **213,000 / pass** |

The same complete bytes compare identical across original, old-red and new-green
replay logs. Limits grow by 10,000 bytes each after actual failures; growth from
previous 192,859 / 202,180-byte reference-source logs is 8,700 / 8,792 bytes.
The repaired limits retain 3,441 / 2,028 bytes of headroom. Line caps stay
**3,700 / 3,900**, whole floors stay **1,060**, and all filters, commands, output,
unit limits, pins/locks/siblings and write gates are unchanged. Both replay floor
checks count all 2,121 passes. Output-budget and all **87 CI-profile checks** pass;
full lint passes. No production or test-body repair is made.

### Retained evidence and working public-access fallback

All remaining focused archives are now downloaded with their advertised digests
verified and workflow metadata bound to exact `fbc334e`. All fifteen main exact
focused logs pass with one test/zero failed/ignored each. Fresh read-only audits
pass **16 setter layouts, eight first-attribute profiles and all 84 removal/
retained-metadata profiles**. The separate setter-policy archive retains 32
ordinary observations, 16 byte-identical Rust/C images and four explicitly
diagnostic duplicate cases. The unlink emission audit passes all 16 layouts /
80 images; its reference audit passes eight profiles / 24 images / 40 error
observations / eight 255-byte removals. The unchanged rmdir reference audit passes
eight profiles / 32 images / 184 error observations / 24 removals. All five
symlink tests and their selected saved namespace/boundary images pass. None of
these ordinary/refusal/scratch-image results enables public casefold writes or
substitutes for recovery/ownership/Windows/card qualification.

The initial authenticated API downloads worked, then the inherited API-helper/
`gh` credential returned **HTTP 401 Bad credentials**. The cloud policy remains
current/enforced, and default Git fetch/push works. Public REST requests **without
an Authorization header** successfully read this public repository's run metadata
and artifact downloads; the usual fresh unauthenticated storage redirect is
followed through the inherited proxy/CA trust. Source metadata and artifact SHA-256
checks remain enforced. No configured credential, proxy, guard variable, network
policy or environment setting is changed. The reconnect request is no longer
needed for this public read workflow. This does not verify authenticated
Actions-write authority while that credential continues returning 401. Prefer the
working default Git route rather than forcing the rejected `gh` credential helper.
Do not treat 401 as Cloudflare inspection failure or as a need to add hosts.

### Next bounded task and remaining closure gate

The measured two-byte-cap repair is pushed as
`0da4221863f14d2283cf2658242a1bdd38c33698`. Its exact
[full CI 39702689248](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39702689248)
is **in progress**. Unit, CLI, fixtures, MSRV, semver, wasm and provenance jobs
already report success; aarch64 and native/guest whole jobs remain pending.
Production/test/workflow bytes remain identical to the independently tested
`fbc334e`; only the two measured caps and review/handoff text change.

**Next bounded task using Sol 6.1:** inspect the budget repair's completed exact
full CI, verify both complete-suite occurrence multisets and normal wrapper
results, all 314 controls/five contexts, unit/image retention and native trailing
twenty shell/159 installed-CLI checks. Repair only actual new failures, then close
the scoped rmdir review within its caller-exclusion, healthy-layout and closed-
reference-handle contract. The standalone source/saved-evidence review is complete;
no additional scoped blocker is found. Reuse completed new/retained archive audits
while their source remains identical rather than redoing them for configuration/
docs changes. Do not reapply promoted patches or advance another unguarded family
before closure. **Stages 4–5 remain not yet quantified; casefold writes remain
blocked.** Windows/SteamOS/card, retained target handles, arbitrary corrupt
ownership/mapping, lifecycle, recovery and indexed-growth qualification remain
separate gates. Full-CI/trailing-check closure is not yet claimed.

## Guarded public rmdir implementation — 2026-10-21

Fetched clean `482bbefc5522a13fcbecd8154e44604a98a13cef`; its exact full
[CI 39654022971](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39654022971)
and preceding docs-only `39646504328` now pass every job and ci-ok. Prior
reference-source focused/full evidence and audited Linux policy remain valid.
The fresh prescribed native baseline passes **1,196 names / 228 controls**.
Before production edits, both unchanged dormant patches are actually reapplied:
**82 tests / 15 pass / 67 fail / zero ignored**, exit 101, 1,151 lines. These
retain the original six/49 safety outcomes plus nine/18 measured policy outcomes.
Four failures are constructed future-SB diagnostics, not current public defects.

### Implemented boundary and measured policy

All five existing Rust/C rmdir entries now use the same nonblocking mount-bound
context as the seven guarded metadata/unlink families. Public paths privately
dispatch through `rmdir_bytes_in` and `rmdir_path_in`, retaining both pure volume
verdicts without reacquiring the guard. Counted calls check names and dot/dotdot
before arbitrary caller conversion; live parent/generation errors precede ordinary
long-name policy. C foreign-argument validation and existing symbolic errno
numbers are unchanged.

The existing deletion algorithm is extracted into owned `plan_rmdir_at` staging.
All existing inline/size-covered empty checks, checksum refusals, matched stored
entry identity, data/map/attribute release, per-group credits, used-directory
counts, freed generation/dtime and buffered parent-inode splicing are retained.
Successful removals now stamp parent mtime/ctime while preserving atime. Ordinary
256-byte final names are NameTooLong after a live directory parent; real 255-byte
names still succeed. Path parents are checked before final-dot policy so a regular
parent yields ENOTDIR, while counted dot-before-conversion/generation stays intact.

`commit_rmdir_in` checks mount/context identity before callbacks, validates the
required staged full primary-SB block/magic, clears only its clean bit and refreshes
its checksum. It retains the owned plan's counters, other state and bytes outside
the SB, then wraps marker-once and complete commit/checkpoint/flush/allocation
publication in failure-latching effects. Returned errors and Rust unwinds retire
all eight guarded families, flush and clean close. Other writers still require
caller exclusion; no broader ownership readiness or arbitrary-buffer admission
is published. The qualified unlink boundary and raw commit are unchanged.

### Executed native controls and retained baseline

Both dormant audits are promoted to active `rmdir_calls` and `policy_rmdir_calls`.
**Do not reapply either rmdir patch onto this promoted source.** Their historical
bytes remain unchanged (SHA-256 `4cf8a2ac2e55d1595530c980cb3bf0adfb399cede3e81d68ee3dfe022bcd5317`
and `2be5a176724603262e9d0a4c2dfc0552ddbb61c3db7c3173fcc39e2a018663d3`).
The four constructed-credit diagnostics now exercise the actual private rmdir
boundary, retaining test names, credits, non-clean state/checksum and 4 KiB
sentinels. They are not full deletion/bitmap or arbitrary-plan admission tests.

All original **82 controls now pass**. Four added controls cover foreign contexts
and missing/malformed staged SBs with zero callbacks, returned marker/commit errors
while retaining the context, post-durability publication unwind while retaining
the context, and a second-volume-verdict precedence edge. All **86** pass with
zero failed/ignored (complete isolated log **878 lines / 70,228 bytes**). All
**340 original observed write/error/flush-unwind injections execute**, with zero
eight-family/flush/close retirement violations. The existing 247 valid ordinary/
inline/shared-attribute/DIR_NLINK/path removals remain exercised. Each measured
policy red now passes all six native layouts and all five entries remain covered.

Initial source review reproduced a new edge on `/ascii/.`: checking the resolved
regular parent before the second writability verdict wrongly returned a path
error when the device revoked writes during lookup. The exact added test first
fails (one actual failure/exit 101), then passes after private path dispatch moves
the second pure verdict before parent policy. This preserves both Linux-measured
path behavior and pre-existing dynamic-volume priority. A first retained-error
harness assertion incorrectly expected Uncertain for a returned error; inspection
of the existing operation contract corrected it to Faulted. Unwinds remain
Uncertain while the context is retained. That assertion correction is not a
production-defect red.

Full native execution retains **all 1,196 preceding passing occurrences and adds
exactly 86**, giving **1,282 passed / zero failed / zero ignored**, with **all
314 guard/operation controls once**. Full lint passes, including formatting,
clippy, shared-agent/family/aggregate/provenance and Unicode gates. Python oracle
syntax parses, and the new needs-host test compiles; it has not run locally.
The cloud host has no harness/fixtures/KVM qualification path; no host e2fsprogs,
mount, fake oracle result or skipped test substitutes for independent execution.

### Measured unit output-budget repair

The actual full native log measures **1,530 lines / 111,458 bytes**, retaining
all test output. The old 1,460 / 106,500 wrapper returns **65** with zero test
failures. Byte-identical complete-log replay reproduces that failure and passes
at the unit-only **1,560 / 115,500** repair, leaving 30 lines / 4,042 bytes of
headroom. Growth from the preceding measured 1,444 / 102,845 log is 86 lines /
8,613 bytes; rounding adds 100 lines / 9,000 bytes to the previous limits.
Both replay logs compare byte-for-byte with the actual saved log; floor 440
still counts all 1,282 tests. The output-budget shell contract and all **87 CI
profile checks** pass. Commands, overflow checking, filters, test floor, output
and whole native/guest caps remain unchanged. A final actual `chore test:unit`
run with the repaired limits now **passes normally**, again retaining exactly
all 1,282 occurrences/all 314 controls, zero failed/ignored. Its complete log is
**1,529 lines / 111,398 bytes** and floor 440 passes.

### Independent emission added; execution and review remain gates

New exact needs-host test
`fs::guarded_chmod::needs_host::rmdir::linux_checks_public_rmdir_metadata`
creates Linux inputs for **16 planned Rust/C × 1/4 KiB × 128/256-byte inode ×
direct/journal profiles**. Each asserts ordinary non-inline/non-indexed parents
and one-block empty targets, real exclusive/shared external attributes, a live
peer and a real 255-byte boundary directory. Three successful removals per profile
exercise Rust string/byte/at and C path/at forms with advancing clocks. Ordinary
refusals compare whole image bytes before any effects. Per-group block/inode/
used-directory credits, actual released bits, shared refcount/peer bytes,
parent timestamps/links, freed generation/dtime and dirty state are asserted.

The new focused **rmdir-emission** job requires exactly one test/floor one under
a separate 3,200-line / 200,000-byte ceiling, with the same pinned tool/core/
harness versions and a 25-minute timeout. It retains intended **112 images**:
16 inputs plus dirty/closed snapshots for each of three phases. Expected oracle
work is **16 mke2fs, 64 kernel mounts, 112 judged e2fsck checks and 320 debugfs
reports**, with saved Linux JSONL parent/peer/name/statvfs readbacks. These are
planned counts, **not executed evidence** before exact CI and saved-byte audit.
The input Linux free ledger is compared with raw SB counts. Kernel readback uses
O_NOATIME and closes target-directory handles; no retained-target-handle claim.

**Next bounded task using Sol 6.1:** inspect this implementation's exact focused
and full CI, prioritizing actual rmdir-emission outcomes and SHA-256-verified
saved images/reports. Repair actual failures without weakening filters, floors
or output, then audit all emitted layouts, retained metadata/policy/symlink
matrices and complete-suite occurrence retention. Expected whole result is all
2,034 preceding passes plus 86 native controls and one emission oracle (**2,121**),
with one unchanged ignored slow fuzz case; this is not a completed-result claim.
Leave native whole 3,700 / 195,000 and guest 3,900 / 203,000 caps until actual
measured failures justify any repair. Require standalone rmdir family/source/
evidence review before scoped closure. Do not reapply promoted unlink or rmdir
patches. Stage 4–5 progress remains **not yet quantified**; broader corrupt
ownership/mapping, lifecycle, recovery, indexed/grown-empty/inline Linux emission,
retained target handles and Windows/SteamOS/card qualification remain open.
**Casefold writes remain blocked.**

## Executed native rmdir policy red preparation — 2026-10-21

At fetched `e5eab76dcc29a26c02c42196915ab92e3f394ffe`, the independently
measured Linux contract is now exercised against all five native Rust/C entry
forms. Production source is unchanged. The new dormant
[CASEFOLD-RMDIR-POLICY-RED.patch](CASEFOLD-RMDIR-POLICY-RED.patch) adds only a
`policy_rmdir_calls` test module and its declaration; apply it for the later
repair, alongside the unchanged 55-control `CASEFOLD-RMDIR-RED.patch`.
Do not reapply the historical unlink patches, whose tests are already promoted.

### Executed controls and measured failures

The final isolated policy execution runs **27 names: nine passed / 18 failed /
zero ignored**, exit **101**, **209 lines / 31,321 bytes**. All 18 failures
reach every one of the **six native profiles** before their aggregate assertion:
1/4 KiB crossed with Ext2 legacy/direct, Ext3 legacy/journal and Ext4
checksummed/direct, with actual inode size, checksum and journal shape asserted.
This native six-layout matrix is distinct from the eight Linux reference layouts.
The source builds and runs; compilation, missing fixtures and zero-match filters
are not counted as policy reds.

| Controls | Actual result and retained contract |
|---|---|
| First and second parent timestamps, all five forms (ten tests) | All fail with stale parent mtime/ctime at advancing clocks `1700000100` and `1700000200`; the two-removal tests check both updates. All 90 removals succeed, with parent atime/link count, target inode release, block/inode credits, peer inode/data/attributes and reopen checks retained. |
| Ordinary 256-byte names, all five forms (five tests) | All fail: current ENOENT versus measured symbolic ENAMETOOLONG. All 30 calls preserve namespace/inode metadata and allocation counts. The C ABI keeps its existing symbolic value 63, not Linux raw 36. |
| Mixed paths, Rust string/bytes and C path (three tests) | All fail after 54 observations each. Valid symlink/directory/dot/dotdot prefixes reach ENOENT instead of ENAMETOOLONG; `regular/.` reaches EINVAL instead of ENOTDIR. Earlier missing/non-directory prefixes and missing-dot/symlink-dot answers remain checked. All metadata/count preservation assertions pass. |
| Ordinary refusals, all five forms (five tests) | All pass: 234 observations preserve raw parent/target/peer inode metadata and allocation counts for missing, regular, symlink, nonempty, dot/dotdot and path trailing-slash cases. |
| Counted-name/generation priorities, Rust/C at forms (two tests) | Both pass across all six profiles: 108 stale, non-directory-parent, invalid-parent, slash/NUL, dot/dotdot and empty-name observations. Invalid names/dot policy precede caller generation checks as before; live parent/generation errors precede ordinary long-name policy. Counted empty remains EINVAL. Counted NUL does not go through CString. |
| Boundary and volume controls (two tests) | Both pass: 30 real 255-byte directory removals free the target and credit block/inode counts; C success clears errno. Thirty read-only refusals take precedence over overlong names and preserve metadata/counts. |

The mixed `regular/.` red is a concrete additional ordering requirement for the
later path repair: parent resolution currently returns a final regular inode,
then the counted helper rejects dot before checking that parent. Validate the
resolved path parent before applying final-dot policy, while preserving the
counted API's existing dot-before-generation/conversion contract. No shared path
walker or unrelated operation is changed in this preparation checkpoint.
Refusal metadata controls deliberately do not claim untouched whole-device bytes;
the original safety audit separately reproduces premature dirty-marker effects.
The new tests execute **120 successful removals and 564 refusal observations**,
excluding fixture construction.

### Composition, source restoration and normal validation

The policy patch is **22,858 bytes / 592 patch lines**, SHA-256:
`2be5a176724603262e9d0a4c2dfc0552ddbb61c3db7c3173fcc39e2a018663d3`.
The original 47,419-byte patch is unchanged, SHA-256:
`4cf8a2ac2e55d1595530c980cb3bf0adfb399cede3e81d68ee3dfe022bcd5317`.
Both patch orders actually apply and recreate identical hashes for the module
and both test files. Both actual combined executions run the exact same
**82 names: 15 passed / 67 failed / zero ignored**, exit **101**. Each original
55-control outcome remains **six pass / 49 fail**; the new 27 remain nine / 18.
The combined failures retain **63 actual public-boundary/policy reds and four
future constructed staged-superblock diagnostics**, not 67 current public
clean-bit defects. All 340 original observed write/error/Rust-device-unwind
injections execute before their retirement assertions. The original 127 plus
new 120 successful removals give 247 combined valid removals. Complete logs are
**1,151 lines / 101,888 bytes** and **1,151 / 101,887**, within the isolated
2,500-line / 220,000-byte audit ceiling; no output is discarded.

Full `chore lint` passes with both audits active, including formatting/clippy,
shared-agent/family/aggregate/provenance and frozen-Unicode checks. Both patches
are reversed and all active source/workflow/pin hashes restored byte-identically.
The prescribed native baseline and restored runs retain the **exact same 1,196
passing occurrences and all 228 controls**, with zero failed/ignored; restored
output is **1,444 lines / 102,845 bytes**, within unchanged 1,460 / 106,500.
No production repair, CI test/filter/floor/global-cap change,
lock/pin/sibling change, write enablement or host oracle execution is included.
An initial overly broad local `--lib` baseline selected 44 fixture/VM-dependent
tests, which failed on unavailable fixtures/harness; those are invocation failures,
not policy reds. The prescribed `chore test:unit` baseline passes all 1,196 names.
An early mixed-path assertion stopped at the first actual ordering mismatch;
the final tested audit aggregates those mismatches and reaches all six profiles.

To reproduce the combined audit inside the documented pinned tool/source view:

```sh
git apply docs/CASEFOLD-RMDIR-RED.patch
git apply docs/CASEFOLD-RMDIR-POLICY-RED.patch
bash scripts/tier.sh 'combined rmdir audit' rmdir-policy-combined 2500 220000 -- scripts/test.sh --locked --lib rmdir_calls:: -- --show-output --test-threads=1
```

Exit 101 is expected until repair; inspect the complete log and exact 82 outcomes.
For preparation-only restoration, reverse both patches before normal native CI:

```sh
git apply -R docs/CASEFOLD-RMDIR-POLICY-RED.patch
git apply -R docs/CASEFOLD-RMDIR-RED.patch
chore test:unit
```

### Independent CI follow-up and next bounded task

The preceding source `ccef341` now passes **every focused job** in
[39644593294](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39644593294),
including the completed main fixture job. Its downloaded job log retains all
15 exact focused test/floor checks, with no failure. The rmdir reference's
previously audited eight profiles/32 images/184 errors/24 removals remain valid;
no new driver-emission or repeated retained-image audit is claimed here.

The SHA-256-verified full-CI guest archive from
[39644593304](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39644593304)
retains **all 2,033 preceding passing occurrences and adds exactly the rmdir
reference test**: **2,034 passed / zero failed / one unchanged ignored slow fuzz
case**. All 228 controls occur once. Its complete log is **3,605 lines /
202,180 bytes**, within the unchanged 3,900 / 203,000 cap. Verified standalone
and aarch64 unit archives retain exactly 1,196 names/all 228 controls; aarch64
retains all 527 image tests. The exact source's full run now passes **every job and ci-ok**. Its separately
SHA-256-verified native whole log also retains **2,033 + one = 2,034 passes**,
zero failed/one unchanged ignored, measuring **3,438 lines / 192,859 bytes**
within unchanged 3,700 / 195,000. All 228 controls occur once in **five contexts**;
all three unit contexts retain 1,196 names and aarch64 retains 527 image tests.
Native trailing checks pass **twenty shell checks and 159 CLI checks**. No cap,
floor, output or filter repair is needed. Subsequent docs-only run `39646504328`
is still pending at this observation; it does not change the verified source.

**Next bounded task using Sol 6.1:** inspect this checkpoint's exact CI outcomes,
then perform the reviewed public-rmdir guarded/semantic repair using both dormant
patches. Preserve one shared context, private nested path dispatch and the second
pure volume verdict; complete an owned plan before marker/commit and normalize
staged SB clean state without losing credits/checksums. Apply measured parent
times/name errors and path-parent precedence while retaining counted-name,
generation, volume and FFI priorities. Promote the exercised controls, add actual
foreign-context/malformed-SB/retained-publication controls when the private API
exists, retain all 1,196 baseline names/228 controls, and add independent driver-
emission evidence for reclamation, attributes/peer preservation and timestamps.
A standalone family/evidence review and complete-suite retention remain later
closure gates. No arbitrary-corruption ownership/mapping, retained-target-handle,
inline/indexed/grown-empty Linux qualification, lifecycle/recovery or Windows/
SteamOS/card readiness is claimed. **Stages 4–5 remain not yet quantified and
casefold writes remain blocked.**

## Executed independent Linux rmdir policy evidence — 2026-10-21

Exact source `ccef341391c69c2bc49529e43766a070eba93c4b` passes the
`rmdir-policy-reference` job in [focused run 39644593294](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39644593294).
Actual job logs execute the exact reference test **once**, with one passed,
zero failed/ignored, **405 lines / 42,482 bytes**, passing floor one and the
unchanged isolated ceiling. Invocation evidence shows **eight mke2fs runs,
32 kernel mounts, 32 judged e2fsck checks and 192 debugfs reports**. These are
actual existing-VM executions, not host oracle calls or inferred qualification
from compilation.

The downloaded compressed archive is **1,302,607 bytes**, preserving
**1,073,926,542 uncompressed bytes / 289 entries**. Its SHA-256 matches GitHub's
advertised digest:
`73f4446c3da7365a1ef481c1c139d666387e682ed2449b7d3b8fb8d65a73ad63`.
Read-only compressed-entry audit verifies all **eight profiles / 32 images**,
184 error records and 24 removals. The matrix asserts ordinary Linux-created
1/4 KiB parents/targets, 128-byte legacy maps or 256-byte checksummed extents,
direct/journal mode, exclusive attributes and actual kernel-created shared
target/peer attribute blocks. The driver only reads these images. Target handles
are closed, controls use O_NOATIME, and timestamps cross legacy resolution.

### Observed Linux behavior and actual saved bytes

All three successful removals per profile (255-byte boundary, exclusive-attribute
directory and shared-attribute directory) update their parent's **mtime and
ctime**, retain parent atime and decrement parent nlink. Each frees one inode
and decrements the correct group's used-directory count. Boundary removal frees
one data block; exclusive removal frees its data and attribute block; shared
removal frees data while the peer retains the shared attribute block at one
reference. The final per-profile ledger is **four blocks / three inodes freed,
used-directory count minus three**. Physical block/inode allocation bits and
every affected group's credits match the global SB counters and Linux statvfs.
Peer whole-inode bytes, data and attribute values remain intact. Parent raw
timestamps agree with kernel/debugfs records; live/freed inode identities,
generation/dtime, namespace, shared references, clean state and replay flags
pass. Raw checksum checks cover **16 SB, 40 BGD, 220 live-inode, 24 freed-inode,
28 attribute-block and 60 directory-block** observations, including repeated
views. No arbitrary-corruption or complete mapping admission is established.

The 23 symbolic error answers agree in every layout:

| Input category | Linux observation |
|---|---|
| Existing 255-byte directory | Successful removal in all eight profiles |
| Ordinary 256-byte component | ENAMETOOLONG (raw Linux 36) |
| Missing/empty name | ENOENT |
| Regular file or symlink, including trailing slash | ENOTDIR |
| Nonempty directory / dotdot, including trailing slash | ENOTEMPTY |
| Dot, including dot through a symlink | EINVAL |
| Missing or regular-file prefix before a long component | Earlier ENOENT / ENOTDIR wins |
| Valid directory, symlink, dot or dotdot prefix before a long component | ENAMETOOLONG |

All refusal loops preserve policy-parent and peer metadata; the policy parent's
whole raw inode matches before and after the names/error phase. These syscall
answers do **not** change existing Rust/C counted-argument validation or the
symbolic C errno table: Linux's empty-name ENOENT is not permission to alter
the counted API's EINVAL priority, and raw Linux numbers are not copied into
the current ABI.

### Retention, remaining CI and next bounded task

Fresh local baseline/post-change units retain the exact **1,196** successful
occurrences and **228** controls, with zero failed/ignored (post-change 1,444
lines / 102,845 bytes). Full lint passes. The downloaded exact-source standalone
CI unit archive matches its advertised SHA-256 and retains the same names and
controls (1,452 / 103,329). The dormant 55-control rmdir audit remains unchanged
and still applies cleanly. Reference source, tasks and budgets are unchanged
by this evidence-only follow-up; only the existing review/handoff advance.

At this checkpoint focused unlink emission, unlink reference and setter policy
jobs also pass. The main focused fixture job and [full run 39644593304](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39644593304)
are still executing; the expected complete-suite baseline is 2,033 retained plus
this reference test, which must be checked against actual completed logs.
No guarded-rmdir implementation, parent/name semantic repair, driver-emitted
image qualification, global output cap/floor/filter, dependency/lock, sibling
or write-gate change. Native synthetic positives do not replace Linux-emitted
ordinary reference evidence, and neither qualifies future inline/indexed/grown-
empty driver emissions, open-target ownership or interruption recovery.

**Next bounded task:** inspect actual remaining focused/full CI outcomes and
investigate any failure without weakening checks. Prepare and actually execute
advancing-clock native rmdir policy controls for successful parent mtime/ctime
and ordinary 255/256-byte names, including refusal metadata and mixed path/name/
generation/volume priorities through all five Rust/C forms across six native
profiles. Preserve the existing 55 outcomes; prove dormant-patch composition
and byte-identical reapplication/rerun, then reverse and verify all active
source/baseline tests restored before implementation. No semantic repair before
its measured red. Stages 4–5 remain not yet quantified; casefold writes and
Windows/SteamOS/card qualification remain blocked.

## Independent Linux rmdir policy oracle added — 2026-10-21

Started from clean fetched `ce2dc95d5e574887a0dbb4e8286a2c4c62c23325`; its
own full run `39635182790` now passes. Runtime policy is current/enforced.
The initial sandbox Git fetch lacked authentication; the supported outside-
sandbox retry succeeded, and actual API branch reads work with the inherited
binding. Subsequent Git reads work with the inherited `gh auth git-credential`
helper selected for that command only. No new secret, environment publication
or persistent credential/config change.

Added `needs_host::rmdir_policy::linux_records_rmdir_policy` and its guest-only
Python observer. It reuses the pinned VM/kernel/oracle helpers and existing
Linux image maker: 1/4 KiB blocks × 128/256-byte legacy/checksummed inodes ×
direct/journal layouts. The observer creates all ordinary directory, attribute,
peer-data, symlink and long-name input through Linux. Driver access is read-only;
there is no driver mutation or constructed disk fixture in this reference.

The test is designed to retain **32 phase images** across **eight profiles**:
input, name/error checks and 255-byte removal, exclusive-attribute directory
removal, then shared-attribute removal with its peer retained. Each phase has
judged non-repairing e2fsck, saved JSON/stderr and six debugfs inode reports.
Read-only driver checks assert actual map/inode/checksum/journal shapes, real
external attribute sharing, inode liveness, SB/BGD/used-directory credits, parent
link counts and peer inode preservation. The guest uses O_NOATIME for control
reads, closes target directory handles, syncs, and crosses whole-second clock
resolution before removals. It observes statvfs, exact parent times/links and
exclusive/shared reclamation, with raw errno symbols for 23 ordinary/mixed
path/dot/long-name cases. Raw Linux numbers do not change the C ABI errno table.
These are assertions to execute, not completed measurement claims yet.

A separate `rmdir-policy-reference` focused job compiles the exact test, requires
floor **one**, preserves its full 3,200-line/200,000-byte initial isolated ceiling
and uploads the named evidence archive. Existing focused jobs and full-CI
workflow are unchanged. The new reference test participates in the complete
suite; expected retention is the 2,033 baseline plus this one test, which still
requires actual completed logs. No global cap/floor/filter change is made.

Local Python syntax checks execute no guest actions; full lint passes, and the
normal native unit tier retains the exact **1,196**-name baseline and **228**
controls. This host cannot run the KVM harness, so fresh kernel/e2fsprogs outcomes
remain pending exact-source CI. The dormant rmdir red patch still applies and
is unchanged; no guard, semantic repair, dependency/lock or sibling change.

**Next bounded task:** inspect the exact focused reference result and verify the
actual downloaded image/report archive. Fix any real fixture/measurement error
before claiming policy equivalence, preserving the source observations versus
independent answers distinction. Then prepare and execute any resulting native
policy reds before guarded implementation. Other focused/full CI outcomes and
complete-suite retention remain required for later family closure. Stages 4–5
remain not yet quantified; casefold writes and Windows/SteamOS/card use remain
unqualified.

## Executed native rmdir red/positive preparation — 2026-10-21

Started from clean fetched `60e25de143f3ad0b273abc5b8424597f07d3ff28` on
`casefold-rw`. Runtime policy is current/enforced; Git/API reads work without
environment changes. That review commit's own [full run 39552501832](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39552501832)
now passes. The preceding fully inspected implementation-source run
`39547402531` remains the complete retained-name/image evidence described below.

### Executed patch and repeatability

Fresh unchanged native baseline: **1,196 passed, zero failed/ignored**, all
**228** guard/operation controls. Saved its complete log before editing. The
new [dormant patch](CASEFOLD-RMDIR-RED.patch) adds only one module registration
and `src/fs/guarded_chmod/tests/rmdir_calls.rs`; no production implementation.
After executing the final source, saved the patch, reversed it, checked that
source matched HEAD, reapplied it and verified both file SHA-256s against the
executed source, then reran. Both final runs contain the identical **55** names
and outcomes: **six pass / 49 fail / zero ignored**, exit **101**. Each complete
log measures **962 lines / 71,082 bytes**, inside the final isolated audit limit
of **1,200 / 100,000**. Active CI/task caps and floors remain unchanged.

Patch SHA-256:
`4cf8a2ac2e55d1595530c980cb3bf0adfb399cede3e81d68ee3dfe022bcd5317`.
Saved local evidence is under ignored `tmp/casefold-rmdir-red`: baseline,
`run1.log`, `run2.log`, exact outcome/source-hash records, `evidence.json`, lint
and restored-unit logs. Those workspace files are not assumed to survive a new
cloud machine; the committed patch and commands reproduce the controls.

```sh
git apply --check docs/CASEFOLD-RMDIR-RED.patch
git apply docs/CASEFOLD-RMDIR-RED.patch
rmdir_audit_rc=0
/workspace/.cloud-setup/run-casefold.sh bash scripts/tier.sh \
  'rmdir dormant native audit' rmdir-red 1200 100000 -- \
  scripts/test.sh --locked --lib fs::guarded_chmod::tests::rmdir_calls:: \
  -- --show-output --test-threads=1 || rmdir_audit_rc=$?
# Expected 101 only after all 55 controls actually execute; inspect the full log.
git apply --reverse docs/CASEFOLD-RMDIR-RED.patch
test "$rmdir_audit_rc" -eq 101
```

As usual, use this chat's documented pinned cloud source/tool setup; the helper
path is workspace-specific. Preliminary test-harness compilation and counted-C-
name construction issues were corrected before these final executions; they
are not driver reds. Foreign-context, malformed-SB and retained-context
publication tests await the actual private rmdir boundary and are not credited
as executed by this public-entry preparation.

### What the measured outcomes establish

The **45 actual public-boundary failures** cover five Rust/C entries under a
held context and generic conversion priority; twenty clean counted-name
refusals plus two complete nine-case path-edge controls; conversion/device/read/
runtime worker reentry in both directions with all seven guarded families;
second-verdict marker effects; Rust/C clean/prior-dirty planning and conversion
unwinds; and complete uncertainty sweeps. Each Rust/C sweep set injects every
**114 observed writes**, **28 flush errors** and **28 Rust-device flush unwinds**,
across 1/4 KiB direct/journal clean/prior-dirty profiles. All **340** injections
execute before aggregate retirement assertions. Writes may change bytes before
returning an error. Tests inspect all eight intended guarded families, flush
and clean close. No C callback is made to unwind across its ABI.

Four further failures are explicitly **constructed future staged-SB ordering
diagnostics**. Synthetic credits without real bitmap/inode frees retain other
state, checksums and outside-SB sentinels but overwrite the dirty marker when a
pre-marker buffer is committed. Current public rmdir marks before planning;
these are not existing public clean-bit defects or valid deletion qualification.

The **six passing controls** preserve volume/FFI/dot/invalid-name priorities and
all twenty prior-dirty refusals, and execute **127 real native removals**: 60
ordinary ext2/ext3/ext4 × 1/4 KiB × five-form × exclusive/shared-attribute cases,
64 ordinary/inline parent/target and literal/DIR_NLINK combinations through Rust
bytes/C parent forms, and three successful parent-symlink/trailing-slash path
cases. Inline construction releases old storage and maintains `system.data`,
features and checksums. Assertions cover namespace, storage/inode bitmaps,
SB/BGD/used-directory counts, parent links, freed generation/dtime/checksums,
shared references and peer inode/data/attributes, dirty/close/reopen state.
These native fixtures do not replace independent Linux layouts, statvfs or
emitted-image validation. Parent timestamp and ordinary long-name policy remain
unmeasured here; no semantic correction is introduced.

### Restored active source and next task

Full lint passes with the final audit active. Reversed the exact patch after
the repeat run, checked active driver/tests/tasks/workflows/pins byte-restored,
and reran the normal native unit tier. Its exact successful-name multiset
retains all **1,196** baseline occurrences and all **228** controls, with zero
failed/ignored (1,444 lines / 102,845 bytes, within unchanged 1,460 / 106,500).
The dormant patch still applies cleanly. Existing whole-suite
baseline is **2,033** passes and one unchanged ignored slow fuzz case. Only this
patch and existing review/progress records change; no guard/semantic repair,
workflow, budget/floor/filter, sibling, dependency/lock or write-gate change.

**Next bounded task:** add and execute independent reference-only Linux rmdir
policy measurement in the existing CI VM, using distinct advancing clocks and
closed reference directory handles. Record successful/refused parent times,
255/256-byte and mixed dot/type/path priorities, inode/block/used-directory
reclamation and exclusive/shared attributes with saved bytes and reports.
Then reproduce any intended semantic corrections as native policy reds before
a reviewed guarded repair. Preserve these executed controls and all baselines;
do not promote/apply the dormant tests as if a repair already existed. No host
oracle/mount, local KVM or forbidden source is needed. Stages 4–5 remain not yet
quantified; casefold writes and Windows/SteamOS/card qualification stay blocked.

## Unlink closure and public-rmdir design audit — 2026-10-21

Started from clean fetched `661ba754839d872e86f680e8cf84a59ddda9bdc7` on
`casefold-rw`. Runtime policy is current/enforced and Git/API plus actual
redirected log/artifact downloads work. No environment configuration changes.

### Completed exact-source full CI and scoped unlink closure

[Full run 39547402531](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39547402531)
now **passes every job and `ci-ok`**. Actual standalone-unit, aarch64, native,
guest and CLI archives match their advertised SHA-256 digests. Exact successful-
name multisets show both whole suites retain every **1,975** preceding successful
occurrence plus **57** native controls and **one** unlink emitted-image oracle:
**2,033 passed, zero failed**, with one unchanged ignored
`deep_walk_byte_flip_fuzz_never_panics`. All **228** guard/operation controls
occur once in **five CI contexts**, all three unit contexts retain all **1,196**
names, and aarch64 retains all **527** image tests. Native trailing checks now
execute and pass all **twenty shell checks and 159 CLI checks**.

The native log is **3,433 lines / 192,301 bytes** within **3,700 / 195,000**;
guest is **3,603 / 201,982** within **3,900 / 203,000**. The measured repairs leave
commands, floors, filters and full output intact. Source, active/dormant tests,
focused workflow, dependency pins and locks match independently tested `cd15806`;
only the recorded budget configuration and review/handoff differ. The existing
complete source review and audited focused Linux evidence remain applicable,
including the sixteen-profile/80-image unlink matrix and retained metadata,
policy and symlink evidence. No repeat source/image review or fresh local test
execution is mislabeled as necessary for this documentation checkpoint.

The standalone unlink-family review is now **complete within the documented
caller-exclusion, closed-reference-handle and healthy-layout contract**. No
additional scoped blocker is found. This does not qualify arbitrary corruption,
aliases, retained open-file deletion ownership, complete mapping/lifecycle/
sealing, every timestamp/path combination, ignored fuzz or recovery; casefold
writes and actual Windows/SteamOS/card use remain blocked.

### Subsequent bounded rmdir design audit

The detailed design is recorded in the existing `CASEFOLD-INDEX-REVIEW.md`.
This is a source/helper/test audit, **not** a new guard, executed red, Linux
reference measurement or emitted-image qualification. It traces Rust string,
byte and parent-inode entries, both C exports, parent resolution/trailing slashes,
dot/dotdot before caller conversion, live generations and final-entry identity,
inline/block emptiness, existing role-dependent checksum checks, extent/indirect
storage and shared attributes, block/inode/used-directory counters, freed inode
generation/dtime, buffered parent links and complete commit/publication effects.

Current entries call marker-writing `refuse_write` before planning; path dispatch
then calls the public parent-inode entry and repeats its verdict. The reviewed
migration acquires one shared nonblocking context before filesystem callbacks,
uses private nested dispatch with both pure verdicts, finishes one owned deletion
buffer before effects, and normalizes its staged SB clean state/checksum without
losing credits. Marker and complete commit/checkpoint/flush/publication remain
under failure-latching effects. Preserve existing error priorities, same-table
and inline-parent splicing, DIR_NLINK unknown count, matched identity and all
storage/attribute/free-counter helpers. Keep the qualified unlink/raw commit
boundaries and other unguarded families unchanged. A future constructed clean-SB
ordering diagnostic is not a current public clean-bit regression.

Concrete proposed red/positive controls cover all five forms, held-context and
callback/worker reentry, Into priority/unwind, clean/prior-dirty refusals, second
verdict, invalid/dot/type/stale/FFI priorities, actual ordinary/inline deletion,
attributes/peer preservation, counters/dirty/close and complete write/flush/
publication failures across 1/4 KiB direct/journal profiles. Source parent edits
do not explicitly stamp mtime/ctime, and ordinary counted names have no 255-byte
verdict. These are not new Linux measurements: independent advancing-clock
reference evidence and executed policy reds precede semantic corrections.

Only the existing review/handoff change. Shared-agent, provenance and whitespace
checks pass. No source/test/task/workflow, cap/floor/filter, sibling/pin/lock or
write-gate change; no host oracle, mount, local VM, forbidden source or actual
Windows/SteamOS/card qualification. Stages 4–5 remain not yet quantified.

**Next bounded task:** prepare and actually execute the reviewed native rmdir
red/positive controls using the existing Device/Clock/C harness. Save a dormant
patch only after actual execution, reapply/rerun with identical source/names and
complete logs, then reverse it and verify active-source restoration and all
**1,196** native baseline passes/**228** controls. Missing fixtures, build
failures and zero matches are not reds. Do not implement rmdir or broaden into
another family in this preparation. Independent Linux policy/emission and
standalone family review remain prerequisites to later closure; preserve the
**2,033** whole-suite baseline and all broader gates.

## Unlink guest full-suite output-budget repair — 2026-10-21

Original exact implementation-source [run 39542365226](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39542365226)
is now complete. Its SHA-256-verified guest archive retains the exact **1,975**
preceding successful occurrences and adds precisely **57 native controls plus
one emitted-image oracle**: **2,033 passed, zero failed**, with only the unchanged
ignored `deep_walk_byte_flip_fuzz_never_panics`. Native has the same complete
retention. A consolidated read-only audit rechecks exact-source artifact digests
and all **228** controls once in each of **five CI contexts**: standalone,
aarch64 and native units, plus native and guest whole suites. Each unit context
retains the complete **1,196**-name multiset. The ignored fuzz case remains a
broader qualification gate, not new unlink evidence.

The actual guest job fails only its output budget, returning **65** after all
tests succeed: **3,601 lines / 201,734 bytes** exceeds **196,500 bytes**. Before
changing the limit, identical saved-log replay through the pinned wrapper
reproduces exit 65. Afterward the same complete bytes pass at **203,000 bytes**,
including the unchanged **1,060-test floor**. The log SHA-256 is
`460976e0bda3b29fee98b0ecb943137e5563c46593b200afe74ec2b997843ff1`.
The preceding guest baseline is **3,544 lines / 195,641 bytes**; its measured
6,093-byte growth supports a 6,500-byte increase and restores **1,266 bytes** of
headroom. The **3,900-line** cap, command, filters and full output stay intact.
Native retains the separately measured **195,000-byte** ceiling from `ae7f863`.

Only the guest byte ceiling/measurement in `chores.yml` and the existing
review/handoff change. No implementation, active/dormant test, workflow, floor,
dependency/lock, sibling or write gate changes. Full lint, the output-budget
contract, shared-agent, aggregate CI-gate, provenance and whitespace checks pass.
The red/green replay is a configuration check against actual saved CI bytes;
it is not fresh guest execution or a green full-CI result. Existing exact-source
focused Linux/image evidence and standalone source review remain applicable.

**Next bounded task:** inspect fresh full CI with both measured limits, retaining
both complete baseline-name multisets and all 228 controls in five contexts.
Native-only repair [run 39546367987](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39546367987)
is still in progress at this checkpoint. Require successful native trailing
twenty shell checks and 159 CLI checks as well as the aggregate gate before
closing the scoped unlink review. Only then perform the public-rmdir
staging/marker/dispatch design audit, before new reds or implementation. Do not
repeat completed source/image reviews because configuration CI is pending.
Stages 4–5 remain not yet quantified; casefold writes and Windows/SteamOS/card
qualification remain blocked.

## Unlink native full-suite output-budget repair — 2026-10-20

Exact implementation-source [full run 39542365226](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39542365226)
has completed its native job. The actual SHA-256-verified tier archive retains
all **1,975** preceding successful occurrences and adds precisely the **57**
native controls plus the emitted-image oracle: **2,033 passed, zero failed**.
The sole ignored case is the unchanged `deep_walk_byte_flip_fuzz_never_panics`;
it remains a broader qualification gate. All **228** controls occur once in the
whole suite, and native debug units retain the complete **1,196**-name multiset.
Together with the previously verified standalone/aarch64 units, four of the five
required CI contexts are now inspected; the guest context is still pending.

The complete native log measures **3,433 lines / 192,299 bytes**. Its actual job
fails only the **189,000-byte** ceiling, with exit **65** after successful tests.
The same saved bytes reproduce exit 65 through the pinned tier wrapper before
the change and pass byte-identically at **195,000 bytes**, including the unchanged
**1,060-test floor**. Its SHA-256 is
`4ac292636d54f7779cc8bfb8e2f44b0c313541d28e1e8ee7154d60200dad78c1`.
Compared with the preceding 186,571-byte baseline, the 5,728-byte growth supports
a 6,000-byte adjustment, restoring 2,701 bytes of headroom. The line ceiling,
test command, filtering and full output remain unchanged. Guest limits are not
raised from a prediction while that job is unfinished.

Only `chores.yml`'s native byte ceiling/measurement and the existing review/handoff
change. No implementation, active/dormant test, workflow, floor, dependency/lock,
sibling or write gate changes. The budget-contract shell check, aggregate CI-gate
check, shared-agent check, clean-room provenance check and full lint pass;
whitespace checks pass. This replay verifies the configuration against an actual saved CI log; it
is not a fresh Linux execution or full-CI pass. Existing exact-source focused
Linux/image evidence and source review remain applicable to unchanged code.

**Next bounded task:** inspect the original guest job's completed log and any
actual failure, then the fresh repair-source full CI. Require both complete
baseline-name multisets, all 228 controls in five contexts, and the native twenty
shell/159 CLI checks after the whole-suite wrapper before scoped unlink closure.
The previous failed native wrapper prevented those trailing checks. Only after
closure perform the public-rmdir staging/marker/dispatch design audit. Stages
4–5 remain not yet quantified; casefold writes and Windows/SteamOS/card
qualification remain blocked.

## Standalone unlink source/saved-evidence review — 2026-10-20

Reviewed clean fetched `3998daaaf18e74efaec9ac6cfcbc17a5b9951a37`, whose driver,
test, tier and workflow bytes still match independently executed `cd15806`.
Runtime policy reports current/enforced; Git fetch, authenticated API and actual
redirected downloads work. No environment or implementation changes are needed
for this review. The detailed source decision is recorded in the existing
`CASEFOLD-INDEX-REVIEW.md`.

The source walk found **no new blocker within the reviewed caller-exclusion,
closed-reference-handle and healthy-layout scope**. It traces all Rust/C entries,
pure verdict/argument/generation ordering, retained entry and buffered parent
splicing, timestamp/length repairs, both hard-link plans and complete existing
commit/publication/failure/close boundaries. It preserves the distinction between
healthy emitted-byte evidence and synthetic future-SB diagnostics. Neither source
review nor the independent matrix extends handle/session lifetimes, arbitrary
corruption/alias admission, mapping/ownership/sealing, deeper index writes or
interrupted-write recovery. Casefold writes remain refused.

### Exact focused CI and retained evidence

[Focused run 39542365213](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39542365213)
now **passes every job** at exact implementation source `cd15806`. Actual main,
policy and symlink archives are downloaded and checked against their advertised
SHA-256 digests; compressed main-image reads also verify ZIP CRCs. All **15**
main exact test logs show one passed / zero failed / zero ignored. The actual
retained bytes and independent reports pass all **16 setter layouts**, **eight
fresh first-attribute cases** and **84 retained metadata profiles** (four private
chmod, eight public chmod, sixteen chown, 24 utimens, sixteen flags and sixteen
removexattr). Checks cover applicable SB/BGD/inode/attribute checksums, exact
metadata/counters/values, shared references, raw timestamps and Linux/debugfs
agreement; existing mixed zero-extra distinctions remain in the audit.

The separate policy archive passes **32 ordinary observations** and all **16
byte-identical Rust/C policy emissions**. Its four constructed duplicate cases
remain diagnostics rather than healthy-image qualification. All **five** exact
symlink tests and saved boundary/tree/casefold-chain images pass their read-only
checks. The new unlink archive still matches its advertised digest, and its
16-profile/80-image audit and 80 judged e2fsck/48 mount-readback evidence remain
verified from the preceding checkpoint. That archive is not mistaken for a
complete ownership, lifecycle or power-loss proof.

### Available full-CI evidence and remaining gate

The exact-source standalone CI unit artifact was already SHA-256-verified and
retains all **1,196** successful occurrences, including all **228** controls
(1,452 lines / 103,329 bytes). The freshly downloaded verified aarch64 artifact
retains the same unit-name multiset and controls (1,446 / 102,892), plus every
**527** preceding image-tier occurrence. Those jobs pass their unchanged floors
and measured unit/image limits. The full native/guest jobs in [run
39542365226](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39542365226)
remain in progress at this checkpoint. No failing test or measured whole-suite
budget failure is asserted from an unfinished job, and caps are not raised in
advance. The existing ignored slow fuzz case remains a broader qualification gate.

**Next bounded task:** inspect the completed full logs and exact successful-name
multisets against the 1,975-pass preceding baseline plus 57 native controls and
one emitted-image oracle; require all **228** controls once in each of five CI
contexts and complete trailing native checks. Investigate actual failures without
weakening checks, then record the scoped unlink-family closure if justified.
Only then perform the next public-rmdir staging/marker/dispatch design audit,
before preparing reds or implementing another migration. Do not rerun completed
source/image reviews or reapply the historical dormant patches onto promoted
source merely because a whole-suite job remains pending. Stages 4–5 remain not
yet quantified and no casefold, Windows, SteamOS or card-write readiness is claimed.

Retained review evidence/scripts and CI metadata are ignored under
`tmp/casefold-unlink-review`; the existing handoff/review remain authoritative.
No host oracle/mount, local VM, forbidden implementation source or sibling change
is used. No new implementation test is needed for this documentation/evidence
checkpoint; exact passed source and byte provenance are verified above.

## Guarded public unlink implementation — 2026-10-20

Started at fetched `223adb3b4811cc728ad9b452fc43d66aef15d014`. Runtime policy
is current/enforced; existing Git/API/artifact access works without environment
changes. Earlier exact full run `39534096164` and focused run `39534096165` now
both pass. Verified native and guest archives retain all preceding 1,974 passes
and add only the reference oracle (1,975 passes, zero failed, one unchanged
ignored fuzz case); all 171 controls remain. Native log: 3,378 lines / 186,571
bytes. Guest/unit/aarch64 retention was already inspected in the prior checkpoint.

Fresh native baseline passes 1,139 tests. Before any implementation the saved
combined **7 passed / 43 failed / zero ignored** outcome is actually reproduced.
Four additional write/flush failure-retirement controls execute red with the
specific missing shared mount-failure latch; a mixed-invalid-name/parent/volume
priority control passes. Those executed logs are retained under ignored
`tmp/casefold-unlink-repair`.

Public Rust string/byte/parent-inode and C dispatch now acquire the existing
mount-bound shared context before device/runtime callbacks or caller conversion.
Path-to-parent dispatch reuses that context and retains its second pure write
verdict; trailing-slash and counted-name validation priorities are retained.
The private planner prepares the complete buffer before marker/effects. It keeps
stored spelling/identity, surviving/final link accounting, storage collectors,
shared attribute behavior, checksums and generation/deletion-time behavior.
Parent timestamps use the current buffered parent inode, preserving prior inline
edits and other inodes sharing its table block; surviving targets update ctime
while retaining atime/mtime. Ordinary overlong names are rejected after parent
liveness checks, preserving mixed invalid-name/parent/volume ordering. The C ABI's
existing symbolic errno table is unchanged.

The unlink-only boundary checks context identity, staged primary-SB full shape
and magic before effects; it clears VALID_FS and restamps only its checksum,
preserving planned credits, other state and outside-SB bytes. Marker-once and
complete commit/checkpoint/flush/publication run as failure-latching effects.
Shared raw commit and deletion/allocation helpers are unchanged. This is the
existing caller-exclusion contract, not whole-API serialization, ownership or
transaction admission, broader mapping support, a sealed plan or crash recovery.

Promotion preserves 27 original contract controls and all 22 policy controls.
Only the historical dormant 256-byte ENOENT behavior-recording test is replaced
by independently grounded long-name expectations; the saved patch itself stays
byte-identical. The four constructed SB diagnostics now call the actual unlink
boundary and keep their state/counter/checksum/sentinel assertions. Eight added
controls cover four write/flush sweeps, mixed-name priorities, planning/every-
flush unwinds, foreign/malformed buffers and post-durability publication panic
with a retained context. All **57** pass: **632 lines / 42,737 bytes**. The write
sweeps each inject **176** observed callback failures; flush sweeps each inject
**52**, across Rust/C, 1/4 KiB, direct/journal, surviving/final and clean/prior-dirty
fixtures. All seven guarded families, flush and clean close refuse after failure.
Full lint passes with final source, including compilation of the new VM test.

The full native unit tier executes **1,196 passed / zero failed / zero ignored**,
retaining the exact 1,139-name baseline multiset and adding only the 57 controls.
Its actual **1,444-line / 102,845-byte** log fails the old 1,400 / 100,000 output
ceiling (exit 65), not a test. The identical log reproduces that failure and
passes 1,460 / 106,500; the measured unit-only repair retains similar headroom,
full output, command/selection and floor 440. Fresh repaired-unit execution
passes all 1,196 with all 228 controls and baseline retention: **1,443 lines /
102,785 bytes**. The actual SHA-256-verified exact-source CI unit artifact also
retains all 1,196 occurrences and 228 controls, with zero failed/ignored:
**1,452 lines / 103,329 bytes**. Whole-suite caps remain unchanged pending actual
measurements.

A separate `unlink-emission` focused job adds one exact oracle with floor one,
retaining its complete log and images. Sixteen profiles span Rust/C × 1/4 KiB ×
128/256-byte inode × direct/journal. Linux creates real payloads and exclusive/
shared external attributes with actual hard links. The test asserts real sharing,
advances the runtime between removals, saves input plus dirty/closed branch images,
checks preservation and exact reclamation, and requires independent judged
non-repairing e2fsck and cold Linux timestamp/name/data/attribute readback. Both
C entries clear seeded errno.

**Executed independent evidence:** implementation source
`cd15806199a698e3e1a78f6bfdee58a62802f36a` passes `unlink-emission` in [focused
run 39542365213](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39542365213).
The exact test executes **one passed / zero failed / zero ignored** across all
16 profiles; the floor passes. The complete log measures **285 lines / 28,950
bytes**, within the new 3,200 / 200,000 ceiling. It records **16 mke2fs, 48 kernel
mount, 80 judged e2fsck and 32 debugfs invocations**. The completed job log and
actual **2,954,758-byte** archive download through the existing authenticated API
and separate unauthenticated redirect; the advertised SHA-256 digest matches.
The **2,684,412,002-byte uncompressed** archive stays compressed to preserve local
disk space. A read-only audit reads every retained image and relevant report
through ZIP CRC verification without extracting or mounting images on the host.

All **80 images** (16 input plus surviving/final dirty and closed snapshots) pass
the raw-byte/report audit:

- All 16 inputs have real Linux-created exclusive and shared external attributes
  with reference counts one and two. Both targets have two actual names; the
  sharing peer has its own inode and payload. Legacy and checksummed geometries,
  journal features and independent clean-input judgments match their profiles.
- Non-final deletion leaves links one, advances target ctime and parent mtime/
  ctime to the first distinct runtime value, and preserves atime/mtime, complete
  payloads/attributes, identity/generation, data/map/attribute allocation and all
  free counts. Parent and target inodes sharing a table block remain intact.
- Final deletion advances the parent times again and credits exactly **two
  inodes** plus **37 legacy 1-KiB / 35 checksummed 1-KiB / 11 4-KiB blocks** for
  the two targets. The complete direct/extent/legacy-indirect target maps and
  exclusive attribute are free; the shared attribute remains allocated with
  reference count one. The sharing peer's entire raw inode, payload and value
  remain unchanged. Freed target mode/link bits, generation, deletion time and
  applicable checksums match the emitted contract.
- All superblock/descriptor counts agree with allocation bits and Linux statvfs;
  checksums and independent parent debugfs timestamps agree. Dirty snapshots
  retain VALID_FS clear, closed snapshots restore the original clean state,
  other state is preserved, and no snapshot has the pending journal-replay flag.

The independent Linux readback checks exact parent/surviving timestamps, names,
link counts, complete payloads, attributes and peer identity; each phase also
requires judged non-repairing e2fsck cleanliness (16 input, 32 closed-emission
and 32 post-Linux judgments). Raw audit totals include 40 SB,
100 descriptor, 200 inode and 88 external-attribute checksum checks. This is
healthy-profile emitted-byte qualification, not crash-recovery or alias/ownership
admission proof. No host oracle/mount, local KVM or forbidden source was used.
Detailed hashes, complete logs, CI metadata, compressed images and read-only audit
script/results are ignored under `tmp/casefold-unlink-repair`.

The reference-only and separate policy/symlink focused jobs also pass. Main
focused fixtures and [full CI 39542365226](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39542365226)
remain pending at this checkpoint; complete-suite retention and re-audit of the
other saved metadata families are not claimed from the new unlink job.

**Next bounded task:** inspect this exact source's completed full/focused CI and
all baseline/control multisets, investigate actual failures without dropping
checks or preemptively raising caps, then perform the standalone unlink-family
implementation/evidence review before closure or another migration. Keep all
caller-exclusion/raw-handle/lifecycle/ignored-fuzz, ownership/mapping/seal,
index-growth and recovery gates; no Windows/SteamOS/card qualification. Stages
4–5 remain not yet quantified; casefold writes remain blocked.

## Executed native unlink policy red preparation — 2026-10-20

Started clean at fetched `44b30bb3b1f50f33ef20766eb428ecd531534c5d`, with
current/enforced runtime policy and working authenticated Git/API/downloads.
Fresh native baseline: **1,139 passed / zero failed / zero ignored**, all
**171** controls; **1,386 lines / 96,782 bytes**. No environment changes.

### Completed CI evidence inspection

Reference source `30c15f4` now passes every job in [focused Linux run
39534096165](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39534096165).
The actual SHA-256-verified main archive remains compressed (9,663,048,471
uncompressed bytes); all **15 exact test logs** and all **15 one-test floors**
are inspected: each has one passed / zero failed / zero ignored. Its saved
main-matrix images are not re-audited here. The preceding checkpoint already
verified all 24 new unlink reference images. Completed main/policy-symlink/guest
job logs download successfully with credentials kept off the storage redirect.

[Exact full CI 39534096164](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39534096164)
still has its native x86_64 job pending at preparation. Actual SHA-256-verified
guest, unit and aarch64 artifacts are inspected. The whole guest passing-name
multiset retains all **1,974** preceding occurrences and adds only the unlink
reference oracle: **1,975 passed / zero failed / one unchanged ignored slow fuzz
test** (`deep_walk_byte_flip_fuzz_never_panics`). All **171** controls occur once.
Its **3,544 lines / 195,641 bytes** fit unchanged 3,900 / 196,500 limits. Both
completed unit contexts retain every **1,139** occurrence and all **171** controls
with zero failed/ignored. These are completed-job results, not a full-CI success
claim. No budget, floor or filter repair is warranted from those measurements.

### New independently grounded native reds

`docs/CASEFOLD-UNLINK-POLICY-RED.patch` is a separate **dormant** patch adding
`tests::policy_unlink_calls`. It is self-contained and composes with the original
`CASEFOLD-UNLINK-RED.patch` in either order. The original patch remains
byte-identical. Native profiles are ext2-style direct, ext3-style journaled and
checksummed ext4-style direct, each at 1/4 KiB: **six**, distinct from the eight
Linux reference profiles. No checksummed-journal native profile is claimed.

The new tests exercise Rust string/byte/parent-inode and C path/parent-inode
entries. Clock values advance from setup to the first unlink and again to final
unlink; seeded parent/target atime/mtime differ from ctime. Every red collects
all six profile results before failing. Timestamp controls check successful
deletion, identity/generation, payload/attributes, preserved timestamps, allocated-
sector/free counts and reopen before the policy assertion; long-name controls
check the unchanged parent/target inode bytes and free counts.

The actual isolated execution is **two passed / 20 failed / zero ignored**, exit
101 (**190 lines / 21,899 bytes**). All **120 profile cases** reach their intended
policy assertion, without compile/fixture/zero-test failures:

- Five first-unlink parent tests observe old mtime/ctime instead of the advanced
  clock; five final-unlink parent tests also retain old times after a second
  clock advance. Parent atime stays unchanged.
- Five surviving-hard-link tests observe unchanged ctime instead of the first
  advanced clock, while links 2→1, identity/generation, atime/mtime, exact data,
  attributes, allocated-sector and free counts retain their required behavior.
- Five 256-byte-name tests observe ENOENT instead of symbolic ENAMETOOLONG.
  The driver C ABI declares macOS errno values: native expectations intentionally
  use its existing **ENAMETOOLONG (63)**, while the independent Linux syscall
  report is **36**. This task does not change that global ABI table or substitute
  raw Linux errno numbers into native C assertions.

Both positives pass: **30** 255-byte removals across all five entries and six
profiles, including clearing seeded C errno, and **60** missing/directory
refusals preserving complete parent/target inode bytes and timestamps. Those
positive refusal controls do not claim the absent pre-marker guard is fixed.

The combined audit actually executes twice, with reversal, opposite-order
reapplication and exact source-byte comparison between runs. Both retain the
same **50 names: seven passed / 43 failed / zero ignored**, exit 101. First log:
**450 lines / 41,930 bytes**; rerun: **451 / 41,990**. Both fit an isolated
1,200-line / 100,000-byte audit ceiling; no existing tier cap/floor/filter changes.
All 28 original outcomes are identical to the earlier saved run: 19 public
guard/refusal/unwind failures, four constructed future SB-order diagnostics,
five positives. Full lint passes with both final audit modules active.

Reproduce both audits from the restored tree:

```sh
git apply docs/CASEFOLD-UNLINK-RED.patch
git apply docs/CASEFOLD-UNLINK-POLICY-RED.patch
/workspace/.cloud-setup/run-casefold.sh scripts/tier.sh unlink:combined-red unlink-combined-red 1200 100000 -- scripts/test.sh --locked --lib unlink_calls:: -- --show-output --test-threads=2
# The current source must fail with the 7/43 outcome, not an empty selection.
git apply -R docs/CASEFOLD-UNLINK-POLICY-RED.patch
git apply -R docs/CASEFOLD-UNLINK-RED.patch
```

Final reversal restores active source byte-for-byte and removes both temporary
modules; the old dormant patch is also byte-identical. Restored native units
pass **1,139 / zero failed / zero ignored**, retaining the exact baseline
passing-name multiset and all **171** controls: **1,387 lines / 96,842 bytes**,
within unchanged unit limits. Detailed hashes, complete logs, outcome maps,
CI archives/metadata and audit results are ignored under
`tmp/casefold-unlink-policy-red`. No production, workflow, pin/lock, sibling,
write gate or active test changes are included. No new Linux/driver-emitted
unlink qualification or family closure is claimed from these native reds.

### Promotion contract and next bounded task

There is one explicit historical contradiction to resolve during implementation:
`oversized_counted_name_retains_existing_block_directory_error` in the original
dormant patch deliberately records the old ENOENT behavior, not a Linux verdict.
It remains unchanged and passes in these audits. The new independently grounded
long-name reds require ENAMETOOLONG. When promoting the controls, retain the old
patch as historical evidence, explicitly supersede that **one behavior-recording
test**, and retain the other **27 original controls** plus all **22 new policy
controls** without weakening their assertions. This replaces no passing active
baseline test and does not relax entry/volume/FFI priority or guard requirements.

**Next bounded task:** inspect the remaining exact-source full-CI outcome, then
implement only the reviewed public-unlink shared-context/private-dispatch and
owned-buffer boundary, with parent mtime/ctime, surviving ctime and long-name
policy repairs against the executed reds. Preserve counted empty/slash/NUL,
generation/trailing-slash/volume/FFI priorities, both write verdicts, planned free
credits, checksums, retained entries, attributes and failure retirement. Keep
the separate C ABI errno mapping unchanged. Add independent Rust/C emitted-image
Linux coverage before family closure; casefold writes remain blocked. Stages
4–5 are not yet quantified; Windows/SteamOS/card qualification is not reached.

## Executed reference-only Linux unlink policy checkpoint — 2026-10-20

Started at fetched `85e7a87708b4c724fb667e029bbe6c0e39d9080e` with a clean
checkout, current/enforced runtime policy, working Git/API access and reported
push authorization. No environment configuration or sibling changes. Fresh
native baseline: **1,139 passed / zero failed / zero ignored**, all **171**
existing controls; **1,387 lines / 96,842 bytes** within unchanged unit caps.

The new `needs_host::unlink_policy::linux_records_unlink_policy` oracle uses
only `mke2fs` and real Linux mounts through the existing guest helpers to create
or change images. A read-only driver device checks block/inode sizes, checksum
and journal features, hard-link identity/generation and surviving ctime. All
eight 1/4 KiB × 128/256-byte inode × direct/journal profiles retain three
unmounted phase images, JSON observations and debugfs parent/target reports;
every phase requires a judged non-repairing e2fsck-clean verdict.

The guest test separates failed operations from successful unlink, waits 1.1
seconds before each timestamp-sensitive deletion (including legacy inodes),
and closes every regular-file descriptor before final reclamation. It checks
that non-final unlink preserves data/attributes and free counts, and final
unlink credits one inode and exactly the former target's allocated blocks.
Five error observations cover 256-byte names, missing/empty names, directories
and a regular file with trailing slash; a separate 255-byte name must succeed.
Embedded NUL is intentionally not described as a kernel errno: Python rejects
it before the syscall, and the driver's counted-entry validation is a separate
API contract. No counted-name semantic answer is credited before actual CI.

The focused workflow adds its own reference job, floor one and retained artifact.
Its initial ceiling reuses the existing policy tier's 3,200 lines / 200,000 bytes;
all pre-existing commands/caps/floors/filters remain unchanged. Post-change native
units pass **1,139 / zero failed / zero ignored**, retaining the exact baseline
passing-name multiset and all 171 controls; **1,387 lines / 96,842 bytes**.
Full lint, frozen Unicode, shared-agent/family/CI-gate, provenance, formatting
and whitespace checks pass with the new files tracked. Guest Python syntax is
checked without executing it on the host.

**Executed independent evidence:** source `30c15f407e0a359ff7fea4439d57b7ab2ac4c77b`
passes `unlink-policy-reference` in [focused run
39534096165](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39534096165).
The exact test executes **one passed / zero failed / zero ignored** across all
**eight** profiles. Complete log: **213 lines / 17,242 bytes**, within the initial
ceiling; the floor passes. The saved log confirms **eight mke2fs, 24 kernel mount,
24 judged e2fsck and 48 debugfs invocations**. The completed job log and actual
915,404-byte archive download successfully through the authenticated API and
unauthenticated storage redirect; its advertised SHA-256 digest and ZIP CRCs match.

A read-only audit of all **24 saved images**, JSON and independent debugfs reports
checks published-format superblock/descriptor/inode/attribute checksums, inode
identity/generation/link counts, raw parent/target timestamps, exact payload and
attributes, direct/extent/legacy-indirect mappings, inode/block allocation bits,
and SB/BGD/statvfs free counts. It finds:

- Both successful deletions advance parent **mtime and ctime**, preserving atime.
- Removing the first hard link changes links **2 → 1** and advances surviving
  target **ctime**, preserving atime, mtime, payload, attributes, identity and
  generation. No target inode, data/map/attribute blocks or free counts change.
- After all regular-file descriptors close, final unlink credits **one inode**
  and exactly **19 1-KiB legacy / 18 1-KiB extent / six 4-KiB blocks** including
  data, any indirect map and the exclusive external attribute. All former target
  blocks and its inode bit are free. Surviving-to-final raw counts agree with
  Linux statvfs; every image has a clean state and no pending journal replay.
- All **40** error observations agree: 256-byte final name → **ENAMETOOLONG (36)**;
  missing/empty → **ENOENT (2)**; directory → **EISDIR (21)**; trailing slash on
  a regular file → **ENOTDIR (20)**. Parent/target snapshots stay unchanged for
  these failures. All **eight 255-byte names** are removed successfully.

The driver's previously saved ordinary-block 256-byte ENOENT control describes
its current behavior and now differs from the independent Linux observation.
The prior source audit also found no explicit parent/surviving ctime update;
prepare actual advancing-clock native reds before repairing those semantics.
The reference is a Linux path-component/unlinkat observation, not permission to
replace the driver's separate empty/slash/NUL counted-entry validation contract.

Ignored retained evidence is in `tmp/casefold-unlink-reference` (downloaded
archive/metadata, completed job log, raw audit script/results and native unit log).
[Full CI 39534096164](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39534096164)
and the main focused fixture job remain pending at this evidence checkpoint.
The separate same-value policy/symlink job now passes, but its saved bytes are
not re-audited here. Neither complete-suite retention nor main-matrix
requalification is claimed from the new reference job. The previously verified whole-suite baseline
remains 1,974 passes plus one existing ignored fuzz case. This is reference work only:
the saved native red patch stays dormant, production unlink is unchanged, and
driver-emitted unlink compatibility, guard migration and complete-suite retention
are separate prerequisites. No host oracle/mount or forbidden source is used.

**Next bounded task:** first inspect this source's exact full CI and other focused
jobs, investigating actual failures without dropping tests or preemptively
changing budgets. Then prepare and execute dormant native advancing-clock
parent/surviving-inode timestamp and overlong-name policy reds from these Linux
answers, preserving all 28 existing unlink controls. Save/reapply/rerun them and
byte-restore active source before implementing semantic/guard changes. Actual
emitted-byte Linux qualification and complete-suite retention remain prerequisites
to later family closure. Stages 4–5 remain not yet quantified; no casefold,
Windows, SteamOS or real-card write qualification.

## Executed dormant native unlink red preparation — 2026-10-20

Started clean at fetched `7ef3929cba92a6f11d610ed6c622953448955368`. The runtime
reports current/enforced network policy and Git reads work; no environment
configuration changes. The native baseline first passes **1,139** tests under
the pinned tool/source-view setup. Existing whole-suite evidence remains the
previously verified 1,974-pass full baseline, not fresh Linux execution here.

The audited next controls were added temporarily under cfg(test), compiled and
actually executed, then saved as `docs/CASEFOLD-UNLINK-RED.patch`. The patch was
reversed and source hashes checked, reapplied with exact matching source bytes,
actually rerun, then reversed again. Both final runs report **five passed / 23
failed / zero ignored**, exit 101, with identical outcomes for all **28** names.
Each complete log is **281 lines / 20,608 bytes** inside a temporary 900-line /
70,000-byte audit ceiling. No existing output budget/floor/filter changes.

The actual red findings are:

- Five Rust string/byte/parent-inode and C path/parent-inode entries bypass held
  shared context and reach device effects.
- Ten ordinary clean-mount Rust/C refusals return the expected missing-path,
  invalid-name, stale-parent, directory or trailing-slash error, but each writes
  and flushes the marker once. The ten prior-dirty variants pass without effects.
- Two worker controls exercise both directions with all six existing metadata
  families; no shared-context exclusion is observed in either direction. Generic
  parent conversion and every selected metadata callback actually execute.
- A second-verdict probe revokes writability at exactly the path edge's second
  policy query. ReadOnly is preserved, but a marker write/flush already happened.
- C catches a Rust deletion-time planning panic as EIO without latching mount
  failure. This does not unwind through a foreign C callback.
- Four 1/4 KiB direct/journal synthetic SB-credit plans preserve credits, other
  state, checksum and applicable outside-SB sentinels but restore the clean bit
  after marker-plus-raw-commit. They diagnose the future family boundary only;
  no actual inode/bitmap deletion is represented, and current public unlink
  still marks before planning.

Five controls pass. The real surviving/final-link test executes **32 successful
unlink calls across sixteen combinations** of block size, direct/journal, Rust/C
and shared/exclusive attributes. It checks actual inode/block bitmap changes,
SB/BGD free counts and unchanged directory counts, payload/peer preservation,
references/attributes/checksums, link/generation/deletion time, error clearing,
dirty/closed state and reopen. These are native self-consistency observations,
not Linux qualification. Other passing controls preserve prior-dirty bytes/status,
volume/FFI priorities, name-before-Into behavior and the current ordinary-block
256-byte counted-name NotFound/C ENOENT result. That last result is explicitly
not a Linux compatibility answer.

The initial authoring attempts' compile constant and unsupported flag/pwrite
setup errors are excluded from the final evidence. Corrected fixtures use a
legal NODUMP change while retaining EXTENTS and the supported legacy content
replacement. All final profile fixtures reach the intended assertions. Full
`chore lint` passes with the final audit controls active. The detailed outcome,
scope and reproduction command are in the existing `CASEFOLD-INDEX-REVIEW.md`.
Ignored exact logs/results/source hashes are in `tmp/casefold-unlink-red`.

After final reversal, native units pass **1,139 / zero failed / zero ignored**,
retaining every pre-audit successful occurrence and all **171** existing controls
with no additions or losses. The complete restored log is **1,387 lines / 96,842
bytes**, within unchanged unit limits. Active source is restored byte-for-byte,
including removal of the temporary test module; production, active tests, workflows,
caps/floors/filters, dependency
pins/locks and siblings are unchanged. The committed patch is dormant. No
independent Linux/reference/emitted-image run, host oracle/mount, local VM,
forbidden implementation source or Windows/SteamOS/card test was used. All
existing qualification gates and casefold write refusal remain.

**Next bounded task:** add and execute independent reference-only Linux unlink
policy observations for parent/surviving-inode timestamps and counted-name errors,
using coherent legacy/checksummed 1/4 KiB direct/journal profiles, distinct times,
closed reference file handles and saved reports/images. Keep driver emission and
guard implementation separate in that reference checkpoint. Then implement the
reviewed shared-context/owned-buffer boundary against the saved red controls,
retaining their assertions and independently qualifying actual emitted bytes.
Demonstrate any semantic mismatch red before a repair. Stages 4–5 remain not yet
quantified; casefold writes remain blocked.

## Public-unlink staging, marker and dispatch design audit — 2026-10-20

Started from clean, fetched `c20dd1b5bbd0800fa97a955e4a38a5991e8771cd` on
`casefold-rw`. The runtime policy is current/enforced and actual Git/API reads
work. No environment configuration changes. Driver, tests and tasks remain
byte-unchanged from verified full source `2c6ece6`: both complete suites retain
1,974 passes, all three unit contexts retain 1,139 and all 171 controls execute
across five contexts. The documentation HEAD's own CI is pending at audit start;
the earlier full results are not relabeled as fresh execution at this checkpoint.

**Completed:** a source/call-path/helper and existing-test design audit in
`docs/CASEFOLD-INDEX-REVIEW.md`. This is not a guard implementation, executed red
reproduction or new Linux/image qualification. Main conclusions:

- Both public filesystem entry paths call marker-writing `refuse_write` before
  validation/planning. A normal path call then enters public `apply_unlink_at`,
  repeating the write verdict. A guard at both public levels would recurse.
- The surviving-hard-link branch removes the matched entry and stages only a
  link-count/checksum edit. The final-link branch also frees data/mapping nodes,
  inode slot and exclusive attributes (or lowers a shared reference), stages all
  bitmap/BGD/SB credits/checksums and a generation-preserving freed inode/dtime.
  Both already own one buffer and have separate terminal raw commit sites.
- Keep stored-spelling/expected-inode rechecks, inline-parent buffer reads and
  same-table inode splicing. Extent release includes external nodes and size-zero
  preallocation; indirect release excludes inline/symlink/device payload bytes.
  These existing helpers are not ownership/admission proof.
- The reviewed boundary acquires the same shared nonblocking mount context
  before filesystem callbacks or generic parent conversion; C foreign-memory
  checks stay at their current boundary. Private nested helpers retain pure
  volume verdicts, including the path edge's second policy check. Preserve
  name-before-Into, parent generation/type, trailing-slash, byte/symlink, errno
  and error-priority behavior. Do not silently add the creation name-length rule.
- Finish one owned plan before any marker/write/flush/publication. An unlink-
  specific boundary checks context and staged SB shape/magic, clears only the
  clean bit and restamps checksum while preserving planned credits, features and
  outside-SB bytes. Guard the marker plus complete commit/checkpoint/flush and
  allocation publication; retain uncertainty/unwind/poison retirement. Keep
  inode-only plans SB-free and leave raw commit/other families unchanged.
- Concrete proposed controls cover all Rust/C entries under a held context,
  worker/Into callbacks, clean/prior-dirty refusals, policy and FFI priorities,
  caught Rust planning unwind, four constructed 1/4 KiB direct/journal SB-order
  diagnostics and real surviving/final-link positives. None was run here; a
  future-boundary constructed clean-bit hazard is not a current public defect.
- Existing C errno/liveness, hard-link, shared-attribute, deep-extent/indirect,
  inline/indexed-directory, byte-name and symlink oracles must remain. New exact
  independent Linux profiles need actual input-shape assertions, saved phase
  images, namespace/identity/data/attribute/link/time/bitmap/refcount/counter/
  checksum reports and judged e2fsck plus cold Linux readback. The old crash
  sweep's remount assertion is not a new complete recovery proof.

**Open semantic prerequisite:** the source's surviving-link branch does not
explicitly stamp target ctime and parent entry removal does not explicitly stamp
parent mtime/ctime. Final deletion stamps dtime. These are source observations,
not newly measured Linux answers. Record independent Linux behavior with distinct
advancing timestamps and no retained open reference files before claiming
equivalence; demonstrate any intended semantic repair red before fixing it.

This checkpoint changes only existing review/progress records; no active source,
test, workflow, output budget/floor/filter, dependency, sibling or lock changes.
No new native or Linux execution is credited. No host oracle/mount, local VM,
forbidden implementation source or real card/Windows/SteamOS test was used.
Caller exclusion, mixed zero-extra, ignored fuzz, insertion aliases and complete
ownership/mapping/lifecycle/seal/admission/index-growth/recovery gates remain.
Stages 4–5 remain not yet quantified; casefold writes remain blocked.

**Next bounded task:** prepare and actually execute the native unlink red/positive
controls identified in the review using the existing Device/Clock/C harness.
Save a dormant `docs/CASEFOLD-UNLINK-RED.patch` only after execution, reapply/rerun
it with exact names/counts and full measured logs, then reverse it and verify
active source is byte-restored. Missing fixtures/build failures/zero matches are
not reds. Preserve all 1,139 unit and 1,974 whole-suite baseline occurrences and
171 controls. Do not implement unlink or broaden into other families in that
preparation; independent timestamp/reference/emission qualification still gates
the later migrated family's review.

## Public setter review closure and successful budget-repair CI — 2026-10-20

Started from clean, fetched `2c6ece6f7789f5ae138d2773147c55b05810936c` on
`casefold-rw`. The runtime reports current/enforced policy and actual Git/API/
redirected artifact access works. No environment configuration changes.

[Full run 39505996964](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39505996964)
now passes every job and the aggregate `ci-ok`. Downloaded standalone-unit,
aarch64, native and guest tier archives all match GitHub's advertised SHA-256.
Complete successful-name Counters show both whole suites retain all **1,974**
preceding occurrences without additions or losses: **1,974 passed / zero failed /
one unchanged ignored** `deep_walk_byte_flip_fuzz_never_panics`. All **171**
guard/operation/parser/fixture controls execute once in each of five contexts:
native and guest whole suites, standalone unit, native unit and aarch64 unit.
Each unit context retains all **1,139** preceding occurrences.

The fresh native complete log is **3,377 lines / 186,486 bytes**, within the
measured repair's unchanged line cap and 189,000-byte cap. The guest log is
**3,541 / 195,440**, within unchanged 3,900 / 196,500. Native trailing checks now
actually run: **twenty shell checks** pass, including `test-scratch-left-behind.sh`
and the output-budget contract, at 40 lines / 2,213 bytes; **159 CLI checks** pass
at 29 / 1,097. The job also confirms VM teardown. This fresh harness result
resolves the missing trailing execution prerequisite; it does not relabel the
previous local missing-fixture/PRoot attempt as passing. No further cap changes.

**Scoped review decision:** standalone public setxattr whole-plan migration,
same-value semantics and refusal-order repair pass implementation/evidence
review under the existing caller-exclusion and healthy-layout contract. The
final source check reconfirms pre-callback shared entry/refusal, complete inline/
external validation before a unique no-effect decision, advancing-clock policy,
one owned staged plan, planned-counter/feature-preserving dirty normalization,
and complete guarded marker/commit/checkpoint/flush/allocation publication.
Fault/unwind/poison retirement and Rust/C error priorities remain covered by the
promoted controls. No further blocker was found within that scope. The full
decision and limits are in the existing `CASEFOLD-INDEX-REVIEW.md`.

The diff from refusal-order implementation `4079f36` to this budget source changes
only `chores.yml` and review/progress records; driver, tests and focused workflow
are byte-unchanged. Its exact implementation's
[focused Linux run 39458800523](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39458800523)
therefore remains applicable: all fifteen main-job oracles, sixteen setter
layouts, eight first-attribute profiles, 84 retained metadata profiles, 32
ordinary policy observations, sixteen byte-identical policy-emission images and
five symlink tests pass their actual saved-evidence audits. This is not a claim
that a new focused run executed at `2c6ece6`. Four duplicate diagnostics remain
separate from healthy-image qualification.

This checkpoint changes only the existing handoff and review. No production,
test, workflow, cap, floor, filter, dependency, sibling or lock changes. Caller
coordination for other mutators, reads, raw handles, maintenance and C lifetimes
still applies. There is no owner proof, buffer seal, Ready/admission state,
arbitrary-corruption safety or whole-operation crash atomicity. Mixed zero-extra,
ignored fuzz, insertion aliases and complete ownership/mapping/lifecycle/seal/
admission/index collision/growth/recovery gates remain open. No host oracle,
mount, forbidden implementation source or Windows/SteamOS/card test was used.

Verified current full evidence is in ignored `tmp/casefold-budget-review/full`,
including `complete-retention.json`, all four archives and the native job log.
Focused implementation evidence remains in `tmp/casefold-priority-review`.
Another machine should retrieve the named runs rather than assume scratch files
survive. Stages 4–5 are still not yet quantified; all casefold writes stay blocked.

**Next bounded task:** audit the existing public unlink path/byte/parent-inode
and C entry points, nested calls, refusal/error ordering, matched-entry planning,
last-link/shared-storage changes and all marker/commit/publication effects.
Specify the shared-context and owned-whole-buffer boundary, concrete red controls
and independent Linux evidence before implementation. Do not recursively enter
the gate or broaden into rmdir/rename/create/data-write migrations in that audit.
Preserve all current tests and qualification gates.

## Complete refusal-order retention and measured native-budget repair — 2026-10-20

Started from clean, fetched `e758f21` on `casefold-rw`. The runtime policy is
current/enforced and actual Git/API/redirected artifact access works. No
environment setting, sibling, dependency pin or lock changes.

[Full run 39458801198](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39458801198)
at implementation source `4079f36bfa8597b4357567cc4dd0b6e28409620a` has completed.
The guest job passes; native and aggregate jobs fail. The actual downloaded
native job log identifies **exit 65 from the whole-suite output budget**, not a
test failure. Both native and guest tier archives match GitHub's advertised
SHA-256; complete successful-name Counters show each retains all **1,972** prior
occurrences and adds exactly the two inline-name/EA-inode refusal-order controls.
Both report **1,974 passed / zero failed / one unchanged ignored**
`deep_walk_byte_flip_fuzz_never_panics`. All **171** guard/operation/parser/fixture
controls execute once in five contexts: both whole suites and standalone,
x86_64 and aarch64 units. Each unit context passes 1,139, retaining all 1,137
previous occurrences and adding those same two controls.

Measured complete output:

- Native: **3,374 lines / 186,123 bytes**, exceeding the old 186,000-byte cap by
  **123 bytes** while staying within the unchanged 3,700-line limit.
- Guest: **3,543 lines / 195,557 bytes**, within unchanged 3,900 / 196,500 limits.

Before the configuration change, replaying the verified native archive through
the actual tier wrapper reproduces **exit 65** at 186,000 bytes. The native-only
byte cap in `chores.yml` is now **189,000**. This restores 2,877 bytes of measured
headroom: the actual log grew 2,523 bytes since the earlier 183,600-byte source,
so a 3,000-byte increase retains roughly the prior 2,400-byte cushion. The table
next to the task records the exact measurement and run. The same complete log
now passes the same wrapper; both replay outputs are byte-identical to the
verified source archive. The unchanged 1,060 execution floor counts all 1,974
passes. No output is suppressed and no filter, line cap, floor or test changes.

This replay is **not fresh execution of the Linux suite**. The failed native
task stopped before its subsequent script/CLI checks, even though the whole
test suite itself passed. Fresh full CI for the numeric configuration repair
must still verify those remaining checks and the aggregate. Current-source
focused Linux run `39458800523` already passes both jobs; all fifteen named main
oracles and the sixteen/eight setter/first-attribute, 84 retained metadata,
sixteen policy-emission and five symlink audits pass at the unchanged driver
implementation. The completed scoped source walk found no further blocker under
the existing caller-exclusion contract. Standalone setter-family closure remains
pending successful repair-source full CI, not another implementation migration.

**Local validation:** the output-budget shell contract passes, as do the shared
agent, provenance and CI-aggregate checks. The broader local `chore test:scripts`
attempt fails at `test-scratch-left-behind.sh`: its integration probe requires
the missing VM-generated `ext4-basic.img`/`ext4-csum-seed.img` fixtures. Its log
also records PRoot Git `Bad address` diagnostics during the consumer-name check.
This is not a passing script tier and is not hidden or skipped. The cloud host
has no KVM; use the existing fresh CI harness for that prerequisite rather than
host oracles/mounts or a weakened check. The failing local log is retained as
`tmp/casefold-priority-review/full/local-scripts-attempt.log`; the relevant
budget contract and complete archived-log replay pass outside PRoot.

Ignored audit evidence is in `tmp/casefold-priority-review/full`, including the
downloaded job log, verified tier archives and `complete-retention.json`; replay
outputs are `tmp/logs/replay-native-old.log` and `replay-native-new.log`. Other
machines should retrieve the named run. No host oracle, mount, local VM,
forbidden implementation source or Windows/SteamOS/card test was used.

**Next bounded task:** inspect fresh full CI for the budget-repair commit,
including native trailing checks, actual tier budgets, both complete occurrence
Counters and all 171 controls across five contexts. Repair only demonstrated
failures, then conclude the scoped setter-family review before selecting the
next family audit. Preserve caller-exclusion/ownership/lifecycle/seal/admission,
mixed zero-extra, ignored-fuzz, insertion-alias and full mapping/collision/growth/
recovery gates. Stages 4–5 remain not yet quantified; casefold writes stay blocked.

## Current-source focused metadata evidence — 2026-10-20

Started from clean, fetched `58cacf3` on `casefold-rw`. The actual runtime policy
is current/enforced, Git/API reads work and redirected artifact downloads work.
No environment configuration changed. This checkpoint audits new evidence for
refusal-order implementation source `4079f36bfa8597b4357567cc4dd0b6e28409620a`.

[Focused Linux run 39458800523](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39458800523)
now passes both jobs. Its main `casefold-fixtures` archive was downloaded,
matched to GitHub's advertised SHA-256 and checked for ZIP integrity. The large
image set was read from the compressed archive without extracting it in full.
All fifteen named main-job oracles execute exactly once, with zero failures or
ignored tests; their complete logs fit the existing line and byte limits.

Current-source saved bytes and independent reports pass the existing audits:

- **Sixteen setter layouts:** 192 setter calls, including no-effect calls,
  and 64 preserved peer/untouched records; this is not a count of actual writes.
- **Eight first-attribute profiles:** 24 emitted and 24 independent reference
  records, with the advancing-clock unchanged-value policy retained.
- Together, **280 emitted Linux records and 304 debugfs records** agree with
  saved inode identity, attributes/values, payload, metadata, allocation/counters,
  shared references, EXT_ATTR publication, dirty/closed state and checksums.
- All **84 retained metadata profiles** pass: four private chmod, eight public
  chmod, sixteen chown, 24 utimens, sixteen flags and sixteen removexattr.
  Removal evidence includes 112 removal calls and 144 Linux/debugfs records.

The same source's separate policy and symlink artifacts were already verified:
32 ordinary unchanged-ctime observations, sixteen byte-identical Rust/C emission
images and all five symlink tests. Four duplicate diagnostics remain outside
healthy-image qualification. The scoped source walk found no further blocker
under the existing caller-exclusion contract; these new healthy-image results
do not establish ownership, unrestricted concurrency, fault or recovery safety.

[Exact full run 39458801198](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39458801198)
is still executing both complete suites at this checkpoint. Standalone and
aarch64 units already retain all 1,137 preceding occurrences and add only two,
with all 171 controls executed. The expected whole-suite count is 1,974, but
only the preceding `25a039e` source's actual 1,972-pass full results are credited.
No new complete-suite log or budget failure has been observed here; no output
cap or floor changes. Standalone setter-family closure remains pending.

Saved evidence/audits are in ignored `tmp/casefold-priority-review`; another
machine should retrieve the named exact-source run. No production/test code,
dependencies, sibling files, lockfiles or write gates change. No local VM,
host oracle/mount or Windows/SteamOS/card test was used.

**Next bounded task:** inspect the completed full run and its actual native/
guest archives; compare both complete successful-name Counters and all 171
controls across five CI contexts. Repair only an observed failure, measuring
actual output before any budget adjustment. Then conclude the scoped setter
review before another family migration. Stages 4–5 remain not yet quantified;
casefold writes and the existing broader qualification gates remain blocked.

## Complete baseline retention and current-source review evidence — 2026-10-20

This is an evidence/review checkpoint on fetched clean
`4079f36bfa8597b4357567cc4dd0b6e28409620a`, not another family migration.
The current runtime reports its network policy enforced, and actual Git/API/
redirected artifact reads work. No environment configuration changed.

**Completed full baseline.** Exact source
`25a039e28a86cead78093244e5d7c1450647474c` now passes
[full CI 39452506096](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39452506096).
All native, guest, standalone-unit and aarch64 tier archives were downloaded and
checked against GitHub's advertised SHA-256. Complete successful-name Counters,
not only totals, show both full suites retain all 1,951 prior occurrences and
add exactly the same 21: seven native symlink controls, five Linux symlink tests,
eight same-value native controls and the separate policy oracle. Both report
**1,972 passed / zero failed / one unchanged ignored**
`deep_walk_byte_flip_fuzz_never_panics`. All 169 guard/operation/parser/fixture
controls execute once in each of five contexts: both full suites, standalone
unit, x86_64 unit and aarch64 unit. The three unit contexts pass 1,137 each.

The native complete-suite log measures **3,372 lines / 185,898 bytes**, under
3,700 / 186,000; guest measures **3,540 / 195,346**, under 3,900 / 196,500.
The source's existing budgets now pass with complete output retained. No cap,
line limit, filter or floor is changed here. The native margin is only 102 bytes;
inspect the current source's actual output rather than assuming its two added
controls still fit. A prospective cap failure is not an observed failure or
permission to claim new-source execution.

**Current refusal-order source.** The separately downloaded unit artifact for
`4079f36` in [full run 39458801198](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39458801198)
passes 1,139 tests, retaining every one of the previous 1,137 occurrences and
adding exactly the two refusal-order controls. All 171 controls execute once.
Its complete log is 1,395 lines / 97,326 bytes under unchanged unit limits.
The separately SHA-256-verified aarch64 unit archive likewise retains all 1,137
occurrences, adds only those two controls and executes all 171 controls once:
1,139 passed, with a 1,389-line / 96,889-byte log.
The full run remains in progress; no new-source complete-suite count is credited.
The expected full-suite total is 1,974 (the measured 1,972 plus those two controls),
subject to actual successful-name comparison, not a substitute for execution.

The policy/symlink job in
[focused run 39458800523](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39458800523)
passes at that same source. Both published archives were SHA-256-verified and
read-only audited. All five symlink tests execute; saved 1/4 KiB targets and
Linux/Rust/C trees agree, and the relative casefold chain retains its byte/
refusal checks. All 32 ordinary policy observations preserve ctime, and all
sixteen Rust/C emission images equal their saved Linux-created input byte for
byte. Four constructed duplicate diagnostics retain both copies; they remain
outside healthy-image qualification. The main focused job, including all fifteen
retained oracles, remains in progress at this checkpoint. Earlier complete
16-layout/8-first-attribute/84-retained-profile audits at `25a039e` are recorded
above; they are not silently relabeled as current-source results.

**Source walk.** Rechecked public Rust string/byte dispatch and C validation/
error clearing, one shared context before filesystem callbacks, the pure inline
preflight with preserved legacy external-only precedence, and the final complete
both-location/unique-value decision. A no-effect call has no clock/marker/commit/
flush/publication; changed values and duplicate cleanup still prepare one owned
buffer. External helpers stage only. The setter boundary preserves planned
counters/features while normalizing only dirty state/checksum, then guards marker
and complete commit/checkpoint/flush/allocation publication. Read failures and
planning refusals stay pre-effect; Rust/C unwind and uncertain-effect retirement
remain covered by the promoted controls. No further implementation blocker was
found within that scope, but **standalone family closure is not yet credited**:
the current source still needs exact full/focused outcomes and complete-suite
retention.

The contract continues to require caller coordination for unguarded mutators,
raw handles, reads, maintenance and C lifetimes. No ownership proof, buffer seal,
admission readiness, general malformed-layout acceptance or crash atomicity is
introduced. The ignored fuzz, mixed zero-extra, insertion aliases and full
ownership/mapping/lifecycle/seal/admission/index collision/growth/recovery gates
remain open. No local VM, host oracle/mount, forbidden implementation source,
real card or Windows/SteamOS test was used.

Ignored artifacts and audits are in `tmp/casefold-final-review/full` (the complete
`25a039e` baseline) and `tmp/casefold-priority-review` (current-source evidence).
Future machines should refetch the named runs. This checkpoint changes only
review/progress records; it does not advance implementation or enable writes.

**Next bounded task:** finish inspecting `4079f36`'s exact full and focused runs,
including main metadata artifacts, all 171 controls in five contexts and both
whole-suite successful-name multisets. Repair only an actual failure, with a
reproduced red before a code fix and actual log measurements before a cap change.
If they pass, conclude the standalone setter-family review within the stated
contract before choosing the next mutation-family design audit. Stages 4–5 stay
not yet quantified; every casefold write gate remains closed.

## Verified symlink evidence and setter refusal-order repair — 2026-10-20

Started from clean, fetched `25a039e28a86cead78093244e5d7c1450647474c`.
The runtime policy is current/enforced and real Git/API/redirected artifact
reads work in this editor. No environment settings, siblings, pins or locks changed.

[Exact focused Linux run 39452506142](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39452506142)
passes both jobs. All three actual archives were downloaded, matched against
the advertised SHA-256 digests and inspected. The complete five-test symlink log
is 71 lines / 4,703 bytes, with five passed / zero failed / zero ignored. The
read-only saved-image audit confirms six accepted links per 1/4 KiB profile,
absent rejected names and exact target contents. The Linux/Rust/C namespace
images have identical six-entry trees, inode identities, link counts and payloads.
The casefold image stores the corrected relative chain and passes the original
read/refusal/byte-identity/e2fsck assertions. This credits the supported relative
Linux namespace, not absolute-target namespace equivalence, general first-refusal
atomicity, interrupted writes or public casefold-write qualification.

The same exact source's sixteen setter layouts, eight first-attribute profiles,
sixteen byte-identical Rust/C policy-emission images and all 84 retained metadata
profiles pass their actual raw-byte/report audits. All fifteen original focused
oracle logs contain one passed / zero failed / zero ignored; the separate policy
log repeats 32 ordinary unchanged-value observations and four distinct duplicate
diagnostics. Fresh standalone and aarch64 unit archives retain every preceding
1,129 occurrence and add eight (1,137 passed), executing all 169 controls.
Full [39452506096](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39452506096)
is still running: complete native/guest multisets and final output measurements
are not inferred from the focused or unit results.

**Review finding and actual red.** The new no-effect candidate read the external
block before invoking inline name/entry validation. The preceding inline-capable
planner rejected invalid namespace, empty suffix and oversized names before
external I/O, and refused an unrelated inline EA-inode entry before external
corruption. Those precedence rules regressed; legacy external-only layouts must
instead retain their external-error priority. A public Rust/C control and a
separate inline EA-inode control were added. Both failed before repair, reporting
external magic corruption; the three retained no-effect controls in the same
focused run passed (three passed / two failed, exit 101).

The repair preflights complete inline validation through the existing private
predicate before reading external storage. Its initial match result never
permits success: both locations are still parsed for the final unique-value
verdict. Legacy layouts with no inline area keep their prior ordering. No new
public API, raw commit, timestamp, format or write-gate behavior is added. Missing
path and C argument validation remain ahead of these checks; planning refusals
have no clock, marker, write or flush effects. Both new controls now pass across
1/4 KiB layouts, including Rust/C errno behavior and byte preservation.

Fresh `chore test:unit` passes **1,139 tests**, retaining every one of the 1,137
preceding successful occurrences and adding exactly two, with zero failed/ignored
and all 171 controls. Output is 1,387 lines / 96,842 bytes under unchanged caps.
Full lint passes, including provenance/shared-agent/aggregate-gate checks, and
whitespace checks pass. No local Linux run is claimed on this no-KVM host.
The source walk and independent artifact review remain open for family closure:
the newly repaired source needs exact CI, and the preceding full run is pending.

Ignored evidence and read-only audits are in `tmp/casefold-final-review`; actual
red/green logs are `tmp/logs/setter-priority-{red,green}.log`. Later machines
should refetch the named exact-source artifacts. No host oracle/mount or forbidden
implementation source was used. No output cap, floor or filter changed here.

**Next bounded task:** inspect the refusal-order repair's exact full/focused
results and complete native/guest occurrence multisets. Reproduce any actual
failure before repair or numeric cap change. The five symlink tests and earlier
setter/retained image audits above need not be rediscovered, but their passing
source is not a substitute for new-source CI. Finish standalone setter-family
closure only after these prerequisites pass, before migrating another operation.
Keep ignored fuzz, mixed zero-extra, insertion aliases, ownership/mapping/lifecycle/
seal/admission, index collisions/growth and recovery open. Stages 4–5 are not yet
quantified; casefold writes and Windows/card/SteamOS qualification remain blocked.

## Repaired setter evidence and symlink fixture correction — 2026-10-20

The next authorized evidence check inspected exact repaired application source
`01f96c8a2ee7bec7b9f69d5563ec7cbd1548938b`.
[Focused workflow 39391995739](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39391995739)
passes both jobs. Both actual artifacts were downloaded and checked against
GitHub's advertised SHA-256. Read-only audits pass for all sixteen setter layouts,
eight strengthened first-attribute profiles and 84 retained metadata profiles.
The sixteen/eight audit again checks 280 emitted Linux and 304 debugfs records,
including the same-value case's preserved original ctime. The separate policy
artifact has all 32 strict ordinary Linux observations and sixteen Rust/C emitted
images, each byte-identical to its saved Linux-created input. Its complete log
measures 169 lines / 11,880 bytes. Four duplicate diagnostics retain both copies
and unchanged inode bytes; they remain distinct from healthy-image qualification.

Three exact CI unit archives (standalone, x86_64 and aarch64) retain all 1,129
preceding successful occurrences and add exactly the eight promoted controls:
1,137 passed / zero failed / zero ignored, with all 169 guard controls executed
once per context. The standalone unit log is 1,393 lines / 97,099 bytes, under
the unchanged 1,400 / 100,000 envelope. These actual results supersede earlier
pending repaired-driver emission status; they do not establish full-suite success.

**Actual full-CI blocker.** Backport full run `39388402569` and exact repair
[39391995709](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39391995709)
fail in the casefold symlink oracle. Both repaired-source native/guest logs show
`expanded_symlink_targets_keep_linux_casefold_reads_and_write_refusal` failing
at the empty-inode parse. The native tier's other two traversal tests pass;
its later target-boundary binary did not execute after cargo stopped. No claim
that all five new tests ran, or that the entire baseline remains covered, is made.
The archive logs show test exit 101, not an output-budget breach.

The fixture stored `/Fold/CHILD` in the root's `Alias` symlink. Linux resolves
that absolute target at the guest system root, outside the mounted scratch image.
`stat` therefore failed, but nesting it inside `printf` produced successful
`inode|` output and hid the command status. The test now stores relative
`Fold/CHILD`, retaining the casefold/chained-relative traversal it intends to
judge. Python `os.stat` supplies the independent inode answer and propagates a
lookup failure before parsing. Every Rust/C comparison, refused writable mount,
full-image byte check and judged e2fsck assertion remains intact. Production
symlink traversal and all filesystem write gates are unchanged.

A focused step in the existing policy-evidence job executes both symlink
integration binaries, requires all five tests, and saves images plus the full log
as `symlink-backport-evidence`. It uses the existing kernel output envelope;
no whole-suite cap, floor or filter changes for this test-only correction.
Both binaries compile locally and lint passes, but no local Linux execution
is possible on this host. Fresh focused and full CI for the correction remain
required. This fixture correction does not qualify absolute symlinks against a
Linux namespace whose system root differs from the driver's volume root.

Actual artifacts, raw-image audits and unit occurrence comparisons are under
ignored `tmp/setter-policy-repair/repaired-ci`; the earlier backport failure is
in `tmp/upstream-backports-ci/full`. Refetch the named runs on another machine.
No host mounts/oracles or forbidden implementation source were used. The existing
caller-exclusion contract, unresolved ownership/lifecycle/seal/admission/index
work, mixed zero-extra and ignored-fuzz limitations remain intact.

**Next:** inspect correction-source focused/full CI and the actual five-test
symlink artifact. Repair actual failures before any new numeric budget change,
compare complete native/guest occurrence multisets, then finish standalone
setter-family review before another migration. Stage 5 is not yet quantified;
casefold writes, Windows/card and SteamOS qualification remain blocked.

## Same-value setter repair and independent policy references — 2026-10-20

The user authorized the next bounded checkpoint after the reproduced timestamp
gap. Work starts from fetched clean `62a4a90`; subsequent reference-only commits
are `290d6db` and its fixture correction `aa266f3`. Runtime routes and actual
Git/API/redirected artifact access work in this editor; no environment settings,
sibling checkouts, dependency pins or locks changed.

**Independent policy evidence.** A separate job in the existing focused workflow
records eight 1/4 KiB × legacy-128/checksummed-256 × direct/journal profiles.
The first job failed before collecting policy because Linux had deduplicated
three equal external values and the retained explicit-share constructor requires
one reference. A distinct shared-source value retains that constructor and all
strict checks. The corrected reference-only job at exact
`aa266f35d72d1c9c1854534001887ec148b7b955`, in
[run 39390763789](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39390763789),
passes. Its `setxattr-policy-reference` archive was downloaded and its advertised
SHA-256 verified. All 32 ordinary observations preserve ctime, raw inode bytes,
attribute values and external block bytes. These include empty and nonempty
values, ordinary external storage and shared external blocks. The saved log is
153 lines / 9,920 bytes. Full workflow status and the separate main fixture job
must be checked separately; a successful reference job alone does not establish
full CI or repaired-driver qualification.

All four constructed duplicate-name cases return success with unchanged ctime,
raw inode bytes and both attribute copies still present. These are diagnostics
of deliberately noncanonical inputs, not healthy-image or repair qualification.
The driver retains its existing cross-location cleanup rather than allowing an
identical first match to bypass it. Same-region duplicate repair, general corrupt
layout acceptance and broader mixed zero-extra qualification are not added.

**Actual red and repair.** The saved Rust/C probes were rerun unchanged and both
failed. Eight native controls then report three passed / five failed before
repair: the original two timestamp failures, clean/dirty no-effect assertions,
shared-block preservation and logical equality despite unused packing bytes.
The three guard/validation/cleanup/feature-initialization controls pass before
and after. All eight now pass. Their matrix covers Rust string/byte and C entry,
1/4 KiB, direct/journal, clean/dirty states, empty/nonempty/external values, real
same-length changes, shared peers, unsupported EA-inode/corrupt external blocks,
held/failed/read-only/casefold/replay gates and preserved publication paths.

The private planner returns `Option<BlockBuffer>`: no plan means an ordinary,
unique, equal-valued attribute after complete parsing of both locations and the
existing external header/checksum checks. A first-match reader is not used.
Current primary-SB feature bytes are checked because the mount snapshot can
predate first EXT_ATTR publication. Missing feature publication, zero-extra
initialization, zero-reference external blocks and duplicates retain the
existing mutation path. An accepted no-effect operation has no clock, marker,
commit, flush or allocation-publication effect and leaves operation status and
device bytes unchanged. Context entry and volume refusal remain ahead of all
planning. Actual changes still use the existing guarded whole-plan boundary;
no public API, raw commit primitive, removal boundary or C lifetime contract
changes. The historical red patch is marked promoted; do not reapply it now.

The original sixteen-layout oracle now expects the unchanged-value case's
original ctime and unchanged epoch, while retaining all mutation assertions.
The eight fresh-attribute emissions use a clock advanced ten seconds between
identical sets and check byte/status identity; their independent Linux reference
assertion now strictly requires preserved ctime. The separate policy job also
checks sixteen Rust/C emitted images against saved Linux-created input bytes,
with a changing clock, no clock calls, stable operation status and judged
non-repairing e2fsck. These revised emitted-image checks compile locally but
need fresh exact-source CI execution. Healthy reference-only answers above
are not proof that this repaired driver has executed in Linux.

Fresh native unit validation passes **1,137 tests**, preserving every preceding
1,129 successful occurrence and all 161 preceding controls, with eight additions
and zero failed/ignored. The measured unit log is 1,385 lines / 96,615 bytes;
its existing caps are unchanged. Full lint and the existing output-budget,
aggregate-gate, provenance, shared-agent and whitespace checks pass. The library
including revised host-only oracles compiles locally, but this host has no KVM;
no local Linux run or Windows/card test is claimed.

**CI prerequisites and limits.** Backport focused source `b6ccad6` now passes
[39388402696](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39388402696).
Its actual SHA-256-verified setter artifact repeats the sixteen/eight-profile
byte/report audit successfully. Its full run still needs inspection for actual
execution of all five new symlink tests and complete-suite retention. The older
budget-only run `39384659463` completes with all 1,951 native and guest test
occurrences retained, but the guest output wrapper fails by 17 bytes. The
separate measured guest-cap repair above preserves the exact archived output,
line cap, floor and filters. New-source output may grow again: measure actual
logs before any further cap change; the replay is not a new-source execution.

Ignored local evidence is under `tmp/setter-policy-repair`, including red/green
logs, the reference artifact, raw policy audit and native occurrence Counter.
Later machines should refetch the named exact-source artifacts. No kernel or
e2fsprogs implementation source, host oracle or host mount was used.

**Next bounded task:** inspect the repaired source's exact full/focused results,
all sixteen policy-emission images, the strengthened first-attribute matrix and
84 retained metadata profiles. Reproduce actual failures, including budgets,
before repair; compare complete-suite occurrences and actual symlink execution.
Then complete standalone setter-family implementation/evidence review within
the caller-exclusion contract before migrating another operation. Keep mixed
zero-extra layout, ignored fuzz, insertion aliases, ownership/mapping/lifecycle/
seal/admission, index collisions/growth and interrupted-write recovery open.
Stages 4–5 remain not yet quantified and all casefold write gates remain closed.
No real-card, Windows or SteamOS qualification is credited.

## Setter artifact audit and reproduced same-value policy gap — 2026-10-20

This checkpoint starts from clean, fetched `b6ccad602cd71f6a6ca96c6d55585982b6bbece6`.
The runtime policy is current/enforced and Git/API plus real redirected downloads
work in this editor. No environment configuration changed, and this does not
verify the previously failing regular-chat runtime.

Exact backport full CI [39388402569](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39388402569)
and focused CI [39388402696](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39388402696)
are still running. The preceding budget-repair full run
[39384659463](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39384659463)
is also still running. No current failure or budget measurement is inferred from
that pending state. The new standalone-unit artifact was downloaded and its
advertised SHA-256 verified: 1,129 passed / zero failed / zero ignored, preserving
all 1,122 preceding successful occurrences and all 161 guard controls, adding
exactly the seven recorded native backport tests. Its full log is 1,385 lines /
96,223 bytes. Full-suite retention and actual execution of the five new Linux
symlink tests remain pending, not credited from compilation or unit results.

While those prerequisites run, the earlier setter source's independent artifact
was audited without claiming new-source qualification. Successful focused run
[39348758499](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39348758499)
is tied to exact `9035b80616c5bdb37940960e27883463752098d3`.
Its downloaded `casefold-fixtures` archive matches GitHub's advertised SHA-256
`db0c3598da4e86d4471bb020236d7309f911fbb9fd0b4fa8dafd84c182cbd6ce`.

Read-only public-format audits of saved input/prepared/dirty/final bytes and
independent debugfs/Linux reports pass for:

- All sixteen Rust/C × 1/4 KiB × legacy-128/checksummed-256 × direct/journal
  setter layouts: 192 setter operations and 64 preserved peer/untouched records.
- All eight fresh EXT_ATTR-clear first-attribute profiles: 24 emitted inode
  records and 24 independent Linux reference records. Attribute values, payload,
  identities, timestamps, free counters, allocation, shared references, dirty/
  closed state, feature publication and applicable checksums agree with the
  saved reports. Totals are 280 emitted Linux and 304 debugfs records.
- All 84 retained metadata profiles: four private chmod, eight public chmod,
  sixteen owner, 24 timestamp, sixteen flag and sixteen removal profiles.
  The retained audits preserve their existing scope and timestamp-display rules.

**The same-value policy review found a blocker.** Every one of the eight saved
Linux reference reports says `same same_value_changed 0`: a second identical
inline set preserves ctime. First nonempty and first empty creation advance it
in all eight profiles. The driver's planner unconditionally stamps ctime. Its
fixed-clock Linux emission test cannot distinguish a second stamp from
preservation, so successful readback does not establish equivalent behavior.

Two temporary advancing-clock native probes reproduced the mismatch through Rust
byte paths and C. Both calls succeed and retain the requested value, but ctime
changes from `1700000100` to `1700000110` instead of remaining unchanged. The
actual run reports zero passed / two failed / zero ignored and exit 101.
The exact executed patch and command are saved in
[CASEFOLD-SETXATTR-SAME-VALUE-RED.patch](CASEFOLD-SETXATTR-SAME-VALUE-RED.patch).
The existing test source was restored byte-for-byte; no failing test or
production repair remains active. These probes cover ordinary inline nonempty
values only. External/shared/duplicate and repeated-empty policies need separate
Linux answers before choosing a broader no-op boundary. Never use a shortcut
that bypasses validation, duplicate cleanup or existing operation/refusal gates.

After restoring the probes, fresh `chore test:unit` passes all 1,129 tests:
1,377 lines / 95,739 bytes, zero failed/ignored. The saved patch passes
`git apply --check`. Shared-agent integrity, provenance and whitespace checks
pass. No new full-suite or local Linux execution is claimed for these docs.

Cached metadata, archive, read-only audit scripts/results and probe logs are in
ignored `tmp/casefold-setter-evidence`; current unit evidence is in
`tmp/upstream-backports-ci`. Later machines should refetch the named artifacts.
The existing review records the finding. This is an evidence/audit checkpoint,
not standalone family closure, an implementation repair or an advance toward
public casefold-write readiness. No host oracle, mount or forbidden source was
used. Dependencies, locks, siblings, output caps and write gates are unchanged.

**Next bounded task:** inspect the pending exact-source runs and reproduce any
actual failure before changing budgets. Qualify same-value policy for ordinary
inline/external/shared values, empty values and duplicate cleanup, then repair
the demonstrated mismatch with advancing-clock controls and fresh independent
Linux evidence. Preserve validation and all baseline occurrences, and complete
standalone setter-family review only after those prerequisites pass. Keep mixed
zero-extra layout, ignored fuzz, insertion aliases, ownership/mapping/lifecycle/
seal/admission and full index collisions/growth/recovery open. Stages 4–5 remain
not yet quantified; all casefold writes are blocked and real-card/Windows/
SteamOS qualification has not been reached.

## Upstream symlink compatibility and CI-download backports — 2026-10-20

Reviewed original `antimatter-studios/rust-fs-ext4` through
`4169ccd0b812c42ff59246951425bd4dd1d54e24`. There were nineteen commits since
the recovery recount fix previously imported here. The user authorized the two
identified library fixes and the older missed CI-download retry improvement.
The original's larger CLI/dependency/CI-policy upgrades were not part of this
backport. Work started from current clean `1057c06`, preserving newer source.

**Target boundary (`4122d20`, upstream `45bed01` / #561):** both path and inode
creation APIs now reserve a terminating byte within the single data block and
4,096-byte path bound. The native regression actually failed on a 1,024-byte
target before the fix, then passed accepted/refused boundaries on 1/2/4 KiB
filesystems. The complete inode API binary passed 12 tests at that first commit.
The upstream Linux syscall comparison is adapted to the library APIs here;
the original test's newer CLI `ln` command is not present in this branch.
Its two 1/4 KiB tests compile for CI, compare Linux's own syscall boundary,
retain both library entry points and verify e2fsck/kernel readback.

**Path traversal (`d38cb90`, upstream `4db7413` / #558):** intermediate links
expand from the root for absolute targets and the link's parent for relative
targets. Operation-parent resolution follows its own final link too; an
operation's final name remains the actual entry. Lookup follows at most forty
links in total and returns `SymlinkLoop` with C ABI `ELOOP` (62, consistent with
this ABI's existing macOS POSIX numbering). Expanded components continue through
our existing checksum-validating, casefold-aware `find_entry`; no upstream
byte-only name matcher replaces the frozen Unicode path.

Four native controls actually failed with `NotADirectory` before that change.
The promoted controls now pass absolute, relative, chained and non-UTF-8 targets;
namespace operations through parent links; 40/41-link, self and two-link loops;
and all six guarded metadata families through a directory link. Added C ABI
coverage checks byte paths, final-link identity, loop errno and later success
clearing the previous error. A native read-only fixture additionally checks
Unicode casefold comparisons, literal dot traversal, stored spelling, malformed
name refusal and retained writable-mount refusal after link expansion.

The retained randomized inode/path comparison initially exposed that its old
inode walker still treated intermediate links as non-directories. The upstream
inode-walk twin was imported too: it uses `lookup_at`/`read_link`, not the new
path method, to express the updated semantics. Existing seeds, operation
sequences, outcome checks and byte-for-byte assertions are unchanged; the full
17-test inode API binary now passes. No test was dropped or weakened.

The adapted three-test Linux path oracle compares successful Rust/C unlink and
rmdir results with a Linux-mutated copy, then e2fsck and complete tree readback.
It also compares the 40/41/self/two-link lookup results with Linux and checks
casefold reads through links on a Linux-created UTF-8 12.1 strict volume, including
refused Rust/C writable mounts. Loop no-write assertions use read-only lookups;
legacy namespace wrappers still mark write intent before planning and have not
been migrated to the shared operation guard. Do not infer first-refusal atomicity,
reader synchronization or lifecycle/ownership qualification from this backport.

There are five additional kernel tests in total. The kernel floor is raised
from 3 to **33**, retaining the measured preceding 28 plus all five new tests.
Both files are selected by the existing automatic tier classifier. Existing
kernel/native/guest output limits and every other floor/filter remain unchanged;
new CI must provide actual measurements before any further cap adjustment.
In particular the preceding guest log had only 94 bytes of remaining headroom.

**CI download retry (upstream `7bb33f6` / #500):** both pinned chore-release
downloads use `--retry 5 --retry-all-errors --retry-delay 2`; TLS and SHA-256
verification remain intact. An actual pinned release download using those options
passed the published checksum. No dependency, lockfile, release packaging,
quick-profile policy or shared-agent upgrade was imported.

Fresh final native evidence is **1,129 passed / zero failed / zero ignored** in
1,376 lines / 95,679 bytes under unchanged 1,400 / 100,000. The required Counter
comparison retains all 1,122 preceding occurrences and adds exactly seven; every
one of the 161 guard/operation/parser/fixture controls remains. Full lint passes
(fmt, clippy, CI/provenance and pinned Unicode checks). Both Linux binaries compile;
the native output-budget shell guard, aggregate gate and whitespace checks pass.
The budget shell guard exposed a SIGPIPE false failure in its `printf | grep -q`
pipeline under PRoot: the consumer found the budget and closed the pipe before
the producer finished. Its input is now supplied with a here-string, retaining
the exact same regex/assertion. Both native and PRoot executions pass; no guard
or assertion was disabled.

Local logs and native occurrence comparisons are under ignored
`tmp/upstream-backports`; the earlier review probe is in `tmp/upstream-oct20-review`.
This host has no KVM, so **none of the five new Linux reference tests has executed
locally**. Fresh full and focused CI for the final backport source must be checked,
including actual kernel execution and complete-suite retention. The preceding
budget-only CI `39384659463` was still running at the last check; it is not proof
for the later backports. Casefold writes, deeper insertion/collisions/growth and
all existing qualification limits stay closed. Stage 4 and Stage 5 are not yet
quantified; this checkpoint does not complete either stage.

**Next bounded task:** inspect exact final-source full/focused CI, reproduce any
actual failure (including output-budget failures) and make only a measured repair.
Verify the new kernel tests actually ran before crediting independent symlink
qualification. Once prerequisites pass, resume the sixteen setter layouts, eight
fresh first-attribute reference/emission profiles, retained metadata images and
first/same/empty-value Linux policy audit, followed by standalone setter-family
review. Keep mixed zero-extra, ignored fuzz, insertion aliases, complete mapping/
ownership/lifecycle/seal/admission and full collision/growth/recovery open.

## Setter native-suite budget repair and retained CI evidence — 2026-10-20

This bounded checkpoint repairs the measured output wrapper failure. It does
not migrate another operation, qualify directory-index writes, or close the
setter implementation/evidence review. The user authorized proceeding after the
startup-instruction work. Fetched `casefold-rw` remained current at `92d6cca`.
Git/API and actual redirected artifact reads work in this editor runtime; that
does not establish recovery in the three reported failing regular chats.

Downloaded and SHA-256-verified all four tier archives (x86_64, whole guest,
standalone unit and aarch64) for both exact runs:

- Baseline `537821fb748395be308a0937bafac236bf8d4302`, successful
  [full CI 39203196663](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39203196663).
- Setter source `9035b80616c5bdb37940960e27883463752098d3`, failed
  [full CI 39348758428](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39348758428).

Run metadata ties every archive to its exact commit. The current application,
tests, header, dependency/toolchain pins, runner scripts and full-CI workflow
match the setter source; subsequent changes before this repair were startup docs
and a separate cloud-setup helper. Local metadata, archives and JSON comparisons
are under ignored `tmp/casefold-setter-ci-budget`; later machines should refetch
the named runs rather than assume that cache exists.

Occurrence comparisons use the required successful-test-name Counter, not only
totals. Every baseline occurrence remains; both full suites add the same 28 names
(27 native setter controls and the independent Linux setter test):

| Context | Baseline passed | Setter passed | Missing | Added | Failed / ignored at setter source |
|---|---:|---:|---:|---:|---|
| Whole native suite | 1,923 | 1,951 | 0 | 28 | 0 / 1 existing slow fuzz |
| Whole guest suite | 1,923 | 1,951 | 0 | 28 | 0 / 1 existing slow fuzz |
| Standalone unit | 1,095 | 1,122 | 0 | 27 | 0 / 0 |
| x86_64 unit | 1,095 | 1,122 | 0 | 27 | 0 / 0 |
| aarch64 unit | 1,095 | 1,122 | 0 | 27 | 0 / 0 |

All 161 guard/operation/report-parser/fixture controls execute exactly once in
each of those five contexts. The ignored test in both full suites is unchanged:
`deep_walk_byte_flip_fuzz_never_panics`. No broader qualification is credited
from its default ignored result. Successful-name counts also match the sums of
the harness result summaries, with zero failed tests in all inspected logs.

The native `suite.log` grew from **3,313 lines / 180,692 bytes** to **3,339 /
183,600**: 2,908 additional bytes, 600 beyond the old 183,000 limit. Its actual
job log explicitly reports a passed command followed by output-budget exit 65.
Replaying the complete archive log through this checkout's existing `tier.sh`
reproduces exit 65 at 3,700 / 183,000. Raising only the native byte cap to
**186,000** makes the same replay pass; `cmp` proves both saved outputs are
identical, and the unchanged 1,060 floor counts all 1,951 occurrences.

The 3,000-byte increase tracks the 2,908-byte growth and restores 2,400 bytes of
headroom, compared with 2,308 at the preceding baseline. The 3,700-line cap,
test command, filters, floor and all output remain intact. The budget table is
updated with this exact measurement and source run. No other cap changes.
The guest log measures 3,506 / 192,906 under its unchanged 3,900 / 193,000 cap;
its small remaining byte margin must be watched in fresh CI, not called a failure
or silently widened here.

Fresh local checks pass: existing output-budget regression script, aggregate CI
gate, chore task parsing, shared-agent integrity, provenance and whitespace.
The replay is **not a fresh execution of the Linux suite**. No host oracle tools
or mounts were used, and no new native unit run is claimed for this configuration
change. Fresh exact-repair full CI remains pending push/execution; report
its actual run/status at handoff rather than reuse an older green verdict.

**Next bounded task:** inspect the repair's full CI and any actual failure. Once
that prerequisite passes, audit the exact setter/retained Linux images and reports:
sixteen setter layouts, eight fresh first-attribute reference/emission profiles,
shared peers, and first/same/empty-value metadata policy. Only then perform
standalone setter-family closure before another migration. The complete full-suite
occurrence and 161-control audits above need not be repeated for unchanged source,
but fresh repair CI must still be checked. Mixed zero-extra layout, ignored fuzz,
insertion aliases, complete ownership/mapping/lifecycle/seal/admission/collision/
growth/recovery remain open. Stage 5 is not yet quantified; every casefold write
gate stays closed, with no real-card, Windows or SteamOS qualification.

## Setter CI investigation and download-access block — 2026-10-19

Fetched and verified `9035b80616c5bdb37940960e27883463752098d3`; no newer remote
commit or local tracked work was replaced. Existing Git/API read and reported
push access work with the existing authentication. This follow-up changes only
the handoff/review record, not implementation, tests, workflows, budgets, pins,
locks or write gates. It does not advance implementation qualification.

[Focused Linux CI 39348758499](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39348758499)
**passed** for that exact source. Its API step records report success for the new
setter/first-attribute test and all six retained metadata oracles, plus the prior
insertion/ownership/fixture steps. This is workflow/step status, **not inspection
of the saved images, reports, exact test counts or same-value policy answers**.

[Full CI 39348758428](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39348758428)
**failed**. The whole-guest suite, aarch64, units, CLI, MSRV, semver, wasm and
provenance jobs report success. The x86_64 job passed lint, then failed `chore test`;
its actual check annotation says **exit code 65**. The aggregate consequently
failed. This points toward the output wrapper but does not identify the affected
tier, its measured bytes/lines, or prove that every test passed. No numeric cap
is changed from that annotation alone. The preceding checkpoint's explicit
full-suite headroom warning remains relevant, not a substitute for the log.

Actual fresh downloads were attempted using authenticated GitHub APIs and the
normal proxy. All required archives/logs are still **blocked**:

- `results-receiver.actions.githubusercontent.com`: full job logs.
- `productionresultssa4.blob.core.windows.net`: current x86_64/guest/unit/aarch64
  artifacts (IDs 12233109673, 12233881973, 12232471687 and 12232173464).
- `productionresultssa10.blob.core.windows.net`: focused `casefold-fixtures`
  artifact 12233130745.
- `productionresultssa8.blob.core.windows.net`: baseline x86_64/guest artifacts
  12165584449 and 12165809274 at `537821f`.

The runtime reports its restricted policy enforced, but its applied custom list
still lacks the requested exact hosts and log destination; the configured blob
wildcard is not sufficient for these observed downloads. The user previously
authorized applying missing routes; actual application remains outstanding. No
new credential, repository admin grant or Actions-write grant is needed to read
these results. Preserve the existing preset, entries and authentication. Report
hostnames only, never signed URLs or tokens.

Artifact metadata and API-provided SHA-256 digests were saved locally and tie the
archives to the correct source commits. **No archive bytes were downloaded or
hash-verified**, so no image audit, complete-suite multiset/count comparison,
161-control execution audit or full-suite budget measurement is credited here.
Local investigation metadata/download and analysis helpers are in the temporary
`/workspace/work/setter-review` directory, not assumed in another environment.

The bounded source walk reconfirmed the pre-callback shared-context entry,
verdict-only volume refusal, one owned buffer and staging-only external helper.
The setter-specific boundary still validates context and staged SB shape/magic,
normalizes only the dirty bit/checksum, and guards marker plus complete commit/
flush/publication. C validation and error clearing remain ahead of dispatch;
other families/readers/raw handles/C lifetimes still need caller coordination.
This partial walk is **not standalone family/evidence closure**. In particular,
the fixed driver clock in the current first-attribute test does not distinguish
whether a second identical set changed ctime; inspect the saved independent
Linux same-value observation and qualify the intended policy before closure.
No new defect or repair is claimed from inaccessible output or model memory.

**Next bounded task:** apply the missing destinations, download and SHA-256-check
the actual exact-source and baseline artifacts, read the x86_64 failure log and
measure the failing tier. If it is only an output-budget failure, retain all
output, record the actual measurement and adjust only that measured cap with
its test floor/filter unchanged; otherwise reproduce the actual failure first.
Compare both complete-suite occurrence multisets and inspect all sixteen setter
layout profiles, eight first-attribute profiles and retained metadata profiles
before standalone setter-family closure or another migration.

Stage 5 remains **not yet quantified**. The ignored slow fuzz, mixed zero-extra,
insertion aliases, complete ownership/mapping/lifecycle/seal/admission/collision/
growth/recovery gates remain open. Every public casefold write gate stays closed;
no real-card, Windows or SteamOS qualification follows from the green focused
status or this documentation-only investigation.

## Guarded public setxattr whole-plan implementation — 2026-10-19

Started from fetched `537821fb748395be308a0937bafac236bf8d4302`; its exact
full CI reports success. Git/API read and reported push permission reuse existing
authentication. Actual CI logs/artifact downloads were blocked at
`results-receiver.actions.githubusercontent.com` and
`productionresultssa8.blob.core.windows.net`; the user is adding those destinations.
No new credentials are needed. Do not credit inaccessible artifacts as inspected.

The saved `CASEFOLD-SETXATTR-RED.patch` was applied unchanged and actually rerun:
one passed / fourteen failed / zero ignored, exit 101. All intended assertions
were reached. Its ten public findings are now promoted without weaker assertions;
only the four explicitly constructed SB-order diagnostics call the new private
setter boundary instead of guarded marker plus raw commit. The original dormant
patch remains historical evidence at its recorded source, not a second active
module to apply over the migrated source.

Public Rust string/byte and existing C setters now enter the shared mount context
before filesystem/device/runtime callbacks, use verdict-only volume refusal and
plan one owned BlockBuffer. The single-caller external helper stages into that
buffer with no terminal commit. Inode/pointer/sector, inline/external duplicate
cleanup, shared refs, bitmaps/BGD/SB counters and feature ORs remain together.
Successful inline setters now stamp ctime and publish EXT_ATTR as external
setters did; independent first-inline reference/emission evidence remains pending.
The setter-specific whole-buffer boundary checks context identity and staged SB
shape/magic, clears only VALID_FS and restamps its checksum without losing
planned feature/counter/unrelated bytes. Marker-once, complete raw commit,
journal checkpoint/flush and allocation-view publication are failure-latching
effects. Raw commit, removal's boundary and shared xattr helpers are unchanged.
Ordinary buffers and context ownership are not seals, ownership/admission proofs
or Ready state. Other operation families, raw handles and C lifetime coordination
still require the existing caller-exclusion contract.

New native coverage retains the prior 134 controls and test names. Existing
real-worker Rust/C exclusion and retired-mount checks now include all six metadata
families in both directions. Setter controls exercise observed write/flush faults
through every callback in inline/grow/shrink/unshared/shared/duplicate/coherent-
zero-extra layouts, Rust planning and effect panics caught by C or Rust, retained-
context publication poison, clean/prior-dirty refusals, C validation/error clearing,
foreign contexts, malformed SBs and byte-preserving normalization. These fault
fixtures and artificial counter/publication diagnostics are native evidence, not
independent valid-volume or power-loss qualification. An initial layout expectation
incorrectly charged a new block for legacy growth from an already external value;
the expected zero delta was corrected, with no production change for that failure.

A new exact Linux oracle retains all six preceding metadata oracles. Its planned
matrix has sixteen Rust/C × 1/4 KiB × legacy128/checksummed256 × direct/journal
profiles, twelve setter targets and four untouched/shared peers per profile.
It covers inline/grow/shrink/remainder/free/shared release/private copy/external/
duplicate/coherent-zero-extra/empty/same-value branches. Eight separate fresh
256-byte-inode first-attribute profiles require EXT_ATTR clear on input, save
independent Linux first/same/empty reference observations, and test actual Rust/C
inline feature/ctime emission. It saves input/prepared/dirty/closed/reference
images and debugfs/Linux reports, checks raw marker/checksum/counter/pointer/ref/
sector state, exact metadata/data/xattrs/times and judged nonrepairing e2fsck.
The same-value Linux reference observation still needs actual artifact/policy
review; do not infer equivalence from successful readback. The focused workflow
adds its exact filter and one-test floor plus saved evidence without removing
prior filters/floors. It has compiled locally; no host mount/oracle is run.

**Fresh validation:** the saved red tier reports one passed / fourteen failed /
zero ignored (155 lines / 10,567 bytes). The fifteen promoted controls pass;
the expanded setter tier passes all 27 (286 lines / 18,443 bytes). Final complete
native units pass **1,122**, zero failed/ignored, retaining all **1,095** baseline
occurrences as a multiset, including duplicates and expected panics, and adding
exactly 27. Its log is **1,369 lines / 96,145 bytes**. The first complete new run
passed all 1,122 but failed output-budget exit 65 at **1,370 / 96,210**, exceeding
95,000 bytes. `chores.yml` records that actual measurement and raises only the
unit byte ceiling to 100,000; its 1,400-line cap, floor 440, every existing full-
suite cap/floor and all filters remain. No output is silenced. Full-suite growth
still needs its own CI measurement before any further cap change.

Fresh `chore lint` passes fmt, clippy `-D warnings`, frozen Unicode and the family,
CI/shared-agent and provenance contracts; whitespace and the retained ownership
red patch applicability pass. The locked release CLI build/install also passes
and stages all four command names. The new Linux oracle compiled, but Linux,
e2fsprogs/kernel, full CI and standalone family/artifact review for this source
remain **pending publication/execution/review**, not passed. This host has no KVM;
no host oracle/mount or new power-loss/Windows/SteamOS qualification is claimed.
Local baseline/red/promoted/expanded/final logs were retained under ignored
`tmp/guarded-setxattr` and `tmp/logs`; a copy is archived outside the owned
worktree at `/workspace/work/guarded-setxattr-evidence` before cleanup. These
instance-specific logs are not assumed to exist in a fresh chat.

The baseline uses Rust 1.95.0, chore 0.11.0, the retained immutable core v0.3.0
archive and harness v0.4.0 pin. Work is isolated in a linked worktree. PRoot needs
executor ptrace permission here; the retained helper works with that permission.
An initial direct symlink source view built the baseline, but full cargo fmt
traversed the enclosing onboarding workspace. The isolated worktree also needs
its sibling path excluded there. Final lint/build uses the same pinned PRoot
source view with a task-local read-only manifest overlay adding that exclusion;
the actual onboarding manifest is unchanged. PyYAML 6.0.3 was restored in the
retained venv for the existing CI-gate script. Original sibling sources/HEADs and
all repository dependency/toolchain pins/locks are unchanged. No older startup instructions replace the handoff setup.

**Next bounded task:** inspect this exact source's full/focused CI and actual
artifacts, repair any concrete failures with red controls, compare both full-suite
occurrence multisets and audit all new and retained Linux images/reports. Then
perform the standalone setter-family implementation/evidence review, including
Linux first/same/empty metadata policy. Do not migrate another family or claim
closure from native results alone. Stage 5 remains **not yet quantified**. Keep
mixed zero-extra layout, ignored slow fuzz, insertion aliases, full ownership/
mapping/lifecycle/seal/admission/collision/growth/recovery and every casefold-write
gate open. No real-card, Windows or SteamOS qualification follows.

## Public setxattr staging audit and executed red preparation — 2026-10-19

Started from clean fetched/published `b6190fdb29fdada30b86507bf4f6a90f471a54ae`.
Its [exact full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39196911445)
passes; the preceding removal-evidence documentation CI also passes. Existing
Git/helper, Bearer API and the same pinned toolchain/core source view work;
no environment configuration, secret, sibling, dependency pin or lock changed.
This checkpoint audits and saves a dormant patch. Production, active tests,
workflows, existing limits and all write gates are restored byte-for-byte.

**Executed red:** [CASEFOLD-SETXATTR-RED.patch](CASEFOLD-SETXATTR-RED.patch)
adds fifteen temporary native controls. Saved it, restored source, applied it
again, actually reran and reversed it: both actual runs report **one passed /
fourteen failed / zero ignored**, exit 101, with identical test-name results.
All fixtures reached the intended assertions; no setup failure is credited as a
behavioral failure. Final red output is **155 lines / 10,568 bytes**, inside the
isolated 800-line / 65,000-byte tier budget. No existing cap/floor/filter changes.

Ten failures exercise public behavior: three Rust string/byte/C guard bypasses,
four Rust/C marker writes on missing-path/invalid-prefix refusals, a C-caught
external planning Runtime panic without mount-failure latching, and inline-only
ctime/EXT_ATTR publication findings. The held-context calls return success and
write. Refusals return their expected error but first issue a marker write and
flush. Inline creation reads back its value with no external block, but leaves
ctime at 1,700,000,000 instead of injected 1,700,000,100 and leaves EXT_ATTR clear.
The latter two are native emission/desired-contract findings; new independent
Linux corruption/compatibility or same-value/empty-value policy is not claimed.
Rust supplies the panic hook; no foreign callback unwinds.

Four diagnostic failures deliberately prepare a clean SB with a feature OR and
synthetic free-count decrement before guarded marker/raw commit. They preserve
those edits and the checksum but restore VALID_FS at 1/4 KiB in direct/journal
profiles. **Current public set marks before planning. These are not current
public clean-bit regressions, valid allocation transactions or Linux evidence.**
The passing control executes external create and rewrite through Rust/C in all
four profiles: eight combinations / sixteen successful sets, preserving values,
ctime, feature bit, sector/free counts, dirty marker and close/reopen.

**Audit and next contract:** the current outer inline branch commits one buffer;
the single-caller external helper owns another buffer and has two terminal commit
sites. The detailed [setter review](CASEFOLD-INDEX-REVIEW.md) traces inline,
grow/shrink, duplicate cleanup, external/shared/free, zero-extra and feature/
counter paths and error priorities. The next migration enters the shared context
before callbacks, uses verdict-only refusal, prepares a complete single owned
buffer with a staging-only external helper, then guards marker and complete
commit/checkpoint/flush/allocation publication. Normalize an already-staged SB's
clean bit/checksum without replacing planned feature/counter/unrelated bytes.
Resolve the captured inline metadata findings explicitly with independent Linux
checks; preserve C validation, namespace/error order and all volume gates. This
is not a sealed plan, ownership/admission proof or Ready state.

**Fresh validation:** the baseline passes **1,095** native tests, zero failed/
ignored. All original tracked hashes were checked after reversing the patch.
Final tests also pass **1,095**, zero failed/ignored, retaining every baseline
occurrence (including duplicates and expected panics) with zero additions/losses
and all 134 guard/family/parser/fixture controls. Final output is **1,343 lines /
92,162 bytes**, within unchanged 1,400-line / 95,000-byte limits and floor
440. Fresh fmt/clippy lint, frozen Unicode, family/CI/shared-agent contracts,
provenance, whitespace and saved setter/ownership patch applicability pass.
This checkpoint adds no active tests or production fix. No new host oracle,
mount, CLI or VM qualification is claimed. Reproducible controls
and invocation live in the saved patch; ignored actual baseline/red/reapplication/
restoration/final evidence is in `tmp/casefold-setxattr-audit`.

**Next bounded task:** rerun the saved public failures before implementing the
private guarded setter whole-plan migration above. Retain all 134 controls,
all six exact metadata oracles and the baseline multiset. Add real six-family
Rust/C worker/fault/panic/close coverage and an independent Linux matrix for all
layout branches, shared peers and coherent zero-extra. Add separate fresh-image
first-attribute reference/emission cases: a normal input that already has attrs
also already has EXT_ATTR and cannot expose the inline feature finding. Qualify
exact metadata/timestamps, allocation/refcounts, dirty/closed state and judged
checks before family closure. Existing measured unit/full-suite output budgets
have limited headroom; measure any legitimate growth under the existing budget
contract. Keep mixed-layout and ignored-fuzz, insertion alias, full ownership/
mapping/lifecycle/seal/admission/collision/growth/recovery gates open. Stage 5
remains **not yet quantified**; casefold writes and real-card/Windows/SteamOS use
remain gated or unqualified.

## Public removal family review closure — 2026-10-18

Started from clean fetched/published `dbfb9f4d93dbcc62f6fb3f8c442d1dbff495fd32`.
Its production/tests/workflows/pins differ from repaired source
`396488818d774de1852b8872cce5ea0350ec5122` by **zero bytes**; only the two
existing handoff/review documents differ. Repaired-source
[full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39192028328)
and [focused Linux CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39192028338)
now **pass**, including the full aggregate. The preceding unrepaired source's
full CI ultimately failed in both whole-suite paths on the same zero-extra
fixture timestamp comparison: 792 library passes / one failure, with later
integration targets unrun. That failure remains historical evidence, not a pass.

**Actual whole-suite evidence.** Downloaded and SHA-256-verified the repaired
x86_64 and whole-guest artifacts. Each reports **1,923 passed / zero failed /
one ignored**. Executed-name Counter comparison, preserving repeated names and
expected panics, retains all **1,895** occurrences from the preceding qualified
flag-family whole-suite logs and adds exactly **28**: 25 removal controls, two
fixture controls and the exact removal Linux oracle. All **134** guard/family/
parser/fixture controls execute in each of five CI contexts: standalone native
unit, aarch64 unit, x86_64 debug unit, x86_64 release suite and whole guest. The
three native unit Counters equal the fresh local **1,095**-occurrence baseline.
The earlier native/aarch64 artifact digests were rechecked against this exact
run's metadata. The existing `deep_walk_byte_flip_fuzz_never_panics` remains
ignored in both full suites and is still a broader qualification gate.

| Exact repaired-source tier | Passed / failed / ignored | Lines / bytes | Unchanged cap |
|---|---|---|---|
| x86_64 debug unit | 1,095 / 0 / 0 | 1,345 / 92,209 | 1,400 / 95,000 |
| x86_64 full release suite | 1,923 / 0 / 1 | 3,313 / 180,692 | 3,700 / 183,000 |
| Whole guest suite | 1,923 / 0 / 1 | 3,477 / 189,631 | 3,900 / 193,000 |

Separate x86_64 oracle/kernel/lwext4 tiers pass **225 / 28 / 9** tests; installed
CLI reports **159** passing checks across six files and all twenty script checks
pass. These are overlapping tiers, not counts to add to the full suite. Existing
floors/caps/filters/artifacts remain unchanged. The preceding independent-evidence
checkpoint below already audited all sixteen removal profiles / 112 mutations /
144 final inode answers per Linux/debugfs stream and all 68 retained metadata
profiles / 136 final inode records; no image qualification is inferred merely
from a green aggregate.

**Review result:** the standalone removal-family implementation/evidence review
is complete with **no blocker within the existing caller-exclusion contract**.
Whole-plan preparation precedes marker/effects; prepared SB normalization keeps
planned counters and unrelated bytes; complete commit/checkpoint/flush/allocation
publication remains inside failure-latching effects. The four constructed
counter diagnostics and publication-poison control remain synthetic evidence.
Other readers/writers/flush/maintenance/raw handles and C lifetimes still need
caller coordination. Ordinary buffers/context identity do not establish ownership,
seals, admission or Ready state. The original mixed zero-extra/nonzero-extension
layout stays unqualified; the fixture repair changes no production timestamp rule.
No power-loss, arbitrary-map, casefold-write or hardware qualification is added.

**Fresh local validation:** baseline and final native tests retain **1,095**
occurrences with zero failed/ignored and no additions/losses. Final output is
**1,342 lines / 92,102 bytes**, within unchanged 1,400-line / 95,000-byte
limits and floor 440. Fresh fmt/clippy lint, frozen Unicode, provenance,
family/CI/shared-agent contracts, whitespace and retained ownership-red patch
applicability pass.
Only these two existing documents change; non-document tracked hashes remain
identical. No host oracle/mount, new local CLI run, environment draft, secret,
sibling or pin/lock change is needed. Ignored logs/scripts/results are in
`tmp/casefold-removal-closure`; exact CI artifacts and committed controls provide
reproducible evidence. This documentation checkpoint's own CI is separate from
repaired-source approval and is inspected after publication.

**Next bounded task:** audit existing public setxattr staging before migration:
inline creation/replacement, growth/shrink moves and duplicate removal, external
rewrite/shared COW/first-block allocation, zero-extra initialization, EXT_ATTR
feature changes and allocation-counter/publication behavior. Capture actual public
guard/marker failures red before proposing a fix. Retain all six exact metadata
oracles and all controls; do not generalize the removal boundary or migrate dispatch
in that design/red checkpoint. Stage 5 remains **not yet quantified**. Keep mixed-
layout, ignored-fuzz, insertion aliases, full ownership/mapping/lifecycle/seal/
admission/collision/growth/recovery gates open; casefold writes and real-card/
Windows/SteamOS use remain blocked or unqualified.

## Repaired removal independent-evidence checkpoint — 2026-10-18

Follow-up: exact repaired-source full CI and whole-suite artifact/control review
now pass; the family review closure above supersedes the pending status recorded
at this earlier evidence checkpoint.

Reviewed clean fetched/published `396488818d774de1852b8872cce5ea0350ec5122`.
Its [exact focused Linux workflow](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39192028338)
**passed**. Its [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39192028328)
is **still running**: unit, aarch64, installed CLI, provenance, wasm, semver,
MSRV and fixture jobs passed; x86_64 and whole-guest suites remain pending.
No result from the earlier failed `499c1b4` source substitutes for repaired-source
qualification. Existing authenticated Git/helper and Bearer API/artifact access
work. No environment draft, credential, sibling, dependency pin or lock changed.

**Independent evidence.** Verified the actual focused artifact's advertised
SHA-256 digest and safe archive paths before read-only inspection. The exact
removal filter executes **one passed, zero failed, zero ignored** test across
all **sixteen profiles**, Rust/C paths, 1/4 KiB blocks, legacy128/checksummed256
inodes and direct/journal commits. Its log is **401 lines / 52,394 bytes**, inside
the unchanged 2,100-line / 130,000-byte limit and one-test floor. All thirteen
other exact focused filters also passed; their existing floors/caps/artifacts
remain unchanged. The removal log records 16 mke2fs, 48 e2fsck, 288 debugfs and
32 kernel invocations, all returning zero.

Inspected all **64 four-phase removal images**, all **112 actual mutations**,
and **144 final debugfs / 144 Linux records**, including the two unchanged sharing
peers per profile. Saved bytes and independent reports agree on file identity,
mode/IDs, data, surviving attribute names/values, exact timestamps, external
pointers, sector/refcounts, free counts and allocation bits. All sixteen saved
Linux stderr reports are empty. Raw audit additionally verifies **32 superblock,
80 group-descriptor, 288 inode and 128 external-xattr checksum observations**.
The dirty image stays not-clean; ordinary journal checkpoint clears RECOVER
separately, and successful close restores the clean state. A freed block reused
by the later COW target is accounted to its new owner. No power-loss or arbitrary
corrupt-map qualification follows from healthy completed transactions.

The coherent zero-extra fixture passes strict Linux timestamp/xattr/data checks
and stays zero-extra after close and Linux readback. The older mixed zero-extra /
nonzero-extension layout remains unqualified; no precise kernel timestamp tuple
was captured for that original failure and no general layout rule is inferred.
The legacy duplicate is deliberately seeded after initial input e2fsck; this
review does not describe it as a normal Linux-created valid input.

All **68 retained metadata profiles / 136 final inode records** also agree:
four private chmod, eight public chmod, sixteen chown, twenty-four utimens and
sixteen flags. Each independent stream retains 96 timestamp and 96 flag phase
answers; the other 56 records agree. Their raw audit verifies 44 superblock,
66 group-descriptor and 88 inode checksum observations. Previous insertion,
FIEMAP, inventory, role, fixed-metadata, allocation and ownership exact filters
pass too; this checkpoint does not expand their earlier bounded qualifications.

**Source and failure review.** Re-read dispatch, whole-plan preparation,
normalization, full commit/publication effects, failure-latch lifetimes and all
25 removal controls. No blocker was found within the existing caller-exclusion
contract. The operation context precedes lookup/device/runtime callbacks;
planning refusals discard local edits before the marker. Any staged primary SB
retains planned counters and unrelated bytes while clearing only VALID_FS and
restamping its checksum. Inode-only plans add no SB. The guarded effect includes
journal/direct writes, checkpoint/flush and post-durability allocation publication.
Errors and unwind latch failure before outer catchers, preventing further writes
or clean close. Four synthetic counter controls prove boundary ordering, not
valid allocation transactions; publication-lock poisoning is synthetic too.
Other readers/writers/flush/maintenance/raw handles and C lifetimes still require
caller coordination. A context and ordinary owned buffer are not an ownership
proof, seal, admission decision or Ready state.

**Fresh validation.** Local baseline and final each pass **1,095**, zero failed/ignored;
actual executed-name Counters, preserving duplicates and expected-panic names,
match both the preceding local checkpoint and exact repaired-source native and
aarch64 CI units. All **134** guard/family/parser/fixture controls execute in
those native contexts, including all 25 removal and two repair controls. Guest
and full x86_64 execution have not yet been verified. Final output is **1,342
lines / 92,102 bytes**, within unchanged 1,400-line / 95,000-byte limits and floor
440. Fresh fmt/clippy lint, frozen Unicode, provenance over 506 tracked files,
family/CI/shared-agent contracts, whitespace and retained ownership-red patch
applicability pass. This checkpoint changes only the two existing handoff/review
documents; source/tests/workflows/caps/pins remain byte-identical. No local host
oracle, mount or new CLI verdict is claimed. Ignored audit scripts/logs/results
are in `tmp/casefold-removal-final-review`; committed tests and exact CI artifacts
provide reproducible evidence for another workspace.

**Next bounded task:** inspect `3964888`'s still-running x86_64/whole-guest full
CI jobs and their actual artifacts. Resolve any failures; verify all retained
controls and the pre-existing ignored slow fuzz gate before closing the standalone
removal-family review. Only after that review may the next checkpoint audit public
setxattr staging/marker/feature-counter behavior and prepare executed red controls;
do not migrate or generalize its boundary in this evidence checkpoint. Keep the
mixed-layout, insertion aliases, ownership/mapping/lifecycle/seal/admission,
collision/growth/recovery and ignored-fuzz gates open. Stage 5 is **not yet
quantified**. Casefold writes and real-card/Windows/SteamOS use remain gated.

## Removal Linux fixture repair checkpoint — 2026-10-18

Started from clean fetched/published `499c1b4898cb3b7046c148802e9cf164c27d1fd8`.
Its [focused workflow](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39190216770)
**failed**; [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39190216678)
was still running during this investigation. Standalone unit, aarch64, provenance,
CLI, wasm, semver, MSRV and fixture jobs passed; this is not full source approval.
Existing Git/helper and Bearer API access and actual artifact download work;
no environment configuration, secret, sibling or dependency pin/lock changed.

**Actual failure.** The one exact removal test reports zero passed / one failed,
zero ignored. Two Rust 1 KiB/128-byte direct/journal profiles completed independent
Linux readback and e2fsck. The third profile (Rust 1 KiB/checksummed256/direct)
completed removal, allocation/refcount/sector/dirty-state assertions, e2fsck and
all nine debugfs records, then failed Linux's exact timestamp comparison on
inode 20, `/zero`. The remaining thirteen profiles did not execute. The failure
report's ordinary stat representation omitted the differing nanosecond values;
no exact kernel tuple was captured and none is invented here.

The fixture had only changed `extra_isize` to zero over a former extended inode.
Its saved input/prepared/dirty/final bytes retain old words `0x41cdb400` at offsets
132 and 148 and `0x6ad583d5` at 144 despite declaring no extra storage. The driver
and debugfs expose only base timestamps in that shape; Linux's exact timestamp
comparison disagreed. A native control built from those captured words executed
red under the old two-byte-only fixture construction before this repair.

**Bounded repair.** A shared cfg(test)-only fixture constructor clears the entire
unused region after byte 128, preserving base inode bytes including data mapping,
external pointer, mode/IDs and generation. It refuses short input, impossible
extra lengths and live inline attributes before mutation. The caller recomputes
the low inode checksum and uses the existing fixture write path. The VM and
native zero-extra removal fixtures now use the same coherent construction;
independent e2fsck must still accept VM input before removal. The two native
fixture controls cover the captured stale extension and refusal preservation.
Linux still compares exact nanosecond tuples and exact xattr/data/metadata
results across all sixteen planned profiles; no assertion, filter, floor, cap,
artifact or format gate is relaxed. Failed readback now saves stdout/stderr
before asserting and prints both exact timestamp tuples and the target name.
Production dispatch/planning/normalization/commit and timestamp parsing/writing
are unchanged. This repairs the test profile; **the original mixed zero-extra /
nonzero-extension layout remains unqualified**, with its precise Linux semantics
requiring separate black-box investigation. Do not infer generic zero-extra
compatibility from this narrower fixture or alter production timestamp rules to
make the old fixture agree without that evidence.

**Artifact audit.** Downloaded and verified the actual focused zip against its
GitHub SHA-256 digest. All five retained metadata filters passed. Read-only saved
bytes and independent report review cover all **68 retained profiles / 136 final
inode records**: four private chmod, eight public chmod, sixteen chown,
twenty-four utimens and sixteen flags. All 96 timestamp and 96 flag phase answers
in each debugfs/Linux stream agree; the other 56 retained records agree too.
The partial removal evidence covers **21 mutations, 27 debugfs records and 18
completed Linux records** over three saved four-phase image sets; the failed
profile has nine missing Linux records and is not graded as passed. Free counts,
pointers, sector/refcounts, surviving values/data and available checksums agree
in those saved bytes. The allocator legitimately reused an earlier freed block
for COW, so final allocation review accounts for that later owner. No new whole-
filesystem or power-loss verdict is claimed. Logs/audit scripts/results are under
ignored `tmp/casefold-removal-review`; committed tests and exact CI artifacts are
the reproducible evidence for another workspace.

**Fresh validation:** baseline **1,093** and repaired **1,095** native tests
pass with zero failed/ignored. The full executed-name Counter retains all 1,093
occurrences, including duplicates/expected panics, and adds exactly the two
fixture controls. Final output is **1,343 lines / 92,162 bytes**, within unchanged
1,400-line / 95,000-byte limits and floor 440. Fresh fmt/clippy lint, frozen
Unicode, family/CI contracts, shared-agent instructions, provenance over 506
tracked files, both embedded guest Python syntax checks, whitespace and retained
ownership-red patch applicability pass. Tracked hashes prove all production
files, workflows, caps and pins unchanged; the guarded module only adds a
cfg(test) module declaration. No fresh CLI/VM verdict is claimed for this repair;
the source's installed-CLI CI job passed. Repaired exact-commit full/focused CI
and standalone family review are pending after publication.

**Next:** inspect repaired exact full/focused CI and actual logs/images/reports,
resolve any remaining failure, verify all sixteen profiles and retained oracles,
then finish the standalone removal-family implementation/evidence review before
another migration. Keep the ignored slow fuzz qualification gate, insertion
aliases, full ownership/mapping/lifecycle/seal/admission/collision/growth/recovery
work open. Stage 5 stays **not yet quantified**; casefold writes, real-card use,
Windows and SteamOS qualification remain gated. No host oracle/mount was run.

## Guarded public xattr-removal implementation checkpoint — 2026-10-18

Started from clean local/fetched `65d99d75dd46336d9cf1a7b6f6d98f3d6471184a`.
Git initially lacked its credential helper, and the GitHub CLI's default API
header returned HTTP 401. Reusing the existing gh helper for Git and existing
`GH_TOKEN` binding with an explicit `Authorization: Bearer` header for API HTTPS
requests restores access; never print/copy its value. Remote freshness and the
audit commit's [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39132280544)
are now verified. No new credential, network destination or environment draft is
needed. Existing checkouts, toolchain/core source view and pins were reused.

Existing Rust string/byte and C path removexattr now enter the same metadata
mount context as chmod/chown/utimens/flags, before lookup or device/runtime
callbacks. The unchanged removal logic is extracted into a private owned
BlockBuffer plan. Planning errors discard it without physical marker writes or
logical allocation publication, preserving existing namespace/error priorities.
The private whole-buffer boundary checks context identity, validates any staged
primary SB shape/magic, clears only its VALID_FS bit and restamps its checksum.
It preserves planned counters, other state bits and all unrelated bytes. It
never fetches a replacement SB or adds one to an inode-only buffer. Marker-once
and the entire raw commit/checkpoint/flush/allocation publication execute inside
guarded effects. Journal RECOVER handling and global raw commit semantics are
unchanged. Stale public removal comments now describe external/shared support.

Fresh baseline passes 1,068 with zero failed/ignored. The saved red patch was
applied and actually rerun before implementation: **one pass, nine failures**,
with all intended assertions reached. Its five public failures are promoted
unchanged; its four constructed counter-order diagnostics now test the new
boundary, rather than changing the raw primitive. The old patch is historical
and applies at `65d99d7`/the audit source, not on top of the promoted module.
The passing legacy four-profile control remains. Native control names now live
under `fs::guarded_chmod::tests::removexattr_calls`.

Native controls cover normalization at 1/4 KiB, clean/prior-dirty state, preserved
counter edits and other bytes/state bits, inode-only buffers, malformed SB shape/
magic, foreign context, missing path/name and namespace priorities, volume gates,
C invalid-UTF-8 path/error clearing, all five families' Rust/C worker reentry in
both directions, every observed write/flush fault, planning/effect/publication
unwinds, failure propagation and actual C close. Workers are joined before C
handle release; panic injection uses Rust-owned runtime/device callbacks, not
foreign callback unwind. Synthetic publication-lock poisoning verifies the whole
post-durability publication is inside the effect even if its context is retained.

Independent evidence is **added, not yet executed**:
`fs::guarded_chmod::needs_host::removexattr::linux_checks_public_removexattr_metadata`.
Its sixteen explicit profiles cover Rust/C × 1/4 KiB × legacy128/checksummed256
inodes × direct/journaled commits. Seven target removals and two unchanged peers
per profile cover unique rewrite, last-block free, shared COW and shared release;
256-byte profiles additionally cover inline, a constructed legacy duplicate and
zero-extra removal. Linux creates the base images/data/xattrs. Shared pointers
follow the existing MIT test fixture pattern and must pass independent e2fsck
before mutation; the duplicate seed is explicitly constructed afterward.
The test checks dirty-before-close and clean-after-close state/checksum,
allocation bits/free counts, pointer/sectors/refcounts, metadata/ctime/generation,
unchanged sharing inode bytes/values, debugfs metadata/xattr lists, Linux exact
xattr sets/values and data, close/reopen and non-repairing e2fsck verdicts.
Focused CI adds its exact filter, one-test floor, image/report/log artifacts and
separate 2,100-line/130,000-byte provisional bounds, retaining all five prior exact
metadata filters and their floors/caps/artifacts. Check actual output measurements
when it can run. Embedded guest Python parses and Rust compiles in local lint;
that is not a Linux verdict.

**Fresh validation:** all **1,093** unit occurrences pass, zero failed/ignored,
retaining the complete 1,068-occurrence baseline Counter and adding exactly
**25 removal controls**. Actual branch coverage executes 40 direct/journal,
block-size and Rust/C combinations, including shared COW/release, unique rewrite,
last-block free, duplicate and zero-extra cases in their supported inode shapes.
Fault controls inject every observed write/flush in each of those prepared
branches and assert sticky five-family/flush/C-close refusal. The initial enhanced
suite passed 1,092 but its **1,340 lines / 91,776 bytes** exceeded the old byte cap;
the unit byte cap is measured up to 95,000, with the 1,400-line cap and floor 440
unchanged. Final output is **1,341 lines / 91,908 bytes**, within those limits.
Fresh fmt/clippy lint and the locked release CLI installation pass. The final
retained ownership-red patch applies; frozen Unicode, family/CI contracts,
shared-agent and whitespace checks pass. Final tracked provenance passes over
505 files. Input/prepared/dirty/final images and branch observations are included in the new Linux artifact plan. Local evidence lives under ignored
`tmp/casefold-xattr-removal`; it is not a substitute for CI artifacts.

**Publication/CI:** this checkpoint is prepared for non-forced publication using
existing GitHub access. Verify its exact remote SHA and full/focused CI after
publication; no new independent verdict or standalone family review is claimed
at commit preparation.

No host oracle or mount was run; this instance has no `/dev/kvm`. Full/focused
CI and artifact review for this exact source, followed by standalone removal-
family review, remain required before another family migration. The existing
ignored slow fuzz gate in broader CI remains unresolved. No casefold write
permission, format support, owner/seal/admission/lifecycle guarantee, sibling,
dependency pin/lock or Windows/SteamOS/real-card qualification is changed.

**Next:** verify this checkpoint's publication and exact full/focused CI using
the existing binding's Bearer API route; inspect actual images/reports, repair
any failed evidence, then review the removal family. Keep all write gates closed. Do not start another metadata migration before those gates.

## Xattr-removal staging/marker audit checkpoint — 2026-10-18

Started from clean fetched `a387b90340f3edfeb80ed8d7e9f3be022276008d`;
its [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39119479035)
passed. Source `4d4df81`'s full/focused evidence and flag-family review remain
applicable. Reused the prepared Rust/core source view and existing GitHub API
binding. This checkpoint changes the two existing handoffs and adds one dormant
red patch; no production, active test, workflow, dependency pin/lock, sibling,
environment configuration or casefold write gate changes.

The final executed patch run reports **one passed / nine failed / zero ignored**.
Five failures exercise current public behavior: Rust string/byte and C path
removal bypass the held metadata context, and Rust/C missing-attribute errors
still write/flush one marker. Four separate constructed cases snapshot a clean
primary superblock, stage a synthetic free-count decrement, and call existing
marker/raw commit routines: direct checksummed ext4 and journaled legacy ext3,
each at 1 KiB and 4 KiB. In all four, state becomes **1** again after commit,
with marker owed and the planned counter edit preserved; checksum-enabled
profiles remain checksummed correctly. These are controlled ordering diagnostics,
not complete valid allocation transactions, independent oracle verdicts or the
current public removal order. The passing control exercises actual last-external-
block removal in all four profiles: pointer/sector/free-count updates succeed
and dirty state remains until close. An initial fixture-size failure was corrected
to single-group 8 MiB at 1 KiB; final 4 KiB fixtures remain 32 MiB.

Saved [CASEFOLD-XATTR-REMOVAL-RED.patch](CASEFOLD-XATTR-REMOVAL-RED.patch) contains
all ten controls and apply/run/reverse instructions. It was reversed after the
actual run; every production/active-test/workflow/dependency hash matches the
starting source, and `git apply --check` succeeds. No active failing/ignored test
was added. Preserve the five public reds for the next fix; turn the constructed
cases into green tests of the new private guarded whole-buffer boundary rather
than changing the global raw commit primitive to accept the constructed order.
Ignored `tmp/casefold-xattr-removal-audit` retains actual red source/log, the
fixture failure, observations, source hashes and baseline/final validation.

**Design decision and next implementation:** read
[the detailed staging audit](CASEFOLD-INDEX-REVIEW.md#xattr-removal-staging-and-recovery-marker-audit--2026-10-18)
before changing public removal dispatch. Acquire the existing shared context
before callbacks/lookup, extract the existing full removal plan privately and
preserve its namespace/error/layout semantics. A private whole-buffer boundary
normalizes an already-staged primary superblock's VALID_FS bit and checksum
before effects, preserving every planned counter/unrelated byte. Never overwrite
planned data with a fresh disk/snapshot copy or repair the dirty marker after
commit. Then cover marker-once and the entire existing commit/checkpoint/flush/
allocation publication in guarded effects. Keep the journal writer's separate
RECOVER handling, current C lifetimes and setxattr semantics intact. Inode-only
buffers need no new superblock staging; no new lock, Ready state, owner proof/
cache/publication, seal, admission or write-gate change is authorized here.

Source audit covers inline and duplicate removal, external rewrite, shared COW,
shared release and last-block free, including zero-delta staged SB counters.
Prospective uninit clears stay buffer-local until successful commit; planning may
memoize already-committed allocation state without publishing proposed clears.
The full commit effect must contain even post-durability publication panics.
The detailed audit records zero-extra/128-byte namespace priorities, whole-entry
EA-inode/corruption refusals and stale public comments to correct during migration.

**Fresh validation:** baseline and restored final suites each pass **1,068**,
zero failed/ignored, with identical executed-name Counters including repeated
names and expected panics. Final output is **1,316 lines / 88,904 bytes**, within
unchanged 1,400-line / 90,000-byte caps. Fresh lint passes fmt/clippy, shared-agent
and family/CI contracts, provenance over 503 tracked files, and frozen Unicode
checks. Both the new removal patch and retained ownership-red patch apply
cleanly; tracked source/build/workflow hashes and whitespace checks pass.
No host oracle/mount or forbidden source was used; no local VM, new independent
Linux evidence or fresh CLI build is claimed for this audit. Source CI above validates the unchanged implementation;
this documentation/patch checkpoint's own CI remains pending after publication.

**Next:** implement only existing Rust string/byte and C path removexattr under
that reviewed boundary, with five-family failure/callback/close controls and
exact independent Linux evidence for all stated xattr branches. Retain all five
previous exact metadata oracles, floors/caps/artifacts and baseline tests, then
review exact source CI/artifacts and the removal family before another migration.
Stage 5 stays **not yet quantified**. Ignored fuzz, insertion aliases, ownership/
lifecycle/seal/admission and complete mapping/collision/growth/recovery remain
open; casefold writes and real-card/Windows/SteamOS qualification stay gated.

## Public inode-flag implementation/evidence review — 2026-10-18

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

This review changes only the existing handoff and design review. Source/tests,
workflows, dependencies/locks, siblings and environment configuration stay
unchanged. Fresh baseline and final full native units each pass **1,068**, zero
failed/ignored, with identical complete Counter multisets; the final log is
**1,315 lines / 88,844 bytes** within unchanged caps. Fresh fmt/clippy lint,
frozen Unicode and family/CI contract checks, shared-agent instructions, provenance
over 502 tracked files, saved insertion-red patch applicability and whitespace
checks pass. All reviewed source/workflow/header hashes remain unchanged. No
local VM run or fresh release CLI build is claimed for this documentation review;
exact source CI above supplies independent VM/build evidence.
Ignored `tmp/casefold-flags-review` retains verified archives, extracted reports,
whole-image audit script/results and CI/control Counter evidence. Fresh chats
must retrieve missing artifacts again. This documentation checkpoint's own CI
remains pending after push.

## Public inode-flag shared-guard checkpoint — 2026-10-18

Started from clean fetched `520e2d3fe9d4e9b1b6063ba184dd0e06f9efe888`.
That timestamp-review documentation HEAD passed its exact
[full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39088805049).
The underlying `f8649b7` passed exact full/focused CI and its standalone family/
evidence review. Existing Git/API pull/push access and prepared pinned Rust/core
source view remain usable; no environment draft, sibling, dependency pin/lock,
main branch or casefold write gate is changed.

**Implemented scope:** existing Rust string/byte `apply_set_flags` and the
existing C path delegate now enter the same mount context as chmod/chown/utimens,
before lookup and device/runtime callbacks. One context is reused through verified
inode planning, buffer preparation, guarded clean-marker and complete commit
effects. No new inode API, lock, ABI, C reference creation or lifetime rule is added.
The extracted raw planner retains the exact changed-bit comparison against
`USER_MODIFIABLE_FLAGS`: already-set managed bits may be passed back unchanged;
changing CASEFOLD, INDEX, EXTENTS, INLINE_DATA and other managed bits remains
refused. Missing path still wins over a bad flag word. Same-flags calls still
update ctime and commit, unlike both-omitted timestamps. Current modes, owner IDs,
generation, data, precision and checksum behavior remain. Refused planning now
leaves clean/prior-dirty bytes, marker and operation versions untouched; failures
and Rust unwinds share sticky write/flush/drop retirement. Other operations and
C caller lifetimes remain externally coordinated; no whole-API concurrency,
owner readiness/cache/publication, builder, seal or admission is added.

**Actual red-to-green:** four new controls executed failing first: Rust string,
Rust byte and C path calls bypassed a held mount context, and a refused managed-
flag change still wrote the clean marker. The same four pass after routing.
Expanded coverage adds **16 controls** and retains all 71 prior mount-family
controls; the **87-control** mount group passes with zero failed/ignored. Tests
cover same-flags ctime/versions, restoring flags, already-set managed bits, every
managed-bit refusal, missing-path/volume priority, invalid UTF-8 bytes, clean/
prior-dirty planning refusals, foreign contexts, prior-family failures and C
error/success clearing. Rust/C callback workers cross all four families in both
directions, own/join handles before C release, and defer timeout assertions until
the outer guard is released. Every observed direct/journal marker/commit write
and flush fault is reached and retires all families; planning/effect Rust panics
and actual exported C close preserve clean or owed-dirty state. No foreign C
callback unwinds. Arbitrary callback termination is not guaranteed.

The first expanded run caught a wrong NOATIME value in the new positive fixture;
the managed-bit policy correctly refused it. The fixture now uses declared flag
constants. Corrected an existing C API comment from `0x200` to NOATIME's `0x80`;
no constant, flag policy or public signature changed.

**New independent evidence contract:** exact test
`fs::guarded_chmod::needs_host::flags::linux_checks_public_flags_metadata`
plans **16 profiles**: Rust/C × 1 KiB/4 KiB × 128-byte legacy/256-byte checksummed
inodes × direct/journal writes. Linux creates known file/directory metadata and
payload. Held-context and managed-bit refusals preserve whole-image bytes before
any marker. Actual public delegates set NODUMP/NOATIME, repeat the same flags with
a later clock, then restore the original word. Each phase finishes the driver
mount before debugfs and Linux readback. Debugfs checks full flags, mode, IDs,
generation and all times; Linux independently checks full flags through its public
GETFLAGS ioctl, inode identity, mode/IDs, exact times and payload. Its 64-bit ioctl
buffer assumption is asserted. O_NOATIME preserves readback timestamps. Judged
e2fsck checks input and every emitted/readback phase. Expected artifacts are
**16 images, 96 debugfs reports and 48 Linux reports**, plus the exact log with
a one-test floor. All four previous exact metadata steps, floors/caps/artifact
entries are byte-identical. This host lacks the required VM/KVM, so the new Linux
oracle has not executed locally; source CI and a standalone family review remain
required before another migration.

**Fresh local validation:** baseline **1,052** passed, zero failed/ignored. Four
actual initial controls then failed and pass after routing; expanded mount controls
pass 87. Final full native units pass **1,068**, zero failed/ignored, retaining all
**1,052** baseline occurrences as complete Counter multisets and adding exactly
the sixteen flag controls. The log is **1,316 lines / 88,904 bytes**, within
unchanged 1,400-line / 90,000-byte caps. Fresh fmt/clippy lint, frozen Unicode,
family/CI contract and shared-agent checks, provenance over 502 tracked files,
guest Bash/Python syntax, previous exact-workflow retention, saved insertion-red
patch applicability and whitespace checks pass. The locked release CLI build/
install also passes. Exact source full/focused CI remains pending after push and
standalone flag-family review is still required. Ignored `tmp/casefold-flags`
retains baseline, actual four-control red source/log, green and expanded logs,
the corrected fixture failure, guest syntax checks and final validation evidence.
No test tier budget, dependency or write gate changed.

**Next:** verify this source's exact full/focused CI, investigate any failure,
download/audit all sixteen flag profiles and retain all four prior metadata
oracles, then complete the standalone family review before another migration.
Stage 5 remains **not yet quantified**. The ignored slow fuzz qualification,
three insertion aliases, lifecycle/ownership/admission/seal dependencies and full
mapping/collision/growth/recovery work stay open. Casefold writes and real-card/
Windows/SteamOS qualification remain gated.

## Public timestamp implementation/evidence review — 2026-10-17

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

This review changes only the existing handoff and design review; source, tests,
workflows, dependencies/locks, siblings and environment configuration are unchanged.
Fresh baseline and final native units each pass **1,052**, zero failed/ignored,
with identical complete Counter multisets; the final log is **1,299 lines /
87,047 bytes** within unchanged caps. All reviewed source/workflow hashes
remain unchanged. Shared-agent instructions, provenance over 500 tracked files,
saved insertion-red patch applicability and whitespace checks pass. Fresh
fmt/clippy lint, including frozen Unicode and family/CI contract checks, passes. No local VM or fresh release CLI build is claimed for this
documentation-only review; exact source CI above supplies independent VM/build
evidence. Ignored
`tmp/casefold-utimens-review` retains verified archives, extracted text reports,
whole-image audit script/results, CI/control Counter checks and local logs. A
fresh environment must retrieve artifacts again rather than assume these files
exist. This documentation checkpoint's own CI remains pending after push.

## Timestamp oracle display repair — 2026-10-17

Started from clean fetched `5dd65f931d6f8c2082f17d3a3778e9a7d563af7c`.
Its [focused CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39078002510)
and [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39078002498)
failed on the same short-extra-12 timestamp report assertion. Both full-suite
library contexts ran 746 passing tests and one failing new timestamp oracle;
they did not complete the full suite. Standalone/native, ARM, fixture, semver,
wasm, provenance, CLI and MSRV jobs passed. This is failed independent validation,
not a completed timestamp review. Prior documentation `719f66c` now passed its
[exact full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39076664949).

Downloaded the actual focused and two failing tier artifacts and verified their
GitHub SHA-256 digests. The first two legacy timestamp profiles passed. On the
first short-extra profile, debugfs `stat` printed only signed legacy seconds,
without any extra timestamp words. The captured file report shows mtime
`0x8000006e` and ctime `0x8000000a`, although a read-only public-format byte audit
of that emitted inode finds full values `(2147483758, 987654321)` and
`(2147483658, 0)` at extra-isize 12. No host oracle or mount was used.

Moved the existing test-only timestamp report decoder into a native-testable
module. A control using the actual report first reproduced the epoch/nanosecond
mismatch and now passes. Only the observed extra-isize-12 profile expects the
legacy display. Two further controls retain complete normal-extra values,
reject an omitted normal extra word and catch incorrect stored seconds in
legacy/short displays. No production timestamp packing or write path changed.

The exact 24-profile VM test now also invokes Linux immediately after explicit
updates, before sentinel updates overwrite any values. It asserts unprojected
seconds/nanoseconds for all three times, mode/type, identity, IDs and payload,
using O_NOATIME for data. The original final Linux check and all debugfs/e2fsck,
no-op, exclusion and generation assertions remain. Every driver mount is finished
before external access. Expected artifacts now include **24 images, 96 debugfs
reports, 24 explicit Linux reports and 24 final Linux reports**. All previous
metadata oracles, workflow upload paths, test floors and output caps are unchanged.
Inode generation remains checked through debugfs, not Python stat.

**Fresh local validation:** baseline **1,049** passed; the captured report control
then failed and passes after correction. All three report controls pass. Final
full native units pass **1,052**, zero failed/ignored, retaining all **1,049**
baseline occurrences as Counter multisets and adding exactly those three
controls. The log is **1,300 lines / 87,107 bytes**, within unchanged
1,400-line / 90,000-byte caps. Embedded guest Python syntax, shared-agent block,
provenance over 500 tracked files and whitespace checks pass. Fresh fmt/clippy lint, including frozen Unicode and family/CI contract checks,
passes. The release CLI was not rerun for this test-only change; its
unchanged production source passed that job at `5dd65f9`. No local VM run is
claimed; this cloud host lacks the required VM/KVM. The repaired exact full/focused CI
and standalone timestamp implementation/evidence review are still required.
Ignored `tmp/casefold-utimens-ci-repair` retains original verified artifacts,
failure logs, the captured-image byte audit, actual red/green controls and fresh
validation evidence. Production writes, dependencies/locks, siblings, environment
configuration and every casefold gate remain unchanged.

**Next:** verify repaired exact-commit full/focused CI, investigate any remaining
failure, audit all 24 timestamp profiles and complete the standalone family
review before another migration. Stage 5 remains **not yet quantified**. The
ignored deep-fuzz qualification, insertion aliases, lifecycle/admission and full
mapping/collision/growth/recovery work remain open; real-card/Windows/SteamOS
qualification and ordinary casefold writes remain gated.

## Public timestamp shared-guard checkpoint — 2026-10-17

Started from clean fetched `719f66cf9765656f3627b7bdcdabab01114fd646`.
The reviewed underlying source `198baba` passed exact full/focused CI and the
standalone chown evidence review; the documentation-only `719f66c` full CI was
still running at this checkpoint's opening. Reused existing Git/API pull/push
access and the prepared Rust/core source view. No sibling, dependency pin/lock,
environment draft, main or existing casefold write gate changed.

**Implemented scope:** public Rust string/byte and generic inode timestamp
updates, including the existing C delegates, enter the same mount context as
chmod/chown. Entry precedes device/runtime callbacks, lookup and conversion;
volume and explicit time verdicts precede arbitrary conversion. Paths retain
invalid-time-before-missing-path priority and reuse the context for inode planning.
The old raw timestamp planner is shared, returning either an inode update or
a validated no-effect plan. One clock value still serves NOW and ctime, including
the existing one-read planning behavior for both-omitted fields. Full buffers
are prepared before guarded marker and complete commit effects.

`TIME_OMIT`, `UTIME_OMIT` and `UTIME_NOW` precedence, signed seconds, nanosecond/
range checks, extra-field precision/refusals, ctime packing, generation checks
and mode/owner/data preservation remain. Both omitted fields validate the volume
and live inode/generation and leave inode, clean/dirty state and operation
versions unchanged, with no marker or commit. Invalid-time checks and generic
conversion now precede the clock and all marker I/O; this intentional planning
ordering prevents arbitrary conversion on rejected inputs. No C ABI/reference
creation, permission policy or timestamp format was changed. Other API families
and caller-owned C lifetimes remain externally coordinated; there is no whole-
API concurrency or owner-readiness claim.

Four actual native controls failed first: public Rust/C accepted timestamp
updates under a held context, conversion ran before overlap refusal, and both
omitted fields wrote/flushed the clean-mount marker. The same four controls now
pass. Nineteen new controls retain the previous 52 family and two parser controls.
The expanded mount group passes **71**, zero failed/ignored. The new controls
cover explicit pre-1970/upper-range seconds and nanoseconds, mixed NOW/OMIT and
both-omit precedence; 128-/256-byte and short-extra 0/8/12/16 layouts; unchanged
mode/IDs/generation/data; validation/missing/stale/volume order and clean/dirty
no-op bytes/versions; conversion/unwind; Rust/C callback workers in both
directions across all three families; every observed marker/commit write and
flush fault on direct/journal mounts; Rust planning/effect panics and C errors;
later cross-family/no-op/flush refusal; and actual exported C close preserving
clean or owed-dirty state after caught planning panic. Workers retain owned join
handles, defer timeout assertions until guard release and join before C handle
release, including cleanup paths. No foreign C callback unwinds.

New exact test
`fs::guarded_chmod::needs_host::utimens::linux_checks_public_utimens_metadata`
covers **24 planned profiles**: Rust/C × 1 KiB/4 KiB × 128-byte legacy /
256-byte short-extra-12 / 256-byte normal-extra-32 × direct/journal writes.
Linux creates known file/directory modes, IDs, generation, payload and times;
debugfs constructs the short-extra profiles and judged e2fsck must accept input.
Actual public delegates perform explicit and mixed-sentinel updates, saving
debugfs reports for both phases and checking full seconds/nanoseconds, metadata
and generation. Fresh mounts exercise both-omitted path/inode no-ops with whole-
image equality and no version/marker advance. Linux then must agree on exact
final times, identity, type/mode, IDs and payload; O_NOATIME preserves the times
while reading data. Each phase is finished before oracle access/reopening, so
independent readers never overlap a live writer. Judged e2fsck checks emitted
phases and final Linux readback. Expected artifacts are 24 images, 96 intermediate
debugfs reports, 24 Linux reports and the exact log, with a one-test floor.
All three existing exact metadata tests, floors and artifacts remain unchanged.
This host lacks the required VM/KVM; the new independent test has not run locally.

Fresh baseline passes **1,030**, zero failed/ignored. All four red controls then
pass after routing, and expanded focused controls pass as above. A path-type
mismatch in the new VM helper was caught by compilation and corrected; fresh
lint passes. Final full local units pass **1,049**, zero failed/ignored, retaining
all **1,030** baseline occurrences as Counter multisets and adding exactly the
nineteen new controls. The final log is **1,297 lines / 86,809 bytes**, within
unchanged 1,400-line / 90,000-byte caps. Fresh fmt/clippy, provenance over 499
tracked files, frozen Unicode checks, shared-agent instructions, guest shell/
Python syntax, saved insertion-red patch applicability and whitespace pass.
The locked release CLI build/install passes. Ignored `tmp/casefold-utimens`
retains actual red/green logs and patch, focused controls, complete Counter
retention, full local/lint/CLI logs and source hashes. Exact-commit full/focused
CI is pending after push; no local VM or standalone timestamp review is claimed.
Existing tier caps/floors are unchanged.

**Next:** verify this source's exact full/focused CI, download the actual 24-profile
timestamp artifacts, and review the family before migrating another. Also verify
the preceding documentation checkpoint's full CI result. The ignored deep-fuzz
qualification, three insertion aliases, full mapping/collision/growth and recovery/
admission dependencies remain open. No Ready state, owner builder/cache/publication,
buffer seal or transaction admission is added. Stage 5 is **not yet quantified**;
all public casefold writes and actual-card/Windows/SteamOS qualification stay gated.

## Public chown implementation/evidence review — 2026-10-17

Reviewed `198baba2e7b31bdac28e3e814812040f2ced5d5d`, equal to fetched
`origin/casefold-rw` with a clean tree. Its exact-commit
[full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39005643503)
and [focused Linux evidence](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39005643486)
passed. Actual downloaded native logs match the fresh baseline as Counter
multisets, including expected-panic tests and repeated occurrences:

| Context | Chown / parser / retained chmod controls | Enclosing result | Lines / bytes |
|---|---|---|---|
| Standalone unit | 16 / 2 / 36 | 1,030 passed, zero failed/ignored | 1,286 / 85,108 |
| x86 host unit | 16 / 2 / 36 | 1,030 passed, zero failed/ignored | 1,280 / 84,671 |
| aarch64 unit | 16 / 2 / 36 | 1,030 passed, zero failed/ignored | 1,280 / 84,671 |
| Compiled guest library | 16 / 2 / 36 | 727 passed, zero failed/ignored | Within guest log |

These are 54 distinct controls in four contexts, not 216 distinct tests.
The whole guest suite reports **1,855 passed, zero failed, one ignored**,
3,408 lines / 181,700 bytes. The only ignored case is the pre-existing slow
`fuzz_smoke::deep_walk_byte_flip_fuzz_never_panics`; broader qualification
remains gated. Every family/parser control and exact metadata oracle executed.
The new standalone unit measurement also confirms the parser controls legitimately
grew past the old 85,000-byte cap; the current 90,000-byte / 1,400-line caps pass.
No budget/floor is changed in this review.

All three exact metadata tests execute in the guest and as focused one-test
steps, each one passed, zero failed/ignored:

| Exact evidence | Profiles | Lines / bytes | Tool / kernel markers |
|---|---|---|---|
| Public Rust/C chown | 16 | 145 / 18,870 | 96 / 32 |
| Retained public Rust/C chmod | 8 | 81 / 9,553 | 48 / 16 |
| Retained private chmod | 4 | 49 / 4,960 | 24 / 8 |

Verified the actual focused artifact digest against GitHub's reported SHA-256;
retained its compressed archive instead of expanding unrelated images. Audited
all 28 relevant images, 56 debugfs reports and 28 Linux reports. All 56 emitted
inode records agree on identity/generation, mode/type, full uid/gid, exact ctime
and preserved mtime. The owner profiles cover Rust/C, 1 KiB/4 KiB blocks,
128-/256-byte inodes and direct/journal writes. All 32 high-bit group reports
normalize from signed decimal to the correct full stored bits; generation stays
unsigned. Linux asserts preserved input atime/mtime and file payload, and judged
e2fsck accepts input, emitted close and Linux readback on every profile.

A separate public-format byte audit verifies all 28 clean superblocks and no
pending recovery flag, current inode/data allocation, payloads, uid/gid halves
and ctime bases/epochs. It additionally verifies 20 enabled superblock, 30 distinct
descriptor and 40 inode checksums. Legacy 128-byte profiles have no metadata
checksum; 256-byte profiles do. Directory atime agrees; Linux's final file read
can update file atime/checksum after the saved pre-read debugfs report, so that
expected difference is accounted for. No host oracle, host mount or forbidden
implementation source was used. Scratch healthy images do not qualify power loss,
whole-API concurrency or actual-card/Windows/SteamOS behavior.

**Decision: no blocker found within the public chown caller contract.** Both
families acquire the same borrowed mount context. Generic inode conversion is
inside entry after verdict-only volume checks; path/inode planning shares one
context. Full owner/sentinel/ctime/checksum planning precedes marker and complete
buffered commit effects. Both-unchanged owner inputs retain their existing ctime
update/commit semantics. Missing/stale and volume refusals preserve bytes and
healthy state; marker/commit errors and Rust planning/effect unwinds latch common
write/flush/close refusal. C errors, success clearing and `FS_EXT4_GEN_ANY` versus
strong generation zero remain unchanged. Actual exported close is exercised
after caught planning panics on both a fresh clean mount and an owed dirty marker.

New callback workers keep owned join handles; their timeout result is recorded
inside the callback and checked after outer guard release and joins. Cleanup is
inspected to join before C handle release; successful worker paths executed.
This is not a guarantee that arbitrary uncooperative callbacks terminate. The
production C reference creation, ABI and caller lifetime/aliasing obligations
are unchanged. Other mutators/readers/maintenance/raw fields and devices remain
externally coordinated. Arbitrary argument destruction, swallowed callback errors,
foreign unwind/panic-abort and callback-duration/fairness guarantees remain outside
the ordering claim. No Ready state, owner builder/cache/publication, buffer seal
or transaction admission is added, and public casefold writes remain blocked.

Fresh baseline and final local units both pass **1,030**, zero failed/ignored,
with identical Counter multisets and no added/lost occurrences. The final log is
**1,277 lines / 84,565 bytes**, within unchanged caps. Fresh fmt/clippy, provenance,
frozen Unicode checks, shared-agent instructions, saved insertion-red patch
applicability and whitespace pass. Only the existing handoff/review documents
change; source/test/workflow hashes stay unchanged. The source's CLI/semver/wasm/
MSRV CI passed and no fresh release CLI or local VM run is claimed. This
documentation checkpoint's own CI remains pending after push. Ignored `tmp/casefold-chown-review` retains the verified artifact,
actual logs, image audit/hashes, scoped CI comparisons and local validation.

**Next bounded implementation: guarded timestamp updates (`utimens`) only.**
Route existing public Rust string/byte/inode and C path/inode delegates through
the same mount context and sticky failure bridge as chmod/chown. Preserve
`TIME_OMIT`, `UTIME_OMIT` and `UTIME_NOW` precedence, ordinary nanosecond/range
checks, one-clock planning semantics, generation checks, existing inode extra-
field precision/refusals and ctime encoding. Both omitted times still validate
the live target, leave inode/ctime unchanged and produce no marker or commit;
all planning refusals precede effects. Preserve invalid-time-before-missing-path
error priority. Require executed overlap/conversion red controls and green Rust/C
cross-family, no-op/sentinel, callback, fault/panic/close and exact Linux timestamp
evidence before review. The detailed scope is in `CASEFOLD-INDEX-REVIEW.md`.
No other operation family or timestamp-format redesign is included. The ignored
fuzz qualification, three insertion aliases, full mapping/collision/growth and
recovery/admission dependencies remain open. Stage 5 is **not yet quantified**.

## Public chown oracle report repair — 2026-10-17

Started from clean fetched `2cedf28c1bca129f95c19f19352b50595976a8dd`.
Its [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39002666402)
and [focused Linux evidence](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/39002666396)
both failed. Downloaded actual logs/artifacts: focused, native x86 library and
compiled guest library all stop at the same new owner-report assertion, with
`ParseIntError::InvalidDigit` on the first Rust/1 KiB/128-byte/direct profile.
The guest and native x86 library each report 724 passed, one failed, zero ignored;
all 52 mount-family controls passed, but the new exact oracle did not complete.
Standalone/x86/aarch64 native units each passed 1,028, zero failed/ignored;
their full passing Counter multisets match this run's fresh local baseline.
Other required jobs passed, including CLI, semver, wasm, provenance and MSRV.
These failures are not the earlier VM/SSH interruption and not budget failures.

The captured debugfs report says `User: 1985229328   Group: -2023406815`.
The group is a signed decimal display of **2271560481 (`0x87654321`)**.
A read-only public-format byte audit of that saved image confirms both full
owner IDs, regular-file mode 06751, generation, clamped ctime and payload.
Both input and emitted-image e2fsck checks had passed, but Linux readback and
the other fifteen profiles had not run. This limited audit does not substitute
for the failed exact test or qualify the driver/family.

**Repair:** factor the original unsigned owner parsing into a test-only helper
used by the exact oracle. The new actual-report native control first executed
red with the same parsing error. The helper now accepts unsigned `u32` displays
or signed `i32` displays reinterpreted as exactly 32 bits; malformed and out-of-
range values fail, with no unchecked wider truncation. Generation still uses
strict unsigned parsing. Both native display/overflow controls now pass.
No filesystem mutation code, public/C ABI, mode/ID/sentinel policy, Linux
assertion, exact test identity, profile, floor or casefold write gate changed.

Two new native controls add output. The prior actual standalone CI artifact
measures **1,284 lines / 84,911 bytes**, leaving only 89 bytes under the old
85,000-byte unit cap. `chores.yml` records that measurement and raises only
the unit byte cap to **90,000** for this legitimate growth; the 1,400-line cap
and every other tier budget/floor remain. This does not suppress an assertion
or mask either CI failure. Fresh full local units pass **1,030**, zero failed/
ignored, retaining all **1,028** baseline occurrences as Counter multisets and
adding exactly the two owner-report controls. All 52 mount-family controls are
retained. The final log is **1,278 lines / 84,625 bytes**, within the new byte
cap and unchanged line cap. Fresh fmt/clippy, provenance over 497 tracked files,
frozen Unicode checks, shared-agent instructions, saved insertion-red patch
applicability and diff whitespace pass. Corrected exact-commit full/focused CI
remain pending after push; the standalone chown family review has not occurred.
No new release CLI build or local VM run is claimed for this test-only repair;
the unchanged production implementation's CLI CI already passed at `2cedf28`.

The older review commit `ab8129a` independently passed
[full CI attempt 2](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38991380517)
at the same SHA. Downloaded its actual guest artifact: **1,836 passed, zero
failed, one ignored**, 3,393 lines / 179,888 bytes. Both exact chmod oracles and
all 36 chmod controls execute. The original VM-loss cause remains unknown;
a completed rerun supplies that missing CI result without establishing a fix.
The historical ignored slow deep-fuzz test is still a broader qualification gate.

Evidence is under ignored `tmp/casefold-chown-ci-followup`: original failure
logs, diagnosis, captured report, byte audit, actual native red/green controls
and local logs. To recover disk room, this run's 69 downloaded image copies
were losslessly gzip-compressed; the archive manifest records original paths,
sizes and SHA-256, verified again by decompressing before replacing each copy.
Existing earlier checkpoint evidence and user files were untouched. The prepared
environment, siblings, pins/locks, main and environment draft are unchanged.

**Next:** verify corrected exact-commit full/focused CI and all sixteen saved
owner profiles, then perform the standalone public chown implementation/evidence
review before another migration. Keep both exact chmod oracles, all 52 existing
family controls and the complete baseline. No owner readiness/cache/publication,
builder, buffer seal or transaction admission is added. Other API paths and
caller-owned C lifetimes remain externally coordinated. Stage 5 is **not yet
quantified**; the ignored fuzz qualification, three insertion aliases, complete
map/collision/growth/recovery work and actual-card/Windows/SteamOS remain gated.
All public casefold writes stay blocked.

## Public chown shared-guard checkpoint — 2026-10-16

Started from clean, fetched `ab8129acc607cc6f637b877a7a69f0fdfe731ca9`.
Reused the existing Git/API access and prepared pinned Rust/core view, without
changing siblings, dependency pins/locks, environment configuration or main.

First investigated that review commit's failed full CI. Downloaded the actual
guest tier and job logs: `test:vm` exited **255**, with Vagrant reporting the
machine unavailable for SSH. The 2,301-line / 131,080-byte log is under the
unchanged 3,900-line / 193,000-byte cap. Completed binaries report 1,413 passed,
zero failed and the existing one ignored fuzz test; another three tests reported
success in the unfinished ten-test `htree_dir_writes_oracle` binary. This is
**an incomplete failed suite**, not a passing baseline or evidence of a specific
driver defect. The cause of VM loss is unknown; neither a timeout nor memory
exhaustion is established by the saved logs. All 36 chmod controls and both
exact chmod metadata tests completed successfully before the interruption.
The separate x86 oracle job passed all ten tests in that binary, and its complete
1,012-unit passing multiset matches this run's fresh local baseline.
Requested only failed-job reruns of run 38991380517 at the same SHA (attempt 2).
That retry is pending; unchanged successful jobs are carried forward. The interruption has no diagnosed fix and prompted no test suppression or
budget change.

**Implemented scope:** public Rust string/byte paths and generic inode chown,
including existing C path/inode delegates, now enter the very same mount control
used by chmod. The historical private module/helper names are retained; there
is no second family lock. Entry and verdict-only volume checks precede generic
inode conversion. Path lookup reuses one context for inode planning. Missing
paths and stale generations refuse before marker writes. The original owner-ID,
ctime and checksum planner is shared unchanged; both families share buffered
inode preparation and guarded marker/complete-commit effects. All full-width
uid/gid halves, each `u32::MAX` leave-unchanged sentinel (including both together),
existing mode/type behavior and 128-byte timestamp clamping remain. No C ABI,
reference creation, permissions policy or feature support is broadened.

Three controls actually failed first: Rust byte-path and C chown accepted writes
while chmod's context was held, and generic inode conversion ran before refusal.
The same controls now pass. Sixteen new controls retain all eighteen public and
eighteen private chmod controls. They cover full-width IDs and sentinel sequences
on both inode layouts; byte paths and strong/any/stale generations; untouched
planning/volume refusals; conversion ordering/unwind; cross-family Rust/C
callback-worker reentry in both directions; every observed marker/commit write
and flush failure on direct and journaled mounts; Rust effect panics caught at
the C boundary; subsequent cross-family write/flush refusal and actual exported
C close without restoring a dirty marker after a caught planning panic.
New worker controls retain owned join handles, record timeout before returning
from the callback, and join after outer guard release; error/unwind cleanup joins
before the borrowed C handle can be released. No foreign C callback unwinds.

New exact VM test
`fs::guarded_chmod::needs_host::chown::linux_checks_public_chown_metadata`
covers sixteen Rust/C × 1 KiB/4 KiB × 128-/256-byte × direct/journal profiles.
Linux creates the file/directory and initial owner IDs, modes, data and times.
The actual public delegates execute high-ID and each/both-sentinel updates;
debugfs and Linux must independently agree on full final IDs, type/mode,
generation, exact ctime, preserved atime/mtime and payload, while judged e2fsck
checks the input, emitted close and Linux readback. Checksummed 256-byte extents
and legacy 128-byte mappings are included; 128-byte ctime retains existing signed
32-bit clamping. The focused workflow adds this exact one-test floor and keeps
all prior tests/floors plus the images, reports and log. It has not run locally:
this host lacks the required VM/KVM. Exact-commit independent CI remains pending.

Fresh baseline: **1,012 passed, zero failed/ignored**. The three initial controls
then fail as intended and pass after routing; expanded mount controls pass
**52**, zero failed/ignored. Final full local units pass **1,028**, zero failed/
ignored: Counter multisets retain all 1,012 baseline occurrences (including
expected-panic tests and repeated names) and add exactly the sixteen new controls.
The log is **1,276 lines / 84,428 bytes**, within unchanged 1,400-line / 85,000-byte
caps. Fresh fmt/clippy, provenance over all 496 tracked files, frozen Unicode
checks, shared-agent instructions, guest shell/Python syntax, saved insertion-red
patch applicability and whitespace pass. The locked release CLI build/install
passes. Expanded actual C close covers both a fresh clean mount and an owed dirty
marker after prior success; a chmod failure also refuses later chown before
conversion. Ignored `tmp/casefold-guest-ci-repair` holds the actual failed-CI
evidence, diagnosis, executed red patch/log, focused controls, full Counter
retention and fresh local lint/CLI/unit logs. Exact source CI remains pending
after push; no local VM run or standalone chown review is claimed.

**Next:** verify the original guest rerun and this source's exact full/focused CI,
download the actual new owner metadata artifacts, and review this family before
migrating another. Other mutators/readers/maintenance/raw handles and C caller
lifetimes still require external coordination. No Ready state, owner cache,
builder publication, buffer seal or transaction admission is added. The ignored
deep fuzz qualification, three insertion aliases and complete mapping/collision/
growth/recovery work remain open. Stage 5 is **not yet quantified**; all casefold
writes and actual-card/Windows/SteamOS qualification remain gated.

## Public chmod dispatch implementation/evidence review — 2026-10-16

Reviewed `c47b328381ea67dceca26fc33c796bd241c7fbac`, equal to fetched
`origin/casefold-rw` with a clean tree. Its exact-commit [full CI](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38983322842)
and [focused Linux evidence](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/38983322854)
passed. Downloaded actual native/guest logs and focused artifacts; comparisons
use test-name Counter multisets, retaining repeated occurrences:

| Evidence | Public / retained private controls | Enclosing result | Lines / bytes |
|---|---|---|---|
| Standalone unit | 18 / 18 | 1,012 passed, zero failed/ignored | 1,268 / 83,155 |
| x86 host unit | 18 / 18 | 1,012 passed, zero failed/ignored | 1,262 / 82,718 |
| aarch64 unit | 18 / 18 | 1,012 passed, zero failed/ignored | 1,262 / 82,718 |
| Compiled guest library | 18 / 18 | 708 passed, zero failed/ignored | Within whole guest log |

These are eighteen public and eighteen private controls in four contexts, not
144 distinct tests. Each native artifact's complete passing multiset matches
the fresh local baseline. Both exact metadata oracles executed in the guest
library and as focused one-test steps, zero failed/ignored:

- Public Rust/C: eight profiles, **81 lines / 9,553 bytes**, 48 tool and sixteen
  kernel invocation markers.
- Retained private: four profiles, **49 lines / 4,960 bytes**, 24 tool and eight
  kernel markers.

The **whole guest suite** reports **1,836 passed, zero failed, one ignored**
(3,391 lines / 179,772 bytes). The existing slow
`fuzz_smoke::deep_walk_byte_flip_fuzz_never_panics` remains an unresolved broader
qualification gate; no public/private control or exact metadata oracle was
ignored. This review does not waive it, remove the ignore or run an unrelated
unbounded sweep. A passing workflow is not a zero-ignored/exhaustive-suite claim.

Inspected all eight public images/sixteen debugfs/eight Linux reports and all
four current private images/eight debugfs/four Linux reports. All 24 emitted
file/directory inode records agree on identity/generation, type, mode 06751/
02750 and exact post-2038 ctime. Linux asserted preserved input atime/mtime and
payload; judged e2fsck accepted input, emitted close and Linux readback. A
separate public-format byte audit verifies final clean superblocks, descriptor
and inode checksums, allocation bits, ctime epoch bits, mtime, directory atime
and all twelve file payloads. Linux's last file read may update file atime and
its checksum after the saved debugfs report; that expected difference is
accounted for. No host mount/oracle tool or implementation source was used.

**Decision:** no blocker found within the stated public chmod contract. String
dispatch reaches byte dispatch once; path/inode helpers share one borrowed
context. Generic inode conversion is inside entry and after verdict-only volume
checks, before marker writes. Planning refusals preserve bytes, while marker
and complete commit retain checked invalidation and sticky uncertainty handling.
Overlap uses the existing Unsupported/ENOTSUP mapping; ordinary C errors and
success clearing remain. `FS_EXT4_GEN_ANY` still differs from a real zero
generation. Rust effect/planning panics retire the mount before the C catcher
returns. The exported C close control leaves a prior dirty marker untouched
after a caught planning panic with ordinary commit flags healthy. Test-only
opaque-handle helpers add no production ABI; successful worker controls join
before the owned handle is released.

Public signatures/reference creation and caller-owned C lifetime obligations
are unchanged. Other mutators, readers, flush/release/repair/replay/refresh and
raw fields/devices still need the stated external coordination. The guard does
not repair unsafe caller aliasing, swallowed errors, foreign unwind/panic-abort
or arbitrary argument destruction; refusal before conversion does not suppress
normal Rust argument drop. There is no fairness/callback-duration or whole-API
concurrency guarantee. No Ready state, owner builder/cache, buffer seal,
transaction admission or casefold-write enablement is provided.

Fresh baseline and final local units both pass **1,012**, zero failed/ignored,
with identical Counter multisets and no additions/losses. The final log is
**1,259 lines / 82,612 bytes**, within unchanged caps. Source/test/workflow
hashes remain unchanged; only the existing handoff/review documents change.
Fresh lint, shared/provenance instructions, whitespace and saved insertion-red
patch applicability pass. The source's
release CLI CI passed; no fresh release rebuild or local VM run is claimed.
Ignored `tmp/casefold-public-chmod-review` retains downloaded artifacts, scoped
comparisons, all image hashes/audit and baseline/lint evidence. No sibling,
dependency pin/lock, environment draft or publication changed.

**Next bounded implementation: guarded chown family only.** Migrate public
Rust string/byte/inode file owner/group changes and their existing C delegates
through the same mount context/failure bridge, with no second family-specific
gate. Share the current uid/gid/ctime/checksum planning; retain full 32-bit
high/low fields, each leave-unchanged sentinel and current mode/type/time
semantics. Enter before generic inode conversion, callbacks/lookup/generation
checks, then keep pre-marker planning and whole marker/commit effects. Require
executed red cross-family/public controls, actual Rust/C fault/panic/close
checks and new exact Linux metadata evidence before standalone family review.
The detailed contract is in `docs/CASEFOLD-INDEX-REVIEW.md`. File owner IDs are
not physical block-ownership proof. Other families, the ignored fuzz gate,
three insertion aliases, dependencies/overlays/seals/publication and full map/
collision/growth work remain open. Stage 5 stays **not yet quantified**;
casefold writes and Windows/SteamOS/actual-card qualification remain gated.

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
| **0 — Establish Linux reference behavior** | Build Linux-created images, record encoding/settings, measure uncertain behavior, and validate with Linux and e2fsprogs. | Established for the current test profiles; extend the matrix when new write behavior needs qualification. The independent Linux rmdir reference now passes eight profiles, 32 audited images, 184 error observations and 24 closed-target removals at `ccef341`. Independent Linux mkdir reference now passes all eight layouts / forty audited images / 216 error observations / 24 creations at `79ed456`. Exclusive regular-file create reference now passes all eight layouts / 48 audited images / 216 consistent errors / 24 creations and 408 debugfs reports at `e384ed9`, with final cold namespace verification. Independent Linux link/linkat(0) reference now passes eight layouts / 80 audited images / 56 links / 560 consistent errors / 960 debugfs reports at `9fd59cf`, with cold alias identity and exact no-allocation accounting. |
| **1 — Recognize and validate casefold policy** | Read filesystem encoding and directory flags; reject unknown or inconsistent settings. | Implemented. Write restrictions remain in place. |
| **2 — Implement frozen Unicode comparison** | Use filesystem-compatible Unicode 12.1 rules, including normalization and full case folding, rather than current platform Unicode tables. | Implemented for the supported valid-UTF-8 subset. Malformed casefold names and normalized dot aliases remain refused. |
| **3 — Resolve names and retain actual entries** | Share comparison rules across lookup and mutation planning; return stored spelling and inode identity. Validate indexed lookup. | Implemented for supported read shapes, including linear directories and indexes with one intermediate level. Intermediate/parent symlink traversal and 1/4 KiB target boundaries now pass all five exact focused Linux tests at 25a039e, with saved-image verification; complete-suite CI remains separate. Casefold indexed reads scan validated referenced leaves; this does not establish correct write routing. The 2026-10-27 trailing-slash/index-dot adaptation passes focused and full native checks; independent Linux path outcomes now pass in exact focused run 40502467167, with saved-source/report/image verification; repaired full CI, the retained Stage 0 rerun and complete source-matched image/report retention now pass at the latest closure. |
| **4 — Implement namespace mutations** | Make create, mkdir, links, symlinks, unlink, rmdir, and rename obey casefold identity and inheritance. Preserve transaction safety. | **Current stage: partially implemented.** Removal and rename retain/recheck matched entries. Parent-symlink namespace traversal passes the existing focused Linux and complete-suite evidence. The public-unlink design audit and dormant executed native red preparation are complete: the saved 28-control patch repeats five passes / 23 failures and active source is restored. Independent reference-only Linux policy now passes eight profiles with audited saved bytes at `30c15f4`; the saved 22-control policy patch now adds two passes / 20 failures across six native profiles, and both combined 50-control runs repeat seven passes / 43 failures with source restored. Shared-context/semantic repair now passes all 57 native controls, 1,196 unit passes and a 16-profile independent Rust/C Linux emission job with all 80 saved images audited at `cd15806`; standalone source/saved-evidence review and all focused/retained audits now pass. Exact repair-source full CI at `661ba75` passes every job and ci-ok, retaining 2,033 passes in both whole suites, 228 controls in five contexts and all trailing native checks; scoped unlink closure is complete. The subsequent public-rmdir design audit and executed dormant red/positive preparation are complete: both 55-control runs report six passes / 49 failures (45 current public boundaries and four future SB diagnostics), and restored native units retain 1,196 passes and all 228 controls. The independent Linux rmdir reference now passes all eight profiles with 32 audited images and 184 consistent error observations at `ccef341`; parent timestamps, long-name policy and reclamation are measured. The new dormant 27-control measured policy patch now runs nine passes / 18 failures across all six native profiles; both combined 82-control reruns retain 15 passes / 67 failures and every original outcome. Both patch orders recreate identical source, active source is restored, and full lint passes. Guarded rmdir dispatch/whole-plan migration and measured policy repair now pass all 86 native controls and retain every preceding name (1,282 passed, 314 controls), with lint passed and a measured unit-only cap repair. The exact focused rmdir job and saved-byte audit now pass all 16 layouts/112 images/48 removals. Both whole suites retain all 2,034 previous occurrences and add 87 (2,121 passes, one existing ignored); actual byte-only wrapper failures are repaired by identical replay at native 205,000/guest 213,000. All remaining retained archive audits and the source/evidence review now pass via verified public artifact access. Budget-repair full run `39702689248` and evidence-checkpoint full run `39703344673` now pass all jobs and ci-ok. Verified archives retain all 2,121 whole-suite passes, 1,282 unit names, 314 controls in five contexts and 527 aarch64 image passes; all twenty trailing shell and 159 installed-CLI checks pass. Scoped rmdir closure is complete under caller exclusion, healthy layouts and closed reference handles. The public-mkdir design audit and executed native safety preparation are complete: the dormant 73-control patch repeats eleven passes / 62 failures (58 public safety reds and four future SB diagnostics), all 352 I/O injections and four public publication unwinds reach their checks, and active source is restored with all 1,282 units/314 controls retained. Independent Linux mkdir reference and saved-byte audit now pass all eight layouts / forty images / 216 errors / 24 creations and 328 debugfs reports at `79ed456`. Exact preceding full/focused CI and 2,122-pass whole-suite retention now pass. The new dormant 27-control policy patch executes fourteen passes / thirteen failures across all six native profiles; both combined 100-control runs repeat 25 passes / 75 failures, retaining every original outcome, all 352 I/O injections and four publication unwinds. Both patch orders recreate identical source, full active lint passes and restored units retain all 1,282 names / 314 controls. Guarded mkdir/owned-plan and measured parent timestamp/path repair now pass all 104 native controls, with 1,386 normal unit passes, all 1,282 prior occurrences and 418 controls once. All 352 measured I/O uncertainties and four public publication unwinds now retire the mount; full lint passes and an actual unit-only output overflow is repaired by identical replay. Both mkdir patches are historical promoted artifacts. The exact separate Linux emission job now passes all sixteen layouts / 112 audited images / 48 creations, with 544 debugfs reports, 432 kernel records and exact per-group/bitmap/CRC/journal checks verified from saved bytes. Exact CI units retain all 1,386 occurrences / 418 controls. Both implementation/docs full runs now retain all 2,227 whole-suite passes, 1,386 unit occurrences, 418 controls in five contexts and 527 architecture image passes. All eight focused jobs and fresh retained-family audits pass; scoped source/evidence review finds no further blocker. Measured native/guest whole-byte failures are repaired by identical old-red/new-green replay at 220,000/230,000, keeping line limits, floors and output. Budget repair `7f9617d` now passes every full CI job and ci-ok in exact run `39997261948`; verified artifacts retain all 2,227 whole-suite passes, 418 controls in five contexts, 1,386 unit occurrences and 527 architecture image passes. Actual whole wrappers/floors and trailing twenty shell/159 installed-CLI checks pass. Scoped mkdir review is closed within caller exclusion, healthy layouts and closed reference handles. The regular-file create staging/marker/dispatch design audit is now complete at `1248f71`: shared context/owned-plan/full-SB/complete-effect/C-readback design and concrete native controls are recorded, preserving create-specific Into-before-name and actual path syntax. No create guard or executed red is claimed. Dormant native create safety preparation now repeats all 72 names/outcomes: 14 pass / 58 current public safety failures / zero ignored; all 288 measured I/O injections and four publication unwinds execute. Fifty-four primary positive profiles and additional same-table/uninit/readback/priority controls execute; lint passes, source is restored and all 1,386 unit occurrences/418 controls are retained. The independent reference-only Linux create policy test and exact focused job are now added: eight layouts/six saved phases plan 48 images, 24 creations and 216 measured error observations with cold final verification. Exact `e384ed9` focused/full CI now passes all jobs/ci-ok. Source/digest/CRC-verified raw audit passes all eight layouts/48 images/24 creations/216 consistent errors/408 debugfs reports. Complete-suite retention is 2,228 passes (all 2,227 previous plus one), with all 418 controls in five contexts, 1,386 unit occurrences, architecture and trailing checks retained. Measured native create policy reds now run fourteen passes/thirteen failures across all six profiles; both composed 99-control runs repeat 28 passes/71 failures with every original safety outcome/288 I/O/four publication injection retained. Active lint passes, source is restored and all 1,386 units/418 controls remain. Guarded create/owned-plan and measured timestamp/path repair is next before independent emission qualification. Explicit API mode/name/FFI contracts remain separate. Stage progress is not yet quantified. Duplicate checks use shared comparison. New-directory inode construction now inherits casefold correctly. Actual casefold writes remain blocked. Guarded public create now passes all 108 promoted/real-boundary controls and 1,494 native occurrences/526 controls, with measured unit-only budget repair and lint passed. Its sixteen-layout/112-image Rust/C emission test is added; actual CI, raw-byte/full-suite/retained-family audits and scoped review remain pending. Both create patches are historical/promoted; do not reapply. Exact `f436f4e` independent create/retained-family audits and 2,337-pass complete-suite retention now pass, all 526 controls in five contexts and 527 architecture image passes retained. Scoped source/evidence review finds no further blocker; actual wrappers fail only measured log caps and identical replay repairs them. Budget-repair CI, actual whole-suite floors and native trailing checks are pending before create closure; casefold writes remain blocked. Exact create budget-repair run `40080977121` at `38295af` now passes all jobs/ci-ok with complete 2,337-pass retention, actual floors and twenty shell/159 native CLI trailing checks. Scoped create source/evidence/full-CI review is closed within caller exclusion, healthy layouts and closed handles; the subsequent public hard-link Rust/C design audit is complete. Casefold writes remain blocked. Public hard-link staging/marker/dispatch design audit is now complete, preserving dual-Into/source/destination priorities and optional-SB allocation-free staging. Executed dormant native safety/positive preparation is next; no link guard, executed red or new independent link evidence is claimed. Dormant hard-link safety preparation now repeats all 84 outcomes (20 pass/64 current safety failures), all 200 measured uncertainty/four publication injections and positive storage profiles; source is restored with 1,503 units/526 controls and lint retained. Independent reference-only link policy is next; no link guard is implemented. Independent Linux hard-link reference/saved-byte audit now passes eight layouts/80 images/56 links/560 errors, including source/destination times and directory version. Exact new full-CI retention remains pending before measured native policy reds/composition; no link guard is implemented. Measured native link policy preparation now repeats 39 controls (12 pass/27 semantic failures) and both 123-control compositions (32 pass/91 fail), with all original safety outcomes/injections retained. Exact reference full CI/2,350-pass five-context retention is closed; active 1,503 units/526 controls and lint are restored. Guard/metadata repair is next; no link guard is implemented. Latest native guarded-link migration now passes all 136 focused controls and 1,639 units/662 controls, with 216 remeasured I/O injections and preserved baseline multiset. Both link patches are promoted historical artifacts; exact-source CI retention and independent Rust/C emission/growth review remain pending. Stage progress is not yet quantified. Allocation-free independent Rust/C link emission tests and source-bound focused retention are now added; native checks pass, but actual sixteen-profile Linux outcomes and saved-byte audit are pending. Growth remains a separate qualification checkpoint. Exact source e6db258 now passes focused ordinary allocation-free link emission and the complete sixteen-layout/112-link/352-image saved-byte audit. Implementation afd24b8 full CI/retention passes; new host-test full CI, growth/counter-wrap qualification and scoped review remain pending. Exact docs-source full CI now retains 2,487 whole successes, all 662 controls in five contexts and trailing checks; a measured 64-byte native log overflow is repaired separately without changing driver/test bytes. Allocating full-linear-parent qualification is next; broader growth/counter-wrap/scoped review remain. The independent full-linear-parent growth test/job is now added with all 1,639 native occurrences retained, but actual Linux/allocation/saved-byte results are pending; no growth qualification is claimed. First linear-growth CI reproduced a test-only omission of external-EA sectors; saved-Linux-input assertion replay is red then green after correction. Corrected-source linear growth now passes exact focused Linux, all 352 saved-image checks and full 2,488-success retention. This supersedes the preceding pending linear-growth status; inline/index/deeper growth, counter wrap and scoped review remain open. Latest checkpoint: the exact inline focused job and all 160 saved-image checks pass, with both repeated linear matrices audited. Full guest retention now proves 2,497 successes / 670 controls; a measured byte-only wrapper failure has an identical-log replayed guest-cap repair. Native retention and fresh repair-source full CI remain pending. No broader write qualification is claimed. |
| **5 — Implement and qualify directory-index writes** | Use frozen Unicode keys for hashing, insertion, collisions, leaf splits, and growth. Preserve Linux-visible routing. | **Hashing foundation, sampled read routing, and root-only in-place insertion implemented behind the volume write gate.** The new insertion primitive validates all referenced entries before changing one leaf. The preceding insertion/mapping source passed independent Linux and full CI. Preceding admission checkpoints through `1a6e8d1` passed exact-commit focused Linux and full CI. The staged superblock contract and allocation-review checkpoints passed full CI; focused Linux insertion passed at `b7b0325`. The current-allocation guard at `96b3b72` passed 839 local units, lint, the CLI build, focused Linux insertion and full CI. The scoped journal guard at `f9a0df0` refuses all three reproduced journal aliases and passed 852 local units, lint, the CLI build and exact-commit focused Linux/full CI. The private bounded complete extent-ownership collector at `7bdfe0c` passes 872 local units and exact-commit Linux FIEMAP/insertion/full CI. Its implementation/resource review found no blocker within the standalone scope and records aggregate-budget, capacity and depth-matrix requirements. The private current inode inventory at `4309ed1` has 15 local controls and passed exact-commit Linux allocation/insertion/FIEMAP/full CI; its implementation review is complete. The private current inode-role reader at `ef1ca24` has 17 controls and passed exact-commit Linux role/inventory/insertion/FIEMAP/full CI; its implementation review is complete. The private fixed-metadata collector at `95cb218` adds 17 controls and passed exact-commit VM location/insertion/inventory/role/FIEMAP/full CI; its implementation/evidence review found no blocker within the standalone scope. The private bounded current block-allocation interval verifier is implemented with 20 focused controls, padding/current-view checks and shared resource caps. Initial independent CI hit an unsupported block-zero oracle query; repair `72b6911` passed exact-commit focused VM/full CI. Its complete standalone allocation review found no blocker within the stated scope. A private complete current-view ownership builder is implemented with shared accounting and 18 focused controls; 961 local units pass, retaining all 943 baseline occurrences. Its six-profile whole-result Linux oracle passed exact-commit focused VM evidence; all 222 allocated records and 215 claims were audited with no blocker within the standalone stable-view contract. Source and review full CI now pass. The concrete operation serialization/mutation/invalidation audit is complete. A disconnected private gate is now implemented with 15 native exclusion/freshness/failure controls; 976 full local units and lint pass, retaining all 961 baseline occurrences. Exact-commit full/focused CI and standalone primitive review are complete; all 15 controls executed in four native/guest contexts. The whole guest suite still has one pre-existing ignored slow fuzz test, an unresolved broader qualification gate. Mount-bound private path/inode chmod adapters and sticky mount-failure bridging now pass exact-commit full/focused CI and standalone family/evidence review, including all four images and reports. Public chmod dispatch now passes exact-commit full/focused CI and standalone migration/evidence review, including all eight public and four retained private Linux profiles. Guarded file owner/group changes (chown), including its signed-report parser repair, now pass exact full/focused CI and standalone implementation/evidence review. All sixteen owner profiles and twelve retained chmod profiles were audited. Guarded timestamp updates (utimens), including the short-extra report repair, now pass exact full/focused CI and standalone implementation/evidence review; all 24 timestamp and 28 retained metadata profiles were audited, with 91 guard/family/parser controls executed. Existing public inode-flag updates now pass exact full/focused CI and standalone implementation/evidence review; all sixteen flag and 52 retained metadata profiles were audited. The multi-block xattr-removal staging/marker design audit is complete with nine executed red controls and one passing four-profile legacy control saved in a dormant patch. Existing public removal is now implemented locally under that contract with native controls and a new sixteen-profile independent matrix; its first focused Linux run failed the mixed zero-extra fixture timestamp comparison. The cfg(test)-only coherent fixture repair now passes exact focused Linux CI; all sixteen removal and 68 retained profiles were audited. Exact full/focused CI and the standalone removal-family implementation/evidence review now pass within the existing caller-exclusion contract. Both full suites retain all preceding test occurrences; all 134 controls execute across five CI contexts. The ignored slow fuzz case remains a broader qualification gate. The public setxattr staging/marker audit and executed red preparation are complete. The saved one-pass/fourteen-failure patch was rerun, and the guarded public setter whole-plan migration now passes 27 added native controls while preserving all 1,095 baseline occurrences. Six-family exclusion/failure retirement and a new exact Linux matrix with separate fresh-image first-inline reference/emission cases are committed. Exact focused Linux CI reports success. Actual SHA-256-verified full-CI artifacts now retain all 1,923 baseline occurrences and add 28 in both full suites, with 1,951 passed / zero failed / one existing ignored. All 161 guard/operation/parser/fixture controls execute in five contexts. The measured 183,600-byte native log reproduces output-budget exit 65 at 183,000; the same complete log passes the native-only 186,000-byte repair with floors, filters and output retained. Fresh full CI for that configuration repair remains pending. The later read-only audit now covers all sixteen setter layouts, eight fresh first-attribute profiles and 84 retained profiles at the earlier setter source. All eight Linux identical-inline-value observations preserve ctime, but executed advancing-clock Rust/C controls reproduce a timestamp change. The saved same-value red patch records this blocker; active source is restored. The later same-value repair now passes eight native policy controls and preserves all 1,129 preceding occurrences (1,137 passed). Independent reference-only evidence confirms all 32 ordinary unchanged-value answers; Exact repaired-source focused Linux emission and all retained-profile audits now pass at 01f96c8, including sixteen byte-identical policy-emission images. Full CI fails on a separate backported casefold-symlink fixture whose absolute target escapes the guest mount. The relative-target/status-propagation correction compiles locally and adds a focused five-test evidence step. Correction-source focused Linux now passes all five symlink tests and all setter/retained image audits at 25a039e. Full CI remains pending. A later refusal-order review found and reproduced two errors; complete inline preflight restores the prior name/EA-inode priorities without bypassing external checks or changing legacy precedence. Native validation retains all 1,137 occurrences and adds two (1,139 passed, 171 controls). The preceding 25a039e source now passes exact full CI: both complete suites retain all 1,951 prior occurrences and add 21 (1,972 passed, one unchanged ignored), with all 169 controls in five contexts. Current 4079f36 passes verified 1,139-test units (171 controls) and both focused Linux jobs; its sixteen/eight setter/first-attribute and 84 retained-profile audits now pass. Its complete suites now both retain all 1,972 preceding occurrences and add two (1,974 passed, one unchanged ignored), with all 171 controls in five contexts. The native job fails only because its actual 186,123-byte log exceeds 186,000; byte-identical red/green replay verifies the native-only 189,000-byte repair with unchanged floors, filters and line limits. Guest passes unchanged. Budget-repair source 2c6ece6 now passes every job and ci-ok in exact full run 39505996964; all 1,974 occurrences are retained in both whole suites and all 171 controls execute in five contexts. Native trailing checks pass twenty shell checks and 159 CLI checks. Actual native/guest output fits the measured limits. Standalone public setter implementation/evidence review is now complete within the caller-exclusion/healthy-layout scope. The scoped implementation source walk found no further blocker. The public-unlink design audit and executed dormant red preparation are now complete; both runs of the saved 28-control patch report five passes / 23 failures and restored units retain all 1,139 occurrences and 171 controls. Independent Linux timestamp/counted-name policy now passes eight profiles with audited saved bytes; the new 22-control native policy patch and both combined 50-control reruns are executed with source restored. Guarded/semantic repair now passes 57 native controls and the 16-profile Rust/C emission job, with all 80 saved images audited at `cd15806`. Exact repair-source full run `39547402531` now passes all jobs and ci-ok at `661ba75`, retaining 2,033 passes in both suites, all 228 controls in five contexts, twenty shell and 159 CLI checks. Scoped unlink source/evidence review is complete under its caller-exclusion/healthy-layout contract. Public-rmdir design audit and executed dormant red preparation are complete: the 55-control patch repeats six passes / 49 failures, including 45 public-boundary reds and four future SB diagnostics, with 127 valid native removals and 340 observed uncertainty injections. Active source is restored and all 1,196 native passes/228 controls are retained. Independent Linux rmdir reference evidence now passes all eight profiles, 32 saved images and 184 error records with exact reclamation and peer preservation at `ccef341`. Native measured policy reds now execute all 27 controls (nine pass / 18 fail), and both combined 82-control runs repeat 15 pass / 67 fail with all original outcomes preserved, source restored and lint passed. Verified preceding full run `39644593304` passes every job and ci-ok: both whole suites retain 2,034 passes and one unchanged ignored fuzz case, all 228 controls execute in five contexts, all unit contexts retain 1,196 names, and trailing native checks pass. Guarded rmdir migration and measured semantics now pass all 86 native controls with 1,282 unit passes/all 314 controls retained and full lint passed. Only the actually exceeded unit log limits are repaired by identical replay. The 16-profile/112-image ordinary Rust/C emission job and raw saved-byte audit now pass, and both whole suites retain 2,121 passes/all 314 controls with one unchanged ignored. Actual byte-only whole-wrapper failures are repaired after identical old/new log replay; all remaining retained archive audits and source/evidence review now pass. Budget-repair full run `39702689248` and evidence-checkpoint run `39703344673` now pass all jobs and ci-ok, with exact 2,121-pass whole-suite retention, all 314 controls in five contexts, all 1,282 unit names and 527 aarch64 image passes. All twenty native shell and 159 installed-CLI checks now pass. Scoped rmdir source/evidence/full-CI closure is complete within caller exclusion, healthy layouts and closed reference handles. The public-mkdir staging/marker/dispatch design audit and executed dormant native safety preparation are complete. Both 73-control runs retain eleven passes / 62 failures, with 58 public safety reds/four future diagnostics, all 352 I/O injections and four public publication unwinds reached, lint passed and source restored with all 1,282 units/314 controls retained. The independent Linux mkdir reference and all forty-image/328-report audits now pass at `79ed456`, with exact per-group accounting and consistent timestamp/name/mode observations across eight layouts. Preceding full/focused CI and verified 2,122-pass retention now pass. Both composed mkdir audits execute the same 100 outcomes (25 passes / 75 failures), with all original I/O/unwind probes retained; native policy reds are now measured, active lint passes and source is restored with all 1,282 units / 314 controls. The mkdir guard/owned-plan and measured policy repair now pass all 104 native controls and retain all 1,282 prior unit occurrences (1,386 total, 418 controls), with every 352 measured I/O uncertainty/four publication unwind retiring correctly and full lint passed. Independent Linux mkdir emission now passes the exact focused job and saved-byte audit for all sixteen layouts / 112 images / 48 creations, preserving counters, protected peers and dirty/closed/journal state. Both exact implementation/docs full runs retain 2,227 whole-suite passes, 1,386 unit occurrences, 418 controls in five contexts and 527 architecture image passes. All focused and retained-family audits pass; scoped source review finds no further blocker. Measured byte-only wrapper failures are repaired by identical replay at native 220,000/guest 230,000; budget repair `7f9617d` now passes exact full run `39997261948` and ci-ok with all 2,227 passes/418 controls retained, actual whole wrappers/floors and trailing twenty shell/159 CLI checks passed. Scoped mkdir review is closed within caller exclusion, healthy layouts and closed reference handles; the regular-file create design audit is now complete, preserving Into-before-name/path contracts and defining the shared-context/owned-plan/complete-effect/C-readback boundary. Actual dormant native create safety preparation now repeats 14 pass / 58 current public safety failures across 72 names, with all 288 measured I/O/four publication injections reached and positive profiles retained. Active lint passes and restored units retain all 1,386 occurrences/418 controls. Independent reference-only Linux create policy collection is now added with eight layouts/six phases, 48 planned images/24 creations/216 errors and a floor-one focused job. Local compilation/contracts/lint and 1,386-unit/418-control retention pass; actual exact-source focused/full CI and all 48-image/408-report audits now pass, retaining 2,228 whole-suite passes/all 418 controls in five contexts and every trailing check. Native measured policy reds now execute fourteen passes/thirteen failures; both patch orders recreate identical source and both 99-control runs repeat 28 passes/71 failures, retaining all original safety coverage and 270 policy creations/714 observations. Active lint passes and restored units retain all 1,386 occurrences/418 controls. Guarded create and measured policy repair are next; no create guard, semantic repair or emitted-image qualification is claimed. Both patches are historical promoted artifacts; syscall observations do not silently override existing API contracts. Default Git and public API reads/downloads work; authenticated Actions-write authority remains unverified after the inherited credential returned 401. Other families and lifecycle remain unguarded. Stage progress remains not yet quantified. Admission remains disconnected. The three other-inode ownership aliases were rerun and remain unresolved. Exclusive ownership and complete mapping shape remain open. Intermediate-level insertion, collisions, splitting/growth, and complete write qualification remain required; index dropping stays refused. The ordinary public-create scoped review is now closed after exact green budget-repair CI and full/trailing-check retention; no ownership admission, mapping-growth qualification or seal integration is implied. The hard-link design now records an optional-SB completed-plan/effect/readback boundary; executed native controls and allocation/growth evidence are still required before migration, with ownership/admission gates unchanged. Native hard-link no-SB/same-table/inline/index/legacy/depth-one/uninit controls are executed, but optional-SB migration/private boundary diagnostics and independent emitted-image qualification remain pending. Native policy/safety composition is now executed and source restored; full reference CI retention passes. Actual LinkPlan/optional-SB/complete-effect/readback migration and real-helper diagnostics remain next before independent emissions. Latest real LinkPlan boundary/optional-SB/debit/effect/publication/C snapshot and Runtime diagnostics pass native checks. Fresh independent emitted-byte/CI qualification is pending; no broader admission or lifecycle integration is implied. New independent allocation-free link tests assert complete descriptor/bitmap/EA preservation and distinguish actual dirty/closed driver bytes from Linux observer copies; actual oracle results remain pending. Allocation-free Rust/C dirty/closed/Linux-copy images now prove exact metadata/version/zero-allocation and protected-inode/EA preservation across sixteen layouts. Allocating/converting/indexed growth and broader transaction/admission/lifecycle gates remain open. New public-link host test now survives complete source-bound full CI (2,487 whole successes); optional-SB allocating-growth evidence remains open. The new independent linear-growth test now passes exact Linux/saved-byte evidence for its bounded one-to-two-block linear shape, including exact optional-SB/group/bitmap debit and peer/shared-table preservation. Inline/index/deeper growth and broader write qualification remain open. Latest checkpoint: the exact inline focused job and all 160 saved-image checks pass, with both repeated linear matrices audited. Full guest retention now proves 2,497 successes / 670 controls; a measured byte-only wrapper failure has an identical-log replayed guest-cap repair. Native retention and fresh repair-source full CI remain pending. No broader write qualification is claimed. |
| **6 — Prove recovery and enable qualified writes** | Test interrupted operations, journal recovery, errors, and refusals; enable only the combinations that pass. | Not yet reached. The ordinary-volume recovery recount regression passed independent Linux checking and readback; its full-CI output-budget repair and subsequent prerequisite checkpoints passed. Casefold operation/recovery qualification remains incomplete; keep repair and unsupported combinations blocked. |
| **7 — Integrate and qualify Windows** | Audit the Windows adapter's name conversion, caching, create/rename semantics, serialization, device ownership, flushing, and eject. Test alternating Windows/Linux use. | Not yet reached. Pin and audit an actual `ext4-win-driver` revision; later qualify specific SteamOS versions/devices. |

## What comes next

1. Reuse this session's verified setup and authorization. Source `6c71cbe` passes all fourteen focused jobs and all three link saved-byte audits. Its exact full guest suite in **40785791674** now retains every baseline occurrence (**2,497 successes / 670 controls once**) but the actual **248,844-byte** log breaches the 248,000-byte ceiling. Identical old-red/new-green replay verifies the guest-only 251,000-byte adjustment with unchanged line cap/floor/output. Its actual CI floor is unrun after wrapper exit 65; repair-source green CI remains required. Next inspect still-running native job **123159535998**, audit exact full native/guest multiset/control/wrapper/floor/trailing retention, and repair only demonstrated failures before closing all five control contexts. Do not infer a complete full gate or begin broader mapping growth yet. Exhaustion classification, scoped review, indexed/deeper/uninitialized-group/counter-wrap and ownership/admission/seal/lifecycle/recovery/fuzz/write/platform gates remain; stages 4–5 stay not quantified.
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
gh run list --branch casefold-rw --limit 5
```

Follow `docs/CLOUD-STARTUP.md` for fresh setup, then load the pinned tool paths:

```sh
source /workspace/.cloud-setup/env.sh
```

Use the repository's tasks, preserve command exit statuses, and inspect failure logs:

```sh
python3 scripts/cloud-casefold-run.py chore test:unit
python3 scripts/cloud-casefold-run.py chore lint
python3 scripts/cloud-casefold-run.py chore cli:install
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

**Next bounded task:** use the latest 2026-10-16 public chmod review above; guard only the public chown family through the same context, with cross-family controls and new exact Linux owner-metadata evidence before another review.
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

Some saved Linux images and local logs live under the checkout's ignored `tmp/` directory. They are useful when present but are not guaranteed in a fresh cloud environment; use committed tests and CI artifacts to reproduce or retrieve evidence. The previous index-routing/refusal checkpoint passed full CI and fresh Stage 0 evidence; the subsequent root-only insertion passed focused Linux evidence at `a7db076`, with full CI also passed. The early mapping fix had successful focused evidence but hit the measured unit output budget in full CI. The repairs and later full CI at `3be872e` and `8f94cf8` passed; the mapping-metadata checkpoint at `986808c` also passed. The staged-descriptor checkpoint at `1a6e8d1` also passed. The staged-superblock checkpoint at `b7b0325` passed focused Linux insertion and full CI; the allocation review at `065f9a8` passed full CI. The current-allocation guard at `96b3b72` passed focused Linux and full CI; its insertion log was inspected. The ownership review retains three unresolved other-inode aliases in a self-contained patch. The scoped journal guard at `f9a0df0` passed exact-commit focused Linux/full CI; its actual insertion log was inspected. The owner-design review reran the three other-inode failures. The subsequent private bounded complete extent-ownership collector at `7bdfe0c` passed exact-commit Linux FIEMAP/insertion/full CI. Its implementation/resource review records the independent depth-zero/one matrix and aggregate-budget/capacity requirements. The private inode inventory at `4309ed1` adds 15 local controls without admission integration and passed exact-commit focused Linux allocation/insertion/FIEMAP/full CI; its implementation review is complete. The private role reader at `ef1ca24` has 17 focused controls and passed exact-commit Linux role/inventory/insertion/FIEMAP/full CI; its implementation review is complete within the narrow profile. The private fixed-metadata collector at `95cb218` adds 17 controls and passed exact-commit focused VM location and existing Linux/full CI; its implementation review and documentation review full CI are complete. The private bounded current allocation interval verifier adds 20 focused controls. Initial CI failed on an unsupported block-zero oracle query; repair `72b6911` passed exact-commit focused VM/full CI and its standalone implementation review is complete. The private complete current-view owner builder now joins those prerequisites under shared accounting, with 18 controls and 961 passing local units. Its six-profile whole-result Linux oracle passed exact-commit focused VM evidence; all six complete reports/images were audited and the standalone constructor review found no blocker within its stable-view contract. Source and review full CI passed. The concrete serialization/mutation/invalidation audit is complete, and the disconnected private operation gate now has 15 native controls. Its exact-commit full/focused CI and standalone review are complete; the private chmod/context/health bridge now passes exact-commit full/focused CI and standalone implementation/evidence review, including all four Linux images and reports. Public chmod dispatch now passes exact-commit full/focused CI and standalone migration/evidence review, including all current twelve public/private Linux images and reports. Guarded chown and its report-parser repair now pass exact full/focused CI and standalone family/evidence review, including all sixteen owner and twelve retained chmod profiles. Guarded utimens and its short-extra display repair now pass exact full/focused CI and standalone family/evidence review; all 24 timestamp and 28 retained metadata profiles were audited. Existing public inode-flag updates now pass exact full/focused CI and standalone family/evidence review. The multi-block xattr-removal staging/marker audit and executed red preparation are complete; existing public removal is now implemented locally under that contract, its initial focused run exposed a mixed zero-extra fixture timestamp mismatch; the coherent fixture repair awaits exact independent CI/artifact/family review. The preceding review HEAD passed its guest retry. Other operation/lifecycle/admission integration remains separate. The broader guest suite retains one pre-existing ignored slow fuzz case, so exhaustive write qualification remains gated. The insertion-level aliases and public write gates remain unchanged. No claim is made here that a separate coverage workflow ran at the latest commit.

This note is committed as `docs/extprogress.md` on `casefold-rw`. A new Codex session should open that branch, read this file, and verify the actual checkout and CI before continuing. Verify the selected commit against its own CI; earlier successful runs are evidence for those earlier checkpoints. Update this handoff as implementation checkpoints are completed.
