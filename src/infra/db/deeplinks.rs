use anyhow::Result;
use chrono::{Duration, Utc};

use super::{Database, types::Deeplink};

impl Database {
    pub async fn create_deeplink(
        &self,
        token: &str,
        action: crate::core::types::DeeplinkAction,
        payload: Option<&str>,
        expected_telegram_id: Option<i64>,
        ttl_seconds: i64,
    ) -> Result<()> {
        let expiry = Utc::now() + Duration::seconds(ttl_seconds);
        let action_str = action.to_string();
        sqlx::query!(
            "INSERT INTO deeplinks (token, action, payload, expected_telegram_id, expiry_time) VALUES (?, ?, ?, ?, ?)",
            token,
            action_str,
            payload,
            expected_telegram_id,
            expiry
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn resolve_deeplink(&self, token: &str) -> Result<Option<Deeplink>> {
        let dl = sqlx::query_as!(
            Deeplink,
            r#"
            SELECT
                token as "token!",
                action as "action!",
                payload,
                expected_telegram_id,
                expiry_time as "expiry_time!"
            FROM deeplinks WHERE token = ?
            "#,
            token
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(d) = dl {
            if d.expiry_time < Utc::now().naive_utc() {
                sqlx::query!("DELETE FROM deeplinks WHERE token = ?", token)
                    .execute(&self.pool)
                    .await?;
                return Ok(None);
            }
            sqlx::query!("DELETE FROM deeplinks WHERE token = ?", token)
                .execute(&self.pool)
                .await?;
            return Ok(Some(d));
        }
        Ok(None)
    }

    pub async fn cleanup_expired_deeplinks(&self) -> Result<u64> {
        let now = Utc::now().naive_utc();
        let res = sqlx::query!("DELETE FROM deeplinks WHERE expiry_time < ?", now)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }
}
