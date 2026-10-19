//! Coherent legacy fixture construction, not a production inode conversion.
use super::*;
pub(super) fn prepare(raw: &mut [u8]) -> Result<()> {
    if raw.len() < 160 {
        return Err(Error::InvalidArgument("zero-extra fixture inode size"));
    }
    let extra = u16::from_le_bytes(raw[128..130].try_into().unwrap()) as usize;
    let start = 128 + extra;
    if start > raw.len() {
        return Err(Error::InvalidArgument("zero-extra fixture extra size"));
    }
    if extra != 0
        && start + 8 <= raw.len()
        && u32::from_le_bytes(raw[start..start + 4].try_into().unwrap())
            == crate::xattr::EXT4_XATTR_MAGIC
        && raw[start + 4..start + 8] != [0; 4]
    {
        return Err(Error::InvalidArgument(
            "zero-extra fixture has inline attributes",
        ));
    }
    // Construct genuinely unused extra storage, rather than declaring it unused
    // over a former extended inode's timestamp words. Caller restamps checksum.
    raw[128..].fill(0);
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_linux_fixture_does_not_retain_hidden_timestamp_storage() {
        let mut raw = vec![0u8; 256];
        raw[0..2].copy_from_slice(&0o100640u16.to_le_bytes());
        raw[104..108].copy_from_slice(&534u32.to_le_bytes());
        raw[128..132].copy_from_slice(&0x4ec80000u32.to_le_bytes());
        // Captured from rust-1024-256-false, source 499c1b4: declaring
        // extra_isize=0 alone left both old timestamp words in the fixture.
        raw[132..136].copy_from_slice(&0x41cdb400u32.to_le_bytes());
        raw[144..148].copy_from_slice(&0x6ad583d5u32.to_le_bytes());
        raw[148..152].copy_from_slice(&0x41cdb400u32.to_le_bytes());
        let base = raw[..128].to_vec();
        prepare(&mut raw).unwrap();
        assert_eq!(raw[..128], base);
        assert!(
            raw[128..].iter().all(|&b| b == 0),
            "zero-extra fixture retained hidden extension bytes"
        );
    }
    #[test]
    fn fixture_preparation_preserves_inline_attributes_and_short_inputs_on_refusal() {
        for mut raw in [vec![0u8; 128], vec![0u8; 256]] {
            if raw.len() == 256 {
                raw[128..130].copy_from_slice(&32u16.to_le_bytes());
                crate::xattr::plan_set_in_inode_region(&mut raw[160..], "user.keep", b"preserve")
                    .unwrap();
            }
            let before = raw.clone();
            assert!(prepare(&mut raw).is_err());
            assert_eq!(raw, before);
        }
    }
}
