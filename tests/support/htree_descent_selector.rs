//! Native candidate selection only; these hashes are not independent evidence.
use fs_ext4::hash::{name_hash, HashVersion, NameHash};
use fs_ext4_test_support::sha256_hex;
use std::io::{self, Write};
use std::time::Duration;

pub const SEEDS: u32 = 65_536;
pub const TARGET: usize = 16;
pub const WALL: Duration = Duration::from_secs(300);
pub const DATA_LIMIT: usize = 16 * 1024 * 1024;
pub const OUTPUT_LIMIT: usize = 32 * 1024 * 1024;
pub const CORPUS_SHA: &str = "8fa9e9ecaaeebae3085ab85d2fc21f4f2946638db6af70c2b3ee0e6ee4b3e30b";

pub fn names() -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    for i in 0..1500u32 {
        let mut name = match i % 5 {
            0 => format!("f{i}").into_bytes(),
            1 => format!("a_name_of_twenty_{i:05}").into_bytes(),
            2 => format!("{i:05}_a_name_long_enough_to_need_two_md4_blocks").into_bytes(),
            3 => format!("caf\u{e9}_{i}").into_bytes(),
            _ => format!("n{i}").into_bytes(),
        };
        if i % 7 == 0 {
            name.push(0xFF);
            name.push(0x80 | (i % 64) as u8);
        }
        out.push(name);
    }
    out
}

pub fn names_hex(names: &[Vec<u8>]) -> String {
    names
        .iter()
        .map(|n| format!("{}\n", hex::encode(n)))
        .collect()
}

pub fn identity(index: u32) -> Result<([u8; 16], [u8; 16]), String> {
    if index >= SEEDS {
        return Err("index outside 00000..65535".into());
    }
    let digest = |field| {
        let text = format!("casefold-descent-targeted-2026-11-12/{field}/{index:05}");
        hex::decode(&sha256_hex(text.as_bytes())[..32])
            .unwrap()
            .try_into()
            .unwrap()
    };
    Ok((digest("seed"), digest("uuid")))
}

pub fn check_identities(values: &mut [[u8; 16]]) -> Result<(), String> {
    values.sort_unstable();
    if values.contains(&[0; 16]) || values.windows(2).any(|v| v[0] == v[1]) {
        return Err("zero or repeated identity".into());
    }
    Ok(())
}

pub fn seed_words(raw: &[u8; 16]) -> [u32; 4] {
    std::array::from_fn(|i| u32::from_le_bytes(raw[i * 4..i * 4 + 4].try_into().unwrap()))
}

#[derive(Debug)]
pub struct Scan {
    // Each group stores corpus indices; majors are equal, minors need not be.
    pub groups: Vec<Vec<usize>>,
    pub special: Vec<usize>,
    pub hashes: Vec<NameHash>,
}

pub fn classify(hashes: Vec<NameHash>) -> Result<Scan, String> {
    if hashes
        .iter()
        .any(|h| h.major & 1 != 0 || h.major == 0xffff_fffe)
    {
        return Err("unexpected native major; do not normalize it".into());
    }
    let mut order: Vec<_> = (0..hashes.len()).collect();
    order.sort_unstable_by_key(|&i| (hashes[i].major, i));
    let mut groups = Vec::new();
    let mut special = Vec::new();
    let mut offset = 0;
    while offset < order.len() {
        let major = hashes[order[offset]].major;
        let end = offset + order[offset..].partition_point(|&i| hashes[i].major == major);
        if major == 0xffff_fffc {
            special.extend_from_slice(&order[offset..end]);
        } else if end - offset > 1 {
            groups.push(order[offset..end].to_vec());
        }
        offset = end;
    }
    Ok(Scan {
        groups,
        special,
        hashes,
    })
}

pub fn analyse(names: &[Vec<u8>], seed: &[u8; 16]) -> Result<Scan, String> {
    let seed = seed_words(seed);
    classify(
        names
            .iter()
            .map(|n| name_hash(n, HashVersion::HalfMd4, &seed))
            .collect(),
    )
}

#[derive(Default, Debug)]
pub struct Selection {
    pub scanned: u32,
    pub indices: Vec<u32>,
}

impl Selection {
    pub fn consider(&mut self, index: u32, scan: &Scan) -> Result<(), String> {
        if self.complete() || index != self.scanned || index >= SEEDS {
            return Err("selection must use the contiguous ascending prefix exactly once".into());
        }
        if !scan.groups.is_empty() {
            self.indices.push(index);
        }
        self.scanned += 1;
        Ok(())
    }

    pub fn complete(&self) -> bool {
        self.indices.len() == TARGET || self.scanned == SEEDS
    }
}

#[derive(Default)]
pub struct Budget {
    pub output: usize,
}

impl Budget {
    pub fn check(&self, elapsed: Duration, live_data: usize, extra: usize) -> io::Result<()> {
        if elapsed >= WALL
            || live_data > DATA_LIMIT
            || extra > OUTPUT_LIMIT.saturating_sub(self.output)
        {
            return Err(io::Error::other(
                "incomplete: time/data/output limit reached",
            ));
        }
        Ok(())
    }

    pub fn write(
        &mut self,
        out: &mut impl Write,
        bytes: &[u8],
        elapsed: Duration,
    ) -> io::Result<()> {
        self.check(elapsed, 0, bytes.len())?;
        out.write_all(bytes)?;
        self.output += bytes.len();
        Ok(())
    }
}

pub fn record(index: u32, scan: &Scan, names: &[Vec<u8>], special: bool) -> String {
    let (seed, uuid) = identity(index).unwrap();
    let groups = if special {
        vec![scan.special.clone()]
    } else {
        scan.groups.clone()
    };
    let groups: Vec<_> =
        groups
            .iter()
            .map(|group| {
                let members: Vec<_> =
                    group
                        .iter()
                        .map(|&i| {
                            format!(
            "{{\"name_index\":{i},\"name_hex\":\"{}\",\"major\":{},\"minor\":{}}}",
            hex::encode(&names[i]), scan.hashes[i].major, scan.hashes[i].minor)
                        })
                        .collect();
                format!("[{}]", members.join(","))
            })
            .collect();
    format!("{{\"index\":{index},\"id\":\"{index:05}\",\"seed_hex\":\"{}\",\"uuid_hex\":\"{}\",\"groups\":[{}]}}\n",
        hex::encode(seed), hex::encode(uuid), groups.join(","))
}

// The completion marker is written last. Partial logs never imply completion.
pub fn publish(
    out: &mut impl Write,
    selection: &Selection,
    bytes: &[u8],
    budget: &mut Budget,
    elapsed: Duration,
) -> io::Result<()> {
    if !selection.complete() {
        return Err(io::Error::other(
            "incomplete: selection prefix is unfinished",
        ));
    }
    budget.write(out, bytes, elapsed)
}
