use crate::app::services::reply_queue::{
    ReplyQueueRepo, get_reply_queue_global_enabled, get_reply_queue_user_enabled,
    is_reply_queue_enabled_for_tt_user,
};
use crate::core::types::LanguageCode;
use crate::infra::db::types::UserSettings;
use anyhow::Result;

#[derive(Default)]
struct FakeReplyQueueRepo {
    global_value: Option<String>,
    user_enabled: bool,
    tt_to_tg: Option<i64>,
}

#[allow(async_fn_in_trait)]
impl ReplyQueueRepo for FakeReplyQueueRepo {
    async fn get_app_setting(&self, _key: &str) -> Result<Option<String>> {
        Ok(self.global_value.clone())
    }

    async fn set_app_setting(&self, _key: &str, _value: &str) -> Result<()> {
        Ok(())
    }

    async fn get_or_create_user(
        &self,
        telegram_id: i64,
        _default_lang: LanguageCode,
    ) -> Result<UserSettings> {
        Ok(UserSettings {
            telegram_id,
            language_code: "en".to_string(),
            notification_settings: "all".to_string(),
            mute_list_mode: "blacklist".to_string(),
            teamtalk_username: None,
            not_on_online_enabled: false,
            not_on_online_confirmed: false,
            reply_queue_enabled: self.user_enabled,
        })
    }

    async fn update_reply_queue_enabled(&self, _telegram_id: i64, _enabled: bool) -> Result<()> {
        Ok(())
    }

    async fn get_telegram_id_by_tt_user(&self, _tt_username: &str) -> Option<i64> {
        self.tt_to_tg
    }
}

#[tokio::test]
async fn global_enabled_accepts_true_values() {
    let repo = FakeReplyQueueRepo {
        global_value: Some("true".to_string()),
        user_enabled: false,
        tt_to_tg: None,
    };
    assert!(get_reply_queue_global_enabled(&repo).await.unwrap());
}

#[tokio::test]
async fn global_enabled_rejects_false_values() {
    let repo = FakeReplyQueueRepo {
        global_value: Some("0".to_string()),
        user_enabled: false,
        tt_to_tg: None,
    };
    assert!(!get_reply_queue_global_enabled(&repo).await.unwrap());
}

#[tokio::test]
async fn user_enabled_reads_settings() {
    let repo = FakeReplyQueueRepo {
        global_value: None,
        user_enabled: true,
        tt_to_tg: None,
    };
    assert!(get_reply_queue_user_enabled(&repo, 1).await.unwrap());
}

#[tokio::test]
async fn reply_queue_disabled_when_global_off() {
    let repo = FakeReplyQueueRepo {
        global_value: Some("0".to_string()),
        user_enabled: true,
        tt_to_tg: Some(42),
    };
    assert!(
        !is_reply_queue_enabled_for_tt_user(&repo, "tt")
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn reply_queue_disabled_without_link() {
    let repo = FakeReplyQueueRepo {
        global_value: Some("1".to_string()),
        user_enabled: true,
        tt_to_tg: None,
    };
    assert!(
        !is_reply_queue_enabled_for_tt_user(&repo, "tt")
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn reply_queue_enabled_when_global_and_user_on() {
    let repo = FakeReplyQueueRepo {
        global_value: Some("1".to_string()),
        user_enabled: true,
        tt_to_tg: Some(7),
    };
    assert!(
        is_reply_queue_enabled_for_tt_user(&repo, "tt")
            .await
            .unwrap()
    );
}
