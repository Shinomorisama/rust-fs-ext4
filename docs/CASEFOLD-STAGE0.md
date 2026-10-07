# Casefold oracle and coverage baseline

This stage adds test infrastructure. It does not enable casefold writes or
claim that the driver's name lookup implements casefold semantics.

## Baseline

The baseline driver source is v0.8.0, `204d8f6518ca5d66de07ab8e84c81501909b5d09`.
Commit `3b9bc5d19dcfe05a00289dd8b1cebbf11e49b742` adds only the branch's CI
trigger. Its complete CI passed in
[run 37593892509](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/37593892509).
The sibling pins are core v0.3.0 (`9e900154a7dff2a6c8f136db31def33bed9f09cc`)
and harness v0.4.0 (`7e0a84291c76fe4c762143f6f6d5787cc11749e7`).

The local debug unit-only measurement with Rust 1.95.0 and cargo-llvm-cov 0.9.1
executed 712 tests. It covered 20,143 of 24,750 reported source lines (81.39%)
and 37,138 of 45,390 regions (81.82%). This is not the full Linux-suite metric.

The separate `casefold baseline coverage` workflow checks out `3b9bc5d`
explicitly, verifies both sibling SHAs and builds fixtures from that baseline's
recipe in the guest. It does not depend on an expiring CI artifact. `scripts/casefold-coverage.sh` instruments all executable library,
binary and integration tests in release mode, including the slow ignored fuzz
test. Documentation snippets are outside this stable-toolchain metric. The
script retains the passing test names, tool versions, source revision,
Cargo.lock hash and JSON/text coverage reports. It fails on missing tests,
failed tests or ignored executable tests. The completed
[coverage run 37657670773](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/37657670773)
passed 1,526 executable tests with none ignored. Its inspected artifact reports
22,767 of 24,745 source lines (92.01%) and 41,930 of 45,359 regions (92.44%)
covered. Branch coverage was not measured. Release and debug line denominators
differ; compare future results using the same profile and tool versions.

## Initial fixture matrix

`tests/casefold_fixture_oracle.rs` uses the existing oracle and kernel helpers.
The `casefold Stage 0 evidence` workflow runs it separately and saves its images
and reports. It is also discovered by the normal kernel test tier.

| Block size | Encoding | Encoding policy |
| --- | --- | --- |
| 1 KiB | UTF-8 12.1 | Non-strict |
| 1 KiB | UTF-8 12.1 | Strict |
| 4 KiB | UTF-8 12.1 | Non-strict |
| 4 KiB | UTF-8 12.1 | Strict |

Each 64 MiB image has a journal, metadata checksums, a fixed UUID/hash seed and
an explicit filesystem feature list. Linux creates an ordinary directory, a
small `+F` directory and a `+F` directory large enough to become indexed. The
guest verifies actual flags and distinct/equivalent inode lookups, including
an NFC/NFD spelling pair. Names are passed as explicit bytes. The manifest
records name bytes, inode numbers and content hashes.

The test requires a clean e2fsck verdict, checks the superblock encoding fields
and the actual directory indexing flags, then verifies the driver's writable
mount is refused without changing any image bytes. Original superblocks,
image SHA-256 hashes and guest reports are retained under `tmp/casefold-stage0`.
Mount timestamps and inode numbers may differ between builds; these are
structurally reproducible fixtures, not byte-identical images.

## Oracle identity and remaining gates

All filesystem tools and mounts run inside the harness guest. All four profiles
passed in [run 37655559368](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/37655559368).
The measured reference identity is frozen in
`test-disks/casefold-oracle-profile.json`: x86_64 Linux 6.1.0-53-amd64, kernel
package 6.1.187-1, e2fsprogs/libext2fs 1.47.0-2+b2, and Python 3.11.2. The
guest refuses a different profile; other guest architectures or package updates
require a separately qualified profile rather than silently changing this
reference. This is a test-oracle restriction, not a driver architecture limit.

`test-disks/casefold-reference-manifest.json` records the captured image hashes,
feature masks, encoding flags and hash settings. Each image contains 1,006
recorded file entries. Fresh generation need not reproduce the image hash,
but must reproduce the asserted metadata and namespace behavior. The feature
masks are compat `0x24`, incompat `0x200c2`, and ro-compat `0x40b`; the default
hash version is 1 with the signed-hash superblock flag set.

The additional [filename behavior experiments](CASEFOLD-BEHAVIOR.md) cover
selected malformed byte sequences, post-version characters, normalization,
name lengths, rename, unlink and directory policy. Their exact Linux answers
are pinned separately from the driver's future support policy.

This matrix does not yet cover all namespace operations, the full malformed
input space, collision continuation, crashes, encrypted directories or other
feature combinations. Those require subsequent experiments and interoperability
stages. A clean fixture and a passing refusal test are not proof of casefold
read/write support.

The public format reference is the
[ext4 superblock specification](https://docs.kernel.org/filesystems/ext4/super.html).
No kernel or e2fsprogs implementation source is used.
