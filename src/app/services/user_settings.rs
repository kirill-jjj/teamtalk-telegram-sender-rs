use crate::core::types::{LanguageCode, MuteListMode, NotificationSetting};
use anyhow::Result;

pub async fn get_or_create(
    db: &impl UserSettingsRepo,
    telegram_id: i64,
    default_lang: LanguageCode,
) -> Result<crate::infra::db::types::UserSettings> {
    db.get_or_create_user(telegram_id, default_lang).await
}

#[allow(async_fn_in_trait)]
pub trait UserSettingsRepo: Sync {
    async fn get_or_create_user(
        &self,
        telegram_id: i64,
        default_lang: LanguageCode,
    ) -> Result<crate::infra::db::types::UserSettings>;
}

impl UserSettingsRepo for crate::infra::db::Database {
    async fn get_or_create_user(
        &self,
        telegram_id: i64,
        default_lang: LanguageCode,
    ) -> Result<crate::infra::db::types::UserSettings> {
        self.get_or_create_user(telegram_id, default_lang).await
    }
}

pub fn parse_notification_setting(raw: &str) -> NotificationSetting {
    NotificationSetting::try_from(raw).unwrap_or(NotificationSetting::All)
}

pub fn parse_mute_list_mode(raw: &str) -> MuteListMode {
    MuteListMode::try_from(raw).unwrap_or(MuteListMode::Blacklist)
}

#[cfg(test)]
#[path = "../../../tests/unit/app_user_settings.rs"]
mod tests;
