use crate::types::NotificationSetting;
use anyhow::Result;

use super::{Database, types::UserSettings};

impl Database {
    pub async fn get_or_create_user(
        &self,
        telegram_id: i64,
        default_lang: &str,
    ) -> Result<UserSettings> {
        let user = sqlx::query_as!(
            UserSettings,
            r#"
            SELECT
                telegram_id as "telegram_id!",
                language_code as "language_code!",
                notification_settings as "notification_settings!",
                mute_list_mode as "mute_list_mode!",
                teamtalk_username,
                not_on_online_enabled as "not_on_online_enabled!",
                not_on_online_confirmed as "not_on_online_confirmed!"
            FROM user_settings
            WHERE telegram_id = ?
            "#,
            telegram_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(u) = user {
            Ok(u)
        } else {
            sqlx::query!(
                "INSERT INTO user_settings (telegram_id, language_code) VALUES (?, ?)",
                telegram_id,
                default_lang
            )
            .execute(&self.pool)
            .await?;

            Ok(UserSettings {
                telegram_id,
                language_code: default_lang.to_string(),
                notification_settings: "all".to_string(),
                mute_list_mode: "blacklist".to_string(),
                teamtalk_username: None,
                not_on_online_enabled: false,
                not_on_online_confirmed: false,
            })
        }
    }

    pub async fn get_user_lang_by_tt_user(&self, tt_username: &str) -> Option<String> {
        let res: Option<String> = match sqlx::query_scalar!(
            "SELECT language_code FROM user_settings WHERE teamtalk_username = ?",
            tt_username
        )
        .fetch_optional(&self.pool)
        .await
        {
            Ok(res) => res,
            Err(e) => {
                log::error!(
                    "Failed to get user lang for tt_user '{}': {}",
                    tt_username,
                    e
                );
                None
            }
        };

        res
    }

    pub async fn update_notification_setting(
        &self,
        telegram_id: i64,
        setting: NotificationSetting,
    ) -> Result<()> {
        let setting_str = setting.to_string();
        sqlx::query!(
            "UPDATE user_settings SET notification_settings = ? WHERE telegram_id = ?",
            setting_str,
            telegram_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_language(&self, telegram_id: i64, lang: &str) -> Result<()> {
        sqlx::query!(
            "UPDATE user_settings SET language_code = ? WHERE telegram_id = ?",
            lang,
            telegram_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn toggle_noon(&self, telegram_id: i64) -> Result<bool> {
        let mut tx = self.pool.begin().await?;

        let current_val: i64 = sqlx::query_scalar!(
            "SELECT CAST(not_on_online_enabled AS INTEGER) FROM user_settings WHERE telegram_id = ?",
            telegram_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .unwrap_or(0);

        let new_bool = current_val == 0;
        let new_int = if new_bool { 1 } else { 0 };

        log::debug!(
            "[DB] Toggling NOON for {}: current={}, new_bool={}",
            telegram_id,
            current_val,
            new_bool
        );

        sqlx::query!(
            "UPDATE user_settings SET not_on_online_enabled = ? WHERE telegram_id = ?",
            new_int,
            telegram_id
        )
        .execute(&mut *tx)
        .await?;

        if new_bool {
            sqlx::query!(
                "UPDATE user_settings SET not_on_online_confirmed = 1 WHERE telegram_id = ?",
                telegram_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(new_bool)
    }

    pub async fn link_tt_account(&self, telegram_id: i64, tt_username: &str) -> Result<()> {
        sqlx::query!("UPDATE user_settings SET teamtalk_username = ?, not_on_online_confirmed = 1 WHERE telegram_id = ?", tt_username, telegram_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn unlink_tt_account(&self, telegram_id: i64) -> Result<()> {
        sqlx::query!(
            "UPDATE user_settings SET teamtalk_username = NULL WHERE telegram_id = ?",
            telegram_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_user_profile(&self, telegram_id: i64) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "DELETE FROM subscribed_users WHERE telegram_id = ?",
            telegram_id
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!("DELETE FROM admins WHERE telegram_id = ?", telegram_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query!(
            "DELETE FROM muted_users WHERE user_settings_telegram_id = ?",
            telegram_id
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            "DELETE FROM user_settings WHERE telegram_id = ?",
            telegram_id
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
}
