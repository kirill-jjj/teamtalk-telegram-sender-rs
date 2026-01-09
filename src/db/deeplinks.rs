use anyhow::Result;
use chrono::{Duration, Utc};

use super::{Database, types::Deeplink};

impl Database {
    pub async fn create_deeplink(
        &self,
        token: &str,
        action: &str,
        payload: Option<&str>,
        ttl_seconds: i64,
    ) -> Result<()> {
        let expiry = Utc::now() + Duration::seconds(ttl_seconds);
        sqlx::query!(
            "INSERT INTO deeplinks (token, action, payload, expiry_time) VALUES (?, ?, ?, ?)",
            token,
            action,
            payload,
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
}
