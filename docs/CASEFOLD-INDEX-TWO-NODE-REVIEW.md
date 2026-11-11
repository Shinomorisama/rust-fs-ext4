# Ordinary two-intermediate-node insertion review

## Checkpoint and current gate — 2026-11-07

This checkpoint reviews ordinary hard-link insertion into a depth-one index
with two intermediate nodes. It records source behavior, checks saved Linux
predecessors and defines the next native/Linux matrix. It changes no code,
tests, selections, ceilings or write gates and qualifies no new writes.

Reviewed source is **`673c16fe5785be5c6f9e6c36be022a0c621a2dbe`**. Its latest
full CI **42339773792** completes with nine successful jobs and failures in
native x86_64 and `ci-ok`. Native stops in the oracle tier before its whole
suite and trailing checks. The earlier full-node checkpoint's exact passing
source/run remains historical evidence; this latest run is failed.

Job **128305339699** fails existing
`htree_index_descent_oracle::half_md4_signed`: five sibling algorithm/signedness
tests pass, while one of 1,500 names, **`a_name_of_twenty_01196`**, is absent
from the leaf returned by single-leaf descent. The test reports its assertion
at `tests/htree_index_descent_oracle.rs:144`; this is exit 101, not an output
overflow. Artifact **13561293488**, `tier-logs-x86_64`, has SHA-256
**`1ebce8882dbca439ac43b435f478792fb2a015cf3c9aef394a52b63f7888334b`**.
The primary verifies its source binding, advertised digest, ZIP CRCs and every
extracted byte. It contains exactly `oracle.log`, `unicode-tables.log` and
`unit.log`; it does not retain the failing image, hash seed or routing dump.
The cause and public lookup/mutation impact remain unestablished.

Implementation of the two-node qualification matrix must first address this
oracle gate with retained diagnostic evidence. A retry alone cannot establish
the cause or repair the failing expectation.

## Source review

| Path / boundary | Observed behavior and implication |
|---|---|
| `src/fs.rs`: `apply_link_at`, `plan_link_at` | Public links validate source/destination and duplicate status, stage source link count and directory changes together, then stamp source ctime and parent mtime/ctime/version once. Source/parent may share an inode-table block. |
| `src/fs/guarded_chmod.rs`: `guarded_link_from`, `guarded_link_with_attr`, `commit_link_in` | The operation context covers planning, preparation, dirty marking, commit/publication and optional C attribute readback. This remains a bounded trusted link plan, not generic ownership admission or crash consistency. |
| `src/fs.rs`: `buffer_add_dir_entry_indexed` | Ordinary insertion validates the root and selected intermediate checksum, descends to one leaf and adds there. A full leaf returns the growth signal. Two root routes do not introduce a special branch. The selected path is not a validation of all unselected nodes or cross-inode ownership. |
| `src/fs.rs`: `buffer_split_htree_leaf_and_add_entry` | At one intermediate level it chooses the routing parent from the query hash, plans a leaf split and adds a route to that selected parent. It appends the new leaf, then stages the selected leaf and node in one buffer. |
| `src/fs.rs`: `extend_dir_and_add_entry` | A private `Ok(false)` decline can invoke the public index-dropping fallback. A declined private split is not a promised public error/no-effect refusal. Multi-node fallback is outside this review's proposed positive scope. |
| `src/htree.rs`: `lookup_leaf_with`, `lookup_leaf_prepared` | The API returns a leaf that might contain a name. `find_entry_for_hash` chooses the last recorded boundary at or below the query. The latest oracle assertion demands that this single leaf always contain the name. |
| `src/path.rs`: ordinary indexed lookup | After a selected-leaf miss, the caller can scan the directory; comments identify collision spill as one reason. The failed artifact cannot establish whether continuation behavior explains this failure. |

Selected-leaf checksum/record errors must propagate before growth or index
dropping. A full selected node, repeated split hash, unsplittable hash run or
deeper tree is outside the proposed retained-index positive matrix. Existing
casefold/encryption and public write refusals remain in force. Broader malformed
geometry, physical role aliases and unselected-node corruption require separate
admission work; healthy inputs are a scope condition, not a proven rejection
policy for every unsupported shape.

