//! Metadata recognition only. Linux-written counterparts are checked in
//! `casefold_fixture_oracle`; these tests cover malformed and unsupported data.

use fs_ext4::{
    features::Incompat,
    inode::{Inode, InodeFlags, S_IFDIR, S_IFLNK, S_IFREG, USER_MODIFIABLE_FLAGS},
    superblock::{CasefoldEncoding, Superblock, EXT4_MAGIC, SUPERBLOCK_SIZE},
    Error,
};

fn superblock(feature: bool, encoding: u16, flags: u16) -> Superblock {
    let mut raw = vec![0; SUPERBLOCK_SIZE];
    raw[0x00..0x04].copy_from_slice(&2048_u32.to_le_bytes());
    raw[0x04..0x08].copy_from_slice(&8192_u32.to_le_bytes());
    raw[0x14..0x18].copy_from_slice(&1_u32.to_le_bytes());
    raw[0x38..0x3a].copy_from_slice(&EXT4_MAGIC.to_le_bytes());
    raw[0x20..0x24].copy_from_slice(&8192_u32.to_le_bytes());
    raw[0x28..0x2c].copy_from_slice(&2048_u32.to_le_bytes());
    raw[0x4c..0x50].copy_from_slice(&1_u32.to_le_bytes());
    raw[0x58..0x5a].copy_from_slice(&256_u16.to_le_bytes());
    if feature {
        raw[0x60..0x64].copy_from_slice(&Incompat::CASEFOLD.bits().to_le_bytes());
    }
    raw[0x27c..0x27e].copy_from_slice(&encoding.to_le_bytes());
    raw[0x27e..0x280].copy_from_slice(&flags.to_le_bytes());
    Superblock::parse(raw).unwrap()
}

fn inode(mode: u16, flags: u32) -> Inode {
    let mut raw = [0; 128];
    raw[0..2].copy_from_slice(&mode.to_le_bytes());
    raw[0x20..0x24].copy_from_slice(&flags.to_le_bytes());
    Inode::parse(&raw).unwrap()
}

#[test]
fn raw_encoding_fields_preserve_both_little_endian_bytes() {
    let sb = superblock(true, 0x1234, 0x5678);
    assert_eq!(sb.encoding_id().unwrap(), 0x1234);
    assert_eq!(sb.encoding_flags().unwrap(), 0x5678);
}

#[test]
fn every_truncated_encoding_field_is_an_error_instead_of_a_default() {
    let original = superblock(true, 1, 0);
    for len in 0..=0x280 {
        let mut sb = original.clone();
        sb.raw.truncate(len);
        if len < 0x27e {
            assert!(matches!(sb.encoding_id(), Err(Error::Corrupt(_))));
        } else {
            assert_eq!(sb.encoding_id().unwrap(), 1);
        }
        if len < 0x280 {
            assert!(matches!(sb.encoding_flags(), Err(Error::Corrupt(_))));
            assert!(matches!(sb.casefold_encoding(), Err(Error::Corrupt(_))));
        } else {
            assert_eq!(sb.encoding_flags().unwrap(), 0);
            assert!(sb.casefold_encoding().unwrap().is_some());
        }
    }
}

#[test]
fn both_known_strictness_modes_are_recognized() {
    for strict in [false, true] {
        let sb = superblock(true, 1, u16::from(strict));
        assert_eq!(
            sb.casefold_encoding().unwrap(),
            Some(CasefoldEncoding::Utf8_12_1 { strict })
        );
    }
}

#[test]
fn every_unknown_encoding_id_is_refused() {
    let mut sb = superblock(true, 1, 0);
    for id in (0..=u16::MAX).filter(|id| *id != 1) {
        sb.raw[0x27c..0x27e].copy_from_slice(&id.to_le_bytes());
        assert!(matches!(sb.casefold_encoding(), Err(Error::Unsupported(_))));
    }
}

#[test]
fn every_unknown_encoding_flag_combination_is_refused() {
    let mut sb = superblock(true, 1, 0);
    for flags in 2..=u16::MAX {
        sb.raw[0x27e..0x280].copy_from_slice(&flags.to_le_bytes());
        assert!(matches!(sb.casefold_encoding(), Err(Error::Unsupported(_))));
    }
}

