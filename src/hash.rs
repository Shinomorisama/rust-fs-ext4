//! Directory-index (htree) name hashes.
//!
//! An indexed ext4 directory orders its entries by a 32-bit hash of each
//! name (kernel.org ext4 documentation, "Hash Tree Directories"). The
//! `dx_root` block records which of three algorithms the directory uses:
//!
//! * **legacy** — a small multiplicative/xor recurrence over the name's
//!   bytes, producing only a major hash;
//! * **half MD4** — the three rounds of MD4 (RFC 1320) applied to 32-byte
//!   blocks of the name, run as a chained compression function whose
//!   chaining value starts at the filesystem's hash seed;
//! * **TEA** — the Tiny Encryption Algorithm (Wheeler & Needham, FSE 1994)
//!   used the same way over 16-byte blocks, with the name block as the
//!   cipher key and the chaining value as the plaintext.
//!
//! Each algorithm comes in a "signed" and an "unsigned" flavour, which
//! differ only in whether a name byte >= 0x80 is widened to 32 bits with or
//! without sign extension. For pure-ASCII names the two agree.
//!
//! The byte-level format facts below (padding word, packing order, block
//! sizes, which chaining words become major/minor, the legacy constants)
//! were taken from the BSD-licensed FreeBSD and lwext4 descriptions of this
//! format and confirmed against `debugfs dx_hash` from e2fsprogs; see
//! `tests/htree_hash_vectors.rs` and `tests/htree_hash_differential.rs`.

/// Hash versions as recorded in `dx_root_info.hash_version` (kernel.org ext4
/// documentation, "Hash Tree Directories").
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashVersion {
    Legacy = 0,
    HalfMd4 = 1,
    Tea = 2,
    LegacyUnsigned = 3,
    HalfMd4Unsigned = 4,
    TeaUnsigned = 5,
}

impl HashVersion {
    /// Map an on-disk hash version byte to a supported version.
    pub fn from_u8(v: u8) -> Option<Self> {
        Some(match v {
            0 => Self::Legacy,
            1 => Self::HalfMd4,
            2 => Self::Tea,
            3 => Self::LegacyUnsigned,
            4 => Self::HalfMd4Unsigned,
            5 => Self::TeaUnsigned,
            _ => return None,
        })
    }

    /// True for the three "unsigned" variants (codes 3, 4, 5).
    pub fn is_unsigned(self) -> bool {
        matches!(
            self,
            Self::LegacyUnsigned | Self::HalfMd4Unsigned | Self::TeaUnsigned
        )
    }

    /// How this version widens a name byte into a 32-bit quantity.
    fn byte_widening(self) -> ByteWidening {
        if self.is_unsigned() {
            ByteWidening::ZeroExtend
        } else {
            ByteWidening::SignExtend
        }
    }
}

/// Result of [`name_hash`]: `major` orders entries in the index, `minor`
/// breaks ties between names whose major hashes collide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NameHash {
    pub major: u32,
    pub minor: u32,
}

/// Hash `name` for a directory index using `version` and the superblock's
/// `s_hash_seed` (as four little-endian 32-bit words).
pub fn name_hash(name: &[u8], version: HashVersion, seed: &[u32; 4]) -> NameHash {
    let widening = version.byte_widening();
    let (major, minor) = match version {
        HashVersion::Legacy | HashVersion::LegacyUnsigned => {
            (LegacyRecurrence::digest(name, widening), 0)
        }
        HashVersion::HalfMd4 | HashVersion::HalfMd4Unsigned => {
            chain_blocks::<8, Md4HalfRounds>(name, widening, seed)
        }
        HashVersion::Tea | HashVersion::TeaUnsigned => {
            chain_blocks::<4, TeaCipher>(name, widening, seed)
        }
    };
    // The lowest bit of a major hash is never part of the value (every
    // `debugfs dx_hash` major is even, for every version): index entries
    // keep that bit free for their own bookkeeping.
    //
    // Note: a major that comes out as 0xFFFFFFFE is reported unchanged by
    // `debugfs dx_hash` (e2fsprogs 1.47.0 and 1.47.2), and this function
    // agrees with it. The BSD references additionally step that one value
    // down to 0xFFFFFFFC; `tests/htree_hash_differential.rs` pins the
    // debugfs behaviour.
    NameHash {
        major: major & !1,
        minor,
    }
}

