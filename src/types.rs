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
    ReplyToUser { user_id: i32, text: String },
    KickUser { user_id: i32 },
    BanUser { user_id: i32 },
    Who { chat_id: i64, lang: LanguageCode },
    LoadAccounts,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiteUser {
    pub id: i32,
    pub nickname: String,
    pub username: String,
    pub channel_name: String,
}
