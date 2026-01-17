use crate::args;
use crate::bootstrap::config::Config;
use crate::core::types::{self, BridgeEvent, LanguageCode, LiteUser};
use crate::infra::db::{Database, types::UserSettings};
use crate::infra::locales;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use teloxide::ApiError;
use teloxide::RequestError;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::{prelude::*, utils::html};
use tokio::task::JoinSet;

struct BridgeDeps<'a> {
    db: &'a Database,
    online_users: &'a Arc<RwLock<HashMap<i32, LiteUser>>>,
    event_bot: Option<&'a Bot>,
    msg_bot: Option<&'a Bot>,
    message_token_present: bool,
    default_lang: LanguageCode,
    admin_id: teloxide::types::ChatId,
    tx_tt_cmd: &'a tokio::sync::mpsc::Sender<types::TtCommand>,
}

struct BroadcastData {
    event_type: types::NotificationType,
    nickname: String,
    server_name: String,
    related_tt_username: String,
}

struct AdminData {
    user_id: i32,
    nick: String,
    tt_username: String,
    msg_content: String,
    server_name: String,
}

struct AdminChannelData {
    channel_id: i32,
    channel_name: String,
    server_name: String,
    msg_content: String,
}

struct WhoReportData {
    chat_id: i64,
    text: String,
    reply_to: Option<i32>,
}

struct BroadcastTaskCtx {
    bot: Bot,
    online_users: Arc<RwLock<HashMap<i32, LiteUser>>>,
    db: Database,
}

pub struct BridgeContext {
    pub db: Database,
    pub online_users: Arc<RwLock<HashMap<i32, LiteUser>>>,
    pub config: Arc<Config>,
    pub event_bot: Option<Bot>,
    pub msg_bot: Option<Bot>,
    pub message_token_present: bool,
    pub tx_tt_cmd: tokio::sync::mpsc::Sender<types::TtCommand>,
    pub cancel_token: tokio_util::sync::CancellationToken,
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
        cancel_token,
    } = ctx;
    let default_lang =
        LanguageCode::from_str_or_default(&config.general.default_lang, LanguageCode::En);
    let admin_id = teloxide::types::ChatId(config.telegram.admin_chat_id);
    let deps = BridgeDeps {
        db: &db_clone,
        online_users: &online_users,
        event_bot: event_bot.as_ref(),
        msg_bot: msg_bot.as_ref(),
        message_token_present,
        default_lang,
        admin_id,
        tx_tt_cmd: &tx_tt_cmd,
    };

    tracing::info!(component = "bridge", "Bridge task started");
    loop {
        let event = tokio::select! {
            () = cancel_token.cancelled() => {
                break;
            }
            maybe_event = rx_bridge.recv() => {
                match maybe_event {
                    Some(event) => event,
                    None => break,
                }
            }
        };

        handle_bridge_event(&deps, event).await;
    }
}

async fn handle_bridge_event(deps: &BridgeDeps<'_>, event: BridgeEvent) {
    match event {
        types::BridgeEvent::Broadcast {
            event_type,
            nickname,
            server_name,
            related_tt_username,
        } => {
            handle_broadcast(
                deps,
                BroadcastData {
                    event_type,
                    nickname,
                    server_name,
                    related_tt_username,
                },
            )
            .await;
        }
        types::BridgeEvent::ToAdmin {
            user_id,
            nick,
            tt_username,
            msg_content,
            server_name,
        } => {
            handle_to_admin(
                deps,
                AdminData {
                    user_id,
                    nick,
                    tt_username,
                    msg_content,
                    server_name,
                },
            )
            .await;
        }
        types::BridgeEvent::ToAdminChannel {
            channel_id,
            channel_name,
            server_name,
            msg_content,
        } => {
            handle_to_admin_channel(
                deps,
                AdminChannelData {
                    channel_id,
                    channel_name,
                    server_name,
                    msg_content,
                },
            )
            .await;
        }
        types::BridgeEvent::WhoReport {
            chat_id,
            text,
            reply_to,
        } => {
            handle_who_report(
                deps,
                WhoReportData {
                    chat_id,
                    text,
                    reply_to,
                },
            )
            .await;
        }
    }
}

