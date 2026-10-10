//! Read-only name policy shared by linear and indexed directory lookup.

use crate::{superblock::CasefoldEncoding, Error, Result};

pub(crate) struct LookupName<'a> {
    raw: &'a [u8],
    folded: Option<Vec<u8>>,
}

fn folded_key(name: &[u8]) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(name)
        .map_err(|_| Error::Unsupported("malformed UTF-8 in casefold lookup"))?;
    let key = super::unicode_12_1::candidate_key(text);
    if key == b"." || key == b".." {
        return Err(Error::Unsupported(
            "casefold name normalizes to a dot entry",
        ));
    }
    Ok(key)
}

impl<'a> LookupName<'a> {
    pub(crate) fn new(raw: &'a [u8], encoding: Option<CasefoldEncoding>) -> Result<Self> {
        match encoding {
            None => Ok(Self { raw, folded: None }),
            Some(CasefoldEncoding::Utf8_12_1 { .. }) => {
                if raw.len() > 255 {
                    return Err(Error::NameTooLong);
                }
                if raw.is_empty() || raw.contains(&0) || raw.contains(&b'/') {
                    return Err(Error::InvalidArgument("invalid casefold path component"));
                }
                // Only literal dots navigate; transformed names never do.
                let folded = if raw == b"." || raw == b".." {
                    None
                } else {
                    Some(folded_key(raw)?)
                };
                Ok(Self { raw, folded })
            }
        }
    }

    pub(crate) fn hash_bytes(&self) -> &[u8] {
        self.folded.as_deref().unwrap_or(self.raw)
    }

    pub(crate) fn is_folded(&self) -> bool {
        self.folded.is_some()
    }

    pub(crate) fn matches(&self, stored: &[u8]) -> Result<bool> {
        let Some(key) = &self.folded else {
            return Ok(stored == self.raw);
        };
        if stored == b"." || stored == b".." {
            return Ok(false);
        }
        if stored.is_empty() || stored.len() > 255 || stored.contains(&0) || stored.contains(&b'/')
        {
            return Err(Error::CorruptDirEntry("invalid stored casefold component"));
        }
        // No byte-equality shortcut: it could hide an unsupported encoding
        // sequence. An undecidable comparison must not become NotFound.
        Ok(folded_key(stored)? == *key)
    }

    pub(crate) fn record_match(
        &self,
        found: &mut Option<crate::dir::DirEntry>,
        entry: crate::dir::DirEntry,
    ) -> Result<()> {
        if found.replace(entry).is_some() {
            return Err(Error::CorruptDirEntry(
                "duplicate equivalent directory names",
            ));
        }
        Ok(())
    }
}
