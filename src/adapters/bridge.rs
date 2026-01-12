use crate::args;
use crate::bootstrap::config::Config;
use crate::core::types::{self, BridgeEvent, LanguageCode, LiteUser};
use crate::infra::db::Database;
use crate::infra::locales;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use teloxide::ApiError;
use teloxide::RequestError;
use teloxide::{prelude::*, utils::html};
use tokio::task::JoinSet;

pub struct BridgeContext {
    pub db: Database,
    pub online_users: Arc<RwLock<HashMap<i32, LiteUser>>>,
    pub config: Arc<Config>,
    pub event_bot: Option<Bot>,
    pub msg_bot: Option<Bot>,
    pub message_token_present: bool,
    pub tx_tt_cmd: std::sync::mpsc::Sender<types::TtCommand>,
    pub shutdown: tokio::sync::watch::Receiver<bool>,
}

pub async fn run_bridge(
    ctx: BridgeContext,
    mut rx_bridge: tokio::sync::mpsc::Receiver<BridgeEvent>,
) {
    let BridgeContext {
        db: db_clone,
        online_users,
        config,
        event_bot,
        msg_bot,
        message_token_present,
        tx_tt_cmd,
        mut shutdown,
    } = ctx;
    let default_lang =
        LanguageCode::from_str_or_default(&config.general.default_lang, LanguageCode::En);
    let admin_id = teloxide::types::ChatId(config.telegram.admin_chat_id);

    tracing::info!("🌉 [BRIDGE] Bridge task started.");
    loop {
        let event = tokio::select! {
            _ = shutdown.changed() => {
                break;
            }
            maybe_event = rx_bridge.recv() => {
                match maybe_event {
                    Some(event) => event,
                    None => break,
                }
            }
        };

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
                    Ok(_) => continue,
                    Err(e) => {
                        tracing::error!(
                            "Failed to load recipients for {:?} event: {}",
                            event_type,
                            e
                        );
                        continue;
                    }
                };

                let escaped_nick = teloxide::utils::html::escape(&nickname);
                let escaped_server = teloxide::utils::html::escape(&server_name);

                let key = match event_type {
                    crate::core::types::NotificationType::Join => "event-join",
                    crate::core::types::NotificationType::Leave => "event-leave",
                };

                let mut rendered_text_cache: HashMap<LanguageCode, String> = HashMap::new();
                let mut set = JoinSet::new();

                for sub in recipients {
                    let bot = bot.clone();
                    let online_users = online_users.clone();

                    let lang = LanguageCode::from_str_or_default(&sub.language_code, default_lang);
                    let text = rendered_text_cache
                        .entry(lang)
                        .or_insert_with(|| {
                            let args = args!(
                                nickname = escaped_nick.clone(),
                                server = escaped_server.clone()
                            );
                            locales::get_text(lang.as_str(), key, args.as_ref())
                        })
                        .clone();

                    let db_for_closure = db_clone.clone();

                    set.spawn(async move {
                        let mut send_silent = false;

                        if sub.not_on_online_enabled
                            && sub.not_on_online_confirmed
                            && let Some(linked_tt) = &sub.teamtalk_username
                        {
                            let guard =
                                online_users.read().unwrap_or_else(|e| e.into_inner());
                            let is_online = guard
                                .values()
                                .any(|entry| entry.username == *linked_tt);
                            if is_online {
                                send_silent = true;
                            }
                        }

                        let res = bot
                            .send_message(teloxide::types::ChatId(sub.telegram_id), text)
                            .parse_mode(teloxide::types::ParseMode::Html)
                            .disable_notification(send_silent)
                            .await;

                        if let Err(e) = res {
                            tracing::warn!("Failed to send notification to {}: {}", sub.telegram_id, e);

                            if let RequestError::Api(api_err) = e {
                                match api_err {
                                    ApiError::BotBlocked |
                                    ApiError::UserDeactivated |
                                    ApiError::ChatNotFound => {
                                        tracing::info!("🗑️ [BRIDGE] Cleaning up: User {} is no longer reachable ({:?}).", sub.telegram_id, api_err);

                                        if let Err(db_err) = db_for_closure.delete_user_profile(sub.telegram_id).await {
                                            tracing::error!("❌ [BRIDGE] DB error during auto-cleanup for {}: {}", sub.telegram_id, db_err);
                                        } else {
                                            tracing::info!("✅ [BRIDGE] Profile for {} removed successfully.", sub.telegram_id);
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
                        tracing::error!(
                            "[BRIDGE] A notification task failed after joining: {:?}",
                            e
                        );
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
                let bot = if let Some(bot) = msg_bot.as_ref() {
                    Some(bot)
                } else if message_token_present {
                    event_bot.as_ref()
                } else {
                    None
                };
                if let Some(bot) = bot {
                    let admin_settings =
                        db_clone.get_or_create_user(admin_id.0, default_lang).await;
                    let admin_lang = match admin_settings {
                        Ok(u) => LanguageCode::from_str_or_default(&u.language_code, default_lang),
                        Err(e) => {
                            tracing::error!(
                                "Failed to get admin settings: {}. Defaulting to 'en'.",
                                e
                            );
                            LanguageCode::En
                        }
                    };

                    let args_admin = args!(
                        server = html::escape(&server_name),
                        nick = html::escape(&nick),
                        msg = html::escape(&msg_content)
                    );
                    let text_admin =
                        locales::get_text(admin_lang.as_str(), "admin-alert", args_admin.as_ref());

                    let res = bot
                        .send_message(admin_id, &text_admin)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .await;
                    let res = res;
                    if let Ok(msg) = &res
                        && let Err(e) = db_clone.add_pending_reply(msg.id.0 as i64, user_id).await
                    {
                        tracing::error!("Failed to save pending reply for {}: {}", msg.id.0, e);
                    }

                    let reply_lang = if !tt_username.is_empty() {
                        db_clone
                            .get_user_lang_by_tt_user(&tt_username)
                            .await
                            .unwrap_or(default_lang)
                    } else {
                        default_lang
                    };

                    let key_reply = if res.is_ok() {
                        "tt-msg-sent"
                    } else {
                        "tt-msg-failed"
                    };
                    let reply_text = locales::get_text(reply_lang.as_str(), key_reply, None);

                    if let Err(e) = tx_tt_cmd.send(types::TtCommand::ReplyToUser {
                        user_id,
                        text: reply_text,
                    }) {
                        tracing::error!(
                            "Failed to send TT reply command for user {}: {}",
                            user_id,
                            e
                        );
                    }
                } else {
                    tracing::debug!("Skipping Admin Alert: message_token is not configured.");
                }
            }
            types::BridgeEvent::ToAdminChannel {
                channel_id,
                channel_name,
                server_name,
                msg_content,
            } => {
                let bot = if let Some(bot) = msg_bot.as_ref() {
                    Some(bot)
                } else if message_token_present {
                    event_bot.as_ref()
                } else {
                    None
                };
                if let Some(bot) = bot {
                    let admin_settings =
                        db_clone.get_or_create_user(admin_id.0, default_lang).await;
                    let admin_lang = match admin_settings {
                        Ok(u) => LanguageCode::from_str_or_default(&u.language_code, default_lang),
                        Err(e) => {
                            tracing::error!(
                                "Failed to get admin settings: {}. Defaulting to 'en'.",
                                e
                            );
                            LanguageCode::En
                        }
                    };

                    let args_admin = args!(
                        server = html::escape(&server_name),
                        channel = html::escape(&channel_name),
                        msg = html::escape(&msg_content)
                    );
                    let text_admin = locales::get_text(
                        admin_lang.as_str(),
                        "admin-channel-pm",
                        args_admin.as_ref(),
                    );

                    let res = bot
                        .send_message(admin_id, &text_admin)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .await;
                    let res = res;
                    if let Ok(msg) = &res
                        && let Err(e) = db_clone
                            .add_pending_channel_reply(
                                msg.id.0 as i64,
                                channel_id,
                                &channel_name,
                                &server_name,
                                &msg_content,
                            )
                            .await
                    {
                        tracing::error!(
                            "Failed to save pending channel reply for {}: {}",
                            msg.id.0,
                            e
                        );
                    }
                } else {
                    tracing::debug!("Skipping Admin Alert: message_token is not configured.");
                }
            }
            types::BridgeEvent::WhoReport { chat_id, text } => {
                if let Some(bot) = &event_bot
                    && let Err(e) = bot
                        .send_message(teloxide::types::ChatId(chat_id), &text)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .await
                {
                    tracing::error!("Failed to send who report to {}: {}", chat_id, e);
                }
            }
        }
    }
}
