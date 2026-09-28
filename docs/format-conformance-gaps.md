# Format-conformance gaps

Where this implementation and the ext4 on-disk format disagree, and where
they used to.

This is not a feature plan (`ext4-full-write-support.md`) or a code-quality
list (`IMPROVEMENT-PLAN.md`). It is about one axis: places where the crate
**accepts a filesystem it does not fully understand**, or reads a field in a
way the format does not sanction. What they share is that they fail
*quietly* — a mount succeeds, and the wrongness shows up later as bad data
rather than as an error.

Started 2026-09-04 as a list of open gaps. Rewritten 2026-09-27 as current
status, because most of them have since been closed and a list of caveats
that overstates the gaps is only slightly better than one that understates
them. The gap numbers are kept: `tests/feature_matrix.rs` and the CHANGELOG
refer to them.

---

## The general shape: in the mask is a promise

`features.rs` keeps two masks, `SUPPORTED_INCOMPAT` and
`SUPPORTED_RO_COMPAT`, and refuses to mount anything carrying a bit outside
them. A bit inside a mask is a promise that the filesystem will be read
correctly. Bits that can be read but not safely written are in a third
list, `WRITE_BREAKING_INCOMPAT`, and a volume carrying one mounts read-only;
`MAINTAINED_RO_COMPAT` does the same job on the RO_COMPAT side.

| bit | status | where |
|---|---|---|
| `BIGALLOC` | read; not written | G1 |
| `INLINE_DATA` | files read in full, including the `system.data` spill; inline directories read only their `i_block` half | G3 |
| `MMP` | read-only: writable mounts refused | G4 |
| `CASEFOLD` | read-only: writable mounts refused; case-insensitive lookup not implemented | below |
| `ENCRYPT` | encrypted inodes refused per inode; writable mounts refused (#76) | — |
| `RECOVER` | journal replayed (`journal_apply`) | — |

**`CASEFOLD` is not implemented, only tolerated for reading.**
`casefold.rs` has the casefolded name hash and no callers: neither
`s_encoding` nor `EXT4_CASEFOLD_FL` is used to choose a hash. Reads work
because `path::find_entry` falls back to a linear scan when the htree
descent misses, so a name is found by its exact bytes and never
case-insensitively. A write would file an entry into the leaf the wrong hash
names, where the kernel cannot find it, so `CASEFOLD` is in
`WRITE_BREAKING_INCOMPAT` (#100). The comment on the constant says the same.

---

## G1 — BIGALLOC: closed, read-only

bigalloc volumes mount and read (#237). The premise the gap was written on —
that bigalloc moves every block-group offset — was wrong:
`s_blocks_per_group` stays the group stride in blocks, and only the bitmaps
and the descriptor free counts count clusters. The first attempt to lift the
refusal failed on a separate defect (the descriptor table located after
`s_first_data_block`, which bigalloc forces to 0 on 1 KiB blocks), which is
recorded on `bigalloc_mounts_and_reads`.

bigalloc is not in `MAINTAINED_RO_COMPAT`, so such a volume is not written.

**Pinned by** `tests/feature_matrix.rs::bigalloc_mounts_and_reads` and
`tests/bigalloc_read_oracle.rs`, which compares file contents with what
e2fsprogs reads.

---

## G2 — timestamps past 2038: closed

`Inode` stores seconds as signed `i64` and applies the two epoch bits of
each `*_extra` field (`inode::decode_extra_time`), so 1901-12-13 through
2446-05-10 read correctly, before 1970 included. `fs_ext4_utimens` encodes
the epoch bits on write (`inode::encode_extra_time`) and refuses a time
outside that range.

**Closed for the driver's own clock too (#324).** The times the driver
stamps itself — `ctime` on every attribute change, and the times of a new
inode — come from `Runtime::now_unix_seconds`, which returns an `i64`, and
go through `inode::set_inode_time`, which writes the epoch bits with the
base. An inode with no `*_extra` word for the field (a 128-byte inode) gets
the time clamped to 2038-01-19 03:14:07, as the kernel does, rather than
wrapped. `i_dtime` stays the kernel's unsigned 32 bits of the clock.

**Pinned by** the `inode.rs` unit tests (`the_epoch_bits_extend_the_range_past_2038`,
`a_pre_1970_timestamp_stays_negative`, `timestamps_round_trip_across_the_2038_boundary`),
`tests/capi_utimens.rs::utimens_round_trips_a_date_past_2038`,
`tests/runtime_provider.rs`, and `tests/timestamps_past_2038_oracle.rs`,
which reads the driver's own stamps back with `debugfs`. The feature
matrix does not cover timestamps: they are not a feature bit.

---

## G3 — INLINE_DATA spill: closed for files, open for directories

`inline_data::read_all` reads the `i_block` part and then the
`system.data` extended attribute, and a file larger than the inline area
whose `system.data` is missing or short is reported as corrupt rather than
returned truncated.

**Still open.** An inline *directory* is searched only in `i_block`
(`path::find_inline`); entries that spilled into `system.data` are not
found.

**Pinned by** `tests/capi_inline_data.rs::reads_medium_inline_file_with_xattr_overflow`
and `inline_data.rs`'s `a_missing_spill_xattr_is_corruption_not_an_empty_tail`.

---

## G4 — MMP: closed by refusing writes

Multi-Mount Protection is still not honoured — nothing reads the MMP block,
checks its sequence or claims it. Instead `MMP` is in
`WRITE_BREAKING_INCOMPAT`, so a volume carrying it mounts read-only, which
cannot interfere with another host's protection. Honouring it is what would
let the bit leave that list.

**Pinned by**
`tests/feature_matrix.rs::an_mmp_filesystem_is_refused_for_writing_but_allowed_read_only`,
and for the lazy-mount paths by the `write_breaking` tests in `fs.rs`.

---

## G5 — the feature masks tested against real images: closed

`tests/feature_matrix.rs` formats a volume with a given feature and asserts
the intended behaviour — a full read or a clean refusal. It covers
bigalloc (G1), an ordinary ext4, an RO_COMPAT bit the reader does nothing
with (`project`), and MMP (G4). It checks feature bits, so it would not have
caught G2, which is a field encoding rather than a feature; the timestamp
tests under G2 are what cover that.
