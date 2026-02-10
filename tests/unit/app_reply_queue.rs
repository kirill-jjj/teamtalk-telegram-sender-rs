use crate::app::services::reply_queue::{
    ReplyQueueRepo, get_reply_queue_global_enabled, get_reply_queue_user_enabled,
    is_reply_queue_enabled_for_tt_user,
};
use crate::core::types::TelegramId;
use crate::core::types::{LanguageCode, MuteListMode, NotificationSetting, TtUsername};
use crate::infra::db::app_settings::AppSettingKey;
use crate::infra::db::types::UserSettings;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{Duration, Utc};

#[derive(Default)]
struct FakeReplyQueueRepo {
    global_value: Option<String>,
    user_enabled: bool,
    tt_to_tg: Option<TelegramId>,
}

#[async_trait]
impl ReplyQueueRepo for FakeReplyQueueRepo {
    async fn get_app_setting(&self, _key: AppSettingKey) -> Result<Option<String>> {
        Ok(self.global_value.clone())
    }

    async fn set_app_setting(&self, _key: AppSettingKey, _value: &str) -> Result<()> {
        Ok(())
    }

    async fn get_or_create_user(
        &self,
        telegram_id: TelegramId,
        _default_lang: LanguageCode,
    ) -> Result<UserSettings> {
        Ok(UserSettings {
            telegram_id,
            language_code: LanguageCode::En,
            notification_settings: NotificationSetting::All,
            mute_list_mode: MuteListMode::Blacklist,
            teamtalk_username: None,
            not_on_online_enabled: false,
            not_on_online_confirmed: false,
            reply_queue_enabled: self.user_enabled,
            admin_sub_events_enabled: false,
        })
    }

    async fn update_reply_queue_enabled(
        &self,
        _telegram_id: TelegramId,
        _enabled: bool,
    ) -> Result<()> {
        Ok(())
    }

    async fn fetch_telegram_id_by_tt_user(&self, _tt_username: &TtUsername) -> Option<TelegramId> {
        self.tt_to_tg
    }

    async fn add_reply_queue_item(
        &self,
        _tt_username: &TtUsername,
        _admin_telegram_id: TelegramId,
        _message_text: &str,
    ) -> Result<()> {
        Ok(())
    }

    async fn get_reply_queue_for_user(
        &self,
        _tt_username: &TtUsername,
    ) -> Result<Vec<crate::infra::db::reply_queue::ReplyQueueItem>> {
        Ok(Vec::new())
    }

    async fn delete_reply_queue_ids(
        &self,
        _ids: &[crate::core::types::DbReplyQueueId],
    ) -> Result<u64> {
        Ok(0)
    }

    async fn clear_reply_queue_for_user(&self, _tt_username: &TtUsername) -> Result<u64> {
        Ok(0)
    }

    async fn clear_reply_queue_all(&self) -> Result<u64> {
        Ok(0)
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
    assert!(
        get_reply_queue_user_enabled(&repo, TelegramId::from(1), LanguageCode::En)
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn reply_queue_disabled_when_global_off() {
    let repo = FakeReplyQueueRepo {
        global_value: Some("0".to_string()),
        user_enabled: true,
        tt_to_tg: Some(TelegramId::from(42)),
    };
    let tt_username = TtUsername::new("tt");
    assert!(
        !is_reply_queue_enabled_for_tt_user(&repo, &tt_username)
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
    let tt_username = TtUsername::new("tt");
    assert!(
        !is_reply_queue_enabled_for_tt_user(&repo, &tt_username)
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn reply_queue_enabled_when_global_and_user_on() {
    let repo = FakeReplyQueueRepo {
        global_value: Some("1".to_string()),
        user_enabled: true,
        tt_to_tg: Some(TelegramId::from(7)),
    };
    let tt_username = TtUsername::new("tt");
    assert!(
        is_reply_queue_enabled_for_tt_user(&repo, &tt_username)
            .await
            .unwrap()
    );
}

#[test]
fn format_queue_message_minutes_en() {
    let now = Utc::now();
    let created = now - Duration::minutes(5);
    let msg = super::format_queue_message(LanguageCode::En, created.naive_utc(), now, "hello");
    assert!(msg.contains("5 minutes"));
    assert!(msg.contains("hello"));
}

#[test]
fn format_queue_message_hours_en() {
    let now = Utc::now();
    let created = now - Duration::hours(3);
    let msg = super::format_queue_message(LanguageCode::En, created.naive_utc(), now, "hello");
    assert!(msg.contains("3 hours"));
    assert!(msg.contains("hello"));
}

#[test]
fn format_queue_message_minutes_ru() {
    let now = Utc::now();
    let created = now - Duration::minutes(2);
    let msg = super::format_queue_message(LanguageCode::Ru, created.naive_utc(), now, "привет");
    assert!(msg.contains('2'));
    assert!(msg.contains("привет"));
}

#[test]
fn queue_items_sorted_orders_by_time_and_id() {
    let mut items = vec![
        crate::infra::db::reply_queue::ReplyQueueItem {
            id: crate::core::types::DbReplyQueueId::from(2),
            message_text: "b".to_string(),
            created_at: chrono::DateTime::from_timestamp(10, 0).unwrap().naive_utc(),
        },
        crate::infra::db::reply_queue::ReplyQueueItem {
            id: crate::core::types::DbReplyQueueId::from(1),
            message_text: "a".to_string(),
            created_at: chrono::DateTime::from_timestamp(10, 0).unwrap().naive_utc(),
        },
        crate::infra::db::reply_queue::ReplyQueueItem {
            id: crate::core::types::DbReplyQueueId::from(3),
            message_text: "c".to_string(),
            created_at: chrono::DateTime::from_timestamp(5, 0).unwrap().naive_utc(),
        },
    ];
    super::queue_items_sorted(&mut items);
    let ids: Vec<i64> = items.iter().map(|item| item.id.as_i64()).collect();
    assert_eq!(ids, vec![3, 1, 2]);
}
