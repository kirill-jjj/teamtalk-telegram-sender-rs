use super::*;
use crate::app::services::user_settings::{UserSettingsRepo, get_or_create};
use crate::core::types::LanguageCode;
use crate::infra::db::types::UserSettings;
use anyhow::Result;

#[test]
fn parse_notification_setting_fallback() {
    assert_eq!(parse_notification_setting(""), NotificationSetting::All);
    assert_eq!(
        parse_notification_setting("unknown"),
        NotificationSetting::All
    );
}

#[test]
fn parse_mute_list_mode_fallback() {
    assert_eq!(parse_mute_list_mode(""), MuteListMode::Blacklist);
    assert_eq!(parse_mute_list_mode("unknown"), MuteListMode::Blacklist);
}

#[derive(Default)]
struct FakeUserSettingsRepo;

#[allow(async_fn_in_trait)]
impl UserSettingsRepo for FakeUserSettingsRepo {
    async fn get_or_create_user(
        &self,
        telegram_id: i64,
        default_lang: LanguageCode,
    ) -> Result<UserSettings> {
        Ok(UserSettings {
            telegram_id,
            language_code: default_lang.as_str().to_string(),
            notification_settings: "all".to_string(),
            mute_list_mode: "blacklist".to_string(),
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
    let user = get_or_create(&repo, 7, LanguageCode::Ru).await.unwrap();
    assert_eq!(user.telegram_id, 7);
    assert_eq!(user.language_code, "ru");
}