/// The version to use for a directory whose `dx_root` records
/// `root_version`, given the superblock's `EXT2_FLAGS_UNSIGNED_HASH` flag.
/// `dx_root` only ever records the signed codes (0, 1, 2); when the flag is
/// set, the matching unsigned variant (3, 4, 5) applies instead.
pub fn effective_version(root_version: HashVersion, unsigned_hash: bool) -> HashVersion {
    if !unsigned_hash {
        return root_version;
    }
    match root_version {
        HashVersion::Legacy => HashVersion::LegacyUnsigned,
        HashVersion::HalfMd4 => HashVersion::HalfMd4Unsigned,
        HashVersion::Tea => HashVersion::TeaUnsigned,
        already_unsigned => already_unsigned,
    }
}

// ---------------------------------------------------------------------------
// Turning name bytes into 32-bit words
// ---------------------------------------------------------------------------

/// Whether a name byte is treated as a signed or an unsigned `char` when it
/// is widened to 32 bits. This is the only difference between the signed
/// and unsigned hash versions.
#[derive(Debug, Clone, Copy)]
enum ByteWidening {
    SignExtend,
    ZeroExtend,
}

impl ByteWidening {
    fn widen(self, byte: u8) -> u32 {
        match self {
            // 0x80..=0xFF become 0xFFFFFF80..=0xFFFFFFFF.
            ByteWidening::SignExtend => byte as i8 as i32 as u32,
            ByteWidening::ZeroExtend => u32::from(byte),
        }
    }
}

/// Splits a name into fixed-size blocks of `N` words for the block-based
/// algorithms (half MD4: 8 words, TEA: 4 words).
///
/// Each block is built from the bytes still unconsumed at that point:
///
/// * A "fill word" repeats the low byte of the number of bytes still
///   unconsumed (the whole remaining tail, not just this block's share)
///   in all four byte lanes.
/// * Each word takes up to four consecutive name bytes. It starts as the
///   fill word, and each byte in turn is appended at the bottom: the word
///   shifts left by 8 and the widened byte is *added* (with wrapping). With
///   four bytes the fill word is shifted out completely and the first byte
///   ends up most significant; with fewer, the top lanes keep fill bytes.
///   Under sign extension, adding a byte >= 0x80 borrows from the lanes
///   above it, which is what makes the signed variants differ.
/// * Words past the end of the name are the plain fill word.
///
/// A block consumes up to `4 * N` bytes. An empty name yields no blocks
/// at all, so the hash is the seed state untouched (confirmed with
/// `debugfs dx_hash` on the empty name).
struct WordFeeder<'a, const N: usize> {
    rest: &'a [u8],
    widening: ByteWidening,
}

impl<'a, const N: usize> WordFeeder<'a, N> {
    const BLOCK_BYTES: usize = 4 * N;

    fn new(name: &'a [u8], widening: ByteWidening) -> Self {
        Self {
            rest: name,
            widening,
        }
    }

    fn fill_word(remaining: usize) -> u32 {
        let r = remaining as u32;
        r | (r << 8) | (r << 16) | (r << 24)
    }

    fn pack(&self, bytes: &[u8], fill: u32) -> u32 {
        bytes.iter().fold(fill, |word, &b| {
            (word << 8).wrapping_add(self.widening.widen(b))
        })
    }
}

impl<const N: usize> Iterator for WordFeeder<'_, N> {
    type Item = [u32; N];

    fn next(&mut self) -> Option<[u32; N]> {
        if self.rest.is_empty() {
            return None;
        }
        let fill = Self::fill_word(self.rest.len());
        let take = self.rest.len().min(Self::BLOCK_BYTES);
        let (this_block, later) = self.rest.split_at(take);
        let mut block = [fill; N];
        for (word, bytes) in block.iter_mut().zip(this_block.chunks(4)) {
            *word = self.pack(bytes, fill);
        }
        self.rest = later;
        Some(block)
    }
}

