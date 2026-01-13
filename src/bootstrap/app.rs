use crate::adapters;
use crate::bootstrap::config::Config;
use crate::infra::db::Database;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc as std_mpsc;
use std::sync::{Arc, RwLock};
use teamtalk::types::UserAccount;
use teloxide::{Bot, prelude::Requester};
use tokio::sync::mpsc as tokio_mpsc;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

pub struct Application {
    config: Arc<Config>,
    db: Database,
    cancel_token: CancellationToken,
}

impl Application {
    pub async fn build(config_path: String) -> Result<Self> {
        tracing::info!("Loading config from: {}", config_path);

        let config_content = std::fs::read_to_string(&config_path)?;
        let mut config: Config = toml::from_str(&config_content)?;

        let config_path_obj = Path::new(&config_path);
        let config_dir = config_path_obj.parent().unwrap_or_else(|| Path::new("."));

        let db_path_buf = if Path::new(&config.database.db_file).is_absolute() {
            Path::new(&config.database.db_file).to_path_buf()
        } else {
            config_dir.join(&config.database.db_file)
        };

        let db_path_str = db_path_buf
            .to_str()
            .ok_or_else(|| anyhow!("Invalid DB path"))?
            .to_string();
        tracing::info!("Database path: {}", db_path_str);

        config.database.db_file = db_path_str.clone();

        let config = Arc::new(config);
        let db = Database::new(&db_path_str).await?;
        let cancel_token = CancellationToken::new();

        Ok(Self {
            config,
            db,
            cancel_token,
        })
    }

