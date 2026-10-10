//! Metadata recognition and lookup validation. Linux-written counterparts are checked in
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

struct DirectoryBlock {
    names: Vec<Vec<u8>>,
    index_depth: Option<u8>,
    reads: std::sync::atomic::AtomicUsize,
}

impl fs_ext4::block_io::BlockDevice for DirectoryBlock {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> fs_ext4::Result<()> {
        assert_eq!((offset, buf.len()), (1024, 1024));
        self.reads
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        buf.fill(0);
        let mut offset = 0;
        for (index, name) in self.names.iter().enumerate() {
            let length = if index + 1 == self.names.len() {
                buf.len() - offset
            } else {
                (8 + name.len()).next_multiple_of(4)
            };
            buf[offset..offset + 4].copy_from_slice(&(42 + index as u32).to_le_bytes());
            buf[offset + 4..offset + 6].copy_from_slice(&(length as u16).to_le_bytes());
            buf[offset + 6..offset + 8].copy_from_slice(&(name.len() as u16).to_le_bytes());
            buf[offset + 8..offset + 8 + name.len()].copy_from_slice(name);
            offset += length;
        }
        if let Some(depth) = self.index_depth {
            buf[24..32].copy_from_slice(&[0, 0, 0, 0, 1, 8, depth, 0]);
        }
        Ok(())
    }

    fn size_bytes(&self) -> u64 {
        2048
    }
}

fn lookup_directory(flags: u32) -> Inode {
    let mut dir = inode(S_IFDIR, flags);
    dir.size = 1024;
    dir.block[..4].copy_from_slice(&1_u32.to_le_bytes());
    dir
}

fn lookup_device(name: &[u8]) -> DirectoryBlock {
    DirectoryBlock {
        names: vec![name.to_vec()],
        index_depth: None,
        reads: std::sync::atomic::AtomicUsize::new(0),
    }
}

#[test]
fn lookup_refuses_unknown_casefold_encoding_before_reading_entries() {
    for (id, flags) in [(0, 0), (2, 0), (1, 2)] {
        for inode_flags in [0, InodeFlags::CASEFOLD.bits()] {
            let device = lookup_device(b"ReadMe");
            let mut read = |_| Ok(lookup_directory(inode_flags));
            let result =
                fs_ext4::path::lookup(&device, &superblock(true, id, flags), &mut read, "/ReadMe");
            assert!(matches!(result, Err(Error::Unsupported(_))), "{result:?}");
            assert_eq!(device.reads.load(std::sync::atomic::Ordering::Relaxed), 0);
        }
    }
}

#[test]
fn lookup_refuses_inconsistent_casefold_flags_in_each_path_component() {
    for path in ["/ReadMe", "/ReadMe/ReadMe"] {
        let device = lookup_device(b"ReadMe");
        let mut read = |ino| {
            let folded = path == "/ReadMe" || ino == 42;
            Ok(lookup_directory(if folded {
                InodeFlags::CASEFOLD.bits()
            } else {
                0
            }))
        };
        let result = fs_ext4::path::lookup(&device, &superblock(false, 1, 0), &mut read, path);
        assert!(matches!(result, Err(Error::Corrupt(_))), "{result:?}");
        assert_eq!(
            device.reads.load(std::sync::atomic::Ordering::Relaxed),
            usize::from(path == "/ReadMe/ReadMe")
        );
    }
}

#[test]
fn lookup_keeps_ordinary_byte_names_and_recognized_encoding_reads() {
    for strict in [0, 1] {
        for inode_flags in [0, InodeFlags::CASEFOLD.bits()] {
            let device = lookup_device(b"ReadMe");
            let mut read = |_| Ok(lookup_directory(inode_flags));
            assert_eq!(
                fs_ext4::path::lookup(&device, &superblock(true, 1, strict), &mut read, "/ReadMe")
                    .unwrap(),
                42
            );
        }
    }
    let device = lookup_device(b"A\xff");
    let sb = superblock(false, u16::MAX, u16::MAX);
    let mut read = |_| Ok(lookup_directory(0));
    let csum = fs_ext4::checksum::Checksummer::from_superblock(&sb);
    assert_eq!(
        fs_ext4::path::lookup_bytes_with_csum(&device, &sb, &mut read, b"/A\xff", &csum).unwrap(),
        42
    );
    assert!(matches!(
        fs_ext4::path::lookup_bytes_with_csum(&device, &sb, &mut read, b"/a\xff", &csum),
        Err(Error::NotFound)
    ));
}