## Saved two-node predecessors

The primary rechecks saved artifact **13556486296**, SHA-256
**`ec12e75cf962c0e2b51d9ad592bda012ebbe2a7b045c4613e1be83152fe06a84`**,
from focused **42328021164**, source `6c29e7f`. Its digest, ZIP CRCs and every
extracted byte agree; all **133** retained input hashes also match reviewed
source `673c16f`. No new Linux operation is executed in this checkpoint.

Each `link-node-driver-source-{block_size}-{journal}.node-grown.img` has depth
one, two root routes, two distinct nonfull nodes, disjoint leaf roles, complete
map/inventory coverage and valid saved checksums/oracle hash routes. Raw record
space is measured for another 255-byte name, requiring a 264-byte aligned
record. Counts below are per node in root-route order; the sentinel lower bound
of the second node comes from its root route.

| Blocks / journal | Aliases | Node logical blocks | Route counts / limit | Leaves with room | Full leaves | Second root boundary |
|---|---:|---|---|---|---|---|
| 1 KiB / no | 250 | 124 / 128 | 64 / 63, limit 126 | 40 / 42 | 24 / 21 | `8de314b2` |
| 1 KiB / yes | 262 | 124 / 128 | 63 / 64, limit 126 | 37 / 43 | 26 / 21 | `7856e036` |
| 4 KiB / no | 5,418 | 508 / 512 | 256 / 255, limit 510 | 242 / 230 | 14 / 25 | `7cf30232` |
| 4 KiB / yes | 5,315 | 508 / 512 | 256 / 255, limit 510 | 236 / 242 | 20 / 13 | `837d8b4e` |

Both routes therefore contain candidate leaf shapes for insertion with space
and for splitting. This does not establish that a chosen new name hashes into
one of them or that Linux/driver insertion passes. Future preflight must verify
the exact query, route, selected leaf space and selected node capacity before
calling either writer.

## Bounded qualification contract

Use the same four 32 MiB ordinary layouts: 1/4 KiB blocks, 256-byte inodes,
FILETYPE, metadata checksums, extents and dir_index, direct/journaled. Start each
case from its own exact copy of the measured two-node predecessor. Cover both
root routes and both operation classes below: **16 independent Linux cases**,
then **32 Rust/C driver profiles** using those exact inputs and query bytes.

| Operation class | Required before state | Required after state |
|---|---|---|
| Insertion with space | Query selects a leaf with room for its 264-byte record. | INDEX/depth/root/two-node routes and all maps stay unchanged; exactly the selected leaf changes; no block or inode allocation. |
| Leaf split | Query selects a full leaf under a node with a free route slot; the split can accommodate the query. | INDEX/depth/root remain unchanged; selected node count increases by one; its old routes remain and its new boundary stays inside the same root interval; the other node is byte-identical. Exactly the selected old leaf changes and one new leaf is appended. |

Record query bytes, effective algorithm, seed, major/minor hashes, root slot,
node/leaf logical and physical blocks and the complete before namespace. Require
unique non-continuation boundaries for this bounded positive matrix; unresolved
collision behavior remains a separate gate. Choose unused 255-byte candidates
with an explicit finite search budget (at most 16,384 candidates) and retain
selection evidence on exhaustion. Search candidates without mutating the input.
An unsuccessful search fails with evidence rather than substituting another
operation class. Preserve the existing preparation bounds: 16,384 live names,
256-link batches/eight bisections, capacity-derived last-slot search, 128
full-node seeks, 1,024 directory blocks and four external extent nodes.

Retain exact old names/inodes and add the query once. If route selection chooses
a later `cap-*` candidate, the expected namespace is the explicit predecessor
inventory plus that query; it cannot assume dense `0..total` names. Rework the
new guest inventory check explicitly, preserving earlier guest behavior.

For a split, debit exactly one directory block and separately measure any new
extent-map node. Preserve every old physical mapping/node, complete protected
inode-table slices, EAs, payloads and root-directory bytes; verify exact bitmap,
group/global counts and checksum changes. Insertion with space must preserve
allocation bitmaps/descriptors/free counts. Both classes preserve parent links,
increment source links once and parent version once, use one clock observation,
and validate exact source/parent metadata and C attribute records.

