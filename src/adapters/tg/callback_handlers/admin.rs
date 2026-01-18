use crate::adapters::tg::admin_logic::bans::{edit_unban_list, send_unban_list};
use crate::adapters::tg::admin_logic::subscribers::{edit_subscribers_list, send_subscribers_list};
use crate::adapters::tg::keyboards::create_user_list_keyboard;
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{
    answer_callback, answer_callback_empty, check_db_err, notify_admin_error,
};
use crate::app::services::admin_cleanup as admin_cleanup_service;
use crate::args;
use crate::core::callbacks::{AdminAction, CallbackAction};
use crate::core::types::{AdminErrorContext, LanguageCode, LiteUser, TtCommand};
use crate::infra::locales;
use teloxide::prelude::*;

pub async fn handle_admin(
    bot: Bot,
    q: CallbackQuery,
    state: AppState,
    action: AdminAction,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let Some(teloxide::types::MaybeInaccessibleMessage::Regular(ref msg)) = q.message else {
        return Ok(());
    };
    let msg = msg.as_ref();
    match action {
        AdminAction::KickList { page } => {
            handle_kick_list(&bot, &q, &state, msg, page, lang).await?;
        }
        AdminAction::BanList { page } => {
            handle_ban_list(&bot, &q, &state, msg, page, lang).await?;
        }
        AdminAction::KickPerform { user_id } => {
            handle_kick_perform(&bot, &q, &state, user_id, lang).await?;
        }
        AdminAction::BanPerform { user_id } => {
            handle_ban_perform(&bot, &q, &state, user_id, lang).await?;
        }
        AdminAction::UnbanList { page } => {
            handle_unban_list(&bot, &q, &state, msg, page, lang).await?;
        }
        AdminAction::UnbanPerform { ban_db_id, page } => {
            handle_unban_perform(&bot, &q, &state, msg, ban_db_id, page, lang).await?;
        }
        AdminAction::SubsList { page } => {
            handle_subs_list(&bot, &q, &state, msg, page, lang).await?;
        }
    }
    Ok(())
}

async fn handle_kick_list(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    page: usize,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let users = sorted_online_users(&state.online_users);
    let args = args!(server = state.config.teamtalk.display_name().to_string());
    let title = locales::get_text(lang.as_str(), "list-kick-title", args.as_ref());
    let keyboard = create_user_list_keyboard(
        &users,
        page,
        |u| {
            (
                u.nickname.clone(),
                CallbackAction::Admin(AdminAction::KickPerform { user_id: u.id }),
            )
        },
        |p| CallbackAction::Admin(AdminAction::KickList { page: p }),
        None,
        lang,
    );
    send_or_edit_list(bot, msg, page, title, keyboard).await?;
    answer_callback_empty(bot, &q.id).await
}

async fn handle_ban_list(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    page: usize,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let users = sorted_online_users(&state.online_users);
    let args = args!(server = state.config.teamtalk.display_name().to_string());
    let title = locales::get_text(lang.as_str(), "list-ban-title", args.as_ref());
    let keyboard = create_user_list_keyboard(
        &users,
        page,
        |u| {
            (
                u.nickname.clone(),
                CallbackAction::Admin(AdminAction::BanPerform { user_id: u.id }),
            )
        },
        |p| CallbackAction::Admin(AdminAction::BanList { page: p }),
        None,
        lang,
    );
    send_or_edit_list(bot, msg, page, title, keyboard).await?;
    answer_callback_empty(bot, &q.id).await
}

async fn handle_kick_perform(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    user_id: i32,
    lang: LanguageCode,
) -> ResponseResult<()> {
    if let Err(e) = state.tx_tt.send(TtCommand::KickUser { user_id }) {
        tracing::error!(user_id, error = %e, "Failed to send kick command");
        notify_admin_error(
            bot,
            &state.config,
            tg_user_id_i64(q.from.id.0),
            AdminErrorContext::TtCommand,
            &e.to_string(),
            lang,
        )
        .await;
    }
    answer_callback(
        bot,
        &q.id,
        locales::get_text(lang.as_str(), "toast-command-sent", None),
        false,
    )
    .await
}