#[test]
fn folded_lookup_uses_frozen_unicode_and_ordinary_lookup_keeps_raw_names() {
    for (stored, alias) in [
        ("ReadMe".to_owned(), "README".to_owned()),
        ("Café".to_owned(), "CAFE\u{0301}".to_owned()),
        ("Straße".to_owned(), "STRASSE".to_owned()),
        ("A\u{00ad}B".to_owned(), "ab".to_owned()),
        ("\u{00ad}".to_owned(), "\u{fe0f}".to_owned()),
        ("ΐ".repeat(80), "\u{1fd3}".repeat(80)),
    ] {
        for strict in [0, 1] {
            let sb = superblock(true, 1, strict);
            for folded in [false, true] {
                let device = lookup_device(stored.as_bytes());
                let mut read = |_| {
                    Ok(lookup_directory(if folded {
                        InodeFlags::CASEFOLD.bits()
                    } else {
                        0
                    }))
                };
                let result = fs_ext4::path::lookup(&device, &sb, &mut read, &alias);
                if folded {
                    assert_eq!(result.unwrap(), 42, "{stored:?} / {alias:?}");
                } else {
                    assert!(matches!(result, Err(Error::NotFound)), "{result:?}");
                }
            }
        }
    }
}

#[test]
fn folded_lookup_refuses_unqualified_names_instead_of_claiming_absence() {
    for strict in [0, 1] {
        let sb = superblock(true, 1, strict);
        let csum = fs_ext4::checksum::Checksummer::from_superblock(&sb);
        for name in [
            b"A\xff".as_slice(),
            "\u{00ad}.".as_bytes(),
            "\u{00ad}..".as_bytes(),
        ] {
            let device = lookup_device(b"other");
            let mut read = |_| Ok(lookup_directory(InodeFlags::CASEFOLD.bits()));
            let result =
                fs_ext4::path::lookup_bytes_with_csum(&device, &sb, &mut read, name, &csum);
            assert!(matches!(result, Err(Error::Unsupported(_))), "{result:?}");
            assert_eq!(device.reads.load(std::sync::atomic::Ordering::Relaxed), 0);
        }
        let device = lookup_device(b"A\xff");
        let mut read = |_| Ok(lookup_directory(InodeFlags::CASEFOLD.bits()));
        assert!(matches!(
            fs_ext4::path::lookup(&device, &sb, &mut read, "absent"),
            Err(Error::Unsupported(_))
        ));
        let device = lookup_device(b".");
        assert_eq!(
            fs_ext4::path::lookup(&device, &sb, &mut read, ".").unwrap(),
            42
        );
        assert!(matches!(
            fs_ext4::path::lookup(&device, &sb, &mut read, &"A".repeat(256)),
            Err(Error::NameTooLong)
        ));
    }
}

#[test]
fn folded_lookup_refuses_duplicate_equivalent_directory_entries() {
    let mut device = lookup_device(b"ReadMe");
    device.names.push(b"README".to_vec());
    let mut read = |_| Ok(lookup_directory(InodeFlags::CASEFOLD.bits()));
    assert!(matches!(
        fs_ext4::path::lookup(&device, &superblock(true, 1, 0), &mut read, "readme"),
        Err(Error::CorruptDirEntry(_))
    ));
}