    pub async fn run(self) -> Result<()> {
        let Application {
            config,
            db,
            cancel_token,
        } = self;

        let db_for_cleanup = db.clone();
        let cleanup_interval = config
            .operational_parameters
            .deeplink_cleanup_interval_seconds;
        {
            let cancel_token = cancel_token.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(cleanup_interval));
                loop {
                    tokio::select! {
                        _ = cancel_token.cancelled() => break,
                        _ = interval.tick() => {}
                    }
                    match db_for_cleanup.cleanup_expired_deeplinks().await {
                        Ok(count) if count > 0 => {
                            tracing::info!("Cleaned up {} expired deeplinks.", count);
                        }
                        Ok(_) => {}
                        Err(e) => {
                            tracing::error!("Failed to clean up expired deeplinks: {}", e);
                        }
                    }
                }
            });
        }

        let pending_cleanup_interval_seconds = 3600u64;
        let pending_ttl_seconds = 3600i64;
        let db_for_pending_cleanup = db.clone();
        {
            let cancel_token = cancel_token.clone();
            tokio::spawn(async move {
                let mut interval =
                    tokio::time::interval(Duration::from_secs(pending_cleanup_interval_seconds));
                loop {
                    tokio::select! {
                        _ = cancel_token.cancelled() => break,
                        _ = interval.tick() => {}
                    }
                    match db_for_pending_cleanup
                        .cleanup_pending_replies(pending_ttl_seconds)
                        .await
                    {
                        Ok(count) if count > 0 => {
                            tracing::info!("Cleaned up {} pending replies.", count);
                        }
                        Ok(_) => {}
                        Err(e) => {
                            tracing::error!("Failed to clean up pending replies: {}", e);
                        }
                    }
                    match db_for_pending_cleanup
                        .cleanup_pending_channel_replies(pending_ttl_seconds)
                        .await
                    {
                        Ok(count) if count > 0 => {
                            tracing::info!("Cleaned up {} pending channel replies.", count);
                        }
                        Ok(_) => {}
                        Err(e) => {
                            tracing::error!("Failed to clean up pending channel replies: {}", e);
                        }
                    }
                }
            });
        }

        let online_users: Arc<RwLock<HashMap<i32, crate::core::types::LiteUser>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let online_users_by_username: Arc<RwLock<HashMap<String, i32>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let all_user_accounts: Arc<RwLock<HashMap<String, UserAccount>>> =
            Arc::new(RwLock::new(HashMap::new()));

        let (tx_bridge, rx_bridge) = tokio_mpsc::channel::<crate::core::types::BridgeEvent>(100);
        let (tx_tt_cmd, rx_tt_cmd) = std_mpsc::channel::<crate::core::types::TtCommand>();

        let event_token = config.telegram.event_token.clone();
        let message_token = config.telegram.message_token.clone();
        let same_token = event_token.is_some() && message_token == event_token;

        let event_bot = if let Some(token) = &event_token {
            Some(Bot::new(token))
        } else {
            tracing::warn!(
                "'event_token' missing. Telegram interactions and notifications disabled."
            );
            None
        };

        let message_bot = if same_token {
            tracing::info!(
                "message_token matches event_token; using event bot for admin messages."
            );
            None
        } else if let Some(token) = &message_token {
            Some(Bot::new(token))
        } else {
            tracing::warn!("'message_token' missing. Admin alerts disabled.");
            None
        };

        let bot_username = if let Some(bot) = &event_bot {
            let me = bot.get_me().await?;
            let username = me
                .username
                .clone()
                .ok_or_else(|| anyhow!("Bot must have a username!"))?;
            tracing::info!("Interaction bot username: @{}", username);
            Some(username)
        } else {
            None
        };

        let tt_users = online_users.clone();
        let tt_users_by_username = online_users_by_username.clone();
        let tt_accounts = all_user_accounts.clone();
        let tx_bridge_clone = tx_bridge.clone();
        let db_for_tt = db.clone();
        let rt_handle = tokio::runtime::Handle::current();
        let bot_username_for_tt = bot_username.clone();

        let config_for_worker = config.clone();
        let tx_tt_cmd_for_worker = tx_tt_cmd.clone();

        let (tx_init, rx_init) = std::sync::mpsc::channel();

        let tt_handle = std::thread::spawn(move || {
            adapters::tt::run_teamtalk_thread(adapters::tt::RunTeamtalkArgs {
                config: config_for_worker,
                online_users: tt_users,
                online_users_by_username: tt_users_by_username,
                user_accounts: tt_accounts,
                tx_bridge: tx_bridge_clone,
                rx_cmd: rx_tt_cmd,
                tx_cmd_clone: tx_tt_cmd_for_worker,
                db: db_for_tt,
                rt: rt_handle,
                bot_username: bot_username_for_tt,
                tx_init,
            });
        });

        match rx_init.recv() {
            Ok(Ok(_)) => tracing::info!("TeamTalk worker started successfully"),
            Ok(Err(e)) => return Err(anyhow!("TeamTalk worker failed to start: {}", e)),
            Err(_) => return Err(anyhow!("TeamTalk worker disconnected during startup")),
        }

        let event_bot_clone = event_bot.clone();
        let msg_bot_clone = message_bot.clone();
        let db_clone = db.clone();
        let online_users_clone = online_users.clone();
        let message_token_present = message_token.is_some();
        let cancel_token_for_bridge = cancel_token.clone();

        let bridge_handle = tokio::spawn(adapters::bridge::run_bridge(
            adapters::bridge::BridgeContext {
                db: db_clone,
                online_users: online_users_clone,
                config: config.clone(),
                event_bot: event_bot_clone,
                msg_bot: msg_bot_clone,
                message_token_present,
                tx_tt_cmd: tx_tt_cmd.clone(),
                cancel_token: cancel_token_for_bridge,
            },
            rx_bridge,
        ));

        let cancel_token_for_signal = cancel_token.clone();
        let tx_tt_cmd_for_shutdown = tx_tt_cmd.clone();
        tokio::spawn(async move {
            wait_for_shutdown_signal(cancel_token_for_signal, tx_tt_cmd_for_shutdown).await;
        });

        if let Some(bot) = event_bot {
            adapters::tg::run_tg_bot(adapters::tg::TgRunArgs {
                event_bot: bot,
                message_bot,
                db: db.clone(),
                online_users,
                user_accounts: all_user_accounts,
                tx_tt_cmd,
                config,
                cancel_token: cancel_token.clone(),
            })
            .await;
            let _ = bridge_handle.await;
            let _ = tt_handle.join();
        } else if let Err(e) = bridge_handle.await {
            tracing::error!("Bridge task failed: {}", e);
            let _ = tt_handle.join();
        }

        tracing::info!("[SHUTDOWN] Closing database pool...");
        db.close().await;
        tracing::info!("[SHUTDOWN] Database pool closed.");

        Ok(())
    }
}

#[cfg(unix)]
async fn wait_for_termination_signal() {
    use tokio::signal::unix::{SignalKind, signal};

    let mut sigterm = match signal(SignalKind::terminate()) {
        Ok(sigterm) => sigterm,
        Err(e) => {
            tracing::error!("Failed to register SIGTERM handler: {}", e);
            tokio::signal::ctrl_c().await.ok();
            return;
        }
    };
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = sigterm.recv() => {}
    }
}

#[cfg(not(unix))]
async fn wait_for_termination_signal() {
    if let Err(e) = tokio::signal::ctrl_c().await {
        tracing::error!("Failed to listen for Ctrl+C: {}", e);
    }
}

async fn wait_for_shutdown_signal(
    cancel_token: CancellationToken,
    tx_tt_cmd: std_mpsc::Sender<crate::core::types::TtCommand>,
) {
    wait_for_termination_signal().await;
    let _ = tx_tt_cmd.send(crate::core::types::TtCommand::Shutdown);
    cancel_token.cancel();
}
