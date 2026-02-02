use anyhow::Result;

use super::Database;

impl Database {
    pub async fn get_app_setting(&self, key: &str) -> Result<Option<String>> {
        let res = sqlx::query_scalar!("SELECT value FROM app_settings WHERE key = ?", key)
            .fetch_optional(&self.pool)
            .await?;
        Ok(res)
    }

    pub async fn set_app_setting(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query!(
            "INSERT INTO app_settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            key,
            value
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
