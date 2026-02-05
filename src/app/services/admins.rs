use crate::bootstrap::config::Config;
use crate::core::types::TelegramId;
use crate::infra::db::Database;
use anyhow::Result;

pub async fn is_admin(db: &Database, config: &Config, telegram_id: TelegramId) -> bool {
    if telegram_id == config.telegram.admin_chat_id {
        return true;
    }
    match db.get_all_admins().await {
        Ok(admins) => admins.contains(&telegram_id),
        Err(e) => {
            tracing::error!(error = %e, "Failed to load admin list");
            false
        }
    }
}

pub async fn get_all_admins(db: &Database) -> Result<Vec<TelegramId>> {
    db.get_all_admins().await
}

pub async fn add_admin(db: &Database, telegram_id: TelegramId) -> Result<()> {
    db.add_admin(telegram_id).await.map(|_| ())
}

pub async fn remove_admin(db: &Database, telegram_id: TelegramId) -> Result<()> {
    db.remove_admin(telegram_id).await.map(|_| ())
}
