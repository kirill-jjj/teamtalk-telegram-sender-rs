pub mod commands;
pub mod events;
pub mod reports;

use crate::config::Config;
use crate::db::Database;
use crate::types::{BridgeEvent, LiteUser, TtCommand};
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use teamtalk::Client;
use teamtalk::client::{ConnectParams, ReconnectConfig, ReconnectHandler};
use teamtalk::types::{UserAccount, UserId};

pub struct WorkerContext {
    pub config: Arc<Config>,
    pub online_users: Arc<DashMap<i32, LiteUser>>,
    pub online_users_by_username: Arc<DashMap<String, i32>>,
    pub user_accounts: Arc<DashMap<String, UserAccount>>,
    pub tx_bridge: tokio::sync::mpsc::Sender<BridgeEvent>,
    pub tx_tt_cmd: Sender<TtCommand>,
    pub db: Database,
    pub rt: tokio::runtime::Handle,
    pub bot_username: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub fn run_teamtalk_thread(
    config: Arc<Config>,
    online_users: Arc<DashMap<i32, LiteUser>>,
    online_users_by_username: Arc<DashMap<String, i32>>,
    user_accounts: Arc<DashMap<String, UserAccount>>,
    tx_bridge: tokio::sync::mpsc::Sender<BridgeEvent>,
    rx_cmd: Receiver<TtCommand>,
    tx_cmd_clone: Sender<TtCommand>,
    db: Database,
    rt: tokio::runtime::Handle,
    bot_username: Option<String>,
    tx_init: std::sync::mpsc::Sender<Result<(), String>>,
) {
    let tt_config = &config.teamtalk;
    let _reconnect_interval = config.operational_parameters.tt_reconnect_retry_seconds;

    let ctx = WorkerContext {
        config: config.clone(),
        online_users: online_users.clone(),
        online_users_by_username,
        user_accounts,
        tx_bridge,
        tx_tt_cmd: tx_cmd_clone,
        db,
        rt,
        bot_username,
    };

    let client = match Client::new() {
        Ok(c) => {
            let _ = tx_init.send(Ok(()));
            c
        }
        Err(e) => {
            let err_msg = format!("Failed to initialize TeamTalk SDK: {}", e);
            log::error!("{}", err_msg);
            let _ = tx_init.send(Err(err_msg));
            return;
        }
    };
    let mut ready_time: Option<std::time::Instant> = None;
    let mut is_connected = false;

    let mut reconnect_handler = ReconnectHandler::new(ReconnectConfig {
        min_delay: Duration::from_millis(200),
        max_delay: Duration::from_secs(60),
        max_attempts: u32::MAX,
        stability_threshold: Duration::from_secs(10),
    });

    let connect_params = ConnectParams {
        host: &tt_config.host_name,
        tcp: tt_config.port as i32,
        udp: tt_config.port as i32,
        encrypted: tt_config.encrypted,
    };

    log::info!(
        "🔌 [TT_WORKER] Connecting to {}:{} (Encrypted: {})...",
        tt_config.host_name,
        tt_config.port,
        tt_config.encrypted
    );

    let _ = client.connect(
        connect_params.host,
        connect_params.tcp,
        connect_params.udp,
        connect_params.encrypted,
    );

    loop {
        if !is_connected {
            client.handle_reconnect(&connect_params, &mut reconnect_handler);
        }

        while let Ok(cmd) = rx_cmd.try_recv() {
            match cmd {
                TtCommand::ReplyToUser { user_id, text } => {
                    client.send_to_user(UserId(user_id), &text);
                }
                TtCommand::KickUser { user_id } => {
                    client.kick_user(UserId(user_id), teamtalk::types::ChannelId(0));
                }
                TtCommand::BanUser { user_id } => {
                    client.ban_user(UserId(user_id), client.my_channel_id());
                }
                TtCommand::Who { chat_id, lang } => {
                    reports::handle_who_command(&client, &ctx, chat_id, lang);
                }
                TtCommand::LoadAccounts => {
                    log::info!("📥 [TT_WORKER] Requesting full user accounts list...");
                    client.list_user_accounts(0, 1000);
                }
            }
        }

        while let Some((event, msg)) = client.poll(100) {
            events::handle_sdk_event(
                &client,
                &ctx,
                event,
                msg,
                &mut is_connected,
                &mut reconnect_handler,
                &mut ready_time,
            );
        }
    }
}
