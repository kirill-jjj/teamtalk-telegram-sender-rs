use crate::adapters;
use crate::bootstrap::config::Config;
use crate::bootstrap::config_errors::load_config;
use crate::core::types::TtUsername;
use crate::infra::db::Database;
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use teamtalk::Client;
use teloxide::{Bot, prelude::Requester};
use tokio::sync::mpsc as tokio_mpsc;
use tokio::sync::oneshot;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;

pub struct Application {
    config: Arc<Config>,
    db: Database,
}

struct BotInit {
    event_bot: Option<Bot>,
    message_bot: Option<Bot>,
    bot_username: Option<TtUsername>,
    message_token_present: bool,
}

struct SharedState {
    state: crate::app::state::StateHandle,
}

struct TeamtalkWorkerConfig {
    config: Arc<Config>,
    state: crate::app::state::StateHandle,
    tx_bridge: tokio_mpsc::Sender<crate::core::types::BridgeEvent>,
    rx_tt_cmd: tokio_mpsc::Receiver<crate::core::types::TtCommand>,
    tx_tt_cmd: tokio_mpsc::Sender<crate::core::types::TtCommand>,
    db: Database,
    bot_username: Option<TtUsername>,
    plugins: crate::app::plugins::PluginManagerHandle,
    client: Client,
}

struct TelegramRunContext {
    event_bot: Option<Bot>,
    message_bot: Option<Bot>,
    db: Database,
    shared: SharedState,
    tx_tt_cmd: tokio_mpsc::Sender<crate::core::types::TtCommand>,
    plugins: crate::app::plugins::PluginManagerHandle,
    config: Arc<Config>,
    cancel_token: CancellationToken,
    bridge_handle: tokio::task::JoinHandle<()>,
    tt_handle: tokio::task::JoinHandle<()>,
}

