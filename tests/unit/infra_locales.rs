use super::*;
use crate::args;
use std::path::PathBuf;
use strum::IntoEnumIterator;

fn read_locale(lang: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("locales");
    path.push(lang);
    path.push("messages.ftl");
    std::fs::read_to_string(path).expect("read locale file")
}

fn file_contains_key(contents: &str, key: &str) -> bool {
    contents
        .lines()
        .any(|line| line.starts_with(key) && line[key.len()..].trim_start().starts_with('='))
}

fn extract_keys(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter_map(|line| {
            if line.is_empty()
                || line.starts_with('#')
                || line.starts_with(' ')
                || line.starts_with('\t')
            {
                return None;
            }
            let (key, rest) = line.split_once('=')?;
            if rest.trim().is_empty() {
                return None;
            }
            Some(key.trim().to_string())
        })
        .collect()
}

fn extract_terms(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter_map(|line| {
            if line.is_empty()
                || line.starts_with('#')
                || line.starts_with(' ')
                || line.starts_with('\t')
            {
                return None;
            }
            let (key, rest) = line.split_once('=')?;
            if !key.trim().starts_with('-') {
                return None;
            }
            if rest.trim().is_empty() {
                return None;
            }
            Some(key.trim().to_string())
        })
        .collect()
}

fn count_key_occurrences(contents: &str, key: &str) -> usize {
    contents
        .lines()
        .filter(|line| line.starts_with(key) && line[key.len()..].trim_start().starts_with('='))
        .count()
}

fn walk_src_files() -> Vec<std::path::PathBuf> {
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, out);
                } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    out.push(path);
                }
            }
        }
    }
    let mut out = Vec::new();
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("src");
    walk(&root, &mut out);
    out
}

fn read_all_src() -> String {
    let mut buf = String::new();
    for path in walk_src_files() {
        if let Ok(text) = std::fs::read_to_string(path) {
            buf.push_str(&text);
            buf.push('\n');
        }
    }
    buf
}

fn has_orphan_indented_line(contents: &str) -> bool {
    let mut last_key_seen = false;
    for line in contents.lines() {
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        let is_cont = line.starts_with(' ') || line.starts_with('\t');
        if is_cont {
            if !last_key_seen {
                return true;
            }
            continue;
        }
        let is_key = line.contains('=') && (line.contains('='));
        last_key_seen = is_key;
    }
    false
}

fn blocks_with_select(contents: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    for key in extract_keys(contents) {
        let block = block_for_key(contents, &key);
        if block.contains("{ $") && block.contains("->") {
            blocks.push(block);
        }
    }
    blocks
}

fn select_variants(block: &str) -> Vec<String> {
    block
        .lines()
        .filter_map(|line| {
            let line = line.trim_start();
            if let Some(rest) = line.strip_prefix("*[") {
                return rest.split(']').next().map(ToString::to_string);
            }
            if let Some(rest) = line.strip_prefix('[') {
                return rest.split(']').next().map(ToString::to_string);
            }
            None
        })
        .collect()
}

fn block_for_key(contents: &str, key: &str) -> String {
    let mut in_block = false;
    let mut block = Vec::new();
    for line in contents.lines() {
        if !in_block {
            if line.starts_with(key) && line[key.len()..].trim_start().starts_with('=') {
                in_block = true;
                block.push(line);
            }
            continue;
        }
        if line.is_empty() {
            break;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') {
            break;
        }
        block.push(line);
    }
    block.join("\n")
}

#[test]
fn args_macro_builds_map() {
    let map = args!(name = "Bob", count = 3).expect("args map");
    let name = map.get("name").expect("name");
    let count = map.get("count").expect("count");
    assert_eq!(name, &FluentValue::from("Bob"));
    assert_eq!(count, &FluentValue::from(3));
}

#[test]
fn args_macro_handles_unicode() {
    let map = args!(name = "Привет").expect("args map");
    let name = map.get("name").expect("name");
    assert_eq!(name, &FluentValue::from("Привет"));
}

#[test]
fn locale_files_exist() {
    let _ = read_locale("en");
    let _ = read_locale("ru");
}

#[test]
fn all_locale_keys_present_in_en() {
    let contents = read_locale("en");
    for key in LocaleKey::iter() {
        assert!(
            file_contains_key(&contents, key.as_str()),
            "missing key in en: {key:?}"
        );
    }
}

#[test]
fn all_locale_keys_present_in_ru() {
    let contents = read_locale("ru");
    for key in LocaleKey::iter() {
        assert!(
            file_contains_key(&contents, key.as_str()),
            "missing key in ru: {key:?}"
        );
    }
}

#[test]
fn tt_report_header_has_plural_variants_en() {
    let contents = read_locale("en");
    let block = block_for_key(&contents, "tt-report-header");
    assert!(block.contains("{ $count ->"));
    assert!(block.contains("[one]"));
    assert!(block.contains("*[other]"));
}

#[test]
fn tt_report_header_has_plural_variants_ru() {
    let contents = read_locale("ru");
    let block = block_for_key(&contents, "tt-report-header");
    assert!(block.contains("{ $count ->"));
    assert!(block.contains("[one]"));
    assert!(block.contains("[few]"));
    assert!(block.contains("*[many]"));
}

#[test]
fn locale_keys_unique_en() {
    let contents = read_locale("en");
    let keys = extract_keys(&contents);
    let mut uniq = keys.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(keys.len(), uniq.len(), "duplicate keys in en");
}

