use crate::bootstrap::config::Config;
use anyhow::{Result, anyhow};
use std::path::Path;

pub fn collect_config_paths(args: &[String]) -> Result<Vec<String>> {
    let mut configs = Vec::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--config" {
            let path = iter
                .next()
                .ok_or_else(|| anyhow!("Missing value for --config"))?;
            configs.push(path.clone());
        }
    }
    if configs.is_empty() {
        configs.push("config.toml".to_string());
    }
    Ok(configs)
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
    let content = std::fs::read_to_string(config_path).ok()?;
    let config: Config = toml::from_str(&content).ok()?;
    Some(config.general.log_level.as_str().to_string())
}

#[cfg(test)]
#[path = "../../tests/unit/bootstrap_cli.rs"]
mod tests;
