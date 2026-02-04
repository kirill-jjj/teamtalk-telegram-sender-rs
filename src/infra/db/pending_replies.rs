use crate::core::types::{TgMessageId, TtUserId, TtUsername};
use anyhow::Result;

use super::Database;

impl Database {
    pub async fn add_pending_reply(
        &self,
        tg_message_id: TgMessageId,
        tt_user_id: TtUserId,
        tt_username: Option<&TtUsername>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO pending_replies (tg_message_id, tt_user_id, tt_username) VALUES (?, ?, ?)",
        )
        .bind(tg_message_id.as_i32())
        .bind(tt_user_id.as_i32())
        .bind(tt_username.map(TtUsername::as_str))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_pending_reply(
        &self,
        tg_message_id: TgMessageId,
    ) -> Result<Option<(TtUserId, Option<TtUsername>)>> {
        let res = sqlx::query_as::<_, (i32, Option<TtUsername>)>(
            "SELECT tt_user_id, tt_username FROM pending_replies WHERE tg_message_id = ?",
        )
        .bind(tg_message_id.as_i32())
        .fetch_optional(&self.pool)
        .await?;
        Ok(res.map(|(id, username)| (TtUserId::from(id), username)))
    }

    pub async fn touch_pending_reply(&self, tg_message_id: TgMessageId) -> Result<()> {
        sqlx::query(
            "UPDATE pending_replies SET last_used_at = CURRENT_TIMESTAMP WHERE tg_message_id = ?",
        )
        .bind(tg_message_id.as_i32())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn cleanup_pending_replies(&self, ttl_seconds: i64) -> Result<u64> {
        let window = format!("-{ttl_seconds} seconds");
        let res =
            sqlx::query("DELETE FROM pending_replies WHERE last_used_at < datetime('now', ?)")
                .bind(window)
                .execute(&self.pool)
                .await?;
        Ok(res.rows_affected())
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/infra_db_pending_replies.rs"]
mod tests;
