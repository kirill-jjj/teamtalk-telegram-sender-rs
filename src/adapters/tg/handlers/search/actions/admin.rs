use crate::adapters::tg::presenter::admin::bans as bans_logic;
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{TgErrorReporter, send_text_key};
use crate::app::services::tg_admin as tg_admin_service;
use crate::app::services::tg_search_actions as tg_search_actions_service;
use crate::core::callbacks::{AdminAction, CallbackAction};
use crate::core::types::{AdminErrorContext, LanguageCode, TtCommand};
use crate::infra::locales;
use teloxide_ng::prelude::*;

use crate::adapters::tg::handlers::search::context::SearchCandidate;

pub(super) async fn handle_kick(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    candidate: &SearchCandidate,
    lang: LanguageCode,
) -> ResponseResult<bool> {
    let CallbackAction::Admin(AdminAction::KickPerform { user_id }) = &candidate.action else {
        return Ok(false);
    };
    send_tt_command(
        bot,
        msg,
        state,
        TtCommand::KickUser { user_id: *user_id },
        lang,
    )
    .await?;
    Ok(true)
}

pub(super) async fn handle_ban(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    candidate: &SearchCandidate,
    lang: LanguageCode,
) -> ResponseResult<bool> {
    let CallbackAction::Admin(AdminAction::BanPerform { user_id }) = &candidate.action else {
        return Ok(false);
    };
    send_tt_command(
        bot,
        msg,
        state,
        TtCommand::BanUser { user_id: *user_id },
        lang,
    )
    .await?;
    Ok(true)
}

pub(super) async fn handle_unban(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    candidate: &SearchCandidate,
    lang: LanguageCode,
) -> ResponseResult<bool> {
    let CallbackAction::Admin(AdminAction::UnbanPerform { ban_db_id, page }) = &candidate.action
    else {
        return Ok(false);
    };
    if let Err(e) = tg_search_actions_service::remove_ban(&state.db, *ban_db_id).await {
        if let Some(requester_id) = super::requester_id_from_message(msg, "search_unban") {
            TgErrorReporter::new(bot, &state.config, requester_id, lang)
                .notify(AdminErrorContext::Callback, &e.to_string())
                .await;
        }
        return Ok(true);
    }
    send_text_key(
        bot,
        msg.chat.id,
        lang,
        locales::LocaleKey::ToastUserUnbanned,
        Some(msg.id),
    )
    .await?;
    bans_logic::edit_unban_list(
        bot,
        msg,
        match tg_admin_service::list_ban_entries(&state.db).await {
            Ok(entries) => entries,
            Err(err) => {
                if let Some(requester_id) =
                    super::requester_id_from_message(msg, "search_bans_reload")
                {
                    TgErrorReporter::new(bot, &state.config, requester_id, lang)
                        .notify(AdminErrorContext::Callback, &err.into_error().to_string())
                        .await;
                }
                Vec::new()
            }
        },
        &state.search_contexts,
        lang,
        *page,
    )
    .await?;
    Ok(true)
}

async fn send_tt_command(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    cmd: TtCommand,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let admin_id = super::requester_id_from_message(msg, "search_send_tt_command");
    if let Err(e) = state.tx_tt.send(cmd).await
        && let Some(admin_id) = admin_id
    {
        TgErrorReporter::new(bot, &state.config, admin_id, lang)
            .notify(AdminErrorContext::TtCommand, &e.to_string())
            .await;
    }
    send_text_key(
        bot,
        msg.chat.id,
        lang,
        locales::LocaleKey::ToastCommandSent,
        Some(msg.id),
    )
    .await?;
    Ok(())
}
