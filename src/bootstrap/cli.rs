use crate::bootstrap::config::Config;
use anyhow::Result;
use clap::Parser;
use std::path::Path;

#[derive(Debug, Parser)]
#[command(name = "teamtalk-telegram-sender-rs")]
struct CliArgs {
    #[arg(short = 'c', long = "config", value_name = "PATH", action = clap::ArgAction::Append)]
    config: Vec<String>,
}

pub fn collect_config_paths(args: &[String]) -> Result<Vec<String>> {
    let cli = CliArgs::try_parse_from(args)?;
    if cli.config.is_empty() {
        return Ok(vec!["config.toml".to_string()]);
    }
    Ok(cli.config)
}

pub fn instance_name_from_path(path: &str) -> String {
    let path_obj = Path::new(path);
    path_obj
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .map_or_else(|| path.to_string(), ToString::to_string)
}

pub fn read_log_level(config_path: &str) -> Option<String> {
    let content = match std::fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(err) => {
            tracing::debug!(
                path = %config_path,
                error = %err,
                "Failed to read config while probing log level"
            );
            return None;
        }
    };
    let config: Config = match toml::from_str(&content) {
        Ok(config) => config,
        Err(err) => {
            tracing::debug!(
                path = %config_path,
                error = %err,
                "Failed to parse config while probing log level"
            );
            return None;
        }
    };
    Some(config.general.log_level.as_str().to_string())
}

#[cfg(test)]
#[path = "../../tests/unit/bootstrap_cli.rs"]
mod tests;
