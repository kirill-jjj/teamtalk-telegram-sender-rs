use crate::core::types::TelegramId;
use anyhow::Result;
use sqlx::Row;

use super::Database;

#[derive(Debug, Clone)]
pub struct SubscriberNotifyOutboxItem {
    pub id: i64,
    pub target_telegram_id: TelegramId,
    pub message_text: String,
    pub attempts: i64,
}

impl Database {
    pub async fn add_subscriber_notify_outbox_item(
        &self,
        target_telegram_id: TelegramId,
        message_text: &str,
    ) -> Result<()> {
        sqlx::query(
            r"
            INSERT INTO subscriber_notify_outbox (target_telegram_id, message_text)
            VALUES (?, ?)
            ",
        )
        .bind(target_telegram_id.as_i64())
        .bind(message_text)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_due_subscriber_notify_outbox_items(
        &self,
        limit: i64,
    ) -> Result<Vec<SubscriberNotifyOutboxItem>> {
        let rows = sqlx::query(
            r"
            SELECT
                id,
                target_telegram_id,
                message_text,
                attempts
            FROM subscriber_notify_outbox
            WHERE next_retry_at <= CURRENT_TIMESTAMP
            ORDER BY next_retry_at ASC, id ASC
            LIMIT ?
            ",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            items.push(SubscriberNotifyOutboxItem {
                id: row.try_get("id")?,
                target_telegram_id: TelegramId::from(row.try_get::<i64, _>("target_telegram_id")?),
                message_text: row.try_get("message_text")?,
                attempts: row.try_get("attempts")?,
            });
        }
        Ok(items)
    }

    pub async fn mark_subscriber_notify_outbox_retry(
        &self,
        id: i64,
        last_error: &str,
        delay_seconds: u64,
    ) -> Result<()> {
        let delay = format!("+{delay_seconds} seconds");
        sqlx::query(
            r"
            UPDATE subscriber_notify_outbox
            SET attempts = attempts + 1,
                last_error = ?,
                next_retry_at = datetime('now', ?)
            WHERE id = ?
            ",
        )
        .bind(last_error)
        .bind(delay)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_subscriber_notify_outbox_item(&self, id: i64) -> Result<u64> {
        let result = sqlx::query("DELETE FROM subscriber_notify_outbox WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/infra_db_subscriber_notify_outbox.rs"]
mod tests;
