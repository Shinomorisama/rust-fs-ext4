//! ext4 inode parsing.
//!
//! Spec: docs/ext4-spec/inodes-extents.md
//!
//! Base inode is 128 bytes; modern ext4 with EXT4_FEATURE_RO_COMPAT_EXTRA_ISIZE
//! adds another 32 bytes (i_extra_isize) for a total of 160 bytes. All fields
//! little-endian. The high halves of uid/gid/size/file_acl/blocks/checksum live
//! at the end of the base 128 bytes; nanosecond timestamps + crtime live in the
//! extra section.

use crate::error::{Error, Result};

/// Minimum on-disk inode size (rev 0).
pub const INODE_BASE_SIZE: usize = 128;
/// The size of the original ext2 inode — `EXT2_GOOD_OLD_INODE_SIZE`.
///
/// Everything up to here is the fixed part every ext2/3/4 inode has;
/// anything past it is the `i_extra_isize` region, which only larger
/// inodes carry. The inode checksum covers every byte of it, so it is the
/// length below which `Checksummer::verify_inode` refuses. Not because a
/// shorter buffer cannot hold `i_checksum_lo` — that field ends at 0x7E,
/// so 126 bytes would — but because it is not the whole inode the
/// checksum is defined over.
///
/// It was declared in `mkfs.rs`, unused, while `checksum.rs` wrote the
/// bare `128` twice.
pub const GOOD_OLD_INODE_SIZE: usize = 128;

/// Offset where the i_extra_isize field begins (start of extra section).
pub const INODE_EXTRA_OFFSET: usize = 128;

// Raw inode field byte offsets (from the start of the on-disk inode, little-endian).
// Named so build_*_inode helpers can write fields without requiring readers to
// memorise the ext4 spec layout. Source: docs/ext4-spec/inodes-extents.md.
pub(crate) const OFF_MODE: usize = 0x00;
pub(crate) const OFF_SIZE_LO: usize = 0x04;
pub(crate) const OFF_ATIME: usize = 0x08;
pub(crate) const OFF_CTIME: usize = 0x0C;
pub(crate) const OFF_MTIME: usize = 0x10;
pub(crate) const OFF_LINKS_COUNT: usize = 0x1A;
pub(crate) const OFF_BLOCKS_LO: usize = 0x1C;
pub(crate) const OFF_FLAGS: usize = 0x20;
pub(crate) const OFF_BLOCK: usize = 0x28; // i_block area start (60 bytes, 0x28..0x64)
pub(crate) const OFF_GENERATION: usize = 0x64;
pub(crate) const OFF_SIZE_HI: usize = 0x6C;
pub(crate) const OFF_BLOCKS_HI: usize = 0x74;
pub(crate) const OFF_CHECKSUM_LO: usize = 0x7C;
pub(crate) const OFF_EXTRA_ISIZE: usize = 0x80;
pub(crate) const OFF_CHECKSUM_HI: usize = 0x82;
pub(crate) const OFF_CRTIME: usize = 0x90;

/// Default i_extra_isize value written into new inodes: covers checksum_hi,
/// nsec timestamps, and i_crtime (32 bytes beyond the 128-byte base).
pub(crate) const EXTRA_ISIZE_DEFAULT: u16 = 32;
/// Minimum inode buffer length for i_crtime (offset 0x90) to be present.
#[cfg(test)]
pub(crate) const INODE_SIZE_WITH_CRTIME: usize = 0x94;
/// Minimum inode buffer length for i_extra_isize + i_checksum_hi.
pub(crate) const INODE_SIZE_WITH_EXTRA: usize = 0x84;

// POSIX file-type bits (high nibble of i_mode).
pub const S_IFMT: u16 = 0xF000;
pub const S_IFREG: u16 = 0x8000;
pub const S_IFDIR: u16 = 0x4000;
pub const S_IFLNK: u16 = 0xA000;
pub const S_IFBLK: u16 = 0x6000;
pub const S_IFCHR: u16 = 0x2000;
pub const S_IFIFO: u16 = 0x1000;
pub const S_IFSOCK: u16 = 0xC000;

bitflags::bitflags! {
    /// `i_flags` — per-inode behaviour flags.
    /// Spec: kernel.org/doc/html/latest/filesystems/ext4/inodes.html
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct InodeFlags: u32 {
        /// Secure deletion (unused).
        const SECRM        = 0x0000_0001;
        /// Undelete (unused).
        const UNRM         = 0x0000_0002;
        /// Compressed file.
        const COMPR        = 0x0000_0004;
        /// Synchronous writes.
        const SYNC         = 0x0000_0008;
        /// Immutable.
        const IMMUTABLE    = 0x0000_0010;
        /// Append-only.
        const APPEND       = 0x0000_0020;
        /// Do not dump.
        const NODUMP       = 0x0000_0040;
        /// Do not update access time.
        const NOATIME      = 0x0000_0080;
        /// fscrypt-encrypted (`EXT4_ENCRYPT_FL`): the contents, or a
        /// directory's entry names, are ciphertext.
        const ENCRYPT      = 0x0000_0800;
        /// Hash-tree-indexed directory.
        const INDEX        = 0x0000_1000;
        /// File data stored in extended attributes.
        const EA_INODE     = 0x0020_0000;
        /// Inode uses extents (EXT4_EXTENTS_FL).
        const EXTENTS      = 0x0008_0000;
        /// Inode stores a huge file (i_blocks counted in fs blocks not 512B sectors).
        const HUGE_FILE    = 0x0004_0000;
        /// Inline data — file contents live inside i_block + xattrs.
        const INLINE_DATA  = 0x1000_0000;
        /// Directory names use the volume's casefold encoding.
        const CASEFOLD     = 0x4000_0000;
        /// Alias for EXTENTS (matches kernel naming `EXT4_EXTENTS_FL`).
        const EXTENT       = 0x0008_0000;
        /// Inode has extra (nanosecond) timestamp fields.
        const EXTRA_ATIME  = 0x0000_0100;
    }
}

