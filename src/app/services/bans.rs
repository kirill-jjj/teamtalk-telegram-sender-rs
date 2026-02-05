use crate::infra::db::Database;
use crate::infra::db::types::BanEntry;
use anyhow::Result;

pub async fn get_banned_users(db: &Database) -> Result<Vec<BanEntry>> {
    db.get_banned_users().await
}
