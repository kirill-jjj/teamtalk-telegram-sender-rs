use crate::adapters::tt::WorkerContext;
use crate::core::types::TtChannelPassword;
use teamtalk::Client;
use teamtalk::types::UserStatus;

use super::parse_gender_cfg;

pub(super) fn handle_connect_success(client: &Client) {
    if let Err(err) = client.login_with_params() {
        tracing::error!(component = "tt_worker", error = %err, "Failed to start TeamTalk login");
    }
}

pub(super) fn handle_disconnect(
    ctx: &WorkerContext,
    event: teamtalk::Event,
    ready_time: &mut Option<std::time::Instant>,
) {
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
        client.set_last_channel(
            chan_id,
            tt_config
                .channel_password
                .as_ref()
                .map(TtChannelPassword::as_str),
        );
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
