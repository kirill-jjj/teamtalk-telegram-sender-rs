use crate::core::types::{LanguageCode, TelegramId};
use anyhow::Result;
use async_trait::async_trait;

pub async fn get_or_create(
    db: &impl UserSettingsRepo,
    telegram_id: TelegramId,
    default_lang: LanguageCode,
) -> Result<crate::infra::db::types::UserSettings> {
    db.get_or_create_user(telegram_id, default_lang).await
}

#[async_trait]
pub trait UserSettingsRepo: Sync {
    async fn get_or_create_user(
        &self,
        telegram_id: TelegramId,
        default_lang: LanguageCode,
    ) -> Result<crate::infra::db::types::UserSettings>;
}

#[async_trait]
impl UserSettingsRepo for crate::infra::db::Database {
    async fn get_or_create_user(
        &self,
        telegram_id: TelegramId,
        default_lang: LanguageCode,
    ) -> Result<crate::infra::db::types::UserSettings> {
        self.get_or_create_user(telegram_id, default_lang).await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app_user_settings.rs"]
mod tests;
