# Ordinary full-root hard-link qualification

## Reviewed boundary — 2026-11-06

The measured Linux reference promotes a full root to one intermediate level.
The existing ordinary driver follows a different path: `plan_link_at` builds
the owned `LinkPlan`; insertion's full-leaf signal reaches
`extend_dir_and_add_entry`; `buffer_split_htree_leaf_and_add_entry` returns
`Ok(false)` when the routing parent cannot accept another entry; then
`buffer_drop_htree_index` converts index records, clears INDEX and appends one
linear directory block in the same buffer. `commit_link_in` retains the mount
operation through optional-superblock preparation, commit, publication and
readback. Corrupt selected-leaf records/checksums propagate errors before that
fallback. Casefold index dropping and public casefold writes remain refused.

This batch qualifies that existing ordinary fallback on healthy root-only
inputs; it does not implement Linux-style promotion. Earlier full-parent
native controls repeat one leaf reference to isolate a private capacity check.
New controls must use all 123/507 physical route slots with distinct leaves,
strict even boundaries and names routed to their actual leaves. Synthetic
native fixtures are not Linux-clean claims.

## Native contract

- At 1/4 KiB, build a complete full root with distinct mapped/checksummed leaves
  and a genuinely full selected leaf. Bound name generation by the root limit
  and the maximum number of 255-byte names per leaf.
- Require private decline to preserve the entire buffer/publication state,
  device, clock, dirty marker and operation health.
- For Rust and C public links, require success, INDEX cleared, exactly one
  appended directory block, unchanged old mappings and names, one source-link
  increment, no inode allocation and a single clock observation.
- Corrupt selected records/checksums must refuse with no publication, device
  writes, allocation, clock observation or health change. Retain all preceding
  safety controls and write gates.

## Independent emitted-byte contract

Reuse the bounded Linux reference's exact full-root predecessor and triggering
name, not a fabricated capacity limit. Four Linux profiles (1/4 KiB, 256-byte
inodes, checksummed, direct/journaled) feed both public Rust and C calls: eight
driver profiles. Preserve the original Linux predecessor and promoted output.
Retain driver input, refusal, dirty, closed and separate cold-readback images.

Before mutation, require the complete healthy root-only role/map/inventory
contract, genuine capacity and the actual selected full leaf. Exercise duplicate,
overlong-name and directory-source refusals without byte/clock changes. Require
the driver's ordinary fallback to retain every old spelling/source identity,
clear INDEX, append exactly one linear leaf, preserve old physical mappings and
extent-node addresses, account for any new extent-map node, and preserve all
protected inode-table slices, EAs, payloads and root-directory bytes. Source and
parent actually share an inode-table block. Verify exact group/global bitmap
debits and no inode allocation, source/parent metadata and C attribute readback.

Use judged non-repairing e2fsck and cold Linux lookup of every spelling, payload,
EA, flags, exact metadata and free count. Separate cold copies must preserve
the closed driver's protected bytes and mapping/allocator/table state. Linux
and driver namespace inventories must agree; their INDEX state and allocation
representations are deliberately measured separately.

Keep 32 MiB images, at most 16,384 live names, 256-link batches, eight replay
bisections, 1,024 directory blocks and four external extent nodes. The focused
job pins the existing siblings/toolchain, records exact source hashes, enforces
one executed test and always retains evidence. Existing jobs/caps/floors stay
unchanged. Full-suite output changes require actual measured replay before any
ceiling adjustment. No host oracle/mount or local KVM substitution is allowed.

## Status

Steps 1–2 are complete. Six distinct-leaf native controls pass at both block
sizes, including private decline, Rust/C positive fallback and corrupt selected
leaf refusals. Full units retain every 1,669 baseline occurrence and add exactly
six controls: 1,675 successes, 1,923 lines/149,531 bytes within unchanged limits.
Full lint and formatting pass. Driver emission, focused CI and saved-byte
qualification remain pending. Step 3 now implements the eight-profile driver
test with independent Linux preparation, exact one-block fallback accounting,
protected bytes, retained C attributes and cold metadata/inventory checks.
Its unique scratch prefix permits concurrent complete-suite execution alongside
the original reference. Debug compilation, full 1,675-unit retention, Python
syntax, formatting and lint pass. Actual Linux execution is pending steps 4–5,
which launch the focused job and audit its evidence. Stages 4–5 remain partial and
unquantified; ownership/admission, lifecycle/recovery, ignored-fuzz, casefold-write
and platform gates remain in place.
