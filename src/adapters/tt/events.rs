#![allow(clippy::pedantic, clippy::nursery)]

use crate::adapters::tt::commands;
use crate::adapters::tt::{WorkerContext, resolve_channel_name, resolve_server_name};
use crate::app::services::reply_queue as reply_queue_service;
use crate::bootstrap::config::GenderConfig;
use crate::core::types::{
    BridgeEvent, LanguageCode, LiteUser, NotificationType, TtCommand, TtUsername,
};
use chrono::Utc;
use std::time::{Duration, Instant};
use teamtalk::client::ReconnectHandler;
use teamtalk::client::ffi;
use teamtalk::types::{UserGender, UserStatus};
use teamtalk::{Client, Event, Message};

pub(super) fn handle_sdk_event(
    client: &Client,
    ctx: &WorkerContext,
    event: Event,
    msg: Message,
    is_connected: &mut bool,
    reconnect_handler: &mut ReconnectHandler,
    ready_time: &mut Option<Instant>,
) {
    tracing::trace!(component = "tt_worker", event = ?event, "Event received");
    let tt_config = &ctx.config.teamtalk;

    match event {
        Event::ConnectSuccess => {
            *is_connected = true;
            reconnect_handler.mark_connected();
            client.login(
                &tt_config.nick_name,
                &tt_config.user_name,
                &tt_config.password,
                &tt_config.client_name,
            );
        }
        e if e.is_reconnect_needed_with(&[Event::MySelfKicked]) => {
            *is_connected = false;
            reconnect_handler.mark_disconnected();
            if let Ok(mut users) = ctx.online_users.write() {
                users.clear();
            }
            *ready_time = None;
            tracing::warn!(
                component = "tt_worker",
                event = ?e,
                "Disconnection event; reconnect pending"
            );
        }
        Event::MySelfLoggedIn => {
            let gender = parse_gender(ctx.config.general.gender);
            let status = UserStatus {
                gender,
                ..UserStatus::default()
            };
            client.set_status(status, &tt_config.status_text);
            let chan_id = client.get_channel_id_from_path(tt_config.channel.as_str());
            if chan_id.0 > 0 {
                let cmd_id = client
                    .join_channel(chan_id, tt_config.channel_password.as_deref().unwrap_or(""));
                if cmd_id <= 0 {
                    tracing::error!(
                        component = "tt_worker",
                        channel = %tt_config.channel,
                        channel_id = chan_id.0,
                        "Failed to join channel"
                    );
                }
            }
            *ready_time = Some(std::time::Instant::now());
            if let Ok(mut accounts) = ctx.user_accounts.write() {
                accounts.clear();
            }
            client.list_user_accounts(0, 1000);
        }

        Event::UserAccount => {
            if let Some(account) = msg.account()
                && !account.username.is_empty()
                && let Ok(mut accounts) = ctx.user_accounts.write()
            {
                accounts.insert(TtUsername::new(account.username.clone()), account);
            }
        }
        Event::UserAccountCreated | Event::UserAccountRemoved => {
            if let Ok(mut accounts) = ctx.user_accounts.write() {
                accounts.clear();
            }
            client.list_user_accounts(0, 1000);
        }

        Event::UserUpdate => {
            if let Some(user) = msg.user()
                && let Ok(mut users) = ctx.online_users.write()
                && let Some(existing_lite_user) = users.get_mut(&user.id.0)
            {
                let new_username = TtUsername::new(user.username.clone());
                if existing_lite_user.username != new_username {
                    if let Ok(mut by_username) = ctx.online_users_by_username.write() {
                        if !existing_lite_user.username.as_str().is_empty() {
                            by_username.remove(&existing_lite_user.username);
                        }
                        if !user.username.is_empty() {
                            by_username.insert(new_username.clone(), user.id.0);
                        }
                    }
                    existing_lite_user.username = new_username;
                }

                if existing_lite_user.nickname != user.nickname {
                    tracing::info!(
                        component = "tt_worker",
                        username = %user.username,
                        old_nick = %existing_lite_user.nickname,
                        new_nick = %user.nickname,
                        "Nickname changed"
                    );
                    existing_lite_user.nickname = user.nickname.clone();
                }
            }
        }
        Event::StreamMediaFile => {
            let raw = msg.raw();
            let info =
                unsafe { teamtalk::types::MediaFileInfo::from(raw.__bindgen_anon_1.mediafileinfo) };
            let gender = parse_gender(ctx.config.general.gender);
            match info.status {
                ffi::MediaFileStatus::MFS_CLOSED
                | ffi::MediaFileStatus::MFS_ERROR
                | ffi::MediaFileStatus::MFS_FINISHED
                | ffi::MediaFileStatus::MFS_ABORTED => {
                    client.stop_streaming_media_file_to_channel();
                    ctx.is_streaming
                        .store(false, std::sync::atomic::Ordering::Relaxed);
                    let status = UserStatus {
                        gender,
                        streaming: false,
                        ..UserStatus::default()
                    };
                    client.set_status(status, &ctx.config.teamtalk.status_text);
                }
                ffi::MediaFileStatus::MFS_PAUSED => {
                    if ctx.is_streaming.load(std::sync::atomic::Ordering::Relaxed) {
                        let status = UserStatus {
                            gender,
                            streaming: true,
                            media_paused: true,
                            ..UserStatus::default()
                        };
                        client.set_status(status, &ctx.config.teamtalk.status_text);
                    }
                }
                ffi::MediaFileStatus::MFS_STARTED | ffi::MediaFileStatus::MFS_PLAYING => {
                    if ctx.is_streaming.load(std::sync::atomic::Ordering::Relaxed) {
                        let status = UserStatus {
                            gender,
                            streaming: true,
                            ..UserStatus::default()
                        };
                        client.set_status(status, &ctx.config.teamtalk.status_text);
                    }
                }
            }
        }
        Event::UserLoggedIn => {
            if let Some(user) = msg.user()
                && user.id != client.my_id()
            {
                let nickname = user.nickname.clone();

                let channel_name = resolve_channel_name(client, user.channel_id, LanguageCode::En);

                let lite_user = LiteUser {
                    id: user.id.0,
                    nickname: nickname.clone(),
                    username: TtUsername::new(user.username.clone()),
                    channel_name,
                };
                if let Ok(mut by_username) = ctx.online_users_by_username.write()
                    && !lite_user.username.as_str().is_empty()
                {
                    by_username.insert(lite_user.username.clone(), lite_user.id);
                }
                if let Ok(mut users) = ctx.online_users.write() {
                    users.insert(user.id.0, lite_user.clone());
                }

                let is_ready = ready_time
                    .map(|t| t.elapsed() >= Duration::from_secs(2))
                    .unwrap_or(false);

                if is_ready
                    && !tt_config
                        .global_ignore_usernames
                        .contains(&lite_user.username)
                {
                    let real_name = client.get_server_properties().map(|p| p.name);
                    let server_name = resolve_server_name(tt_config, real_name.as_deref());

                    let tx_bridge = ctx.tx_bridge.clone();
                    let related_tt_username = TtUsername::new(user.username.clone());
                    tokio::task::spawn_local(async move {
                        if let Err(e) = tx_bridge
                            .send(BridgeEvent::Broadcast {
                                event_type: NotificationType::Join,
                                nickname,
                                server_name,
                                related_tt_username,
                            })
                            .await
                        {
                            tracing::error!(error = %e, "Failed to send join broadcast");
                        }
                    });
                }

                if !user.username.is_empty() {
                    let db = ctx.db.clone();
                    let tx_tt_cmd = ctx.tx_tt_cmd.clone();
                    let tt_username = TtUsername::new(user.username.clone());
                    let default_lang = ctx.config.general.default_lang;
                    let user_id = user.id.0;
                    tokio::task::spawn_local(async move {
                        let mut items = match db.get_reply_queue_for_user(&tt_username).await {
                            Ok(items) => items,
                            Err(e) => {
                                tracing::error!(error = %e, "Failed to load reply queue");
                                return;
                            }
                        };
                        if items.is_empty() {
                            return;
                        }
                        reply_queue_service::queue_items_sorted(&mut items);
                        let lang = db
                            .get_user_lang_by_tt_user(&tt_username)
                            .await
                            .unwrap_or(default_lang);
                        let now = Utc::now();
                        let mut sent_ids = Vec::new();
                        for item in items {
                            let formatted = reply_queue_service::format_queue_message(
                                lang,
                                item.created_at,
                                now,
                                &item.message_text,
                            );
                            if let Err(e) = tx_tt_cmd
                                .send(TtCommand::ReplyToUser {
                                    user_id,
                                    text: formatted,
                                })
                                .await
                            {
                                tracing::error!(error = %e, "Failed to send queued reply");
                                break;
                            }
                            sent_ids.push(item.id);
                        }
                        if !sent_ids.is_empty()
                            && let Err(e) = db.delete_reply_queue_ids(&sent_ids).await
                        {
                            tracing::error!(error = %e, "Failed to clear sent queue items");
                        }
                    });
                }
            }
        }
        Event::UserJoined => {
            if let Some(user) = msg.user()
                && user.id != client.my_id()
            {
                let nickname = user.nickname.clone();
                let channel_name = resolve_channel_name(client, user.channel_id, LanguageCode::En);

                let lite_user = LiteUser {
                    id: user.id.0,
                    nickname,
                    username: TtUsername::new(user.username.clone()),
                    channel_name,
                };
                if let Ok(mut by_username) = ctx.online_users_by_username.write()
                    && !lite_user.username.as_str().is_empty()
                {
                    by_username.insert(lite_user.username.clone(), lite_user.id);
                }
                if let Ok(mut users) = ctx.online_users.write() {
                    users.insert(user.id.0, lite_user);
                }
            }
        }

        Event::UserLoggedOut => {
            if let Some(user) = msg.user() {
                let removed = if let Ok(mut users) = ctx.online_users.write() {
                    users.remove(&user.id.0)
                } else {
                    None
                };
                if let Some(u) = removed {
                    if let Ok(mut by_username) = ctx.online_users_by_username.write()
                        && !u.username.as_str().is_empty()
                    {
                        by_username.remove(&u.username);
                    }
                    if user.id != client.my_id() {
                        let is_ready = ready_time
                            .map(|t| t.elapsed() >= Duration::from_secs(2))
                            .unwrap_or(false);
                        if is_ready && !tt_config.global_ignore_usernames.contains(&u.username) {
                            let real_name = client.get_server_properties().map(|p| p.name);
                            let server_name = resolve_server_name(tt_config, real_name.as_deref());

                            let tx_bridge = ctx.tx_bridge.clone();
                            let nickname = u.nickname.clone();
                            let related_tt_username = u.username.clone();
                            tokio::task::spawn_local(async move {
                                if let Err(e) = tx_bridge
                                    .send(BridgeEvent::Broadcast {
                                        event_type: NotificationType::Leave,
                                        nickname,
                                        server_name,
                                        related_tt_username,
                                    })
                                    .await
                                {
                                    tracing::error!(error = %e, "Failed to send leave broadcast");
                                }
                            });
                        }
                    }
                }
            }
        }
        Event::UserLeft => {
            if let Some(user) = msg.user() {
                let channel_name = resolve_channel_name(client, user.channel_id, LanguageCode::En);

                if let Ok(mut users) = ctx.online_users.write()
                    && let Some(u) = users.get_mut(&user.id.0)
                {
                    u.channel_name = channel_name;
                }
            }
        }

        Event::TextMessage => {
            if let Some(txt_msg) = msg.text() {
                commands::handle_text_message(client, ctx, txt_msg);
            }
        }

        _ => {}
    }
}

fn parse_gender(cfg: GenderConfig) -> UserGender {
    cfg.to_user_gender()
}
