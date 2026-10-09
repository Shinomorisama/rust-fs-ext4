use super::*;
use saphyr::{LoadableYamlNode, Yaml};

const NORMALIZATION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/unicode/12.1.0/NormalizationTest.txt"
));

fn sequence(text: &str) -> String {
    text.split_whitespace()
        .map(|word| char::from_u32(u32::from_str_radix(word, 16).unwrap()).unwrap())
        .collect()
}

fn normalization_rows() -> impl Iterator<Item = Vec<String>> {
    NORMALIZATION.lines().filter_map(|line| {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() || line.starts_with('@') {
            None
        } else {
            Some(line.split(';').take(5).map(sequence).collect())
        }
    })
}

#[test]
fn official_nfd_conformance() {
    let mut count = 0;
    for columns in normalization_rows() {
        for (i, source) in columns.iter().enumerate() {
            let expected = &columns[if i < 3 { 2 } else { 4 }];
            assert_eq!(&nfd(source), expected, "row {count}, column {i}");
        }
        count += 1;
    }
    assert_eq!(count, 18_820);
}

#[test]
fn canonical_equivalents_have_equal_keys_across_the_official_suite() {
    let mut count = 0;
    for columns in normalization_rows() {
        let keys: Vec<_> = columns.iter().map(|s| canonical_key(s)).collect();
        assert_eq!(keys[0], keys[1], "row {count}");
        assert_eq!(keys[1], keys[2], "row {count}");
        assert_eq!(keys[3], keys[4], "row {count}");
        count += 1;
    }
    assert_eq!(count, 18_820);
}

#[test]
fn full_folding_matches_every_official_default_mapping() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/unicode/12.1.0/CaseFolding.txt"
    ));
    let mut count = 0;
    for line in source.lines() {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split(';').map(str::trim).collect();
        if matches!(fields[1], "C" | "F") {
            assert_eq!(
                full_fold(&sequence(fields[0])),
                sequence(fields[2]),
                "{}",
                fields[0]
            );
            count += 1;
        }
    }
    assert_eq!(count, data::FULL_CASE_FOLDING.len());
    assert!(count > 1400);
    assert_eq!(full_fold("Iİıẞ"), "ii\u{0307}ıss");
}

#[test]
fn canonical_order_is_stable_and_respects_starters() {
    assert_eq!(
        nfd("\u{0301}\u{0323}A\u{0315}\u{0300}"),
        "\u{0323}\u{0301}A\u{0300}\u{0315}"
    );
    assert_eq!(nfd("A\u{0301}\u{0300}"), "A\u{0301}\u{0300}");
    assert_eq!(
        nfd("A\u{0301}\u{034f}\u{0323}"),
        "A\u{0301}\u{034f}\u{0323}"
    );
    assert_eq!(nfd(""), "");
    assert_eq!(nfd("각"), "\u{1100}\u{1161}\u{11a8}");
}

#[test]
fn folding_orders_ypogegrammeni_before_it_becomes_a_starter() {
    // Unicode D145 requires NFD before folding as well as after it.
    let expected = "η\u{0301}ι";
    assert_eq!(canonical_key("ῃ\u{0301}"), expected);
    assert_eq!(canonical_key("η\u{0301}\u{0345}"), expected);
    assert_eq!(canonical_key("\u{0345}\u{0301}"), "\u{0301}ι");
}

#[test]
fn ignorables_are_removed_without_implying_name_validity() {
    assert_eq!(candidate_key("A\u{00ad}\u{200d}\u{fe0f}B"), b"ab");
    assert_eq!(candidate_key("\u{00ad}\u{fe0f}"), b"");
    assert_eq!(candidate_key("\u{00ad}."), b".");
    assert_eq!(candidate_key("\u{00ad}.."), b"..");
    assert_eq!(
        candidate_key("A\u{0301}\u{034f}\u{0323}"),
        "a\u{0323}\u{0301}".as_bytes()
    );
    // This primitive does not authorize any of these as path components.
    assert_eq!(candidate_key("/\0"), b"/\0");
}