/// The `i_flags` bits a caller may change through `set_flags`.
///
/// The kernel's `EXT4_FL_USER_MODIFIABLE`, less the bits it honours only
/// under conditions this driver does not implement: `EXTENTS` (a
/// migration that rewrites the block map), `DAX` (a mount option and a
/// device that supports it) and `CASEFOLD` (an empty directory on a
/// volume with the casefold feature), and the obsolete `EOFBLOCKS`.
/// Every other bit — `INDEX`, `HUGE_FILE`, `ENCRYPT`, `VERITY`,
/// `INLINE_DATA`, `EA_INODE` among them — describes how the bytes the
/// inode already holds are read, so flipping it alone makes them read
/// wrong.
pub const USER_MODIFIABLE_FLAGS: u32 = InodeFlags::SECRM.bits()
    | InodeFlags::UNRM.bits()
    | InodeFlags::COMPR.bits()
    | InodeFlags::SYNC.bits()
    | InodeFlags::IMMUTABLE.bits()
    | InodeFlags::APPEND.bits()
    | InodeFlags::NODUMP.bits()
    | InodeFlags::NOATIME.bits()
    | 0x0000_4000 // JOURNAL_DATA
    | 0x0000_8000 // NOTAIL
    | 0x0001_0000 // DIRSYNC
    | 0x0002_0000 // TOPDIR
    | 0x2000_0000; // PROJINHERIT

/// Parsed ext4 inode.
///
/// Combines hi+lo halves for uid, gid, size, file_acl, blocks, and checksum so
/// callers don't have to reassemble them. Nanosecond timestamps come from the
/// `*_extra` fields when present (top 30 bits = nsec, low 2 bits = epoch).
#[derive(Debug, Clone)]
pub struct Inode {
    pub mode: u16,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,
    /// Seconds since the Unix epoch, **signed and 64-bit**.
    ///
    /// The on-disk base field is a signed 32-bit value, so dates before
    /// 1970 are representable and must not be read as far-future ones.
    /// When `i_extra_isize` is large enough, the low two bits of the
    /// matching `*_extra` field add whole multiples of 2^32 seconds,
    /// widening the range from 1901..2038 to 1901..2446. Both are
    /// applied here; see `unpack_seconds`.
    pub atime: i64,
    pub mtime: i64,
    pub ctime: i64,
    pub dtime: i64,
    pub crtime: i64,
    pub atime_nsec: u32,
    pub mtime_nsec: u32,
    pub ctime_nsec: u32,
    pub crtime_nsec: u32,
    pub links_count: u16,
    pub blocks: u64, // 512-byte sectors (per spec; HUGE_FILE flag changes meaning)
    pub flags: u32,
    /// Raw 60-byte i_block area — extent header / direct pointers / inline data.
    /// Parsed by the extent module.
    pub block: [u8; 60],
    pub generation: u32,
    pub file_acl: u64,
    pub checksum: u32,
}

// ---------------------------------------------------------------------------
// Timestamps
// ---------------------------------------------------------------------------
//
// The format (kernel.org ext4 documentation, inodes.html, "Inode
// Timestamps"): each of atime, ctime, mtime and crtime has a 32-bit base
// word holding a *signed* count of seconds from 1970-01-01 UTC, and --
// where `i_extra_isize` reaches it -- a 32-bit companion word. The
// companion's two low bits count whole 2^32-second "eras" to add to the
// signed base, and its upper thirty bits are nanoseconds. dtime has no
// companion.
//
// The documentation tabulates the result: era 0 covers 1901-12-13 ..
// 2038-01-19 (the signed base on its own), and each further era slides
// that window 2^32 seconds later, so era 3 ends on 2446-05-10. The
// windows abut, so every second in 1901..2446 has exactly one encoding.
// A kernel in the harness VM, asked to `touch -d @N` files on a 256-byte
// inode volume, wrote exactly those words for every boundary tested
// (`needs_host::the_kernel_writes_and_reads_the_documented_words`), and
// clamped anything later than 2446 to the last second of era 3.

/// Seconds one era step adds: 2^32.
const ERA_SECONDS: i64 = 1 << 32;

/// The two low bits of a companion word: the era count.
const ERA_FIELD: u32 = 0b11;

/// Read a timestamp from its base word and its companion word (pass 0
/// for the companion when the inode has none).
///
/// The base is signed: a base with its top bit set is a date before
/// 1970 in era 0, and a date 2^32 seconds later than that in each
/// further era. The nanosecond bits of `extra` are ignored here.
fn unpack_seconds(base: u32, extra: u32) -> i64 {
    let signed_base = i64::from(base as i32);
    let era = i64::from(extra & ERA_FIELD);
    signed_base + era * ERA_SECONDS
}

