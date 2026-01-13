use crate::adapters::tg::admin_logic::bans::send_unban_list;
use crate::adapters::tg::admin_logic::subscribers::send_subscribers_list;
use crate::adapters::tg::keyboards::{
    confirm_cancel_keyboard, create_main_menu_keyboard, create_user_list_keyboard,
};
use crate::adapters::tg::settings_logic::send_main_settings;
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{ensure_subscribed, notify_admin_error, send_text_key};
use crate::app::services::deeplink as deeplink_service;
use crate::app::services::pending as pending_service;
use crate::app::services::subscription as subscription_service;
use crate::app::services::user_settings as user_settings_service;
use crate::args;
use crate::core::callbacks::{AdminAction, CallbackAction, UnsubAction};
use crate::core::types::{AdminErrorContext, DeeplinkAction, LanguageCode, LiteUser, TtCommand};
use crate::infra::locales;
use std::time::{SystemTime, UNIX_EPOCH};
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::types::{ParseMode, Voice};
use teloxide::utils::command::BotCommands;
use tokio::fs::File;

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
    let settings = match user_settings_service::get_or_create(db, telegram_id, default_lang).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to get or create user {}: {}", telegram_id, e);
            notify_admin_error(
                &bot,
                config,
                telegram_id,
                AdminErrorContext::Command,
                &e.to_string(),
                default_lang,
            )
            .await;
            send_text_key(&bot, msg.chat.id, default_lang, "cmd-error").await?;
            return Ok(());
        }
    };
    let lang = LanguageCode::from_str_or_default(&settings.language_code, default_lang);
    let is_admin = if telegram_id == config.telegram.admin_chat_id {
        true
    } else {
        match db.get_all_admins().await {
            Ok(admins) => admins.contains(&telegram_id),
            Err(e) => {
                tracing::error!("Failed to load admin list: {}", e);
                false
            }
        }
    };

    match cmd {
        Command::Start(token) => {
            if !token.is_empty() {
                match deeplink_service::resolve_for_user(db, &token, telegram_id).await {
                    Ok(Some(deeplink)) => match deeplink.action {
                        DeeplinkAction::Subscribe => {
                            match subscription_service::subscribe_via_deeplink(
                                db,
                                telegram_id,
                                deeplink.payload,
                            )
                            .await
                            {
                                Ok(subscription_service::SubscribeOutcome::BannedUser) => {
                                    send_text_key(&bot, msg.chat.id, lang, "cmd-user-banned")
                                        .await?;
                                    return Ok(());
                                }
                                Ok(subscription_service::SubscribeOutcome::BannedTeamTalk {
                                    username,
                                }) => {
                                    let args = args!(name = username);
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
                                Ok(subscription_service::SubscribeOutcome::SubscribedLinked) => {
                                    let msg_key = "cmd-success-sub";
                                    send_text_key(&bot, msg.chat.id, lang, msg_key).await?;
                                }
                                Ok(subscription_service::SubscribeOutcome::SubscribedGuest) => {
                                    let msg_key = "cmd-success-sub-guest";
                                    bot.send_message(
                                        msg.chat.id,
                                        locales::get_text(lang.as_str(), msg_key, None),
                                    )
                                    .parse_mode(ParseMode::Html)
                                    .await?;
                                }
                                Err(e) => {
                                    tracing::error!("DB error adding sub: {}", e);
                                    notify_admin_error(
                                        &bot,
                                        config,
                                        telegram_id,
                                        AdminErrorContext::Command,
                                        &e.to_string(),
                                        lang,
                                    )
                                    .await;
                                    send_text_key(&bot, msg.chat.id, lang, "cmd-error").await?;
                                    return Ok(());
                                }
                            }
                        }
                        DeeplinkAction::Unsubscribe => {
                            if let Err(e) = subscription_service::unsubscribe(db, telegram_id).await
                            {
                                tracing::error!("DB error unsubscribing: {}", e);
                                notify_admin_error(
                                    &bot,
                                    config,
                                    telegram_id,
                                    AdminErrorContext::Command,
                                    &e.to_string(),
                                    lang,
                                )
                                .await;
                                send_text_key(&bot, msg.chat.id, lang, "cmd-error").await?;
                                return Ok(());
                            }
                            send_text_key(&bot, msg.chat.id, lang, "cmd-success-unsub").await?;
                        }
                    },
                    Ok(None) => {
                        send_text_key(&bot, msg.chat.id, lang, "cmd-invalid-deeplink").await?;
                    }
                    Err(e) => {
                        tracing::error!("DB error resolving deeplink: {}", e);
                        notify_admin_error(
                            &bot,
                            config,
                            telegram_id,
                            AdminErrorContext::Command,
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
                    AdminErrorContext::TtCommand,
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

            let mut users: Vec<LiteUser> = online_users
                .read()
                .unwrap_or_else(|e| e.into_inner())
                .values()
                .cloned()
                .collect();
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
            let _ = state.tx_tt.send(TtCommand::Shutdown);
            state.cancel_token.cancel();
        }
    }
    Ok(())
}

pub async fn answer_message(bot: Bot, msg: Message, state: AppState) -> ResponseResult<()> {
    let user = if let Some(user) = &msg.from {
        user
    } else {
        return Ok(());
    };
    let telegram_id = user.id.0 as i64;
    let config = &state.config;
    let db = &state.db;

    let is_admin = if telegram_id == config.telegram.admin_chat_id {
        true
    } else {
        match db.get_all_admins().await {
            Ok(admins) => admins.contains(&telegram_id),
            Err(e) => {
                tracing::error!("Failed to load admin list: {}", e);
                false
            }
        }
    };
    if !is_admin {
        return Ok(());
    }

    let default_lang =
        LanguageCode::from_str_or_default(&config.general.default_lang, LanguageCode::En);
    let admin_lang = user_settings_service::get_or_create(db, telegram_id, default_lang)
        .await
        .map(|u| LanguageCode::from_str_or_default(&u.language_code, default_lang))
        .unwrap_or(default_lang);

    let reply_to = msg.reply_to_message();
    let text = msg.text();
    let voice = msg.voice();

    if reply_to.is_none() {
        if let Some(voice) = voice {
            let reply_key = match stream_voice(&bot, &state, None, voice).await {
                Ok(_) => "tg-reply-sent",
                Err(e) => {
                    notify_admin_error(
                        &bot,
                        config,
                        telegram_id,
                        AdminErrorContext::Command,
                        &e.to_string(),
                        admin_lang,
                    )
                    .await;
                    "tg-reply-failed"
                }
            };
            let reply_text = locales::get_text(admin_lang.as_str(), reply_key, None);
            let _ = bot
                .send_message(msg.chat.id, reply_text)
                .reply_to(msg.id)
                .await;
        }
        return Ok(());
    }

    let reply_to = match reply_to {
        Some(reply_to) => reply_to,
        None => return Ok(()),
    };
    let reply_id = reply_to.id.0 as i64;

    if let Ok(Some((channel_id, _channel_name, _server_name, original_text))) =
        pending_service::get_pending_channel_reply(db, reply_id).await
    {
        let mut reply_key = "tg-reply-sent";
        if let Some(voice) = voice {
            let duration = format_duration(voice.duration.seconds());
            let args = args!(msg = original_text.clone(), duration = duration);
            let announce_text =
                locales::get_text(admin_lang.as_str(), "tt-channel-reply", args.as_ref());
            if let Err(e) =
                stream_voice(&bot, &state, Some((channel_id, announce_text)), voice).await
            {
                notify_admin_error(
                    &bot,
                    config,
                    telegram_id,
                    AdminErrorContext::Command,
                    &e.to_string(),
                    admin_lang,
                )
                .await;
                reply_key = "tg-reply-failed";
            }
        } else if let Some(text) = text {
            let args = args!(msg = original_text.clone(), reply = text.to_string());
            let channel_text =
                locales::get_text(admin_lang.as_str(), "tt-channel-reply-text", args.as_ref());
            if let Err(e) = state.tx_tt.send(TtCommand::SendToChannel {
                channel_id,
                text: channel_text,
            }) {
                tracing::error!("Failed to send TT channel reply to {}: {}", channel_id, e);
                notify_admin_error(
                    &bot,
                    config,
                    telegram_id,
                    AdminErrorContext::Command,
                    &e.to_string(),
                    admin_lang,
                )
                .await;
                reply_key = "tg-reply-failed";
            }
        } else {
            return Ok(());
        }

        let reply_text = locales::get_text(admin_lang.as_str(), reply_key, None);
        let _ = bot
            .send_message(msg.chat.id, reply_text)
            .reply_to(msg.id)
            .await;

        if let Err(e) = pending_service::touch_pending_channel_reply(db, reply_id).await {
            tracing::error!("Failed to update pending channel reply {}: {}", reply_id, e);
        }

        return Ok(());
    }

    let text = if let Some(text) = text {
        text
    } else {
        return Ok(());
    };

    let tt_user_id = match pending_service::get_pending_reply_user_id(db, reply_id).await {
        Ok(Some(id)) => id,
        Ok(None) => return Ok(()),
        Err(e) => {
            tracing::error!("Failed to load pending reply {}: {}", reply_id, e);
            notify_admin_error(
                &bot,
                config,
                telegram_id,
                AdminErrorContext::Command,
                &e.to_string(),
                LanguageCode::from_str_or_default(&config.general.default_lang, LanguageCode::En),
            )
            .await;
            return Ok(());
        }
    };

    let is_online = state
        .online_users
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .contains_key(&tt_user_id);
    let reply_key = if !is_online {
        "tg-reply-offline"
    } else {
        let send_res = state.tx_tt.send(TtCommand::ReplyToUser {
            user_id: tt_user_id,
            text: text.to_string(),
        });
        if let Err(e) = send_res {
            tracing::error!("Failed to send TT reply command for {}: {}", tt_user_id, e);
            notify_admin_error(
                &bot,
                config,
                telegram_id,
                AdminErrorContext::Command,
                &e.to_string(),
                admin_lang,
            )
            .await;
            "tg-reply-failed"
        } else {
            "tg-reply-sent"
        }
    };
    let reply_text = locales::get_text(admin_lang.as_str(), reply_key, None);
    let _ = bot
        .send_message(msg.chat.id, reply_text)
        .reply_to(msg.id)
        .await;

    if let Err(e) = pending_service::touch_pending_reply(db, reply_id).await {
        tracing::error!("Failed to update pending reply {}: {}", reply_id, e);
    }

    Ok(())
}

fn format_duration(duration_secs: u32) -> String {
    let minutes = duration_secs / 60;
    let seconds = duration_secs % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

async fn stream_voice(
    bot: &Bot,
    state: &AppState,
    announce: Option<(i32, String)>,
    voice: &Voice,
) -> Result<(), String> {
    let file_info = bot
        .get_file(voice.file.id.clone())
        .await
        .map_err(|e| e.to_string())?;
    let mut temp_path = std::env::temp_dir();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    temp_path.push(format!("tg-voice-{}-{}.ogg", voice.file.id, now));
    let mut dst = File::create(&temp_path).await.map_err(|e| e.to_string())?;
    bot.download_file(&file_info.path, &mut dst)
        .await
        .map_err(|e| e.to_string())?;

    let duration_ms = voice.duration.seconds().saturating_mul(1000);
    let (channel_id, announce_text) = announce
        .map(|(id, text)| (id, Some(text)))
        .unwrap_or((0, None));
    state
        .tx_tt
        .send(TtCommand::EnqueueStream {
            channel_id,
            file_path: temp_path.to_string_lossy().to_string(),
            duration_ms,
            announce_text,
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}