Keep duplicate, overlong-name and directory-source no-effect refusals. Native
controls must cover both root ranges and selected root/node/leaf corruption,
requiring unchanged device bytes, writes/flushes, clock, marker and health on
error. Full selected-node private declines remain distinct from public fallback.
Do not infer a no-effect public refusal from a private decline.

Retain Linux reference input/output/cold images and driver input/refused/dirty/
closed/cold images. Use judged non-repairing checks and separate cold Linux
copies with exact names/inodes, metadata, flags, EAs, payloads and free counts.
Verify complete raw roles, checksum/map/routing inventories, protected bytes
and namespace equality between Linux and each API. Dirty snapshots describe
observed post-call state, not interruption/recovery proof. Keep caller exclusion,
closed reference handles and healthy-layout assumptions explicit.

The current emitter asserts one root route for depth-one input; its split
checker asserts one intermediate node, and its linear observer assumes a block
append. It cannot model these four cases by merely accepting depth one. Add
explicit two-node operation/query context in separate support or a separately
reviewed mode; preserve all three existing modes, assertions and scratch names.
Reuse read-only complete indexed observation where its full-role checks apply.

## Next implementation sequence and validation

1. **First preserve the current descent failure.** Extend diagnostics without
   changing the six existing oracle assertions or test names. Always retain
   each completed formatter/rebuild image, exact fixture names, UUID/hash seed,
   algorithm/signedness, complete raw index/leaf inventories, tool reports and
   concise failure identity. Include the exact oracle test file in the source
   hash manifest, alongside source/support/dependency/workflow/script inputs.
   A focused six-test/floor-six harness job must always upload the evidence.
   Run all tools/mounts only in the prescribed guest. Repeated public-image
   observations may then distinguish hash disagreement, routing disagreement,
   malformed records or legitimate continuation candidates. Preserve an actual
   failing witness before changing production or the test expectation. Do not
   choose a convenient seed, add an ignored test, hide a mismatch or treat a
   green retry as diagnosis.
2. Resolve that oracle gate with a reproduced failure, a scoped correction and
   independent saved-byte validation. Retain all six algorithm/signedness tests
   and all 1,500 names per test, including long/high-byte spellings.
3. Add native two-node insertion/split and corruption controls under both root
   ranges. Preserve the exact 1,685-success unit baseline and all 34 current
   indexed-boundary controls; record added occurrences explicitly.
4. Implement the independent Linux cases, then Rust/C emissions and pinned
   focused jobs with explicit exact tests/floors and complete evidence. Keep
   all existing job bodies, selections, output, ceilings and floors. Any actual
   overflow needs identical saved-log old-red/new-green replay before a limit
   change. No preemptive limit increases.
5. Review actual source-bound images/reports and complete test-name multisets
   in every unit/whole context; verify every actual floor and trailing verdict
   plus all focused/full jobs and `ci-ok` before scoped qualification.

This review and saved-input audit pass. The latest oracle failure is confirmed
but unresolved; its diagnostic capture, new native tests, new Linux operations,
two-node emissions and new qualification are unrun. No helper is used: the
previous Terra audit reached its usage limit and is not retried here. Stages
4–5 remain partial/not quantified, casefold writes blocked and stages 6–7 not
reached. Windows/SteamOS/card, deeper writes, generic ownership/admission and
lifecycle/recovery/ignored-fuzz qualification remain open.

## Retained descent diagnostics preparation — 2026-11-07

The six existing algorithm/signedness tests now retain their observations
without changing the 1,500-name generator or the selected-leaf mismatch
assertion. Test-only support records formatted, configured and rebuilt 64 MiB
images with SHA-256 sidecars; formatter/configuration/rebuild streams and
statuses; judged non-repairing checks; filesystem UUID, seed and effective
algorithm; complete raw directory/inode bytes, logical/physical mappings,
root/node routes and leaf inventories. Each query records raw name bytes,
computed major/minor hash, selected leaf, actual containing leaves and the
original membership verdict. Independent debugfs tree/stat reports and a raw
1,500-request/answer hash batch retain separate oracle observations.

