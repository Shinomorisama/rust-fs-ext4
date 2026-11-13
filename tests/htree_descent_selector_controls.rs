//! Candidate-selection controls run without a filesystem tool or image.
#[path = "support/htree_descent_selector.rs"]
mod selector;
use fs_ext4::hash::NameHash;
use selector::*;
use std::time::Duration;

fn scan(values: &[(u32, u32)]) -> Scan {
    classify(
        values
            .iter()
            .map(|&(major, minor)| NameHash { major, minor })
            .collect(),
    )
    .unwrap()
}

#[test]
fn published_identities_and_little_endian_words_match() {
    for (i, seed, uuid) in [
        (
            0,
            "c65970a3cf3e35a508757024b26e0b02",
            "d9be9528c825f38d3a41859eeabff5c9",
        ),
        (
            65535,
            "a7ff3f28b71f6055c6782c98aa08b035",
            "a8210fe30e1dc2f5385e2c0b4b4538cd",
        ),
    ] {
        let (s, u) = identity(i).unwrap();
        assert_eq!(hex::encode(s), seed);
        assert_eq!(hex::encode(u), uuid);
    }
    assert_eq!(seed_words(&identity(0).unwrap().0)[0], 0xa37059c6);
    assert!(identity(SEEDS).is_err());
    assert!(identity(u32::MAX).is_err());
    assert!(check_identities(&mut [[0; 16]]).is_err());
    assert!(check_identities(&mut [[1; 16], [1; 16]]).is_err());
    check_identities(&mut [[2; 16], [1; 16]]).unwrap();
}

#[test]
fn corpus_matches_the_original_raw_byte_inventory() {
    let names = names();
    assert_eq!(names.len(), 1500);
    assert_eq!(
        fs_ext4_test_support::sha256_hex(names_hex(&names).as_bytes()),
        CORPUS_SHA
    );
    assert!(names.iter().any(|n| n.contains(&0xff)));
    let scan = analyse(&names, &identity(0).unwrap().0).unwrap();
    assert_eq!(scan.hashes.len(), 1500);
    assert!(scan.hashes.iter().all(|h| h.major & 1 == 0));
}

#[test]
fn duplicate_majors_ignore_minor_order_and_keep_all_names() {
    let scan = scan(&[(20, 7), (10, 8), (20, 2), (10, 8), (20, 9), (30, 0)]);
    assert_eq!(scan.groups, vec![vec![1, 3], vec![0, 2, 4]]);
    assert!(scan.special.is_empty());
    let names: Vec<_> = (0..6).map(|i| vec![i, 0xff]).collect();
    let record = record(0, &scan, &names, false);
    assert!(record.contains("\"name_hex\":\"03ff\",\"major\":10,\"minor\":8"));
    assert_eq!(record.matches("name_index").count(), 5);
}

#[test]
fn special_values_are_recorded_without_selecting_or_normalizing() {
    let scan = scan(&[(0xffff_fffc, 1), (2, 3), (0xffff_fffc, 4)]);
    assert!(scan.groups.is_empty());
    assert_eq!(scan.special, vec![0, 2]);
    let names = vec![vec![1], vec![2], vec![3]];
    assert_eq!(
        record(0, &scan, &names, true).matches("4294967292").count(),
        2
    );
    for major in [1, 0xffff_fffe] {
        assert!(classify(vec![NameHash { major, minor: 0 }]).is_err());
    }
}

#[test]
fn first_sixteen_candidates_follow_the_prefix_without_substitution() {
    let yes = scan(&[(2, 1), (2, 2)]);
    let no = scan(&[(2, 1), (4, 2)]);
    let mut selection = Selection::default();
    assert!(selection.consider(1, &yes).is_err());
    for i in 0..32 {
        selection
            .consider(i, if i % 2 == 0 { &no } else { &yes })
            .unwrap();
    }
    assert_eq!(
        selection.indices,
        (0..16).map(|i| i * 2 + 1).collect::<Vec<_>>()
    );
    assert!(selection.complete());
    assert!(selection.consider(32, &yes).is_err());
}

#[test]
fn finite_exhaustion_includes_empty_and_fewer_candidate_results() {
    let no = scan(&[(0xffff_fffc, 1), (0xffff_fffc, 2)]);
    let yes = scan(&[(2, 1), (2, 2)]);
    for selected in [false, true] {
        let mut selection = Selection::default();
        for i in 0..SEEDS {
            selection
                .consider(i, if selected && i == 7 { &yes } else { &no })
                .unwrap();
        }
        assert!(selection.complete());
        assert_eq!(selection.indices.len(), usize::from(selected));
        assert!(selection.consider(SEEDS, &no).is_err());
    }
}

#[test]
fn time_data_and_output_limits_refuse_without_a_completion_marker() {
    let mut budget = Budget::default();
    assert!(budget.check(WALL, 0, 0).is_err());
    assert!(budget.check(Duration::ZERO, DATA_LIMIT + 1, 0).is_err());
    assert!(budget
        .check(Duration::ZERO, DATA_LIMIT, OUTPUT_LIMIT)
        .is_ok());
    let mut out = Vec::new();
    budget.write(&mut out, b"log", Duration::ZERO).unwrap();
    assert_eq!(budget.output, 3);
    assert!(budget.check(Duration::ZERO, 0, OUTPUT_LIMIT - 2).is_err());
    assert!(publish(
        &mut out,
        &Selection::default(),
        b"manifest",
        &mut budget,
        Duration::ZERO
    )
    .is_err());
    let selection = Selection {
        scanned: SEEDS,
        indices: vec![],
    };
    assert!(publish(&mut out, &selection, b"manifest", &mut budget, WALL).is_err());
    assert_eq!(out, b"log");
    publish(
        &mut out,
        &selection,
        b"manifest",
        &mut budget,
        Duration::ZERO,
    )
    .unwrap();
    assert_eq!(out, b"logmanifest");
    // A real failing writer never consumes the output allowance.
    assert!(budget
        .write(
            &mut std::io::Cursor::new(&mut [0u8; 0][..]),
            b"x",
            Duration::ZERO
        )
        .is_err());
    assert_eq!(budget.output, 11);
}
