use anyhow::Result;

use crate::db::Database;
use sqlx::Row;

impl Database {
    pub async fn add_pending_channel_reply(
        &self,
        tg_message_id: i64,
        channel_id: i32,
        channel_name: &str,
        server_name: &str,
        original_text: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO pending_channel_replies (tg_message_id, channel_id, channel_name, server_name, original_text) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(tg_message_id)
        .bind(channel_id)
        .bind(channel_name)
        .bind(server_name)
        .bind(original_text)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_pending_channel_reply(
        &self,
        tg_message_id: i64,
    ) -> Result<Option<(i32, String, String, String)>> {
        let row = sqlx::query(
            r#"
            SELECT
                channel_id,
                channel_name,
                server_name,
                original_text
            FROM pending_channel_replies
            WHERE tg_message_id = ?
            "#,
        )
        .bind(tg_message_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            (
                r.get::<i32, _>("channel_id"),
                r.get::<String, _>("channel_name"),
                r.get::<String, _>("server_name"),
                r.get::<String, _>("original_text"),
            )
        }))
    }

    pub async fn touch_pending_channel_reply(&self, tg_message_id: i64) -> Result<()> {
        sqlx::query(
            "UPDATE pending_channel_replies SET last_used_at = CURRENT_TIMESTAMP WHERE tg_message_id = ?",
        )
        .bind(tg_message_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn cleanup_pending_channel_replies(&self, ttl_seconds: i64) -> Result<u64> {
        let window = format!("-{} seconds", ttl_seconds);
        let res = sqlx::query(
            "DELETE FROM pending_channel_replies WHERE last_used_at < datetime('now', ?)",
        )
        .bind(window)
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected())
    }
}
