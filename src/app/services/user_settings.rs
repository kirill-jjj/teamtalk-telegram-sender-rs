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

pub async fn update_language(
    db: &impl UserSettingsRepo,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> Result<()> {
    db.update_language(telegram_id, lang).await
}

pub async fn toggle_noon(db: &impl UserSettingsRepo, telegram_id: TelegramId) -> Result<bool> {
    db.toggle_noon(telegram_id).await
}

#[async_trait]
pub trait UserSettingsRepo: Sync {
    async fn get_or_create_user(
        &self,
        telegram_id: TelegramId,
        default_lang: LanguageCode,
    ) -> Result<crate::infra::db::types::UserSettings>;
    async fn update_language(&self, telegram_id: TelegramId, lang: LanguageCode) -> Result<()>;
    async fn toggle_noon(&self, telegram_id: TelegramId) -> Result<bool>;
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

    async fn update_language(&self, telegram_id: TelegramId, lang: LanguageCode) -> Result<()> {
        self.update_language(telegram_id, lang).await
    }

    async fn toggle_noon(&self, telegram_id: TelegramId) -> Result<bool> {
        self.toggle_noon(telegram_id).await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app_user_settings.rs"]
mod tests;
