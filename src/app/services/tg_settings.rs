use crate::app::services::subscriber_actions as subscriber_actions_service;
use crate::app::services::user_settings as user_settings_service;
use crate::core::types::{LanguageCode, NotificationSetting, TelegramId};
use crate::infra::db::Database;
use anyhow::Result;

pub async fn load_settings(
    db: &Database,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> Result<crate::infra::db::types::UserSettings> {
    user_settings_service::get_or_create(db, telegram_id, lang).await
}

pub async fn update_notifications(
    db: &Database,
    telegram_id: TelegramId,
    setting: NotificationSetting,
) -> Result<()> {
    subscriber_actions_service::update_notifications(db, telegram_id, setting).await
}

pub async fn update_language(
    db: &Database,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> Result<()> {
    user_settings_service::update_language(db, telegram_id, lang).await
}

pub async fn toggle_noon(db: &Database, telegram_id: TelegramId) -> Result<bool> {
    user_settings_service::toggle_noon(db, telegram_id).await
}

pub async fn admin_sub_events_enabled(
    db: &Database,
    telegram_id: TelegramId,
    default_lang: LanguageCode,
) -> Result<bool> {
    Ok(load_settings(db, telegram_id, default_lang)
        .await?
        .admin_sub_events_enabled)
}

pub async fn set_admin_sub_events_enabled(
    db: &Database,
    telegram_id: TelegramId,
    enabled: bool,
) -> Result<()> {
    db.update_admin_sub_events_enabled(telegram_id, enabled)
        .await
}