#[test]
fn absent_feature_ignores_unused_encoding_fields() {
    let ordinary = inode(S_IFDIR | 0o755, 0);
    let mut sb = superblock(false, u16::MAX, u16::MAX);
    assert_eq!(sb.casefold_encoding().unwrap(), None);
    assert_eq!(ordinary.directory_casefold_encoding(&sb).unwrap(), None);
    sb.raw.clear();
    assert_eq!(sb.casefold_encoding().unwrap(), None);
    assert_eq!(ordinary.directory_casefold_encoding(&sb).unwrap(), None);
}

#[test]
fn naming_policy_is_per_directory_in_both_strictness_modes() {
    for strict in [false, true] {
        let sb = superblock(true, 1, u16::from(strict));
        for layout in [0, InodeFlags::INDEX.bits(), InodeFlags::EXTENTS.bits()] {
            let ordinary = inode(S_IFDIR | 0o755, layout);
            let folded = inode(S_IFDIR | 0o755, layout | InodeFlags::CASEFOLD.bits());
            assert_eq!(ordinary.directory_casefold_encoding(&sb).unwrap(), None);
            assert_eq!(
                folded.directory_casefold_encoding(&sb).unwrap(),
                Some(CasefoldEncoding::Utf8_12_1 { strict })
            );
        }
    }
}

#[test]
fn directories_do_not_hide_an_unknown_or_truncated_volume_encoding() {
    for flags in [0, InodeFlags::CASEFOLD.bits()] {
        let dir = inode(S_IFDIR, flags);
        for sb in [superblock(true, 0, 0), superblock(true, 1, 2)] {
            assert!(matches!(
                dir.directory_casefold_encoding(&sb),
                Err(Error::Unsupported(_))
            ));
        }
        let mut sb = superblock(true, 1, 0);
        sb.raw.truncate(0x27f);
        assert!(matches!(
            dir.directory_casefold_encoding(&sb),
            Err(Error::Corrupt(_))
        ));
    }
}

#[test]
fn a_folded_inode_requires_the_volume_feature() {
    let sb = superblock(false, 1, 0);
    let dir = inode(S_IFDIR, InodeFlags::CASEFOLD.bits());
    assert!(matches!(
        dir.directory_casefold_encoding(&sb),
        Err(Error::Corrupt(_))
    ));
}

#[test]
fn non_directories_cannot_have_directory_casefold_settings() {
    let sb = superblock(true, 1, 0);
    for mode in [0, S_IFREG, S_IFLNK, 0x1000, 0x2000, 0x6000, 0xc000] {
        assert!(matches!(
            inode(mode, 0).directory_casefold_encoding(&sb),
            Err(Error::NotADirectory)
        ));
        assert!(matches!(
            inode(mode, InodeFlags::CASEFOLD.bits()).directory_casefold_encoding(&sb),
            Err(Error::Corrupt(_))
        ));
    }
}

#[test]
fn encrypted_directories_never_get_a_plaintext_name_policy() {
    for feature in [false, true] {
        let sb = superblock(feature, 1, 0);
        let encrypted = inode(S_IFDIR, InodeFlags::ENCRYPT.bits());
        assert!(matches!(
            encrypted.directory_casefold_encoding(&sb),
            Err(Error::Unsupported(_))
        ));
    }
    let sb = superblock(true, 1, 0);
    let encrypted_folded = inode(S_IFDIR, (InodeFlags::ENCRYPT | InodeFlags::CASEFOLD).bits());
    assert!(matches!(
        encrypted_folded.directory_casefold_encoding(&sb),
        Err(Error::Unsupported(_))
    ));
}

#[test]
fn metadata_inspection_uses_current_superblock_and_inode_values() {
    let mut sb = superblock(true, 1, 0);
    let mut dir = inode(S_IFDIR, InodeFlags::CASEFOLD.bits());
    assert_eq!(
        dir.directory_casefold_encoding(&sb).unwrap(),
        Some(CasefoldEncoding::Utf8_12_1 { strict: false })
    );
    sb = superblock(true, 1, 1);
    assert_eq!(
        dir.directory_casefold_encoding(&sb).unwrap(),
        Some(CasefoldEncoding::Utf8_12_1 { strict: true })
    );
    dir.flags = 0;
    assert_eq!(dir.directory_casefold_encoding(&sb).unwrap(), None);
}

#[test]
fn recognizing_the_casefold_inode_bit_does_not_make_it_user_modifiable() {
    let dir = inode(S_IFDIR, 0x4000_0000);
    assert!(dir.flag_set().contains(InodeFlags::CASEFOLD));
    assert_eq!(dir.flag_set().bits(), 0x4000_0000);
    assert_eq!(USER_MODIFIABLE_FLAGS & dir.flag_set().bits(), 0);
}
