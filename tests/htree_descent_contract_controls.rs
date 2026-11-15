//! Pure byte-level controls for a disconnected ordinary-directory oracle contract.
#[path = "support/htree_descent_contract.rs"]
mod contract;

#[path = "support/htree_descent_answers.rs"]
mod answers;

#[path = "support/htree_descent_verdict.rs"]
mod verdict;

use contract::{validate, validate_corpus, Profile, Query, Refusal};

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
    let raw_hashes = include_bytes!("data/htree-descent-13835-hashes.bin");
    assert_eq!(
        fs_ext4_test_support::sha256_hex(raw_hashes),
        "9e5fb6fd318f6f4a5945c8a24e6d7902833b1cd3ef532755b09279e0feed1fdc"
    );
    let expected_names: Vec<_> = saved
        .lines()
        .map(|line| hex::decode(line.split('\t').next().unwrap()).unwrap())
        .collect();
    let parsed_answers = answers::parse(raw_hashes, &expected_names, 1, &seed).unwrap();
    let mut spills = 0;
    let mut total = 0;
    let mut all_queries = Vec::new();
    let mut single_admissions = Vec::new();
    let mut observed = Vec::new();
    for (index, line) in saved.lines().enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 6);
        let name = hex::decode(fields[0]).unwrap();
        let independent = (
            u32::from_str_radix(fields[1], 16).unwrap(),
            u32::from_str_radix(fields[2], 16).unwrap(),
        );
        let answer = &parsed_answers[index];
        assert_eq!(answer.name, name);
        assert_eq!((answer.major, answer.minor), independent);
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
        assert_eq!(accepted.live_entries, 1500);
        single_admissions.push(accepted);
        all_queries.push(Query {
            name: &expected_names[index],
            computed: (computed.major, computed.minor),
            independent,
            independent_name: &answer.name,
            effective_version: 1,
            selected: current_selected,
        });
        observed.push(verdict::Observed {
            name: &expected_names[index],
            computed: (computed.major, computed.minor),
            selected: current_selected,
            found: current_selected == expected_leaf,
            actual: vec![expected_leaf],
        });
        total += 1;
    }
    assert_eq!((total, spills), (1500, 1));
    assert_eq!(
        validate_corpus(LINUX, profile(1024, true), &expected_names, &all_queries).unwrap(),
        single_admissions
    );
    let corrected = verdict::evaluate(
        LINUX,
        profile(1024, true),
        &seed,
        1,
        &expected_names,
        raw_hashes,
        &observed,
    );
    assert_eq!(corrected.as_ref().unwrap(), &single_admissions);
    assert_eq!(observed.iter().filter(|o| !o.found).count(), 1);
    let (summary, rows) = verdict::capture(&expected_names, &corrected);
    assert!(summary.contains("\"queries\":1500,\"accepted\":1500,\"continued\":1,\"passed\":true"));
    assert_eq!(rows.lines().count(), 1500);
    assert!(rows
        .lines()
        .any(|line| line.contains("\"name_hex\":\"636166c3a95f383633\"")
            && line.contains("\"candidate\":49,\"actual_leaf\":50,\"inode\":882")
            && line.contains("\"permitted_leaves\":[49, 50],\"continued\":true")));
    integrated_verdict_refusals(&expected_names, &seed, raw_hashes, &observed);
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
        // assertion. Original observations and the failed run stay unchanged.
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
    strict_hash_transcript_refusals();
    complete_corpus_pairing_refusals();
}