/// Split a count of seconds into the base word and the era bits for
/// the low end of the companion word -- the inverse of
/// [`unpack_seconds`] for every value in
/// [`MIN_ENCODABLE_TIME`]`..=`[`MAX_ENCODABLE_TIME`].
///
/// The era is the number of 2^32-second windows the value lies past the
/// window centred on the epoch, `[-2^31, 2^31)`. Counted in half-windows
/// of 2^31 (an arithmetic shift, so it floors for negative values too),
/// that is the half-window index plus one, halved. The base word is
/// then simply the low 32 bits: the signed reading of those bits plus
/// the era's 2^32-multiples gives the value back.
///
/// 2100-01-01 (4 102 444 800) shows why the era is not just "the bits
/// above bit 31": it lies in the second window, so it is stored with
/// era 1 and a base whose signed reading is negative (-192 522 496).
///
/// Values outside the encodable range have their era taken modulo four;
/// callers clamp or refuse first.
pub(crate) fn pack_seconds(secs: i64) -> (u32, u32) {
    let half_windows = secs >> 31;
    let era = (((half_windows + 1) >> 1) as u32) & ERA_FIELD;
    (secs as u32, era)
}

/// The earliest second the base-plus-era encoding holds: era 0 with
/// the most negative base, 1901-12-13 20:45:52 UTC.
pub(crate) const MIN_ENCODABLE_TIME: i64 = i32::MIN as i64;
/// The latest: era 3 with the most positive base, 2446-05-10 22:38:55
/// UTC. The kernel clamps later times to exactly this value.
pub(crate) const MAX_ENCODABLE_TIME: i64 = i32::MAX as i64 + 3 * ERA_SECONDS;

/// One of an inode's four timestamps, by where its base and its
/// companion word live.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InodeTime {
    Atime,
    Ctime,
    Mtime,
    Crtime,
}

impl InodeTime {
    /// Offset of the 32-bit base.
    fn base_offset(self) -> usize {
        match self {
            InodeTime::Atime => OFF_ATIME,
            InodeTime::Ctime => OFF_CTIME,
            InodeTime::Mtime => OFF_MTIME,
            InodeTime::Crtime => OFF_CRTIME,
        }
    }

    /// Offset of the companion word (nanoseconds above, era below).
    fn extra_offset(self) -> usize {
        match self {
            InodeTime::Ctime => 0x84,
            InodeTime::Mtime => 0x88,
            InodeTime::Atime => 0x8C,
            InodeTime::Crtime => 0x94,
        }
    }
}

/// Whether the four bytes at `offset` lie inside the part of the inode
/// that `i_extra_isize` declares in use (and inside `raw`). A field
/// past that point is not part of this inode, whatever the inode size.
fn extra_field_fits(raw: &[u8], offset: usize) -> bool {
    let Some(size_bytes) = raw.get(OFF_EXTRA_ISIZE..OFF_EXTRA_ISIZE + 2) else {
        return false;
    };
    let declared_end =
        INODE_EXTRA_OFFSET + usize::from(u16::from_le_bytes([size_bytes[0], size_bytes[1]]));
    let field_end = offset + 4;
    field_end <= raw.len() && field_end <= declared_end
}

/// Store `secs` (whole seconds; nanoseconds zero) as `field`.
///
/// - Where the field's companion word is present: the base and the era
///   bits from [`pack_seconds`], after clamping to the encodable range.
///   The companion's nanosecond bits are cleared.
/// - Where it is not (a 128-byte inode, or an `i_extra_isize` too small
///   to reach it): the base alone, clamped to the signed 32-bit range,
///   so a time after 2038 is stored as 2038-01-19 03:14:07 rather than
///   wrapping round to 1901.
/// - crtime's base word is itself in the extended area; if even that is
///   out of reach, nothing is written.
pub(crate) fn set_inode_time(raw: &mut [u8], field: InodeTime, secs: i64) {
    let base_at = field.base_offset();
    let extra_at = field.extra_offset();
    let (base, extra) = if extra_field_fits(raw, extra_at) {
        let (base, era) = pack_seconds(secs.clamp(MIN_ENCODABLE_TIME, MAX_ENCODABLE_TIME));
        (base, Some(era))
    } else if field != InodeTime::Crtime || extra_field_fits(raw, base_at) {
        let clamped = secs.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
        (clamped as u32, None)
    } else {
        return;
    };
    raw[base_at..base_at + 4].copy_from_slice(&base.to_le_bytes());
    if let Some(era) = extra {
        raw[extra_at..extra_at + 4].copy_from_slice(&era.to_le_bytes());
    }
}

