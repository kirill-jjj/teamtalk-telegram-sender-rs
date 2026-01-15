//! `TeamTalk` 5 to Telegram bridge bot.

use self_update::cargo_crate_version;

mod adapters;
mod app;
mod bootstrap;
mod core;
mod infra;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

fn update_bot() -> Result<()> {
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

    tracing::info!(version = %status.version(), "Update completed");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--update") {
        update_bot()?;
        return Ok(());
    }
    let config_path = args
        .iter()
        .position(|a| a == "--config")
        .and_then(|idx| args.get(idx + 1))
        .cloned()
        .unwrap_or_else(|| "config.toml".to_string());

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let level = read_log_level(&config_path).unwrap_or_else(|| "info".to_string());
        EnvFilter::new(level)
    });
    tracing_subscriber::fmt().with_env_filter(filter).init();
    tracing::info!(component = "main", "Starting application");

    let app = bootstrap::app::Application::build(config_path).await?;
    app.run().await?;

    Ok(())
}

fn read_log_level(config_path: &str) -> Option<String> {
    let content = std::fs::read_to_string(config_path).ok()?;
    let config: bootstrap::config::Config = toml::from_str(&content).ok()?;
    Some(config.general.log_level)
}
