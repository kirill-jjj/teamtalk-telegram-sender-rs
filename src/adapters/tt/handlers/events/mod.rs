use crate::adapters::tt::WorkerContext;
use crate::bootstrap::config::GenderConfig;
use std::time::Instant;
use teamtalk::client::ReconnectHandler;
use teamtalk::{Client, Event, Message};

mod accounts;
mod connection;
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
            users::handle_user_joined(client, ctx, msg);
        }
        Event::UserLoggedOut => {
            users::handle_user_logged_out(client, ctx, msg, ready_time.as_ref());
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

const fn parse_gender(cfg: GenderConfig) -> teamtalk::types::UserGender {
    cfg.to_user_gender()
}

pub(super) const fn parse_gender_cfg(cfg: GenderConfig) -> teamtalk::types::UserGender {
    parse_gender(cfg)
}
