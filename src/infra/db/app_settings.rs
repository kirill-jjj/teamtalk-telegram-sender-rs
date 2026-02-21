use anyhow::Result;

use super::Database;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppSettingKey {
    ReplyQueueEnabledGlobal,
    FollowOwnerEnabledOverride,
    FollowOwnerPolicyOverride,
    FollowOwnerFallbackChannelOverride,
    FollowOwnerFallbackPasswordOverride,
}

impl AppSettingKey {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReplyQueueEnabledGlobal => "reply_queue_enabled_global",
            Self::FollowOwnerEnabledOverride => "follow_owner_enabled_override",
            Self::FollowOwnerPolicyOverride => "follow_owner_policy_override",
            Self::FollowOwnerFallbackChannelOverride => "follow_owner_fallback_channel_override",
            Self::FollowOwnerFallbackPasswordOverride => "follow_owner_fallback_password_override",
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