impl Inode {
    /// Parse an inode from its on-disk bytes.
    /// Accepts any length >= 128; if >= 160 and i_extra_isize >= 28, parses the
    /// extra (nsec + crtime + checksum_hi) section as well.
    pub fn parse(raw: &[u8]) -> Result<Self> {
        if raw.len() < INODE_BASE_SIZE {
            return Err(Error::Corrupt("inode buffer too small"));
        }

        let mode = u16::from_le_bytes(raw[OFF_MODE..OFF_MODE + 2].try_into().unwrap());
        let uid_lo = u16::from_le_bytes(raw[0x02..0x04].try_into().unwrap());
        let size_lo = u32::from_le_bytes(raw[OFF_SIZE_LO..OFF_SIZE_LO + 4].try_into().unwrap());
        let atime_base = u32::from_le_bytes(raw[OFF_ATIME..OFF_ATIME + 4].try_into().unwrap());
        let ctime_base = u32::from_le_bytes(raw[OFF_CTIME..OFF_CTIME + 4].try_into().unwrap());
        let mtime_base = u32::from_le_bytes(raw[OFF_MTIME..OFF_MTIME + 4].try_into().unwrap());
        let dtime = u32::from_le_bytes(raw[0x14..0x18].try_into().unwrap());
        let gid_lo = u16::from_le_bytes(raw[0x18..0x1A].try_into().unwrap());
        let links_count = u16::from_le_bytes(
            raw[OFF_LINKS_COUNT..OFF_LINKS_COUNT + 2]
                .try_into()
                .unwrap(),
        );
        let blocks_lo =
            u32::from_le_bytes(raw[OFF_BLOCKS_LO..OFF_BLOCKS_LO + 4].try_into().unwrap());
        let flags = u32::from_le_bytes(raw[OFF_FLAGS..OFF_FLAGS + 4].try_into().unwrap());
        // 0x24..0x28 is i_osd1 (Linux: i_version_lo) — ignored here.

        let mut block = [0u8; 60];
        block.copy_from_slice(&raw[OFF_BLOCK..OFF_BLOCK + 60]);

        let generation =
            u32::from_le_bytes(raw[OFF_GENERATION..OFF_GENERATION + 4].try_into().unwrap());
        let file_acl_lo = u32::from_le_bytes(raw[0x68..0x6C].try_into().unwrap());
        let size_hi = u32::from_le_bytes(raw[OFF_SIZE_HI..OFF_SIZE_HI + 4].try_into().unwrap());
        // 0x70..0x74 obso_faddr ignored.
        let blocks_hi =
            u16::from_le_bytes(raw[OFF_BLOCKS_HI..OFF_BLOCKS_HI + 2].try_into().unwrap());
        let file_acl_hi = u16::from_le_bytes(raw[0x76..0x78].try_into().unwrap());
        let uid_hi = u16::from_le_bytes(raw[0x78..0x7A].try_into().unwrap());
        let gid_hi = u16::from_le_bytes(raw[0x7A..0x7C].try_into().unwrap());
        let checksum_lo = u16::from_le_bytes(
            raw[OFF_CHECKSUM_LO..OFF_CHECKSUM_LO + 2]
                .try_into()
                .unwrap(),
        );
        // 0x7E..0x80 i_reserved2.

        // Defaults (when no extra section present).
        let mut atime_nsec = 0u32;
        let mut mtime_nsec = 0u32;
        let mut ctime_nsec = 0u32;
        let mut crtime_nsec = 0u32;
        let mut crtime_base = 0u32;
        // The `*_extra` words, zero when i_extra_isize is too small to
        // hold them — which correctly yields no epoch extension.
        let mut atime_extra = 0u32;
        let mut mtime_extra = 0u32;
        let mut ctime_extra = 0u32;
        let mut crtime_extra = 0u32;
        let mut checksum_hi = 0u16;

        // Extra fields — only present when on-disk inode size is >= 160 AND
        // i_extra_isize covers them (>= 28 includes through i_projid; we read
        // what we need at >= 24 to cover up to crtime_extra).
        if raw.len() >= INODE_EXTRA_OFFSET + 4 {
            let i_extra_isize = u16::from_le_bytes(
                raw[OFF_EXTRA_ISIZE..OFF_EXTRA_ISIZE + 2]
                    .try_into()
                    .unwrap(),
            );
            // Sanity: i_extra_isize is the number of bytes beyond the 128-byte
            // base that are valid. Must fit inside the on-disk inode.
            let extra_end = INODE_EXTRA_OFFSET + i_extra_isize as usize;
            if extra_end > raw.len() {
                return Err(Error::Corrupt("i_extra_isize exceeds inode size"));
            }

            // Read each extra field only if i_extra_isize covers it.
            // Layout (offset from inode start):
            //   0x80 u16 i_extra_isize
            //   0x82 u16 i_checksum_hi          (needs >= 4)
            //   0x84 u32 i_ctime_extra          (needs >= 8)
            //   0x88 u32 i_mtime_extra          (needs >= 12)
            //   0x8C u32 i_atime_extra          (needs >= 16)
            //   0x90 u32 i_crtime               (needs >= 20)
            //   0x94 u32 i_crtime_extra         (needs >= 24)
            if i_extra_isize >= 4 {
                checksum_hi = u16::from_le_bytes(
                    raw[OFF_CHECKSUM_HI..OFF_CHECKSUM_HI + 2]
                        .try_into()
                        .unwrap(),
                );
            }
            if i_extra_isize >= 8 {
                let extra = u32::from_le_bytes(raw[0x84..0x88].try_into().unwrap());
                ctime_nsec = extra >> 2;
                ctime_extra = extra;
            }
            if i_extra_isize >= 12 {
                let extra = u32::from_le_bytes(raw[0x88..0x8C].try_into().unwrap());
                mtime_nsec = extra >> 2;
                mtime_extra = extra;
            }
            if i_extra_isize >= 16 {
                let extra = u32::from_le_bytes(raw[0x8C..0x90].try_into().unwrap());
                atime_nsec = extra >> 2;
                atime_extra = extra;
            }
            if i_extra_isize >= 20 {
                crtime_base =
                    u32::from_le_bytes(raw[OFF_CRTIME..OFF_CRTIME + 4].try_into().unwrap());
            }
            if i_extra_isize >= 24 {
                let extra = u32::from_le_bytes(raw[0x94..0x98].try_into().unwrap());
                crtime_nsec = extra >> 2;
                crtime_extra = extra;
            }
        }

        Ok(Self {
            mode,
            uid: join16(uid_hi, uid_lo),
            gid: join16(gid_hi, gid_lo),
            size: join32(size_hi, size_lo),
            atime: unpack_seconds(atime_base, atime_extra),
            mtime: unpack_seconds(mtime_base, mtime_extra),
            ctime: unpack_seconds(ctime_base, ctime_extra),
            // dtime has no *_extra field in the format: deletion time
            // is a plain signed 32-bit value with no epoch extension.
            dtime: dtime as i32 as i64,
            crtime: unpack_seconds(crtime_base, crtime_extra),
            atime_nsec,
            mtime_nsec,
            ctime_nsec,
            crtime_nsec,
            links_count,
            blocks: join32(blocks_hi, blocks_lo),
            flags,
            block,
            generation,
            file_acl: join32(file_acl_hi, file_acl_lo),
            checksum: join16(checksum_hi, checksum_lo),
        })
    }

