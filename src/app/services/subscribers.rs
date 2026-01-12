use crate::infra::db::Database;
use anyhow::Result;

pub async fn list_subscribers(
    db: &Database,
) -> Result<Vec<crate::infra::db::types::SubscriberInfo>> {
    db.get_subscribers().await
}

pub async fn get_user_settings(
    db: &Database,
    telegram_id: i64,
    default_lang: crate::core::types::LanguageCode,
) -> Result<crate::infra::db::types::UserSettings> {
    db.get_or_create_user(telegram_id, default_lang).await
}
