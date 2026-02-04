use crate::bootstrap::config::Config;
use crate::core::types::{
    BridgeEvent, LanguageCode, LiteUser, TtChannelName, TtCommand, TtUsername,
};
use crate::infra::db::Database;
use crate::infra::locales;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};
use teamtalk::Client;
use teamtalk::types::{ChannelId, UserAccount};
use tokio::sync::Semaphore;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;

pub fn resolve_server_name(
    tt_config: &crate::bootstrap::config::TeamTalkConfig,
    real_name: Option<&str>,
) -> String {
    tt_config
        .server_name
        .as_deref()
        .filter(|s| !s.is_empty())
        .or_else(|| real_name.filter(|s| !s.is_empty()))
        .unwrap_or(&tt_config.host_name)
        .to_string()
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
    pub online_users: Arc<RwLock<HashMap<i32, LiteUser>>>,
    pub online_users_by_username: Arc<RwLock<HashMap<TtUsername, i32>>>,
    pub user_accounts: Arc<RwLock<HashMap<TtUsername, UserAccount>>>,
    pub tx_bridge: tokio::sync::mpsc::Sender<BridgeEvent>,
    pub tx_tt_cmd: Sender<TtCommand>,
    pub db: Database,
    pub bot_username: Option<TtUsername>,
    pub is_streaming: Arc<std::sync::atomic::AtomicBool>,
    pub tt_msg_sem: Arc<Semaphore>,
    pub tt_lang_cache: Arc<RwLock<HashMap<TtUsername, LanguageCode>>>,
    pub tt_tg_cache: Arc<RwLock<HashMap<TtUsername, i64>>>,
    pub tt_cache_stats: Arc<TtCacheStats>,
}

pub struct TtCacheStats {
    pub lang_hits: AtomicU64,
    pub lang_misses: AtomicU64,
    pub tg_hits: AtomicU64,
    pub tg_misses: AtomicU64,
}

pub struct RunTeamtalkArgs {
    pub config: Arc<Config>,
    pub online_users: Arc<RwLock<HashMap<i32, LiteUser>>>,
    pub online_users_by_username: Arc<RwLock<HashMap<TtUsername, i32>>>,
    pub user_accounts: Arc<RwLock<HashMap<TtUsername, UserAccount>>>,
    pub tx_bridge: tokio::sync::mpsc::Sender<BridgeEvent>,
    pub rx_cmd: Receiver<TtCommand>,
    pub tx_cmd_clone: Sender<TtCommand>,
    pub db: Database,
    pub bot_username: Option<TtUsername>,
    pub client: Client,
    pub tx_init: oneshot::Sender<Result<(), String>>,
}