fn integrated_verdict_refusals(
    names: &[Vec<u8>],
    seed: &[u32; 4],
    stdout: &[u8],
    observed: &[verdict::Observed<'_>],
) {
    use verdict::Refusal as R;
    let p = profile(1024, true);
    let evaluate =
        |o: &[verdict::Observed<'_>]| verdict::evaluate(LINUX, p, seed, 1, names, stdout, o);
    let check = |label: &str, result: verdict::Verdict, expected| {
        assert_eq!(result, Err(expected), "{label}");
        let (summary, rows) = verdict::capture(names, &result);
        assert!(
            summary.contains("\"accepted\":0,\"continued\":null,\"passed\":false"),
            "{label}"
        );
        assert!(rows.is_empty(), "{label}");
    };
    let mut cases = Vec::new();
    let mut changed = observed.to_vec();
    changed[0].name = b"another-name";
    cases.push(("observed name binding", changed, R::Observations));
    let mut changed = observed.to_vec();
    changed[0].computed.0 ^= 2;
    cases.push(("computed major", changed, R::Contract(Refusal::Hash)));
    let mut changed = observed.to_vec();
    changed[0].computed.1 ^= 1;
    cases.push(("computed minor", changed, R::Contract(Refusal::Hash)));
    let mut changed = observed.to_vec();
    changed[0].selected = 0;
    cases.push((
        "initial selection",
        changed,
        R::Contract(Refusal::Selection),
    ));
    let mut changed = observed.to_vec();
    changed[0].found = !changed[0].found;
    cases.push(("original found observation", changed, R::Observations));
    let mut changed = observed.to_vec();
    changed[0].actual.clear();
    cases.push(("missing actual observation", changed, R::Observations));
    let mut changed = observed.to_vec();
    changed[0].actual.push(observed[0].actual[0]);
    cases.push(("duplicate actual observation", changed, R::Observations));
    let mut changed = observed.to_vec();
    changed[0].actual[0] = 0;
    cases.push(("wrong actual observation", changed, R::Observations));
    assert_eq!(cases.len(), 8);
    for (label, changed, expected) in cases {
        check(label, evaluate(&changed), expected);
    }
    check(
        "observation count",
        evaluate(&observed[..1499]),
        R::Observations,
    );
    let mut wrong_stdout = stdout.to_vec();
    wrong_stdout[0] = b'!';
    check(
        "raw transcript",
        verdict::evaluate(LINUX, p, seed, 1, names, &wrong_stdout, observed),
        R::Answers(answers::Refusal::Echo),
    );
    check(
        "effective version",
        verdict::evaluate(LINUX, p, seed, 4, names, stdout, observed),
        R::Answers(answers::Refusal::Echo),
    );
    let mut wrong_seed = *seed;
    wrong_seed[0] ^= 1;
    check(
        "seed",
        verdict::evaluate(LINUX, p, &wrong_seed, 1, names, stdout, observed),
        R::Answers(answers::Refusal::Echo),
    );
    let mut excluded = p;
    excluded.casefold = true;
    check(
        "profile",
        verdict::evaluate(LINUX, excluded, seed, 1, names, stdout, observed),
        R::Contract(Refusal::Unsupported),
    );
    let short_stdout: Vec<_> = stdout
        .split_inclusive(|b| *b == b'\n')
        .take(2 * 1499)
        .flatten()
        .copied()
        .collect();
    check(
        "complete corpus",
        verdict::evaluate(
            LINUX,
            p,
            seed,
            1,
            &names[..1499],
            &short_stdout,
            &observed[..1499],
        ),
        R::Contract(Refusal::Corpus),
    );
    for expected in [names, &[]] {
        let (summary, rows) = verdict::capture(expected, &Ok(Vec::new()));
        assert!(summary.contains("\"passed\":false"));
        assert!(rows.is_empty());
    }
}

fn complete_corpus_pairing_refusals() {
    let original = synthetic(
        1024,
        true,
        &[(0, 1), (0x101, 2), (0x200, 3)],
        &[b"a", b"b", b"c"],
    );
    let p = profile(1024, true);
    let names: Vec<_> = [b"a", b"b", b"c"].map(|n| n.to_vec()).to_vec();
    let queries = vec![
        query(b"a", (0x80, 1), 1),
        query(b"b", (0x100, 7), 1),
        query(b"c", (0x200, 9), 3),
    ];
    let accepted = validate_corpus(&original, p, &names, &queries).unwrap();
    assert_eq!(accepted.len(), 3);
    assert!(accepted.iter().all(|a| a.live_entries == 3));
    assert_eq!((accepted[1].candidate, accepted[1].actual_leaf), (1, 2));
    assert_eq!(
        validate_corpus(
            &original,
            p,
            &[names[2].clone(), names[0].clone(), names[1].clone()],
            &[queries[2], queries[0], queries[1]],
        )
        .unwrap(),
        [
            accepted[2].clone(),
            accepted[0].clone(),
            accepted[1].clone()
        ]
    );
    // A second record in one leaf must be counted independently of leaf count.
    let mut extra = original.clone();
    put16(&mut extra, 1024 + 4, 12);
    put32(&mut extra, 1024 + 12, 901);
    put16(&mut extra, 1024 + 16, 1000);
    extra[1024 + 18..1024 + 21].copy_from_slice(&[1, 1, b'z']);
    let mut unused = extra.clone();
    put32(&mut unused, 1024 + 12, 0);
    assert_eq!(
        validate_corpus(&unused, p, &names, &queries).unwrap(),
        accepted
    );

    type Case<'a> = (&'static str, Vec<u8>, Vec<Vec<u8>>, Vec<Query<'a>>, Refusal);
    let mut cases: Vec<Case<'_>> = vec![
        (
            "empty corpus",
            original.clone(),
            vec![],
            vec![],
            Refusal::Corpus,
        ),
        (
            "corpus bound",
            original.clone(),
            vec![b"a".to_vec(); 1501],
            vec![queries[0]; 1501],
            Refusal::Corpus,
        ),
        (
            "query count",
            original.clone(),
            names.clone(),
            queries[..2].to_vec(),
            Refusal::Corpus,
        ),
        (
            "duplicate expected name",
            original.clone(),
            vec![names[0].clone(), names[0].clone(), names[2].clone()],
            vec![queries[0], queries[0], queries[2]],
            Refusal::Corpus,
        ),
        (
            "query order binding",
            original.clone(),
            names.clone(),
            vec![queries[1], queries[0], queries[2]],
            Refusal::Corpus,
        ),
        (
            "omitted corpus name",
            original.clone(),
            names[..2].to_vec(),
            queries[..2].to_vec(),
            Refusal::Corpus,
        ),
        (
            "only continuation query",
            original.clone(),
            vec![names[1].clone()],
            vec![queries[1]],
            Refusal::Corpus,
        ),
        (
            "extra live record in same leaf",
            extra,
            names.clone(),
            queries.clone(),
            Refusal::Corpus,
        ),
        (
            "extra referenced leaf",
            synthetic(
                1024,
                true,
                &[(0, 1), (0x101, 2), (0x200, 3), (0x300, 4)],
                &[b"a", b"b", b"c", b"d"],
            ),
            names.clone(),
            queries.clone(),
            Refusal::Corpus,
        ),
    ];
    let mut changed = queries.clone();
    changed[2].name = b"d";
    cases.push((
        "query name binding",
        original.clone(),
        names.clone(),
        changed,
        Refusal::Corpus,
    ));
    let mut bytes = original.clone();
    bytes[3 * 1024 + 8] = b'd';
    cases.push((
        "same count replacement",
        bytes,
        names.clone(),
        queries.clone(),
        Refusal::Missing,
    ));
    let mut bytes = original.clone();
    put32(&mut bytes, 3 * 1024, 0);
    cases.push((
        "deleted live record",
        bytes,
        names.clone(),
        queries.clone(),
        Refusal::Corpus,
    ));
    let mut bytes = original.clone();
    bytes[3 * 1024 + 8] = b'a';
    cases.push((
        "duplicate live record",
        bytes,
        names.clone(),
        queries.clone(),
        Refusal::Duplicate,
    ));
    let mut bytes = original.clone();
    bytes[4 * 1024 - 5] = 0;
    cases.push((
        "malformed later leaf",
        bytes,
        names.clone(),
        queries.clone(),
        Refusal::Leaf,
    ));
    for (label, changed) in [
        (
            "later major agreement",
            Query {
                computed: (0x202, 9),
                ..queries[2]
            },
        ),
        (
            "later minor agreement",
            Query {
                computed: (0x200, 8),
                ..queries[2]
            },
        ),
        (
            "later independent name",
            Query {
                independent_name: b"d",
                ..queries[2]
            },
        ),
        (
            "later effective version",
            Query {
                effective_version: 4,
                ..queries[2]
            },
        ),
    ] {
        let mut all = queries.clone();
        all[2] = changed;
        cases.push((label, original.clone(), names.clone(), all, Refusal::Hash));
    }
    let mut changed = queries.clone();
    changed[2].selected = 1;
    cases.push((
        "later initial selection",
        original.clone(),
        names.clone(),
        changed,
        Refusal::Selection,
    ));
    let mut changed = queries.clone();
    changed[2].computed = (0x100, 9);
    changed[2].independent = changed[2].computed;
    changed[2].selected = 1;
    cases.push((
        "later genuine misrouting",
        original.clone(),
        names.clone(),
        changed,
        Refusal::Misrouted,
    ));
    assert_eq!(cases.len(), 20);
    for (label, bytes, names, queries, expected) in cases {
        assert_eq!(
            validate_corpus(&bytes, p, &names, &queries),
            Err(expected),
            "{label}"
        );
    }
    let mut excluded = p;
    excluded.casefold = true;
    assert_eq!(
        validate_corpus(&original, excluded, &names, &queries),
        Err(Refusal::Unsupported)
    );
}

fn strict_hash_transcript_refusals() {
    use answers::Refusal as R;
    let names = vec![b"a".to_vec(), b"b".to_vec()];
    let base = b"debugfs: dx_hash -h 1 -s 00000000-0000-0000-0000-000000000000 -- \"a\"\nHash of a is 0x100 (minor 0x7)\ndebugfs: dx_hash -h 1 -s 00000000-0000-0000-0000-000000000000 -- \"b\"\nHash of b is 0x200 (minor 0x0)\n";
    let replace = |from: &[u8], to: &[u8]| {
        let at = base.windows(from.len()).position(|s| s == from).unwrap();
        let mut raw = base.to_vec();
        raw.splice(at..at + from.len(), to.iter().copied());
        raw
    };
    let mut cases: Vec<(&str, Vec<u8>, R)> = Vec::new();
    for (label, from, to, expected) in [
        (
            "request name",
            b"-- \"a\"".as_slice(),
            b"-- \"x\"".as_slice(),
            R::Echo,
        ),
        ("answer name", b"Hash of a ", b"Hash of x ", R::Answer),
        ("request version", b"-h 1 ", b"-h 4 ", R::Echo),
        ("request seed", b"-s 00000000-", b"-s 10000000-", R::Echo),
        ("request options", b"dx_hash ", b"dx_hash -c ", R::Echo),
        ("empty major", b" is 0x100 ", b" is 0x ", R::Answer),
        (
            "major overflow",
            b" is 0x100 ",
            b" is 0x100000000 ",
            R::Answer,
        ),
        ("major nonhex", b" is 0x100 ", b" is 0x10g ", R::Answer),
        ("major uppercase", b" is 0x100 ", b" is 0x1A0 ", R::Answer),
        ("major sign", b" is 0x100 ", b" is 0x+100 ", R::Answer),
        ("odd major", b" is 0x100 ", b" is 0x101 ", R::Answer),
        ("empty minor", b"minor 0x7", b"minor 0x", R::Answer),
        (
            "minor overflow",
            b"minor 0x7",
            b"minor 0x100000000",
            R::Answer,
        ),
        ("minor nonhex", b"minor 0x7", b"minor 0xg", R::Answer),
        ("minor sign", b"minor 0x7", b"minor 0x+7", R::Answer),
        ("missing minor", b" (minor 0x7)", b")", R::Answer),
        ("missing close", b"(minor 0x7)", b"(minor 0x7", R::Answer),
        (
            "answer trailing junk",
            b"(minor 0x7)",
            b"(minor 0x7) junk",
            R::Answer,
        ),
        ("nonASCII suffix", b"minor 0x7", b"minor 0x\xff", R::Answer),
        (
            "tool error",
            b"Hash of a is 0x100 (minor 0x7)",
            b"dx_hash: error",
            R::Answer,
        ),
    ] {
        cases.push((label, replace(from, to), expected));
    }
    cases.push(("empty output", Vec::new(), R::Echo));
    cases.push((
        "missing final newline",
        base[..base.len() - 1].to_vec(),
        R::Trailing,
    ));
    cases.push((
        "extra blank line",
        [base.as_slice(), b"\n"].concat(),
        R::Trailing,
    ));
    cases.push((
        "duplicate batch",
        [base.as_slice(), base.as_slice()].concat(),
        R::Trailing,
    ));
    let lines: Vec<_> = base.split_inclusive(|b| *b == b'\n').collect();
    cases.push(("missing pair", lines[..2].concat(), R::Echo));
    cases.push((
        "reordered pairs",
        [lines[2], lines[3], lines[0], lines[1]].concat(),
        R::Echo,
    ));
    cases.push((
        "swapped answers",
        [lines[0], lines[3], lines[2], lines[1]].concat(),
        R::Answer,
    ));
    cases.push((
        "extra answer",
        [lines[0], lines[1], lines[1], lines[2], lines[3]].concat(),
        R::Echo,
    ));
    assert_eq!(cases.len(), 28);
    for (label, raw, expected) in cases {
        if label == "swapped answers" {
            // The old answer-count check accepts this mismatched transcript.
            assert_eq!(
                raw.split(|b| *b == b'\n')
                    .filter(|line| line.starts_with(b"Hash of "))
                    .count(),
                names.len()
            );
        }
        assert_eq!(
            answers::parse(&raw, &names, 1, &[0; 4]),
            Err(expected),
            "{label}"
        );
    }
    for name in [
        b"".as_slice(),
        b".",
        b"..",
        b"a\0",
        b"a\n",
        b"a\r",
        b"a\"",
        b"a\\",
        b"a/",
        &[b'a'; 256],
    ] {
        assert_eq!(
            answers::parse(base, &[name.to_vec()], 1, &[0; 4]),
            Err(R::Name),
            "name={name:?}"
        );
    }
    assert_eq!(answers::parse(base, &[], 1, &[0; 4]), Err(R::Bounds));
    assert_eq!(
        answers::parse(base, &vec![b"a".to_vec(); 1501], 1, &[0; 4]),
        Err(R::Bounds)
    );
    assert_eq!(
        answers::parse(&vec![0; 1024 * 1024 + 1], &names, 1, &[0; 4]),
        Err(R::Bounds)
    );
    assert_eq!(
        answers::parse(base, &[b"a".to_vec(), b"a".to_vec()], 1, &[0; 4]),
        Err(R::Duplicate)
    );
    assert_eq!(answers::parse(base, &names, 6, &[0; 4]), Err(R::Profile));

    // Synthetic grammar controls, not independent hashes for these names.
    for version in 0..=5 {
        for name in [b"raw\xff\x80".as_slice(), b"n is 0x0 (minor 0x0)"] {
            let mut raw = format!(
                "debugfs: dx_hash -h {version} -s 00000000-0000-0000-0000-000000000000 -- \""
            )
            .into_bytes();
            raw.extend_from_slice(name);
            raw.extend_from_slice(b"\"\nHash of ");
            raw.extend_from_slice(name);
            raw.extend_from_slice(b" is 0x0 (minor 0xffffffff)\n");
            let parsed = answers::parse(&raw, &[name.to_vec()], version, &[0; 4]).unwrap();
            assert_eq!(
                (parsed[0].name.as_slice(), parsed[0].major, parsed[0].minor),
                (name, 0, u32::MAX)
            );
        }
    }
}