    /// File type from i_mode.
    pub fn file_type(&self) -> u16 {
        self.mode & S_IFMT
    }

    pub fn is_dir(&self) -> bool {
        self.file_type() == S_IFDIR
    }

    pub fn is_file(&self) -> bool {
        self.file_type() == S_IFREG
    }

    pub fn is_symlink(&self) -> bool {
        self.file_type() == S_IFLNK
    }

    /// True when EXT4_EXTENTS_FL is set in i_flags — i_block holds an extent
    /// tree rather than legacy direct/indirect block pointers.
    pub fn has_extents(&self) -> bool {
        self.flags & InodeFlags::EXTENTS.bits() != 0
    }

    /// True when INLINE_DATA flag is set — file contents live inside i_block.
    pub fn has_inline_data(&self) -> bool {
        self.flags & InodeFlags::INLINE_DATA.bits() != 0
    }

    /// Decode i_flags into a typed bitflags value (silently drops unknown bits).
    pub fn flag_set(&self) -> InodeFlags {
        InodeFlags::from_bits_truncate(self.flags)
    }

    /// Inspect this directory's naming metadata, without performing name folding.
    ///
    /// `None` means byte-sensitive names; `Some` identifies a recognized
    /// casefold encoding. Recognition does not promise lookup or write support.
    /// Encrypted directories are unsupported, and a CASEFOLD flag on a
    /// non-directory or without the volume feature is corrupt metadata.
    /// The volume's encoding is validated even for an ordinary directory when
    /// the volume declares CASEFOLD. Other mount restrictions remain separate.
    pub fn directory_casefold_encoding(
        &self,
        sb: &crate::superblock::Superblock,
    ) -> Result<Option<crate::superblock::CasefoldEncoding>> {
        let folded = self.flag_set().contains(InodeFlags::CASEFOLD);
        if !self.is_dir() {
            return Err(if folded {
                Error::Corrupt("casefold flag on a non-directory inode")
            } else {
                Error::NotADirectory
            });
        }
        if folded && sb.feature_incompat & crate::features::Incompat::CASEFOLD.bits() == 0 {
            return Err(Error::Corrupt("casefold inode without the volume feature"));
        }
        if self.flag_set().contains(InodeFlags::ENCRYPT) {
            return Err(Error::Unsupported("encrypted directory name policy"));
        }
        let encoding = sb.casefold_encoding()?;
        Ok(if folded { encoding } else { None })
    }
}

/// Combine two 16-bit halves into a 32-bit value (hi occupies the upper 16 bits).
/// Used when the on-disk layout stores a 32-bit field split across two u16 words.
#[inline]
fn join16(hi: u16, lo: u16) -> u32 {
    ((hi as u32) << 16) | lo as u32
}

/// Combine a hi half (any type that fits in u64) and a 32-bit lo half into a
/// 64-bit value. Used for size, file_acl, and i_blocks whose hi halves have
/// different widths (u16 or u32) in the on-disk layout.
#[inline]
fn join32<H: Into<u64>>(hi: H, lo: u32) -> u64 {
    (hi.into() << 32) | lo as u64
}

#[cfg(test)]
mod timestamp_tests {
    use super::{pack_seconds, unpack_seconds, MAX_ENCODABLE_TIME, MIN_ENCODABLE_TIME};

