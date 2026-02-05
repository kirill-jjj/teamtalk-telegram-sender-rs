use crate::app::services::admins as admins_service;
use crate::app::services::reply_queue as reply_queue_service;
use crate::app::services::subscribers as subscribers_service;
use crate::args;
use crate::core::types::TelegramId;
use crate::infra::locales;
use fluent_templates::fluent_bundle::FluentValue;
use std::borrow::Cow;
use std::collections::HashMap;

use super::user::UserCtx;

pub(super) async fn handle_queue(ctx: &UserCtx) {
    let Some(tg_id) = resolve_tg_id(ctx).await else {
        return;
    };
    let parts: Vec<&str> = ctx.content.split_whitespace().collect();
    let Some(cmd) = parts.get(1) else {
        send_text(ctx, locales::LocaleKey::TtQueueHelp, None).await;
        return;
    };
    let is_admin = resolve_is_admin(ctx, tg_id).await;

    match (*cmd, parts.get(2)) {
        ("on" | "off", _) => handle_global_toggle(ctx, is_admin, *cmd == "on").await,
        ("me", Some(value)) if *value == "on" || *value == "off" => {
            handle_user_toggle(ctx, tg_id, *value == "on").await;
        }
        ("clear", None) => handle_clear_user(ctx).await,
        ("clear", Some(all)) if *all == "all" => handle_clear_all(ctx, is_admin).await,
        _ => send_text(ctx, locales::LocaleKey::TtQueueHelp, None).await,
    }
}

async fn resolve_tg_id(ctx: &UserCtx) -> Option<TelegramId> {
    if ctx.username.as_str().is_empty() {
        send_text(ctx, locales::LocaleKey::TtQueueNoLink, None).await;
        return None;
    }
    let tt_tg_id =
        subscribers_service::get_telegram_id_by_tt_user(&ctx.services.db, &ctx.username).await;
    if tt_tg_id.is_none() {
        send_text(ctx, locales::LocaleKey::TtQueueNoLink, None).await;
    }
    tt_tg_id
}

async fn resolve_is_admin(ctx: &UserCtx, tg_id: TelegramId) -> bool {
    if ctx
        .admin_username
        .as_ref()
        .is_some_and(|u| u == &ctx.username)
    {
        return true;
    }
    admins_service::get_all_admins(&ctx.services.db)
        .await
        .is_ok_and(|admins| admins.contains(&tg_id))
}

async fn handle_global_toggle(ctx: &UserCtx, is_admin: bool, enabled: bool) {
    if !is_admin {
        send_text(ctx, locales::LocaleKey::CmdUnauth, None).await;
        return;
    }
    let current = reply_queue_service::get_reply_queue_global_enabled(&ctx.services.db).await;
    let text_key = match current {
        Ok(val) if val == enabled => {
            if enabled {
                locales::LocaleKey::TtQueueGlobalAlreadyEnabled
            } else {
                locales::LocaleKey::TtQueueGlobalAlreadyDisabled
            }
        }
        Ok(_) => {
            if reply_queue_service::set_reply_queue_global_enabled(&ctx.services.db, enabled)
                .await
                .is_ok()
            {
                if enabled {
                    locales::LocaleKey::TtQueueGlobalEnabled
                } else {
                    locales::LocaleKey::TtQueueGlobalDisabled
                }
            } else {
                locales::LocaleKey::TtErrorGeneric
            }
        }
        Err(_) => locales::LocaleKey::TtErrorGeneric,
    };
    send_text(ctx, text_key, None).await;
}

async fn handle_user_toggle(ctx: &UserCtx, tg_id: TelegramId, enabled: bool) {
    let global_enabled =
        reply_queue_service::get_reply_queue_global_enabled(&ctx.services.db).await;
    let text_key = match global_enabled {
        Ok(false) => locales::LocaleKey::TtQueueGlobalDisabledUser,
        Ok(true) => {
            let current =
                reply_queue_service::get_reply_queue_user_enabled(&ctx.services.db, tg_id).await;
            match current {
                Ok(val) if val == enabled => {
                    if enabled {
                        locales::LocaleKey::TtQueueUserAlreadyEnabled
                    } else {
                        locales::LocaleKey::TtQueueUserAlreadyDisabled
                    }
                }
                Ok(_) => {
                    if reply_queue_service::set_reply_queue_user_enabled(
                        &ctx.services.db,
                        tg_id,
                        enabled,
                    )
                    .await
                    .is_ok()
                    {
                        if enabled {
                            locales::LocaleKey::TtQueueUserEnabled
                        } else {
                            locales::LocaleKey::TtQueueUserDisabled
                        }
                    } else {
                        locales::LocaleKey::TtErrorGeneric
                    }
                }
                Err(_) => locales::LocaleKey::TtErrorGeneric,
            }
        }
        Err(_) => locales::LocaleKey::TtErrorGeneric,
    };
    send_text(ctx, text_key, None).await;
}

async fn handle_clear_user(ctx: &UserCtx) {
    let count =
        reply_queue_service::clear_reply_queue_for_user(&ctx.services.db, &ctx.username).await;
    let text = count.map_or_else(
        |_| {
            locales::get_text(
                ctx.reply_lang.as_str(),
                locales::LocaleKey::TtErrorGeneric,
                None,
            )
        },
        |count| {
            locales::get_text(
                ctx.reply_lang.as_str(),
                locales::LocaleKey::TtQueueCleared,
                args!(count = count).as_ref(),
            )
        },
    );
    ctx.send_reply(text).await;
}

async fn handle_clear_all(ctx: &UserCtx, is_admin: bool) {
    if !is_admin {
        send_text(ctx, locales::LocaleKey::CmdUnauth, None).await;
        return;
    }
    let count = reply_queue_service::clear_reply_queue_all(&ctx.services.db).await;
    let text = count.map_or_else(
        |_| {
            locales::get_text(
                ctx.reply_lang.as_str(),
                locales::LocaleKey::TtErrorGeneric,
                None,
            )
        },
        |count| {
            locales::get_text(
                ctx.reply_lang.as_str(),
                locales::LocaleKey::TtQueueClearedAll,
                args!(count = count).as_ref(),
            )
        },
    );
    ctx.send_reply(text).await;
}

type LocaleArgs = HashMap<Cow<'static, str>, FluentValue<'static>>;

async fn send_text(ctx: &UserCtx, key: locales::LocaleKey, args: Option<LocaleArgs>) {
    let text = locales::get_text(ctx.reply_lang.as_str(), key, args.as_ref());
    drop(args);
    ctx.send_reply(text).await;
}
