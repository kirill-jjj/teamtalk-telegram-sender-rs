use crate::bootstrap::config::Config;
use anyhow::{Context, Result, anyhow};
use std::fmt::Write as _;
use std::ops::Range;
use std::path::Path;

pub fn load_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config file: {}", path.display()))?;

    toml::from_str::<Config>(&content)
        .map_err(|err| anyhow!(format_toml_config_error(path, &content, &err)))
}

fn format_toml_config_error(path: &Path, content: &str, err: &toml::de::Error) -> String {
    let mut out = format!(
        "Configuration error in {}\nReason: {}",
        path.display(),
        err.message()
    );

    if let Some(span) = err.span() {
        let (line, column) = line_col_from_offset(content, span.start);
        let _ = write!(out, "\nLocation: line {line}, column {column}");

        if let Some(snippet) = snippet_with_pointer(content, span) {
            out.push_str("\n\n");
            out.push_str(&snippet);
        }
    }

    out.push_str("\n\nHow to fix:");
    for hint in classify_hints(err.message()) {
        out.push_str("\n- ");
        out.push_str(hint);
    }

    out
}

fn line_col_from_offset(content: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    let mut byte_pos = 0usize;

    for ch in content.chars() {
        if byte_pos >= offset {
            break;
        }
        byte_pos += ch.len_utf8();
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    (line, col)
}

fn snippet_with_pointer(content: &str, span: Range<usize>) -> Option<String> {
    let (line_idx, col_idx0) = zero_based_line_col(content, span.start)?;
    let line_text = content.lines().nth(line_idx)?;
    let line_num = line_idx + 1;

    let mut pointer_len = span.len().max(1);
    if col_idx0 + pointer_len > line_text.len() {
        pointer_len = 1;
    }

    Some(format!(
        "{line_num} | {line_text}\n{}| {}{}",
        " ".repeat(line_num.to_string().len()),
        " ".repeat(col_idx0),
        "^".repeat(pointer_len)
    ))
}

fn zero_based_line_col(content: &str, offset: usize) -> Option<(usize, usize)> {
    if offset > content.len() {
        return None;
    }

    let mut line = 0usize;
    let mut line_start = 0usize;

    for (idx, ch) in content.char_indices() {
        if idx >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            line_start = idx + 1;
        }
    }

    Some((line, offset.saturating_sub(line_start)))
}

fn classify_hints(message: &str) -> &'static [&'static str] {
    let msg = message.to_ascii_lowercase();

    if msg.contains("missing field") {
        &[
            "Check that all required sections and keys are present.",
            "Compare your file with config.toml.example and add missing entries.",
            "Keep key names exact (snake_case) and place them in the correct section.",
        ]
    } else if msg.contains("invalid type") {
        &[
            "Check value types: numbers must be numeric, booleans true/false, strings in quotes.",
            "For Telegram IDs and ports, use plain integers without quotes.",
            "For paths and tokens, use quoted strings.",
        ]
    } else if msg.contains("unknown variant") {
        &[
            "Use only supported enum values from config.toml.example.",
            "Enum values are case-sensitive and usually lowercase.",
            "Remove typos or unsupported custom values.",
        ]
    } else {
        &[
            "Validate TOML syntax (quotes, commas, brackets, and section headers).",
            "Compare the file with config.toml.example and keep required sections intact.",
            "Check that Telegram IDs/ports are integers and booleans are true/false.",
        ]
    }
}

#[cfg(test)]
#[path = "../../tests/unit/bootstrap_config_errors.rs"]
mod tests;
