use crate::app::services::subscriber_actions as subscriber_actions_service;
use crate::app::services::user_settings as user_settings_service;
use crate::core::types::{LanguageCode, MuteListMode, NotificationSetting, TelegramId};
use crate::infra::db::Database;
use anyhow::Result;

pub async fn update_language(db: &Database, sub_id: TelegramId, lang: LanguageCode) -> Result<()> {
    user_settings_service::update_language(db, sub_id, lang).await
}

pub async fn update_notifications(
    db: &Database,
    sub_id: TelegramId,
    setting: NotificationSetting,
) -> Result<()> {
    subscriber_actions_service::update_notifications(db, sub_id, setting).await
}

pub async fn toggle_noon(db: &Database, sub_id: TelegramId) -> Result<()> {
    user_settings_service::toggle_noon(db, sub_id)
        .await
        .map(|_| ())
}

pub async fn update_mute_mode(db: &Database, sub_id: TelegramId, mode: MuteListMode) -> Result<()> {
    subscriber_actions_service::update_mute_mode(db, sub_id, mode).await
}