    /// `(seconds, base word, era bits)` as the kernel in the harness VM
    /// wrote them for `touch -m -d @seconds` on a 256-byte-inode volume,
    /// read back with `debugfs stat` (`mtime: 0xBASE:EXTRA`). The same
    /// list drives the live check in `needs_host` below, so a change in
    /// what the kernel writes fails there rather than going unnoticed.
    const KERNEL_WORDS: &[(i64, u32, u32)] = &[
        (-2_147_483_648, 0x8000_0000, 0), // 1901-12-13, the earliest
        (-1_000, 0xFFFF_FC18, 0),         // 1969
        (-1, 0xFFFF_FFFF, 0),             // the second before the epoch
        (0, 0x0000_0000, 0),              // 1970-01-01
        (1, 0x0000_0001, 0),
        (2_147_483_647, 0x7FFF_FFFF, 0),  // 2038-01-19 03:14:07
        (2_147_483_648, 0x8000_0000, 1),  // one second later: era 1
        (4_294_967_295, 0xFFFF_FFFF, 1),  // 2106-02-07 06:28:15
        (4_294_967_296, 0x0000_0000, 1),  // 2106-02-07 06:28:16
        (6_442_450_943, 0x7FFF_FFFF, 1),  // 2174
        (6_442_450_944, 0x8000_0000, 2),  // 2174: era 2
        (8_589_934_592, 0x0000_0000, 2),  // 2242-03-16
        (12_884_901_887, 0xFFFF_FFFF, 3), // 2378, already era 3
        (12_884_901_888, 0x0000_0000, 3),
        (15_032_385_535, 0x7FFF_FFFF, 3), // 2446-05-10 22:38:55, the last
    ];

    #[test]
    fn every_kernel_written_word_pair_reads_back_as_its_seconds() {
        for &(secs, base, era) in KERNEL_WORDS {
            assert_eq!(unpack_seconds(base, era), secs, "{base:#x}:{era}");
        }
    }

    #[test]
    fn every_second_in_the_table_packs_to_the_kernels_words() {
        for &(secs, base, era) in KERNEL_WORDS {
            assert_eq!(pack_seconds(secs), (base, era), "{secs}");
        }
    }

    /// The bounds are the table's first and last rows.
    #[test]
    fn the_encodable_range_is_1901_to_2446() {
        assert_eq!(MIN_ENCODABLE_TIME, KERNEL_WORDS[0].0);
        assert_eq!(MAX_ENCODABLE_TIME, KERNEL_WORDS[KERNEL_WORDS.len() - 1].0);
    }

    /// Nanoseconds live above the era bits and never reach the seconds.
    #[test]
    fn nanosecond_bits_are_not_seconds() {
        let all_nanosecond_bits = !0b11u32;
        assert_eq!(unpack_seconds(1_000, all_nanosecond_bits), 1_000);
        assert_eq!(
            unpack_seconds(1_000, all_nanosecond_bits | 0b10),
            1_000 + (2 << 32)
        );
    }

    /// Every window edge, and a second either side of it, survives a
    /// pack and an unpack.
    #[test]
    fn window_edges_survive_a_round_trip() {
        for era in 0..4i64 {
            let lowest = i64::from(i32::MIN) + (era << 32);
            let highest = i64::from(i32::MAX) + (era << 32);
            for secs in [lowest, lowest + 1, highest - 1, highest] {
                let (base, bits) = pack_seconds(secs);
                assert_eq!(bits as i64, era, "{secs}");
                assert_eq!(unpack_seconds(base, bits), secs, "{secs}");
            }
        }
    }

    /// Tests that ask the kernel and debugfs in the harness VM.
    mod needs_host {
        use super::KERNEL_WORDS;
        use crate::block_io::FileDevice;
        use crate::fs::Filesystem;
        use std::sync::Arc;

        /// The kernel stamps one file per row of [`KERNEL_WORDS`] (and three
        /// past the end of the range, which it clamps); `debugfs` reports
        /// the raw words it wrote and `stat` the seconds it reads back.
        /// Both must match the table, and this crate must read every file's
        /// mtime as the kernel does.
        #[test]
        fn the_kernel_writes_and_reads_the_documented_words() {
            let image =
                fs_ext4_test_support::temp_path!("fs_ext4_inode_times_{}.img", std::process::id());
            std::fs::File::create(&image)
                .and_then(|f| f.set_len(8 << 20))
                .unwrap();
            let out = fs_ext4_test_support::oracle("mkfs.ext4")
                .args(["-q", "-F", "-I", "256", &image])
                .output();
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );

            let past_the_end = [15_032_385_536i64, 17_179_869_184, 99_999_999_999];
            let mut all: Vec<i64> = KERNEL_WORDS.iter().map(|row| row.0).collect();
            all.extend(past_the_end);
            let mut script = String::from("set -e\n");
            for secs in &all {
                script.push_str(&format!(
                    "touch \"$MNT/t{secs}\"; touch -m -d @{secs} \"$MNT/t{secs}\"\n"
                ));
            }
            for secs in &all {
                script.push_str(&format!(
                    "printf '%s %s\\n' {secs} \"$(stat -c %Y \"$MNT/t{secs}\")\"\n"
                ));
            }
            let out = fs_ext4_test_support::guest_kernel_write(&image, &script);
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            let kernel_reads: std::collections::BTreeMap<i64, i64> =
                String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .filter_map(|l| {
                        let (a, b) = l.split_once(' ')?;
                        Some((a.parse().ok()?, b.trim().parse().ok()?))
                    })
                    .collect();