`scripts/test.sh` removes its owned scratch directory on failure as well as
success. Evidence therefore lives in a separate repository-local
`tmp/htree-descent-evidence/` directory, with unique profile instances. Completed
snapshots survive cleanup; normal panic unwinding also attempts to retain the
current working image and records copy errors without masking the original
panic. A standalone local probe uses the actual helper to verify that completed
and failure-time bytes remain after deleting their original scratch directory;
it invokes no oracle or mount. No new test names enter the six-test binary.

A new twenty-fourth focused job runs that complete binary with floor six,
the existing pins, 4,200/340,000 initial ceiling, exact source hashes (including
the oracle test/helper, workflows, dependency/chores/toolchain and scripts) and
an always-retained artifact. Every previous focused job body is semantically
unchanged; only the new job and two relevant path triggers are added. Native
and guest full-CI jobs add separate evidence uploads when these tests are
reached, preserving every existing tier-log archive path and all test commands.
Thus a failure unique to a full-suite seed can also retain its witness.

Compilation, formatting, full lint/eight Unicode checks and exact **1,685**
native unit successes pass, with unchanged **1,932-line / 150,431-byte** output.
The six existing test names, all generated names and the final mismatch
assertion are unchanged; workflow semantic comparisons and budget/VM-order
guards pass. Linux execution and actual saved-artifact review remain pending.
This is diagnostic preparation, not a hash/routing correction or new two-node
write qualification. A green diagnostic run does not explain the earlier red;
a retained failing witness and independent classification are still required.

## Diagnostic execution and independent artifact audit — 2026-11-07

Exact preparation source **`4981c7b67d63a3e8501733f3bdb507f249ebdd59`** passes
focused [**42357296709**](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/42357296709),
all **24** jobs and **39** actual positive floor verdicts. Diagnostic job
**128358071475** executes the six unchanged tests against floor six. Its actual
wrapper is **186 lines / 12,567 bytes**, within unchanged 4,200/340,000 limits.
All **143** unique source-manifest paths match the exact Git tree with no
missing or extra inputs, including the oracle test, helper, workflows and scripts.

GitHub API/Git authentication failed during the initial collection, after the
source was pushed; collection resumed once existing access worked again. The
primary verifies source/run bindings, advertised SHA-256, ZIP CRCs and every
extracted byte of the following artifacts:

| Artifact | ID | SHA-256 |
|---|---|---|
| Focused descent diagnostics | 13568655816 | `5142b6a5b6b1172b247ceb7edda0502d8fd8e1714ff15db0d714597f000e0635` |
| Full-CI native descent evidence | 13570007027 | `7f6dc6dfc65bc09e32bedbee4dde94a2057e3c7601722c41bd186aef67f30366` |
| Full-CI guest descent evidence | 13569434372 | `c601d7f33a3e9b979451e5179829d18f6c99fcfbf9242fc120db8fc217872aa4` |
| Original failed guest tier log | 13569618093 | `dd51b392cd83fa3e515b9b1cfec4a37e16dba221ec8534d3340904ec5de103af` |

Independent byte review covers all six algorithm/signedness combinations in
each execution context. Native runs the binary in the oracle tier and again
in its whole suite; guest runs it once. Every profile retains all three completed
64 MiB phase images with verified digests and unchanged UUID/seed across phases.

| Context | Profiles | Completed phase images | Query records | Independent hash answers | Selected-leaf misses / hash disagreements |
|---|---:|---:|---:|---:|---|
| Focused | 6 | 18 | 9,000 | 9,000 | 0 / 0 |
| Native oracle and whole suite | 12 | 36 | 18,000 | 18,000 | 0 / 0 |
| Guest whole-suite prefix | 6 | 18 | 9,000 | 9,000 | 0 / 0 |
| Total | 24 | 72 | 36,000 | 36,000 | 0 / 0 |

