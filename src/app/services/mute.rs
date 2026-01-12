use crate::core::types::MuteListMode;
use crate::infra::db::Database;
use anyhow::Result;

pub async fn toggle_mute(db: &Database, telegram_id: i64, username: &str) -> Result<()> {
    db.toggle_muted_user(telegram_id, username).await
}

pub async fn update_mode(db: &Database, telegram_id: i64, mode: MuteListMode) -> Result<()> {
    db.update_mute_mode(telegram_id, mode).await
}

pub async fn list_muted_users(db: &Database, telegram_id: i64) -> Result<Vec<String>> {
    db.get_muted_users_list(telegram_id).await
}
