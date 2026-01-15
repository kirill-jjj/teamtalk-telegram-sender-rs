use crate::core::types::{LanguageCode, MuteListMode, NotificationSetting, TtUsername};
use anyhow::{Result, anyhow};
use derive_more::From;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, From)]
pub enum CallbackAction {
    Menu(MenuAction),
    Admin(AdminAction),
    Settings(SettingsAction),
    Subscriber(SubAction),
    Mute(MuteAction),
    Unsub(UnsubAction),
    NoOp,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum MenuAction {
    Who,
    Settings,
    Help,
    Unsub,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum AdminAction {
    KickList { page: usize },
    KickPerform { user_id: i32 },
    BanList { page: usize },
    BanPerform { user_id: i32 },
    UnbanList { page: usize },
    UnbanPerform { ban_db_id: i64, page: usize },
    SubsList { page: usize },
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
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
        username: TtUsername,
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

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum MuteAction {
    ModeSet { mode: MuteListMode },
    Menu { mode: MuteListMode },
    List { page: usize },
    ServerList { page: usize },
    Toggle { username: TtUsername, page: usize },
    ServerToggle { username: TtUsername, page: usize },
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum UnsubAction {
    Confirm,
    Cancel,
}

pub trait AsCallbackData {
    fn into_data(self) -> String;
}

impl<T> AsCallbackData for T
where
    T: Into<CallbackAction>,
{
    fn into_data(self) -> String {
        let action: CallbackAction = self.into();
        encode_callback(&action)
    }
}

fn encode_callback(action: &CallbackAction) -> String {
    let bytes = match postcard::to_stdvec(action) {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!(error = %e, "Failed to serialize callback action");
            return "noop".to_string();
        }
    };
    let encoded = z85::encode(bytes);
    if encoded.len() > 64 {
        tracing::error!(len = encoded.len(), "Callback data too long");
        return "noop".to_string();
    }
    encoded
}

impl FromStr for CallbackAction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "noop" {
            return Ok(Self::NoOp);
        }
        let bytes =
            z85::decode(s.as_bytes()).map_err(|e| anyhow!("Invalid callback encoding: {e}"))?;
        postcard::from_bytes(&bytes).map_err(|e| anyhow!("Invalid callback data: {e}"))
    }
}
