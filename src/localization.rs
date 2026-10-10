//! Offline, community-provided display catalogs. No network on the display path.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{fs, path::Path};

pub const LOCALES: &[&str] = &[
    "en", "zh-CN", "zh-TW", "ja-JP", "ko-KR", "de-DE", "fr-FR", "es-ES", "it-IT", "pt-BR", "ru-RU",
    "pl-PL", "cs-CZ", "tr-TR",
];

pub fn supported(locale: &str) -> bool {
    LOCALES.contains(&locale)
}

pub fn selected(state: &Path) -> String {
    fs::read(state.join("settings.json"))
        .ok()
        .and_then(|bytes| {
            serde_json::from_slice::<Value>(
                bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes),
            )
            .ok()
        })
        .and_then(|value| value["translation_language"].as_str().map(str::to_owned))
        .filter(|locale| supported(locale))
        .unwrap_or_else(|| "zh-CN".into())
}

pub fn messages(root: &Path, locale: &str) -> Result<Value> {
    ensure!(supported(locale), "Unsupported translation language");
    if locale == "en" {
        return Ok(json!({}));
    }
    let value: Value = serde_json::from_slice(&fs::read(
        root.join(format!("assets/locales/{locale}.json")),
    )?)?;
    ensure!(
        value["schema"] == 1 && value["locale"] == locale,
        "Invalid language catalog"
    );
    let messages = value["messages"]
        .as_object()
        .context("Missing language messages")?;
    ensure!(
        messages.iter().all(
            |(source, text)| text.as_str().is_some_and(|text| !text.is_empty()
                && source != text
                && text.encode_utf16().count() <= source.encode_utf16().count()
                && !source
                    .chars()
                    .chain(text.chars())
                    .any(|c| "{}\n\r\t`*_[]<>\0".contains(c)))
        ),
        "Unsafe display text"
    );
    Ok(value["messages"].clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalogs_are_safe_and_language_selection_persists() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        for locale in LOCALES {
            let catalog = messages(root, locale).unwrap();
            assert!(locale == &"en" || !catalog.as_object().unwrap().is_empty());
        }
        assert!(!supported("../zh-CN"));
        let state = tempfile::tempdir().unwrap();
        assert_eq!(selected(state.path()), "zh-CN");
        fs::write(
            state.path().join("settings.json"),
            r#"{"translation_language":"ja-JP"}"#,
        )
        .unwrap();
        assert_eq!(selected(state.path()), "ja-JP");
    }
}
