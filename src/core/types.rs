use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LanguageCode {
    En,
    Ru,
}

impl LanguageCode {
    pub fn as_str(self) -> &'static str {
        match self {
            LanguageCode::En => "en",
            LanguageCode::Ru => "ru",
        }
    }

    pub fn from_str_or_default(value: &str, fallback: LanguageCode) -> LanguageCode {
        LanguageCode::try_from(value).unwrap_or(fallback)
    }
}

impl fmt::Display for LanguageCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<&str> for LanguageCode {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "en" => Ok(LanguageCode::En),
            "ru" => Ok(LanguageCode::Ru),
            _ => Err("unsupported language code"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotificationSetting {
    All,
    JoinOff,
    LeaveOff,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeeplinkAction {
    Subscribe,
    Unsubscribe,
}

impl fmt::Display for DeeplinkAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeeplinkAction::Subscribe => write!(f, "subscribe"),
            DeeplinkAction::Unsubscribe => write!(f, "unsubscribe"),
        }
    }
}

impl TryFrom<&str> for DeeplinkAction {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "subscribe" => Ok(DeeplinkAction::Subscribe),
            "unsubscribe" => Ok(DeeplinkAction::Unsubscribe),
            _ => Err("unsupported deeplink action"),
        }
    }
}

impl fmt::Display for NotificationSetting {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotificationSetting::All => write!(f, "all"),
            NotificationSetting::JoinOff => write!(f, "join_off"),
            NotificationSetting::LeaveOff => write!(f, "leave_off"),
            NotificationSetting::None => write!(f, "none"),
        }
    }
}

impl TryFrom<&str> for NotificationSetting {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "all" => Ok(NotificationSetting::All),
            "join_off" => Ok(NotificationSetting::JoinOff),
            "leave_off" => Ok(NotificationSetting::LeaveOff),
            "none" => Ok(NotificationSetting::None),
            _ => Err("unsupported notification setting"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MuteListMode {
    Blacklist,
    Whitelist,
}

impl fmt::Display for MuteListMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MuteListMode::Blacklist => write!(f, "blacklist"),
            MuteListMode::Whitelist => write!(f, "whitelist"),
        }
    }
}

impl TryFrom<&str> for MuteListMode {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "blacklist" => Ok(MuteListMode::Blacklist),
            "whitelist" => Ok(MuteListMode::Whitelist),
            _ => Err("unsupported mute list mode"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TtUsername(String);

impl TtUsername {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TtUsername {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for TtUsername {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for TtUsername {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for TtUsername {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminErrorContext {
    Command,
    Callback,
    Subscription,
    TtCommand,
    UpdateListener,
}

impl AdminErrorContext {
    pub fn as_str(self) -> &'static str {
        match self {
            AdminErrorContext::Command => "admin-error-context-command",
            AdminErrorContext::Callback => "admin-error-context-callback",
            AdminErrorContext::Subscription => "admin-error-context-subscription",
            AdminErrorContext::TtCommand => "admin-error-context-tt-command",
            AdminErrorContext::UpdateListener => "admin-error-context-update-listener",
        }
    }
}

#[derive(Debug)]
pub enum BridgeEvent {
    Broadcast {
        event_type: NotificationType,
        nickname: String,
        server_name: String,
        related_tt_username: String,
    },
    ToAdmin {
        user_id: i32,
        nick: String,
        tt_username: String,
        msg_content: String,
        server_name: String,
    },
    ToAdminChannel {
        channel_id: i32,
        channel_name: String,
        server_name: String,
        msg_content: String,
    },
    WhoReport {
        chat_id: i64,
        text: String,
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NotificationType {
    Join,
    Leave,
}

#[derive(Debug)]
pub enum TtCommand {
    ReplyToUser {
        user_id: i32,
        text: String,
    },
    SendToChannel {
        channel_id: i32,
        text: String,
    },
    EnqueueStream {
        channel_id: i32,
        file_path: String,
        duration_ms: u32,
        announce_text: Option<String>,
    },
    StopStreamingIf {
        stream_id: u64,
    },
    SkipStream,
    SetStreamingStatus {
        streaming: bool,
    },
    KickUser {
        user_id: i32,
    },
    BanUser {
        user_id: i32,
    },
    Who {
        chat_id: i64,
        lang: LanguageCode,
    },
    LoadAccounts,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiteUser {
    pub id: i32,
    pub nickname: String,
    pub username: String,
    pub channel_name: String,
}