The 24 filesystem seeds and UUIDs are distinct; these counts span all six
variants rather than 24 signed half-MD4 trials. Raw superblock/inode/extent/
index/leaf checksums agree. Inode-derived physical mappings reconstruct the
saved directory bytes; route records and complete raw entry membership agree
with every saved query. All 1,500 raw names per profile, including long/high-byte
spellings, and every raw independent hash request/answer are retained and
compared. All saved non-repairing reports run five passes and are judged clean.
These observations validate capture completeness on passing profiles. The
local actual-helper unwind probe separately validates failure-time retention;
no new failing oracle witness occurs in these artifacts.

## Native retention and incomplete guest run

Full [**42357296778**](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/42357296778)
initially completes with nine jobs successful and guest/`ci-ok` failed. The
primary verifies all seven tier-log archives and compares successful test-name
multisets against the preceding complete baseline. Native retains exactly
**2,544** whole-suite successes and the same ignored fuzz case; all three unit
contexts retain **1,685**, with all **34** boundary controls intact. Oracle/
kernel/lwext4 remain 225/36/9; ARM images 527 and browser tests five. Twenty
trailing shell and 159 CLI checks execute with baseline-identical logs. Native
executes the actual six-tier/**4,658-execution** verdict. Its actual whole wrapper
is **3,979 / 245,361**, oracle **3,055 / 239,103**, within unchanged ceilings.

Original guest job **128358614482** passes all six descent tests, then its
transport stops while `htree_split_write_cut` is running. The saved log ends
with Vagrant reporting that its provider is not ready for SSH; the wrapper
exits **255**, with no completed verdict for that test or the rest of the suite.
This is not whole-suite completion or an output-budget failure. Its already
completed descent artifacts are valid observations, not a passing guest gate.
The transport's underlying cause is not established by that message.

After preserving the original log, a failed-job retry retains the nine
successful jobs and reruns guest/`ci-ok` on the same source. Guest retry job
**128570238103** is running at this checkpoint; its final result and complete
guest retention are pending. No source, test, ceiling or floor changes accompany
the retry, and the original failure remains recorded.

## Result and next bounded task

The diagnostic capture and saved-artifact audit checkpoint is complete. It
does not resolve the `673c16f` signed half-MD4 selected-leaf mismatch: that
incident's image/seed were not retained, and the new observations contain no
failing witness. No production or oracle-expectation correction is claimed.

Next, verify the guest retry and its final source-bound evidence, then prepare
a finite reproducibility experiment prioritizing signed half-MD4. Preserve
the six current assertions and names, document candidate seeds/inputs and the
stop condition, and retain any actual failing image and independent tool
reports for classification. Seek a failing witness rather than pinning a seed
that avoids the failure; passing samples cannot establish its cause. A scoped
correction and new two-node mutation implementation still require that gate.
No helper is used; all audits are performed by the primary. Stages 4–5 remain
partial/not quantified, casefold writes blocked, stages 6–7 not reached and
Windows/SteamOS/card use unqualified.

## Guest retry closure and finite reproduction plan — 2026-11-11

This is an evidence/design checkpoint. It implements no new experiment runner,
changes no executable input or assertion, and qualifies no new writes. The
original `673c16f` signed half-MD4 miss remains unexplained.

### Retry evidence and present baseline

Full run [**42357296778**, attempt 2](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/42357296778)
now completes successfully on exact source
`4981c7b67d63a3e8501733f3bdb507f249ebdd59`. Guest retry **128570238103** and
aggregate **128588191192** succeed, retaining the nine successful peers. The
original failed guest **128358614482**, its incomplete suite and transport
error, and original artifact **13569618093** remain historical failed evidence.
They are not replaced by the retry's observations.

| Retry artifact | ID | Verified SHA-256 |
|---|---|---|
| Complete guest tier log | 13598781476 | `1939e8a8cc62af5c1d707c38cb0bf0767105a4ba3c3927b635e303acb64d17c3` |
| Guest descent evidence | 13598975326 | `f06cd9efed4eab578d9c276e39b42580fb87afc66bd8734ab227c44287a635c9` |

API source/run binding, actual downloaded ZIP SHA-256, all ZIP CRCs and extracted
bytes pass. The job log executes the real **2,544-test / floor-1,060** verdict;
the wrapper measures **4,151 lines / 255,415 bytes** within current limits.
The saved guest log has 2,544 successes, zero failures and the same one ignored
`deep_walk_byte_flip_fuzz_never_panics`. All six descent names pass. Its library
binary has **1,400** successes; do not attribute the separate native unit tier's
1,685 total to that guest binary.

Read-only saved-byte checks pass six profiles, all eighteen image sidecar
digests, unchanged UUID/hash seed across each profile's three phases, and six
distinct UUIDs/seeds. Inode-table bytes match the saved inode; independent
depth-zero extent parsing confirms all **305** logical/physical mappings and
reconstructs the saved directory. Raw record parsing confirms all **9,000**
leaf names/inodes against the inventory. Each of the 9,000 queries selects a
containing leaf, and every saved independent hash answer and request name
agrees. Reports retain `Clean` and successful hash-tool statuses. This retry
audit does **not** independently recompute checksums or index routing, nor
execute another oracle; the earlier full raw audit is distinct evidence.

Current pre-checkpoint source **`6c6fec8ff36e8fcbaaf028db6906a5c42ffa670f`**
passes full [**42428433787**](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/42428433787),
all eleven jobs. Native artifact **13599185548**, SHA-256
`316d44f31fcc0fdfe9cf732cbd9a0573ba3ee3cfcaf1ae52bba583b26d33e935`,
and guest artifact **13598658676**, SHA-256
`2a74133f96655bad9f3fbd3622fe980957568ce26ef262388d9c5e61816cf6ab`,
pass downloaded digest/CRC/extracted-byte checks. Native/guest successful-name
multisets and the retry agree at 2,544 occurrences / 2,530 distinct names, with
the same ignored fuzz. Native units retain 1,685 successes; oracle/kernel/
lwext4 remain 225/36/9. Fresh local native setup/CLI and unit execution pass:
1,685 occurrences, zero failed/ignored, floor 440, at 1,934 / 150,573 output;
its passing-name multiset equals that CI unit tier. This is not fresh full-suite
execution of the documentation commit. Executable inputs (`src`, `tests`,
dependencies, toolchain, chores, scripts and workflows) are unchanged from
`4981c7b` through this checkpoint; fresh documentation-commit CI is separate.

Documentation-checkpoint validation passes full `chore lint` through the native
runner (including provenance, family/aggregate gates, eight Unicode controls,
formatting and workspace/all-target clippy), `git diff --check`, local document
targets and the replay/corpus/32-candidate identity and resource calculations.
No new test execution or filesystem qualification is attributed to the plan.

Current-chat network/source/tool/write-path checks pass. GitHub reports fork
pull/push authority; write scopes were checked by metadata, without dummy
mutations. Direct storage downloads return a proxy 403; the GitHub connector
successfully supplies actual logs/artifact ZIP bytes for verification. No new
secret, host oracle, sibling reset or PRoot is used. One temporary authorized
Terra/medium read-only helper checked retained evidence; the primary inspected
its scripts and reran the consequential checks. It initially stopped at direct
storage denial and required the connector fallback. Usage/cost is unavailable;
no cost saving is asserted.

### Finite experiment to implement next

The next implementation checkpoint prepares a separate **test-only** seeded
runner and bounded focused CI path. Do not change the six current test names,
their 1,500-name generator, assertions, existing job bodies, selections, limits,
floors or ignored-fuzz gate. Keep production code and the write gate unchanged.
The existing complete six-test/floor-six job remains mandatory alongside the
new experiment. Run formatter, rebuild, hash and consistency tools only in the
prescribed guest with the existing pinned Rust/core/harness/tool setup.

Use the exact existing 64 MiB / 1 KiB-block corpus, including its long and
high-byte names. The 1,500-line `names.hex` SHA-256 is
`8fa9e9ecaaeebae3085ab85d2fc21f4f2946638db6af70c2b3ee0e6ee4b3e30b`.
Require that identity for every trial. Prioritize signed half-MD4 (effective
version 1, flags 1); do not substitute casefold Unicode hashing for this ordinary
directory failure.

1. **One exact-input replay control.** Verify retry artifact **13598975326**
   and take a private copy of `half_md4-1`'s `configured.img`, SHA-256
   `7aabbbb9c73d4b7d400d7b74e9d69387c30c87152bb1209194f2bc4952ff19c1`.
   Its UUID bytes are `c0ecd51e9d34484a87bed3c1ac73065d` and seed bytes
   `6d2c9f2f80bf492db620a3c18010d6c9`. Retain the original phase images and
   rebuild only the copy. Compare all names, hashes, routes and verdicts; require
   semantic agreement, not byte equality of time-varying tool output. This
   replays a **passing** witness, not the lost original failing fixture. Artifact
   unavailability stops this prerequisite rather than substituting a new image.
2. **Exactly 32 predeclared fresh candidates, indices 00–31.** For each index,
   take the first sixteen bytes of SHA-256 of the ASCII string
   `casefold-descent-2026-11-11/seed/NN` as raw seed bytes and of
   `casefold-descent-2026-11-11/uuid/NN` as raw filesystem UUID bytes; `NN` is
   the two-digit decimal index. UUID notation groups those bytes in order;
   it does not reverse words or modify UUID version/variant bits. For index
   00 the seed is `ef274b78-3e6f-9950-4f60-7095197e643e` and UUID is
   `c2e9fe60-07be-94f9-504e-7f6eb52da206`. Materialize and retain the complete
   manifest before execution. Check uniqueness, nonzero seeds and no match
   to the replay control. Do not select/reject candidates using driver hash
   answers or previous pass/fail outcomes. Confirm the guest's seed-setting
   interface, then read back raw superblock UUID/seed/algorithm/flags before
   rebuilding; a mismatch is a setup failure, never a reason to reseed.
3. **Finite execution and retention.** Run the replay before fresh candidates,
   then four batches of eight fresh indices in order, with at most nine trials
   in a batch including the initial replay. Each trial executes all 1,500
   original membership checks and records an actual floor-one verdict.
   The maximum is **33 signed trials / 49,500 queries / 99 phase-image slots**;
   these are planned maxima, not executed evidence. At 64 MiB each that is
   6.1875 GiB of phase-image bytes across the run, at most 1.6875 GiB for the
   nine-trial batch, before working copies/tool reports/build/VM space. Check
   total available space before starting each batch. Archive/upload each batch
   independently with always-retained evidence; keep data until its upload and
   downloaded digest verification are confirmed. Use existing tier wrappers
   and measured limits per invocation; no preemptive budget increases. Bound
   each batch to forty minutes and do not extend automatically on exhaustion.
4. **Stop and classify.** Stop scheduling further trials on the first captured
   membership/hash/routing discrepancy. Preserve the actual failed image,
   completed phases, partial query records, exact trial manifest, source hashes,
   tool versions, filesystem settings, complete raw roles/maps/routes/leaves,
   independent tool reports, checksums and original exit status. Artifact or
   transport failure also stops execution, recorded as infrastructure/incomplete
   rather than a filesystem witness. No ignored test, automatic retry, assertion
   weakening or production repair may turn that attempt green. After 32 fresh
   candidates without a witness, stop as **failure not reproduced**; the oracle
   gate remains unresolved and another experiment needs separate review.

Before running the new path, verify deterministic candidate/corpus identity,
actual seed readback, exact-source capture on success and failure, retention
after owned scratch cleanup, and per-trial exit/floor handling. Preserve the
complete baseline test-name multisets and all focused/full checks when adding
the test-only runner. A setup failure is not a reproduction of the old miss.

Classification requires independent saved-image evidence distinguishing hash
disagreement, route/record disagreement, corruption and a possible continuation
case. A continuation explanation requires demonstrating equal major hashes,
the raw boundary flag, the actual containing leaves and independent lookup;
it cannot be inferred from a passing retry. Reproduce the same captured failure
before any scoped correction or change to oracle expectations. The sixteen
Linux / 32 Rust-C two-node mutation matrix remains later work.

This finite plan is prepared and its corpus/control/seed identities are checked;
the seeded runner, seed-setting preflight, 33 trials and their independent
qualification are **unimplemented/unrun**. Stages 4–5 remain partial/not
quantified, stages 6–7 not reached, casefold writes blocked and real
Windows/SteamOS/card use unqualified.
