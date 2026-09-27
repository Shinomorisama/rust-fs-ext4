#!/usr/bin/env bash
# Rebuild fuzz/corpus from filesystems mke2fs wrote.
#
# ext4 is the widest parser surface in the family: a superblock, group
# descriptors, inodes, directory blocks, extent trees, htree indexes and
# a jbd2 journal, each read from an offset the one before it supplied.
#
# The seeds are real filesystems and real structures cut out of them. A
# random byte string is refused by the 0xEF53 magic on the first line
# and never reaches the arithmetic underneath.
#
# Usage: scripts/make-fuzz-corpus.sh
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
work="$(mktemp -d "${TMPDIR:-/tmp}/ext4-fuzz-corpus.XXXXXX")"
trap 'rm -rf "$work"' EXIT

for tool in mke2fs e2fsck; do
    command -v "$tool" >/dev/null || {
        echo "$tool not found; install e2fsprogs" >&2
        exit 1
    }
done

# Populated through mke2fs's -d, not by mounting: mounting needs root
# and -d does not, and it still produces trees a real e2fsprogs wrote.
tree="$work/tree"
mkdir -p "$tree/sub"
head -c 120000 /dev/urandom > "$tree/random.bin"
python3 -c "import sys; open(sys.argv[1],'w').write('the quick brown fox. ' * 4000)" "$tree/text.txt"
echo "deep" > "$tree/sub/deep.txt"
ln -sf sub/deep.txt "$tree/link"
# Enough entries that the root directory spans several blocks, which is
# what lets e2fsck -D index it with an htree below. mke2fs -d alone
# never does: it writes every directory as a linear list.
for i in $(seq 1 600); do
    : > "$tree/entry-$(printf '%04d' "$i")"
done
command -v setfattr >/dev/null && {
    setfattr -n user.colour -v blue "$tree/text.txt"
    setfattr -n user.long -v "$(printf 'v%.0s' $(seq 1 100))" "$tree/text.txt"
} || true

# Only what this script wrote last time is replaced. The rest of
# fuzz/corpus is reproducers the fuzzer found, committed so the gate
# replays them, and a rebuild must never throw one away.
corpus="$here/fuzz/corpus"
mkdir -p "$corpus"/{image,superblock,inode,dir_block,journal}
for stem in ext2 ext4 ext4-4k ext4-64bit; do
    rm -f "$corpus/image/$stem.img" "$corpus/superblock/$stem.bin" "$corpus/inode/$stem-root.bin"
    find "$corpus/dir_block" "$corpus/journal" -maxdepth 1 -type f \
        -name "$stem-at[0-9]*.bin" -delete
done

build() {
    local name="$1" size="$2"; shift 2
    local img="$here/fuzz/corpus/image/$name.img"
    truncate -s "$size" "$img"
    mke2fs -q -F -d "$tree" "$@" "$img" 2>/dev/null || {
        echo "mke2fs could not build the '$name' filesystem" >&2
        exit 1
    }
}

# Rebuild every directory as e2fsck would, which on a filesystem with
# dir_index turns each directory of more than one block into an htree.
# Exit 1 is "the filesystem was changed", which is the point; anything
# above it is a failure. A second, read-only pass then has to find the
# result clean, so the seed is an index e2fsck itself accepts.
index() {
    local img="$corpus/image/$1.img" rc=0
    e2fsck -fyD "$img" >/dev/null 2>&1 || rc=$?
    [ "$rc" -le 1 ] || { echo "e2fsck -D failed on '$1' (exit $rc)" >&2; exit 1; }
    e2fsck -fn "$img" >/dev/null 2>&1 || {
        echo "'$1' is not clean after e2fsck -D" >&2
        exit 1
    }
}

# One per shape that changes a decoder rather than the layout: ext2 has
# no extents and no journal, ext4 has both, 1 KiB blocks move every
# offset, and 64bit widens the group descriptors.
build ext2      4M  -t ext2 -b 1024
build ext4      8M  -t ext4 -b 1024 -O ^64bit
build ext4-4k   8M  -t ext4 -b 4096 -O ^64bit
build ext4-64bit 8M  -t ext4 -b 4096 -O 64bit,metadata_csum

# The ext4 filesystems carry the htree; ext2 stays a linear directory on
# purpose, because that is the other directory decoder.
for stem in ext4 ext4-4k ext4-64bit; do
    index "$stem"
done

python3 - "$here/fuzz/corpus" <<'PY'
import os, struct, sys

root = sys.argv[1]
SB_AT = 1024
SB_LEN = 1024
EXT_MAGIC = 0xEF53
JBD2_MAGIC = b'\xc0\x3b\x39\x98'
EXTENTS_FL = 0x80000
INDEX_FL = 0x1000

