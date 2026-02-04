use anyhow::Result;

use crate::core::types::TelegramId;

use super::Database;

impl Database {
    pub async fn add_admin(&self, telegram_id: TelegramId) -> Result<bool> {
        let res = sqlx::query!(
            "INSERT OR IGNORE INTO admins (telegram_id) VALUES (?)",
            telegram_id
        )
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn remove_admin(&self, telegram_id: TelegramId) -> Result<bool> {
        let res = sqlx::query!("DELETE FROM admins WHERE telegram_id = ?", telegram_id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn get_all_admins(&self) -> Result<Vec<TelegramId>> {
        let rows: Vec<i64> = sqlx::query_scalar!("SELECT telegram_id FROM admins")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(TelegramId::from).collect())
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/infra_db_admins.rs"]
mod tests;
