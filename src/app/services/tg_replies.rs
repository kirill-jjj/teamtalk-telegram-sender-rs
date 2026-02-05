use crate::app::services::{pending as pending_service, reply_queue as reply_queue_service};
use crate::app::state::StateHandle;
use crate::core::types::{TelegramId, TtUserId, TtUsername};
use crate::infra::db::Database;
use anyhow::Result;

#[derive(Debug, Clone)]
pub enum UserReplyOutcome {
    Queued,
    Offline,
}

#[derive(Debug)]
pub enum ReplyError {
    Notify(anyhow::Error),
    Silent(anyhow::Error),
}

impl ReplyError {
    pub const fn should_notify(&self) -> bool {
        matches!(self, Self::Notify(_))
    }

    pub fn into_error(self) -> anyhow::Error {
        match self {
            Self::Notify(e) | Self::Silent(e) => e,
        }
    }
}

pub async fn load_pending_reply(
    db: &Database,
    reply_id: crate::core::types::TgMessageId,
) -> Result<Option<(TtUserId, Option<TtUsername>)>, ReplyError> {
    pending_service::get_pending_reply(db, reply_id)
        .await
        .map_err(ReplyError::Notify)
}

pub async fn touch_pending_reply(
    db: &Database,
    reply_id: crate::core::types::TgMessageId,
) -> Result<(), ReplyError> {
    pending_service::touch_pending_reply(db, reply_id)
        .await
        .map_err(ReplyError::Silent)
}

pub async fn load_pending_channel_reply(
    db: &Database,
    reply_id: crate::core::types::TgMessageId,
) -> Result<
    Option<(
        crate::core::types::TtChannelId,
        crate::core::types::TtChannelName,
        crate::core::types::TtServerName,
        String,
    )>,
    ReplyError,
> {
    pending_service::get_pending_channel_reply(db, reply_id)
        .await
        .map_err(ReplyError::Notify)
}

pub async fn touch_pending_channel_reply(
    db: &Database,
    reply_id: crate::core::types::TgMessageId,
) -> Result<(), ReplyError> {
    pending_service::touch_pending_channel_reply(db, reply_id)
        .await
        .map_err(ReplyError::Silent)
}

pub async fn resolve_current_tt_user_id(
    state: &StateHandle,
    tt_username: Option<&TtUsername>,
    tt_user_id: TtUserId,
) -> Result<Option<TtUserId>, ReplyError> {
    if let Some(username) = tt_username {
        let user_id = state
            .user_id_by_username(username)
            .await
            .map_err(|e| ReplyError::Silent(e.into()))?;
        Ok(user_id.or(Some(tt_user_id)))
    } else {
        Ok(Some(tt_user_id))
    }
}

pub async fn is_tt_user_online(state: &StateHandle, user_id: TtUserId) -> Result<bool, ReplyError> {
    let user = state
        .online_user_by_id(user_id)
        .await
        .map_err(|e| ReplyError::Silent(e.into()))?;
    Ok(user.is_some())
}

pub async fn queue_reply(
    db: &Database,
    tt_username: &TtUsername,
    telegram_id: TelegramId,
    text: &str,
) -> Result<UserReplyOutcome, ReplyError> {
    let enabled = reply_queue_service::is_reply_queue_enabled_for_tt_user(db, tt_username)
        .await
        .map_err(ReplyError::Silent)?;
    if !enabled {
        return Ok(UserReplyOutcome::Offline);
    }
    reply_queue_service::add_reply_queue_item(db, tt_username, telegram_id, text)
        .await
        .map_err(ReplyError::Silent)?;
    Ok(UserReplyOutcome::Queued)
}
