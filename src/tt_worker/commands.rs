use crate::args;
use crate::locales;
use crate::tt_worker::{WorkerContext, resolve_server_name};
use crate::types::{LanguageCode, TtCommand};
use teamtalk::Client;
use teamtalk::types::TextMessage;
use uuid::Uuid;

pub(super) fn handle_text_message(client: &Client, ctx: &WorkerContext, msg: TextMessage) {
    if msg.from_id == client.my_id() {
        return;
    }

    let real_name_from_client = client.get_server_properties().map(|p| p.name);
    let tx_tt_cmd = ctx.tx_tt_cmd.clone();

    let db = ctx.db.clone();
    let online_users = ctx.online_users.clone();

    let default_lang =
        LanguageCode::from_str_or_default(&ctx.config.general.default_lang, LanguageCode::En);
    let admin_username = ctx.config.general.admin_username.clone();
    let tt_config = ctx.config.teamtalk.clone();
    let deeplink_ttl = ctx.config.operational_parameters.deeplink_ttl_seconds;

    let bot_username = ctx.bot_username.clone();
    let tx_bridge = ctx.tx_bridge.clone();

    ctx.rt.spawn(async move {
        if msg.msg_type == teamtalk::client::ffi::TextMsgType::MSGTYPE_USER {
            let content = msg.text.trim();
            let from_uid = msg.from_id.0;

            let (nick, username): (String, String) = if let Some(u) = online_users.get(&from_uid) {
                (u.nickname.clone(), u.username.clone())
            } else {
                ("Unknown".to_string(), "".to_string())
            };

            tracing::info!("💬 [TT_WORKER] Msg from {}: {}", nick, content);

            let reply_lang = if !username.is_empty() {
                db.get_user_lang_by_tt_user(&username)
                    .await
                    .unwrap_or(default_lang)
            } else {
                default_lang
            };

            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.is_empty() {
                return;
            }
            let cmd = parts[0].to_lowercase();

            let send_reply = |text: String| {
                if let Err(e) = tx_tt_cmd.send(TtCommand::ReplyToUser {
                    user_id: from_uid,
                    text,
                }) {
                    tracing::error!("Failed to send TT reply command for {}: {}", from_uid, e);
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
                            "subscribe",
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
                            send_reply(text);
                        }
                        Err(_) => {
                            let text =
                                locales::get_text(reply_lang.as_str(), "tt-error-generic", None);
                            send_reply(text);
                        }
                    }
                } else {
                    send_reply(
                        "Telegram integration is currently disabled (Event Token missing)."
                            .to_string(),
                    );
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
                            "unsubscribe",
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
                            send_reply(text);
                        }
                        Err(_) => {
                            let text =
                                locales::get_text(reply_lang.as_str(), "tt-error-generic", None);
                            send_reply(text);
                        }
                    }
                } else {
                    send_reply(
                        "Telegram integration is currently disabled (Event Token missing)."
                            .to_string(),
                    );
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
                send_reply(help_msg);
            } else if cmd == "/add_admin" {
                let is_main_admin = admin_username
                    .as_ref()
                    .map(|u| u == &username)
                    .unwrap_or(false);
                if !is_main_admin {
                    let text = locales::get_text(reply_lang.as_str(), "cmd-unauth", None);
                    send_reply(text);
                    return;
                }
                if parts.len() < 2 {
                    let text = locales::get_text(reply_lang.as_str(), "tt-admin-no-ids", None);
                    send_reply(text);
                    return;
                }
                let mut added_count = 0;
                let mut failed_count = 0;
                for id_str in &parts[1..] {
                    if let Ok(tg_id) = id_str.parse::<i64>() {
                        let success = match db.add_admin(tg_id).await {
                            Ok(val) => val,
                            Err(e) => {
                                tracing::error!("DB error adding admin {}: {}", tg_id, e);
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
                    send_reply(text);
                }
                if failed_count > 0 {
                    let args = args!(count = failed_count);
                    let text =
                        locales::get_text(reply_lang.as_str(), "tt-admin-add-fail", args.as_ref());
                    send_reply(text);
                }
            } else if cmd == "/remove_admin" {
                let is_main_admin = admin_username
                    .as_ref()
                    .map(|u| u == &username)
                    .unwrap_or(false);
                if !is_main_admin {
                    let text = locales::get_text(reply_lang.as_str(), "cmd-unauth", None);
                    send_reply(text);
                    return;
                }
                if parts.len() < 2 {
                    let text = locales::get_text(reply_lang.as_str(), "tt-admin-no-ids", None);
                    send_reply(text);
                    return;
                }
                let mut removed_count = 0;
                let mut failed_count = 0;
                for id_str in &parts[1..] {
                    if let Ok(tg_id) = id_str.parse::<i64>() {
                        let success = match db.remove_admin(tg_id).await {
                            Ok(val) => val,
                            Err(e) => {
                                tracing::error!("DB error removing admin {}: {}", tg_id, e);
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
                    send_reply(text);
                }
                if failed_count > 0 {
                    let args = args!(count = failed_count);
                    let text = locales::get_text(
                        reply_lang.as_str(),
                        "tt-admin-remove-fail",
                        args.as_ref(),
                    );
                    send_reply(text);
                }
            } else {
                let server_name = resolve_server_name(&tt_config, real_name_from_client.as_deref());

                if let Err(e) = tx_bridge
                    .send(crate::types::BridgeEvent::ToAdmin {
                        user_id: from_uid,
                        nick,
                        tt_username: username,
                        msg_content: content.to_string(),
                        server_name,
                    })
                    .await
                {
                    tracing::error!("Failed to send admin bridge event: {}", e);
                }
            }
        }
    });
}
