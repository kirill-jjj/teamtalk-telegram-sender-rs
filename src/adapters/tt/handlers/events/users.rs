use crate::adapters::tt::{WorkerContext, resolve_channel_name, resolve_server_name};
use crate::app::services::reply_queue as reply_queue_service;
use crate::app::services::tt_users as tt_users_service;
use crate::core::types::{
    BridgeEvent, LanguageCode, LiteUser, NotificationType, TtNickname, TtUserId, TtUsername,
};
use chrono::Utc;
use std::time::Duration;
use teamtalk::{Client, Message};

pub(super) fn handle_user_update(ctx: &WorkerContext, msg: &Message) {
    if let Some(user) = msg.user() {
        let state = ctx.state.clone();
        let user_id = TtUserId::from(user.id.0);
        let new_username = TtUsername::from(user.username.as_str());
        let new_nickname = TtNickname::from(user.nickname.as_str());
        tokio::task::spawn_local(async move {
            if let Some(existing) = state.online_user_by_id(user_id).await.ok().flatten() {
                if existing.username != new_username {
                    state.notify_update_user_username(user_id, new_username.clone());
                }
                if existing.nickname != new_nickname {
                    tracing::info!(
                        component = "tt_worker",
                        username = %new_username,
                        old_nick = %existing.nickname,
                        new_nick = %new_nickname,
                        "Nickname changed"
                    );
                    state.notify_update_user_nickname(user_id, new_nickname.clone());
                }
            }
        });
    }
}

pub(super) fn handle_user_logged_in(
    client: &Client,
    ctx: &WorkerContext,
    msg: &Message,
    ready_time: Option<&std::time::Instant>,
) {
    let tt_config = &ctx.config.teamtalk;
    if let Some(user) = msg.user()
        && user.id != client.my_id()
    {
        let nickname = TtNickname::from(user.nickname.as_str());

        let channel_name = resolve_channel_name(client, user.channel_id, LanguageCode::En);

        let lite_user = LiteUser {
            id: TtUserId::from(user.id.0),
            nickname: nickname.clone(),
            username: TtUsername::from(user.username.as_str()),
            channel_name,
        };
        ctx.state.notify_upsert_online_user(lite_user.clone());

        let is_ready = ready_time.is_some_and(|t| t.elapsed() >= Duration::from_secs(2));

        if is_ready
            && !tt_config
                .global_ignore_usernames
                .contains(&lite_user.username)
        {
            let real_name = client.get_server_properties().map(|p| p.name);
            let server_name = resolve_server_name(tt_config, real_name.as_deref());

            let tx_bridge = ctx.tx_bridge.clone();
            let related_tt_username = TtUsername::from(user.username.as_str());
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
            let services = ctx.services();
            let tx_tt_cmd = ctx.tx_tt_cmd.clone();
            let tt_username = TtUsername::from(user.username.as_str());
            let default_lang = ctx.config.general.default_lang;
            let user_id = TtUserId::from(user.id.0);
            tokio::task::spawn_local(async move {
                let mut items =
                    match reply_queue_service::get_reply_queue_for_user(&services.db, &tt_username)
                        .await
                    {
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
                let lang = tt_users_service::get_user_lang_by_tt_user(
                    &services,
                    &tt_username,
                    default_lang,
                )
                .await;
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
                        .send(crate::core::types::TtCommand::ReplyToUser {
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
                    && let Err(e) =
                        reply_queue_service::delete_reply_queue_ids(&services.db, &sent_ids).await
                {
                    tracing::error!(error = %e, "Failed to clear sent queue items");
                }
            });
        }
    }
}

pub(super) fn handle_user_joined(client: &Client, ctx: &WorkerContext, msg: &Message) {
    if let Some(user) = msg.user()
        && user.id != client.my_id()
    {
        let nickname = TtNickname::from(user.nickname.as_str());
        let channel_name = resolve_channel_name(client, user.channel_id, LanguageCode::En);

        let lite_user = LiteUser {
            id: TtUserId::from(user.id.0),
            nickname,
            username: TtUsername::from(user.username.as_str()),
            channel_name,
        };
        ctx.state.notify_upsert_online_user(lite_user);
    }
}

pub(super) fn handle_user_logged_out(
    client: &Client,
    ctx: &WorkerContext,
    msg: &Message,
    ready_time: Option<&std::time::Instant>,
) {
    if let Some(user) = msg.user() {
        let is_ready = ready_time.is_some_and(|t| t.elapsed() >= Duration::from_secs(2));
        let real_name = client.get_server_properties().map(|p| p.name);
        let server_name = resolve_server_name(&ctx.config.teamtalk, real_name.as_deref());
        let tx_bridge = ctx.tx_bridge.clone();
        let state = ctx.state.clone();
        let user_id = TtUserId::from(user.id.0);
        let is_self = user.id == client.my_id();
        let tt_config = ctx.config.teamtalk.clone();
        tokio::task::spawn_local(async move {
            let removed = state.remove_online_user(user_id).await.ok().flatten();
            if let Some(u) = removed
                && !is_self
                && is_ready
                && !tt_config.global_ignore_usernames.contains(&u.username)
                && let Err(e) = tx_bridge
                    .send(BridgeEvent::Broadcast {
                        event_type: NotificationType::Leave,
                        nickname: u.nickname.clone(),
                        server_name,
                        related_tt_username: u.username.clone(),
                    })
                    .await
            {
                tracing::error!(error = %e, "Failed to send leave broadcast");
            }
        });
    }
}

pub(super) fn handle_user_left(client: &Client, ctx: &WorkerContext, msg: &Message) {
    if let Some(user) = msg.user() {
        let channel_name = resolve_channel_name(client, user.channel_id, LanguageCode::En);
        ctx.state
            .notify_update_user_channel(TtUserId::from(user.id.0), channel_name);
    }
}
