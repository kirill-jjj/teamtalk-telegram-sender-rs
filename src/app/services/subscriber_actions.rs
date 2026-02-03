use crate::core::types::{MuteListMode, NotificationSetting, TtUsername};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait SubscriberActionsRepo: Sync {
    async fn delete_user_profile(&self, telegram_id: i64) -> Result<()>;
    async fn unlink_tt_account(&self, telegram_id: i64) -> Result<()>;
    async fn link_tt_account(&self, telegram_id: i64, username: &TtUsername) -> Result<()>;
    async fn update_notification_setting(
        &self,
        telegram_id: i64,
        setting: NotificationSetting,
    ) -> Result<()>;
    async fn update_mute_mode(&self, telegram_id: i64, mode: MuteListMode) -> Result<()>;
}

pub async fn delete_user(db: &impl SubscriberActionsRepo, telegram_id: i64) -> Result<()> {
    db.delete_user_profile(telegram_id).await
}

pub async fn unlink_tt(db: &impl SubscriberActionsRepo, telegram_id: i64) -> Result<()> {
    db.unlink_tt_account(telegram_id).await
}

pub async fn link_tt(
    db: &impl SubscriberActionsRepo,
    telegram_id: i64,
    username: &TtUsername,
) -> Result<()> {
    db.link_tt_account(telegram_id, username).await
}

pub async fn update_notifications(
    db: &impl SubscriberActionsRepo,
    telegram_id: i64,
    setting: NotificationSetting,
) -> Result<()> {
    db.update_notification_setting(telegram_id, setting).await
}

pub async fn update_mute_mode(
    db: &impl SubscriberActionsRepo,
    telegram_id: i64,
    mode: MuteListMode,
) -> Result<()> {
    db.update_mute_mode(telegram_id, mode).await
}

#[async_trait]
impl SubscriberActionsRepo for crate::infra::db::Database {
    async fn delete_user_profile(&self, telegram_id: i64) -> Result<()> {
        self.delete_user_profile(telegram_id).await
    }

    async fn unlink_tt_account(&self, telegram_id: i64) -> Result<()> {
        self.unlink_tt_account(telegram_id).await
    }

    async fn link_tt_account(&self, telegram_id: i64, username: &TtUsername) -> Result<()> {
        self.link_tt_account(telegram_id, username).await
    }

    async fn update_notification_setting(
        &self,
        telegram_id: i64,
        setting: NotificationSetting,
    ) -> Result<()> {
        self.update_notification_setting(telegram_id, setting).await
    }

    async fn update_mute_mode(&self, telegram_id: i64, mode: MuteListMode) -> Result<()> {
        self.update_mute_mode(telegram_id, mode).await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app_subscriber_actions.rs"]
mod tests;
