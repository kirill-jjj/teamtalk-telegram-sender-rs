use crate::infra::db::Database;
use anyhow::Result;

pub async fn cleanup_deleted_banned_user(db: &Database, telegram_id: i64) -> Result<()> {
    db.delete_user_profile(telegram_id).await
}

pub async fn get_telegram_id_by_tt_user(db: &Database, tt_username: &str) -> Option<i64> {
    db.get_telegram_id_by_tt_user(tt_username).await
}