async fn handle_broadcast(deps: &BridgeDeps<'_>, data: BroadcastData) {
    let Some(bot) = deps.event_bot else {
        return;
    };

    let recipients = match deps
        .db
        .get_recipients_for_event(&data.related_tt_username, data.event_type)
        .await
    {
        Ok(r) if !r.is_empty() => r,
        Ok(_) => return,
        Err(e) => {
            tracing::error!(
                component = "bridge",
                event_type = ?data.event_type,
                tt_username = %data.related_tt_username,
                error = %e,
                "Failed to load recipients"
            );
            return;
        }
    };

    let escaped_nick = teloxide::utils::html::escape(&data.nickname);
    let escaped_server = teloxide::utils::html::escape(&data.server_name);

    let key = match data.event_type {
        types::NotificationType::Join => "event-join",
        types::NotificationType::Leave => "event-leave",
    };

    let mut rendered_text_cache: HashMap<LanguageCode, String> = HashMap::new();
    let mut set = JoinSet::new();

    for sub in recipients {
        let task_ctx = BroadcastTaskCtx {
            bot: bot.clone(),
            online_users: deps.online_users.clone(),
            db: deps.db.clone(),
        };

        let lang = LanguageCode::from_str_or_default(&sub.language_code, deps.default_lang);
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

        set.spawn(async move {
            send_broadcast_to_recipient(task_ctx, sub, text).await;
        });
    }

    while let Some(res) = set.join_next().await {
        if let Err(e) = res {
            tracing::error!(
                component = "bridge",
                error = ?e,
                "Notification task failed after join"
            );
        }
    }
}

