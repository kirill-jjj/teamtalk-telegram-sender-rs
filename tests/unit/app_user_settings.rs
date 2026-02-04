use crate::app::services::user_settings::{UserSettingsRepo, get_or_create};
use crate::core::types::TelegramId;
use crate::core::types::{LanguageCode, MuteListMode, NotificationSetting};
use crate::infra::db::types::UserSettings;
use anyhow::Result;
use async_trait::async_trait;

#[derive(Default)]
struct FakeUserSettingsRepo;

#[async_trait]
impl UserSettingsRepo for FakeUserSettingsRepo {
    async fn get_or_create_user(
        &self,
        telegram_id: TelegramId,
        default_lang: LanguageCode,
    ) -> Result<UserSettings> {
        Ok(UserSettings {
            telegram_id,
            language_code: default_lang,
            notification_settings: NotificationSetting::All,
            mute_list_mode: MuteListMode::Blacklist,
            teamtalk_username: None,
            not_on_online_enabled: false,
            not_on_online_confirmed: false,
            reply_queue_enabled: false,
        })
    }
}

#[tokio::test]
async fn get_or_create_delegates() {
    let repo = FakeUserSettingsRepo;
    let user = get_or_create(&repo, TelegramId::from(7), LanguageCode::Ru)
        .await
        .unwrap();
    assert_eq!(user.telegram_id, TelegramId::from(7));
    assert_eq!(user.language_code, LanguageCode::Ru);
}
