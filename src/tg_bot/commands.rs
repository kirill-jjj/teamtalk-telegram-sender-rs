use crate::args;
use crate::locales;
use crate::tg_bot::admin_logic::bans::send_unban_list;
use crate::tg_bot::admin_logic::subscribers::send_subscribers_list;
use crate::tg_bot::callbacks_types::{AdminAction, CallbackAction, UnsubAction};
use crate::tg_bot::keyboards::{
    confirm_cancel_keyboard, create_main_menu_keyboard, create_user_list_keyboard,
};
use crate::tg_bot::settings_logic::send_main_settings;
use crate::tg_bot::state::AppState;
use crate::tg_bot::utils::{ensure_subscribed, notify_admin_error, send_text_key};
use crate::types::{LanguageCode, LiteUser, TtCommand};
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "Available Commands:")]
pub enum Command {
    #[command(description = "Start")]
    Start(String),
    #[command(description = "Main Menu")]
    Menu,
    #[command(description = "Help")]
    Help,
    #[command(description = "Who is online")]
    Who,
    #[command(description = "Settings")]
    Settings,
    #[command(description = "Unsubscribe")]
    Unsub,
    #[command(description = "Kick (Admin)")]
    Kick,
    #[command(description = "Ban (Admin)")]
    Ban,
    #[command(description = "Unban (Admin)")]
    Unban,
    #[command(description = "Subscribers (Admin)")]
    Subscribers,
    #[command(description = "Exit (Admin)")]
    Exit,
}

