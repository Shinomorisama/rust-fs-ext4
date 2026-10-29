# Upstream and fork review register

This register distinguishes **metadata checked**, **selected implementation
reviewed**, and **not checked**. A listed fork is not blanket approval to reuse
its code. Preserve the clean-room rules in `AGENTS.md`; older forks can contain
the prohibited pre-rewrite implementations even when their license says MIT.

Append a dated review when sources change. Keep previous dates and revisions;
do not replace an earlier check date merely because a repository was listed
again. The [machine-readable snapshot](reviews/2026-10-29-fork-snapshot.json)
records every branch name and exact tip examined in this pass.

## Review on 2026-10-29 — America/Chicago

Scope: the public fork networks of `antimatter-studios/rust-fs-ext4` and
`antimatter-studios/ext4-win-driver`, excluding our own two forks from the
external-fork count. All API pages were enumerated. **Eight external forks /
49 branches** were checked at the snapshot's UTC timestamp: 47 MIT library
branches and two GPL Windows-driver branches. Licenses and provenance boundaries
were checked before selecting implementation to inspect. No external fork code
was imported into `casefold-rw`.

### Reviewed main baseline

| Item | Exact revision or outcome |
|---|---|
| Previous fork `main` | `763ecf9deb89d4d6dae8790310bcb1c38e173d15` |
| Reviewed upstream source | `90073d67e2af5459c380220222cd23f2a905c37d` |
| Main import commit | `5045433a20428b24d3f072556e151af07d5cf517` |
| Scope | 39 upstream commits, 116 changed paths; one squash import preserving fork history |
| Main dependency | Core v0.4.2, commit `9e841ae7ce94ed0ef9c37b9b217e03b9c978febc` |
| Casefold dependency | Remains core v0.3.0; no automatic upstream merge into casefold |
| Main local validation | 728 unit successes; exact upstream passing-name multiset retained, zero ignored; actual floor 443; full lint passed |
| Upstream independent CI | [40661813529](https://github.com/antimatter-studios/rust-fs-ext4/actions/runs/40661813529), exact source above: all 12 jobs including aggregate passed |
| Fork main independent CI | [40862413814](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/40862413814), exact import `5045433`: all 12 jobs including aggregate passed; API source/run/job conclusions verified on 2026-10-29 |
| Unit archive | Artifact 12804361422; API source/run, SHA-256 and all ZIP CRCs verified |
| Candidate-source equality | Implementation, tests, dependencies and workflows exactly match the reviewed upstream tree; only the fork AGENTS pointer and approved README introduction/review link differ |
| Recovery | Annotated `archive/main/2026-10-29/pre-upstream-90073d6` at the old main tip, plus a verified local bundle |

This is a baseline import review, not an exhaustive new audit of every CLI line
or a Windows/card qualification. The source changes include recovery recount,
symlink traversal/limits, truncate limits, raw attribute-name handling and
first-attribute feature publication, with their upstream regression tests. CLI
tree import/export, metadata preservation, namespace tools, CI planning and
core/tooling changes are imported on main as that complete upstream baseline.
Some filesystem fixes already have separate casefold backport evidence in
`extprogress.md`; do not apply them again based only on this inventory.

The import is squashed to preserve the existing fork main history. Therefore
GitHub's commit ahead/behind counts alone do not establish whether this baseline
was imported. For the next update, compare **from the recorded upstream source
above to the new upstream source**, and check actual file differences. Preserve
the fork-only documentation. Publishing does not establish the new main CI
result: inspect the exact import commit's own run separately.

### External forks checked

All default heads below are full SHAs in the JSON snapshot; abbreviated SHAs
are used here for readability. Branch counts include each default branch.

| Fork | Default head | Branches | What was checked and useful findings |
|---|---|---:|---|
| [cyberphantom52/rust-fs-ext4](https://github.com/cyberphantom52/rust-fs-ext4) | `de7706a9381c` | 2 | Both branch histories/paths checked. Selected post-clean-room inline-attribute patch and its guest oracle tests inspected. The fix is already upstream as #596; the casefold planner also publishes the feature. No duplicate patch needed. |
| [zubvit/rust-fs-ext4](https://github.com/zubvit/rust-fs-ext4) | `3a95a96dd70a` | 13 | All branch metadata checked. Default head has no unique commits. Multi-descriptor journal and backup/reserved-metadata branches are regression-scenario leads; historical implementation not inspected. |
| [1vivy/rust-fs-ext4](https://github.com/1vivy/rust-fs-ext4) | `26c5f463c198` | 20 | All branch metadata checked. Distinct recovery/release lifecycle, failed-handle retirement, cache invalidation, legacy group checksum, multilevel truncate and portable-runtime branches. They predate the clean-room rewrite; implementations remain unreviewed and are not approved patches. |
| [Hybrid-Mount/rust-fs-ext4](https://github.com/Hybrid-Mount/rust-fs-ext4) | `d4e01155c2fc` | 4 | All branch metadata checked. C ABI signed/unsigned `c_char` and published-core dependency packaging changes. Current upstream already uses `c_char` and has ARM validation; old implementation not inspected. |
| [sntns/rust-fs-ext4](https://github.com/sntns/rust-fs-ext4) | `b57d5bb05358` | 7 | All branch metadata checked. Uninitialized group flag/bitmap preservation and unsigned-`c_char` targets are useful regression leads. Default head has no unique commits; side branches are not approved backports. |
| [NightProg/rust-fs-ext4](https://github.com/NightProg/rust-fs-ext4) | `0ec6e90f666f` | 1 | Default branch is entirely upstream history; no unique commits found. No new implementation review needed for this unchanged head. |
| [rcstor-2023/ext4-win-driver](https://github.com/rcstor-2023/ext4-win-driver) | `393665e6cf72` | 1 | GPL metadata only. No unique commits; ten commits behind driver upstream. Implementation not inspected. |
| [ByronScottJones/ext4-win-driver](https://github.com/ByronScottJones/ext4-win-driver) | `11b0261a0f60` | 1 | GPL metadata only. No unique commits; 98 commits behind driver upstream. Implementation not inspected. |

No PRs were listed in the external forks themselves at the checked snapshot.
That does not exclude contributions submitted from those forks to upstream.
"Unique commits" is a history comparison, not proof that their behavior is
missing upstream: rebased and squashed work can have different commit IDs.

### Useful follow-ups and decisions

| Lead | Relevance to shared Windows/Linux/SteamOS storage | Decision at this check |
|---|---|---|
| Recovery, explicit release, failed-handle retirement and cache invalidation (`1vivy`) | Interrupted writes, safe close/eject, stale cached state | Keep as scenario checklist. Current casefold has its own guarded effect/lifecycle work and existing upstream compatibility evidence. Establish a concrete missing case with native red/green and independent guest checks before any new implementation. |
| Legacy group checksums, uninitialized metadata allocation and multi-descriptor journals (`1vivy`, `sntns`, `zubvit`) | Card formats and larger transactions must preserve allocator/metadata consistency | Cross-check current tests and declared support. Current upstream contains dedicated tests for these families; old fork presence alone does not broaden casefold qualification. Protected historical implementations stay unread. |
| Multilevel truncate, holes and node disposal (`1vivy`) | File resizing and cleanup on fragmented/large files | Current upstream has multilevel truncate tests. Treat the fork as a test-scenario lead; no blanket old-code import. |
| C ABI `c_char` portability (`Hybrid-Mount`, `sntns`) | ARM/SteamOS and Windows adapter interoperability | Check each actual adapter target and current ABI layout. Existing upstream ARM coverage is evidence for its tested target, not Windows integration qualification. |
| In-inode attribute publication (`cyberphantom52`) | Small attributes must be visible to Linux and its checking tools | Already in this main baseline and the active casefold plan; two guest-oracle regression tests inspected. No duplicate implementation imported. |
| Driver upstream #45: failed stat must fail a listing rather than omit the entry | Windows must not silently hide files when metadata lookup fails | High-priority Stage 7 metadata lead. Review licensed adapter behavior and validate on a Windows runner when integration begins. No GPL implementation inspected or imported here. |
| Driver upstream #58: WinFsp dependency with guard-strategy correction | Adapter locking/lifetime behavior and compatible dependency selection | Stage 7 dependency/behavior review, together with actual caching, ownership, flushing and eject tests. Do not assume the casefold core alone solves this. |

Driver comparison reference: upstream
`876c6c781d77862ac899d3bdd8188b8b3a902ab5`; our driver remains
`393665e6cf7277422385154deeb8d2e4401dfa93`. Its ten newer upstream commits
were checked as subjects and changed-path metadata only. The driver checkout
and all existing siblings were left unchanged.

### Explicitly not checked or not qualified

- Private/unlisted forks, branches created after the recorded snapshot, and
  branches of upstream roots other than the core main baseline.
- Full implementation of old forks predating the clean-room rewrite; protected
  hash and restated routines were not opened.
- GPL Windows-driver implementation and source-level adapter correctness.
- New execution of the fork experiments or their claimed tests; metadata and
  test names do not establish passing results.
- Windows/card round trips, SteamOS hardware, safe eject, concurrent mounts,
  crash recovery for casefold operations, or activation of casefold writes.

### How a later chat should resume

1. Read this register and the JSON snapshot before repeating a fork search.
2. Re-list the two public fork networks, then compare branch names and tips
   against the snapshot. New repositories/branches and changed tips are unchecked
   until reviewed. An unchanged metadata-reviewed tip still has the implementation
   limits recorded above; it is not an implementation-approved tip.
3. Add a new dated snapshot and state its exact scope. Record unchanged, blocked,
   partial and not-reviewed cases explicitly. Never infer "looked at" from a
   fork's name appearing in a search result.
4. For main, diff from upstream `90073d6` to the new source, keeping dependency
   versions and upstream validation context explicit. For casefold, preserve its
   separate pins and test/effect gates and backport only a demonstrated need.
5. Resume the bounded project checkpoint in `extprogress.md`: counter-emission
   source `b14369d` now passes focused/full CI and saved-byte/complete-retention
   review. Prepare bounded ordinary indexed-parent reference evidence next.
   This review enables no casefold writes.

## CI follow-up on 2026-10-29

This follow-up checks validation evidence; it does not refresh the external-fork
inspection timestamp or expand its scope. Exact casefold source `bbc25e8` now
has full **40820596239** and focused **40820596147** closed: **2,502 successes**
in each whole suite, **1,651** in each unit context, **674 controls once in five
contexts**, all prior occurrence multisets retained, actual wrappers/floors and
real trailing checks passed. Seven tier archives have source/run/digest/all-CRC
verification. The existing ignored slow fuzz remains open. See
[the current project handoff](extprogress.md) for the detailed evidence and next
Rust/C driver-emission task.

The main import's own full run **40862413814** and notes-source casefold full
**40862413612** are still running without reported failures at this observation.
They are not credited as complete and must be checked at their exact source.
The main source `5045433` and unchanged casefold implementation remain separate
validation contexts, with core v0.4.2 and v0.3.0 respectively.

### Main import CI completed — 2026-10-29

The later exact-source check confirms main import **`5045433`** passed all
**twelve jobs**, including its aggregate, in **40862413814**. This closes the
pending main CI status above; it is an API source/run/job-conclusion check,
not a new full image or individual-log audit. It does not refresh the dated
external-fork snapshot, import more code, or qualify casefold/Windows/card use.

The subsequent API check also confirms notes-only casefold source **`5e9c3ac`**
passed all **eleven jobs** in **40862413612**. This resolves its earlier pending
status without claiming a new per-log/image audit. Its implementation remains
the separately audited `bbc25e8` source; newer counter-emission evidence and
remaining work belong to the current checkpoint in `extprogress.md`.

### Counter-emission checkpoint completed — 2026-10-29

Exact casefold source **`b14369d`** passes focused **40867303009** (16/16 jobs)
and full **40867302848** (11/11). Independent audits close **48 Rust/C links /
192 refusals / 240 saved images** against the fresh Linux reference, retaining
**2,503 successes in each whole suite / 1,651 units / 674 controls once in five
contexts**, all prior occurrences, actual limits/floors and trailing checks.
The ignored fuzz and broader write/platform gates remain open. See the current
handoff for exact source/artifact identity and the bounded indexed-parent next
task. This CI follow-up does not refresh the external-fork snapshot or approve
more source imports; main and casefold retain their separate core pins.
