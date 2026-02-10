mod manager;
mod runtime;
mod watcher;

use crate::core::types::{TtCommand, TtUserId, TtUsername};
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;
use teloxide_ng::Bot;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

pub use manager::PluginManagerHandle;

#[derive(Clone)]
pub struct TgCommandContext {
    pub chat_id: i64,
    pub user_id: i64,
    pub is_admin: bool,
    pub text: String,
}

#[derive(Clone)]
pub struct TtCommandContext {
    pub user_id: TtUserId,
    pub username: TtUsername,
    pub nickname: String,
    pub is_admin: bool,
    pub text: String,
}

#[derive(Clone)]
pub struct PluginInit {
    pub config: Arc<crate::bootstrap::config::Config>,
    pub tx_tt_cmd: Sender<TtCommand>,
    pub event_bot: Option<Bot>,
    pub cancel_token: CancellationToken,
}

#[derive(Clone)]
pub struct PluginEvent {
    pub name: String,
    pub source: String,
    pub normalized: Value,
    pub raw: Value,
}

pub fn parse_command_text(text: &str) -> Option<(String, Vec<String>)> {
    let mut parts = text.split_whitespace();
    let first = parts.next()?;
    let command = first
        .trim()
        .trim_start_matches('/')
        .split('@')
        .next()?
        .to_lowercase();
    if command.is_empty() {
        return None;
    }
    let args = parts.map(std::string::ToString::to_string).collect();
    Some((command, args))
}

pub fn plugin_name_from_path(root: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?;
    let first = rel.components().next()?;
    Some(first.as_os_str().to_string_lossy().to_string())
}

#[cfg(test)]
#[path = "../../../tests/unit/app_plugins.rs"]
mod tests;
