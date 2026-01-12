use crate::core::types::{LanguageCode, NotificationSetting};
use anyhow::Result;

use super::{Database, types::UserSettings};

impl Database {
    pub async fn get_or_create_user(
        &self,
        telegram_id: i64,
        default_lang: LanguageCode,
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
            let default_lang_code = default_lang.as_str();
            sqlx::query!(
                "INSERT OR IGNORE INTO user_settings (telegram_id, language_code) VALUES (?, ?)",
                telegram_id,
                default_lang_code
            )
            .execute(&self.pool)
            .await?;

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
            .fetch_one(&self.pool)
            .await?;

            Ok(user)
        }
    }

    pub async fn get_user_lang_by_tt_user(&self, tt_username: &str) -> Option<LanguageCode> {
        let res: Option<String> = match sqlx::query_scalar!(
            "SELECT language_code FROM user_settings WHERE teamtalk_username = ?",
            tt_username
        )
        .fetch_optional(&self.pool)
        .await
        {
            Ok(res) => res,
            Err(e) => {
                tracing::error!(
                    "Failed to get user lang for tt_user '{}': {}",
                    tt_username,
                    e
                );
                None
            }
        };

        res.and_then(|lang| LanguageCode::try_from(lang.as_str()).ok())
    }

    pub async fn get_telegram_id_by_tt_user(&self, tt_username: &str) -> Option<i64> {
        match sqlx::query_scalar!(
            "SELECT telegram_id FROM user_settings WHERE teamtalk_username = ?",
            tt_username
        )
        .fetch_optional(&self.pool)
        .await
        {
            Ok(res) => res.flatten(),
            Err(e) => {
                tracing::error!(
                    "Failed to get telegram_id for tt_user '{}': {}",
                    tt_username,
                    e
                );
                None
            }
        }
    }

    pub async fn get_tt_username_by_telegram_id(&self, telegram_id: i64) -> Result<Option<String>> {
        let res: Option<String> = match sqlx::query_scalar!(
            "SELECT teamtalk_username FROM user_settings WHERE telegram_id = ?",
            telegram_id
        )
        .fetch_optional(&self.pool)
        .await
        {
            Ok(res) => res.flatten(),
            Err(e) => {
                tracing::error!(
                    "Failed to get tt_user for telegram_id '{}': {}",
                    telegram_id,
                    e
                );
                None
            }
        };

        Ok(res)
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

    pub async fn update_language(&self, telegram_id: i64, lang: LanguageCode) -> Result<()> {
        let lang_code = lang.as_str();
        sqlx::query!(
            "UPDATE user_settings SET language_code = ? WHERE telegram_id = ?",
            lang_code,
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

        tracing::debug!(
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
