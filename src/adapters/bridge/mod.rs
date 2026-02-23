use crate::app::services::tt_context::TtServiceContext;
use crate::app::state::StateHandle;
use crate::bootstrap::config::Config;
use crate::core::types::{self, BridgeEvent, LanguageCode};
use crate::infra::db::Database;
use std::sync::Arc;
use teloxide_ng::prelude::Bot;
use tokio::sync::mpsc::Sender;

mod admin;
mod admin_channel;
mod broadcast;
mod tg_document;
mod who;

struct BridgeDeps<'a> {
    services: TtServiceContext,
    state: &'a StateHandle,
    event_bot: Option<&'a Bot>,
    msg_bot: Option<&'a Bot>,
    message_token_present: bool,
    default_lang: LanguageCode,
    admin_id: teloxide_ng::types::ChatId,
    tx_tt_cmd: &'a Sender<types::TtCommand>,
}

pub struct BridgeContext {
    pub db: Database,
    pub state: StateHandle,
    pub config: Arc<Config>,
    pub event_bot: Option<Bot>,
    pub msg_bot: Option<Bot>,
    pub message_token_present: bool,
    pub tx_tt_cmd: Sender<types::TtCommand>,
    pub cancel_token: tokio_util::sync::CancellationToken,
}

pub async fn run_bridge(
    ctx: BridgeContext,
    mut rx_bridge: tokio::sync::mpsc::Receiver<BridgeEvent>,
) {
    let BridgeContext {
        db: db_clone,
        state,
        config,
        event_bot,
        msg_bot,
        message_token_present,
        tx_tt_cmd,
        cancel_token,
    } = ctx;
    let default_lang = config.general.default_lang;
    let admin_id = teloxide_ng::types::ChatId(config.telegram.admin_chat_id.as_i64());
    let deps = BridgeDeps {
        services: TtServiceContext::new(db_clone.clone(), state.clone()),
        state: &state,
        event_bot: event_bot.as_ref(),
        msg_bot: msg_bot.as_ref(),
        message_token_present,
        default_lang,
        admin_id,
        tx_tt_cmd: &tx_tt_cmd,
    };

    tracing::info!(component = "bridge", "Bridge task started");
    loop {
        let event = tokio::select! {
            () = cancel_token.cancelled() => {
                break;
            }
            bridge_event_opt = rx_bridge.recv() => {
                match bridge_event_opt {
                    Some(event) => event,
                    None => break,
                }
            }
        };

        handle_bridge_event(&deps, event).await;
    }
}

async fn handle_bridge_event(deps: &BridgeDeps<'_>, event: BridgeEvent) {
    match event {
        types::BridgeEvent::Broadcast {
            event_type,
            nickname,
            server_name,
            related_tt_username,
            gender,
        } => {
            broadcast::handle_broadcast(
                deps,
                event_type,
                nickname,
                server_name,
                related_tt_username,
                gender,
            )
            .await;
        }
        types::BridgeEvent::ToAdmin {
            user_id,
            nick,
            tt_username,
            msg_content,
            server_name,
        } => {
            admin::handle_to_admin(deps, user_id, nick, tt_username, msg_content, server_name)
                .await;
        }
        types::BridgeEvent::ToAdminChannel {
            channel_id,
            channel_name,
            server_name,
            msg_content,
        } => {
            admin_channel::handle_to_admin_channel(
                deps,
                channel_id,
                channel_name,
                server_name,
                msg_content,
            )
            .await;
        }
        types::BridgeEvent::WhoReport {
            chat_id,
            text,
            reply_to,
        } => {
            who::handle_who_report(deps, chat_id, text, reply_to).await;
        }
        types::BridgeEvent::TgDocument {
            chat_id,
            file_path,
            caption,
            delete_after_send,
        } => {
            tg_document::handle_tg_document(deps, chat_id, file_path, caption, delete_after_send)
                .await;
        }
        types::BridgeEvent::AfkStatus {
            recipient,
            nickname,
            is_afk,
        } => {
            broadcast::handle_afk_status(deps, recipient, nickname, is_afk).await;
        }
    }
}
