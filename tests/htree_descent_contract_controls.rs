//! Pure byte-level controls for a disconnected ordinary-directory oracle contract.
#[path = "support/htree_descent_contract.rs"]
mod contract;

use contract::{validate, Profile, Query, Refusal};

const LINUX: &[u8] = include_bytes!("data/htree-descent-13835-directory.bin");
const FIXTURE_SHA: &str = "fde0da6acad7a9b3afea4194ef09ddb5126d72d4d25324fac6800bc9a4cd3424";

fn profile(bs: usize, metadata_csum: bool) -> Profile {
    Profile {
        block_size: bs,
        filetype: true,
        metadata_csum,
        unsigned_hash: false,
        casefold: false,
        encrypted: false,
    }
}
fn query(name: &[u8], hash: (u32, u32), selected: u32) -> Query<'_> {
    Query {
        name,
        computed: hash,
        independent: hash,
        independent_name: name,
        effective_version: 1,
        selected,
    }
}
fn put16(data: &mut [u8], at: usize, value: u16) {
    data[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn put32(data: &mut [u8], at: usize, value: u32) {
    data[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

fn synthetic(bs: usize, csum: bool, routes: &[(u32, u32)], names: &[&[u8]]) -> Vec<u8> {
    assert_eq!(routes.len(), names.len());
    let mut data = vec![0; bs * (routes.len() + 1)];
    put32(&mut data, 0, 12);
    put16(&mut data, 4, 12);
    data[6..9].copy_from_slice(&[1, 2, b'.']);
    put32(&mut data, 12, 2);
    put16(&mut data, 16, (bs - 12) as u16);
    data[18..22].copy_from_slice(&[2, 2, b'.', b'.']);
    data[28] = 1;
    data[29] = 8;
    let capacity = (bs - 32 - if csum { 8 } else { 0 }) / 8;
    put16(&mut data, 32, capacity as u16);
    put16(&mut data, 34, routes.len() as u16);
    for (slot, &(hash, leaf)) in routes.iter().enumerate() {
        if slot > 0 {
            put32(&mut data, 32 + slot * 8, hash);
        }
        put32(&mut data, 36 + slot * 8, leaf);
    }
    for (index, name) in names.iter().enumerate() {
        let offset = (index + 1) * bs;
        put32(&mut data, offset, (100 + index) as u32);
        put16(
            &mut data,
            offset + 4,
            (bs - if csum { 12 } else { 0 }) as u16,
        );
        data[offset + 6] = name.len() as u8;
        data[offset + 7] = 1;
        data[offset + 8..offset + 8 + name.len()].copy_from_slice(name);
        if csum {
            data[offset + bs - 12..offset + bs - 4].copy_from_slice(&[0, 0, 0, 0, 12, 0, 0, 0xde]);
        }
    }
    data
}

#[test]
fn valid_linux_witness_and_bounded_continuations_are_admitted() {
    assert_eq!(fs_ext4_test_support::sha256_hex(LINUX), FIXTURE_SHA);
    let metadata = include_str!("data/htree-descent-13835-directory.json");
    assert!(metadata.contains(FIXTURE_SHA));
    let saved = include_str!("data/htree-descent-13835-queries.tsv");
    let saved_sha = "d883a719da864c62d2cc24a80e3a14678a560c7ca27c242f6fe38385ad42cb3a";
    assert_eq!(
        fs_ext4_test_support::sha256_hex(saved.as_bytes()),
        saved_sha
    );
    assert!(metadata.contains(saved_sha));
    let seed_bytes = hex::decode("0fca7f62ba785c0649a14ab6381e9270").unwrap();
    let seed: [u32; 4] = std::array::from_fn(|i| {
        u32::from_le_bytes(seed_bytes[4 * i..4 * i + 4].try_into().unwrap())
    });
    let mut spills = 0;
    let mut total = 0;
    for (index, line) in saved.lines().enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 6);
        let name = hex::decode(fields[0]).unwrap();
        let independent = (
            u32::from_str_radix(fields[1], 16).unwrap(),
            u32::from_str_radix(fields[2], 16).unwrap(),
        );
        let selected = fields[3].parse().unwrap();
        let expected_leaf: u32 = fields[4].parse().unwrap();
        let expected_inode: u32 = fields[5].parse().unwrap();
        let computed = fs_ext4::hash::name_hash(
            &name,
            fs_ext4::hash::HashVersion::from_u8(1).unwrap(),
            &seed,
        );
        let current_selected =
            fs_ext4::htree::lookup_leaf_with(&name, &LINUX[..1024], &seed, false, |logical| {
                Ok(LINUX[logical as usize * 1024..(logical as usize + 1) * 1024].to_vec())
            })
            .unwrap()
            .unwrap();
        assert_eq!(current_selected, selected, "saved initial route {index}");
        let mut q = query(&name, independent, current_selected);
        q.computed = (computed.major, computed.minor);
        let accepted = validate(LINUX, profile(1024, true), q)
            .unwrap_or_else(|e| panic!("saved query {index}: {e:?}"));
        assert_eq!(
            (accepted.candidate, accepted.actual_leaf, accepted.inode),
            (selected, expected_leaf, expected_inode)
        );
        spills += usize::from(accepted.actual_leaf != accepted.candidate);
        total += 1;
    }
    assert_eq!((total, spills), (1500, 1));
    for (name, hash, leaf, inode) in [
        (
            b"a_name_of_twenty_00121".as_slice(),
            (0xffd2c260, 0xb71eb1ce),
            49,
            337,
        ),
        ("café_863".as_bytes(), (0xffd2c260, 0xe5af7c8f), 50, 882),
    ] {
        let accepted = validate(LINUX, profile(1024, true), query(name, hash, 49)).unwrap();
        assert_eq!(
            (accepted.candidate, accepted.actual_leaf, accepted.inode),
            (49, leaf, inode)
        );
        assert_eq!(accepted.permitted_leaves, [49, 50]);
        // The preserved Linux-valid counterexample still falsifies the old
        // assertion. The live oracle and original failed run are unchanged.
        let old_found = fs_ext4::dir::parse_block(&LINUX[49 * 1024..50 * 1024], true)
            .unwrap()
            .iter()
            .any(|e| e.name == name);
        assert_eq!(old_found, leaf == 49);
    }
    let raw = validate(
        LINUX,
        profile(1024, true),
        query(b"f0\xff\x80", (766888728, 3732967549), 10),
    )
    .unwrap();
    assert_eq!((raw.actual_leaf, raw.inode), (10, 913));
    assert_eq!(raw.permitted_leaves, [10]);
    let bs = 1024;
    let csum = true;
    let bytes = synthetic(
        bs,
        csum,
        &[(0, 1), (0x101, 2), (0x101, 3), (0x200, 4)],
        &[b"a", b"b", b"c", b"d"],
    );
    let accepted = validate(&bytes, profile(bs, csum), query(b"c", (0x100, 7), 1)).unwrap();
    assert_eq!(accepted.permitted_leaves, [1, 2, 3]);
    assert_eq!(accepted.actual_leaf, 3);
    let accepted = validate(&bytes, profile(bs, csum), query(b"c", (0x180, 8), 3)).unwrap();
    assert_eq!(accepted.permitted_leaves, [3]);
    assert_eq!(
        validate(&bytes, profile(bs, csum), query(b"d", (0x200, 9), 4))
            .unwrap()
            .actual_leaf,
        4
    );
    for base in 0..3 {
        for unsigned in [false, true] {
            let mut six = bytes.clone();
            six[28] = base;
            let mut p = profile(bs, csum);
            p.unsigned_hash = unsigned;
            let mut q = query(b"c", (0x100, 7), 1);
            q.effective_version = base + if unsigned { 3 } else { 0 };
            assert!(
                validate(&six, p, q).is_ok(),
                "version={base}, unsigned={unsigned}"
            );
        }
    }
}

#[test]
fn hash_identity_routing_shape_and_membership_failures_are_refused() {
    let original = synthetic(
        1024,
        true,
        &[(0, 1), (0x101, 2), (0x200, 3)],
        &[b"a", b"b", b"c"],
    );
    let p = profile(1024, true);
    let q = query(b"b", (0x100, 7), 1);
    let mut cases: Vec<(&str, Vec<u8>, Profile, Query<'_>, Refusal)> = Vec::new();
    let mut changed = q;
    changed.computed.0 ^= 2;
    cases.push((
        "major agreement",
        original.clone(),
        p,
        changed,
        Refusal::Hash,
    ));
    let mut changed = q;
    changed.computed.1 ^= 1;
    cases.push((
        "minor agreement",
        original.clone(),
        p,
        changed,
        Refusal::Hash,
    ));
    let mut changed = q;
    changed.effective_version = 4;
    cases.push((
        "effective version",
        original.clone(),
        p,
        changed,
        Refusal::Hash,
    ));
    let mut changed = q;
    changed.computed.0 |= 1;
    changed.independent = changed.computed;
    cases.push((
        "odd query hash",
        original.clone(),
        p,
        changed,
        Refusal::Hash,
    ));
    let mut changed = q;
    changed.selected = 2;
    cases.push((
        "not the initial candidate",
        original.clone(),
        p,
        changed,
        Refusal::Selection,
    ));
    let mut changed = q;
    changed.independent_name = b"another-name";
    cases.push((
        "independent answer name binding",
        original.clone(),
        p,
        changed,
        Refusal::Hash,
    ));
    let mut changed = q;
    changed.name = b"absent";
    changed.independent_name = changed.name;
    cases.push((
        "missing query",
        original.clone(),
        p,
        changed,
        Refusal::Missing,
    ));
    let mut changed = q;
    changed.name = b"c";
    changed.independent_name = changed.name;
    cases.push((
        "unrestricted scan cannot rescue",
        original.clone(),
        p,
        changed,
        Refusal::Misrouted,
    ));
    let mut changed = p;
    changed.casefold = true;
    cases.push((
        "casefold excluded",
        original.clone(),
        changed,
        q,
        Refusal::Unsupported,
    ));
    let mut changed = p;
    changed.encrypted = true;
    cases.push((
        "encryption excluded",
        original.clone(),
        changed,
        q,
        Refusal::Unsupported,
    ));
    let mut changed = p;
    changed.filetype = false;
    cases.push((
        "filetype required",
        original.clone(),
        changed,
        q,
        Refusal::Unsupported,
    ));
    let mut changed = p;
    changed.metadata_csum = false;
    cases.push((
        "metadata checksum profile required",
        original.clone(),
        changed,
        q,
        Refusal::Unsupported,
    ));
    let mut changed = p;
    changed.block_size = 4096;
    cases.push((
        "unqualified block size",
        original.clone(),
        changed,
        q,
        Refusal::Unsupported,
    ));
    let mut bytes = original.clone();
    bytes[30] = 1;
    cases.push(("intermediate depth", bytes, p, q, Refusal::Unsupported));
    let mut bytes = original.clone();
    bytes[28] = 6;
    cases.push(("unknown version", bytes, p, q, Refusal::Unsupported));
    let mut bytes = original.clone();
    bytes[29] = 9;
    cases.push(("root info length", bytes, p, q, Refusal::Root));
    let mut bytes = original.clone();
    bytes[24] = 1;
    cases.push(("root reserved", bytes, p, q, Refusal::Root));
    let mut bytes = original.clone();
    put16(&mut bytes, 34, 124);
    cases.push(("root capacity", bytes, p, q, Refusal::Routes));
    let mut bytes = original.clone();
    put16(&mut bytes, 34, 2);
    cases.push(("unreferenced leaf", bytes, p, q, Refusal::Routes));
    let mut bytes = original.clone();
    put32(&mut bytes, 44, 1);
    cases.push(("aliased leaf", bytes, p, q, Refusal::Routes));
    let mut bytes = original.clone();
    put32(&mut bytes, 44, 4);
    cases.push(("out of range leaf", bytes, p, q, Refusal::Routes));
    let mut bytes = original.clone();
    put32(&mut bytes, 48, 0x80);
    cases.push(("decreasing boundary", bytes, p, q, Refusal::Routes));
    let mut bytes = original.clone();
    put32(&mut bytes, 40, 0);
    cases.push(("repeated even boundary", bytes, p, q, Refusal::Routes));
    let mut bytes = original.clone();
    put32(&mut bytes, 40, 0x103);
    cases.push(("wrong continuation hash", bytes, p, q, Refusal::Misrouted));
    let mut bytes = original.clone();
    put32(&mut bytes, 40, 0x102);
    cases.push(("missing continuation bit", bytes, p, q, Refusal::Misrouted));
    let mut bytes = original.clone();
    bytes[3 * 1024 + 8] = b'b';
    cases.push(("duplicate live name", bytes, p, q, Refusal::Duplicate));
    let mut bytes = original.clone();
    put16(&mut bytes, 1024 + 4, 1024);
    cases.push(("record absorbs checksum tail", bytes, p, q, Refusal::Leaf));
    let mut bytes = original.clone();
    put16(&mut bytes, 1024 + 4, 7);
    cases.push(("short record", bytes, p, q, Refusal::Leaf));
    let mut bytes = original.clone();
    bytes[1024 + 6] = 255;
    put16(&mut bytes, 1024 + 4, 12);
    cases.push(("name exceeds record", bytes, p, q, Refusal::Leaf));
    let mut bytes = original.clone();
    bytes[1024 + 7] = 0xde;
    cases.push(("live tail type", bytes, p, q, Refusal::Leaf));
    let mut bytes = original.clone();
    bytes[1024 + 8] = b'/';
    cases.push(("slash in live name", bytes, p, q, Refusal::Leaf));
    let mut bytes = original.clone();
    bytes[2 * 1024 - 5] = 0;
    cases.push(("leaf tail shape", bytes, p, q, Refusal::Leaf));
    let mut bytes = original.clone();
    bytes.pop();
    cases.push(("partial directory block", bytes, p, q, Refusal::Bounds));
    assert_eq!(cases.len(), 33);
    for (name, bytes, profile, query, expected) in cases {
        assert_eq!(validate(&bytes, profile, query), Err(expected), "{name}");
    }
}
