use crate::app::services::admins as admins_service;
use crate::app::services::user_settings as user_settings_service;
use crate::core::types::{LanguageCode, TelegramId};
use crate::infra::db::Database;
use crate::infra::db::types::SubscriberInfo;
use crate::infra::db::types::UserSettings;
use anyhow::Result;

pub struct SubscriberDetails {
    pub settings: UserSettings,
    pub is_admin: bool,
}

pub async fn get_subscribers(db: &Database) -> Result<Vec<SubscriberInfo>> {
    db.get_subscribers().await
}

pub async fn get_subscriber_details(
    db: &Database,
    telegram_id: TelegramId,
    default_lang: LanguageCode,
) -> Result<SubscriberDetails> {
    let settings = user_settings_service::get_or_create(db, telegram_id, default_lang).await?;
    let admins = admins_service::get_all_admins(db).await?;
    let is_admin = admins.contains(&telegram_id);
    Ok(SubscriberDetails { settings, is_admin })
}

pub async fn get_tt_username_by_telegram_id(
    db: &Database,
    telegram_id: TelegramId,
) -> Result<Option<crate::core::types::TtUsername>> {
    db.get_tt_username_by_telegram_id(telegram_id).await
}

pub async fn get_telegram_id_by_tt_user(
    db: &Database,
    tt_username: &crate::core::types::TtUsername,
) -> Option<TelegramId> {
    db.get_telegram_id_by_tt_user(tt_username).await
}
