use crate::app::state::StateHandle;
use crate::bootstrap::config::Config;
use crate::core::types::{
    BridgeEvent, LanguageCode, TtChannelName, TtCommand, TtServerName, TtUsername,
};
use crate::infra::db::Database;
use crate::infra::locales;
use std::sync::Arc;
use teamtalk::Client;
use teamtalk::types::ChannelId;
use tokio::sync::Semaphore;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;

pub fn resolve_server_name(
    tt_config: &crate::bootstrap::config::TeamTalkConfig,
    real_name: Option<&str>,
) -> TtServerName {
    let resolved = tt_config
        .server_name
        .as_deref()
        .filter(|s| !s.is_empty())
        .or_else(|| real_name.filter(|s| !s.is_empty()))
        .unwrap_or(&tt_config.host_name)
        .to_string();
    TtServerName::from(resolved)
}

pub fn resolve_channel_name(
    client: &Client,
    channel_id: ChannelId,
    lang: LanguageCode,
) -> TtChannelName {
    if channel_id.0 == 0 {
        return TtChannelName::from(locales::get_text(
            lang.as_str(),
            locales::LocaleKey::TtRootChannelName,
            None,
        ));
    }
    match client.get_channel(channel_id) {
        Some(channel) if !channel.name.is_empty() => TtChannelName::from(channel.name),
        Some(_) => TtChannelName::from(locales::get_text(
            lang.as_str(),
            locales::LocaleKey::TtRootChannelName,
            None,
        )),
        None => TtChannelName::new("Unknown"),
    }
}

pub struct WorkerContext {
    pub config: Arc<Config>,
    pub state: StateHandle,
    pub tx_bridge: tokio::sync::mpsc::Sender<BridgeEvent>,
    pub tx_tt_cmd: Sender<TtCommand>,
    pub db: Database,
    pub bot_username: Option<TtUsername>,
    pub is_streaming: Arc<std::sync::atomic::AtomicBool>,
    pub tt_msg_sem: Arc<Semaphore>,
}

pub struct RunTeamtalkArgs {
    pub config: Arc<Config>,
    pub state: StateHandle,
    pub tx_bridge: tokio::sync::mpsc::Sender<BridgeEvent>,
    pub rx_cmd: Receiver<TtCommand>,
    pub tx_cmd_clone: Sender<TtCommand>,
    pub db: Database,
    pub bot_username: Option<TtUsername>,
    pub client: Client,
    pub tx_init: oneshot::Sender<Result<(), String>>,
}
