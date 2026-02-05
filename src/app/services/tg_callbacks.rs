use crate::app::services::subscription as subscription_service;
use crate::app::services::user_settings as user_settings_service;
use crate::core::types::{LanguageCode, TelegramId};
use crate::infra::db::Database;
use anyhow::Result;

#[derive(Debug)]
pub enum CallbackError {
    Notify(anyhow::Error),
}

impl CallbackError {
    pub fn into_error(self) -> anyhow::Error {
        match self {
            Self::Notify(e) => e,
        }
    }
}

pub async fn load_user_lang(
    db: &Database,
    telegram_id: TelegramId,
    default_lang: LanguageCode,
) -> Result<LanguageCode, CallbackError> {
    let user_settings = user_settings_service::get_or_create(db, telegram_id, default_lang)
        .await
        .map_err(CallbackError::Notify)?;
    Ok(user_settings.language_code)
}

pub async fn ensure_subscribed(
    db: &Database,
    telegram_id: TelegramId,
) -> Result<bool, CallbackError> {
    subscription_service::is_subscribed(db, telegram_id)
        .await
        .map_err(CallbackError::Notify)
}
