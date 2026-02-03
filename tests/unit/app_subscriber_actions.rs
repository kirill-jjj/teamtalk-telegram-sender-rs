use crate::app::services::subscriber_actions::{
    SubscriberActionsRepo, delete_user, link_tt, unlink_tt, update_mute_mode, update_notifications,
};
use crate::core::types::{MuteListMode, NotificationSetting, TtUsername};
use anyhow::Result;

#[derive(Default)]
struct FakeSubscriberRepo;

#[allow(async_fn_in_trait)]
impl SubscriberActionsRepo for FakeSubscriberRepo {
    async fn delete_user_profile(&self, _telegram_id: i64) -> Result<()> {
        Ok(())
    }

    async fn unlink_tt_account(&self, _telegram_id: i64) -> Result<()> {
        Ok(())
    }

    async fn link_tt_account(&self, _telegram_id: i64, _username: &TtUsername) -> Result<()> {
        Ok(())
    }

    async fn update_notification_setting(
        &self,
        _telegram_id: i64,
        _setting: NotificationSetting,
    ) -> Result<()> {
        Ok(())
    }

    async fn update_mute_mode(&self, _telegram_id: i64, _mode: MuteListMode) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn subscriber_actions_delegates() {
    let repo = FakeSubscriberRepo;
    delete_user(&repo, 1).await.unwrap();
    unlink_tt(&repo, 1).await.unwrap();
    link_tt(&repo, 1, &TtUsername::new("tt")).await.unwrap();
    update_notifications(&repo, 1, NotificationSetting::All)
        .await
        .unwrap();
    update_mute_mode(&repo, 1, MuteListMode::Whitelist)
        .await
        .unwrap();
}
