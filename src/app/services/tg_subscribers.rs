use crate::app::services::admin_bans as admin_bans_service;
use crate::app::services::subscriber_actions as subscriber_actions_service;
use crate::app::services::subscribers as subscribers_service;
use crate::core::types::TelegramId;
use crate::infra::db::Database;
use anyhow::Result;

pub async fn delete_subscriber(db: &Database, sub_id: TelegramId) -> Result<()> {
    subscriber_actions_service::delete_user(db, sub_id).await
}

pub async fn ban_subscriber(db: &Database, sub_id: TelegramId) -> Result<()> {
    let tt_user = subscribers_service::get_tt_username_by_telegram_id(db, sub_id).await?;
    admin_bans_service::add_ban(db, Some(sub_id), tt_user.clone(), "Admin Ban").await?;

    if let Err(e) = subscriber_actions_service::delete_user(db, sub_id).await {
        tracing::error!(
            telegram_id = sub_id.as_i64(),
            tt_username = ?tt_user,
            error = %e,
            "Partial failure during ban"
        );
    }

    Ok(())
}
