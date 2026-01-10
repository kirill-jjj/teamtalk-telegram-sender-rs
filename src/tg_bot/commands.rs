use crate::args;
use crate::locales;
use crate::tg_bot::admin_logic::bans::send_unban_list;
use crate::tg_bot::admin_logic::subscribers::send_subscribers_list;
use crate::tg_bot::callbacks_types::{AdminAction, CallbackAction};
use crate::tg_bot::keyboards::{create_main_menu_keyboard, create_user_list_keyboard};
use crate::tg_bot::settings_logic::send_main_settings;
use crate::tg_bot::state::AppState;
use crate::tg_bot::utils::ensure_subscribed;
use crate::types::{LiteUser, TtCommand};
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
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

    let settings = match db
        .get_or_create_user(telegram_id, &config.general.default_lang)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to get or create user {}: {}", telegram_id, e);
            bot.send_message(msg.chat.id, "Database error. Please try again later.")
                .await?;
            return Ok(());
        }
    };
    let lang = &settings.language_code;
    let is_admin = db
        .get_all_admins()
        .await
        .unwrap_or_default()
        .contains(&telegram_id);

    match cmd {
        Command::Start(token) => {
            if !token.is_empty() {
                if let Ok(Some(deeplink)) = db.resolve_deeplink(&token).await {
                    match deeplink.action.as_str() {
                        "subscribe" => {
                            if db.is_telegram_id_banned(telegram_id).await.unwrap_or(false) {
                                bot.send_message(
                                    msg.chat.id,
                                    locales::get_text(lang, "cmd-user-banned", None),
                                )
                                .await?;
                                return Ok(());
                            }

                            if let Some(tt_nick) = &deeplink.payload
                                && db
                                    .is_teamtalk_username_banned(tt_nick)
                                    .await
                                    .unwrap_or(false)
                            {
                                let args = args!(name = tt_nick.clone());
                                bot.send_message(
                                    msg.chat.id,
                                    locales::get_text(lang, "cmd-tt-banned", args.as_ref()),
                                )
                                .await?;
                                return Ok(());
                            }

                            db.add_subscriber(telegram_id).await.ok();

                            if let Some(tt_nick) = deeplink.payload {
                                db.link_tt_account(telegram_id, &tt_nick).await.ok();
                                let msg_key = "cmd-success-sub";
                                bot.send_message(
                                    msg.chat.id,
                                    locales::get_text(lang, msg_key, None),
                                )
                                .await?;
                            } else {
                                let msg_key = "cmd-success-sub-guest";
                                bot.send_message(
                                    msg.chat.id,
                                    locales::get_text(lang, msg_key, None),
                                )
                                .parse_mode(ParseMode::Html)
                                .await?;
                            }
                        }
                        "unsubscribe" => {
                            db.delete_user_profile(telegram_id).await.ok();
                            bot.send_message(
                                msg.chat.id,
                                locales::get_text(lang, "cmd-success-unsub", None),
                            )
                            .await?;
                        }
                        _ => {
                            bot.send_message(
                                msg.chat.id,
                                locales::get_text(lang, "cmd-invalid-deeplink", None),
                            )
                            .await?;
                        }
                    }
                } else {
                    bot.send_message(
                        msg.chat.id,
                        locales::get_text(lang, "cmd-invalid-deeplink", None),
                    )
                    .await?;
                }
            } else {
                bot.send_message(msg.chat.id, locales::get_text(lang, "hello-start", None))
                    .await?;
            }
        }
        Command::Menu => {
            if !ensure_subscribed(&bot, &msg, db, lang).await {
                return Ok(());
            }
            let keyboard = create_main_menu_keyboard(lang, is_admin);
            bot.send_message(msg.chat.id, locales::get_text(lang, "menu-title", None))
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboard)
                .await?;
        }
        Command::Help => {
            if !ensure_subscribed(&bot, &msg, db, lang).await {
                return Ok(());
            }
            bot.send_message(msg.chat.id, locales::get_text(lang, "help-text", None))
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Command::Who => {
            if !ensure_subscribed(&bot, &msg, db, lang).await {
                return Ok(());
            }
            let _ = tx_tt.send(TtCommand::Who {
                chat_id: msg.chat.id.0,
                lang: lang.clone(),
            });
        }
        Command::Settings => {
            if !ensure_subscribed(&bot, &msg, db, lang).await {
                return Ok(());
            }
            send_main_settings(&bot, msg.chat.id, lang).await?;
        }
        Command::Unsub => {
            if !ensure_subscribed(&bot, &msg, db, lang).await {
                return Ok(());
            }
            let text = locales::get_text(lang, "unsub-confirm-text", None);
            let keyboard = InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::callback(
                    locales::get_text(lang, "btn-yes", None),
                    "unsub_confirm",
                ),
                InlineKeyboardButton::callback(
                    locales::get_text(lang, "btn-no", None),
                    "unsub_cancel",
                ),
            ]]);

            bot.send_message(msg.chat.id, text)
                .reply_markup(keyboard)
                .await?;
        }
        Command::Kick | Command::Ban => {
            if !is_admin {
                bot.send_message(msg.chat.id, locales::get_text(lang, "cmd-unauth", None))
                    .await?;
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
            let title = locales::get_text(lang, title_key, args.as_ref());

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
                bot.send_message(msg.chat.id, locales::get_text(lang, "cmd-unauth", None))
                    .await?;
                return Ok(());
            }
            send_unban_list(&bot, msg.chat.id, db, lang, 0).await?;
        }
        Command::Subscribers => {
            if !is_admin {
                bot.send_message(msg.chat.id, locales::get_text(lang, "cmd-unauth", None))
                    .await?;
                return Ok(());
            }
            send_subscribers_list(&bot, msg.chat.id, db, lang, 0).await?;
        }
        Command::Exit => {
            if !is_admin {
                bot.send_message(msg.chat.id, locales::get_text(lang, "cmd-unauth", None))
                    .await?;
                return Ok(());
            }
            bot.send_message(
                msg.chat.id,
                locales::get_text(lang, "cmd-shutting-down", None),
            )
            .await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            std::process::exit(0);
        }
    }
    Ok(())
}
