use crate::core::types::{LanguageCode, MuteListMode, NotificationSetting};
use anyhow::{Result, anyhow};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum CallbackAction {
    Menu(MenuAction),
    Admin(AdminAction),
    Settings(SettingsAction),
    Subscriber(SubAction),
    Mute(MuteAction),
    Unsub(UnsubAction),
    NoOp,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum MenuAction {
    Who,
    Settings,
    Help,
    Unsub,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum AdminAction {
    KickList { page: usize },
    KickPerform { user_id: i32 },
    BanList { page: usize },
    BanPerform { user_id: i32 },
    UnbanList { page: usize },
    UnbanPerform { ban_db_id: i64, page: usize },
    SubsList { page: usize },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum SettingsAction {
    Main,
    LangSelect,
    LangSet { lang: LanguageCode },
    SubSelect,
    SubSet { setting: NotificationSetting },
    NotifSelect,
    NoonToggle,
    MuteManage,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum SubAction {
    Details {
        sub_id: i64,
        page: usize,
    },
    Delete {
        sub_id: i64,
        page: usize,
    },
    Ban {
        sub_id: i64,
        page: usize,
    },
    ManageTt {
        sub_id: i64,
        page: usize,
    },
    Unlink {
        sub_id: i64,
        page: usize,
    },
    LinkList {
        sub_id: i64,
        page: usize,
        list_page: usize,
    },
    LinkPerform {
        sub_id: i64,
        page: usize,
        username: String,
    },
    LangMenu {
        sub_id: i64,
        page: usize,
    },
    LangSet {
        sub_id: i64,
        page: usize,
        lang: LanguageCode,
    },
    NotifMenu {
        sub_id: i64,
        page: usize,
    },
    NotifSet {
        sub_id: i64,
        page: usize,
        val: NotificationSetting,
    },
    NoonToggle {
        sub_id: i64,
        page: usize,
    },
    ModeMenu {
        sub_id: i64,
        page: usize,
    },
    ModeSet {
        sub_id: i64,
        page: usize,
        mode: MuteListMode,
    },
    MuteView {
        sub_id: i64,
        page: usize,
        view_page: usize,
    },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum MuteAction {
    ModeSet { mode: MuteListMode },
    Menu { mode: MuteListMode },
    List { page: usize },
    Toggle { username: String, page: usize },
    ServerList { page: usize },
    ServerToggle { username: String, page: usize },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum UnsubAction {
    Confirm,
    Cancel,
}

fn encode_callback(action: &CallbackAction) -> String {
    let bytes = match postcard::to_stdvec(action) {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to serialize callback action: {}", e);
            return "noop".to_string();
        }
    };
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    if encoded.len() > 64 {
        tracing::error!("Callback data too long ({} bytes)", encoded.len());
        return "noop".to_string();
    }
    encoded
}

impl fmt::Display for CallbackAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encoded = encode_callback(self);
        write!(f, "{}", encoded)
    }
}

impl fmt::Display for MenuAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            encode_callback(&CallbackAction::Menu(self.clone()))
        )
    }
}

impl fmt::Display for AdminAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            encode_callback(&CallbackAction::Admin(self.clone()))
        )
    }
}

impl fmt::Display for SettingsAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            encode_callback(&CallbackAction::Settings(self.clone()))
        )
    }
}

impl fmt::Display for SubAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            encode_callback(&CallbackAction::Subscriber(self.clone()))
        )
    }
}

impl fmt::Display for MuteAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            encode_callback(&CallbackAction::Mute(self.clone()))
        )
    }
}

impl fmt::Display for UnsubAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            encode_callback(&CallbackAction::Unsub(self.clone()))
        )
    }
}

impl FromStr for CallbackAction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "noop" {
            return Ok(CallbackAction::NoOp);
        }
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|e| anyhow!("Invalid callback encoding: {}", e))?;
        postcard::from_bytes(&bytes).map_err(|e| anyhow!("Invalid callback data: {}", e))
    }
}
