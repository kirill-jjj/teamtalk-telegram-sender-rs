use crate::adapters;
use crate::bootstrap::config::Config;
use crate::infra::db::Database;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use teamtalk::types::UserAccount;
use teloxide::{Bot, prelude::Requester};
use tokio::sync::mpsc as tokio_mpsc;
use tokio::sync::oneshot;
use tokio::task::{LocalSet, spawn_local};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

pub struct Application {
    config: Arc<Config>,
    db: Database,
    cancel_token: CancellationToken,
}

struct BotInit {
    event_bot: Option<Bot>,
    message_bot: Option<Bot>,
    bot_username: Option<String>,
    message_token_present: bool,
}

struct SharedState {
    online_users: Arc<RwLock<HashMap<i32, crate::core::types::LiteUser>>>,
    online_users_by_username: Arc<RwLock<HashMap<String, i32>>>,
    all_user_accounts: Arc<RwLock<HashMap<String, UserAccount>>>,
}

struct TeamtalkWorkerConfig {
    config: Arc<Config>,
    online_users: Arc<RwLock<HashMap<i32, crate::core::types::LiteUser>>>,
    online_users_by_username: Arc<RwLock<HashMap<String, i32>>>,
    user_accounts: Arc<RwLock<HashMap<String, UserAccount>>>,
    tx_bridge: tokio_mpsc::Sender<crate::core::types::BridgeEvent>,
    rx_tt_cmd: tokio_mpsc::Receiver<crate::core::types::TtCommand>,
    tx_tt_cmd: tokio_mpsc::Sender<crate::core::types::TtCommand>,
    db: Database,
    bot_username: Option<String>,
}

struct TelegramRunContext {
    event_bot: Option<Bot>,
    message_bot: Option<Bot>,
    db: Database,
    shared: SharedState,
    tx_tt_cmd: tokio_mpsc::Sender<crate::core::types::TtCommand>,
    config: Arc<Config>,
    cancel_token: CancellationToken,
    bridge_handle: tokio::task::JoinHandle<()>,
    tt_handle: tokio::task::JoinHandle<()>,
}

impl Application {
    pub async fn build(config_path: String) -> Result<Self> {
        tracing::info!(path = %config_path, "Loading config");

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
        tracing::info!(db_path = %db_path_str, "Database path");

        config.database.db_file.clone_from(&db_path_str);

        let config = Arc::new(config);
        let db = Database::new(&db_path_str).await?;
        let cancel_token = CancellationToken::new();

        Ok(Self {
            config,
            db,
            cancel_token,
        })
    }

    #[allow(clippy::future_not_send, clippy::large_futures)]
    pub async fn run(self) -> Result<()> {
        let Self {
            config,
            db,
            cancel_token,
        } = self;

        spawn_deeplink_cleanup_task(
            db.clone(),
            config.operational_parameters.deeplink_cleanup_interval,
            cancel_token.clone(),
        );
        spawn_pending_cleanup_task(db.clone(), 3600, 3600, cancel_token.clone());

        let local = LocalSet::new();
        local
            .run_until(async move {
                let shared = init_shared_state();
                let (tx_bridge, rx_bridge) =
                    tokio_mpsc::channel::<crate::core::types::BridgeEvent>(100);
                let (tx_tt_cmd, rx_tt_cmd) =
                    tokio_mpsc::channel::<crate::core::types::TtCommand>(256);

                let bots = init_bots(&config).await?;
                let tt_handle = start_teamtalk_worker(TeamtalkWorkerConfig {
                    config: config.clone(),
                    online_users: shared.online_users.clone(),
                    online_users_by_username: shared.online_users_by_username.clone(),
                    user_accounts: shared.all_user_accounts.clone(),
                    tx_bridge: tx_bridge.clone(),
                    rx_tt_cmd,
                    tx_tt_cmd: tx_tt_cmd.clone(),
                    db: db.clone(),
                    bot_username: bots.bot_username.clone(),
                })
                .await?;

                let bridge_handle = tokio::spawn(adapters::bridge::run_bridge(
                    adapters::bridge::BridgeContext {
                        db: db.clone(),
                        online_users: shared.online_users.clone(),
                        config: config.clone(),
                        event_bot: bots.event_bot.clone(),
                        msg_bot: bots.message_bot.clone(),
                        message_token_present: bots.message_token_present,
                        tx_tt_cmd: tx_tt_cmd.clone(),
                        cancel_token: cancel_token.clone(),
                    },
                    rx_bridge,
                ));

                tokio::spawn(wait_for_shutdown_signal(
                    cancel_token.clone(),
                    tx_tt_cmd.clone(),
                ));

                run_telegram_or_wait(TelegramRunContext {
                    event_bot: bots.event_bot,
                    message_bot: bots.message_bot,
                    db,
                    shared,
                    tx_tt_cmd,
                    config,
                    cancel_token,
                    bridge_handle,
                    tt_handle,
                })
                .await?;

                Ok::<(), anyhow::Error>(())
            })
            .await?;

        Ok(())
    }
}

fn spawn_deeplink_cleanup_task(
    db: Database,
    cleanup_interval: u64,
    cancel_token: CancellationToken,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(cleanup_interval));
        loop {
            tokio::select! {
                () = cancel_token.cancelled() => break,
                _ = interval.tick() => {}
            }
            match db.cleanup_expired_deeplinks().await {
                Ok(count) if count > 0 => {
                    tracing::info!(count, "Cleaned up expired deeplinks");
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::error!(error = %e, "Failed to clean up expired deeplinks");
                }
            }
        }
    });
}

