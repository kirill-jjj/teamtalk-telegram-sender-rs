//! `TeamTalk` 5 to Telegram bridge bot.

use self_update::cargo_crate_version;

mod adapters;
mod app;
mod bootstrap;
mod core;
mod infra;

use anyhow::Result;
use tokio::task::{JoinSet, LocalSet};
use tracing::Instrument;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

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
#[allow(clippy::large_futures)]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--update") {
        update_bot()?;
        return Ok(());
    }

    let config_paths = bootstrap::cli::collect_config_paths(&args)?;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let cancel_token = tokio_util::sync::CancellationToken::new();

    let local = LocalSet::new();
    local
        .run_until(async move {
            let mut set = JoinSet::new();
            for config_path in config_paths {
                let instance_name = bootstrap::cli::instance_name_from_path(&config_path);
                let level = bootstrap::cli::read_log_level(&config_path)
                    .unwrap_or_else(|| "info".to_string());
                let dispatch = build_dispatch(&level);
                let token = cancel_token.clone();
                set.spawn_local(async move {
                    let _guard = tracing::dispatcher::set_default(&dispatch);
                    let span = tracing::info_span!(
                        "instance",
                        instance = %instance_name,
                        config = %config_path
                    );
                    async move {
                        tracing::info!(component = "main", "Starting application");
                        let app = bootstrap::app::Application::build(std::path::PathBuf::from(
                            &config_path,
                        ))
                        .await?;
                        app.run(token).await
                    }
                    .instrument(span)
                    .await
                });
            }

            let mut first_err: Option<anyhow::Error> = None;
            while let Some(res) = set.join_next().await {
                match res {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        if first_err.is_none() {
                            first_err = Some(e);
                            cancel_token.cancel();
                        }
                    }
                    Err(e) => {
                        if first_err.is_none() {
                            first_err = Some(anyhow::anyhow!(e));
                            cancel_token.cancel();
                        }
                    }
                }
            }
            if let Some(err) = first_err {
                return Err(err);
            }
            Ok(())
        })
        .await?;

    Ok(())
}

fn build_dispatch(level: &str) -> tracing::Dispatch {
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::new(level))
        .with(tracing_subscriber::fmt::layer());
    tracing::Dispatch::new(subscriber)
}
