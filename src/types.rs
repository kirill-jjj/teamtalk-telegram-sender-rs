use std::fmt;

#[derive(Debug, Clone, PartialEq)]
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

impl From<&str> for NotificationSetting {
    fn from(s: &str) -> Self {
        match s {
            "join_off" => NotificationSetting::JoinOff,
            "leave_off" => NotificationSetting::LeaveOff,
            "none" => NotificationSetting::None,
            _ => NotificationSetting::All,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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

impl From<&str> for MuteListMode {
    fn from(s: &str) -> Self {
        match s {
            "whitelist" => MuteListMode::Whitelist,
            _ => MuteListMode::Blacklist,
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
    Who { chat_id: i64, lang: String },
    LoadAccounts,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiteUser {
    pub id: i32,
    pub nickname: String,
    pub username: String,
    pub channel_name: String,
}