async fn send_broadcast_to_recipient(ctx: BroadcastTaskCtx, sub: UserSettings, text: String) {
    let mut send_silent = false;

    if sub.not_on_online_enabled
        && sub.not_on_online_confirmed
        && let Some(linked_tt) = &sub.teamtalk_username
    {
        let is_online = ctx
            .online_users
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .any(|entry| entry.username == *linked_tt);
        if is_online {
            send_silent = true;
        }
    }

    let res = ctx
        .bot
        .send_message(teloxide::types::ChatId(sub.telegram_id), text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .disable_notification(send_silent)
        .await;

    if let Err(e) = res {
        tracing::warn!(
            component = "bridge",
            telegram_id = sub.telegram_id,
            tt_username = ?sub.teamtalk_username,
            error = %e,
            "Failed to send notification"
        );

        if let RequestError::Api(api_err) = e {
            match api_err {
                ApiError::BotBlocked | ApiError::UserDeactivated | ApiError::ChatNotFound => {
                    tracing::info!(
                        component = "bridge",
                        telegram_id = sub.telegram_id,
                        tt_username = ?sub.teamtalk_username,
                        api_error = ?api_err,
                        "Cleaning up unreachable user"
                    );

                    if let Err(db_err) = ctx.db.delete_user_profile(sub.telegram_id).await {
                        tracing::error!(
                            component = "bridge",
                            telegram_id = sub.telegram_id,
                            tt_username = ?sub.teamtalk_username,
                            error = %db_err,
                            "DB error during auto-cleanup"
                        );
                    } else {
                        tracing::info!(
                            component = "bridge",
                            telegram_id = sub.telegram_id,
                            tt_username = ?sub.teamtalk_username,
                            "Profile removed successfully"
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

async fn handle_to_admin(deps: &BridgeDeps<'_>, data: AdminData) {
    let bot = deps.msg_bot.or(if deps.message_token_present {
        deps.event_bot
    } else {
        None
    });
    let Some(bot) = bot else {
        tracing::debug!(
            component = "bridge",
            "Skipping admin alert: message_token not configured"
        );
        return;
    };

    let admin_settings = deps
        .db
        .get_or_create_user(deps.admin_id.0, deps.default_lang)
        .await;
    let admin_lang = match admin_settings {
        Ok(u) => LanguageCode::from_str_or_default(&u.language_code, deps.default_lang),
        Err(e) => {
            tracing::error!(
                component = "bridge",
                error = %e,
                "Failed to get admin settings; defaulting to 'en'"
            );
            LanguageCode::En
        }
    };

    let args_admin = args!(
        server = html::escape(&data.server_name),
        nick = html::escape(&data.nick),
        msg = html::escape(&data.msg_content)
    );
    let text_admin = locales::get_text(admin_lang.as_str(), "admin-alert", args_admin.as_ref());

    let res = bot
        .send_message(deps.admin_id, &text_admin)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await;
    if let Ok(msg) = &res
        && let Err(e) = deps
            .db
            .add_pending_reply(i64::from(msg.id.0), data.user_id)
            .await
    {
        tracing::error!(
            component = "bridge",
            message_id = msg.id.0,
            tt_username = %data.tt_username,
            error = %e,
            "Failed to save pending reply"
        );
    }

    let reply_lang = if data.tt_username.is_empty() {
        deps.default_lang
    } else {
        deps.db
            .get_user_lang_by_tt_user(&data.tt_username)
            .await
            .unwrap_or(deps.default_lang)
    };

    let key_reply = if res.is_ok() {
        "tt-msg-sent"
    } else {
        "tt-msg-failed"
    };
    let reply_text = locales::get_text(reply_lang.as_str(), key_reply, None);

    if let Err(e) = deps
        .tx_tt_cmd
        .send(types::TtCommand::ReplyToUser {
            user_id: data.user_id,
            text: reply_text,
        })
        .await
    {
        tracing::error!(
            component = "bridge",
            user_id = data.user_id,
            tt_username = %data.tt_username,
            error = %e,
            "Failed to send TT reply command"
        );
    }
}

async fn handle_to_admin_channel(deps: &BridgeDeps<'_>, data: AdminChannelData) {
    let bot = deps.msg_bot.or(if deps.message_token_present {
        deps.event_bot
    } else {
        None
    });
    let Some(bot) = bot else {
        tracing::debug!(
            component = "bridge",
            "Skipping admin alert: message_token not configured"
        );
        return;
    };

    let admin_settings = deps
        .db
        .get_or_create_user(deps.admin_id.0, deps.default_lang)
        .await;
    let admin_lang = match admin_settings {
        Ok(u) => LanguageCode::from_str_or_default(&u.language_code, deps.default_lang),
        Err(e) => {
            tracing::error!(
                component = "bridge",
                error = %e,
                "Failed to get admin settings; defaulting to 'en'"
            );
            LanguageCode::En
        }
    };

    let args_admin = args!(
        server = html::escape(&data.server_name),
        channel = html::escape(&data.channel_name),
        msg = html::escape(&data.msg_content)
    );
    let text_admin =
        locales::get_text(admin_lang.as_str(), "admin-channel-pm", args_admin.as_ref());

    let res = bot
        .send_message(deps.admin_id, &text_admin)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await;
    if let Ok(msg) = &res
        && let Err(e) = deps
            .db
            .add_pending_channel_reply(
                i64::from(msg.id.0),
                data.channel_id,
                &data.channel_name,
                &data.server_name,
                &data.msg_content,
            )
            .await
    {
        tracing::error!(
            component = "bridge",
            message_id = msg.id.0,
            error = %e,
            "Failed to save pending channel reply"
        );
    }
}

async fn handle_who_report(deps: &BridgeDeps<'_>, data: WhoReportData) {
    if let Some(bot) = deps.event_bot
        && let Err(e) = {
            let req = bot
                .send_message(teloxide::types::ChatId(data.chat_id), &data.text)
                .parse_mode(teloxide::types::ParseMode::Html);
            if let Some(reply_to) = data.reply_to {
                req.reply_to(teloxide::types::MessageId(reply_to)).await
            } else {
                req.await
            }
        }
    {
        tracing::error!(
            component = "bridge",
            chat_id = data.chat_id,
            error = %e,
            "Failed to send who report"
        );
    }
}
