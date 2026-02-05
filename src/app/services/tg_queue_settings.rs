use crate::app::services::reply_queue as reply_queue_service;
use crate::app::services::user_settings as user_settings_service;
use crate::core::types::{LanguageCode, TelegramId, TtUsername};
use crate::infra::db::Database;
use anyhow::Result;

pub async fn load_settings(
    db: &Database,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> Result<crate::infra::db::types::UserSettings> {
    user_settings_service::get_or_create(db, telegram_id, lang).await
}

pub async fn global_enabled(db: &Database) -> Result<bool> {
    reply_queue_service::get_reply_queue_global_enabled(db).await
}

pub async fn set_user_enabled(db: &Database, telegram_id: TelegramId, enabled: bool) -> Result<()> {
    reply_queue_service::set_reply_queue_user_enabled(db, telegram_id, enabled).await
}

pub async fn set_global_enabled(db: &Database, enabled: bool) -> Result<()> {
    reply_queue_service::set_reply_queue_global_enabled(db, enabled).await
}

pub async fn clear_user(db: &Database, tt_username: &TtUsername) -> Result<u64> {
    reply_queue_service::clear_reply_queue_for_user(db, tt_username).await
}

pub async fn clear_all(db: &Database) -> Result<u64> {
    reply_queue_service::clear_reply_queue_all(db).await
}
