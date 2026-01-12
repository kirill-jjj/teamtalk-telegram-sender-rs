use crate::infra::db::Database;
use anyhow::Result;

pub async fn list_bans(db: &Database) -> Result<Vec<crate::infra::db::types::BanEntry>> {
    db.get_banned_users().await
}

pub async fn add_ban(
    db: &Database,
    telegram_id: Option<i64>,
    tt_username: Option<String>,
    reason: Option<String>,
) -> Result<()> {
    db.add_ban(telegram_id, tt_username, reason).await
}

pub async fn remove_ban(db: &Database, ban_id: i64) -> Result<()> {
    db.remove_ban_by_id(ban_id).await
}

pub async fn get_tt_username_by_telegram_id(
    db: &Database,
    telegram_id: i64,
) -> Result<Option<String>> {
    db.get_tt_username_by_telegram_id(telegram_id).await
}
