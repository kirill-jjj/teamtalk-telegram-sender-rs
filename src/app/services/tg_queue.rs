use crate::app::services::{
    reply_queue as reply_queue_service, subscribers as subscribers_service,
};
use crate::core::types::{LanguageCode, TelegramId};
use crate::infra::db::Database;
use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub enum QueueOutcome {
    Help,
    Unauth,
    GlobalAlready { enabled: bool },
    GlobalSet { enabled: bool },
    UserNoLink,
    GlobalDisabledForUser,
    UserAlready { enabled: bool },
    UserSet { enabled: bool },
    ClearedAll { count: u64 },
    ClearedUser { count: u64 },
}

#[derive(Debug)]
pub enum QueueError {
    Notify(anyhow::Error),
    Silent(anyhow::Error),
}

impl QueueError {
    pub const fn should_notify(&self) -> bool {
        matches!(self, Self::Notify(_))
    }

    pub fn into_error(self) -> anyhow::Error {
        match self {
            Self::Notify(e) | Self::Silent(e) => e,
        }
    }
}

#[derive(Debug)]
enum QueueCommand {
    Help,
    GlobalToggle { enabled: bool },
    UserToggle { enabled: bool },
    ClearAll,
    ClearUser,
}

pub async fn handle_queue(
    db: &Database,
    telegram_id: TelegramId,
    is_admin: bool,
    default_lang: LanguageCode,
    text: &str,
) -> Result<QueueOutcome, QueueError> {
    let cmd = parse_queue_command(text);
    match cmd {
        QueueCommand::Help => Ok(QueueOutcome::Help),
        QueueCommand::GlobalToggle { enabled } => {
            if !is_admin {
                return Ok(QueueOutcome::Unauth);
            }
            let current = reply_queue_service::get_reply_queue_global_enabled(db)
                .await
                .map_err(QueueError::Notify)?;
            if current == enabled {
                return Ok(QueueOutcome::GlobalAlready { enabled });
            }
            reply_queue_service::set_reply_queue_global_enabled(db, enabled)
                .await
                .map_err(QueueError::Notify)?;
            Ok(QueueOutcome::GlobalSet { enabled })
        }
        QueueCommand::UserToggle { enabled } => {
            let tt_username = subscribers_service::get_tt_username_by_telegram_id(db, telegram_id)
                .await
                .map_err(QueueError::Silent)?;
            let Some(_tt_username) = tt_username else {
                return Ok(QueueOutcome::UserNoLink);
            };
            let global_enabled = reply_queue_service::get_reply_queue_global_enabled(db)
                .await
                .map_err(QueueError::Silent)?;
            if !global_enabled {
                return Ok(QueueOutcome::GlobalDisabledForUser);
            }
            let current =
                reply_queue_service::get_reply_queue_user_enabled(db, telegram_id, default_lang)
                    .await
                    .map_err(QueueError::Silent)?;
            if current == enabled {
                return Ok(QueueOutcome::UserAlready { enabled });
            }
            reply_queue_service::set_reply_queue_user_enabled(db, telegram_id, enabled)
                .await
                .map_err(QueueError::Notify)?;
            Ok(QueueOutcome::UserSet { enabled })
        }
        QueueCommand::ClearAll => {
            if !is_admin {
                return Ok(QueueOutcome::Unauth);
            }
            let cleared = reply_queue_service::clear_reply_queue_all(db)
                .await
                .map_err(QueueError::Notify)?;
            Ok(QueueOutcome::ClearedAll { count: cleared })
        }
        QueueCommand::ClearUser => {
            let tt_username = subscribers_service::get_tt_username_by_telegram_id(db, telegram_id)
                .await
                .map_err(QueueError::Silent)?;
            let Some(tt_username) = tt_username else {
                return Ok(QueueOutcome::UserNoLink);
            };
            let cleared = reply_queue_service::clear_reply_queue_for_user(db, &tt_username)
                .await
                .map_err(QueueError::Notify)?;
            Ok(QueueOutcome::ClearedUser { count: cleared })
        }
    }
}

fn parse_queue_command(text: &str) -> QueueCommand {
    let text = text.trim();
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.is_empty() {
        return QueueCommand::Help;
    }
    match parts.as_slice() {
        ["on" | "off"] => QueueCommand::GlobalToggle {
            enabled: parts[0] == "on",
        },
        ["me", "on" | "off"] => QueueCommand::UserToggle {
            enabled: parts[1] == "on",
        },
        ["clear"] | ["clear", "all"] => {
            if parts.len() == 2 && parts[1] == "all" {
                QueueCommand::ClearAll
            } else {
                QueueCommand::ClearUser
            }
        }
        _ => QueueCommand::Help,
    }
}
