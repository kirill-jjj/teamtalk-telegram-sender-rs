use crate::core::types::{
    AfkListMode, DbBanId, DeeplinkAction, LanguageCode, MuteListMode, NotificationSetting,
    TelegramId, TtUsername,
};
use chrono::NaiveDateTime;

#[derive(sqlx::FromRow, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct UserSettings {
    pub telegram_id: TelegramId,
    pub language_code: LanguageCode,
    pub notification_settings: NotificationSetting,
    pub mute_list_mode: MuteListMode,
    pub teamtalk_username: Option<TtUsername>,
    pub not_on_online_enabled: bool,
    pub not_on_online_confirmed: bool,
    pub reply_queue_enabled: bool,
    pub admin_sub_events_enabled: bool,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Deeplink {
    pub action: DeeplinkAction,
    pub payload: Option<String>,
    pub expected_telegram_id: Option<TelegramId>,
    pub expiry_time: NaiveDateTime,
}

#[derive(sqlx::FromRow, Debug)]
pub struct BanEntry {
    pub id: DbBanId,
    pub telegram_id: Option<TelegramId>,
    pub teamtalk_username: Option<TtUsername>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct SubscriberInfo {
    pub telegram_id: TelegramId,
    pub teamtalk_username: Option<TtUsername>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct AfkUserSettings {
    pub enabled: bool,
    pub threshold_minutes: i64,
    pub list_mode: AfkListMode,
    pub cooldown_seconds: i64,
}

#[derive(Debug, Clone)]
pub struct AfkResolvedSettings {
    pub enabled: bool,
    pub threshold_minutes: i64,
    pub list_mode: AfkListMode,
    pub cooldown_seconds: i64,
}

#[derive(Debug, Clone)]
pub struct AfkRecipient {
    pub telegram_id: TelegramId,
    pub threshold_minutes: i64,
    pub cooldown_seconds: i64,
}