#[test]
fn folded_inline_lookup_remains_unsupported() {
    let device = lookup_device(b"unused");
    let mut read = |_| {
        Ok(lookup_directory(
            InodeFlags::CASEFOLD.bits() | InodeFlags::INLINE_DATA.bits(),
        ))
    };
    assert!(matches!(
        fs_ext4::path::lookup(&device, &superblock(true, 1, 0), &mut read, "name"),
        Err(Error::Unsupported(_))
    ));
    assert_eq!(device.reads.load(std::sync::atomic::Ordering::Relaxed), 0);
}

#[test]
fn folded_lookup_refuses_deeper_indexes_before_using_them() {
    for depth in [2, 255] {
        let mut device = lookup_device(b"ReadMe");
        device.index_depth = Some(depth);
        let mut read = |_| {
            Ok(lookup_directory(
                InodeFlags::CASEFOLD.bits() | InodeFlags::INDEX.bits(),
            ))
        };
        assert!(matches!(
            fs_ext4::path::lookup(&device, &superblock(true, 1, 0), &mut read, "README"),
            Err(Error::Unsupported(_))
        ));
    }
}

// Small synthetic trees isolate damage checks. The large-directory oracle
// separately qualifies the supported layout using Linux-created bytes.
struct IndexedBlocks(Vec<Vec<u8>>);

impl fs_ext4::block_io::BlockDevice for IndexedBlocks {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> fs_ext4::Result<()> {
        assert_eq!(offset % 1024, 0);
        assert_eq!(buf.len(), 1024);
        buf.copy_from_slice(&self.0[offset as usize / 1024 - 1]);
        Ok(())
    }

    fn size_bytes(&self) -> u64 {
        (self.0.len() as u64 + 1) * 1024
    }
}

fn deep_index() -> (IndexedBlocks, Inode, fs_ext4::checksum::Checksummer) {
    let csum = fs_ext4::checksum::Checksummer {
        seed: 123,
        enabled: true,
    };
    let mut blocks = vec![vec![0; 1024]; 5];
    blocks[0][24..32].copy_from_slice(&[0, 0, 0, 0, 1, 8, 1, 0]);
    blocks[0][32..36].copy_from_slice(&[123, 0, 2, 0]);
    blocks[0][36..40].copy_from_slice(&1_u32.to_le_bytes());
    blocks[0][40..44].copy_from_slice(&0x8000_0000_u32.to_le_bytes());
    blocks[0][44..48].copy_from_slice(&2_u32.to_le_bytes());
    assert!(csum.patch_dx_tail(2, 0, &mut blocks[0], 32));
    for node in [1, 2] {
        blocks[node][4..6].copy_from_slice(&1024_u16.to_le_bytes());
        blocks[node][8..12].copy_from_slice(&[126, 0, 1, 0]);
        blocks[node][12..16].copy_from_slice(&(node as u32 + 2).to_le_bytes());
        assert!(csum.patch_dx_tail(2, 0, &mut blocks[node], 8));
    }
    for (index, name) in [(3, "Café"), (4, "ReadMe")] {
        let block = &mut blocks[index];
        block[..4].copy_from_slice(&(40 + index as u32).to_le_bytes());
        block[4..6].copy_from_slice(&1012_u16.to_le_bytes());
        block[6] = name.len() as u8;
        block[8..8 + name.len()].copy_from_slice(name.as_bytes());
        block[1016..1018].copy_from_slice(&12_u16.to_le_bytes());
        block[1019] = 0xde;
        assert!(csum.patch_dir_entry_tail(2, 0, block));
    }
    let mut inode = lookup_directory(InodeFlags::CASEFOLD.bits() | InodeFlags::INDEX.bits());
    inode.size = 5 * 1024;
    for index in 0..5 {
        inode.block[index * 4..index * 4 + 4].copy_from_slice(&(index as u32 + 1).to_le_bytes());
    }
    (IndexedBlocks(blocks), inode, csum)
}

