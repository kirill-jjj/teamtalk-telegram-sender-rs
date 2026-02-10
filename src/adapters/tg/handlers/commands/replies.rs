use crate::adapters::tg::utils::TgErrorReporter;
use crate::app::services::tg_replies as tg_replies_service;
use crate::args;
use crate::bootstrap::config::Config;
use crate::core::types::{AdminErrorContext, LanguageCode, TelegramId, TtCommand, TtUsername};
use crate::infra::locales;
use teloxide_ng::prelude::*;
use teloxide_ng::sugar::request::RequestReplyExt;
use teloxide_ng::types::Voice;

use super::format_duration;
use super::voice::stream_voice;
use crate::adapters::tg::state::AppState;

async fn send_voice_reply(
    ctx: &ChannelReplyCtx<'_>,
    channel_id: crate::core::types::TtChannelId,
    original_text: &str,
    voice: &Voice,
) -> Result<(), super::voice::StreamVoiceError> {
    let duration = format_duration(voice.duration.seconds());
    let args = args!(msg = original_text.to_string(), duration = duration);
    let announce_text = locales::get_text(
        ctx.admin_lang.as_str(),
        locales::LocaleKey::TtChannelReply,
        args.as_ref(),
    );
    stream_voice(ctx.bot, ctx.state, Some((channel_id, announce_text)), voice).await
}

async fn send_text_reply(
    ctx: &ChannelReplyCtx<'_>,
    channel_id: crate::core::types::TtChannelId,
    original_text: &str,
    text: &str,
) -> Result<(), tokio::sync::mpsc::error::SendError<TtCommand>> {
    let args = args!(msg = original_text.to_string(), reply = text.to_string());
    let channel_text = locales::get_text(
        ctx.admin_lang.as_str(),
        locales::LocaleKey::TtChannelReplyText,
        args.as_ref(),
    );
    ctx.state
        .tx_tt
        .send(TtCommand::SendToChannel {
            channel_id,
            text: channel_text,
        })
        .await
}

async fn notify_send_error(ctx: &ChannelReplyCtx<'_>, error: &str) {
    TgErrorReporter::new(ctx.bot, &ctx.state.config, ctx.telegram_id, ctx.admin_lang)
        .notify(AdminErrorContext::Command, error)
        .await;
}

pub(super) struct ChannelReplyCtx<'a> {
    pub(super) bot: &'a Bot,
    pub(super) msg: &'a Message,
    pub(super) state: &'a AppState,
    pub(super) telegram_id: TelegramId,
    pub(super) admin_lang: LanguageCode,
}

pub(super) struct ChannelReplyInput<'a> {
    pub(super) reply_id: crate::core::types::TgMessageId,
    pub(super) text: Option<&'a str>,
    pub(super) voice: Option<&'a Voice>,
}

pub(super) async fn handle_channel_reply(
    ctx: ChannelReplyCtx<'_>,
    input: ChannelReplyInput<'_>,
) -> ResponseResult<bool> {
    let db = &ctx.state.db;
    match tg_replies_service::load_pending_channel_reply(db, input.reply_id).await {
        Ok(Some((channel_id, _channel_name, _server_name, original_text))) => {
            if let Some(voice) = input.voice {
                if let Err(e) = send_voice_reply(&ctx, channel_id, &original_text, voice).await {
                    let error = e.to_string();
                    notify_send_error(&ctx, &error).await;
                    let reply_text = locales::get_text(
                        ctx.admin_lang.as_str(),
                        locales::LocaleKey::TgReplyFailed,
                        None,
                    );
                    let _ = ctx
                        .bot
                        .send_message(ctx.msg.chat.id, reply_text)
                        .reply_to(ctx.msg.id)
                        .await;
                    return Ok(true);
                }
            } else if let Some(text) = input.text {
                if let Err(e) = send_text_reply(&ctx, channel_id, &original_text, text).await {
                    tracing::error!(
                        channel_id = channel_id.as_i32(),
                        error = %e,
                        "Failed to send TT channel reply"
                    );
                    let error = e.to_string();
                    notify_send_error(&ctx, &error).await;
                    let reply_text = locales::get_text(
                        ctx.admin_lang.as_str(),
                        locales::LocaleKey::TgReplyFailed,
                        None,
                    );
                    let _ = ctx
                        .bot
                        .send_message(ctx.msg.chat.id, reply_text)
                        .reply_to(ctx.msg.id)
                        .await;
                    return Ok(true);
                }
            } else {
                return Ok(true);
            }

            let reply_text = locales::get_text(
                ctx.admin_lang.as_str(),
                locales::LocaleKey::TgReplySent,
                None,
            );
            let _ = ctx
                .bot
                .send_message(ctx.msg.chat.id, reply_text)
                .reply_to(ctx.msg.id)
                .await;

            if let Err(err) =
                tg_replies_service::touch_pending_channel_reply(db, input.reply_id).await
            {
                let error = err.into_error();
                tracing::error!(
                    reply_id = input.reply_id.as_i32(),
                    error = %error,
                    "Failed to update pending channel reply"
                );
            }

            Ok(true)
        }
        Ok(None) => Ok(false),
        Err(err) => {
            let notify = err.should_notify();
            let error = err.into_error();
            tracing::error!(error = %error, "Failed to load pending channel reply");
            if notify {
                TgErrorReporter::new(ctx.bot, &ctx.state.config, ctx.telegram_id, ctx.admin_lang)
                    .notify(AdminErrorContext::Command, &error.to_string())
                    .await;
            }
            Ok(false)
        }
    }
}

pub(super) async fn load_pending_reply(
    bot: &Bot,
    db: &crate::infra::db::Database,
    config: &Config,
    telegram_id: TelegramId,
    reply_id: crate::core::types::TgMessageId,
) -> ResponseResult<Option<(crate::core::types::TtUserId, Option<TtUsername>)>> {
    match tg_replies_service::load_pending_reply(db, reply_id).await {
        Ok(Some(data)) => Ok(Some(data)),
        Ok(None) => Ok(None),
        Err(err) => {
            let notify = err.should_notify();
            let error = err.into_error();
            tracing::error!(
                reply_id = reply_id.as_i32(),
                error = %error,
                "Failed to load pending reply"
            );
            if notify {
                TgErrorReporter::new(bot, config, telegram_id, config.general.default_lang)
                    .notify(AdminErrorContext::Command, &error.to_string())
                    .await;
            }
            Ok(None)
        }
    }
}
