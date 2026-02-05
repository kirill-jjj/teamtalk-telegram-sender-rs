use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{answer_callback, notify_admin_error, telegram_id_from_user_id};
use crate::app::services::tg_admin as tg_admin_service;
use crate::core::types::{AdminErrorContext, LanguageCode, TtCommand, TtUserId};
use crate::infra::locales;
use teloxide::prelude::*;

pub(super) async fn handle_kick_perform(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    user_id: TtUserId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let Some(admin_id) = telegram_id_from_user_id(q.from.id.0, "handle_kick_perform") else {
        return Ok(());
    };
    if let Err(e) = state.tx_tt.send(TtCommand::KickUser { user_id }).await {
        tracing::error!(user_id = user_id.as_i32(), error = %e, "Failed to send kick command");
        notify_admin_error(
            bot,
            &state.config,
            admin_id,
            AdminErrorContext::TtCommand,
            &e.to_string(),
            lang,
        )
        .await;
    }
    answer_callback(
        bot,
        &q.id,
        locales::get_text(lang.as_str(), locales::LocaleKey::ToastCommandSent, None),
        false,
    )
    .await
}

pub(super) async fn handle_ban_perform(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    user_id: TtUserId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let Some(admin_id) = telegram_id_from_user_id(q.from.id.0, "handle_ban_perform") else {
        return Ok(());
    };
    let user = match tg_admin_service::online_user_by_id(&state.state, user_id).await {
        Ok(user) => user,
        Err(err) => {
            let error = err.into_error();
            tracing::error!(user_id = user_id.as_i32(), error = %error, "Failed to resolve online user");
            notify_admin_error(
                bot,
                &state.config,
                admin_id,
                AdminErrorContext::Callback,
                &error.to_string(),
                lang,
            )
            .await;
            None
        }
    };
    let Some(u) = user else {
        return answer_callback(
            bot,
            &q.id,
            locales::get_text(lang.as_str(), locales::LocaleKey::CmdNoUsers, None),
            true,
        )
        .await;
    };

    if let Err(err) = tg_admin_service::ban_user(&state.db, &u).await {
        let error = err.into_error();
        tracing::error!(tt_username = %u.username, error = %error, "Failed to add ban");
        notify_admin_error(
            bot,
            &state.config,
            admin_id,
            AdminErrorContext::Callback,
            &error.to_string(),
            lang,
        )
        .await;
        answer_callback(
            bot,
            &q.id,
            locales::get_text(lang.as_str(), locales::LocaleKey::CmdError, None),
            true,
        )
        .await?;
        return Ok(());
    }

    if let Err(e) = state.tx_tt.send(TtCommand::BanUser { user_id }).await {
        tracing::error!(
            user_id = user_id.as_i32(),
            tt_username = %u.username,
            error = %e,
            "Failed to send ban command"
        );
        notify_admin_error(
            bot,
            &state.config,
            admin_id,
            AdminErrorContext::TtCommand,
            &e.to_string(),
            lang,
        )
        .await;
    }
    answer_callback(
        bot,
        &q.id,
        locales::get_text(lang.as_str(), locales::LocaleKey::ToastCommandSent, None),
        false,
    )
    .await
}