            let fs = Filesystem::mount(Arc::new(FileDevice::open(&image).unwrap())).unwrap();
            let last = KERNEL_WORDS[KERNEL_WORDS.len() - 1];
            for &secs in &all {
                let (want_secs, want_base, want_era) = KERNEL_WORDS
                    .iter()
                    .copied()
                    .find(|row| row.0 == secs)
                    .unwrap_or(last);
                assert_eq!(
                    kernel_reads.get(&secs),
                    Some(&want_secs),
                    "kernel stat of @{secs}"
                );

                let out = fs_ext4_test_support::oracle("debugfs")
                    .args(["-R", &format!("stat /t{secs}"), &image])
                    .output();
                let text = String::from_utf8_lossy(&out.stdout).into_owned();
                let words = text
                    .lines()
                    .find_map(|l| l.trim().strip_prefix("mtime: 0x"))
                    .unwrap_or_else(|| panic!("no mtime line for @{secs}: {text}"));
                let (base, extra) = words.split_once(":").unwrap();
                let base = u32::from_str_radix(base, 16).unwrap();
                let extra = u32::from_str_radix(&extra[..8], 16).unwrap();
                assert_eq!(
                    (base, extra & 3),
                    (want_base, want_era),
                    "debugfs words for @{secs}"
                );

                let mut lookup = |ino: u32| fs.read_inode_verified(ino).map(|(inode, _)| inode);
                let ino =
                    crate::path::lookup(fs.dev.as_ref(), &fs.sb, &mut lookup, &format!("/t{secs}"))
                        .unwrap();
                let inode = fs.read_inode_verified(ino).unwrap().0;
                assert_eq!(inode.mtime, want_secs, "this crate's reading of @{secs}");
            }
            drop(fs);
            let _ = std::fs::remove_file(&image);
        }

        /// The other direction: this crate stamps each row's seconds with
        /// `apply_utimens`, and the kernel's `stat` must read them back.
        #[test]
        fn the_kernel_reads_what_this_crate_stamps() {
            let image = fs_ext4_test_support::temp_path!(
                "fs_ext4_inode_times_w_{}.img",
                std::process::id()
            );
            std::fs::File::create(&image)
                .and_then(|f| f.set_len(8 << 20))
                .unwrap();
            let out = fs_ext4_test_support::oracle("mkfs.ext4")
                .args(["-q", "-F", "-I", "256", &image])
                .output();
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            {
                let fs = Filesystem::mount(Arc::new(FileDevice::open_rw(&image).unwrap())).unwrap();
                for &(secs, _, _) in KERNEL_WORDS {
                    let path = format!("/w{secs}");
                    fs.apply_create(&path, 0o644).unwrap();
                    fs.apply_utimens(&path, secs, 0, secs, 0).unwrap();
                }
            }
            // No `e2fsck` verdict here: e2fsck 1.47 reports a time in era 3
            // whose base is negative (2310-04-04 .. 2378-04-22) as "likely
            // pre-1970" -- the documentation's note on old kernels that
            // wrote era 3 for 1901..1970 -- although the kernel above wrote
            // those same words for those same seconds. The kernel is the
            // reader that matters.
            let mut script = String::from("set -e\n");
            for &(secs, _, _) in KERNEL_WORDS {
                script.push_str(&format!(
                    "printf '%s %s\\n' {secs} \"$(stat -c %Y \"$MNT/w{secs}\")\"\n"
                ));
            }
            let out = fs_ext4_test_support::guest_kernel_write(&image, &script);
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            let stdout = String::from_utf8_lossy(&out.stdout);
            for &(secs, _, _) in KERNEL_WORDS {
                assert!(
                    stdout.lines().any(|l| l == format!("{secs} {secs}")),
                    "the kernel did not read @{secs} back:\n{stdout}"
                );
            }
            let _ = std::fs::remove_file(&image);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join16_combines_halves() {
        assert_eq!(join16(0x0001, 0x0002), 0x0001_0002);
        assert_eq!(join16(0xFFFF, 0x0000), 0xFFFF_0000);
        assert_eq!(join16(0x0000, 0xFFFF), 0x0000_FFFF);
        assert_eq!(join16(0, 0), 0);
    }

    #[test]
    fn join32_combines_halves_u16_hi() {
        assert_eq!(join32(0x0001u16, 0x0000_0002), 0x0000_0001_0000_0002);
        assert_eq!(join32(0xFFFFu16, 0x0000_0000), 0x0000_FFFF_0000_0000);
        assert_eq!(join32(0x0000u16, 0xFFFF_FFFF), 0x0000_0000_FFFF_FFFF);
    }

    #[test]
    fn join32_combines_halves_u32_hi() {
        assert_eq!(join32(0x0000_0001u32, 0x0000_0002), 0x0000_0001_0000_0002);
        assert_eq!(join32(0xFFFF_FFFFu32, 0x0000_0000), 0xFFFF_FFFF_0000_0000);
    }

    #[test]
    fn parse_rejects_short_buffer() {
        let short = vec![0u8; 64];
        assert!(matches!(
            Inode::parse(&short),
            Err(crate::error::Error::Corrupt(_))
        ));
    }

    #[test]
    fn parse_rejects_invalid_extra_isize() {
        // 160-byte inode with i_extra_isize claiming 200 bytes (exceeds buffer).
        let mut raw = vec![0u8; 160];
        raw[0x80] = 200; // i_extra_isize lo byte — claims 200 bytes extra
        raw[0x81] = 0;
        assert!(matches!(
            Inode::parse(&raw),
            Err(crate::error::Error::Corrupt(_))
        ));
    }

    #[test]
    fn parse_mode_and_links_roundtrip() {
        let mut raw = vec![0u8; 128];
        raw[0x00..0x02].copy_from_slice(&0x81A4u16.to_le_bytes()); // S_IFREG | 0644
        raw[0x1A..0x1C].copy_from_slice(&3u16.to_le_bytes()); // links_count
        let inode = Inode::parse(&raw).unwrap();
        assert_eq!(inode.mode, 0x81A4);
        assert_eq!(inode.links_count, 3);
        assert!(inode.is_file());
    }
}

/// `set_inode_time` stores the era bits where the inode has room for
/// them, and a clamp where it does not.
#[cfg(test)]
mod set_inode_time_tests {
    use super::{set_inode_time, Inode, InodeTime, OFF_ATIME, OFF_CRTIME, OFF_EXTRA_ISIZE};

