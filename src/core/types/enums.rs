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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum MuteListMode {
    Blacklist,
    Whitelist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum AfkListMode {
    None,
    Blacklist,
    Whitelist,
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

impl fmt::Display for AfkListMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Blacklist => write!(f, "blacklist"),
            Self::Whitelist => write!(f, "whitelist"),
        }
    }
}

impl TryFrom<&str> for AfkListMode {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "none" => Ok(Self::None),
            "blacklist" => Ok(Self::Blacklist),
            "whitelist" => Ok(Self::Whitelist),
            _ => Err("unsupported afk list mode"),
        }
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NotificationType {
    Join,
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingFileFormat {
    ChannelCodec,
    Wave,
    Mp3_128,
}

impl TryFrom<&str> for RecordingFileFormat {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "channel_codec" | "ogg" => Ok(Self::ChannelCodec),
            "wave" | "wav" => Ok(Self::Wave),
            "mp3_128" | "mp3" => Ok(Self::Mp3_128),
            _ => Err("unsupported recording file format"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum JoinGender {
    Male,
    Female,
    Neutral,
}
