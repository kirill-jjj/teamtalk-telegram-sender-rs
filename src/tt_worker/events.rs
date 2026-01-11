use crate::tt_worker::WorkerContext;
use crate::tt_worker::commands;
use crate::types::{BridgeEvent, LiteUser, NotificationType};
use std::time::{Duration, Instant};
use teamtalk::client::ReconnectHandler;
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
    tracing::trace!("📥 [TT_WORKER] Event received: {:?}", event);
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
            ctx.online_users.clear();
            *ready_time = None;
            tracing::warn!(
                "❌ [TT_WORKER] Disconnection event ({:?}). Reconnect pending...",
                e
            );
        }
        Event::MySelfLoggedIn => {
            client.set_status_message(&tt_config.status_text);
            let chan_id = client.get_channel_id_from_path(&tt_config.channel);
            if chan_id.0 > 0 {
                client.join_channel(chan_id, tt_config.channel_password.as_deref().unwrap_or(""));
            }
            *ready_time = Some(std::time::Instant::now());
            ctx.user_accounts.clear();
            client.list_user_accounts(0, 1000);
        }

        Event::UserAccount => {
            if let Some(account) = msg.account()
                && !account.username.is_empty()
            {
                ctx.user_accounts.insert(account.username.clone(), account);
            }
        }
        Event::UserAccountCreated | Event::UserAccountRemoved => {
            ctx.user_accounts.clear();
            client.list_user_accounts(0, 1000);
        }

        Event::UserUpdate => {
            if let Some(user) = msg.user()
                && let Some(mut existing_lite_user) = ctx.online_users.get_mut(&user.id.0)
            {
                if existing_lite_user.username != user.username {
                    if !existing_lite_user.username.is_empty() {
                        ctx.online_users_by_username
                            .remove(&existing_lite_user.username);
                    }
                    if !user.username.is_empty() {
                        ctx.online_users_by_username
                            .insert(user.username.clone(), user.id.0);
                    }
                    existing_lite_user.username = user.username.clone();
                }

                if existing_lite_user.nickname != user.nickname {
                    tracing::info!(
                        "🔄 [TT_WORKER] Nickname changed for {}: {} -> {}",
                        user.username,
                        existing_lite_user.nickname,
                        user.nickname
                    );
                    existing_lite_user.nickname = user.nickname.clone();
                }
            }
        }
        Event::UserLoggedIn => {
            if let Some(user) = msg.user()
                && user.id != client.my_id()
            {
                let nickname = user.nickname.clone();

                let channel_name = client
                    .get_channel(user.channel_id)
                    .map(|c| c.name)
                    .unwrap_or_else(|| "Unknown".to_string());

                let lite_user = LiteUser {
                    id: user.id.0,
                    nickname: nickname.clone(),
                    username: user.username.clone(),
                    channel_name,
                };
                if !lite_user.username.is_empty() {
                    ctx.online_users_by_username
                        .insert(lite_user.username.clone(), lite_user.id);
                }
                ctx.online_users.insert(user.id.0, lite_user.clone());

                let is_ready = ready_time
                    .map(|t| t.elapsed() >= Duration::from_secs(2))
                    .unwrap_or(false);

                if is_ready && !tt_config.global_ignore_usernames.contains(&user.username) {
                    let real_name = client.get_server_properties().map(|p| p.name);
                    let server_name = tt_config
                        .server_name
                        .as_deref()
                        .filter(|&s| !s.is_empty())
                        .or(real_name.as_deref().filter(|&s| !s.is_empty()))
                        .unwrap_or(&tt_config.host_name)
                        .to_string();

                    let _ = ctx.tx_bridge.blocking_send(BridgeEvent::Broadcast {
                        event_type: NotificationType::Join,
                        nickname,
                        server_name,
                        related_tt_username: user.username.clone(),
                    });
                }
            }
        }
        Event::UserJoined => {
            if let Some(user) = msg.user()
                && user.id != client.my_id()
            {
                let nickname = user.nickname.clone();
                let channel_name = client
                    .get_channel(user.channel_id)
                    .map(|c| c.name)
                    .unwrap_or_else(|| "Unknown".to_string());

                let lite_user = LiteUser {
                    id: user.id.0,
                    nickname,
                    username: user.username.clone(),
                    channel_name,
                };
                if !lite_user.username.is_empty() {
                    ctx.online_users_by_username
                        .insert(lite_user.username.clone(), lite_user.id);
                }
                ctx.online_users.insert(user.id.0, lite_user);
            }
        }

        Event::UserLoggedOut => {
            if let Some(user) = msg.user()
                && let Some((_, u)) = ctx.online_users.remove(&user.id.0)
            {
                if !u.username.is_empty() {
                    ctx.online_users_by_username.remove(&u.username);
                }
                if user.id != client.my_id() {
                    let is_ready = ready_time
                        .map(|t| t.elapsed() >= Duration::from_secs(2))
                        .unwrap_or(false);
                    if is_ready && !tt_config.global_ignore_usernames.contains(&u.username) {
                        let real_name = client.get_server_properties().map(|p| p.name);
                        let server_name = tt_config
                            .server_name
                            .as_deref()
                            .filter(|&s| !s.is_empty())
                            .or(real_name.as_deref().filter(|&s| !s.is_empty()))
                            .unwrap_or(&tt_config.host_name)
                            .to_string();

                        let _ = ctx.tx_bridge.blocking_send(BridgeEvent::Broadcast {
                            event_type: NotificationType::Leave,
                            nickname: u.nickname.clone(),
                            server_name,
                            related_tt_username: u.username.clone(),
                        });
                    }
                }
            }
        }
        Event::UserLeft => {
            if let Some(user) = msg.user() {
                let chan = client.get_channel(user.channel_id);
                let channel_name = chan
                    .as_ref()
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                if let Some(mut u) = ctx.online_users.get_mut(&user.id.0) {
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