    const PAST_2038: i64 = (1i64 << 31) + 10;

    fn inode(len: usize, extra_isize: u16) -> Vec<u8> {
        let mut raw = vec![0u8; len];
        if len >= OFF_EXTRA_ISIZE + 2 {
            raw[OFF_EXTRA_ISIZE..OFF_EXTRA_ISIZE + 2].copy_from_slice(&extra_isize.to_le_bytes());
        }
        raw
    }

    fn le32(raw: &[u8], off: usize) -> u32 {
        u32::from_le_bytes(raw[off..off + 4].try_into().unwrap())
    }

    #[test]
    fn a_large_inode_keeps_the_epoch_bits() {
        let mut raw = inode(256, 32);
        for field in [
            InodeTime::Atime,
            InodeTime::Ctime,
            InodeTime::Mtime,
            InodeTime::Crtime,
        ] {
            set_inode_time(&mut raw, field, PAST_2038);
        }
        let parsed = Inode::parse(&raw).unwrap();
        assert_eq!(parsed.atime, PAST_2038);
        assert_eq!(parsed.ctime, PAST_2038);
        assert_eq!(parsed.mtime, PAST_2038);
        assert_eq!(parsed.crtime, PAST_2038);
        // Era 1 with a base whose signed reading is negative; no nsec.
        assert_eq!(le32(&raw, OFF_ATIME), 0x8000_000A);
        assert_eq!(le32(&raw, 0x8C), 1);
    }

    #[test]
    fn a_new_stamp_clears_the_old_nanoseconds() {
        let mut raw = inode(256, 32);
        raw[0x88..0x8C].copy_from_slice(&(123u32 << 2).to_le_bytes());
        set_inode_time(&mut raw, InodeTime::Mtime, 1_700_000_000);
        let parsed = Inode::parse(&raw).unwrap();
        assert_eq!(parsed.mtime, 1_700_000_000);
        assert_eq!(parsed.mtime_nsec, 0);
    }

    #[test]
    fn a_small_inode_clamps_instead_of_wrapping() {
        let mut raw = inode(128, 0);
        set_inode_time(&mut raw, InodeTime::Mtime, PAST_2038);
        assert_eq!(Inode::parse(&raw).unwrap().mtime, i32::MAX as i64);
        set_inode_time(&mut raw, InodeTime::Mtime, i32::MIN as i64 - 10);
        assert_eq!(Inode::parse(&raw).unwrap().mtime, i32::MIN as i64);
    }

    #[test]
    fn an_extra_isize_too_small_for_the_field_clamps_it() {
        // 12 covers ctime_extra and mtime_extra, not atime_extra.
        let mut raw = inode(256, 12);
        set_inode_time(&mut raw, InodeTime::Mtime, PAST_2038);
        set_inode_time(&mut raw, InodeTime::Atime, PAST_2038);
        let parsed = Inode::parse(&raw).unwrap();
        assert_eq!(parsed.mtime, PAST_2038);
        assert_eq!(parsed.atime, i32::MAX as i64);
        assert_eq!(le32(&raw, 0x8C), 0, "atime_extra is outside i_extra_isize");
    }

    #[test]
    fn crtime_is_not_written_where_the_inode_has_no_room_for_it() {
        let mut raw = inode(128, 0);
        set_inode_time(&mut raw, InodeTime::Crtime, 1_700_000_000);
        assert!(raw.iter().all(|&b| b == 0));
        let mut raw = inode(256, 16);
        set_inode_time(&mut raw, InodeTime::Crtime, 1_700_000_000);
        assert_eq!(le32(&raw, OFF_CRTIME), 0);
    }

    #[test]
    fn past_the_format_ceiling_is_clamped_to_it() {
        let mut raw = inode(256, 32);
        set_inode_time(&mut raw, InodeTime::Mtime, i64::MAX);
        assert_eq!(Inode::parse(&raw).unwrap().mtime, super::MAX_ENCODABLE_TIME);
    }
}
