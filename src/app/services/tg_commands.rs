use crate::app::services::tg_admin as tg_admin_service;
use crate::app::services::tg_settings as tg_settings_service;
use crate::core::types::{LanguageCode, TelegramId};
use crate::infra::db::Database;
use anyhow::Result;

#[derive(Debug)]
pub enum CommandInitError {
    Notify(anyhow::Error),
}

impl CommandInitError {
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
) -> Result<LanguageCode, CommandInitError> {
    tg_settings_service::load_settings(db, telegram_id, default_lang)
        .await
        .map(|s| s.language_code)
        .map_err(CommandInitError::Notify)
}

pub async fn is_admin(
    db: &Database,
    config: &crate::bootstrap::config::Config,
    telegram_id: TelegramId,
) -> bool {
    tg_admin_service::is_admin(db, config, telegram_id).await
}
