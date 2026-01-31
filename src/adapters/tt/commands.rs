#![allow(clippy::pedantic, clippy::nursery)]

use crate::adapters::tt::{WorkerContext, resolve_channel_name, resolve_server_name};
use crate::args;
use crate::core::types::{DeeplinkAction, LanguageCode, TtCommand};
use crate::infra::locales;
use teamtalk::Client;
use teamtalk::types::TextMessage;
use tokio::task::spawn_local;
use uuid::Uuid;

pub(super) fn handle_text_message(client: &Client, ctx: &WorkerContext, msg: TextMessage) {
    if msg.from_id == client.my_id() {
        return;
    }

    let real_name_from_client = client.get_server_properties().map(|p| p.name);
    let tx_tt_cmd = ctx.tx_tt_cmd.clone();

    let db = ctx.db.clone();
    let online_users = ctx.online_users.clone();

    let default_lang = ctx.config.general.default_lang;
    let admin_username = ctx.config.general.admin_username.clone();
    let tt_config = ctx.config.teamtalk.clone();
    let deeplink_ttl = ctx.config.operational_parameters.deeplink_ttl;

    let bot_username = ctx.bot_username.clone();
    let tx_bridge = ctx.tx_bridge.clone();
    let tt_msg_sem = ctx.tt_msg_sem.clone();
    let tt_lang_cache = ctx.tt_lang_cache.clone();
    let tt_tg_cache = ctx.tt_tg_cache.clone();
    let tt_cache_stats = ctx.tt_cache_stats.clone();

    if msg.msg_type == teamtalk::client::ffi::TextMsgType::MSGTYPE_CHANNEL {
        let content = msg.text.trim();
        let cmd = content
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_lowercase();
        if cmd == "/skip" {
            let from_uid = msg.from_id.0;
            let channel_id = msg.channel_id.0;
            let db = db.clone();
            let online_users = online_users.clone();
            let tx_tt_cmd = tx_tt_cmd.clone();
            let tt_lang_cache = tt_lang_cache.clone();
            let tt_tg_cache = tt_tg_cache.clone();
            spawn_local(async move {
                let _permit = tt_msg_sem.acquire_owned().await;
                let username = if let Ok(users) = online_users.read() {
                    users
                        .get(&from_uid)
                        .map(|u| u.username.clone())
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                let reply_lang = if username.is_empty() {
                    default_lang
                } else if let Ok(cache) = tt_lang_cache.read()
                    && let Some(lang) = cache.get(&username)
                {
                    tt_cache_stats
                        .lang_hits
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    *lang
                } else {
                    let lang = db
                        .get_user_lang_by_tt_user(&username)
                        .await
                        .unwrap_or(default_lang);
                    if let Ok(mut cache) = tt_lang_cache.write() {
                        if cache.len() > 5000 {
                            cache.clear();
                        }
                        cache.insert(username.clone(), lang);
                    }
                    tt_cache_stats
                        .lang_misses
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    lang
                };
                let is_admin = if username.is_empty() {
                    false
                } else if admin_username
                    .as_ref()
                    .map(|u| u == &username)
                    .unwrap_or(false)
                {
                    true
                } else if let Some(tg_id) = if let Ok(cache) = tt_tg_cache.read() {
                    cache.get(&username).copied()
                } else {
                    None
                } {
                    tt_cache_stats
                        .tg_hits
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    db.get_all_admins()
                        .await
                        .map(|admins| admins.contains(&tg_id))
                        .unwrap_or(false)
                } else if let Some(tg_id) = db.get_telegram_id_by_tt_user(&username).await {
                    if let Ok(mut cache) = tt_tg_cache.write() {
                        if cache.len() > 5000 {
                            cache.clear();
                        }
                        cache.insert(username.clone(), tg_id);
                    }
                    tt_cache_stats
                        .tg_misses
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    db.get_all_admins()
                        .await
                        .map(|admins| admins.contains(&tg_id))
                        .unwrap_or(false)
                } else {
                    tt_cache_stats
                        .tg_misses
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    false
                };
                let text_key = if is_admin {
                    if let Err(e) = tx_tt_cmd.send(TtCommand::SkipStream).await {
                        tracing::error!(
                            tt_username = %username,
                            error = %e,
                            "Failed to send TT skip command"
                        );
                        "tt-error-generic"
                    } else {
                        "tt-skip-sent"
                    }
                } else {
                    "cmd-unauth"
                };
                let text = locales::get_text(reply_lang.as_str(), text_key, None);
                if let Err(e) = tx_tt_cmd
                    .send(TtCommand::SendToChannel { channel_id, text })
                    .await
                {
                    tracing::error!(
                        channel_id,
                        error = %e,
                        "Failed to send TT channel reply"
                    );
                }
            });
            return;
        }
        if let Some(rest) = content.strip_prefix("/pm ") {
            let pm_text = rest.trim();
            if pm_text.is_empty() {
                return;
            }
            let channel_name = resolve_channel_name(client, msg.channel_id, LanguageCode::En);
            let server_name = resolve_server_name(&tt_config, real_name_from_client.as_deref());
            let tx_bridge = tx_bridge.clone();
            let msg_content = pm_text.to_string();
            let channel_id = msg.channel_id.0;
            spawn_local(async move {
                let _permit = tt_msg_sem.acquire_owned().await;
                if let Err(e) = tx_bridge
                    .send(crate::core::types::BridgeEvent::ToAdminChannel {
                        channel_id,
                        channel_name,
                        server_name,
                        msg_content,
                    })
                    .await
                {
                    tracing::error!(error = %e, "Failed to send channel PM event");
                }
            });
        }
        return;
    }

    let tt_lang_cache = tt_lang_cache.clone();
    let tt_cache_stats = tt_cache_stats.clone();
    spawn_local(async move {
        if msg.msg_type == teamtalk::client::ffi::TextMsgType::MSGTYPE_USER {
            let content = msg.text.trim();
            let from_uid = msg.from_id.0;

            let (nick, username): (String, String) = if let Ok(users) = online_users.read() {
                users
                    .get(&from_uid)
                    .map(|u| (u.nickname.clone(), u.username.clone()))
                    .unwrap_or(("Unknown".to_string(), "".to_string()))
            } else {
                ("Unknown".to_string(), "".to_string())
            };

            tracing::info!(
                component = "tt_worker",
                nick = %nick,
                tt_username = %username,
                "Received TT message"
            );

            let reply_lang = if username.is_empty() {
                default_lang
            } else if let Ok(cache) = tt_lang_cache.read()
                && let Some(lang) = cache.get(&username)
            {
                tt_cache_stats
                    .lang_hits
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                *lang
            } else {
                let lang = db
                    .get_user_lang_by_tt_user(&username)
                    .await
                    .unwrap_or(default_lang);
                if let Ok(mut cache) = tt_lang_cache.write() {
                    if cache.len() > 5000 {
                        cache.clear();
                    }
                    cache.insert(username.clone(), lang);
                }
                tt_cache_stats
                    .lang_misses
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                lang
            };

            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.is_empty() {
                return;
            }
            let cmd = parts[0].to_lowercase();
            let needs_heavy = matches!(
                cmd.as_str(),
                "/sub" | "/unsub" | "/skip" | "/help" | "/start"
            );
            let _permit = if needs_heavy {
                Some(tt_msg_sem.acquire_owned().await)
            } else {
                None
            };

            let send_reply = |text: String| async {
                if let Err(e) = tx_tt_cmd
                    .send(TtCommand::ReplyToUser {
                        user_id: from_uid,
                        text,
                    })
                    .await
                {
                    tracing::error!(
                        user_id = from_uid,
                        tt_username = %username,
                        error = %e,
                        "Failed to send TT reply command"
                    );
                }
            };

            if cmd == "/sub" {
                if let Some(bot_user) = &bot_username {
                    let is_guest = username.is_empty()
                        || tt_config
                            .guest_username
                            .as_ref()
                            .map(|g| g == &username)
                            .unwrap_or(false);

                    let payload = if is_guest {
                        None
                    } else {
                        Some(username.as_str())
                    };

                    let token = Uuid::now_v7().to_string().replace('-', "");
                    let expected_telegram_id = if username.is_empty() {
                        None
                    } else {
                        db.get_telegram_id_by_tt_user(&username).await
                    };
                    let res = db
                        .create_deeplink(
                            &token,
                            DeeplinkAction::Subscribe,
                            payload,
                            expected_telegram_id,
                            deeplink_ttl,
                        )
                        .await;

                    match res {
                        Ok(_) => {
                            let link = format!("https://t.me/{}?start={}", bot_user, token);
                            let text = locales::get_text(
                                reply_lang.as_str(),
                                "tt-sub-link",
                                args!(link = link).as_ref(),
                            );
                            send_reply(text).await;
                        }
                        Err(_) => {
                            let text =
                                locales::get_text(reply_lang.as_str(), "tt-error-generic", None);
                            send_reply(text).await;
                        }
                    }
                } else {
                    send_reply(
                        "Telegram integration is currently disabled (Event Token missing)."
                            .to_string(),
                    )
                    .await;
                }
            } else if cmd == "/unsub" {
                if let Some(bot_user) = &bot_username {
                    let token = Uuid::now_v7().to_string().replace('-', "");
                    let expected_telegram_id = if username.is_empty() {
                        None
                    } else {
                        db.get_telegram_id_by_tt_user(&username).await
                    };
                    let res = db
                        .create_deeplink(
                            &token,
                            DeeplinkAction::Unsubscribe,
                            None,
                            expected_telegram_id,
                            deeplink_ttl,
                        )
                        .await;

                    match res {
                        Ok(_) => {
                            let link = format!("https://t.me/{}?start={}", bot_user, token);
                            let text = locales::get_text(
                                reply_lang.as_str(),
                                "tt-unsub-link",
                                args!(link = link).as_ref(),
                            );
                            send_reply(text).await;
                        }
                        Err(_) => {
                            let text =
                                locales::get_text(reply_lang.as_str(), "tt-error-generic", None);
                            send_reply(text).await;
                        }
                    }
                } else {
                    send_reply(
                        "Telegram integration is currently disabled (Event Token missing)."
                            .to_string(),
                    )
                    .await;
                }
            } else if cmd == "/help" {
                let is_main_admin = admin_username
                    .as_ref()
                    .map(|u| u == &username)
                    .unwrap_or(false);
                let mut help_msg = locales::get_text(reply_lang.as_str(), "help-text", None);
                if is_main_admin {
                    let header =
                        locales::get_text(reply_lang.as_str(), "tt-admin-help-header", None);
                    let cmds = locales::get_text(reply_lang.as_str(), "tt-admin-help-cmds", None);
                    help_msg.push_str(&header);
                    help_msg.push_str(&cmds);
                }
                send_reply(help_msg).await;
            } else if cmd == "/skip" {
                let is_admin = if username.is_empty() {
                    false
                } else if admin_username
                    .as_ref()
                    .map(|u| u == &username)
                    .unwrap_or(false)
                {
                    true
                } else if let Some(tg_id) = db.get_telegram_id_by_tt_user(&username).await {
                    db.get_all_admins()
                        .await
                        .map(|admins| admins.contains(&tg_id))
                        .unwrap_or(false)
                } else {
                    false
                };
                if !is_admin {
                    let text = locales::get_text(reply_lang.as_str(), "cmd-unauth", None);
                    send_reply(text).await;
                    return;
                }
                if let Err(e) = tx_tt_cmd.send(TtCommand::SkipStream).await {
                    tracing::error!(
                        tt_username = %username,
                        error = %e,
                        "Failed to send TT skip command"
                    );
                    let text = locales::get_text(reply_lang.as_str(), "tt-error-generic", None);
                    send_reply(text).await;
                    return;
                }
                let text = locales::get_text(reply_lang.as_str(), "tt-skip-sent", None);
                send_reply(text).await;
            } else if cmd == "/add_admin" {
                let is_main_admin = admin_username
                    .as_ref()
                    .map(|u| u == &username)
                    .unwrap_or(false);
                if !is_main_admin {
                    let text = locales::get_text(reply_lang.as_str(), "cmd-unauth", None);
                    send_reply(text).await;
                    return;
                }
                if parts.len() < 2 {
                    let text = locales::get_text(reply_lang.as_str(), "tt-admin-no-ids", None);
                    send_reply(text).await;
                    return;
                }
                let mut added_count = 0;
                let mut failed_count = 0;
                for id_str in &parts[1..] {
                    if let Ok(tg_id) = id_str.parse::<i64>() {
                        let success = match db.add_admin(tg_id).await {
                            Ok(val) => val,
                            Err(e) => {
                                tracing::error!(
                                    telegram_id = tg_id,
                                    error = %e,
                                    "DB error adding admin"
                                );
                                false
                            }
                        };
                        if success {
                            added_count += 1;
                        }
                    } else {
                        failed_count += 1;
                    }
                }
                if added_count > 0 {
                    let args = args!(count = added_count);
                    let text =
                        locales::get_text(reply_lang.as_str(), "tt-admin-added", args.as_ref());
                    send_reply(text).await;
                }
                if failed_count > 0 {
                    let args = args!(count = failed_count);
                    let text =
                        locales::get_text(reply_lang.as_str(), "tt-admin-add-fail", args.as_ref());
                    send_reply(text).await;
                }
            } else if cmd == "/remove_admin" {
                let is_main_admin = admin_username
                    .as_ref()
                    .map(|u| u == &username)
                    .unwrap_or(false);
                if !is_main_admin {
                    let text = locales::get_text(reply_lang.as_str(), "cmd-unauth", None);
                    send_reply(text).await;
                    return;
                }
                if parts.len() < 2 {
                    let text = locales::get_text(reply_lang.as_str(), "tt-admin-no-ids", None);
                    send_reply(text).await;
                    return;
                }
                let mut removed_count = 0;
                let mut failed_count = 0;
                for id_str in &parts[1..] {
                    if let Ok(tg_id) = id_str.parse::<i64>() {
                        let success = match db.remove_admin(tg_id).await {
                            Ok(val) => val,
                            Err(e) => {
                                tracing::error!(
                                    telegram_id = tg_id,
                                    error = %e,
                                    "DB error removing admin"
                                );
                                false
                            }
                        };
                        if success {
                            removed_count += 1;
                        } else {
                            failed_count += 1;
                        }
                    } else {
                        failed_count += 1;
                    }
                }
                if removed_count > 0 {
                    let args = args!(count = removed_count);
                    let text =
                        locales::get_text(reply_lang.as_str(), "tt-admin-removed", args.as_ref());
                    send_reply(text).await;
                }
                if failed_count > 0 {
                    let args = args!(count = failed_count);
                    let text = locales::get_text(
                        reply_lang.as_str(),
                        "tt-admin-remove-fail",
                        args.as_ref(),
                    );
                    send_reply(text).await;
                }
            } else {
                let server_name = resolve_server_name(&tt_config, real_name_from_client.as_deref());

                if let Err(e) = tx_bridge
                    .send(crate::core::types::BridgeEvent::ToAdmin {
                        user_id: from_uid,
                        nick,
                        tt_username: username,
                        msg_content: content.to_string(),
                        server_name,
                    })
                    .await
                {
                    tracing::error!(error = %e, "Failed to send admin bridge event");
                }
            }
        }
    });
}
