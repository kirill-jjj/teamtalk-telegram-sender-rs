use anyhow::Result;
use crate::core::types::{TgMessageId, TtChannelId};

use super::Database;
use sqlx::Row;

impl Database {
    pub async fn add_pending_channel_reply(
        &self,
        tg_message_id: TgMessageId,
        channel_id: TtChannelId,
        channel_name: &str,
        server_name: &str,
        original_text: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO pending_channel_replies (tg_message_id, channel_id, channel_name, server_name, original_text) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(tg_message_id.as_i32())
        .bind(channel_id.as_i32())
        .bind(channel_name)
        .bind(server_name)
        .bind(original_text)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_pending_channel_reply(
        &self,
        tg_message_id: TgMessageId,
    ) -> Result<Option<(TtChannelId, String, String, String)>> {
        let row = sqlx::query(
            r"
            SELECT
                channel_id,
                channel_name,
                server_name,
                original_text
            FROM pending_channel_replies
            WHERE tg_message_id = ?
            ",
        )
        .bind(tg_message_id.as_i32())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            (
                TtChannelId::from(r.get::<i32, _>("channel_id")),
                r.get::<String, _>("channel_name"),
                r.get::<String, _>("server_name"),
                r.get::<String, _>("original_text"),
            )
        }))
    }

    pub async fn touch_pending_channel_reply(&self, tg_message_id: TgMessageId) -> Result<()> {
        sqlx::query(
            "UPDATE pending_channel_replies SET last_used_at = CURRENT_TIMESTAMP WHERE tg_message_id = ?",
        )
        .bind(tg_message_id.as_i32())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn cleanup_pending_channel_replies(&self, ttl_seconds: i64) -> Result<u64> {
        let window = format!("-{ttl_seconds} seconds");
        let res = sqlx::query(
            "DELETE FROM pending_channel_replies WHERE last_used_at < datetime('now', ?)",
        )
        .bind(window)
        .execute(&self.pool)
        .await?;
        Ok(res.rows_affected())
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/infra_db_pending_channel_replies.rs"]
mod tests;
