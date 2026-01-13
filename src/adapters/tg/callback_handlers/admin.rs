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
            let mut users: Vec<LiteUser> = online_users
                .read()
                .unwrap_or_else(|e| e.into_inner())
                .values()
                .cloned()
                .collect();
            users.sort_by(|a, b| a.nickname.to_lowercase().cmp(&b.nickname.to_lowercase()));

            let args = args!(server = config.teamtalk.display_name().to_string());
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

            if should_send_page(&msg, page) {
                bot.send_message(chat_id, title)
                    .reply_markup(keyboard)
                    .await?;
            } else {
                bot.edit_message_text(chat_id, msg.id, title)
                    .reply_markup(keyboard)
                    .await?;
            }
            answer_callback_empty(&bot, &q.id).await?;
        }
        AdminAction::BanList { page } => {
            let mut users: Vec<LiteUser> = online_users
                .read()
                .unwrap_or_else(|e| e.into_inner())
                .values()
                .cloned()
                .collect();
            users.sort_by(|a, b| a.nickname.to_lowercase().cmp(&b.nickname.to_lowercase()));

            let args = args!(server = config.teamtalk.display_name().to_string());
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

            if should_send_page(&msg, page) {
                bot.send_message(chat_id, title)
                    .reply_markup(keyboard)
                    .await?;
            } else {
                bot.edit_message_text(chat_id, msg.id, title)
                    .reply_markup(keyboard)
                    .await?;
            }
            answer_callback_empty(&bot, &q.id).await?;
        }
        AdminAction::KickPerform { user_id } => {
            if let Err(e) = state.tx_tt.send(TtCommand::KickUser { user_id }) {
                tracing::error!("Failed to send kick command for {}: {}", user_id, e);
                notify_admin_error(
                    &bot,
                    config,
                    q.from.id.0 as i64,
                    AdminErrorContext::TtCommand,
                    &e.to_string(),
                    lang,
                )
                .await;
            }
            answer_callback(
                &bot,
                &q.id,
                locales::get_text(lang.as_str(), "toast-command-sent", None),
                false,
            )
            .await?;
        }
        AdminAction::BanPerform { user_id } => {
            let user = online_users
                .read()
                .unwrap_or_else(|e| e.into_inner())
                .get(&user_id)
                .cloned();
            if let Some(u) = user {
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
                        AdminErrorContext::Callback,
                        &e.to_string(),
                        lang,
                    )
                    .await;
                    answer_callback(
                        &bot,
                        &q.id,
                        locales::get_text(lang.as_str(), "cmd-error", None),
                        true,
                    )
                    .await?;
                    return Ok(());
                }

                if let Some(tg_id) =
                    admin_cleanup_service::get_telegram_id_by_tt_user(db, &u.username).await
                {
                    if let Err(e) =
                        admin_cleanup_service::cleanup_deleted_banned_user(db, tg_id).await
                    {
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
                        AdminErrorContext::TtCommand,
                        &e.to_string(),
                        lang,
                    )
                    .await;
                }
                answer_callback(
                    &bot,
                    &q.id,
                    locales::get_text(lang.as_str(), "toast-command-sent", None),
                    false,
                )
                .await?;
            } else {
                answer_callback(
                    &bot,
                    &q.id,
                    locales::get_text(lang.as_str(), "cmd-no-users", None),
                    true,
                )
                .await?;
            }
        }
        AdminAction::UnbanList { page } => {
            if should_send_page(&msg, page) {
                send_unban_list(&bot, chat_id, db, lang, 0).await?;
            } else {
                edit_unban_list(&bot, &msg, db, lang, page).await?;
            }
            answer_callback_empty(&bot, &q.id).await?;
        }
        AdminAction::UnbanPerform { ban_db_id, page } => {
            if check_db_err(
                &bot,
                &q.id.0,
                db.remove_ban_by_id(ban_db_id).await,
                config,
                q.from.id.0 as i64,
                AdminErrorContext::Callback,
                lang,
            )
            .await?
            {
                return Ok(());
            }
            answer_callback(
                &bot,
                &q.id,
                locales::get_text(lang.as_str(), "toast-user-unbanned", None),
                false,
            )
            .await?;
            edit_unban_list(&bot, &msg, db, lang, page).await?;
        }
        AdminAction::SubsList { page } => {
            if should_send_page(&msg, page) {
                send_subscribers_list(&bot, chat_id, db, lang, 0).await?;
            } else {
                edit_subscribers_list(&bot, &msg, db, lang, page).await?;
            }
            answer_callback_empty(&bot, &q.id).await?;
        }
    }
    Ok(())
}

fn should_send_page(msg: &Message, page: usize) -> bool {
    page == 0 && !msg.text().unwrap_or("").contains("Page")
}
