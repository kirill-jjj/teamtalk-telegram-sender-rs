use crate::core::types::{LanguageCode, NotificationSetting};
use crate::infra::db::Database;
use anyhow::Result;

pub async fn update_language(db: &Database, telegram_id: i64, lang: LanguageCode) -> Result<()> {
    db.update_language(telegram_id, lang).await
}

pub async fn update_notifications(
    db: &Database,
    telegram_id: i64,
    setting: NotificationSetting,
) -> Result<()> {
    db.update_notification_setting(telegram_id, setting).await
}

pub async fn toggle_noon(db: &Database, telegram_id: i64) -> Result<bool> {
    db.toggle_noon(telegram_id).await
}
