//! Differential test of the htree name hash against `debugfs dx_hash`.
//!
//! `htree_hash_vectors.rs` pins a hand-picked table. This file asks
//! `debugfs` about a few thousand pseudo-random names instead — every
//! length from 0 to 255, the block boundaries of both block algorithms
//! (16 bytes for TEA, 32 for half MD4) and their neighbours, bytes >= 0x80
//! (where the signed and unsigned versions part ways), several random
//! seeds plus the all-zero seed — and requires major and minor to agree
//! for all six hash versions. The requests go to one `debugfs -f -`
//! script per batch of a few hundred, so the run costs a few dozen tool
//! calls rather than one per hash.
//!
//! debugfs reads each request with its own command-line parser, so a name
//! is sent inside double quotes and may not contain `"`, `\`, NUL, CR or
//! LF (the parser cannot carry those through). The name follows `--`,
//! since a quoted name starting with `-` is still read as an option. `/` is left out too: no
//! directory entry can contain it.

use fs_ext4::hash::{name_hash, HashVersion};

/// A small deterministic generator (SplitMix64), so every run asks the
/// same questions and a failure can be reproduced.
struct SplitMix(u64);

impl SplitMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// True for a byte debugfs can take inside a quoted argument.
fn sendable(byte: u8) -> bool {
    !matches!(byte, 0 | b'\n' | b'\r' | b'"' | b'\\' | b'/')
}

fn random_name(rng: &mut SplitMix, len: usize) -> Vec<u8> {
    // Half the names are drawn from printable ASCII, half from the whole
    // byte range, so both plain and high-bit names are well represented.
    let high = rng.below(2) == 0;
    let mut name = Vec::with_capacity(len);
    while name.len() < len {
        let byte = if high {
            rng.below(256) as u8
        } else {
            0x20 + rng.below(0x5F) as u8
        };
        if sendable(byte) {
            name.push(byte);
        }
    }
    name
}

/// The seed as debugfs's `-s` takes it: the 16 superblock bytes as a UUID,
/// which are the four seed words stored little-endian.
fn seed_uuid(seed: &[u32; 4]) -> String {
    let bytes: Vec<u8> = seed.iter().flat_map(|w| w.to_le_bytes()).collect();
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

struct Case {
    seed: [u32; 4],
    version: u8,
    name: Vec<u8>,
}

/// Parse `Hash of <name> is 0x<major> (minor 0x<minor>)`.
fn parse_answer(line: &[u8]) -> Option<(u32, u32)> {
    let text = String::from_utf8_lossy(line);
    let at = text.rfind(" is 0x")?;
    let rest = &text[at + " is 0x".len()..];
    let (major, rest) = rest.split_once(" (minor 0x")?;
    let minor = rest.strip_suffix(')')?;
    Some((
        u32::from_str_radix(major, 16).ok()?,
        u32::from_str_radix(minor, 16).ok()?,
    ))
}

fn ask_debugfs(cases: &[Case]) -> Vec<(u32, u32)> {
    let mut script = Vec::new();
    for case in cases {
        script.extend_from_slice(
            format!(
                "dx_hash -h {} -s {} -- \"",
                case.version,
                seed_uuid(&case.seed)
            )
            .as_bytes(),
        );
        script.extend_from_slice(&case.name);
        script.extend_from_slice(b"\"\n");
    }
    let out = fs_ext4_test_support::oracle("debugfs")
        .arg("-f")
        .arg("-")
        .stdin(script)
        .output();
    let answers: Vec<(u32, u32)> = out
        .stdout
        .split(|&b| b == b'\n')
        .filter(|line| line.starts_with(b"Hash of "))
        .map(|line| {
            parse_answer(line).unwrap_or_else(|| {
                panic!(
                    "unreadable debugfs answer: {:?}",
                    String::from_utf8_lossy(line)
                )
            })
        })
        .collect();
    assert_eq!(
        answers.len(),
        cases.len(),
        "debugfs answered {} of {} requests; stderr:\n{}",
        answers.len(),
        cases.len(),
        String::from_utf8_lossy(&out.stderr)
    );
    answers
}

/// Requests per `debugfs` call. The script travels to the tool inside a
/// single shell argument, which Linux caps at 128 KiB; 400 requests of at
/// most ~320 bytes each stay under that once base64-encoded.
const BATCH: usize = 400;

#[test]
fn random_names_agree_with_debugfs() {
    let mut rng = SplitMix(0x6874_7265_6568_6173);

    let mut seeds = vec![[0u32; 4]];
    for _ in 0..5 {
        seeds.push([
            rng.next() as u32,
            rng.next() as u32,
            rng.next() as u32,
            rng.next() as u32,
        ]);
    }

    // Every length once, the block boundaries several times over, and
    // then random lengths up to the 255-byte name limit.
    let boundaries = [
        0usize, 1, 3, 4, 5, 15, 16, 17, 31, 32, 33, 63, 64, 65, 254, 255,
    ];
    let mut lengths: Vec<usize> = (0..=255).collect();
    for _ in 0..8 {
        lengths.extend_from_slice(&boundaries);
    }
    while lengths.len() < 2400 {
        lengths.push(rng.below(256) as usize);
    }

    let mut cases = Vec::new();
    for (i, &len) in lengths.iter().enumerate() {
        let name = random_name(&mut rng, len);
        let seed = seeds[i % seeds.len()];
        for version in 0..=5 {
            cases.push(Case {
                seed,
                version,
                name: name.clone(),
            });
        }
    }

    let mut wrong = Vec::new();
    for batch in cases.chunks(BATCH) {
        let answers = ask_debugfs(batch);
        for (case, &(major, minor)) in batch.iter().zip(&answers) {
            let version = HashVersion::from_u8(case.version).unwrap();
            let got = name_hash(&case.name, version, &case.seed);
            if (got.major, got.minor) != (major, minor) {
                wrong.push(format!(
                    "v{} seed {} len {} {:02x?}: got ({:#010x}, {:#010x}) debugfs ({major:#010x}, {minor:#010x})",
                    case.version,
                    seed_uuid(&case.seed),
                    case.name.len(),
                    case.name,
                    got.major,
                    got.minor
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{} of {} hashes differ from debugfs (first 20):\n{}",
        wrong.len(),
        cases.len(),
        wrong[..wrong.len().min(20)].join("\n")
    );
    println!(
        "{} names, {} hashes checked against debugfs",
        lengths.len(),
        cases.len()
    );
}

/// A major hash whose value before the low bit is cleared is 0xFFFFFFFF.
/// TEA on the empty name reports the seed's first word, so this is
/// reachable directly; debugfs leaves the result at 0xFFFFFFFE.
#[test]
fn top_of_range_major_matches_debugfs() {
    let seed = [0xFFFF_FFFF, 0x1234_5678, 0, 0];
    let cases: Vec<Case> = (0..=5)
        .map(|version| Case {
            seed,
            version,
            name: Vec::new(),
        })
        .collect();
    let answers = ask_debugfs(&cases);
    for (case, &(major, minor)) in cases.iter().zip(&answers) {
        let got = name_hash(b"", HashVersion::from_u8(case.version).unwrap(), &seed);
        assert_eq!((got.major, got.minor), (major, minor), "v{}", case.version);
    }
    let tea = name_hash(b"", HashVersion::Tea, &seed);
    assert_eq!(tea.major, 0xFFFF_FFFE);
}
