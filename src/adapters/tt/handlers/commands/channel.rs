use crate::adapters::tt::{WorkerContext, resolve_channel_name, resolve_server_name};
use crate::core::types::{LanguageCode, TtChannelId, TtCommand, TtUserId, TtUsername};
use crate::infra::locales;
use teamtalk::Client;
use teamtalk::types::TextMessage;
use tokio::task::spawn_local;

use crate::app::services::tt_users as tt_users_service;

pub(super) fn handle_channel_message(client: &Client, ctx: &WorkerContext, msg: &TextMessage) {
    let real_name_from_client = client.get_server_properties().map(|p| p.name);
    let tt_config = ctx.config.teamtalk.clone();
    let tx_tt_cmd = ctx.tx_tt_cmd.clone();
    let tx_bridge = ctx.tx_bridge.clone();
    let tt_msg_sem = ctx.tt_msg_sem.clone();
    let state_handle = ctx.state.clone();
    let services = ctx.services();
    let default_lang = ctx.config.general.default_lang;
    let admin_username = ctx.config.general.admin_username.clone();

    let content = msg.text.trim();
    let cmd = content
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_lowercase();

    if cmd == "/skip" {
        let from_uid = TtUserId::from(msg.from_id.0);
        let channel_id = TtChannelId::from(msg.channel_id.0);
        spawn_local(async move {
            let _permit = tt_msg_sem.acquire_owned().await;
            let username = state_handle
                .online_user_by_id(from_uid)
                .await
                .ok()
                .flatten()
                .map_or_else(|| TtUsername::new(String::new()), |u| u.username);
            let reply_lang =
                tt_users_service::resolve_reply_lang(&services, &username, default_lang).await;
            let is_admin =
                tt_users_service::resolve_is_admin(&services, &username, admin_username.as_ref())
                    .await;
            let text_key = if is_admin {
                if let Err(e) = tx_tt_cmd.send(TtCommand::SkipStream).await {
                    tracing::error!(
                        tt_username = %username,
                        error = %e,
                        "Failed to send TT skip command"
                    );
                    locales::LocaleKey::TtErrorGeneric
                } else {
                    locales::LocaleKey::TtSkipSent
                }
            } else {
                locales::LocaleKey::CmdUnauth
            };
            let text = locales::get_text_or_log(reply_lang.as_str(), text_key, None);
            if let Err(e) = tx_tt_cmd
                .send(TtCommand::SendToChannel { channel_id, text })
                .await
            {
                tracing::error!(
                    channel_id = channel_id.as_i32(),
                    error = %e,
                    "Failed to send TT channel reply"
                );
            }
        });
        return;
    }

    if let Some(rest) = content.strip_prefix("/pm ") {
        let pm_text = rest.trim();
        if pm_text.is_empty() {
            return;
        }
        let channel_name = resolve_channel_name(client, msg.channel_id, LanguageCode::En);
        let server_name = resolve_server_name(&tt_config, real_name_from_client.as_deref());
        let msg_content = pm_text.to_string();
        let channel_id = TtChannelId::from(msg.channel_id.0);
        spawn_local(async move {
            let _permit = tt_msg_sem.acquire_owned().await;
            if let Err(e) = tx_bridge
                .send(crate::core::types::BridgeEvent::ToAdminChannel {
                    channel_id,
                    channel_name,
                    server_name,
                    msg_content,
                })
                .await
            {
                tracing::error!(error = %e, "Failed to send channel PM event");
            }
        });
    }
}
