use crate::core::types::{
    LanguageCode, TelegramId, TtChannelName, TtChannelPassword, TtClientName, TtHostName,
    TtLoginName, TtNickname, TtPassword, TtServerName, TtUsername,
};
use serde::Deserialize;
use serde_with::{NoneAsEmptyString, serde_as};
use std::path::PathBuf;
use teamtalk::types::UserGender;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub general: GeneralConfig,
    pub database: DatabaseConfig,
    pub telegram: TelegramConfig,
    pub teamtalk: TeamTalkConfig,
    #[serde(default)]
    pub plugins: PluginsConfig,

    #[serde(default)]
    pub operational_parameters: OperationalParameters,
}

#[serde_as]
#[derive(Deserialize, Clone)]
pub struct GeneralConfig {
    #[serde(default = "default_lang")]
    pub default_lang: LanguageCode,
    #[serde(default = "default_log_level")]
    pub log_level: LogLevelConfig,

    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub admin_username: Option<TtUsername>,

    #[serde(default)]
    pub gender: GenderConfig,
}

const fn default_lang() -> LanguageCode {
    LanguageCode::En
}

const fn default_log_level() -> LogLevelConfig {
    LogLevelConfig::Info
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum GenderConfig {
    Male,
    Female,
    #[default]
    #[serde(alias = "none")]
    Neutral,
}

impl GenderConfig {
    pub const fn to_user_gender(self) -> UserGender {
        match self {
            Self::Male => UserGender::Male,
            Self::Female => UserGender::Female,
            Self::Neutral => UserGender::Neutral,
        }
    }
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevelConfig {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevelConfig {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

const fn default_deeplink_cleanup_interval_seconds() -> u64 {
    3600
}

#[derive(Deserialize, Clone)]
pub struct DatabaseConfig {
    pub db_file: PathBuf,
}

#[derive(Deserialize, Clone)]
pub struct TelegramConfig {
    pub event_token: Option<String>,
    pub message_token: Option<String>,
    pub admin_chat_id: TelegramId,
}

#[derive(Deserialize, Clone)]
pub struct TeamTalkConfig {
    pub host_name: TtHostName,
    pub port: u32,
    pub encrypted: bool,
    pub user_name: TtLoginName,
    pub password: TtPassword,
    pub channel: TtChannelName,
    pub channel_password: Option<TtChannelPassword>,
    pub nick_name: TtNickname,
    #[serde(default)]
    pub status_text: String,
    pub client_name: TtClientName,
    pub server_name: Option<TtServerName>,
    #[serde(default)]
    pub global_ignore_usernames: Vec<TtUsername>,
    pub guest_username: Option<TtUsername>,
}

impl TeamTalkConfig {
    pub fn display_name(&self) -> &str {
        self.server_name
            .as_ref()
            .map(TtServerName::as_str)
            .filter(|s| !s.is_empty())
            .unwrap_or(self.host_name.as_str())
    }
}

fn default_plugins_dir() -> PathBuf {
    PathBuf::from("plugins")
}

const fn default_plugin_timeout_ms() -> u64 {
    500
}

const fn default_plugin_error_window_seconds() -> u64 {
    60
}

const fn default_plugin_error_threshold() -> u32 {
    10
}

const fn default_true() -> bool {
    true
}

#[derive(Deserialize, Clone)]
pub struct PluginsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_plugins_dir")]
    pub dir: PathBuf,
    #[serde(default = "default_true")]
    pub auto_reload: bool,
    #[serde(default = "default_plugin_timeout_ms")]
    pub call_timeout_ms: u64,
    #[serde(default = "default_plugin_error_window_seconds")]
    pub error_window_seconds: u64,
    #[serde(default = "default_plugin_error_threshold")]
    pub error_threshold: u32,
    #[serde(default)]
    pub disabled: Vec<String>,
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            dir: default_plugins_dir(),
            auto_reload: true,
            call_timeout_ms: default_plugin_timeout_ms(),
            error_window_seconds: default_plugin_error_window_seconds(),
            error_threshold: default_plugin_error_threshold(),
            disabled: Vec::new(),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/bootstrap_config.rs"]
mod tests;

#[derive(Deserialize, Clone)]
pub struct OperationalParameters {
    #[serde(rename = "deeplink_ttl_seconds")]
    pub deeplink_ttl: i64,
    #[serde(rename = "tt_reconnect_retry_seconds")]
    pub tt_reconnect_retry: u64,
    #[serde(default = "default_deeplink_cleanup_interval_seconds")]
    #[serde(rename = "deeplink_cleanup_interval_seconds")]
    pub deeplink_cleanup_interval: u64,
    #[serde(rename = "tt_reconnect_check_interval_seconds")]
    pub tt_reconnect_check_interval: u64,
}

impl Default for OperationalParameters {
    fn default() -> Self {
        Self {
            deeplink_ttl: 300,
            tt_reconnect_retry: 10,
            deeplink_cleanup_interval: 3600,
            tt_reconnect_check_interval: 30,
        }
    }
}
