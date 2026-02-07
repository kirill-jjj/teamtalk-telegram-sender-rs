use crate::adapters::tg::handlers::search::{
    SearchContext, SearchListType, append_search_hint, set_search_context,
};
use crate::adapters::tg::presenter::admin::bans::send_unban_list;
use crate::adapters::tg::presenter::admin::subscribers::{
    prepare_display_list, send_subscribers_list,
};
use crate::adapters::tg::presenter::keyboards::create_user_list_keyboard;
use crate::adapters::tg::utils::send_text_key;
use crate::app::services::tg_admin as tg_admin_service;
use crate::args;
use crate::core::callbacks::{AdminAction, CallbackAction};
use crate::core::types::{AdminErrorContext, LiteUser, TtCommand};
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;

use super::{Command, CommandCtx};

async fn reject_if_not_admin(ctx: &CommandCtx<'_>) -> ResponseResult<bool> {
    if ctx.is_admin {
        return Ok(false);
    }
    send_text_key(
        ctx.bot,
        ctx.msg.chat.id,
        ctx.lang,
        locales::LocaleKey::CmdUnauth,
        Some(ctx.msg.id),
    )
    .await?;
    Ok(true)
}

pub(super) async fn handle_kick_or_ban(ctx: &CommandCtx<'_>, cmd: Command) -> ResponseResult<()> {
    if reject_if_not_admin(ctx).await? {
        return Ok(());
    }
    let users: Vec<LiteUser> = match tg_admin_service::list_online_users(&ctx.state.state).await {
        Ok(users) => users,
        Err(err) => {
            let notify = err.should_notify();
            let error = err.into_error();
            tracing::error!(error = %error, "Failed to list online users");
            if notify {
                ctx.errors()
                    .notify(AdminErrorContext::Command, &error.to_string())
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
            return Ok(());
        }
    };

    let is_kick = matches!(cmd, Command::Kick);
    let title_key = if is_kick {
        locales::LocaleKey::ListKickTitle
    } else {
        locales::LocaleKey::ListBanTitle
    };

    let args = args!(server = ctx.config.teamtalk.display_name().to_string());
    let base = locales::get_text(ctx.lang.as_str(), title_key, args.as_ref());
    let title = append_search_hint(&base, ctx.lang);

    let keyboard = create_user_list_keyboard(
        &users,
        0,
        move |u| {
            let action = if is_kick {
                AdminAction::KickPerform { user_id: u.id }
            } else {
                AdminAction::BanPerform { user_id: u.id }
            };
            (
                u.nickname.as_str().to_string(),
                CallbackAction::Admin(action),
            )
        },
        move |p| {
            let action = if is_kick {
                AdminAction::KickList { page: p }
            } else {
                AdminAction::BanList { page: p }
            };
            CallbackAction::Admin(action)
        },
        None,
        ctx.lang,
    );

    let sent = ctx
        .bot
        .send_message(ctx.msg.chat.id, title)
        .reply_to(ctx.msg.id)
        .reply_markup(keyboard)
        .await?;
    let list_type = if is_kick {
        SearchListType::Kick
    } else {
        SearchListType::Ban
    };
    set_search_context(
        ctx.state,
        sent.chat.id,
        SearchContext {
            message_id: sent.id,
            list_type,
        },
    )
    .await;
    Ok(())
}

pub(super) async fn handle_unban(ctx: &CommandCtx<'_>) -> ResponseResult<()> {
    if reject_if_not_admin(ctx).await? {
        return Ok(());
    }
    send_unban_list(
        ctx.bot,
        ctx.msg.chat.id,
        match tg_admin_service::list_ban_entries(ctx.db).await {
            Ok(entries) => entries,
            Err(err) => {
                let notify = err.should_notify();
                let error = err.into_error();
                tracing::error!(error = %error, "Failed to load ban entries");
                if notify {
                    ctx.errors()
                        .notify(AdminErrorContext::Command, &error.to_string())
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
                return Ok(());
            }
        },
        &ctx.state.search_contexts,
        ctx.lang,
        0,
        Some(ctx.msg.id),
    )
    .await
}

pub(super) async fn handle_subscribers(ctx: &CommandCtx<'_>) -> ResponseResult<()> {
    if reject_if_not_admin(ctx).await? {
        return Ok(());
    }
    send_subscribers_list(
        ctx.bot,
        ctx.msg.chat.id,
        prepare_display_list(
            ctx.bot,
            match tg_admin_service::list_subscribers(ctx.db).await {
                Ok(subs) => subs,
                Err(err) => {
                    let notify = err.should_notify();
                    let error = err.into_error();
                    tracing::error!(error = %error, "Failed to load subscribers");
                    if notify {
                        ctx.errors()
                            .notify(AdminErrorContext::Command, &error.to_string())
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
                    return Ok(());
                }
            },
        )
        .await,
        &ctx.state.search_contexts,
        ctx.lang,
        0,
        Some(ctx.msg.id),
    )
    .await
}

pub(super) async fn handle_exit(ctx: &CommandCtx<'_>) -> ResponseResult<()> {
    if reject_if_not_admin(ctx).await? {
        return Ok(());
    }
    ctx.bot
        .send_message(
            ctx.msg.chat.id,
            locales::get_text(ctx.lang.as_str(), locales::LocaleKey::CmdShuttingDown, None),
        )
        .reply_to(ctx.msg.id)
        .await?;
    if let Err(err) = ctx.state.tx_tt.send(TtCommand::Shutdown).await {
        tracing::error!(error = %err, "Failed to send shutdown command");
        ctx.errors()
            .notify(AdminErrorContext::TtCommand, &err.to_string())
            .await;
    }
    ctx.state.cancel_token.cancel();
    Ok(())
}

pub(super) async fn handle_broadcast(ctx: &CommandCtx<'_>, text: String) -> ResponseResult<()> {
    if reject_if_not_admin(ctx).await? {
        return Ok(());
    }

    let text = text.trim().to_string();
    if text.is_empty() {
        send_text_key(
            ctx.bot,
            ctx.msg.chat.id,
            ctx.lang,
            locales::LocaleKey::CmdBroadcastEmpty,
            Some(ctx.msg.id),
        )
        .await?;
        return Ok(());
    }

    if let Err(err) = tg_admin_service::broadcast(&ctx.state.tx_tt, text).await {
        let notify = err.should_notify();
        let error = err.into_error();
        tracing::error!(error = %error, "Failed to send broadcast command");
        if notify {
            ctx.errors()
                .notify(AdminErrorContext::Command, &error.to_string())
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
        return Ok(());
    }

    send_text_key(
        ctx.bot,
        ctx.msg.chat.id,
        ctx.lang,
        locales::LocaleKey::CmdBroadcastSent,
        Some(ctx.msg.id),
    )
    .await
}

pub(super) async fn handle_message(ctx: &CommandCtx<'_>, text: String) -> ResponseResult<()> {
    if reject_if_not_admin(ctx).await? {
        return Ok(());
    }

    let text = text.trim().to_string();
    if text.is_empty() {
        send_text_key(
            ctx.bot,
            ctx.msg.chat.id,
            ctx.lang,
            locales::LocaleKey::CmdMessageEmpty,
            Some(ctx.msg.id),
        )
        .await?;
        return Ok(());
    }

    let subs = match tg_admin_service::list_subscribers(ctx.db).await {
        Ok(subs) => subs,
        Err(err) => {
            let notify = err.should_notify();
            let error = err.into_error();
            tracing::error!(error = %error, "Failed to load subscribers");
            if notify {
                ctx.errors()
                    .notify(AdminErrorContext::Command, &error.to_string())
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
            return Ok(());
        }
    };

    let (sent, failed) =
        tg_admin_service::send_direct_message(ctx.bot, &subs, ctx.telegram_id, &text).await;

    let args = args!(sent = sent, failed = failed);
    let reply = locales::get_text(
        ctx.lang.as_str(),
        locales::LocaleKey::CmdMessageSent,
        args.as_ref(),
    );
    ctx.bot
        .send_message(ctx.msg.chat.id, reply)
        .reply_to(ctx.msg.id)
        .await?;
    Ok(())
}

pub(super) async fn handle_plugins(ctx: &CommandCtx<'_>, text: String) -> ResponseResult<()> {
    if reject_if_not_admin(ctx).await? {
        return Ok(());
    }
    let input = text.trim();
    let mut parts = input.split_whitespace();
    let sub = parts.next().unwrap_or("status");

    let result = match sub {
        "status" => Ok(ctx.state.plugins.status_text().await),
        "reload" => match parts.next() {
            Some(name) => ctx
                .state
                .plugins
                .reload_named(name, &ctx.config.plugins.disabled)
                .await
                .map(|()| format!("Plugin reloaded: {name}"))
                .map_err(|error| error.to_string()),
            None => Err("Usage: /plugins reload <name>".to_string()),
        },
        "enable" => match parts.next() {
            Some(name) => ctx
                .state
                .plugins
                .set_enabled(name, true)
                .await
                .map(|()| format!("Plugin enabled: {name}"))
                .map_err(|error| error.to_string()),
            None => Err("Usage: /plugins enable <name>".to_string()),
        },
        "disable" => match parts.next() {
            Some(name) => ctx
                .state
                .plugins
                .set_enabled(name, false)
                .await
                .map(|()| format!("Plugin disabled: {name}"))
                .map_err(|error| error.to_string()),
            None => Err("Usage: /plugins disable <name>".to_string()),
        },
        _ => Err("Usage: /plugins status|reload|enable|disable".to_string()),
    };

    let reply = match result {
        Ok(message) => message,
        Err(message) => format!("Plugin command failed: {message}"),
    };
    ctx.bot
        .send_message(ctx.msg.chat.id, reply)
        .reply_to(ctx.msg.id)
        .await?;
    Ok(())
}