pub async fn answer_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: AppState,
) -> ResponseResult<()> {
    let user = if let Some(user) = &msg.from {
        user
    } else {
        return Ok(());
    };
    let telegram_id = user.id.0 as i64;

    let db = &state.db;
    let config = &state.config;
    let online_users = &state.online_users;
    let tx_tt = &state.tx_tt;

    let default_lang =
        LanguageCode::from_str_or_default(&config.general.default_lang, LanguageCode::En);
    let settings = match db.get_or_create_user(telegram_id, default_lang).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to get or create user {}: {}", telegram_id, e);
            notify_admin_error(
                &bot,
                config,
                telegram_id,
                "admin-error-context-command",
                &e.to_string(),
                default_lang,
            )
            .await;
            send_text_key(&bot, msg.chat.id, default_lang, "cmd-error").await?;
            return Ok(());
        }
    };
    let lang = LanguageCode::from_str_or_default(&settings.language_code, default_lang);
    let is_admin = match db.get_all_admins().await {
        Ok(admins) => admins.contains(&telegram_id),
        Err(e) => {
            tracing::error!("Failed to load admin list: {}", e);
            false
        }
    };

    match cmd {
        Command::Start(token) => {
            if !token.is_empty() {
                match db.resolve_deeplink(&token).await {
                    Ok(Some(deeplink)) => {
                        if let Some(expected_id) = deeplink.expected_telegram_id
                            && expected_id != telegram_id
                        {
                            send_text_key(&bot, msg.chat.id, lang, "cmd-invalid-deeplink").await?;
                            return Ok(());
                        }
                        match deeplink.action.as_str() {
                            "subscribe" => {
                                let is_banned = match db.is_telegram_id_banned(telegram_id).await {
                                    Ok(val) => val,
                                    Err(e) => {
                                        tracing::error!(
                                            "DB error checking ban for {}: {}",
                                            telegram_id,
                                            e
                                        );
                                        notify_admin_error(
                                            &bot,
                                            config,
                                            telegram_id,
                                            "admin-error-context-command",
                                            &e.to_string(),
                                            lang,
                                        )
                                        .await;
                                        false
                                    }
                                };
                                if is_banned {
                                    send_text_key(&bot, msg.chat.id, lang, "cmd-user-banned")
                                        .await?;
                                    return Ok(());
                                }

                                if let Some(tt_nick) = &deeplink.payload {
                                    let is_tt_banned =
                                        match db.is_teamtalk_username_banned(tt_nick).await {
                                            Ok(val) => val,
                                            Err(e) => {
                                                tracing::error!(
                                                    "DB error checking TT ban for {}: {}",
                                                    tt_nick,
                                                    e
                                                );
                                                notify_admin_error(
                                                    &bot,
                                                    config,
                                                    telegram_id,
                                                    "admin-error-context-command",
                                                    &e.to_string(),
                                                    lang,
                                                )
                                                .await;
                                                false
                                            }
                                        };
                                    if is_tt_banned {
                                        let args = args!(name = tt_nick.clone());
                                        bot.send_message(
                                            msg.chat.id,
                                            locales::get_text(
                                                lang.as_str(),
                                                "cmd-tt-banned",
                                                args.as_ref(),
                                            ),
                                        )
                                        .await?;
                                        return Ok(());
                                    }
                                }

                                if let Err(e) = db.add_subscriber(telegram_id).await {
                                    tracing::error!("DB error adding sub: {}", e);
                                    notify_admin_error(
                                        &bot,
                                        config,
                                        telegram_id,
                                        "admin-error-context-command",
                                        &e.to_string(),
                                        lang,
                                    )
                                    .await;
                                    send_text_key(&bot, msg.chat.id, lang, "cmd-error").await?;
                                    return Ok(());
                                }

                                if let Some(tt_nick) = deeplink.payload {
                                    if let Err(e) = db.link_tt_account(telegram_id, &tt_nick).await
                                    {
                                        tracing::error!("DB error linking: {}", e);
                                        notify_admin_error(
                                            &bot,
                                            config,
                                            telegram_id,
                                            "admin-error-context-command",
                                            &e.to_string(),
                                            lang,
                                        )
                                        .await;
                                        send_text_key(&bot, msg.chat.id, lang, "cmd-error").await?;
                                        return Ok(());
                                    }
                                    let msg_key = "cmd-success-sub";
                                    send_text_key(&bot, msg.chat.id, lang, msg_key).await?;
                                } else {
                                    let msg_key = "cmd-success-sub-guest";
                                    bot.send_message(
                                        msg.chat.id,
                                        locales::get_text(lang.as_str(), msg_key, None),
                                    )
                                    .parse_mode(ParseMode::Html)
                                    .await?;
                                }
                            }
                            "unsubscribe" => {
                                if let Err(e) = db.delete_user_profile(telegram_id).await {
                                    tracing::error!("DB error unsubscribing: {}", e);
                                    notify_admin_error(
                                        &bot,
                                        config,
                                        telegram_id,
                                        "admin-error-context-command",
                                        &e.to_string(),
                                        lang,
                                    )
                                    .await;
                                    send_text_key(&bot, msg.chat.id, lang, "cmd-error").await?;
                                    return Ok(());
                                }
                                send_text_key(&bot, msg.chat.id, lang, "cmd-success-unsub").await?;
                            }
                            _ => {
                                send_text_key(&bot, msg.chat.id, lang, "cmd-invalid-deeplink")
                                    .await?;
                            }
                        }
                    }
                    Ok(None) => {
                        send_text_key(&bot, msg.chat.id, lang, "cmd-invalid-deeplink").await?;
                    }
                    Err(e) => {
                        tracing::error!("DB error resolving deeplink: {}", e);
                        notify_admin_error(
                            &bot,
                            config,
                            telegram_id,
                            "admin-error-context-command",
                            &e.to_string(),
                            lang,
                        )
                        .await;
                        send_text_key(&bot, msg.chat.id, lang, "cmd-error").await?;
                    }
                }
            } else {
                send_text_key(&bot, msg.chat.id, lang, "hello-start").await?;
            }
        }
        Command::Menu => {
            if !ensure_subscribed(&bot, &msg, db, config, lang).await {
                return Ok(());
            }
            let keyboard = create_main_menu_keyboard(lang, is_admin);
            bot.send_message(
                msg.chat.id,
                locales::get_text(lang.as_str(), "menu-title", None),
            )
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await?;
        }
        Command::Help => {
            if !ensure_subscribed(&bot, &msg, db, config, lang).await {
                return Ok(());
            }
            bot.send_message(
                msg.chat.id,
                locales::get_text(lang.as_str(), "help-text", None),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        Command::Who => {
            if !ensure_subscribed(&bot, &msg, db, config, lang).await {
                return Ok(());
            }
            if let Err(e) = tx_tt.send(TtCommand::Who {
                chat_id: msg.chat.id.0,
                lang,
            }) {
                tracing::error!("Failed to send TT who command: {}", e);
                notify_admin_error(
                    &bot,
                    config,
                    telegram_id,
                    "admin-error-context-tt-command",
                    &e.to_string(),
                    lang,
                )
                .await;
            }
        }
        Command::Settings => {
            if !ensure_subscribed(&bot, &msg, db, config, lang).await {
                return Ok(());
            }
            send_main_settings(&bot, msg.chat.id, lang).await?;
        }
        Command::Unsub => {
            if !ensure_subscribed(&bot, &msg, db, config, lang).await {
                return Ok(());
            }
            let text = locales::get_text(lang.as_str(), "unsub-confirm-text", None);
            let keyboard = confirm_cancel_keyboard(
                lang,
                "btn-yes",
                CallbackAction::Unsub(UnsubAction::Confirm),
                "btn-no",
                CallbackAction::Unsub(UnsubAction::Cancel),
            );

            bot.send_message(msg.chat.id, text)
                .reply_markup(keyboard)
                .await?;
        }
        Command::Kick | Command::Ban => {
            if !is_admin {
                send_text_key(&bot, msg.chat.id, lang, "cmd-unauth").await?;
                return Ok(());
            }

            let mut users: Vec<LiteUser> = online_users.iter().map(|u| u.value().clone()).collect();
            users.sort_by(|a, b| a.nickname.to_lowercase().cmp(&b.nickname.to_lowercase()));

            let is_kick = matches!(cmd, Command::Kick);
            let title_key = if is_kick {
                "list-kick-title"
            } else {
                "list-ban-title"
            };

            let args = args!(server = config.teamtalk.display_name().to_string());
            let title = locales::get_text(lang.as_str(), title_key, args.as_ref());

            let keyboard = create_user_list_keyboard(
                &users,
                0,
                move |u| {
                    let action = if is_kick {
                        AdminAction::KickPerform { user_id: u.id }
                    } else {
                        AdminAction::BanPerform { user_id: u.id }
                    };
                    (u.nickname.clone(), CallbackAction::Admin(action))
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
                lang,
            );

            bot.send_message(msg.chat.id, title)
                .reply_markup(keyboard)
                .await?;
        }
        Command::Unban => {
            if !is_admin {
                send_text_key(&bot, msg.chat.id, lang, "cmd-unauth").await?;
                return Ok(());
            }
            send_unban_list(&bot, msg.chat.id, db, lang, 0).await?;
        }
        Command::Subscribers => {
            if !is_admin {
                send_text_key(&bot, msg.chat.id, lang, "cmd-unauth").await?;
                return Ok(());
            }
            send_subscribers_list(&bot, msg.chat.id, db, lang, 0).await?;
        }
        Command::Exit => {
            if !is_admin {
                send_text_key(&bot, msg.chat.id, lang, "cmd-unauth").await?;
                return Ok(());
            }
            bot.send_message(
                msg.chat.id,
                locales::get_text(lang.as_str(), "cmd-shutting-down", None),
            )
            .await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            std::process::exit(0);
        }
    }
    Ok(())
}
