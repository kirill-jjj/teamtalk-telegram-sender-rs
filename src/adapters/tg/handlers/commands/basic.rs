use crate::adapters::tg::presenter::keyboards::{
    confirm_cancel_keyboard, create_main_menu_keyboard,
};
use crate::adapters::tg::presenter::settings::send_main_settings;
use crate::adapters::tg::utils::{ensure_subscribed, notify_admin_error, send_text_key};
use crate::app::services::tg_basic as tg_basic_service;
use crate::args;
use crate::core::callbacks::{CallbackAction, UnsubAction};
use crate::core::types::{AdminErrorContext, TtCommand};
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::types::ParseMode;

use super::CommandCtx;

pub(super) async fn handle_start(ctx: &CommandCtx<'_>, token: String) -> ResponseResult<()> {
    match tg_basic_service::resolve_start(ctx.db, ctx.telegram_id, &token).await {
        Ok(tg_basic_service::StartOutcome::NoToken) => {
            send_text_key(
                ctx.bot,
                ctx.msg.chat.id,
                ctx.lang,
                locales::LocaleKey::HelloStart,
                Some(ctx.msg.id),
            )
            .await
        }
        Ok(tg_basic_service::StartOutcome::InvalidToken) => {
            send_text_key(
                ctx.bot,
                ctx.msg.chat.id,
                ctx.lang,
                locales::LocaleKey::CmdInvalidDeeplink,
                Some(ctx.msg.id),
            )
            .await
        }
        Ok(tg_basic_service::StartOutcome::SubscribeBannedUser) => {
            send_text_key(
                ctx.bot,
                ctx.msg.chat.id,
                ctx.lang,
                locales::LocaleKey::CmdUserBanned,
                Some(ctx.msg.id),
            )
            .await
        }
        Ok(tg_basic_service::StartOutcome::SubscribeBannedTeamTalk { username }) => {
            let args = args!(name = username.to_string());
            ctx.bot
                .send_message(
                    ctx.msg.chat.id,
                    locales::get_text_or_log(
                        ctx.lang.as_str(),
                        locales::LocaleKey::CmdTtBanned,
                        args.as_ref(),
                    ),
                )
                .reply_to(ctx.msg.id)
                .await?;
            Ok(())
        }
        Ok(tg_basic_service::StartOutcome::SubscribeLinked) => {
            send_text_key(
                ctx.bot,
                ctx.msg.chat.id,
                ctx.lang,
                locales::LocaleKey::CmdSuccessSub,
                Some(ctx.msg.id),
            )
            .await
        }
        Ok(tg_basic_service::StartOutcome::SubscribeGuest) => {
            ctx.bot
                .send_message(
                    ctx.msg.chat.id,
                    locales::get_text_or_log(
                        ctx.lang.as_str(),
                        locales::LocaleKey::CmdSuccessSubGuest,
                        None,
                    ),
                )
                .parse_mode(ParseMode::Html)
                .reply_to(ctx.msg.id)
                .await?;
            Ok(())
        }
        Ok(tg_basic_service::StartOutcome::Unsubscribe) => handle_unsubscribe(ctx).await,
        Err(e) => {
            tracing::error!(error = %e, "DB error resolving deeplink");
            notify_admin_error(
                ctx.bot,
                ctx.config,
                ctx.telegram_id,
                AdminErrorContext::Command,
                &e.to_string(),
                ctx.lang,
            )
            .await;
            send_text_key(
                ctx.bot,
                ctx.msg.chat.id,
                ctx.lang,
                locales::LocaleKey::CmdError,
                Some(ctx.msg.id),
            )
            .await
        }
    }
}

