//! Locate the UI text-layout caller by code structure, independent of releases.
//! The caller remains a required runtime guard against translating user text.
use aho_corasick::AhoCorasick;
use anyhow::{Result, ensure};

const PATTERN: &str = "000000c6850e02000001488d8d98000000488d95b00100004531c0e810a1f5ff9031db488b85f8010000488bb8f0010000488d87d0000000488985a8010000b10131c0f00fb08fd00000000f8548010000899df8010000488b85800100004889";
const MARKER_OFFSET: usize = 32;

pub fn runtime_signature() -> serde_json::Value {
    serde_json::json!({"bytes":PATTERN,"caller_offset":MARKER_OFFSET,"relocations":[[28,32],[77,81]]})
}

fn matches(bytes: &[u8], pattern: &[u8]) -> bool {
    bytes.len() == pattern.len()
        && bytes
            .iter()
            .zip(pattern)
            .enumerate()
            .all(|(index, (actual, expected))| {
                // CALL/Jcc relative displacements move when the linker relocates
                // functions. Preserve opcodes, stack fields and the full context.
                (28..32).contains(&index) || (77..81).contains(&index) || actual == expected
            })
}

fn locate_regions<'a>(
    regions: impl IntoIterator<Item = (usize, &'a [u8])>,
) -> Result<Option<usize>> {
    let pattern = hex::decode(PATTERN)?;
    let anchor = AhoCorasick::new([&pattern[..26]])?;
    let mut found = None;
    for (rva, bytes) in regions {
        for candidate in anchor.find_iter(bytes) {
            let start = candidate.start();
            if bytes
                .get(start..start + pattern.len())
                .is_some_and(|window| matches(window, &pattern))
            {
                ensure!(
                    found.is_none(),
                    "Ambiguous UI display structure; translation was not applied"
                );
                found = Some(rva + start + MARKER_OFFSET);
            }
        }
    }
    Ok(found)
}

pub fn detect(bytes: &[u8]) -> Result<Option<usize>> {
    let pe = goblin::pe::PE::parse(bytes)?;
    ensure!(
        pe.is_64 && pe.header.coff_header.machine == 0x8664,
        "Expected x86_64 PE image"
    );
    let mut regions = Vec::new();
    for section in &pe.sections {
        if section.characteristics & 0x2000_0000 == 0 {
            continue;
        }
        let start = section.pointer_to_raw_data as usize;
        let length = section.size_of_raw_data.min(section.virtual_size) as usize;
        let end = start
            .checked_add(length)
            .ok_or_else(|| anyhow::anyhow!("Invalid executable section"))?;
        let region = bytes
            .get(start..end)
            .ok_or_else(|| anyhow::anyhow!("Truncated executable section"))?;
        regions.push((section.virtual_address as usize, region));
    }
    locate_regions(regions)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn linker_moves_are_accepted_but_changed_code_is_rejected() {
        let mut bytes = hex::decode(PATTERN).unwrap();
        assert_eq!(
            locate_regions([(0x1000, bytes.as_slice())]).unwrap(),
            Some(0x1020)
        );
        bytes[28..32].copy_from_slice(&[1, 2, 3, 4]);
        bytes[77..81].copy_from_slice(&[5, 6, 7, 8]);
        assert_eq!(
            locate_regions([(0x3000, bytes.as_slice())]).unwrap(),
            Some(0x3020)
        );
        bytes[40] ^= 1;
        assert_eq!(locate_regions([(0, bytes.as_slice())]).unwrap(), None);
    }
    #[test]
    fn multiple_matches_and_truncated_structures_do_not_select_a_caller() {
        let bytes = hex::decode(PATTERN).unwrap();
        assert!(locate_regions([(0, bytes.as_slice()), (1000, bytes.as_slice())]).is_err());
        assert_eq!(locate_regions([(0, &bytes[..80])]).unwrap(), None);
        assert_eq!(locate_regions([(0, &[0u8; 120][..])]).unwrap(), None);
    }
}
