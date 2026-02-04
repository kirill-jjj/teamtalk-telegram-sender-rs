use crate::core::types::{MuteListMode, TelegramId};
use anyhow::Result;

use super::Database;

impl Database {
    pub async fn update_mute_mode(
        &self,
        telegram_id: TelegramId,
        mode: MuteListMode,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE user_settings SET mute_list_mode = ? WHERE telegram_id = ?",
            mode,
            telegram_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_muted_users_list(
        &self,
        telegram_id: TelegramId,
        mode: MuteListMode,
    ) -> Result<Vec<crate::core::types::TtUsername>> {
        let rows: Vec<String> = sqlx::query_scalar!(
            "SELECT muted_teamtalk_username FROM muted_users WHERE user_settings_telegram_id = ? AND list_mode = ?",
            telegram_id,
            mode
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(crate::core::types::TtUsername::new)
            .collect())
    }

    pub async fn toggle_muted_user(
        &self,
        telegram_id: TelegramId,
        mode: MuteListMode,
        username: &crate::core::types::TtUsername,
    ) -> Result<()> {
        let username = username.as_str();
        let count: i32 = sqlx::query_scalar(
            "SELECT count(*) FROM muted_users WHERE user_settings_telegram_id = ? AND muted_teamtalk_username = ? AND list_mode = ?",
        )
        .bind(telegram_id)
        .bind(username)
        .bind(&mode)
        .fetch_one(&self.pool)
        .await?;
        let is_muted = count > 0;

        let query = if is_muted {
            "DELETE FROM muted_users WHERE user_settings_telegram_id = ? AND muted_teamtalk_username = ? AND list_mode = ?"
        } else {
            "INSERT INTO muted_users (user_settings_telegram_id, muted_teamtalk_username, list_mode) VALUES (?, ?, ?)"
        };

        sqlx::query(query)
            .bind(telegram_id)
            .bind(username)
            .bind(&mode)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/infra_db_mutes.rs"]
mod tests;
