//! The bounded experiment rejects changed inputs before asking an independent tool.
#![cfg(unix)]

#[path = "support/htree_descent_experiment.rs"]
mod experiment;

use experiment::Trial;
const TARGETED: &[u8] =
    include_bytes!("../docs/evidence/htree-descent-targeted-2026-11-12/manifest.json");
use std::collections::HashSet;
use std::path::Path;

fn raw(trial: &Trial) -> Vec<u8> {
    let mut raw = vec![0; 1024];
    raw[0x38..0x3a].copy_from_slice(&[0x53, 0xef]);
    raw[0x68..0x78].copy_from_slice(&trial.uuid);
    raw[0xec..0xfc].copy_from_slice(&trial.seed);
    raw[0xfc] = 1;
    raw[0x160] = 1;
    raw
}

#[test]
fn fixed_candidates_match_the_published_endpoints() {
    let first = Trial::parse("00", None).unwrap();
    assert_eq!(
        experiment::uuid(&first.seed),
        "ef274b78-3e6f-9950-4f60-7095197e643e"
    );
    assert_eq!(
        experiment::uuid(&first.uuid),
        "c2e9fe60-07be-94f9-504e-7f6eb52da206"
    );
    let targeted = Trial::parse_targeted("05103", None, TARGETED).unwrap();
    assert_eq!(
        hex::encode(targeted.seed),
        "28d4c66054563be79d2ce25b5dd7beaa"
    );
    assert_eq!(
        hex::encode(targeted.uuid),
        "13c79abb3cb2cebf992d16ba77590979"
    );
    let targeted = Trial::parse_targeted("35297", None, TARGETED).unwrap();
    assert_eq!(
        hex::encode(targeted.seed),
        "7778b2b1f5158df8fdd4fc43abcfcdc8"
    );
    assert_eq!(
        hex::encode(targeted.uuid),
        "025191e461c818cc330c3039a7da4ba7"
    );
    let last = Trial::parse("31", None).unwrap();
    assert_eq!(
        experiment::uuid(&last.seed),
        "e0e304ac-5c71-88d0-31cf-ad6fd8355714"
    );
    assert_eq!(
        experiment::uuid(&last.uuid),
        "61261780-8762-408e-6ade-2e77190aae8d"
    );
}

#[test]
fn all_candidates_are_distinct_nonzero_and_exclude_replay() {
    let replay = Trial::parse("replay", Some(Path::new("saved"))).unwrap();
    assert_eq!(replay.replay.as_deref(), Some(Path::new("saved")));
    let mut seeds = HashSet::new();
    let mut uuids = HashSet::new();
    for i in 0..32 {
        let trial = Trial::parse(&format!("{i:02}"), None).unwrap();
        assert_ne!(trial.seed, [0; 16]);
        assert_ne!(trial.seed, replay.seed);
        assert_ne!(trial.uuid, replay.uuid);
        assert!(seeds.insert(trial.seed));
        assert!(uuids.insert(trial.uuid));
    }
    assert_eq!((seeds.len(), uuids.len()), (32, 32));
    let text = std::str::from_utf8(TARGETED).unwrap();
    let ids: Vec<_> = text
        .split("\"candidate_ids\":[")
        .nth(1)
        .unwrap()
        .split(']')
        .next()
        .unwrap()
        .split('"')
        .filter(|s| s.len() == 5 && s.bytes().all(|b| b.is_ascii_digit()))
        .collect();
    assert_eq!(ids.len(), 16);
    let mut seeds = HashSet::new();
    let mut uuids = HashSet::new();
    for id in ids {
        let trial = Trial::parse_targeted(id, None, TARGETED).unwrap();
        assert_ne!(trial.seed, [0; 16]);
        assert_ne!(trial.uuid, [0; 16]);
        assert_ne!(trial.seed, replay.seed);
        assert_ne!(trial.uuid, replay.uuid);
        assert!(seeds.insert(trial.seed));
        assert!(uuids.insert(trial.uuid));
        assert!(Trial::parse(id, None).is_err());
        trial.check_settings(&raw(&trial)).unwrap();
    }
    assert_eq!((seeds.len(), uuids.len()), (16, 16));
}

#[test]
fn ambiguous_indices_and_wrong_replay_modes_are_refused() {
    for id in ["", "0", "000", "32", "99", "-1", "+1", " 0", "0a", "é"] {
        assert!(Trial::parse(id, None).is_err(), "{id}");
    }
    assert!(Trial::parse("replay", None).is_err());
    assert!(Trial::parse("00", Some(Path::new("saved"))).is_err());
    for id in [
        "", "5103", "051030", "05104", "65536", "00000", "00", "é", " 5103",
    ] {
        assert!(Trial::parse_targeted(id, None, TARGETED).is_err(), "{id}");
    }
    assert!(Trial::parse_targeted("05103", Some(Path::new("saved")), TARGETED).is_err());
    assert!(Trial::parse_targeted("replay", None, TARGETED).is_err());
    assert!(Trial::parse_targeted("05103", None, b"{}").is_err());
    let saved = Path::new("saved");
    assert!(Trial::parse_targeted("replay", Some(saved), TARGETED).is_ok());
    assert!(experiment::select(None, None, None, None)
        .unwrap()
        .is_none());
    assert!(experiment::select(Some("00"), None, None, None)
        .unwrap()
        .is_some());
    assert!(
        experiment::select(None, Some("05103"), Some(TARGETED), None)
            .unwrap()
            .is_some()
    );
    for (old, new, manifest, replay) in [
        (Some("00"), Some("05103"), Some(TARGETED), None),
        (Some("00"), None, Some(TARGETED), None),
        (None, Some("05103"), None, None),
        (None, None, Some(TARGETED), None),
        (None, None, None, Some(saved)),
    ] {
        assert!(experiment::select(old, new, manifest, replay).is_err());
    }
}

#[test]
fn settings_preserve_raw_seed_byte_order() {
    let trial = Trial::parse("00", None).unwrap();
    let mut bytes = raw(&trial);
    trial.check_settings(&bytes).unwrap();
    for word in bytes[0xec..0xfc].chunks_mut(4) {
        word.reverse();
    }
    assert!(trial.check_settings(&bytes).is_err());
}

#[test]
fn every_declared_setting_and_truncated_readback_is_checked() {
    let trial = Trial::parse("17", None).unwrap();
    for offset in [0x38, 0x18, 0x68, 0x77, 0xec, 0xfb, 0xfc, 0x160, 0x163] {
        let mut bytes = raw(&trial);
        bytes[offset] ^= 1;
        assert!(trial.check_settings(&bytes).is_err(), "{offset:x}");
    }
    for len in [0, 1023, 1025] {
        assert!(trial.check_settings(&vec![0; len]).is_err());
    }
}

#[test]
fn name_serialization_keeps_raw_bytes_order_and_empty_names() {
    assert_eq!(
        experiment::names_hex(&[vec![0xff, 0x80], b"A".to_vec(), vec![]]),
        b"ff80\n41\n\n"
    );
    assert!(experiment::check_corpus(&[]).is_err());
    assert!(experiment::check_corpus(&vec![b"x".to_vec(); 1500]).is_err());
}

#[test]
fn absent_replay_data_is_an_error() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tmp/seed-controls");
    let directory = fs_ext4_test_support::create_unique_temp_dir(&base).unwrap();
    assert!(experiment::check_replay(&directory).is_err());
    std::fs::write(directory.join("names.hex"), b"changed\n").unwrap();
    assert!(experiment::check_replay(&directory).is_err());
    std::fs::remove_dir_all(directory).unwrap();
}
