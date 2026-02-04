use crate::app::services::subscriber_actions::{
    SubscriberActionsRepo, delete_user, link_tt, unlink_tt, update_mute_mode, update_notifications,
};
use crate::core::types::TelegramId;
use crate::core::types::{MuteListMode, NotificationSetting, TtUsername};
use anyhow::Result;
use async_trait::async_trait;

#[derive(Default)]
struct FakeSubscriberRepo;

#[async_trait]
impl SubscriberActionsRepo for FakeSubscriberRepo {
    async fn delete_user_profile(&self, _telegram_id: TelegramId) -> Result<()> {
        Ok(())
    }

    async fn unlink_tt_account(&self, _telegram_id: TelegramId) -> Result<()> {
        Ok(())
    }

    async fn link_tt_account(
        &self,
        _telegram_id: TelegramId,
        _username: &TtUsername,
    ) -> Result<()> {
        Ok(())
    }

    async fn update_notification_setting(
        &self,
        _telegram_id: TelegramId,
        _setting: NotificationSetting,
    ) -> Result<()> {
        Ok(())
    }

    async fn update_mute_mode(&self, _telegram_id: TelegramId, _mode: MuteListMode) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn subscriber_actions_delegates() {
    let repo = FakeSubscriberRepo;
    delete_user(&repo, TelegramId::from(1)).await.unwrap();
    unlink_tt(&repo, TelegramId::from(1)).await.unwrap();
    link_tt(&repo, TelegramId::from(1), &TtUsername::new("tt"))
        .await
        .unwrap();
    update_notifications(&repo, TelegramId::from(1), NotificationSetting::All)
        .await
        .unwrap();
    update_mute_mode(&repo, TelegramId::from(1), MuteListMode::Whitelist)
        .await
        .unwrap();
}