fn deep_lookup(
    device: &IndexedBlocks,
    inode: &Inode,
    csum: &fs_ext4::checksum::Checksummer,
    name: &str,
) -> fs_ext4::Result<u32> {
    fs_ext4::path::lookup_bytes_with_csum(
        device,
        &superblock(true, 1, 0),
        &mut |_| Ok(inode.clone()),
        name.as_bytes(),
        csum,
    )
}

#[test]
fn folded_lookup_reads_both_branches_of_a_deeper_index() {
    let (device, inode, csum) = deep_index();
    assert_eq!(
        deep_lookup(&device, &inode, &csum, "CAFE\u{0301}").unwrap(),
        43
    );
    assert_eq!(deep_lookup(&device, &inode, &csum, "README").unwrap(), 44);
    assert!(matches!(
        deep_lookup(&device, &inode, &csum, "missing"),
        Err(Error::NotFound)
    ));
}

#[test]
fn folded_deep_lookup_verifies_root_node_and_leaf_checksums() {
    for (index, byte) in [(0, 40), (1, 12), (2, 12), (3, 8), (4, 8)] {
        let (mut device, inode, csum) = deep_index();
        device.0[index][byte] ^= 1;
        assert!(
            matches!(
                deep_lookup(&device, &inode, &csum, "missing"),
                Err(Error::BadChecksum { .. })
            ),
            "block {index}"
        );
    }
}

#[test]
fn folded_deep_lookup_refuses_cycles_repeated_and_out_of_range_blocks() {
    for target in [0_u32, 1, 2, 3, 5, u32::MAX] {
        let (mut device, inode, csum) = deep_index();
        device.0[2][12..16].copy_from_slice(&target.to_le_bytes());
        assert!(csum.patch_dx_tail(2, 0, &mut device.0[2], 8));
        assert!(
            matches!(
                deep_lookup(&device, &inode, &csum, "missing"),
                Err(Error::CorruptDirEntry(_))
            ),
            "target {target}"
        );
    }
}

#[test]
fn folded_deep_lookup_refuses_sparse_nodes_and_leaves() {
    for logical in 0..5 {
        let (device, mut inode, csum) = deep_index();
        inode.block[logical * 4..logical * 4 + 4].fill(0);
        assert!(
            matches!(
                deep_lookup(&device, &inode, &csum, "missing"),
                Err(Error::CorruptDirEntry(_))
            ),
            "block {logical}"
        );
    }
}

#[test]
fn folded_deep_lookup_refuses_malformed_nodes_even_with_valid_checksums() {
    for (offset, bytes) in [
        (0, vec![1]),
        (4, vec![12, 0]),
        (6, vec![1]),
        (10, vec![0, 0]),
    ] {
        let (mut device, inode, csum) = deep_index();
        device.0[1][offset..offset + bytes.len()].copy_from_slice(&bytes);
        assert!(csum.patch_dx_tail(2, 0, &mut device.0[1], 8));
        assert!(deep_lookup(&device, &inode, &csum, "missing").is_err());
    }
}

#[test]
fn folded_deep_lookup_refuses_equivalent_names_in_different_leaves() {
    let (mut device, inode, csum) = deep_index();
    device.0[4] = device.0[3].clone();
    device.0[4][..4].copy_from_slice(&44_u32.to_le_bytes());
    assert!(csum.patch_dir_entry_tail(2, 0, &mut device.0[4]));
    assert!(matches!(
        deep_lookup(&device, &inode, &csum, "CAFE\u{0301}"),
        Err(Error::CorruptDirEntry(_))
    ));
}

#[test]
fn folded_deep_lookup_handles_no_checksum_layout_and_ignores_unlisted_blocks() {
    let (mut device, mut inode, mut csum) = deep_index();
    csum.enabled = false;
    device.0[0][32..34].copy_from_slice(&124_u16.to_le_bytes());
    for node in [1, 2] {
        device.0[node][8..10].copy_from_slice(&127_u16.to_le_bytes());
    }
    // An unreferenced stale copy must not become an ambiguous live entry.
    device.0.push(device.0[4].clone());
    inode.size += 1024;
    inode.block[20..24].copy_from_slice(&6_u32.to_le_bytes());
    assert_eq!(deep_lookup(&device, &inode, &csum, "README").unwrap(), 44);
}

