//! Strict parsing of retained ordinary-directory dx_hash batches.
//!
//! This private helper serves the live oracle and its controls. The caller checks
//! tool success and supplies the exact ordered request names/version/seed. The
//! accepted grammar is the observed raw stdout, including each command echo;
//! no lossy UTF-8 conversion or answer-count-only fallback is permitted.
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq)]
pub struct Answer {
    pub name: Vec<u8>,
    pub major: u32,
    pub minor: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Bounds,
    Profile,
    Name,
    Duplicate,
    Echo,
    Answer,
    Trailing,
}

fn hexadecimal(digits: &str) -> Result<u32, Refusal> {
    if digits.is_empty()
        || digits.len() > 8
        || !digits
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(Refusal::Answer);
    }
    u32::from_str_radix(digits, 16).map_err(|_| Refusal::Answer)
}

pub fn parse(
    stdout: &[u8],
    names: &[Vec<u8>],
    version: u8,
    seed: &[u32; 4],
) -> Result<Vec<Answer>, Refusal> {
    // The existing oracle has 1,500 names; even maximum-length quoted names
    // and both transcript lines per name fit within this explicit byte bound.
    if names.is_empty() || names.len() > 1500 || stdout.len() > 1024 * 1024 {
        return Err(Refusal::Bounds);
    }
    if version > 5 {
        return Err(Refusal::Profile);
    }
    let mut seen = HashSet::new();
    for name in names {
        if name.is_empty()
            || name.len() > 255
            || matches!(name.as_slice(), b"." | b"..")
            || name
                .iter()
                .any(|b| matches!(b, 0 | b'\n' | b'\r' | b'"' | b'\\' | b'/'))
        {
            return Err(Refusal::Name);
        }
        if !seen.insert(name.as_slice()) {
            return Err(Refusal::Duplicate);
        }
    }
    let seed_bytes: Vec<_> = seed.iter().flat_map(|w| w.to_le_bytes()).collect();
    let hex = hex::encode(seed_bytes);
    let uuid = format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    );
    let prefix = format!("debugfs: dx_hash -h {version} -s {uuid} -- \"");
    let mut lines = stdout.split(|b| *b == b'\n');
    let mut answers = Vec::with_capacity(names.len());
    for name in names {
        let mut echo = prefix.as_bytes().to_vec();
        echo.extend_from_slice(name);
        echo.push(b'"');
        if lines.next() != Some(echo.as_slice()) {
            return Err(Refusal::Echo);
        }
        let mut answer_prefix = b"Hash of ".to_vec();
        answer_prefix.extend_from_slice(name);
        answer_prefix.extend_from_slice(b" is 0x");
        let payload = lines
            .next()
            .and_then(|line| line.strip_prefix(answer_prefix.as_slice()))
            .and_then(|rest| rest.strip_suffix(b")"))
            .ok_or(Refusal::Answer)?;
        // Only the numeric suffix is text. Filenames remain raw bytes above.
        let payload = std::str::from_utf8(payload).map_err(|_| Refusal::Answer)?;
        let (major, minor) = payload.split_once(" (minor 0x").ok_or(Refusal::Answer)?;
        let major = hexadecimal(major)?;
        let minor = hexadecimal(minor)?;
        if major & 1 != 0 {
            return Err(Refusal::Answer);
        }
        answers.push(Answer {
            name: name.clone(),
            major,
            minor,
        });
    }
    // Every expected pair and exactly one final newline must be present.
    if lines.next() != Some(b"".as_slice()) || lines.next().is_some() {
        return Err(Refusal::Trailing);
    }
    Ok(answers)
}
