//! Fixed identities and admission checks for the bounded test-only experiment.
use std::path::{Path, PathBuf};

pub const CORPUS_SHA: &str = "8fa9e9ecaaeebae3085ab85d2fc21f4f2946638db6af70c2b3ee0e6ee4b3e30b";
pub const REPLAY_CONFIG_SHA: &str =
    "7aabbbb9c73d4b7d400d7b74e9d69387c30c87152bb1209194f2bc4952ff19c1";

#[derive(Debug)]
pub struct Trial {
    pub id: String,
    pub seed: [u8; 16],
    pub uuid: [u8; 16],
    pub replay: Option<PathBuf>,
}

fn bytes(hex: &str) -> [u8; 16] {
    hex::decode(hex).unwrap().try_into().unwrap()
}

impl Trial {
    pub fn parse(id: &str, replay: Option<&Path>) -> Result<Self, String> {
        if id == "replay" {
            return Ok(Self {
                id: id.into(),
                seed: bytes("6d2c9f2f80bf492db620a3c18010d6c9"),
                uuid: bytes("c0ecd51e9d34484a87bed3c1ac73065d"),
                replay: Some(replay.ok_or("replay directory is required")?.to_path_buf()),
            });
        }
        if id.len() != 2 || !id.bytes().all(|b| b.is_ascii_digit()) || replay.is_some() {
            return Err("expected replay or an exact two-digit index 00..31".into());
        }
        if id.parse::<u8>().map_err(|e| e.to_string())? >= 32 {
            return Err("candidate index is outside 00..31".into());
        }
        let digest = |field| {
            let text = format!("casefold-descent-2026-11-11/{field}/{id}");
            bytes(&fs_ext4_test_support::sha256_hex(text.as_bytes())[..32])
        };
        Ok(Self {
            id: id.into(),
            seed: digest("seed"),
            uuid: digest("uuid"),
            replay: None,
        })
    }

    /// Separate admission lane: only the immutable reviewed selection is accepted.
    pub fn parse_targeted(
        id: &str,
        replay: Option<&Path>,
        manifest: &[u8],
    ) -> Result<Self, String> {
        if fs_ext4_test_support::sha256_hex(manifest)
            != "45e6d0c5b6a808e00ecc2269397bf8278b5adbbca1515194091516109cb98558"
        {
            return Err("targeted manifest identity differs".into());
        }
        if id == "replay" {
            return Self::parse(id, replay);
        }
        let text = std::str::from_utf8(manifest).map_err(|e| e.to_string())?;
        if id.len() != 5
            || !id.bytes().all(|b| b.is_ascii_digit())
            || replay.is_some()
            || !text.contains(&format!("\"{id}\""))
        {
            return Err("expected one of the sixteen frozen five-digit identities".into());
        }
        let digest = |field| {
            let input = format!("casefold-descent-targeted-2026-11-12/{field}/{id}");
            bytes(&fs_ext4_test_support::sha256_hex(input.as_bytes())[..32])
        };
        Ok(Self {
            id: id.into(),
            seed: digest("seed"),
            uuid: digest("uuid"),
            replay: None,
        })
    }

    pub fn check_settings(&self, raw: &[u8]) -> Result<(), String> {
        if raw.len() != 1024 {
            return Err("superblock readback must contain 1024 bytes".into());
        }
        if raw[0x38..0x3a] != [0x53, 0xef]
            || raw[0x18..0x1c] != [0; 4]
            || raw[0x68..0x78] != self.uuid
            || raw[0xec..0xfc] != self.seed
            || raw[0xfc] != 1
            || raw[0x160..0x164] != [1, 0, 0, 0]
        {
            return Err(format!(
                "{}: raw UUID/seed/algorithm/flags/block-size mismatch",
                self.id
            ));
        }
        Ok(())
    }
}

pub fn uuid(bytes: &[u8; 16]) -> String {
    let hex = hex::encode(bytes);
    format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    )
}

pub fn names_hex(names: &[Vec<u8>]) -> Vec<u8> {
    names
        .iter()
        .flat_map(|name| format!("{}\n", hex::encode(name)).into_bytes())
        .collect()
}

pub fn check_corpus(names: &[Vec<u8>]) -> Result<(), String> {
    if names.len() != 1500 || fs_ext4_test_support::sha256_hex(&names_hex(names)) != CORPUS_SHA {
        return Err("the original 1500-name corpus changed".into());
    }
    Ok(())
}

pub fn check_replay(directory: &Path) -> Result<(), String> {
    let read = |name: &str| std::fs::read(directory.join(name)).map_err(|e| format!("{name}: {e}"));
    if fs_ext4_test_support::sha256_hex(&read("names.hex")?) != CORPUS_SHA {
        return Err("replay corpus identity mismatch".into());
    }
    for phase in ["formatted", "configured", "indexed"] {
        let image = read(&format!("{phase}.img"))?;
        let recorded =
            String::from_utf8(read(&format!("{phase}.sha256"))?).map_err(|e| e.to_string())?;
        let digest = fs_ext4_test_support::sha256_hex(&image);
        if image.len() != 64 * 1024 * 1024
            || digest != recorded.trim()
            || (phase == "configured" && digest != REPLAY_CONFIG_SHA)
        {
            return Err(format!("replay {phase} image identity mismatch"));
        }
    }
    Ok(())
}

/// Resolve explicit test inputs without process-global environment changes in controls.
pub fn select(
    legacy: Option<&str>,
    targeted: Option<&str>,
    manifest: Option<&[u8]>,
    replay: Option<&Path>,
) -> Result<Option<Trial>, String> {
    match (legacy, targeted, manifest) {
        (None, None, None) if replay.is_none() => Ok(None),
        (Some(id), None, None) => Trial::parse(id, replay).map(Some),
        (None, Some(id), Some(manifest)) => Trial::parse_targeted(id, replay, manifest).map(Some),
        _ => Err("ambiguous or incomplete descent experiment inputs".into()),
    }
}
