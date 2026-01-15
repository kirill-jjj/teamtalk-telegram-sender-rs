use anyhow::Result;
use chrono::Utc;

use super::{Database, types::BanEntry};

impl Database {
    pub async fn add_ban(
        &self,
        telegram_id: Option<i64>,
        teamtalk_username: Option<String>,
        reason: Option<String>,
    ) -> Result<()> {
        let now = Utc::now().naive_utc();
        sqlx::query!(
            "INSERT INTO ban_list (telegram_id, teamtalk_username, ban_reason, banned_at) VALUES (?, ?, ?, ?)",
            telegram_id,
            teamtalk_username,
            reason,
            now
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_banned_users(&self) -> Result<Vec<BanEntry>> {
        let rows = sqlx::query_as!(
            BanEntry,
            "SELECT id, telegram_id, teamtalk_username FROM ban_list ORDER BY banned_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn remove_ban_by_id(&self, id: i64) -> Result<()> {
        sqlx::query!("DELETE FROM ban_list WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn is_telegram_id_banned(&self, telegram_id: i64) -> Result<bool> {
        let record = sqlx::query!(
            "SELECT count(*) as count FROM ban_list WHERE telegram_id = ?",
            telegram_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(record.count > 0)
    }

    pub async fn is_teamtalk_username_banned(&self, tt_username: &str) -> Result<bool> {
        let record = sqlx::query!(
            "SELECT count(*) as count FROM ban_list WHERE teamtalk_username = ? COLLATE NOCASE",
            tt_username
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(record.count > 0)
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/infra_db_bans.rs"]
mod tests;