async fn handle_ban_perform(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    user_id: i32,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let user = state
        .online_users
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&user_id)
        .cloned();
    let Some(u) = user else {
        return answer_callback(
            bot,
            &q.id,
            locales::get_text(lang.as_str(), "cmd-no-users", None),
            true,
        )
        .await;
    };

    if let Err(e) = state
        .db
        .add_ban(
            None,
            Some(u.username.clone()),
            Some("Banned via Telegram".to_string()),
        )
        .await
    {
        tracing::error!(tt_username = %u.username, error = %e, "Failed to add ban");
        notify_admin_error(
            bot,
            &state.config,
            tg_user_id_i64(q.from.id.0),
            AdminErrorContext::Callback,
            &e.to_string(),
            lang,
        )
        .await;
        answer_callback(
            bot,
            &q.id,
            locales::get_text(lang.as_str(), "cmd-error", None),
            true,
        )
        .await?;
        return Ok(());
    }

    if let Some(tg_id) =
        admin_cleanup_service::get_telegram_id_by_tt_user(&state.db, &u.username).await
    {
        if let Err(e) = admin_cleanup_service::cleanup_deleted_banned_user(&state.db, tg_id).await {
            tracing::error!(
                tt_username = %u.username,
                error = %e,
                "Failed to delete user profile during ban"
            );
        }
        if let Err(e) = state
            .db
            .add_ban(
                Some(tg_id),
                Some(u.username.clone()),
                Some("TG+TT Ban".to_string()),
            )
            .await
        {
            tracing::error!(
                tt_username = %u.username,
                error = %e,
                "Failed to add second ban record"
            );
        }
    }
    if let Err(e) = state.tx_tt.send(TtCommand::BanUser { user_id }) {
        tracing::error!(
            user_id,
            tt_username = %u.username,
            error = %e,
            "Failed to send ban command"
        );
        notify_admin_error(
            bot,
            &state.config,
            tg_user_id_i64(q.from.id.0),
            AdminErrorContext::TtCommand,
            &e.to_string(),
            lang,
        )
        .await;
    }
    answer_callback(
        bot,
        &q.id,
        locales::get_text(lang.as_str(), "toast-command-sent", None),
        false,
    )
    .await
}

async fn handle_unban_list(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    page: usize,
    lang: LanguageCode,
) -> ResponseResult<()> {
    if should_send_page(msg, page) {
        send_unban_list(bot, msg.chat.id, &state.db, lang, 0, None).await?;
    } else {
        edit_unban_list(bot, msg, &state.db, lang, page).await?;
    }
    answer_callback_empty(bot, &q.id).await
}

async fn handle_unban_perform(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    ban_db_id: i64,
    page: usize,
    lang: LanguageCode,
) -> ResponseResult<()> {
    if check_db_err(
        bot,
        &q.id.0,
        state.db.remove_ban_by_id(ban_db_id).await,
        &state.config,
        tg_user_id_i64(q.from.id.0),
        AdminErrorContext::Callback,
        lang,
    )
    .await?
    {
        return Ok(());
    }
    answer_callback(
        bot,
        &q.id,
        locales::get_text(lang.as_str(), "toast-user-unbanned", None),
        false,
    )
    .await?;
    edit_unban_list(bot, msg, &state.db, lang, page).await
}

async fn handle_subs_list(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    page: usize,
    lang: LanguageCode,
) -> ResponseResult<()> {
    if should_send_page(msg, page) {
        send_subscribers_list(bot, msg.chat.id, &state.db, lang, 0, None).await?;
    } else {
        edit_subscribers_list(bot, msg, &state.db, lang, page).await?;
    }
    answer_callback_empty(bot, &q.id).await
}

fn sorted_online_users(
    online_users: &std::sync::RwLock<std::collections::HashMap<i32, LiteUser>>,
) -> Vec<LiteUser> {
    let mut users: Vec<LiteUser> = online_users
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .values()
        .cloned()
        .collect();
    users.sort_by(|a, b| a.nickname.to_lowercase().cmp(&b.nickname.to_lowercase()));
    users
}

async fn send_or_edit_list(
    bot: &Bot,
    msg: &Message,
    page: usize,
    title: String,
    keyboard: teloxide::types::InlineKeyboardMarkup,
) -> ResponseResult<()> {
    if should_send_page(msg, page) {
        bot.send_message(msg.chat.id, title)
            .reply_markup(keyboard)
            .await?;
    } else {
        bot.edit_message_text(msg.chat.id, msg.id, title)
            .reply_markup(keyboard)
            .await?;
    }
    Ok(())
}

fn should_send_page(msg: &Message, page: usize) -> bool {
    page == 0 && !msg.text().unwrap_or("").contains("Page")
}

fn tg_user_id_i64(user_id: u64) -> i64 {
    i64::try_from(user_id).unwrap_or(i64::MAX)
}
