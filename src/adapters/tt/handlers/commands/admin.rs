use crate::app::services::admins as admins_service;
use crate::args;
use crate::core::types::TelegramId;
use crate::infra::locales;

use super::user::UserCtx;

pub(super) async fn handle_add_admin(ctx: &UserCtx) {
    let is_main_admin = ctx
        .admin_username
        .as_ref()
        .is_some_and(|u| u == &ctx.username);
    if !is_main_admin {
        let text = locales::get_text_or_log(ctx.reply_lang.as_str(), locales::LocaleKey::CmdUnauth, None);
        ctx.send_reply(text).await;
        return;
    }
    let parts: Vec<&str> = ctx.content.split_whitespace().collect();
    if parts.len() < 2 {
        let text = locales::get_text_or_log(
            ctx.reply_lang.as_str(),
            locales::LocaleKey::TtAdminNoIds,
            None,
        );
        ctx.send_reply(text).await;
        return;
    }
    let mut added_count = 0;
    let mut failed_count = 0;
    for id_str in &parts[1..] {
        let tg_id: i64 = if let Ok(val) = id_str.parse() {
            val
        } else {
            failed_count += 1;
            continue;
        };
        {
            let success =
                match admins_service::add_admin(&ctx.services.db, TelegramId::from(tg_id)).await {
                    Ok(()) => true,
                    Err(e) => {
                        tracing::error!(telegram_id = tg_id, error = %e, "DB error adding admin");
                        false
                    }
                };
            if success {
                added_count += 1;
            }
        }
    }
    if added_count > 0 {
        let args = args!(count = added_count);
        let text = locales::get_text_or_log(
            ctx.reply_lang.as_str(),
            locales::LocaleKey::TtAdminAdded,
            args.as_ref(),
        );
        ctx.send_reply(text).await;
    }
    if failed_count > 0 {
        let args = args!(count = failed_count);
        let text = locales::get_text_or_log(
            ctx.reply_lang.as_str(),
            locales::LocaleKey::TtAdminAddFail,
            args.as_ref(),
        );
        ctx.send_reply(text).await;
    }
}

pub(super) async fn handle_remove_admin(ctx: &UserCtx) {
    let is_main_admin = ctx
        .admin_username
        .as_ref()
        .is_some_and(|u| u == &ctx.username);
    if !is_main_admin {
        let text = locales::get_text_or_log(ctx.reply_lang.as_str(), locales::LocaleKey::CmdUnauth, None);
        ctx.send_reply(text).await;
        return;
    }
    let parts: Vec<&str> = ctx.content.split_whitespace().collect();
    if parts.len() < 2 {
        let text = locales::get_text_or_log(
            ctx.reply_lang.as_str(),
            locales::LocaleKey::TtAdminNoIds,
            None,
        );
        ctx.send_reply(text).await;
        return;
    }
    let mut removed_count = 0;
    let mut failed_count = 0;
    for id_str in &parts[1..] {
        let tg_id: i64 = if let Ok(val) = id_str.parse() {
            val
        } else {
            failed_count += 1;
            continue;
        };
        {
            let success =
                match admins_service::remove_admin(&ctx.services.db, TelegramId::from(tg_id)).await
                {
                    Ok(()) => true,
                    Err(e) => {
                        tracing::error!(telegram_id = tg_id, error = %e, "DB error removing admin");
                        false
                    }
                };
            if success {
                removed_count += 1;
            } else {
                failed_count += 1;
            }
        }
    }
    if removed_count > 0 {
        let args = args!(count = removed_count);
        let text = locales::get_text_or_log(
            ctx.reply_lang.as_str(),
            locales::LocaleKey::TtAdminRemoved,
            args.as_ref(),
        );
        ctx.send_reply(text).await;
    }
    if failed_count > 0 {
        let args = args!(count = failed_count);
        let text = locales::get_text_or_log(
            ctx.reply_lang.as_str(),
            locales::LocaleKey::TtAdminRemoveFail,
            args.as_ref(),
        );
        ctx.send_reply(text).await;
    }
}