def write(kind, name, data):
    with open(os.path.join(root, kind, name), 'wb') as f:
        f.write(data)

inodes = dirs = journals = 0
for img_name in sorted(os.listdir(os.path.join(root, 'image'))):
    stem = img_name[:-len('.img')]
    img = open(os.path.join(root, 'image', img_name), 'rb').read()

    sb = img[SB_AT:SB_AT + SB_LEN]
    magic, = struct.unpack_from('<H', sb, 56)
    assert magic == EXT_MAGIC, f"{img_name}: superblock magic is {magic:#x}"
    write('superblock', f'{stem}.bin', sb)

    log_block_size, = struct.unpack_from('<I', sb, 24)
    blocksize = 1024 << log_block_size
    first_data_block, = struct.unpack_from('<I', sb, 20)
    inode_size, = struct.unpack_from('<H', sb, 88)
    feature_incompat, = struct.unpack_from('<I', sb, 96)
    sixty_four = bool(feature_incompat & 0x80)
    desc_size, = struct.unpack_from('<H', sb, 254)
    if not sixty_four or desc_size == 0:
        desc_size = 32

    # The group descriptor table begins in the block after the one
    # holding the superblock.
    gdt_at = (first_data_block + 1) * blocksize
    inode_table_lo, = struct.unpack_from('<I', img, gdt_at + 8)
    inode_table = inode_table_lo * blocksize

    # Inode 2 is the root; inodes are one-based, so it is the second
    # slot in the table. 256 bytes rather than `inode_size` so the
    # inline xattr area behind a large inode comes with it.
    root_inode_at = inode_table + inode_size
    write('inode', f'{stem}-root.bin', img[root_inode_at:root_inode_at + max(inode_size, 256)])
    inodes += 1

    # The root directory's first block, found through the root inode's
    # own block map rather than by scanning for something that looks
    # like a directory: that is what makes it the root's, and on an
    # indexed root it is the dx_root.
    root_inode = img[root_inode_at:root_inode_at + inode_size]
    flags, = struct.unpack_from('<I', root_inode, 32)
    i_block = root_inode[40:100]
    if flags & EXTENTS_FL:
        eh_magic, eh_entries, _, eh_depth = struct.unpack_from('<HHHH', i_block, 0)
        assert eh_magic == 0xF30A and eh_entries >= 1, f"{stem}: root has no extent header"
        assert eh_depth == 0, f"{stem}: root extent tree is {eh_depth} deep; walk it"
        _, _, start_hi, start_lo = struct.unpack_from('<IHHI', i_block, 12)
        first = (start_hi << 32) | start_lo
    else:
        first, = struct.unpack_from('<I', i_block, 0)
    block = img[first * blocksize:(first + 1) * blocksize]
    assert block[8:9] == b'.', f"{stem}: root block {first} does not start with '.'"

    if stem.startswith('ext4'):
        # Indexed, and the block parses as a dx_root: "." and ".." take
        # 24 bytes, then the 8-byte dx_root_info, then limit and count.
        assert flags & INDEX_FL, f"{stem}: root i_flags {flags:#x} lacks INDEX_FL"
        reserved, hash_version, info_length, levels = struct.unpack_from('<IBBB', block, 24)
        limit, count = struct.unpack_from('<HH', block, 32)
        most = (blocksize - 32) // 8
        assert reserved == 0 and info_length == 8 and levels <= 2, \
            f"{stem}: root block is not a dx_root (info_length {info_length})"
        assert most - 1 <= limit <= most and 1 <= count <= limit, \
            f"{stem}: dx_root limit {limit} count {count} for a {blocksize}-byte block"
    else:
        assert not flags & INDEX_FL, f"{stem}: the linear seed came out indexed"
    write('dir_block', f'{stem}-at{first}.bin', block)
    dirs += 1

    # The journal superblock, found by the jbd2 magic. ext2 has no
    # journal, which is part of why it is in the corpus.
    for at in range(0, len(img) - blocksize + 1, blocksize):
        if img[at:at + 4] == JBD2_MAGIC:
            write('journal', f'{stem}-at{at // blocksize}.bin', img[at:at + blocksize])
            journals += 1
            break

assert inodes and dirs, "no inode or directory block found -- the layout walk needs revisiting"
assert journals, "no journal superblock found -- did every filesystem come out without one?"
print(f"{inodes} inodes, {dirs} directory blocks, {journals} journals")
PY

echo "corpus rebuilt under fuzz/corpus:"
find "$here/fuzz/corpus" -type f | sort | sed "s#$here/##"
echo "total: $(find "$here/fuzz/corpus" -type f | wc -l) seeds, $(du -sh "$here/fuzz/corpus" | cut -f1)"
