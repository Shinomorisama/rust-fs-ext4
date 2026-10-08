# Measured Linux casefold filename behavior

These experiments establish reference answers before implementing driver
semantics. They do not enable production writes. Only Linux modifies the
experiment images; the original Stage 0 fixtures remain intact.

## Evidence and method

The discovery source is `365cd9227a29b597bf7a4d6df22448ae07af7215` and
[run 37698167738](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/37698167738)
passed all four profiles. The guest identity remains the exact Linux 6.1 and
e2fsprogs 1.47 profile in `test-disks/casefold-oracle-profile.json`.

`tests/casefold_behavior_guest.py` uses explicit filename bytes and filesystem
calls. It never uses Python's Unicode normalization or casefold as an oracle.
The test records numeric errno values, enumerated name bytes, inode identity,
link counts and directory flags. The full inventory includes file hashes and
is checked again after unmount/remount.

Each profile runs 23 filename pairs in ordinary, small casefold and indexed
casefold directories, plus six isolated boundary pairs: 75 probes per image,
300 across the four profiles. Small comparison directories are explicitly
checked to remain non-indexed, including on 1 KiB images. Namespace operations
are additional observations within the same Rust integration test, not hundreds
of separate Rust test functions.

On the verification mount, every alternate spelling is queried before any
inventory scan or access to that probe's stored filename. Cold lookup results
are recorded separately from cached results. Only after these queries does the
test walk the image and verify the earlier cached-lookup observations.
Every image must still receive a clean, non-repairing e2fsck verdict.

`test-disks/casefold-behavior-reference.json` pins both observation sections.
The 1 KiB and 4 KiB results agree exactly within each encoding mode. Future
runs compare the complete sections, so dropping a probe or changing a result
fails. Reports, images and image hashes are retained as workflow artifacts.
Numeric errno values below describe this Linux guest, not a proposed C ABI.

## Ordinary, reproducible behavior

| Probe | Observed in casefold directories, both modes |
| --- | --- |
| ASCII case, sharp-s/SS, sigma forms, dotted-I/decomposed dotted-i | Same inode; exclusive alias creation returns EEXIST (17). |
| NFC/NFD accents, reordered combining marks, Hangul decomposition | Same inode; original stored bytes are preserved. |
| Deseret uppercase/lowercase and the fi ligature | Same inode for the tested pairs. |
| Dotless-i versus I; full-width A versus ASCII a | Distinct entries. Casefold is neither locale-sensitive casing nor general compatibility normalization. |
| Soft hyphen, joiner and variation selector inside a name | Tested aliases compare equal after the ignorable character is omitted. |
| U+1FAE0, U+0378, U+FDD0 and U+E000 with A/a | Accepted in both modes; the ASCII portion still folds. New/unassigned characters are not automatically invalid names. |
| Only soft hyphen versus only variation selector | One accepted entry; the alternate spelling resolves it even after remount. |
| 255-byte ASCII name | Accepted, with case-insensitive alias lookup. |
| 256-byte ASCII name | Create and lookup return ENAMETOOLONG (36). |

All 23 ordinary-directory control pairs are distinct, including on a strict
filesystem. The filesystem feature and encoding mode do not make every
directory case-insensitive.

Both case-only and canonically equivalent rename succeed but leave the
original name bytes in a casefold directory. In an ordinary directory those
renames change the bytes. Renaming one of two hard links onto the other leaves
both names and a link count of two. Unlink through an alias removes the stored
entry in a casefold directory and returns ENOENT (2) in the ordinary control.

New child directories inherit casefold; ordinary children do not acquire it.
Regular files do not acquire the flag. Moving an existing directory preserves
its own flag in both directions across ordinary/casefold parents.

## Exceptions that must not become shortcuts

Most tested malformed sequences (invalid leading byte, overlong encoding,
encoded surrogate, truncated sequence) are rejected with EINVAL (22) in
strict casefold directories. Non-strict mode accepts the tested names and
keeps their A/a spellings distinct for those sequences.

The isolated continuation byte is an exception: the tested names ending in
hex bytes `41 80` and `61 80` compare equal in non-strict mode, including cold lookup, and
exclusive creation of the alias returns EEXIST. Strict mode rejects creation
with EINVAL, while these lookups return ENOENT rather than EINVAL. This result
does **not** justify ASCII-lowercasing arbitrary malformed byte strings. More
malformed-name, cold-create and multiple-entry experiments are required before
claiming safe mutation support for those inputs.

A soft hyphen followed by a dot fails creation and lookup with EUCLEAN (117).
A soft hyphen followed by two dots fails with ELOOP (40). No such entries are
created and all images remain clean under e2fsck. These outcomes must not be
converted to "name absent, therefore safe to create" or folded into ordinary
path traversal by a new implementation.

Eighty repetitions of U+0390 form an accepted 160-byte name whose decomposed
comparison representation is 480 bytes. Its explicit 480-byte alias initially
returns ENAMETOOLONG on a fresh mount, but after the stored name is accessed,
the alias resolves it and exclusive alias creation returns EEXIST. The test
preserves both results instead of calling them a single equivalence rule.
The future driver must validate raw component lengths consistently and must
not truncate comparison keys to 255 bytes or depend on cached acceptance of
an overlong path component.

## Cold creation and competing entries

[Run 37733726483](https://github.com/Shinomorisama/rust-fs-ext4/actions/runs/37733726483),
at `230e81907e3079579ba23cfaea40133bc53545a9`, extends the investigation with
`tests/casefold_cold_create_guest.py`. Five pairs are tested in both creation
orders in ordinary, small casefold and indexed casefold directories: 30 probes
per profile, 120 in total. All four images passed e2fsck.

Each probe seeds one name, unmounts, then attempts exclusive creation of the
competing spelling without querying the stored name first. Two further mounts
read the first spelling first and the second spelling first, respectively.
Different payload markers distinguish the original file from a competing file.
Names, contents, exact inode identities and link counts agree across all three
readback phases in this matrix. No lookup-order ambiguity was observed.

The reference is `test-disks/casefold-cold-create-reference.json`. It omits
absolute inode allocation between independently generated images but retains
same-inode relationships. Within each image, exact inode identities must also
survive remounting and changed lookup order. Both block sizes give identical
recorded outcomes after this normalization.

Non-strict casefold treats the tested ASCII-case variants containing an isolated
`80` byte as the same name, even when that byte precedes the varying letter.
This is not simply removal of the byte: the tested `41 80` name and its `41`
counterpart coexist as distinct files. The `41 ff`/`61 ff` pair remains distinct
too. Strict casefold rejects malformed creates, while ordinary directories
accept both members of every control pair.

These results narrow uncertainty for these particular byte patterns; they do
not establish a general malformed-UTF-8 algorithm or qualify driver writes.
More byte patterns and actual destructive operations still need tests.

## Implementation consequences and remaining work

The next implementation stage can parse and validate the filesystem encoding
and per-directory policy while leaving the production write gate closed.
The Unicode engine must be versioned, preserve accepted name bytes, distinguish
raw length from comparison length, and avoid treating every unknown code point
as an invalid byte sequence. These observations do not justify using the host's
current Unicode tables or copying a kernel implementation.

The initial mutation policy for malformed names and normalized dot-like names
still needs explicit qualification or a tested conservative refusal. Cached
exceptions are evidence to investigate, not behavior to reproduce blindly.
Additional error paths, symlinks, no-replace rename, concurrent operations,
index collisions/growth and crash recovery remain separate stages. No result
here establishes casefold read/write readiness or compatibility with an
untested Linux release or device.
