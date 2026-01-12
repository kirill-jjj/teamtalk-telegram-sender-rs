use crate::infra::db::Database;
use anyhow::Result;

pub async fn is_admin(db: &Database, telegram_id: i64) -> Result<bool> {
    let admins = db.get_all_admins().await?;
    Ok(admins.contains(&telegram_id))
}

pub async fn add_admin(db: &Database, telegram_id: i64) -> Result<bool> {
    db.add_admin(telegram_id).await
}

pub async fn remove_admin(db: &Database, telegram_id: i64) -> Result<bool> {
    db.remove_admin(telegram_id).await
}

pub async fn list_admins(db: &Database) -> Result<Vec<i64>> {
    db.get_all_admins().await
}
