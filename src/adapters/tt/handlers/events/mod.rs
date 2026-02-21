use crate::adapters::tt::WorkerContext;
use crate::bootstrap::config::GenderConfig;
use crate::core::types::TtUsername;
use serde_json::json;
use std::time::Instant;
use teamtalk::client::ReconnectHandler;
use teamtalk::{Client, Event, Message};

mod accounts;
mod connection;
mod follow;
mod streaming;
mod text;
mod users;

pub fn handle_sdk_event(
    client: &Client,
    ctx: &WorkerContext,
    event: Event,
    msg: &Message,
    is_connected: &mut bool,
    reconnect_handler: &mut ReconnectHandler,
    ready_time: &mut Option<Instant>,
) {
    tracing::trace!(component = "tt_worker", event = ?event, "Event received");
    emit_plugin_event(ctx, event, msg);

    match event {
        Event::ConnectSuccess => {
            connection::handle_connect_success(client, ctx, is_connected, reconnect_handler);
        }
        e if e.is_reconnect_needed_with(&[Event::MySelfKicked]) => {
            connection::handle_disconnect(ctx, e, is_connected, reconnect_handler, ready_time);
        }
        Event::MySelfLoggedIn => {
            connection::handle_logged_in(client, ctx, ready_time);
        }
        Event::UserAccount => {
            accounts::handle_user_account(ctx, msg);
        }
        Event::UserAccountCreated | Event::UserAccountRemoved => {
            accounts::handle_user_account_change(client, ctx);
        }
        Event::UserUpdate => {
            users::handle_user_update(ctx, msg);
        }
        Event::StreamMediaFile => {
            streaming::handle_stream_media_file(client, ctx, msg);
        }
        Event::UserLoggedIn => {
            users::handle_user_logged_in(client, ctx, msg, ready_time.as_ref());
        }
        Event::UserJoined => {
            users::handle_user_joined(client, ctx, msg, ready_time.as_ref());
        }
        Event::UserLoggedOut => {
            users::handle_user_logged_out(client, ctx, msg, ready_time.as_ref());
        }
        Event::CmdSuccess | Event::CmdError => {
            follow::handle_command_result(event, ctx, msg);
        }
        Event::UserLeft => {
            users::handle_user_left(client, ctx, msg);
        }
        Event::TextMessage => {
            text::handle_text_message_event(client, ctx, msg);
        }
        _ => {}
    }
}

fn emit_plugin_event(ctx: &WorkerContext, event: Event, msg: &Message) {
    let plugins = ctx.plugins.clone();
    let db = ctx.db.clone();
    let event_name = format!("{event:?}");
    let has_user = msg.user().is_some();
    let has_text = msg.text().is_some();
    let source = msg.source().to_string();
    let linked_tt_username = msg.user().and_then(|user| {
        (!user.username.is_empty()).then(|| TtUsername::from(user.username.as_str()))
    });
    let raw = build_raw_payload(msg);
    tokio::task::spawn_local(async move {
        let linked_telegram_id = if let Some(tt_username) = linked_tt_username {
            db.get_telegram_id_by_tt_user(&tt_username)
                .await
                .map(crate::core::types::TelegramId::as_i64)
        } else {
            None
        };
        let normalized = json!({
            "event": event_name,
            "has_user": has_user,
            "has_text": has_text,
            "source": source,
            "linked_telegram_id": linked_telegram_id,
            "is_linked": linked_telegram_id.is_some(),
        });
        plugins
            .dispatch_event(crate::app::plugins::PluginEvent {
                name: normalized["event"].as_str().unwrap_or_default().to_string(),
                source: "tt".to_string(),
                normalized,
                raw,
            })
            .await;
    });
}

fn build_raw_payload(msg: &Message) -> serde_json::Value {
    let mut payload = serde_json::Map::new();
    payload.insert("source".to_string(), json!(msg.source().to_string()));
    if let Some(text) = msg.text() {
        payload.insert(
            "text".to_string(),
            json!({
                "from_id": text.from_id.0,
                "to_id": text.to_id.0,
                "channel_id": text.channel_id.0,
                "text": text.text,
                "msg_type": format!("{:?}", text.msg_type),
            }),
        );
    }
    if let Some(user) = msg.user() {
        payload.insert(
            "user".to_string(),
            json!({
                "id": user.id.0,
                "username": user.username,
                "nickname": user.nickname,
                "status_mode": user.status.to_bits(),
                "status_msg": user.status_msg,
                "channel_id": user.channel_id.0,
            }),
        );
    }
    serde_json::Value::Object(payload)
}

const fn parse_gender(cfg: GenderConfig) -> teamtalk::types::UserGender {
    cfg.to_user_gender()
}

pub(super) const fn parse_gender_cfg(cfg: GenderConfig) -> teamtalk::types::UserGender {
    parse_gender(cfg)
}
