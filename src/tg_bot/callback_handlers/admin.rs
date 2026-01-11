use crate::args;
use crate::locales;
use crate::tg_bot::admin_logic::bans::{edit_unban_list, send_unban_list};
use crate::tg_bot::admin_logic::subscribers::{edit_subscribers_list, send_subscribers_list};
use crate::tg_bot::callbacks_types::{AdminAction, CallbackAction};
use crate::tg_bot::keyboards::create_user_list_keyboard;
use crate::tg_bot::state::AppState;
use crate::tg_bot::utils::{check_db_err, notify_admin_error};
use crate::types::{LiteUser, TtCommand};
use teloxide::prelude::*;

pub async fn handle_admin(
    bot: Bot,
    q: CallbackQuery,
    state: AppState,
    action: AdminAction,
    lang: &str,
) -> ResponseResult<()> {
    let msg = match q.message {
        Some(teloxide::types::MaybeInaccessibleMessage::Regular(m)) => m,
        _ => return Ok(()),
    };
    let chat_id = msg.chat.id;
    let db = &state.db;
    let online_users = &state.online_users;
    let config = &state.config;

    match action {
        AdminAction::KickList { page } => {
            let mut users: Vec<LiteUser> = online_users.iter().map(|u| u.value().clone()).collect();
            users.sort_by(|a, b| a.nickname.to_lowercase().cmp(&b.nickname.to_lowercase()));

            let args = args!(server = config.teamtalk.display_name().to_string());
            let title = locales::get_text(lang, "list-kick-title", args.as_ref());

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

            if page == 0 && !msg.text().unwrap_or("").contains("Page") {
                bot.send_message(chat_id, title)
                    .reply_markup(keyboard)
                    .await?;
            } else {
                bot.edit_message_text(chat_id, msg.id, title)
                    .reply_markup(keyboard)
                    .await?;
            }
            bot.answer_callback_query(q.id).await?;
        }
        AdminAction::BanList { page } => {
            let mut users: Vec<LiteUser> = online_users.iter().map(|u| u.value().clone()).collect();
            users.sort_by(|a, b| a.nickname.to_lowercase().cmp(&b.nickname.to_lowercase()));

            let args = args!(server = config.teamtalk.display_name().to_string());
            let title = locales::get_text(lang, "list-ban-title", args.as_ref());

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

            if page == 0 && !msg.text().unwrap_or("").contains("Page") {
                bot.send_message(chat_id, title)
                    .reply_markup(keyboard)
                    .await?;
            } else {
                bot.edit_message_text(chat_id, msg.id, title)
                    .reply_markup(keyboard)
                    .await?;
            }
            bot.answer_callback_query(q.id).await?;
        }
        AdminAction::KickPerform { user_id } => {
            if let Err(e) = state.tx_tt.send(TtCommand::KickUser { user_id }) {
                tracing::error!("Failed to send kick command for {}: {}", user_id, e);
                notify_admin_error(
                    &bot,
                    config,
                    q.from.id.0 as i64,
                    "admin-error-context-tt-command",
                    &e.to_string(),
                    lang,
                )
                .await;
            }
            bot.answer_callback_query(q.id)
                .text(locales::get_text(lang, "toast-command-sent", None))
                .await?;
        }
        AdminAction::BanPerform { user_id } => {
            if let Some(u) = online_users.get(&user_id) {
                if let Err(e) = db
                    .add_ban(
                        None,
                        Some(u.username.clone()),
                        Some("Banned via Telegram".to_string()),
                    )
                    .await
                {
                    tracing::error!("Failed to add ban: {}", e);
                    notify_admin_error(
                        &bot,
                        config,
                        q.from.id.0 as i64,
                        "admin-error-context-callback",
                        &e.to_string(),
                        lang,
                    )
                    .await;
                    bot.answer_callback_query(q.id)
                        .text(locales::get_text(lang, "cmd-error", None))
                        .show_alert(true)
                        .await?;
                    return Ok(());
                }

                if let Ok(Some(tg_id)) = sqlx::query_scalar::<_, i64>(
                    "SELECT telegram_id FROM user_settings WHERE teamtalk_username = ?",
                )
                .bind(&u.username)
                .fetch_optional(&db.pool)
                .await
                {
                    if let Err(e) = db.delete_user_profile(tg_id).await {
                        tracing::error!("Failed to delete user profile during ban: {}", e);
                    }
                    if let Err(e) = db
                        .add_ban(
                            Some(tg_id),
                            Some(u.username.clone()),
                            Some("TG+TT Ban".to_string()),
                        )
                        .await
                    {
                        tracing::error!("Failed to add second ban record: {}", e);
                    }
                }
                if let Err(e) = state.tx_tt.send(TtCommand::BanUser { user_id }) {
                    tracing::error!("Failed to send ban command for {}: {}", user_id, e);
                    notify_admin_error(
                        &bot,
                        config,
                        q.from.id.0 as i64,
                        "admin-error-context-tt-command",
                        &e.to_string(),
                        lang,
                    )
                    .await;
                }
                bot.answer_callback_query(q.id)
                    .text(locales::get_text(lang, "toast-command-sent", None))
                    .await?;
            } else {
                bot.answer_callback_query(q.id)
                    .text(locales::get_text(lang, "cmd-no-users", None))
                    .show_alert(true)
                    .await?;
            }
        }
        AdminAction::UnbanList { page } => {
            if page == 0 && !msg.text().unwrap_or("").contains("Page") {
                send_unban_list(&bot, chat_id, db, lang, 0).await?;
            } else {
                edit_unban_list(&bot, &msg, db, lang, page).await?;
            }
            bot.answer_callback_query(q.id).await?;
        }
        AdminAction::UnbanPerform { ban_db_id, page } => {
            if check_db_err(
                &bot,
                &q.id.0,
                db.remove_ban_by_id(ban_db_id).await,
                config,
                q.from.id.0 as i64,
                "admin-error-context-callback",
                lang,
            )
            .await?
            {
                return Ok(());
            }
            bot.answer_callback_query(q.id)
                .text(locales::get_text(lang, "toast-user-unbanned", None))
                .await?;
            edit_unban_list(&bot, &msg, db, lang, page).await?;
        }
        AdminAction::SubsList { page } => {
            if page == 0 && !msg.text().unwrap_or("").contains("Page") {
                send_subscribers_list(&bot, chat_id, db, lang, 0).await?;
            } else {
                edit_subscribers_list(&bot, &msg, db, lang, page).await?;
            }
            bot.answer_callback_query(q.id).await?;
        }
    }
    Ok(())
}
