use crate::adapters::tt::WorkerContext;
use crate::core::types::TtChannelPassword;
use teamtalk::Client;
use teamtalk::client::ReconnectHandler;
use teamtalk::types::UserStatus;

use super::parse_gender_cfg;

pub(super) fn handle_connect_success(
    client: &Client,
    ctx: &WorkerContext,
    is_connected: &mut bool,
    reconnect_handler: &mut ReconnectHandler,
) {
    *is_connected = true;
    reconnect_handler.mark_connected();
    let tt_config = &ctx.config.teamtalk;
    client.login(
        tt_config.nick_name.as_str(),
        tt_config.user_name.as_str(),
        tt_config.password.as_str(),
        tt_config.client_name.as_str(),
    );
}

pub(super) fn handle_disconnect(
    ctx: &WorkerContext,
    event: teamtalk::Event,
    is_connected: &mut bool,
    reconnect_handler: &mut ReconnectHandler,
    ready_time: &mut Option<std::time::Instant>,
) {
    *is_connected = false;
    reconnect_handler.mark_disconnected();
    ctx.state.notify_clear_online_users();
    *ready_time = None;
    tracing::warn!(
        component = "tt_worker",
        event = ?event,
        "Disconnection event; reconnect pending"
    );
}

pub(super) fn handle_logged_in(
    client: &Client,
    ctx: &WorkerContext,
    ready_time: &mut Option<std::time::Instant>,
) {
    let tt_config = &ctx.config.teamtalk;
    let gender = parse_gender_cfg(ctx.config.general.gender);
    let status = UserStatus {
        gender,
        ..UserStatus::default()
    };
    client.set_status(status, &tt_config.status_text);
    let chan_id = client.get_channel_id_from_path(tt_config.channel.as_str());
    if chan_id.0 > 0 {
        let cmd_id = client.join_channel(
            chan_id,
            tt_config
                .channel_password
                .as_ref()
                .map_or("", TtChannelPassword::as_str),
        );
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
    ctx.state.notify_clear_user_accounts();
    client.list_user_accounts(0, 1000);
}
