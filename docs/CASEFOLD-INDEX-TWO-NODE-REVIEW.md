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
