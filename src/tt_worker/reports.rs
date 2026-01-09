use crate::args;
use crate::locales;
use crate::tt_worker::WorkerContext;
use crate::types::BridgeEvent;
use std::fmt::Write;
use teamtalk::Client;

pub(super) fn handle_who_command(client: &Client, ctx: &WorkerContext, chat_id: i64, lang: String) {
    let tt_config = &ctx.config.teamtalk;

    let real_name = client.get_server_properties().map(|p| p.name);
    let server_name = tt_config
        .server_name
        .as_deref()
        .filter(|&s| !s.is_empty())
        .or(real_name.as_deref().filter(|&s| !s.is_empty()))
        .unwrap_or(&tt_config.host_name)
        .to_string();

    let users = client.get_server_users();
    let mut channels_data: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    let mut unauth_users: Vec<String> = Vec::new();

    for user in &users {
        let nickname = user.nickname.clone();

        if user.channel_id.0 <= 0 {
            unauth_users.push(nickname);
            continue;
        }

        let chan = client.get_channel(user.channel_id);
        let chan_name = chan.as_ref().map(|c| c.name.clone()).unwrap_or_default();

        let chan_display = if chan_name.is_empty() && user.channel_id.0 == 1 {
            "ROOT_MARKER".to_string()
        } else {
            chan_name
        };
        channels_data
            .entry(chan_display)
            .or_default()
            .push(nickname);
    }

    let total = users.len();

    let header_args = args!(server = server_name, count = total);
    let header = locales::get_text(&lang, "tt-report-header", header_args.as_ref());

    let mut report = String::with_capacity(1024);
    writeln!(report, "{}\n", header).unwrap();

    for (chan_name, mut nicks) in channels_data {
        nicks.sort_by_key(|a| a.to_lowercase());

        let user_list = nicks.join(", ");

        let location = if chan_name == "ROOT_MARKER" {
            locales::get_text(&lang, "tt-report-root", None)
        } else {
            chan_name
        };

        let row_args = args!(users = user_list, channel = location);
        let row_text = locales::get_text(&lang, "tt-report-row", row_args.as_ref());

        writeln!(report, "{}", row_text).unwrap();
    }
    if !unauth_users.is_empty() {
        let unauth_label = locales::get_text(&lang, "tt-report-unauth", None);
        writeln!(report, "{} {}", unauth_users.join(", "), unauth_label).unwrap();
    }

    let _ = ctx.tx_bridge.blocking_send(BridgeEvent::WhoReport {
        chat_id,
        text: report.trim_end().to_string(),
    });
}
