use crate::core::types::MuteListMode;
use anyhow::Result;

use super::Database;

impl Database {
    pub async fn update_mute_mode(&self, telegram_id: i64, mode: MuteListMode) -> Result<()> {
        let mode_str = mode.to_string();
        sqlx::query!(
            "UPDATE user_settings SET mute_list_mode = ? WHERE telegram_id = ?",
            mode_str,
            telegram_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_muted_users_list(&self, telegram_id: i64) -> Result<Vec<String>> {
        let rows = sqlx::query_scalar!(
            "SELECT muted_teamtalk_username FROM muted_users WHERE user_settings_telegram_id = ?",
            telegram_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn toggle_muted_user(&self, telegram_id: i64, username: &str) -> Result<()> {
        let count: i32 = sqlx::query_scalar(
            "SELECT count(*) FROM muted_users WHERE user_settings_telegram_id = ? AND muted_teamtalk_username = ?",
        )
        .bind(telegram_id)
        .bind(username)
        .fetch_one(&self.pool)
        .await?;
        let is_muted = count > 0;

        let query = if is_muted {
            "DELETE FROM muted_users WHERE user_settings_telegram_id = ? AND muted_teamtalk_username = ?"
        } else {
            "INSERT INTO muted_users (user_settings_telegram_id, muted_teamtalk_username) VALUES (?, ?)"
        };

        sqlx::query(query)
            .bind(telegram_id)
            .bind(username)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/infra_db_mutes.rs"]
mod tests;
