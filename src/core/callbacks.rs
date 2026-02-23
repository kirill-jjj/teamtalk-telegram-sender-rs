use crate::core::types::{
    AfkListMode, DbBanId, LanguageCode, MuteListMode, NotificationSetting, TelegramId, TtUserId,
    TtUsername,
};
use anyhow::{Result, anyhow};
use derive_more::From;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

const CALLBACK_NOOP_MARKER: &str = "noop";
const CALLBACK_TOO_LONG_MARKER: &str = "__cb_too_long__";
const CALLBACK_SERIALIZE_ERROR_MARKER: &str = "__cb_serialize_error__";

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
    KickPerform { user_id: TtUserId },
    BanList { page: usize },
    BanPerform { user_id: TtUserId },
    UnbanList { page: usize },
    UnbanPerform { ban_db_id: DbBanId, page: usize },
    SubsList { page: usize },
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum SettingsAction {
    Main,
    LangSelect,
    LangSet {
        lang: LanguageCode,
    },
    SubSelect,
    SubSet {
        setting: NotificationSetting,
    },
    NotifSelect,
    AfkMenu,
    NoonToggle,
    AdminSubEventsToggle,
    MuteManage,
    QueueMenu,
    QueueToggleUser,
    QueueToggleGlobal,
    QueueClearSelf,
    QueueClearAll,
    AfkToggle,
    AfkThresholdStep {
        delta: i32,
    },
    AfkCooldownStep {
        delta: i32,
    },
    AfkModeSet {
        mode: AfkListMode,
    },
    AfkList {
        mode: AfkListMode,
        page: usize,
    },
    AfkListToggle {
        mode: AfkListMode,
        username: TtUsername,
        page: usize,
    },
    AfkOverrides {
        page: usize,
    },
    AfkOverrideDelete {
        username: TtUsername,
        page: usize,
    },
    AfkOverrideSetPreset {
        username: TtUsername,
        minutes: i64,
        page: usize,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum SubAction {
    Details {
        sub_id: TelegramId,
        page: usize,
    },
    AdminAddConfirm {
        sub_id: TelegramId,
        page: usize,
    },
    AdminAdd {
        sub_id: TelegramId,
        page: usize,
    },
    AdminRemoveConfirm {
        sub_id: TelegramId,
        page: usize,
    },
    AdminRemove {
        sub_id: TelegramId,
        page: usize,
    },
    Delete {
        sub_id: TelegramId,
        page: usize,
    },
    Ban {
        sub_id: TelegramId,
        page: usize,
    },
    ManageTt {
        sub_id: TelegramId,
        page: usize,
    },
    Unlink {
        sub_id: TelegramId,
        page: usize,
    },
    LinkList {
        sub_id: TelegramId,
        page: usize,
        list_page: usize,
    },
    LinkPerform {
        sub_id: TelegramId,
        page: usize,
        username: TtUsername,
    },
    LangMenu {
        sub_id: TelegramId,
        page: usize,
    },
    LangSet {
        sub_id: TelegramId,
        page: usize,
        lang: LanguageCode,
    },
    NotifMenu {
        sub_id: TelegramId,
        page: usize,
    },
    NotifSet {
        sub_id: TelegramId,
        page: usize,
        val: NotificationSetting,
    },
    NoonToggle {
        sub_id: TelegramId,
        page: usize,
    },
    ModeMenu {
        sub_id: TelegramId,
        page: usize,
    },
    ModeSet {
        sub_id: TelegramId,
        page: usize,
        mode: MuteListMode,
    },
    MuteView {
        sub_id: TelegramId,
        page: usize,
        view_page: usize,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum MuteAction {
    ModeSet {
        mode: MuteListMode,
    },
    Menu {
        mode: MuteListMode,
    },
    List {
        mode: MuteListMode,
        page: usize,
    },
    ServerList {
        mode: MuteListMode,
        page: usize,
    },
    Toggle {
        mode: MuteListMode,
        username: TtUsername,
        page: usize,
    },
    ServerToggle {
        mode: MuteListMode,
        username: TtUsername,
        page: usize,
    },
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
            return CALLBACK_SERIALIZE_ERROR_MARKER.to_string();
        }
    };
    let encoded = z85::encode(bytes);
    if encoded.len() > 64 {
        tracing::error!(len = encoded.len(), "Callback data too long");
        return CALLBACK_TOO_LONG_MARKER.to_string();
    }
    encoded
}

impl FromStr for CallbackAction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == CALLBACK_NOOP_MARKER {
            return Ok(Self::NoOp);
        }
        if s == CALLBACK_TOO_LONG_MARKER {
            return Err(anyhow!("Callback data exceeded Telegram 64-byte limit"));
        }
        if s == CALLBACK_SERIALIZE_ERROR_MARKER {
            return Err(anyhow!("Callback data serialization failed"));
        }
        let bytes =
            z85::decode(s.as_bytes()).map_err(|e| anyhow!("Invalid callback encoding: {e}"))?;
        postcard::from_bytes(&bytes).map_err(|e| anyhow!("Invalid callback data: {e}"))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/core_callbacks.rs"]
mod tests;