impl Application {
    pub async fn build(config_path: PathBuf) -> Result<Self> {
        tracing::info!(path = %config_path.display(), "Loading config");

        let mut config: Config = load_config(&config_path)?;

        let config_dir = config_path.parent().unwrap_or_else(|| Path::new("."));

        let db_path_buf = if config.database.db_file.is_absolute() {
            config.database.db_file.clone()
        } else {
            config_dir.join(&config.database.db_file)
        };

        let db_path_str = db_path_buf.to_string_lossy().to_string();
        tracing::info!(db_path = %db_path_str, "Database path");

        config.database.db_file = db_path_buf;

        let config = Arc::new(config);
        let db = Database::new(&db_path_str).await?;
        Ok(Self { config, db })
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<()> {
        let Self { config, db } = self;

        spawn_deeplink_cleanup_task(
            db.clone(),
            config.operational_parameters.deeplink_cleanup_interval,
            cancel_token.clone(),
        );
        spawn_pending_cleanup_task(db.clone(), 3600, 3600, cancel_token.clone());

        let client = tokio::task::spawn_blocking(Client::new)
            .await
            .map_err(|e| anyhow!("Failed to join TeamTalk SDK init task: {e}"))?
            .map_err(|e| anyhow!("Failed to initialize TeamTalk SDK: {e}"))?;
        let shared = init_shared_state();
        let (tx_bridge, rx_bridge) = tokio_mpsc::channel::<crate::core::types::BridgeEvent>(100);
        let (tx_tt_cmd, rx_tt_cmd) = tokio_mpsc::channel::<crate::core::types::TtCommand>(256);

        let bots = init_bots(&config).await?;
        if let Some(bot) = bots.event_bot.clone() {
            crate::adapters::tg::subscriber_notify::spawn_subscriber_notify_retry_worker(
                bot,
                db.clone(),
                config
                    .operational_parameters
                    .subscriber_notify_retry_interval,
                config
                    .operational_parameters
                    .subscriber_notify_retry_backoff,
                config.operational_parameters.subscriber_notify_max_attempts,
                cancel_token.clone(),
            );
        }
        let plugins =
            crate::app::plugins::PluginManagerHandle::new(crate::app::plugins::PluginInit {
                config: config.clone(),
                tx_tt_cmd: tx_tt_cmd.clone(),
                event_bot: bots.event_bot.clone(),
                cancel_token: cancel_token.clone(),
            })
            .await?;
        let tt_handle = start_teamtalk_worker(TeamtalkWorkerConfig {
            config: config.clone(),
            state: shared.state.clone(),
            tx_bridge: tx_bridge.clone(),
            rx_tt_cmd,
            tx_tt_cmd: tx_tt_cmd.clone(),
            db: db.clone(),
            bot_username: bots.bot_username.clone(),
            plugins: plugins.clone(),
            client,
        })
        .await?;

        let bridge_handle = tokio::spawn(adapters::bridge::run_bridge(
            adapters::bridge::BridgeContext {
                db: db.clone(),
                state: shared.state.clone(),
                config: config.clone(),
                event_bot: bots.event_bot.clone(),
                msg_bot: bots.message_bot.clone(),
                message_token_present: bots.message_token_present,
                tx_tt_cmd: tx_tt_cmd.clone(),
                cancel_token: cancel_token.clone(),
            },
            rx_bridge,
        ));

        tokio::spawn(wait_for_cancel(cancel_token.clone(), tx_tt_cmd.clone()));

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
            plugins,
            config,
            cancel_token,
            bridge_handle,
            tt_handle,
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
    SharedState {
        state: crate::app::state::StateHandle::new(),
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
        Some(TtUsername::from(username))
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
    let tt_handle = tokio::task::spawn_blocking(move || {
        let TeamtalkWorkerConfig {
            config,
            state,
            tx_bridge,
            rx_tt_cmd,
            tx_tt_cmd,
            db,
            bot_username,
            plugins,
            client,
        } = cfg;
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(e) => {
                let _ = tx_init.send(Err(format!("Failed to create TeamTalk runtime: {e}")));
                return;
            }
        };
        let local = tokio::task::LocalSet::new();
        local.block_on(&runtime, async move {
            adapters::tt::run_teamtalk_worker(adapters::tt::RunTeamtalkArgs {
                config,
                state,
                tx_bridge,
                rx_cmd: rx_tt_cmd,
                tx_cmd_clone: tx_tt_cmd,
                db,
                bot_username,
                plugins,
                client,
                tx_init,
            })
            .await;
        });
    });

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
            state: ctx.shared.state,
            tx_tt_cmd: ctx.tx_tt_cmd,
            plugins: ctx.plugins,
            config: ctx.config,
            cancel_token: ctx.cancel_token,
        })
        .await;
        if let Err(e) = ctx.bridge_handle.await {
            tracing::error!(error = %e, "Bridge task failed");
        }
        if let Err(e) = ctx.tt_handle.await {
            tracing::error!(error = %e, "TeamTalk worker task failed");
        }
    } else if let Err(e) = ctx.bridge_handle.await {
        tracing::error!(error = %e, "Bridge task failed");
        if let Err(e) = ctx.tt_handle.await {
            tracing::error!(error = %e, "TeamTalk worker task failed");
        }
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
            if let Err(err) = tokio::signal::ctrl_c().await {
                tracing::error!(error = %err, "Failed to listen for Ctrl+C");
            }
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
    if let Err(err) = tx_tt_cmd
        .send(crate::core::types::TtCommand::Shutdown)
        .await
    {
        tracing::error!(error = %err, "Failed to send shutdown command");
    }
    cancel_token.cancel();
}

async fn wait_for_cancel(
    cancel_token: CancellationToken,
    tx_tt_cmd: tokio_mpsc::Sender<crate::core::types::TtCommand>,
) {
    cancel_token.cancelled().await;
    if let Err(err) = tx_tt_cmd
        .send(crate::core::types::TtCommand::Shutdown)
        .await
    {
        tracing::error!(error = %err, "Failed to send shutdown command on cancel");
    }
}
