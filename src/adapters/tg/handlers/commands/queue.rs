use crate::adapters::tg::utils::{notify_admin_error, send_text_key};
use crate::app::services::tg_queue as tg_queue_service;
use crate::args;
use crate::core::types::AdminErrorContext;
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;

use super::CommandCtx;

async fn send_key(ctx: &CommandCtx<'_>, key: locales::LocaleKey) -> ResponseResult<()> {
    send_text_key(ctx.bot, ctx.msg.chat.id, ctx.lang, key, Some(ctx.msg.id)).await
}

async fn send_count(
    ctx: &CommandCtx<'_>,
    key: locales::LocaleKey,
    count: u64,
) -> ResponseResult<()> {
    let text = locales::get_text_or_log(ctx.lang.as_str(), key, args!(count = count).as_ref());
    ctx.bot
        .send_message(ctx.msg.chat.id, text)
        .reply_to(ctx.msg.id)
        .await?;
    Ok(())
}

async fn send_outcome(
    ctx: &CommandCtx<'_>,
    outcome: tg_queue_service::QueueOutcome,
) -> ResponseResult<()> {
    match outcome {
        tg_queue_service::QueueOutcome::Help => {
            send_key(ctx, locales::LocaleKey::CmdQueueHelp).await
        }
        tg_queue_service::QueueOutcome::Unauth => {
            send_key(ctx, locales::LocaleKey::CmdUnauth).await
        }
        tg_queue_service::QueueOutcome::GlobalAlready { enabled } => {
            let key = if enabled {
                locales::LocaleKey::RespQueueGlobalAlreadyEnabled
            } else {
                locales::LocaleKey::RespQueueGlobalAlreadyDisabled
            };
            send_key(ctx, key).await
        }
        tg_queue_service::QueueOutcome::GlobalSet { enabled } => {
            let key = if enabled {
                locales::LocaleKey::RespQueueGlobalEnabled
            } else {
                locales::LocaleKey::RespQueueGlobalDisabled
            };
            send_key(ctx, key).await
        }
        tg_queue_service::QueueOutcome::UserNoLink => {
            send_key(ctx, locales::LocaleKey::CmdQueueNoLink).await
        }
        tg_queue_service::QueueOutcome::GlobalDisabledForUser => {
            send_key(ctx, locales::LocaleKey::RespQueueGlobalDisabledUser).await
        }
        tg_queue_service::QueueOutcome::UserAlready { enabled } => {
            let key = if enabled {
                locales::LocaleKey::RespQueueUserAlreadyEnabled
            } else {
                locales::LocaleKey::RespQueueUserAlreadyDisabled
            };
            send_key(ctx, key).await
        }
        tg_queue_service::QueueOutcome::UserSet { enabled } => {
            let key = if enabled {
                locales::LocaleKey::RespQueueUserEnabled
            } else {
                locales::LocaleKey::RespQueueUserDisabled
            };
            send_key(ctx, key).await
        }
        tg_queue_service::QueueOutcome::ClearedAll { count } => {
            send_count(ctx, locales::LocaleKey::RespQueueClearedAll, count).await
        }
        tg_queue_service::QueueOutcome::ClearedUser { count } => {
            send_count(ctx, locales::LocaleKey::RespQueueCleared, count).await
        }
    }
}

pub(super) async fn handle_queue(ctx: &CommandCtx<'_>, text: String) -> ResponseResult<()> {
    match tg_queue_service::handle_queue(ctx.db, ctx.telegram_id, ctx.is_admin, &text).await {
        Ok(outcome) => {
            send_outcome(ctx, outcome).await?;
        }
        Err(err) => {
            let notify = err.should_notify();
            let error = err.into_error();
            tracing::error!(error = %error, "Failed to handle queue command");
            if notify {
                notify_admin_error(
                    ctx.bot,
                    ctx.config,
                    ctx.telegram_id,
                    AdminErrorContext::Command,
                    &error.to_string(),
                    ctx.lang,
                )
                .await;
            }
            send_text_key(
                ctx.bot,
                ctx.msg.chat.id,
                ctx.lang,
                locales::LocaleKey::CmdError,
                Some(ctx.msg.id),
            )
            .await?;
        }
    }
    Ok(())
}
