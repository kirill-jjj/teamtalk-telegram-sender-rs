use anyhow::Result;

use super::Database;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppSettingKey {
    ReplyQueueEnabledGlobal,
    AfkDefaultEnabled,
    AfkDefaultThresholdMinutes,
    AfkDefaultListMode,
    AfkDefaultCooldownSeconds,
}

impl AppSettingKey {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReplyQueueEnabledGlobal => "reply_queue_enabled_global",
            Self::AfkDefaultEnabled => "afk_default_enabled",
            Self::AfkDefaultThresholdMinutes => "afk_default_threshold_minutes",
            Self::AfkDefaultListMode => "afk_default_list_mode",
            Self::AfkDefaultCooldownSeconds => "afk_default_cooldown_seconds",
        }
    }
}

impl Database {
    pub async fn get_app_setting(&self, key: AppSettingKey) -> Result<Option<String>> {
        let key = key.as_str();
        let res = sqlx::query_scalar!("SELECT value FROM app_settings WHERE key = ?", key)
            .fetch_optional(&self.pool)
            .await?;
        Ok(res)
    }

    pub async fn set_app_setting(&self, key: AppSettingKey, value: &str) -> Result<()> {
        let key = key.as_str();
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
