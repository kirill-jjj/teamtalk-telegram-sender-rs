use crate::core::types::{LanguageCode, MuteListMode, NotificationSetting};
use crate::infra::db::Database;
use anyhow::Result;

pub async fn get_or_create(
    db: &Database,
    telegram_id: i64,
    default_lang: LanguageCode,
) -> Result<crate::infra::db::types::UserSettings> {
    db.get_or_create_user(telegram_id, default_lang).await
}

pub fn parse_notification_setting(raw: &str) -> NotificationSetting {
    NotificationSetting::try_from(raw).unwrap_or(NotificationSetting::All)
}

pub fn parse_mute_list_mode(raw: &str) -> MuteListMode {
    MuteListMode::try_from(raw).unwrap_or(MuteListMode::Blacklist)
}
