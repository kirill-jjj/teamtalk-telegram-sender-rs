use serde::Deserialize;

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
    pub default_lang: String,

    pub admin_username: Option<String>,

    #[serde(default = "default_gender")]
    pub gender: String,
}

fn default_lang() -> String {
    "en".to_string()
}

fn default_gender() -> String {
    "None".to_string()
}

fn default_deeplink_cleanup_interval_seconds() -> u64 {
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

#[derive(Deserialize, Clone)]
pub struct OperationalParameters {
    pub deeplink_ttl_seconds: i64,
    pub tt_reconnect_retry_seconds: u64,
    #[serde(default = "default_deeplink_cleanup_interval_seconds")]
    pub deeplink_cleanup_interval_seconds: u64,
    #[allow(dead_code)]
    pub tt_reconnect_check_interval_seconds: u64,
}

impl Default for OperationalParameters {
    fn default() -> Self {
        Self {
            deeplink_ttl_seconds: 300,
            tt_reconnect_retry_seconds: 10,
            deeplink_cleanup_interval_seconds: 3600,
            tt_reconnect_check_interval_seconds: 30,
        }
    }
}
