#![allow(clippy::pedantic)]

use crate::adapters::tt::{WorkerContext, resolve_server_name};
use crate::args;
use crate::core::types::{BridgeEvent, TgChatId, TgMessageId, TtNickname};
use crate::infra::locales;
use std::fmt::Write;
use teamtalk::Client;

use crate::core::types::LanguageCode;

pub(super) fn handle_who_command(
    client: &Client,
    ctx: &WorkerContext,
    chat_id: TgChatId,
    lang: LanguageCode,
    reply_to: Option<TgMessageId>,
) {
    let tt_config = &ctx.config.teamtalk;

    let real_name = client.get_server_properties().map(|p| p.name);
    let server_name = resolve_server_name(tt_config, real_name.as_deref());

    let users = client.get_server_users();
    let mut channels_data: std::collections::BTreeMap<String, Vec<TtNickname>> =
        std::collections::BTreeMap::new();
    let mut unauth_users: Vec<TtNickname> = Vec::new();

    for user in &users {
        let nickname = TtNickname::from(user.nickname.clone());

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

    let header_args = args!(server = server_name.as_str(), count = total);
    let header = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::TtReportHeader,
        header_args.as_ref(),
    );

    let mut report = String::with_capacity(1024);
    if let Err(e) = writeln!(report, "{}\n", header) {
        tracing::error!(error = %e, "Failed to write who report header");
    }

    for (chan_name, mut nicks) in channels_data {
        nicks.sort_by_key(|a| a.as_str().to_lowercase());

        let user_list = nicks
            .iter()
            .map(|nick| nick.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        let location = if chan_name == "ROOT_MARKER" {
            locales::get_text(lang.as_str(), locales::LocaleKey::TtReportRoot, None)
        } else {
            chan_name
        };

        let row_args = args!(users = user_list, channel = location);
        let row_text = locales::get_text(
            lang.as_str(),
            locales::LocaleKey::TtReportRow,
            row_args.as_ref(),
        );

        if let Err(e) = writeln!(report, "{}", row_text) {
            tracing::error!(error = %e, "Failed to write who report row");
        }
    }
    if !unauth_users.is_empty() {
        let unauth_label =
            locales::get_text(lang.as_str(), locales::LocaleKey::TtReportUnauth, None);
        let unauth_list = unauth_users
            .iter()
            .map(|nick| nick.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        if let Err(e) = writeln!(report, "{} {}", unauth_list, unauth_label) {
            tracing::error!(error = %e, "Failed to write who report unauth row");
        }
    }

    let tx_bridge = ctx.tx_bridge.clone();
    let text = report.trim_end().to_string();
    tokio::task::spawn_local(async move {
        if let Err(e) = tx_bridge
            .send(BridgeEvent::WhoReport {
                chat_id,
                text,
                reply_to,
            })
            .await
        {
            tracing::error!(
                chat_id = chat_id.as_i64(),
                error = %e,
                "Failed to send who report to bridge"
            );
        }
    });
}
