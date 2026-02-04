use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum LanguageCode {
    #[serde(alias = "EN", alias = "En")]
    En,
    #[serde(alias = "RU", alias = "Ru")]
    Ru,
}

impl LanguageCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ru => "ru",
        }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum NotificationSetting {
    All,
    JoinOff,
    LeaveOff,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum MuteListMode {
    Blacklist,
    Whitelist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct TelegramId(i64);

impl TelegramId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_i64(self) -> i64 {
        self.0
    }
}

impl From<i64> for TelegramId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<TelegramId> for i64 {
    fn from(value: TelegramId) -> Self {
        value.0
    }
}

impl fmt::Display for TelegramId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TgChatId(i64);

impl TgChatId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_i64(self) -> i64 {
        self.0
    }
}

impl From<i64> for TgChatId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<TgChatId> for i64 {
    fn from(value: TgChatId) -> Self {
        value.0
    }
}

impl fmt::Display for TgChatId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TgMessageId(i32);

impl TgMessageId {
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    pub const fn as_i32(self) -> i32 {
        self.0
    }
}

impl From<i32> for TgMessageId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<TgMessageId> for i32 {
    fn from(value: TgMessageId) -> Self {
        value.0
    }
}

impl fmt::Display for TgMessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TtUserId(i32);

impl TtUserId {
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    pub const fn as_i32(self) -> i32 {
        self.0
    }
}

impl From<i32> for TtUserId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<TtUserId> for i32 {
    fn from(value: TtUserId) -> Self {
        value.0
    }
}

impl fmt::Display for TtUserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TtChannelId(i32);

impl TtChannelId {
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    pub const fn as_i32(self) -> i32 {
        self.0
    }
}

impl From<i32> for TtChannelId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<TtChannelId> for i32 {
    fn from(value: TtChannelId) -> Self {
        value.0
    }
}

impl fmt::Display for TtChannelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TtMessageId(i64);

impl TtMessageId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_i64(self) -> i64 {
        self.0
    }
}

impl From<i64> for TtMessageId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<TtMessageId> for i64 {
    fn from(value: TtMessageId) -> Self {
        value.0
    }
}

impl fmt::Display for TtMessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct DbBanId(i64);

impl DbBanId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_i64(self) -> i64 {
        self.0
    }
}

impl From<i64> for DbBanId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<DbBanId> for i64 {
    fn from(value: DbBanId) -> Self {
        value.0
    }
}

impl fmt::Display for DbBanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, sqlx::Type,
)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct DbReplyQueueId(i64);

impl DbReplyQueueId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn as_i64(self) -> i64 {
        self.0
    }
}

impl From<i64> for DbReplyQueueId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<DbReplyQueueId> for i64 {
    fn from(value: DbReplyQueueId) -> Self {
        value.0
    }
}

impl fmt::Display for DbReplyQueueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionStatus {
    Toggled,
}

impl ActionStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Toggled => "toggled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("unsupported mute list mode")]
pub struct MuteListModeParseError;

impl fmt::Display for MuteListMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Blacklist => write!(f, "blacklist"),
            Self::Whitelist => write!(f, "whitelist"),
        }
    }
}

impl TryFrom<&str> for MuteListMode {
    type Error = MuteListModeParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "blacklist" => Ok(Self::Blacklist),
            "whitelist" => Ok(Self::Whitelist),
            _ => Err(MuteListModeParseError),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct TtChannelName(String);

impl TtChannelName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TtChannelName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for TtChannelName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for TtChannelName {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err("channel name is empty");
        }
        Ok(Self(trimmed.to_string()))
    }
}

impl From<String> for TtChannelName {
    fn from(value: String) -> Self {
        Self(value)
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

#[derive(Debug)]
pub enum BridgeEvent {
    Broadcast {
        event_type: NotificationType,
        nickname: String,
        server_name: String,
        related_tt_username: TtUsername,
    },
    ToAdmin {
        user_id: TtUserId,
        nick: String,
        tt_username: TtUsername,
        msg_content: String,
        server_name: String,
    },
    ToAdminChannel {
        channel_id: TtChannelId,
        channel_name: TtChannelName,
        server_name: String,
        msg_content: String,
    },
    WhoReport {
        chat_id: TgChatId,
        text: String,
        reply_to: Option<TgMessageId>,
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
    Broadcast {
        text: String,
    },
    ReplyToUser {
        user_id: TtUserId,
        text: String,
    },
    SendToChannel {
        channel_id: TtChannelId,
        text: String,
    },
    EnqueueStream {
        channel_id: TtChannelId,
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
        user_id: TtUserId,
    },
    BanUser {
        user_id: TtUserId,
    },
    Who {
        chat_id: TgChatId,
        lang: LanguageCode,
        reply_to: Option<TgMessageId>,
    },
    LoadAccounts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteUser {
    pub id: TtUserId,
    pub nickname: String,
    pub username: TtUsername,
    pub channel_name: TtChannelName,
}

#[cfg(test)]
#[path = "../../tests/unit/core_types.rs"]
mod tests;