pub(super) async fn handle_menu(ctx: &CommandCtx<'_>) -> ResponseResult<()> {
    if !ensure_subscribed(ctx.bot, ctx.msg, ctx.db, ctx.config, ctx.lang).await {
        return Ok(());
    }
    let keyboard = create_main_menu_keyboard(ctx.lang, ctx.is_admin);
    ctx.bot
        .send_message(
            ctx.msg.chat.id,
            locales::get_text_or_log(ctx.lang.as_str(), locales::LocaleKey::MenuTitle, None),
        )
        .parse_mode(ParseMode::Html)
        .reply_to(ctx.msg.id)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

pub(super) async fn handle_help(ctx: &CommandCtx<'_>) -> ResponseResult<()> {
    if !ensure_subscribed(ctx.bot, ctx.msg, ctx.db, ctx.config, ctx.lang).await {
        return Ok(());
    }
    ctx.bot
        .send_message(
            ctx.msg.chat.id,
            locales::get_text_or_log(ctx.lang.as_str(), locales::LocaleKey::HelpText, None),
        )
        .parse_mode(ParseMode::Html)
        .reply_to(ctx.msg.id)
        .await?;
    Ok(())
}

pub(super) async fn handle_who(ctx: &CommandCtx<'_>) -> ResponseResult<()> {
    if !ensure_subscribed(ctx.bot, ctx.msg, ctx.db, ctx.config, ctx.lang).await {
        return Ok(());
    }
    if let Err(e) = ctx
        .tx_tt
        .send(TtCommand::Who {
            chat_id: crate::core::types::TgChatId::from(ctx.msg.chat.id.0),
            lang: ctx.lang,
            reply_to: Some(crate::core::types::TgMessageId::from(ctx.msg.id.0)),
        })
        .await
    {
        tracing::error!(error = %e, "Failed to send TT who command");
        notify_admin_error(
            ctx.bot,
            ctx.config,
            ctx.telegram_id,
            AdminErrorContext::TtCommand,
            &e.to_string(),
            ctx.lang,
        )
        .await;
    }
    Ok(())
}

pub(super) async fn handle_settings(ctx: &CommandCtx<'_>) -> ResponseResult<()> {
    if !ensure_subscribed(ctx.bot, ctx.msg, ctx.db, ctx.config, ctx.lang).await {
        return Ok(());
    }
    send_main_settings(ctx.bot, ctx.msg.chat.id, ctx.lang, Some(ctx.msg.id)).await
}

pub(super) async fn handle_unsub(ctx: &CommandCtx<'_>) -> ResponseResult<()> {
    if !ensure_subscribed(ctx.bot, ctx.msg, ctx.db, ctx.config, ctx.lang).await {
        return Ok(());
    }
    let text = locales::get_text_or_log(
        ctx.lang.as_str(),
        locales::LocaleKey::UnsubConfirmText,
        None,
    );
    let keyboard = confirm_cancel_keyboard(
        ctx.lang,
        locales::LocaleKey::BtnYes,
        CallbackAction::Unsub(UnsubAction::Confirm),
        locales::LocaleKey::BtnNo,
        CallbackAction::Unsub(UnsubAction::Cancel),
    );
    ctx.bot
        .send_message(ctx.msg.chat.id, text)
        .reply_to(ctx.msg.id)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

pub(super) async fn handle_unsubscribe(ctx: &CommandCtx<'_>) -> ResponseResult<()> {
    if let Err(e) = tg_basic_service::unsubscribe(ctx.db, ctx.telegram_id).await {
        tracing::error!(error = %e, "DB error unsubscribing");
        notify_admin_error(
            ctx.bot,
            ctx.config,
            ctx.telegram_id,
            AdminErrorContext::Command,
            &e.to_string(),
            ctx.lang,
        )
        .await;
        return send_text_key(
            ctx.bot,
            ctx.msg.chat.id,
            ctx.lang,
            locales::LocaleKey::CmdError,
            Some(ctx.msg.id),
        )
        .await;
    }
    send_text_key(
        ctx.bot,
        ctx.msg.chat.id,
        ctx.lang,
        locales::LocaleKey::CmdSuccessUnsub,
        Some(ctx.msg.id),
    )
    .await
}
