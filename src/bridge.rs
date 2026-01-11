use crate::{
    args,
    config::Config,
    db::Database,
    locales,
    types::{self, BridgeEvent},
};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::ApiError;
use teloxide::RequestError;
use teloxide::{prelude::*, utils::html};
use tokio::task::JoinSet;

#[allow(clippy::too_many_arguments)]
pub async fn run_bridge(
    mut rx_bridge: tokio::sync::mpsc::Receiver<BridgeEvent>,
    db_clone: Database,
    online_users_by_username: Arc<DashMap<String, i32>>,
    config: Arc<Config>,
    event_bot: Option<Bot>,
    msg_bot: Option<Bot>,
    tx_tt_cmd: std::sync::mpsc::Sender<types::TtCommand>,
) {
    let default_lang = &config.general.default_lang;
    let admin_id = teloxide::types::ChatId(config.telegram.admin_chat_id);

    log::info!("🌉 [BRIDGE] Bridge task started.");
    while let Some(event) = rx_bridge.recv().await {
        match event {
            types::BridgeEvent::Broadcast {
                event_type,
                nickname,
                server_name,
                related_tt_username,
            } => {
                let bot = if let Some(bot) = &event_bot {
                    bot
                } else {
                    continue;
                };

                let recipients = match db_clone
                    .get_recipients_for_event(&related_tt_username, event_type)
                    .await
                {
                    Ok(r) if !r.is_empty() => r,
                    _ => continue,
                };

                let escaped_nick = teloxide::utils::html::escape(&nickname);
                let escaped_server = teloxide::utils::html::escape(&server_name);

                let key = match event_type {
                    crate::types::NotificationType::Join => "event-join",
                    crate::types::NotificationType::Leave => "event-leave",
                };

                let mut rendered_text_cache: HashMap<String, String> = HashMap::new();
                let mut set = JoinSet::new();

                for sub in recipients {
                    let bot = bot.clone();
                    let online_users_by_username = online_users_by_username.clone();

                    let text = rendered_text_cache
                        .entry(sub.language_code.clone())
                        .or_insert_with(|| {
                            let args = args!(
                                nickname = escaped_nick.clone(),
                                server = escaped_server.clone()
                            );
                            locales::get_text(&sub.language_code, key, args.as_ref())
                        })
                        .clone();

                    let db_for_closure = db_clone.clone();

                    set.spawn(async move {
                        let mut send_silent = false;

                        if sub.not_on_online_enabled
                            && sub.not_on_online_confirmed
                            && let Some(linked_tt) = &sub.teamtalk_username
                            && online_users_by_username.contains_key(linked_tt)
                        {
                            send_silent = true;
                        }

                        let res = bot
                            .send_message(teloxide::types::ChatId(sub.telegram_id), text)
                            .parse_mode(teloxide::types::ParseMode::Html)
                            .disable_notification(send_silent)
                            .await;

                        if let Err(e) = res {
                            log::warn!("Failed to send notification to {}: {}", sub.telegram_id, e);

                            if let RequestError::Api(api_err) = e {
                                match api_err {
                                    ApiError::BotBlocked |
                                    ApiError::UserDeactivated |
                                    ApiError::ChatNotFound => {
                                        log::info!("🗑️ [BRIDGE] Cleaning up: User {} is no longer reachable ({:?}).", sub.telegram_id, api_err);

                                        if let Err(db_err) = db_for_closure.delete_user_profile(sub.telegram_id).await {
                                            log::error!("❌ [BRIDGE] DB error during auto-cleanup for {}: {}", sub.telegram_id, db_err);
                                        } else {
                                            log::info!("✅ [BRIDGE] Profile for {} removed successfully.", sub.telegram_id);
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        }
                    });
                }

                while let Some(res) = set.join_next().await {
                    if let Err(e) = res {
                        log::error!("[BRIDGE] A notification task failed after joining: {:?}", e);
                    }
                }
            }
            types::BridgeEvent::ToAdmin {
                user_id,
                nick,
                tt_username,
                msg_content,
                server_name,
            } => {
                if let Some(bot) = &msg_bot {
                    let admin_settings =
                        db_clone.get_or_create_user(admin_id.0, default_lang).await;
                    let admin_lang = match admin_settings {
                        Ok(u) => u.language_code,
                        Err(e) => {
                            log::error!("Failed to get admin settings: {}. Defaulting to 'en'.", e);
                            "en".to_string()
                        }
                    };

                    let args_admin = args!(
                        server = html::escape(&server_name),
                        nick = html::escape(&nick),
                        msg = html::escape(&msg_content)
                    );
                    let text_admin =
                        locales::get_text(&admin_lang, "admin-alert", args_admin.as_ref());

                    let res = bot
                        .send_message(admin_id, &text_admin)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .await;

                    let reply_lang = if !tt_username.is_empty() {
                        db_clone
                            .get_user_lang_by_tt_user(&tt_username)
                            .await
                            .unwrap_or_else(|| default_lang.to_string())
                    } else {
                        default_lang.to_string()
                    };

                    let key_reply = if res.is_ok() {
                        "tt-msg-sent"
                    } else {
                        "tt-msg-failed"
                    };
                    let reply_text = locales::get_text(&reply_lang, key_reply, None);

                    tx_tt_cmd
                        .send(types::TtCommand::ReplyToUser {
                            user_id,
                            text: reply_text,
                        })
                        .ok();
                } else {
                    log::debug!("Skipping Admin Alert: 'message_token' is not configured.");
                }
            }
            types::BridgeEvent::WhoReport { chat_id, text } => {
                if let Some(bot) = &event_bot {
                    let _ = bot
                        .send_message(teloxide::types::ChatId(chat_id), &text)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .await;
                }
            }
        }
    }
}