#[test]
fn tables_and_keys_stay_at_unicode_12_1() {
    assert_eq!(data::UNICODE_VERSION, (12, 1, 0));
    assert!(!data::CANONICAL_DECOMPOSITIONS.is_empty());
    assert!(!data::COMBINING_CLASSES.is_empty());
    assert!(!data::DEFAULT_IGNORABLE_RANGES.is_empty());
    assert!(data::AGE_RANGES
        .iter()
        .any(|&(lo, hi, major, minor)| lo <= 0x32ff && hi >= 0x32ff && (major, minor) == (12, 1)));
    // This upper/lower pair was introduced after the frozen version.
    assert_eq!(candidate_key("A\u{a7d0}"), "a\u{a7d0}".as_bytes());
    assert_ne!(candidate_key("\u{a7d0}"), candidate_key("\u{a7d1}"));
    // Canonical normalization is not compatibility normalization.
    assert_ne!(candidate_key("Ａ"), candidate_key("a"));
    assert_eq!(candidate_key("Ａ"), "ａ".as_bytes());
}

#[test]
fn unknown_private_and_noncharacter_scalars_are_preserved() {
    for cp in [0x1fae0, 0x0378, 0xfdd0, 0xe000, 0xffff, 0x10ffff] {
        let c = char::from_u32(cp).unwrap();
        assert_eq!(candidate_key(&format!("A{c}")), format!("a{c}").as_bytes());
    }
}

#[test]
fn expanded_keys_are_not_truncated_to_the_raw_name_limit() {
    let name = "ΐ".repeat(80);
    let original = name.clone();
    assert_eq!(name.len(), 160);
    let key = candidate_key(&name);
    assert_eq!(key, "ι\u{0308}\u{0301}".repeat(80).as_bytes());
    assert_eq!(key.len(), 480);
    assert_eq!(name, original);
    assert_eq!(candidate_key(&"A".repeat(255)), vec![b'a'; 255]);
}

#[test]
fn captured_linux_valid_utf8_pairs_agree_with_candidate_keys() {
    const LABELS: [&str; 18] = [
        "ascii",
        "sharp_s",
        "sigma",
        "dotted_i",
        "dotless_i",
        "canonical",
        "combining_order",
        "hangul",
        "supplementary",
        "ligature",
        "width",
        "soft_hyphen",
        "joiner",
        "variation_selector",
        "post_12_1",
        "unassigned",
        "noncharacter",
        "private_use",
    ];
    let documents = Yaml::load_from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test-disks/casefold-behavior-reference.json"
    )))
    .unwrap();
    let mut count = 0;
    for mode in ["opaque", "strict"] {
        let profile = &documents[0]["profiles"][mode];
        for directory in ["fold_small", "fold_indexed"] {
            let pairs = profile["observations"][directory]["pairs"]
                .as_sequence()
                .unwrap();
            let cold = profile["cold_lookups"][directory].as_sequence().unwrap();
            for label in LABELS {
                let pair = pairs
                    .iter()
                    .find(|p| p["label"].as_str() == Some(label))
                    .unwrap();
                let after_remount = cold
                    .iter()
                    .find(|p| p["label"].as_str() == Some(label))
                    .unwrap();
                let stored = hex::decode(pair["stored_hex"].as_str().unwrap()).unwrap();
                let alias = hex::decode(pair["alias_hex"].as_str().unwrap()).unwrap();
                let same = candidate_key(std::str::from_utf8(&stored).unwrap())
                    == candidate_key(std::str::from_utf8(&alias).unwrap());
                assert_eq!(
                    same,
                    pair["same_inode_after"].as_bool().unwrap(),
                    "{mode}/{directory}/{label}"
                );
                assert_eq!(
                    same,
                    after_remount["same_inode"].as_bool().unwrap(),
                    "cold {mode}/{directory}/{label}"
                );
                count += 1;
            }
        }
    }
    assert_eq!(count, 72);
}

#[test]
fn every_scalar_candidate_key_is_idempotent() {
    for c in (0..=0x10ffff).filter_map(char::from_u32) {
        let key = candidate_key(c.encode_utf8(&mut [0; 4]));
        assert_eq!(
            candidate_key(std::str::from_utf8(&key).unwrap()),
            key,
            "U+{:04X}",
            c as u32
        );
    }
}
