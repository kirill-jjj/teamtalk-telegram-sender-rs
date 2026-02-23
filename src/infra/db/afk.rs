use crate::core::types::{AfkListMode, TelegramId, TtUsername};
use anyhow::Result;
use sqlx::Row;

use super::{
    Database,
    app_settings::AppSettingKey,
    types::{AfkRecipient, AfkResolvedSettings, AfkUserSettings},
};

const DEFAULT_THRESHOLD_MINUTES: i64 = 10;
const DEFAULT_COOLDOWN_SECONDS: i64 = 0;

impl Database {
    pub async fn get_afk_user_settings(
        &self,
        telegram_id: TelegramId,
    ) -> Result<Option<AfkUserSettings>> {
        let row = sqlx::query(
            r"
            SELECT
                telegram_id,
                enabled,
                threshold_minutes,
                list_mode,
                cooldown_seconds
            FROM afk_user_settings
            WHERE telegram_id = ?
            ",
        )
        .bind(telegram_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let list_mode_raw: String = row.try_get("list_mode")?;
        let list_mode = AfkListMode::try_from(list_mode_raw.as_str()).unwrap_or(AfkListMode::None);

        Ok(Some(AfkUserSettings {
            enabled: row.try_get::<i64, _>("enabled")? != 0,
            threshold_minutes: row.try_get("threshold_minutes")?,
            list_mode,
            cooldown_seconds: row.try_get("cooldown_seconds")?,
        }))
    }

    pub async fn set_afk_enabled(&self, telegram_id: TelegramId, enabled: bool) -> Result<()> {
        self.ensure_afk_settings_row(telegram_id).await?;
        sqlx::query("UPDATE afk_user_settings SET enabled = ? WHERE telegram_id = ?")
            .bind(i64::from(enabled))
            .bind(telegram_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn set_afk_threshold_minutes(
        &self,
        telegram_id: TelegramId,
        minutes: i64,
    ) -> Result<()> {
        self.ensure_afk_settings_row(telegram_id).await?;
        sqlx::query("UPDATE afk_user_settings SET threshold_minutes = ? WHERE telegram_id = ?")
            .bind(minutes)
            .bind(telegram_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn set_afk_list_mode(
        &self,
        telegram_id: TelegramId,
        mode: AfkListMode,
    ) -> Result<()> {
        self.ensure_afk_settings_row(telegram_id).await?;
        sqlx::query("UPDATE afk_user_settings SET list_mode = ? WHERE telegram_id = ?")
            .bind(mode.to_string())
            .bind(telegram_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn set_afk_cooldown_seconds(
        &self,
        telegram_id: TelegramId,
        seconds: i64,
    ) -> Result<()> {
        self.ensure_afk_settings_row(telegram_id).await?;
        sqlx::query("UPDATE afk_user_settings SET cooldown_seconds = ? WHERE telegram_id = ?")
            .bind(seconds)
            .bind(telegram_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn toggle_afk_tracked_user(
        &self,
        telegram_id: TelegramId,
        mode: AfkListMode,
        username: &TtUsername,
    ) -> Result<()> {
        let mode_str = mode.to_string();
        let username = username.as_str();
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM afk_tracked_users WHERE user_settings_telegram_id = ? AND tt_username = ? AND list_mode = ?",
        )
        .bind(telegram_id)
        .bind(username)
        .bind(&mode_str)
        .fetch_one(&self.pool)
        .await?;

        if count > 0 {
            sqlx::query(
                "DELETE FROM afk_tracked_users WHERE user_settings_telegram_id = ? AND tt_username = ? AND list_mode = ?",
            )
            .bind(telegram_id)
            .bind(username)
            .bind(mode_str)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query(
                "INSERT INTO afk_tracked_users (user_settings_telegram_id, tt_username, list_mode) VALUES (?, ?, ?)",
            )
            .bind(telegram_id)
            .bind(username)
            .bind(mode_str)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_afk_tracked_users(
        &self,
        telegram_id: TelegramId,
        mode: AfkListMode,
    ) -> Result<Vec<TtUsername>> {
        let mode_str = mode.to_string();
        let rows: Vec<String> = sqlx::query_scalar(
            "SELECT tt_username FROM afk_tracked_users WHERE user_settings_telegram_id = ? AND list_mode = ? ORDER BY tt_username COLLATE NOCASE",
        )
        .bind(telegram_id)
        .bind(mode_str)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(TtUsername::new).collect())
    }

    pub async fn set_afk_threshold_override(
        &self,
        telegram_id: TelegramId,
        username: &TtUsername,
        minutes: i64,
    ) -> Result<()> {
        sqlx::query(
            r"
            INSERT INTO afk_threshold_overrides (user_settings_telegram_id, tt_username, threshold_minutes)
            VALUES (?, ?, ?)
            ON CONFLICT(user_settings_telegram_id, tt_username)
            DO UPDATE SET threshold_minutes = excluded.threshold_minutes
            ",
        )
        .bind(telegram_id)
        .bind(username.as_str())
        .bind(minutes)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_afk_threshold_override(
        &self,
        telegram_id: TelegramId,
        username: &TtUsername,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM afk_threshold_overrides WHERE user_settings_telegram_id = ? AND tt_username = ?",
        )
        .bind(telegram_id)
        .bind(username.as_str())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_afk_threshold_overrides(
        &self,
        telegram_id: TelegramId,
    ) -> Result<Vec<(TtUsername, i64)>> {
        let rows = sqlx::query(
            "SELECT tt_username, threshold_minutes FROM afk_threshold_overrides WHERE user_settings_telegram_id = ? ORDER BY tt_username COLLATE NOCASE",
        )
        .bind(telegram_id)
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let username: String = row.try_get("tt_username")?;
            let minutes: i64 = row.try_get("threshold_minutes")?;
            out.push((TtUsername::new(username), minutes));
        }
        Ok(out)
    }

    pub async fn get_afk_recipients_for_username(
        &self,
        tt_username: &TtUsername,
    ) -> Result<Vec<AfkRecipient>> {
        let username = tt_username.as_str();
        let rows = sqlx::query(
            r"
            SELECT
                su.telegram_id,
                COALESCE(au.enabled, CASE WHEN LOWER(COALESCE(ae.value, 'false')) = 'true' THEN 1 ELSE 0 END) AS enabled,
                COALESCE(ov.threshold_minutes,
                    COALESCE(au.threshold_minutes, CAST(COALESCE(at.value, '10') AS INTEGER), 10)
                ) AS threshold_minutes,
                COALESCE(au.cooldown_seconds, CAST(COALESCE(ac.value, '0') AS INTEGER), 0) AS cooldown_seconds,
                COALESCE(au.list_mode, COALESCE(al.value, 'none')) AS list_mode,
                mu.id AS list_match_id
            FROM subscribed_users su
            LEFT JOIN afk_user_settings au ON au.telegram_id = su.telegram_id
            LEFT JOIN app_settings ae ON ae.key = 'afk_default_enabled'
            LEFT JOIN app_settings at ON at.key = 'afk_default_threshold_minutes'
            LEFT JOIN app_settings al ON al.key = 'afk_default_list_mode'
            LEFT JOIN app_settings ac ON ac.key = 'afk_default_cooldown_seconds'
            LEFT JOIN afk_threshold_overrides ov
                ON ov.user_settings_telegram_id = su.telegram_id
                AND ov.tt_username = ?
            LEFT JOIN afk_tracked_users mu
                ON mu.user_settings_telegram_id = su.telegram_id
                AND mu.tt_username = ?
                AND mu.list_mode = COALESCE(au.list_mode, COALESCE(al.value, 'none'))
            ",
        )
        .bind(username)
        .bind(username)
        .fetch_all(&self.pool)
        .await?;

        let mut recipients = Vec::new();
        for row in rows {
            let enabled = row.try_get::<i64, _>("enabled")? != 0;
            if !enabled {
                continue;
            }
            let list_mode_raw: String = row.try_get("list_mode")?;
            let list_mode =
                AfkListMode::try_from(list_mode_raw.as_str()).unwrap_or(AfkListMode::None);
            let list_match_id: Option<i64> = row.try_get("list_match_id")?;

            let list_allowed = match list_mode {
                AfkListMode::None => true,
                AfkListMode::Blacklist => list_match_id.is_none(),
                AfkListMode::Whitelist => list_match_id.is_some(),
            };
            if !list_allowed {
                continue;
            }

            recipients.push(AfkRecipient {
                telegram_id: row.try_get("telegram_id")?,
                threshold_minutes: row.try_get("threshold_minutes")?,
                cooldown_seconds: row.try_get("cooldown_seconds")?,
            });
        }

        Ok(recipients)
    }

    pub async fn resolve_afk_settings_for_user(
        &self,
        telegram_id: TelegramId,
    ) -> Result<AfkResolvedSettings> {
        let default_enabled = self
            .get_app_setting(AppSettingKey::AfkDefaultEnabled)
            .await?
            .unwrap_or_else(|| "false".to_string());
        let default_threshold = self
            .get_app_setting(AppSettingKey::AfkDefaultThresholdMinutes)
            .await?
            .unwrap_or_else(|| DEFAULT_THRESHOLD_MINUTES.to_string())
            .parse::<i64>()
            .unwrap_or(DEFAULT_THRESHOLD_MINUTES);
        let default_list_mode = self
            .get_app_setting(AppSettingKey::AfkDefaultListMode)
            .await?
            .and_then(|v| AfkListMode::try_from(v.as_str()).ok())
            .unwrap_or(AfkListMode::None);
        let default_cooldown = self
            .get_app_setting(AppSettingKey::AfkDefaultCooldownSeconds)
            .await?
            .unwrap_or_else(|| DEFAULT_COOLDOWN_SECONDS.to_string())
            .parse::<i64>()
            .unwrap_or(DEFAULT_COOLDOWN_SECONDS);

        if let Some(current) = self.get_afk_user_settings(telegram_id).await? {
            return Ok(AfkResolvedSettings {
                enabled: current.enabled,
                threshold_minutes: current.threshold_minutes,
                list_mode: current.list_mode,
                cooldown_seconds: current.cooldown_seconds,
            });
        }

        Ok(AfkResolvedSettings {
            enabled: default_enabled.eq_ignore_ascii_case("true"),
            threshold_minutes: default_threshold,
            list_mode: default_list_mode,
            cooldown_seconds: default_cooldown,
        })
    }

    async fn ensure_afk_settings_row(&self, telegram_id: TelegramId) -> Result<()> {
        sqlx::query("INSERT OR IGNORE INTO afk_user_settings (telegram_id) VALUES (?)")
            .bind(telegram_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
