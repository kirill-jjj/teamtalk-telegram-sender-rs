use crate::core::types::{MuteListMode, NotificationSetting};
use crate::infra::db::Database;
use anyhow::Result;

pub async fn delete_user(db: &Database, telegram_id: i64) -> Result<()> {
    db.delete_user_profile(telegram_id).await
}

pub async fn unlink_tt(db: &Database, telegram_id: i64) -> Result<()> {
    db.unlink_tt_account(telegram_id).await
}

pub async fn link_tt(db: &Database, telegram_id: i64, username: &str) -> Result<()> {
    db.link_tt_account(telegram_id, username).await
}

pub async fn update_notifications(
    db: &Database,
    telegram_id: i64,
    setting: NotificationSetting,
) -> Result<()> {
    db.update_notification_setting(telegram_id, setting).await
}

pub async fn update_mute_mode(db: &Database, telegram_id: i64, mode: MuteListMode) -> Result<()> {
    db.update_mute_mode(telegram_id, mode).await
}
