//! Frozen Unicode 12.1 transformations for read-only casefold lookup.
//!
//! NFD and canonical caseless matching follow Unicode 12, sections 3.11–3.13:
//! https://www.unicode.org/versions/Unicode12.0.0/ch03.pdf
//! The comparison key additionally removes default ignorables. Inputs here
//! are valid UTF-8; `LookupName` enforces component limits and refuses malformed
//! bytes and normalized dot aliases. This does not qualify filesystem writes.

// The generated file also retains version/age metadata for conformance tests.
// Age is not a validity filter: unassigned and newer scalars remain accepted.
#[allow(dead_code)]
mod data {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/unicode/12.1.0/tables.rs"
    ));
}

fn scalar(value: u32) -> char {
    // Generation and conformance checks validate every stored mapping scalar.
    char::from_u32(value).expect("the frozen Unicode table contains only scalars")
}

fn mapping(table: &'static [(u32, &'static [u32])], c: char) -> Option<&'static [u32]> {
    table
        .binary_search_by_key(&(c as u32), |entry| entry.0)
        .ok()
        .map(|index| table[index].1)
}

fn combining_class(c: char) -> u8 {
    data::COMBINING_CLASSES
        .binary_search_by_key(&(c as u32), |entry| entry.0)
        .map(|index| data::COMBINING_CLASSES[index].1)
        .unwrap_or(0)
}

fn decompose(c: char, output: &mut Vec<char>) {
    let cp = c as u32;
    if (0xac00..=0xd7a3).contains(&cp) {
        // Unicode 12 section 3.12: full canonical Hangul decomposition.
        // 19 leading x 21 vowel x 28 trailing choices, including no trailing.
        let index = cp - 0xac00;
        output.push(scalar(0x1100 + index / (21 * 28)));
        output.push(scalar(0x1161 + (index % (21 * 28)) / 28));
        if !index.is_multiple_of(28) {
            output.push(scalar(0x11a7 + index % 28));
        }
    } else if let Some(values) = mapping(data::CANONICAL_DECOMPOSITIONS, c) {
        // The verified mappings are acyclic. Compatibility mappings are absent.
        for &value in values {
            decompose(scalar(value), output);
        }
    } else {
        output.push(c);
    }
}

fn nfd(name: &str) -> String {
    let mut characters = Vec::new();
    for c in name.chars() {
        decompose(c, &mut characters);
    }
    // A class-zero starter ends the preceding run; leading nonstarters form
    // their own run. Stable sorting preserves the order of equal classes.
    let mut start = 0;
    for end in 0..=characters.len() {
        if end == characters.len() || combining_class(characters[end]) == 0 {
            characters[start..end].sort_by_key(|&c| combining_class(c));
            start = end + 1;
        }
    }
    characters.into_iter().collect()
}

fn full_fold(name: &str) -> String {
    let mut output = String::new();
    for c in name.chars() {
        if let Some(values) = mapping(data::FULL_CASE_FOLDING, c) {
            output.extend(values.iter().map(|&cp| scalar(cp)));
        } else {
            output.push(c);
        }
    }
    output
}

fn canonical_key(name: &str) -> String {
    // D145: normalization is required on both sides of full case folding.
    nfd(&full_fold(&nfd(name)))
}

fn is_default_ignorable(c: char) -> bool {
    let cp = c as u32;
    let index = data::DEFAULT_IGNORABLE_RANGES.partition_point(|&(_, last)| last < cp);
    data::DEFAULT_IGNORABLE_RANGES
        .get(index)
        .is_some_and(|&(first, _)| first <= cp)
}

pub(super) fn candidate_key(name: &str) -> Vec<u8> {
    // Removing a class-zero ignorable can join combining runs. Remove first,
    // then normalize, so a former barrier cannot leave them out of order.
    // Component validity and directory policy are enforced by the caller.
    let retained: String = name.chars().filter(|&c| !is_default_ignorable(c)).collect();
    canonical_key(&retained).into_bytes()
}

#[cfg(test)]
mod tests;
