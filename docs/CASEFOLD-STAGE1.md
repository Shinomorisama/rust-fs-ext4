# Casefold Stage 1A: recognizing metadata

This step teaches the library to report which filename rules a directory
declares. It does not implement those rules. Casefold writes remain blocked;
mount, journal replay, repair, lookup and hashing behavior are unchanged.

## Scope and design

- `Superblock::encoding_id` and `encoding_flags` read the two little-endian
  fields at 0x27c and 0x27e. They return errors for incomplete fields and retain
  unknown values for diagnostics.
- `Superblock::casefold_encoding` recognizes encoding ID 1 (UTF-8/Unicode
  12.1) and flags 0 or 1 (strict). It refuses other IDs or flag bits. Without
  the CASEFOLD feature it ignores these unused fields and returns `None`.
- `InodeFlags::CASEFOLD` names bit 0x40000000. The user-modifiable mask
  still excludes it; naming the bit does not permit setting it.
- `Inode::directory_casefold_encoding` distinguishes ordinary byte-sensitive
  directories from directories declaring casefold. It rejects encrypted
  directories, inconsistent feature/flag combinations and non-directories.
  When the volume declares CASEFOLD, even an ordinary directory's inspection
  refuses an unrecognized volume encoding.

These additive Rust inspection methods are the single classification path.
Making them callable independently allows diagnostics and integration tests
without prematurely changing mount behavior or adding unused internal code.
The returned `CasefoldEncoding` enum is non-exhaustive. No mandatory fields
were added to publicly constructible structs, and the C ABI is unchanged.
The methods read the supplied current metadata; there is no cached policy
that could become stale after a superblock reload.

This is deliberately not a complete filesystem validator. In particular,
recognizing an encoding does not admit a volume carrying ENCRYPT, MMP or other
unsupported features. Existing feature gates still make those decisions.
The old generic folding helper is not connected to these methods or to lookup.

## Evidence and checks

`tests/casefold_metadata.rs` covers raw byte order, every unknown encoding ID
and flag combination, truncated buffers, both strictness modes, absent feature
bits, per-directory settings, encryption, inode types and refreshed metadata.
It also checks that the named inode flag remains outside the writable mask.
The existing mount/replay and flag-setting regression tests remain intact.

`tests/casefold_fixture_oracle.rs` now checks the same API against the four
Linux-created profiles from Stage 0: 1 KiB/4 KiB and strict/non-strict, with
ordinary, small folded and indexed folded directories. Its existing checks
still require e2fsck's clean verdict and byte-identical images after refused
writable mounts. See [the oracle profile](CASEFOLD-STAGE0.md) for qualification
limits; these checks do not qualify a specific SteamOS release.

Run local checks with `chore test:unit`, `chore cli:install` and `chore lint`.
Full CI and the casefold evidence workflow run live VM checks. The casefold
coverage workflow now measures the selected commit instead of always checking
out the old baseline; manual dispatch accepts an exact SHA to reproduce an
earlier measurement. Evidence records the actual commit, tool and sibling
versions. Check completed workflow results before calling this stage validated.

## Pause and next gate

Stop after this bounded change. Before Stage 1B, confirm that all baseline
passing tests remain, live metadata checks pass, coverage meets its prior floor,
and API compatibility, CLI, lint and other full CI checks succeed.

Stage 1B should apply the validated encoding policy throughout mount and
metadata reload/recovery, without removing CASEFOLD write refusal. Some existing
synthetic mount fixtures declare CASEFOLD with encoding ID zero; their encoding
must be deliberately corrected when enforcing that new validation, with their
original safety assertions preserved. Unicode tables, folded lookup, namespace
mutation, and enabling writes each require later independent stages.