// ---------------------------------------------------------------------------
// Block mixers and the loop that drives them
// ---------------------------------------------------------------------------

/// A compression step that folds one block of name words into a 128-bit
/// chaining state.
trait BlockMixer<const N: usize> {
    /// Fold `block` into `state`.
    fn absorb(state: &mut [u32; 4], block: &[u32; N]);
    /// Which chaining words are reported as (major, minor).
    fn report(state: &[u32; 4]) -> (u32, u32);
}

/// The initial chaining values from RFC 1320 section 3.3 (words A, B, C, D).
/// They are used whenever the superblock's hash seed is entirely zero —
/// an unset seed — which `debugfs dx_hash -s 00000000-...` confirms.
/// A seed with any non-zero word is used exactly as given.
const RFC1320_INITIAL_STATE: [u32; 4] = [0x6745_2301, 0xEFCD_AB89, 0x98BA_DCFE, 0x1032_5476];

fn starting_state(seed: &[u32; 4]) -> [u32; 4] {
    if seed.iter().all(|&w| w == 0) {
        RFC1320_INITIAL_STATE
    } else {
        *seed
    }
}

/// Feed every `N`-word block of `name` through mixer `M`, starting from
/// the seed, and report the mixer's (major, minor) words.
fn chain_blocks<const N: usize, M: BlockMixer<N>>(
    name: &[u8],
    widening: ByteWidening,
    seed: &[u32; 4],
) -> (u32, u32) {
    let mut state = starting_state(seed);
    for block in WordFeeder::<N>::new(name, widening) {
        M::absorb(&mut state, &block);
    }
    M::report(&state)
}

// --- Half MD4 --------------------------------------------------------------

/// MD4's three rounds (RFC 1320 section 3.4) cut down to an 8-word block:
/// each round makes two passes of four steps instead of four passes, and
/// the result is added back into the chaining state (feed-forward), as in
/// MD4. Major is chaining word B, minor is word C.
struct Md4HalfRounds;

/// RFC 1320 round 1 auxiliary function: bitwise "if X then Y else Z".
fn rfc1320_f(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | (!x & z)
}

/// RFC 1320 round 2 auxiliary function: bitwise majority of X, Y, Z.
fn rfc1320_g(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | (x & z) | (y & z)
}

/// RFC 1320 round 3 auxiliary function: bitwise parity.
fn rfc1320_h(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}

/// RFC 1320 additive constants for rounds 2 and 3 (round 1 adds none).
const RFC1320_ROUND2_CONSTANT: u32 = 0x5A82_7999;
const RFC1320_ROUND3_CONSTANT: u32 = 0x6ED9_EBA1;

/// One MD4 round over an 8-word block: which word each of the 8 steps
/// reads and how far it rotates.
struct Md4Round {
    mix: fn(u32, u32, u32) -> u32,
    constant: u32,
    words: [usize; 8],
    rotations: [u32; 4],
}

const HALF_MD4_ROUNDS: [Md4Round; 3] = [
    Md4Round {
        mix: rfc1320_f,
        constant: 0,
        words: [0, 1, 2, 3, 4, 5, 6, 7],
        rotations: [3, 7, 11, 19],
    },
    Md4Round {
        mix: rfc1320_g,
        constant: RFC1320_ROUND2_CONSTANT,
        words: [1, 3, 5, 7, 0, 2, 4, 6],
        rotations: [3, 5, 9, 13],
    },
    Md4Round {
        mix: rfc1320_h,
        constant: RFC1320_ROUND3_CONSTANT,
        words: [3, 7, 2, 6, 1, 5, 0, 4],
        rotations: [3, 9, 11, 15],
    },
];

