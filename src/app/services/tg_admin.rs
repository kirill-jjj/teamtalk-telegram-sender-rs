use crate::app::services::admin_bans as admin_bans_service;
use crate::app::services::admin_cleanup as admin_cleanup_service;
use crate::app::services::subscribers as subscribers_service;
use crate::app::state::StateHandle;
use crate::core::types::{LiteUser, TelegramId, TtUserId};
use crate::infra::db::Database;
use anyhow::Result;
use teloxide_ng::prelude::Requester;

#[derive(Debug)]
pub enum AdminError {
    Notify(anyhow::Error),
    Silent(anyhow::Error),
}

impl AdminError {
    pub const fn should_notify(&self) -> bool {
        matches!(self, Self::Notify(_))
    }

    pub fn into_error(self) -> anyhow::Error {
        match self {
            Self::Notify(e) | Self::Silent(e) => e,
        }
    }
}

pub async fn list_online_users(state: &StateHandle) -> Result<Vec<LiteUser>, AdminError> {
    state
        .online_users_sorted()
        .await
        .map_err(|e| AdminError::Silent(e.into()))
}

pub async fn online_user_by_id(
    state: &StateHandle,
    user_id: TtUserId,
) -> Result<Option<LiteUser>, AdminError> {
    state
        .online_user_by_id(user_id)
        .await
        .map_err(|e| AdminError::Silent(e.into()))
}

pub async fn list_subscribers(
    db: &Database,
) -> Result<Vec<crate::infra::db::types::SubscriberInfo>, AdminError> {
    subscribers_service::get_subscribers(db)
        .await
        .map_err(AdminError::Notify)
}

pub async fn list_ban_entries(
    db: &Database,
) -> Result<Vec<crate::infra::db::types::BanEntry>, AdminError> {
    crate::app::services::bans::get_banned_users(db)
        .await
        .map_err(AdminError::Notify)
}

pub async fn list_admins(db: &Database) -> Result<Vec<TelegramId>, AdminError> {
    db.get_all_admins().await.map_err(AdminError::Notify)
}

pub async fn remove_ban(
    db: &Database,
    ban_db_id: crate::core::types::DbBanId,
) -> Result<(), AdminError> {
    crate::app::services::admin_bans::remove_ban_by_id(db, ban_db_id)
        .await
        .map_err(AdminError::Notify)
}

pub async fn add_admin(db: &Database, telegram_id: TelegramId) -> Result<(), AdminError> {
    crate::app::services::admins::add_admin(db, telegram_id)
        .await
        .map_err(AdminError::Notify)
}

pub async fn remove_admin(db: &Database, telegram_id: TelegramId) -> Result<(), AdminError> {
    crate::app::services::admins::remove_admin(db, telegram_id)
        .await
        .map_err(AdminError::Notify)
}

pub async fn is_admin(
    db: &Database,
    config: &crate::bootstrap::config::Config,
    telegram_id: TelegramId,
) -> bool {
    crate::app::services::admins::is_admin(db, config, telegram_id).await
}

pub async fn ban_user(db: &Database, user: &LiteUser) -> Result<(), AdminError> {
    admin_bans_service::add_ban(db, None, Some(user.username.clone()), "Banned via Telegram")
        .await
        .map_err(AdminError::Notify)?;

    if let Some(tg_id) = admin_cleanup_service::get_telegram_id_by_tt_user(db, &user.username).await
    {
        if let Err(e) = admin_cleanup_service::cleanup_deleted_banned_user(db, tg_id).await {
            tracing::error!(
                tt_username = %user.username,
                error = %e,
                "Failed to delete user profile during ban"
            );
        }
        if let Err(e) =
            admin_bans_service::add_ban(db, Some(tg_id), Some(user.username.clone()), "TG+TT Ban")
                .await
        {
            tracing::error!(
                tt_username = %user.username,
                error = %e,
                "Failed to add second ban record"
            );
        }
    }

    Ok(())
}

pub async fn broadcast(
    tx_tt: &tokio::sync::mpsc::Sender<crate::core::types::TtCommand>,
    text: String,
) -> Result<(), AdminError> {
    tx_tt
        .send(crate::core::types::TtCommand::Broadcast { text })
        .await
        .map_err(|e| AdminError::Notify(e.into()))
}

pub async fn send_direct_message(
    bot: &teloxide_ng::prelude::Bot,
    subs: &[crate::infra::db::types::SubscriberInfo],
    sender: TelegramId,
    text: &str,
) -> (usize, usize) {
    let mut sent = 0usize;
    let mut failed = 0usize;
    for sub in subs {
        if sub.telegram_id == sender {
            continue;
        }
        let chat_id = teloxide_ng::types::ChatId(sub.telegram_id.as_i64());
        match bot.send_message(chat_id, text.to_string()).await {
            Ok(_) => sent += 1,
            Err(e) => {
                failed += 1;
                tracing::warn!(
                    telegram_id = sub.telegram_id.as_i64(),
                    error = %e,
                    "Failed to send broadcast message"
                );
            }
        }
    }
    (sent, failed)
}
