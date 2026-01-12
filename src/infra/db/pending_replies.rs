use anyhow::Result;

use super::Database;

impl Database {
    pub async fn add_pending_reply(&self, tg_message_id: i64, tt_user_id: i32) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO pending_replies (tg_message_id, tt_user_id) VALUES (?, ?)",
        )
        .bind(tg_message_id)
        .bind(tt_user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_pending_reply_user_id(&self, tg_message_id: i64) -> Result<Option<i32>> {
        let res = sqlx::query_scalar::<_, i32>(
            "SELECT tt_user_id FROM pending_replies WHERE tg_message_id = ?",
        )
        .bind(tg_message_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(res)
    }

    pub async fn touch_pending_reply(&self, tg_message_id: i64) -> Result<()> {
        sqlx::query(
            "UPDATE pending_replies SET last_used_at = CURRENT_TIMESTAMP WHERE tg_message_id = ?",
        )
        .bind(tg_message_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn cleanup_pending_replies(&self, ttl_seconds: i64) -> Result<u64> {
        let window = format!("-{} seconds", ttl_seconds);
        let res =
            sqlx::query("DELETE FROM pending_replies WHERE last_used_at < datetime('now', ?)")
                .bind(window)
                .execute(&self.pool)
                .await?;
        Ok(res.rows_affected())
    }
}
