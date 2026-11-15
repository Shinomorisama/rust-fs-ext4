//! Shared ordinary-directory verdict and separate corrected-contract capture.
use super::{answers, contract};
use std::fmt::Write as _;

#[derive(Clone, Debug)]
pub struct Observed<'a> {
    pub name: &'a [u8],
    pub computed: (u32, u32),
    pub selected: u32,
    pub found: bool,
    pub actual: Vec<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Observations,
    Answers(answers::Refusal),
    Contract(contract::Refusal),
}

pub type Verdict = Result<Vec<contract::Admission>, Refusal>;

pub fn evaluate(
    bytes: &[u8],
    profile: contract::Profile,
    seed: &[u32; 4],
    version: u8,
    names: &[Vec<u8>],
    stdout: &[u8],
    observed: &[Observed<'_>],
) -> Verdict {
    if observed.len() != names.len() || observed.iter().zip(names).any(|(o, n)| o.name != n) {
        return Err(Refusal::Observations);
    }
    let independent = answers::parse(stdout, names, version, seed).map_err(Refusal::Answers)?;
    let queries: Vec<_> = observed
        .iter()
        .zip(&independent)
        .map(|(o, a)| contract::Query {
            name: o.name,
            computed: o.computed,
            independent: (a.major, a.minor),
            independent_name: &a.name,
            effective_version: version,
            selected: o.selected,
        })
        .collect();
    let accepted =
        contract::validate_corpus(bytes, profile, names, &queries).map_err(Refusal::Contract)?;
    for (o, a) in observed.iter().zip(&accepted) {
        if o.actual != [a.actual_leaf] || o.found != (a.actual_leaf == a.candidate) {
            return Err(Refusal::Observations);
        }
    }
    Ok(accepted)
}

/// These files describe the corrected verdict only. Original selected/found/
/// actual observations and result.json.wrong keep their first-leaf meanings.
pub fn capture(names: &[Vec<u8>], verdict: &Verdict) -> (String, String) {
    let accepted = match verdict {
        Ok(accepted)
            if !names.is_empty() && names.len() <= 1500 && accepted.len() == names.len() =>
        {
            accepted
        }
        other => {
            let refusal = match other {
                Err(error) => *error,
                Ok(_) => Refusal::Observations,
            };
            return (
                format!(
                    "{{\"schema\":1,\"contract\":\"ordinary-root-collision-v1\",\"queries\":{},\"accepted\":0,\"continued\":null,\"passed\":false,\"refusal\":\"{refusal:?}\"}}\n",
                    names.len()
                ),
                String::new(),
            );
        }
    };
    let continued = accepted
        .iter()
        .filter(|a| a.actual_leaf != a.candidate)
        .count();
    let mut rows = String::new();
    for (name, a) in names.iter().zip(accepted) {
        writeln!(rows, "{{\"name_hex\":\"{}\",\"candidate\":{},\"actual_leaf\":{},\"inode\":{},\"permitted_leaves\":{:?},\"continued\":{},\"live_entries\":{}}}",
            hex::encode(name), a.candidate, a.actual_leaf, a.inode, a.permitted_leaves,
            a.actual_leaf != a.candidate, a.live_entries).unwrap();
    }
    (
        format!(
            "{{\"schema\":1,\"contract\":\"ordinary-root-collision-v1\",\"queries\":{},\"accepted\":{},\"continued\":{continued},\"passed\":true,\"refusal\":null}}\n",
            names.len(), accepted.len()
        ),
        rows,
    )
}
