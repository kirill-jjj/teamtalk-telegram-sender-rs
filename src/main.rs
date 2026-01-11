use self_update::cargo_crate_version;

mod bridge;
mod config;
mod db;
mod locales;
mod tg_bot;
mod tt_worker;
mod types;

use anyhow::{Result, anyhow};
use dashmap::DashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc as std_mpsc;
use teamtalk::types::UserAccount;
use teloxide::{Bot, prelude::Requester};
use tokio::sync::mpsc as tokio_mpsc;
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
    let mut config: config::Config = toml::from_str(&config_content)?;

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

    let db = db::Database::new(&db_path_str).await?;
    let db_for_cleanup = db.clone();
    let cleanup_interval = shared_config
        .operational_parameters
        .deeplink_cleanup_interval_seconds;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(cleanup_interval));
        loop {
            interval.tick().await;
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

    let online_users: Arc<DashMap<i32, types::LiteUser>> = Arc::new(DashMap::new());
    let online_users_by_username: Arc<DashMap<String, i32>> = Arc::new(DashMap::new());
    let all_user_accounts: Arc<DashMap<String, UserAccount>> = Arc::new(DashMap::new());

    let (tx_bridge, rx_bridge) = tokio_mpsc::channel::<crate::types::BridgeEvent>(100);
    let (tx_tt_cmd, rx_tt_cmd) = std_mpsc::channel::<crate::types::TtCommand>();

    let event_bot = if let Some(token) = &shared_config.telegram.event_token {
        Some(Bot::new(token))
    } else {
        tracing::warn!(
            "⚠️ 'event_token' missing. Telegram interactions and notifications disabled."
        );
        None
    };

    let message_bot = if let Some(token) = &shared_config.telegram.message_token {
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

    std::thread::spawn(move || {
        tt_worker::run_teamtalk_thread(tt_worker::RunTeamtalkArgs {
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
    let users_by_username_clone = online_users_by_username.clone();

    let bridge_handle = tokio::spawn(bridge::run_bridge(
        bridge::BridgeContext {
            db: db_clone,
            online_users_by_username: users_by_username_clone,
            config: shared_config.clone(),
            event_bot: event_bot_clone,
            msg_bot: msg_bot_clone,
            tx_tt_cmd: tx_tt_cmd.clone(),
        },
        rx_bridge,
    ));

    if let Some(bot) = event_bot {
        tg_bot::run_tg_bot(
            bot,
            db,
            online_users,
            all_user_accounts,
            tx_tt_cmd,
            shared_config,
        )
        .await;
    } else if let Err(e) = bridge_handle.await {
        tracing::error!("Bridge task failed: {}", e);
    }

    Ok(())
}