#[test]
fn folded_deep_lookup_requires_leaf_checksum_tails() {
    let (mut device, inode, csum) = deep_index();
    device.0[4][1019] = 0;
    assert!(matches!(
        deep_lookup(&device, &inode, &csum, "README"),
        Err(Error::BadChecksum { .. })
    ));
}

fn dot_index() -> (IndexedBlocks, Inode, fs_ext4::checksum::Checksummer) {
    let (mut device, inode, csum) = deep_index();
    let root = &mut device.0[0];
    root[..4].copy_from_slice(&2_u32.to_le_bytes());
    root[4..6].copy_from_slice(&12_u16.to_le_bytes());
    root[6] = 1;
    root[8] = b'.';
    root[12..16].copy_from_slice(&42_u32.to_le_bytes());
    root[16..18].copy_from_slice(&1012_u16.to_le_bytes());
    root[18] = 2;
    root[20..22].copy_from_slice(b"..");
    assert!(csum.patch_dx_tail(2, 0, root, 32));
    (device, inode, csum)
}

#[test]
fn folded_index_dot_entries_navigate_without_hashing() {
    for depth in [0, 1] {
        for checksums in [false, true] {
            let (mut device, inode, mut csum) = dot_index();
            device.0[0][30] = depth;
            // Linux-created index roots can retain a recognizable old leaf
            // tail in padding. Only the current index checksum applies.
            device.0[0][1016..1018].copy_from_slice(&12_u16.to_le_bytes());
            device.0[0][1019] = 0xde;
            assert!(csum.patch_dx_tail(2, 0, &mut device.0[0], 32));
            csum.enabled = checksums;
            assert_eq!(deep_lookup(&device, &inode, &csum, ".").unwrap(), 2);
            assert_eq!(deep_lookup(&device, &inode, &csum, "..").unwrap(), 42);
        }
    }
}

#[test]
fn folded_index_navigation_checks_the_index_checksum() {
    for name in [".", ".."] {
        let (mut device, inode, csum) = dot_index();
        device.0[0][12] ^= 1;
        assert!(matches!(
            deep_lookup(&device, &inode, &csum, name),
            Err(Error::BadChecksum { .. })
        ));
    }
}

#[test]
fn folded_index_navigation_refuses_malformed_dot_records() {
    for (offset, bytes) in [
        (0, vec![3, 0, 0, 0]), // wrong self inode
        (4, vec![16, 0]),      // misplaced parent entry
        (8, vec![b'x']),
        (12, vec![0, 0, 0, 0]),                // missing parent
        (12, u32::MAX.to_le_bytes().to_vec()), // impossible parent
        (16, vec![12, 0]),                     // parent no longer covers the root
        (20, vec![b'x']),
    ] {
        let (mut device, inode, csum) = dot_index();
        device.0[0][offset..offset + bytes.len()].copy_from_slice(&bytes);
        assert!(csum.patch_dx_tail(2, 0, &mut device.0[0], 32));
        assert!(
            matches!(
                deep_lookup(&device, &inode, &csum, ".."),
                Err(Error::CorruptDirEntry(_))
            ),
            "offset {offset}"
        );
    }
}

#[test]
fn folded_index_navigation_keeps_unsupported_depth_and_alias_refusals() {
    let (mut device, inode, csum) = dot_index();
    for name in ["\u{00ad}.", "\u{00ad}.."] {
        assert!(matches!(
            deep_lookup(&device, &inode, &csum, name),
            Err(Error::Unsupported(_))
        ));
    }
    device.0[0][30] = 2;
    assert!(csum.patch_dx_tail(2, 0, &mut device.0[0], 32));
    assert!(matches!(
        deep_lookup(&device, &inode, &csum, "."),
        Err(Error::Unsupported(_))
    ));
}