impl BlockMixer<8> for Md4HalfRounds {
    fn absorb(state: &mut [u32; 4], block: &[u32; 8]) {
        // Registers A, B, C, D live at indices 0..4. As in RFC 1320, the
        // step targets cycle A, D, C, B, and each step's three inputs are
        // the registers that follow its target in the order A B C D.
        let mut reg = *state;
        for round in &HALF_MD4_ROUNDS {
            for (step, &word) in round.words.iter().enumerate() {
                let target = (4 - step % 4) % 4;
                let x = reg[(target + 1) % 4];
                let y = reg[(target + 2) % 4];
                let z = reg[(target + 3) % 4];
                reg[target] = reg[target]
                    .wrapping_add((round.mix)(x, y, z))
                    .wrapping_add(block[word])
                    .wrapping_add(round.constant)
                    .rotate_left(round.rotations[step % 4]);
            }
        }
        for (s, r) in state.iter_mut().zip(reg) {
            *s = s.wrapping_add(r);
        }
    }

    fn report(state: &[u32; 4]) -> (u32, u32) {
        (state[1], state[2])
    }
}

// --- TEA -------------------------------------------------------------------

/// TEA (Wheeler & Needham) with the 4-word name block as the 128-bit key
/// and the first two chaining words as the 64-bit plaintext. Only 16
/// cycles are run rather than the paper's recommended 32. The ciphertext
/// is added back into those two chaining words; the other two are never
/// touched. Major is chaining word 0, minor is word 1.
struct TeaCipher;

/// The TEA key schedule constant, 2^32 / golden ratio (TEA paper).
const TEA_DELTA: u32 = 0x9E37_79B9;
const TEA_CYCLES: u32 = 16;

/// The TEA encipher routine from the paper, for `cycles` cycles.
fn tea_encipher(plain: [u32; 2], key: &[u32; 4], cycles: u32) -> [u32; 2] {
    let [mut y, mut z] = plain;
    let mut sum: u32 = 0;
    for _ in 0..cycles {
        sum = sum.wrapping_add(TEA_DELTA);
        y = y.wrapping_add(
            (z << 4).wrapping_add(key[0]) ^ z.wrapping_add(sum) ^ (z >> 5).wrapping_add(key[1]),
        );
        z = z.wrapping_add(
            (y << 4).wrapping_add(key[2]) ^ y.wrapping_add(sum) ^ (y >> 5).wrapping_add(key[3]),
        );
    }
    [y, z]
}

impl BlockMixer<4> for TeaCipher {
    fn absorb(state: &mut [u32; 4], block: &[u32; 4]) {
        let [y, z] = tea_encipher([state[0], state[1]], block, TEA_CYCLES);
        state[0] = state[0].wrapping_add(y);
        state[1] = state[1].wrapping_add(z);
    }

    fn report(state: &[u32; 4]) -> (u32, u32) {
        (state[0], state[1])
    }
}

// --- Legacy ----------------------------------------------------------------

/// The original ext3 directory hash: a three-term recurrence. Each byte,
/// widened and multiplied by a fixed odd constant, is xored into the most
/// recent term and added to the term before it; a sum that lands in the
/// upper half of the 32-bit range is pulled back down by 2^31 - 1. The
/// result is the last term doubled. It ignores the hash seed and has no
/// minor hash.
struct LegacyRecurrence;

impl LegacyRecurrence {
    const MULTIPLIER: u32 = 0x006D_22F5;
    const FIRST_TERMS: (u32, u32) = (0x37AB_E8F9, 0x12A3_FE2D);
    const REDUCTION: u32 = 0x7FFF_FFFF;

    fn digest(name: &[u8], widening: ByteWidening) -> u32 {
        let (_older, newest) = name
            .iter()
            .fold(Self::FIRST_TERMS, |(older, newer), &byte| {
                let scaled = widening.widen(byte).wrapping_mul(Self::MULTIPLIER);
                let mut next = older.wrapping_add(newer ^ scaled);
                if next & 0x8000_0000 != 0 {
                    next = next.wrapping_sub(Self::REDUCTION);
                }
                (newer, next)
            });
        newest << 1
    }
}
