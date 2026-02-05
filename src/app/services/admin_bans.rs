use crate::core::types::{DbBanId, TelegramId, TtUsername};
use crate::infra::db::Database;
use anyhow::Result;

pub async fn add_ban(
    db: &Database,
    telegram_id: Option<TelegramId>,
    tt_username: Option<TtUsername>,
    reason: &str,
) -> Result<()> {
    db.add_ban(telegram_id, tt_username, Some(reason.to_string()))
        .await
}

pub async fn remove_ban_by_id(db: &Database, ban_db_id: DbBanId) -> Result<()> {
    db.remove_ban_by_id(ban_db_id).await
}
