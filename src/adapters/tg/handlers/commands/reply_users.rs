use crate::adapters::tg::utils::notify_admin_error;
use crate::app::services::tg_replies as tg_replies_service;
use crate::core::types::{AdminErrorContext, LanguageCode, TelegramId, TtCommand};
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;

use super::replies::load_pending_reply;
use crate::adapters::tg::state::AppState;

pub(super) async fn handle_user_reply(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    telegram_id: TelegramId,
    admin_lang: LanguageCode,
    reply_id: crate::core::types::TgMessageId,
    text: &str,
) -> ResponseResult<()> {
    let db = &state.db;
    let config = &state.config;
    let Some((tt_user_id, tt_username)) =
        load_pending_reply(bot, db, config, telegram_id, reply_id).await?
    else {
        return Ok(());
    };

    let current_tt_user_id = match tg_replies_service::resolve_current_tt_user_id(
        &state.state,
        tt_username.as_ref(),
        tt_user_id,
    )
    .await
    {
        Ok(value) => value,
        Err(err) => {
            let error = err.into_error();
            tracing::error!(error = %error, "Failed to resolve TT user id");
            None
        }
    };
    let is_online = match current_tt_user_id {
        Some(id) => tg_replies_service::is_tt_user_online(&state.state, id)
            .await
            .unwrap_or(false),
        None => false,
    };

    let reply_key = if is_online {
        let Some(target_id) = current_tt_user_id else {
            return Ok(());
        };
        let send_res = state
            .tx_tt
            .send(TtCommand::ReplyToUser {
                user_id: target_id,
                text: text.to_string(),
            })
            .await;
        if let Err(e) = send_res {
            tracing::error!(
                tt_user_id = tt_user_id.as_i32(),
                error = %e,
                "Failed to send TT reply command"
            );
            notify_admin_error(
                bot,
                config,
                telegram_id,
                AdminErrorContext::Command,
                &e.to_string(),
                admin_lang,
            )
            .await;
            locales::LocaleKey::TgReplyFailed
        } else {
            locales::LocaleKey::TgReplySent
        }
    } else if let Some(tt_username) = tt_username.as_ref() {
        match tg_replies_service::queue_reply(db, tt_username, telegram_id, text).await {
            Ok(tg_replies_service::UserReplyOutcome::Queued) => locales::LocaleKey::TgReplyQueued,
            Ok(tg_replies_service::UserReplyOutcome::Offline) => locales::LocaleKey::TgReplyOffline,
            Err(err) => {
                let error = err.into_error();
                tracing::error!(error = %error, "Failed to queue reply");
                locales::LocaleKey::TgReplyFailed
            }
        }
    } else {
        locales::LocaleKey::TgReplyOffline
    };
    let reply_text = locales::get_text_or_log(admin_lang.as_str(), reply_key, None);
    let _ = bot
        .send_message(msg.chat.id, reply_text)
        .reply_to(msg.id)
        .await;

    if let Err(err) = tg_replies_service::touch_pending_reply(db, reply_id).await {
        let error = err.into_error();
        tracing::error!(
            reply_id = reply_id.as_i32(),
            error = %error,
            "Failed to update pending reply"
        );
    }

    Ok(())
}