fn spawn_pending_cleanup_task(
    db: Database,
    cleanup_interval: u64,
    ttl_seconds: i64,
    cancel_token: CancellationToken,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(cleanup_interval));
        loop {
            tokio::select! {
                () = cancel_token.cancelled() => break,
                _ = interval.tick() => {}
            }
            match db.cleanup_pending_replies(ttl_seconds).await {
                Ok(count) if count > 0 => {
                    tracing::info!(count, "Cleaned up pending replies");
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::error!(error = %e, "Failed to clean up pending replies");
                }
            }
            match db.cleanup_pending_channel_replies(ttl_seconds).await {
                Ok(count) if count > 0 => {
                    tracing::info!(count, "Cleaned up pending channel replies");
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        "Failed to clean up pending channel replies"
                    );
                }
            }
        }
    });
}

fn init_shared_state() -> SharedState {
    let online_users: Arc<RwLock<HashMap<i32, crate::core::types::LiteUser>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let online_users_by_username: Arc<RwLock<HashMap<String, i32>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let all_user_accounts: Arc<RwLock<HashMap<String, UserAccount>>> =
        Arc::new(RwLock::new(HashMap::new()));

    SharedState {
        online_users,
        online_users_by_username,
        all_user_accounts,
    }
}

async fn init_bots(config: &Arc<Config>) -> Result<BotInit> {
    let event_token = config.telegram.event_token.clone();
    let message_token = config.telegram.message_token.clone();
    let same_token = event_token.is_some() && message_token == event_token;

    let event_bot = event_token.as_ref().map_or_else(
        || {
            tracing::warn!(
                config_key = "event_token",
                "Telegram interactions and notifications disabled"
            );
            None
        },
        |token| Some(Bot::new(token)),
    );

    let message_bot = if same_token {
        tracing::info!("message_token matches event_token; using event bot for admin messages");
        None
    } else if let Some(token) = &message_token {
        Some(Bot::new(token))
    } else {
        tracing::warn!(config_key = "message_token", "Admin alerts disabled");
        None
    };

    let bot_username = if let Some(bot) = &event_bot {
        let me = bot.get_me().await?;
        let username = me
            .username
            .clone()
            .ok_or_else(|| anyhow!("Bot must have a username!"))?;
        tracing::info!(username = %username, "Interaction bot username");
        Some(username)
    } else {
        None
    };

    Ok(BotInit {
        event_bot,
        message_bot,
        bot_username,
        message_token_present: message_token.is_some(),
    })
}

async fn start_teamtalk_worker(cfg: TeamtalkWorkerConfig) -> Result<tokio::task::JoinHandle<()>> {
    let (tx_init, rx_init) = oneshot::channel();
    let tt_handle = spawn_local(adapters::tt::run_teamtalk_worker(
        adapters::tt::RunTeamtalkArgs {
            config: cfg.config,
            online_users: cfg.online_users,
            online_users_by_username: cfg.online_users_by_username,
            user_accounts: cfg.user_accounts,
            tx_bridge: cfg.tx_bridge,
            rx_cmd: cfg.rx_tt_cmd,
            tx_cmd_clone: cfg.tx_tt_cmd,
            db: cfg.db,
            bot_username: cfg.bot_username,
            tx_init,
        },
    ));

    match rx_init.await {
        Ok(Ok(())) => tracing::info!("TeamTalk worker started successfully"),
        Ok(Err(e)) => return Err(anyhow!("TeamTalk worker failed to start: {e}")),
        Err(_) => return Err(anyhow!("TeamTalk worker disconnected during startup")),
    }

    Ok(tt_handle)
}

async fn run_telegram_or_wait(ctx: TelegramRunContext) -> Result<()> {
    if let Some(bot) = ctx.event_bot {
        adapters::tg::run_tg_bot(adapters::tg::TgRunArgs {
            event_bot: bot,
            message_bot: ctx.message_bot,
            db: ctx.db.clone(),
            online_users: ctx.shared.online_users,
            user_accounts: ctx.shared.all_user_accounts,
            tx_tt_cmd: ctx.tx_tt_cmd,
            config: ctx.config,
            cancel_token: ctx.cancel_token,
        })
        .await;
        let _ = ctx.bridge_handle.await;
        let _ = ctx.tt_handle.await;
    } else if let Err(e) = ctx.bridge_handle.await {
        tracing::error!(error = %e, "Bridge task failed");
        let _ = ctx.tt_handle.await;
    }

    tracing::info!(component = "shutdown", "Closing database pool");
    ctx.db.close().await;
    tracing::info!(component = "shutdown", "Database pool closed");

    Ok(())
}

#[cfg(unix)]
async fn wait_for_termination_signal() {
    use tokio::signal::unix::{SignalKind, signal};

    let mut sigterm = match signal(SignalKind::terminate()) {
        Ok(sigterm) => sigterm,
        Err(e) => {
            tracing::error!(error = %e, "Failed to register SIGTERM handler");
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
        tracing::error!(error = %e, "Failed to listen for Ctrl+C");
    }
}

async fn wait_for_shutdown_signal(
    cancel_token: CancellationToken,
    tx_tt_cmd: tokio_mpsc::Sender<crate::core::types::TtCommand>,
) {
    wait_for_termination_signal().await;
    let _ = tx_tt_cmd
        .send(crate::core::types::TtCommand::Shutdown)
        .await;
    cancel_token.cancel();
}
