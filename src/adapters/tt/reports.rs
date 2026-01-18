#![allow(clippy::pedantic)]

use crate::adapters::tt::{WorkerContext, resolve_server_name};
use crate::args;
use crate::core::types::BridgeEvent;
use crate::infra::locales;
use std::fmt::Write;
use teamtalk::Client;

use crate::core::types::LanguageCode;

pub(super) fn handle_who_command(
    client: &Client,
    ctx: &WorkerContext,
    chat_id: i64,
    lang: LanguageCode,
    reply_to: Option<i32>,
) {
    let tt_config = &ctx.config.teamtalk;

    let real_name = client.get_server_properties().map(|p| p.name);
    let server_name = resolve_server_name(tt_config, real_name.as_deref());

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
    let header = locales::get_text(lang.as_str(), "tt-report-header", header_args.as_ref());

    let mut report = String::with_capacity(1024);
    if let Err(e) = writeln!(report, "{}\n", header) {
        tracing::error!(error = %e, "Failed to write who report header");
    }

    for (chan_name, mut nicks) in channels_data {
        nicks.sort_by_key(|a| a.to_lowercase());

        let user_list = nicks.join(", ");

        let location = if chan_name == "ROOT_MARKER" {
            locales::get_text(lang.as_str(), "tt-report-root", None)
        } else {
            chan_name
        };

        let row_args = args!(users = user_list, channel = location);
        let row_text = locales::get_text(lang.as_str(), "tt-report-row", row_args.as_ref());

        if let Err(e) = writeln!(report, "{}", row_text) {
            tracing::error!(error = %e, "Failed to write who report row");
        }
    }
    if !unauth_users.is_empty() {
        let unauth_label = locales::get_text(lang.as_str(), "tt-report-unauth", None);
        if let Err(e) = writeln!(report, "{} {}", unauth_users.join(", "), unauth_label) {
            tracing::error!(error = %e, "Failed to write who report unauth row");
        }
    }

    if let Err(e) = ctx.tx_bridge.blocking_send(BridgeEvent::WhoReport {
        chat_id,
        text: report.trim_end().to_string(),
        reply_to,
    }) {
        tracing::error!(chat_id, error = %e, "Failed to send who report to bridge");
    }
}
