use crate::infra::db::Database;
use anyhow::Result;

pub async fn is_admin(db: &Database, telegram_id: i64) -> Result<bool> {
    let admins = db.get_all_admins().await?;
    Ok(admins.contains(&telegram_id))
}
