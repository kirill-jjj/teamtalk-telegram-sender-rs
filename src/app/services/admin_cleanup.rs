use crate::core::types::{TelegramId, TtUsername};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait AdminCleanupRepo: Sync {
    async fn delete_user_profile(&self, telegram_id: TelegramId) -> Result<()>;
    async fn get_telegram_id_by_tt_user(&self, tt_username: &TtUsername) -> Option<TelegramId>;
}

pub async fn cleanup_deleted_banned_user(
    db: &impl AdminCleanupRepo,
    telegram_id: TelegramId,
) -> Result<()> {
    db.delete_user_profile(telegram_id).await
}

pub async fn get_telegram_id_by_tt_user(
    db: &impl AdminCleanupRepo,
    tt_username: &TtUsername,
) -> Option<TelegramId> {
    db.get_telegram_id_by_tt_user(tt_username).await
}

#[async_trait]
impl AdminCleanupRepo for crate::infra::db::Database {
    async fn delete_user_profile(&self, telegram_id: TelegramId) -> Result<()> {
        self.delete_user_profile(telegram_id).await
    }

    async fn get_telegram_id_by_tt_user(&self, tt_username: &TtUsername) -> Option<TelegramId> {
        self.get_telegram_id_by_tt_user(tt_username).await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app_admin_cleanup.rs"]
mod tests;
