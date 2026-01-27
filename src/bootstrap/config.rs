use crate::core::types::LanguageCode;
use serde::Deserialize;
use teamtalk::types::UserGender;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub general: GeneralConfig,
    pub database: DatabaseConfig,
    pub telegram: TelegramConfig,
    pub teamtalk: TeamTalkConfig,

    #[serde(default)]
    pub operational_parameters: OperationalParameters,
}

#[derive(Deserialize, Clone)]
pub struct GeneralConfig {
    #[serde(default = "default_lang")]
    pub default_lang: LanguageCode,
    #[serde(default = "default_log_level")]
    pub log_level: LogLevelConfig,

    pub admin_username: Option<String>,

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
    pub db_file: String,
}

#[derive(Deserialize, Clone)]
pub struct TelegramConfig {
    pub event_token: Option<String>,
    pub message_token: Option<String>,
    pub admin_chat_id: i64,
}

#[derive(Deserialize, Clone)]
pub struct TeamTalkConfig {
    pub host_name: String,
    pub port: u32,
    pub encrypted: bool,
    pub user_name: String,
    pub password: String,
    pub channel: String,
    pub channel_password: Option<String>,
    pub nick_name: String,
    #[serde(default)]
    pub status_text: String,
    pub client_name: String,
    pub server_name: Option<String>,
    #[serde(default)]
    pub global_ignore_usernames: Vec<String>,
    pub guest_username: Option<String>,
}

impl TeamTalkConfig {
    pub fn display_name(&self) -> &str {
        self.server_name
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(&self.host_name)
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
