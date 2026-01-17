use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LanguageCode {
    En,
    Ru,
}

impl LanguageCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ru => "ru",
        }
    }

    pub fn from_str_or_default(value: &str, fallback: Self) -> Self {
        Self::try_from(value).unwrap_or(fallback)
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
            "en" => Ok(Self::En),
            "ru" => Ok(Self::Ru),
            _ => Err("unsupported language code"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
            Self::Subscribe => write!(f, "subscribe"),
            Self::Unsubscribe => write!(f, "unsubscribe"),
        }
    }
}

impl TryFrom<&str> for DeeplinkAction {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "subscribe" => Ok(Self::Subscribe),
            "unsubscribe" => Ok(Self::Unsubscribe),
            _ => Err("unsupported deeplink action"),
        }
    }
}

impl fmt::Display for NotificationSetting {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::JoinOff => write!(f, "join_off"),
            Self::LeaveOff => write!(f, "leave_off"),
            Self::None => write!(f, "none"),
        }
    }
}

impl TryFrom<&str> for NotificationSetting {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "all" => Ok(Self::All),
            "join_off" => Ok(Self::JoinOff),
            "leave_off" => Ok(Self::LeaveOff),
            "none" => Ok(Self::None),
            _ => Err("unsupported notification setting"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MuteListMode {
    Blacklist,
    Whitelist,
}

impl fmt::Display for MuteListMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Blacklist => write!(f, "blacklist"),
            Self::Whitelist => write!(f, "whitelist"),
        }
    }
}

impl TryFrom<&str> for MuteListMode {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "blacklist" => Ok(Self::Blacklist),
            "whitelist" => Ok(Self::Whitelist),
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
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Command => "admin-error-context-command",
            Self::Callback => "admin-error-context-callback",
            Self::Subscription => "admin-error-context-subscription",
            Self::TtCommand => "admin-error-context-tt-command",
            Self::UpdateListener => "admin-error-context-update-listener",
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
        reply_to: Option<i32>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NotificationType {
    Join,
    Leave,
}

#[derive(Debug)]
pub enum TtCommand {
    Shutdown,
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
        reply_to: Option<i32>,
    },
    LoadAccounts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteUser {
    pub id: i32,
    pub nickname: String,
    pub username: String,
    pub channel_name: String,
}

#[cfg(test)]
#[path = "../../tests/unit/core_types.rs"]
mod tests;
