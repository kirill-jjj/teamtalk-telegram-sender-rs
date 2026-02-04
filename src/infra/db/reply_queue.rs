use crate::core::types::{DbReplyQueueId, TelegramId, TtUsername};
use anyhow::Result;
use chrono::NaiveDateTime;

use super::Database;

#[derive(sqlx::FromRow, Debug)]
pub struct ReplyQueueItem {
    pub id: DbReplyQueueId,
    pub message_text: String,
    pub created_at: NaiveDateTime,
}

impl Database {
    pub async fn add_reply_queue_item(
        &self,
        tt_username: &TtUsername,
        admin_telegram_id: TelegramId,
        message_text: &str,
    ) -> Result<()> {
        let tt_username = tt_username.as_str();
        sqlx::query!(
            "INSERT INTO reply_queue (tt_username, admin_telegram_id, message_text) VALUES (?, ?, ?)",
            tt_username,
            admin_telegram_id,
            message_text
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_reply_queue_for_user(
        &self,
        tt_username: &TtUsername,
    ) -> Result<Vec<ReplyQueueItem>> {
        let tt_username = tt_username.as_str();
        let rows = sqlx::query_as!(
            ReplyQueueItem,
            r#"
            SELECT
                id as "id!: DbReplyQueueId",
                message_text as "message_text!",
                created_at as "created_at!"
            FROM reply_queue
            WHERE tt_username = ?
            ORDER BY created_at ASC, id ASC
            "#,
            tt_username
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn delete_reply_queue_ids(&self, ids: &[DbReplyQueueId]) -> Result<u64> {
        if ids.is_empty() {
            return Ok(0);
        }
        let mut tx = self.pool.begin().await?;
        let mut removed = 0u64;
        for id in ids {
            let res = sqlx::query("DELETE FROM reply_queue WHERE id = ?")
                .bind(id.as_i64())
                .execute(&mut *tx)
                .await?;
            removed += res.rows_affected();
        }
        tx.commit().await?;
        Ok(removed)
    }

    pub async fn clear_reply_queue_for_user(&self, tt_username: &TtUsername) -> Result<u64> {
        let tt_username = tt_username.as_str();
        let res = sqlx::query("DELETE FROM reply_queue WHERE tt_username = ?")
            .bind(tt_username)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }

    pub async fn clear_reply_queue_all(&self) -> Result<u64> {
        let res = sqlx::query("DELETE FROM reply_queue")
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }
}
