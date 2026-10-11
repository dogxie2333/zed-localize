use aho_corasick::AhoCorasick;
use anyhow::{Context, Result, bail, ensure};
use memmap2::Mmap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs::File, path::Path};

#[derive(Deserialize)]
pub struct Catalog {
    pub schema: u32,
    pub upstream_commit: String,
    pub source_version: String,
    pub source_commit: String,
    pub validator_fingerprint: String,
    pub dictionary_entries: usize,
    pub ignored_entries: usize,
    pub all_sources: Vec<String>,
    pub messages: Vec<Message>,
}

#[derive(Deserialize)]
pub struct Message {
    pub source: String,
    pub translation: String,
    pub origins: Vec<Value>,
    pub plain_ui: bool,
    #[serde(default)]
    pub extended_ui: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub source: String,
    pub translation: String,
    pub rva: usize,
    pub before: String,
    pub after: String,
    pub padding_bytes: usize,
    pub origins: Vec<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Section {
    pub rva: usize,
    pub size: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Plan {
    #[serde(default)]
    pub extended: bool,
    pub schema: u32,
    pub backend: String,
    pub exe: String,
    pub sha256: String,
    pub supported: bool,
    pub version: Option<String>,
    pub upstream_commit: String,
    pub source_commit: String,
    pub dictionary_entries: usize,
    pub candidate_count: usize,
    pub skip_counts: BTreeMap<String, usize>,
    pub section: Section,
    pub entries: Vec<Entry>,
}

#[derive(Deserialize)]
struct Targets {
    targets: Vec<Target>,
}
#[derive(Deserialize)]
struct Target {
    sha256: String,
    version: String,
    source_commit: String,
    architecture: String,
}

pub fn embedded_catalog() -> Result<Catalog> {
    let catalog: Catalog = serde_json::from_str(include_str!("../assets/catalog.json"))?;
    ensure!(catalog.schema == 1, "Unsupported catalogue schema");
    Ok(catalog)
}

pub fn padded_translation(source: &str, translated: &str) -> Option<Vec<u8>> {
    if source == translated
        || source.len() <= 23
        || translated.len() > source.len()
        || source
            .chars()
            .chain(translated.chars())
            .any(|c| "{}\n\r\t`\0".contains(c))
    {
        return None;
    }
    let mut bytes = translated.as_bytes().to_vec();
    bytes.resize(source.len(), b' ');
    Some(bytes)
}

pub fn hash_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    std::io::copy(&mut file, &mut digest)?;
    Ok(hex::encode(digest.finalize()))
}

pub fn check_spans(section: &Section, entries: &[Entry]) -> Result<()> {
    let mut end = section.rva;
    let section_end = section
        .rva
        .checked_add(section.size)
        .context("Section overflow")?;
    for entry in entries {
        let before = hex::decode(&entry.before)?;
        let after = hex::decode(&entry.after)?;
        ensure!(
            !before.is_empty() && before.len() == after.len(),
            "Patch length mismatch"
        );
        std::str::from_utf8(&before)?;
        std::str::from_utf8(&after)?;
        ensure!(entry.rva >= end, "Overlapping or unsorted patches");
        end = entry
            .rva
            .checked_add(before.len())
            .context("Patch overflow")?;
        ensure!(
            entry.rva >= section.rva && end <= section_end,
            "Patch outside data section"
        );
    }
    Ok(())
}

pub fn make_plan(exe: &Path) -> Result<Plan> {
    make_plan_with_scope(exe, false)
}

pub fn make_plan_with_scope(exe: &Path, extended: bool) -> Result<Plan> {
    make_plan_for_language(exe, extended, None)
}
pub fn make_plan_for_language(
    exe: &Path,
    extended: bool,
    locale: Option<(&Path, &str)>,
) -> Result<Plan> {
    let file = File::open(exe)?;
    // Windows keeps the mapped file alive; no file offsets or raw references
    // escape this function. The caller rechecks the hash before applying.
    let binary = unsafe { Mmap::map(&file)? };
    let digest = hex::encode(Sha256::digest(&binary));
    let (start, size, address, backend) = match goblin::Object::parse(&binary)? {
        goblin::Object::PE(pe) => {
            ensure!(
                pe.is_64 && pe.header.coff_header.machine == 0x8664,
                "Only Windows x64 is supported"
            );
            let data = pe
                .sections
                .iter()
                .find(|section| section.name().ok() == Some(".rdata"))
                .context("No .rdata section")?;
            (
                data.pointer_to_raw_data as usize,
                data.size_of_raw_data as usize,
                data.virtual_address as usize,
                "experimental-equal-byte-rdata",
            )
        }
        goblin::Object::Elf(elf) => {
            ensure!(
                elf.is_64 && elf.header.e_machine == goblin::elf::header::EM_X86_64,
                "Only Linux x86_64 is supported"
            );
            ensure!(
                elf.header.e_type == goblin::elf::header::ET_DYN,
                "Linux backend requires a PIE executable"
            );
            let data = elf
                .section_headers
                .iter()
                .find(|section| elf.shdr_strtab.get_at(section.sh_name) == Some(".rodata"))
                .context("No ELF .rodata section")?;
            (
                data.sh_offset as usize,
                data.sh_size as usize,
                data.sh_addr as usize,
                "experimental-linux-equal-byte-rodata",
            )
        }
        _ => bail!("Unsupported executable format"),
    };
    let raw = binary
        .get(start..start.checked_add(size).context("Section overflow")?)
        .context("Truncated data section")?;
    let section = Section { rva: address, size };
    let mut catalog = embedded_catalog()?;
    if let Some((root, language)) = locale {
        let translations = crate::localization::messages(root, language)?;
        catalog.messages.retain_mut(|message| {
            if let Some(text) = translations[&message.source].as_str() {
                message.translation = text.into();
                true
            } else {
                false
            }
        });
    }
    let targets: Targets = serde_json::from_str(include_str!("../data/approved-targets.json"))?;
    let target = targets.targets.iter().find(|target| {
        target.sha256 == digest
            && target.source_commit == catalog.source_commit
            && target.architecture == "x86_64"
    });
    let mut skipped = BTreeMap::new();
    skipped.insert("not_accepted_or_invalid".into(), catalog.ignored_entries);
    let mut candidates = Vec::new();
    for message in &catalog.messages {
        let eligible = if extended {
            message.extended_ui
        } else {
            message.plain_ui
        };
        let reason = if !eligible {
            Some("outside_plain_ui_scope")
        } else if extended
            && message
                .source
                .chars()
                .chain(message.translation.chars())
                .any(|c| "*_[]<>".contains(c))
        {
            Some("rich_text_syntax")
        } else if padded_translation(&message.source, &message.translation).is_none() {
            Some("short_formatted_or_translation_too_long")
        } else {
            None
        };
        if let Some(reason) = reason {
            *skipped.entry(reason.into()).or_default() += 1;
        } else {
            candidates.push(message);
        }
    }
    if candidates.is_empty() {
        bail!("No candidate UI sources");
    }
    // Reuse Aho-Corasick to scan the entire section once, instead of scanning
    // the ~78 MiB section separately for every community message.
    let matcher = AhoCorasick::new(candidates.iter().map(|m| &m.source))?;
    let mut substring = vec![false; candidates.len()];
    for source in &catalog.all_sources {
        for matched in matcher.find_overlapping_iter(source.as_bytes()) {
            let index = matched.pattern().as_usize();
            if candidates[index].source != *source {
                substring[index] = true;
            }
        }
    }
    let mut hits = vec![(0usize, 0usize); candidates.len()];
    for matched in matcher.find_overlapping_iter(raw) {
        let hit = &mut hits[matched.pattern().as_usize()];
        hit.0 += 1;
        if hit.0 == 1 {
            hit.1 = matched.start();
        }
    }
    let mut entries = Vec::new();
    for (index, message) in candidates.iter().enumerate() {
        let reason = if substring[index] {
            Some("substring_of_other_catalogue_message")
        } else if hits[index].0 == 0 {
            Some("not_contiguous_in_rdata")
        } else if hits[index].0 > 1 {
            Some("multiple_binary_matches")
        } else {
            None
        };
        if let Some(reason) = reason {
            *skipped.entry(reason.into()).or_default() += 1;
            continue;
        }
        let after = padded_translation(&message.source, &message.translation)
            .context("Invalid candidate")?;
        entries.push(Entry {
            source: message.source.clone(),
            translation: message.translation.clone(),
            rva: section.rva + hits[index].1,
            before: hex::encode(&message.source),
            after: hex::encode(&after),
            padding_bytes: after.len() - message.translation.len(),
            origins: message.origins.clone(),
        });
    }
    entries.sort_by_key(|e| e.rva);
    let mut overlapping = vec![false; entries.len()];
    for index in 0..entries.len() {
        let end = entries[index].rva + entries[index].source.len();
        for next in index + 1..entries.len() {
            if entries[next].rva >= end {
                break;
            }
            overlapping[index] = true;
            overlapping[next] = true;
        }
    }
    skipped.insert(
        "overlapping_binary_match".into(),
        overlapping.iter().filter(|v| **v).count(),
    );
    entries = entries
        .into_iter()
        .enumerate()
        .filter_map(|(index, entry)| (!overlapping[index]).then_some(entry))
        .collect();
    check_spans(&section, &entries)?;
    Ok(Plan {
        extended,
        schema: 1,
        backend: backend.into(),
        exe: exe.canonicalize()?.display().to_string(),
        sha256: digest,
        supported: target.is_some(),
        version: target.map(|t| t.version.clone()),
        upstream_commit: catalog.upstream_commit,
        source_commit: catalog.source_commit,
        dictionary_entries: catalog.dictionary_entries,
        candidate_count: entries.len(),
        skip_counts: skipped,
        section,
        entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reject_short_rich_and_oversized_strings() {
        assert!(padded_translation("File", "文件").is_none());
        assert!(padded_translation("Translate this `{placeholder}`", "占位符").is_none());
        assert!(
            padded_translation(
                "A sufficiently long label",
                "这是远远超出原文长度的中文翻译内容"
            )
            .is_none()
        );
    }
    #[test]
    fn preserve_utf8_and_exact_byte_length() {
        let source = "A sufficiently long label";
        let value = padded_translation(source, "长标签").expect("valid");
        assert_eq!(value.len(), source.len());
        assert_eq!(
            std::str::from_utf8(&value).expect("utf8").trim_end(),
            "长标签"
        );
    }
    #[test]
    fn reject_overlap_and_invalid_utf8() {
        let section = Section { rva: 100, size: 20 };
        let mut entries = vec![Entry {
            source: "ab".into(),
            translation: "cd".into(),
            rva: 100,
            before: "6162".into(),
            after: "6364".into(),
            padding_bytes: 0,
            origins: vec![],
        }];
        let mut second = entries[0].clone();
        second.rva = 101;
        entries.push(second);
        assert!(check_spans(&section, &entries).is_err());
        entries.truncate(1);
        entries[0].after = "e4b8".into();
        assert!(check_spans(&section, &entries).is_err());
    }
}
