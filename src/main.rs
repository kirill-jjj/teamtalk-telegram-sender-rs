use self_update::cargo_crate_version;

mod adapters;
mod app;
mod bootstrap;
mod core;
mod infra;

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc as std_mpsc;
use std::sync::{Arc, RwLock};
use teamtalk::types::UserAccount;
use teloxide::{Bot, prelude::Requester};
use tokio::sync::mpsc as tokio_mpsc;
use tokio::sync::watch;
use tokio::time::Duration;
use tracing_subscriber::EnvFilter;

fn update_bot() -> Result<(), Box<dyn std::error::Error>> {
    let target = if cfg!(windows) { "windows" } else { "linux" };

    let status = self_update::backends::github::Update::configure()
        .repo_owner("kirill-jjj")
        .repo_name("teamtalk-telegram-sender-rs")
        .bin_name("teamtalk-telegram-sender-rs")
        .target(target)
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;

    println!("Update status: `{}`!", status.version());
    Ok(())
}

async fn wait_for_shutdown_signal(
    shutdown_tx: watch::Sender<bool>,
    tx_tt_cmd: std_mpsc::Sender<crate::core::types::TtCommand>,
) {
    wait_for_termination_signal().await;
    crate::core::shutdown::request_shutdown(&shutdown_tx, &tx_tt_cmd);
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

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--update".to_string()) {
        println!("Checking for updates...");
        if let Err(e) = update_bot() {
            eprintln!("Update failed: {}", e);
            std::process::exit(1);
        }
        println!("Update completed successfully! Please restart the bot.");
        std::process::exit(0);
    }

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    tracing::info!("🚀 Starting Application...");

    let args: Vec<String> = std::env::args().collect();
    let config_path = if let Some(idx) = args.iter().position(|a| a == "--config") {
        args.get(idx + 1)
            .cloned()
            .unwrap_or_else(|| "config.toml".to_string())
    } else {
        "config.toml".to_string()
    };

    tracing::info!("📂 Loading config from: {}", config_path);

    let config_content = std::fs::read_to_string(&config_path)?;
    let mut config: bootstrap::config::Config = toml::from_str(&config_content)?;

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
    tracing::info!("💾 Database path: {}", db_path_str);

    config.database.db_file = db_path_str.clone();

    let shared_config = Arc::new(config);

    let db = infra::db::Database::new(&db_path_str).await?;
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let db_for_cleanup = db.clone();
    let cleanup_interval = shared_config
        .operational_parameters
        .deeplink_cleanup_interval_seconds;
    {
        let mut shutdown = shutdown_rx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(cleanup_interval));
            loop {
                tokio::select! {
                    _ = shutdown.changed() => break,
                    _ = interval.tick() => {}
                }
                match db_for_cleanup.cleanup_expired_deeplinks().await {
                    Ok(count) if count > 0 => {
                        tracing::info!("🧹 Cleaned up {} expired deeplinks.", count);
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
        let mut shutdown = shutdown_rx.clone();
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(Duration::from_secs(pending_cleanup_interval_seconds));
            loop {
                tokio::select! {
                    _ = shutdown.changed() => break,
                    _ = interval.tick() => {}
                }
                match db_for_pending_cleanup
                    .cleanup_pending_replies(pending_ttl_seconds)
                    .await
                {
                    Ok(count) if count > 0 => {
                        tracing::info!("🧹 Cleaned up {} pending replies.", count);
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
                        tracing::info!("🧹 Cleaned up {} pending channel replies.", count);
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("Failed to clean up pending channel replies: {}", e);
                    }
                }
            }
        });
    }

    let online_users: Arc<RwLock<HashMap<i32, core::types::LiteUser>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let online_users_by_username: Arc<RwLock<HashMap<String, i32>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let all_user_accounts: Arc<RwLock<HashMap<String, UserAccount>>> =
        Arc::new(RwLock::new(HashMap::new()));

    let (tx_bridge, rx_bridge) = tokio_mpsc::channel::<crate::core::types::BridgeEvent>(100);
    let (tx_tt_cmd, rx_tt_cmd) = std_mpsc::channel::<crate::core::types::TtCommand>();

    let event_token = shared_config.telegram.event_token.clone();
    let message_token = shared_config.telegram.message_token.clone();
    let same_token = event_token.is_some() && message_token == event_token;

    let event_bot = if let Some(token) = &event_token {
        Some(Bot::new(token))
    } else {
        tracing::warn!(
            "⚠️ 'event_token' missing. Telegram interactions and notifications disabled."
        );
        None
    };

    let message_bot = if same_token {
        tracing::info!("message_token matches event_token; using event bot for admin messages.");
        None
    } else if let Some(token) = &message_token {
        Some(Bot::new(token))
    } else {
        tracing::warn!("⚠️ 'message_token' missing. Admin alerts disabled.");
        None
    };

    let bot_username = if let Some(bot) = &event_bot {
        let me = bot.get_me().await?;
        let username = me
            .username
            .clone()
            .ok_or_else(|| anyhow!("Bot must have a username!"))?;
        tracing::info!("✅ Interaction Bot username: @{}", username);
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

    let config_for_worker = shared_config.clone();
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
        Ok(Ok(_)) => tracing::info!("✅ TeamTalk Worker started successfully"),
        Ok(Err(e)) => return Err(anyhow!("❌ TeamTalk Worker failed to start: {}", e)),
        Err(_) => return Err(anyhow!("❌ TeamTalk Worker disconnected during startup")),
    }

    let event_bot_clone = event_bot.clone();
    let msg_bot_clone = message_bot.clone();
    let db_clone = db.clone();
    let online_users_clone = online_users.clone();
    let message_token_present = message_token.is_some();
    let shutdown_for_bridge = shutdown_rx.clone();

    let bridge_handle = tokio::spawn(adapters::bridge::run_bridge(
        adapters::bridge::BridgeContext {
            db: db_clone,
            online_users: online_users_clone,
            config: shared_config.clone(),
            event_bot: event_bot_clone,
            msg_bot: msg_bot_clone,
            message_token_present,
            tx_tt_cmd: tx_tt_cmd.clone(),
            shutdown: shutdown_for_bridge,
        },
        rx_bridge,
    ));

    let shutdown_signal = shutdown_tx.clone();
    let tx_tt_cmd_for_shutdown = tx_tt_cmd.clone();
    tokio::spawn(async move {
        wait_for_shutdown_signal(shutdown_signal, tx_tt_cmd_for_shutdown).await;
    });

    if let Some(bot) = event_bot {
        adapters::tg::run_tg_bot(adapters::tg::TgRunArgs {
            event_bot: bot,
            message_bot,
            db: db.clone(),
            online_users,
            user_accounts: all_user_accounts,
            tx_tt_cmd,
            config: shared_config,
            shutdown: shutdown_rx.clone(),
            shutdown_tx: shutdown_tx.clone(),
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