#[test]
fn locale_keys_unique_ru() {
    let contents = read_locale("ru");
    let keys = extract_keys(&contents);
    let mut uniq = keys.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(keys.len(), uniq.len(), "duplicate keys in ru");
}

#[test]
fn locale_key_sets_match_between_en_and_ru() {
    let mut en = extract_keys(&read_locale("en"));
    let mut ru = extract_keys(&read_locale("ru"));
    en.sort();
    ru.sort();
    assert_eq!(en, ru);
}

#[test]
fn locale_key_names_are_kebab_case() {
    for key in LocaleKey::iter() {
        let name = key.as_str();
        assert!(!name.is_empty());
        assert!(!name.starts_with('-') && !name.ends_with('-'));
        assert!(!name.contains('_'));
        for ch in name.chars() {
            assert!(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-');
        }
    }
}

#[test]
fn term_keys_unique_en() {
    let contents = read_locale("en");
    let terms = extract_terms(&contents);
    let mut uniq = terms.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(terms.len(), uniq.len(), "duplicate term keys in en");
}

#[test]
fn term_keys_unique_ru() {
    let contents = read_locale("ru");
    let terms = extract_terms(&contents);
    let mut uniq = terms.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(terms.len(), uniq.len(), "duplicate term keys in ru");
}

#[test]
fn locale_key_occurs_once_in_en() {
    let contents = read_locale("en");
    for key in LocaleKey::iter() {
        let count = count_key_occurrences(&contents, key.as_str());
        assert_eq!(
            count,
            1,
            "key {} occurs {} times in en",
            key.as_str(),
            count
        );
    }
}

#[test]
fn locale_key_occurs_once_in_ru() {
    let contents = read_locale("ru");
    for key in LocaleKey::iter() {
        let count = count_key_occurrences(&contents, key.as_str());
        assert_eq!(
            count,
            1,
            "key {} occurs {} times in ru",
            key.as_str(),
            count
        );
    }
}

#[test]
fn select_blocks_have_default_variant_en() {
    let contents = read_locale("en");
    for block in blocks_with_select(&contents) {
        assert!(block.contains("*["), "missing default variant in en block");
    }
}

#[test]
fn select_blocks_have_default_variant_ru() {
    let contents = read_locale("ru");
    for block in blocks_with_select(&contents) {
        assert!(block.contains("*["), "missing default variant in ru block");
    }
}

#[test]
fn select_blocks_have_multiple_variants_en() {
    let contents = read_locale("en");
    for block in blocks_with_select(&contents) {
        let variants = select_variants(&block);
        assert!(
            variants.len() >= 2,
            "select block has fewer than 2 variants in en"
        );
    }
}

#[test]
fn select_blocks_have_multiple_variants_ru() {
    let contents = read_locale("ru");
    for block in blocks_with_select(&contents) {
        let variants = select_variants(&block);
        assert!(
            variants.len() >= 2,
            "select block has fewer than 2 variants in ru"
        );
    }
}

#[test]
fn plural_variants_match_lang_en() {
    let contents = read_locale("en");
    for block in blocks_with_select(&contents) {
        let variants = select_variants(&block);
        assert!(variants.contains(&"one".to_string()));
        assert!(variants.contains(&"other".to_string()));
    }
}

#[test]
fn plural_variants_match_lang_ru() {
    let contents = read_locale("ru");
    for block in blocks_with_select(&contents) {
        let variants = select_variants(&block);
        assert!(variants.contains(&"one".to_string()));
        assert!(variants.contains(&"few".to_string()));
        assert!(variants.contains(&"many".to_string()));
    }
}

#[test]
fn locale_keys_have_no_double_dashes_or_spaces() {
    for key in LocaleKey::iter() {
        let name = key.as_str();
        assert!(!name.contains("--"));
        assert!(!name.contains(' '));
    }
}

#[test]
fn locale_files_have_no_tabs() {
    for lang in ["en", "ru"] {
        let contents = read_locale(lang);
        assert!(!contents.contains('\t'), "tabs found in {lang}");
    }
}

#[test]
fn locale_files_have_no_orphan_indented_lines() {
    for lang in ["en", "ru"] {
        let contents = read_locale(lang);
        assert!(
            !has_orphan_indented_line(&contents),
            "orphan indented line in {lang}"
        );
    }
}

#[test]
fn all_locale_keys_are_referenced_in_code() {
    let src = read_all_src();
    for key in LocaleKey::iter() {
        let variant = format!("LocaleKey::{key:?}");
        assert!(src.contains(&variant), "unused key in code: {key:?}");
    }
}

#[test]
fn locale_keys_have_nonempty_blocks() {
    for lang in ["en", "ru"] {
        let contents = read_locale(lang);
        for key in extract_keys(&contents) {
            let block = block_for_key(&contents, &key);
            assert!(
                !block.trim().is_empty(),
                "empty block for key {key} in {lang}"
            );
        }
    }
}

#[test]
fn get_text_requires_args_for_report_header() {
    assert!(get_text("en", LocaleKey::TtReportHeader, None).is_err());
    assert!(get_text("ru", LocaleKey::TtReportHeader, None).is_err());
}

#[test]
fn get_text_with_args_formats_report_header() {
    let args = args!(server = "Test", count = 2);
    let en = get_text("en", LocaleKey::TtReportHeader, args.as_ref()).unwrap();
    let ru = get_text("ru", LocaleKey::TtReportHeader, args.as_ref()).unwrap();
    assert!(en.contains("Test"));
    assert!(ru.contains("Test"));
}
