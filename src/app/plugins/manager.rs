use super::runtime::{
    PluginAction, PluginManifest, PluginRuntime, event_envelope, normalized_tg_context,
    normalized_tt_context, should_disable,
};
use super::{PluginEvent, PluginInit, TgCommandContext, TtCommandContext, plugin_name_from_path};
use anyhow::Context;
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use teloxide::Bot;
use teloxide::prelude::Requester;
use teloxide::sugar::request::RequestReplyExt;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio_util::sync::CancellationToken;

pub struct PluginManagerHandle {
    inner: Arc<Mutex<PluginManager>>,
    root: PathBuf,
}

#[derive(Clone, Debug)]
pub struct PluginStatus {
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub forced_disabled: bool,
    pub command_calls: u64,
    pub event_calls: u64,
    pub failures: u64,
    pub timeout_hits: u64,
    pub last_error: Option<String>,
}

struct PluginEntry {
    name: String,
    version: String,
    runtime: PluginRuntime,
    errors: VecDeque<std::time::Instant>,
    enabled: bool,
    forced_disabled: bool,
    command_calls: u64,
    event_calls: u64,
    failures: u64,
    timeout_hits: u64,
    last_error: Option<String>,
}

struct PluginManager {
    plugins: HashMap<String, PluginEntry>,
    action_tx: UnboundedSender<PluginAction>,
    call_timeout: Duration,
    error_window: Duration,
    error_threshold: u32,
}

impl Clone for PluginManagerHandle {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            root: self.root.clone(),
        }
    }
}

impl PluginManagerHandle {
    pub async fn new(init: PluginInit) -> anyhow::Result<Self> {
        let plugins_cfg = &init.config.plugins;
        let root = plugins_cfg.dir.clone();
        if !plugins_cfg.enabled {
            let (action_tx, _action_rx) = unbounded_channel();
            return Ok(Self {
                inner: Arc::new(Mutex::new(PluginManager {
                    plugins: HashMap::new(),
                    action_tx,
                    call_timeout: Duration::from_millis(plugins_cfg.call_timeout_ms),
                    error_window: Duration::from_secs(plugins_cfg.error_window_seconds),
                    error_threshold: plugins_cfg.error_threshold,
                })),
                root,
            });
        }

        std::fs::create_dir_all(&root)
            .with_context(|| format!("failed to create plugins dir {}", root.display()))?;

        let (action_tx, action_rx) = unbounded_channel();
        tokio::spawn(process_actions(
            init.event_bot.clone(),
            init.tx_tt_cmd.clone(),
            action_rx,
            init.cancel_token.clone(),
        ));

        let handle = Self {
            inner: Arc::new(Mutex::new(PluginManager {
                plugins: HashMap::new(),
                action_tx,
                call_timeout: Duration::from_millis(plugins_cfg.call_timeout_ms),
                error_window: Duration::from_secs(plugins_cfg.error_window_seconds),
                error_threshold: plugins_cfg.error_threshold,
            })),
            root: root.clone(),
        };

        handle.reload_all(plugins_cfg.disabled.as_slice()).await;
        spawn_metrics_logger(handle.clone(), init.cancel_token.clone());
        if plugins_cfg.auto_reload {
            super::watcher::spawn_watcher(
                root.as_path(),
                handle.clone(),
                init.cancel_token,
                plugins_cfg.disabled.clone(),
            )?;
        }
        Ok(handle)
    }

    pub async fn statuses(&self) -> Vec<PluginStatus> {
        let mut statuses = {
            let manager = self.inner.lock().await;
            manager
                .plugins
                .values()
                .map(|plugin| PluginStatus {
                    name: plugin.name.clone(),
                    version: plugin.version.clone(),
                    enabled: plugin.enabled,
                    forced_disabled: plugin.forced_disabled,
                    command_calls: plugin.command_calls,
                    event_calls: plugin.event_calls,
                    failures: plugin.failures,
                    timeout_hits: plugin.timeout_hits,
                    last_error: plugin.last_error.clone(),
                })
                .collect::<Vec<_>>()
        };
        statuses.sort_by(|a, b| a.name.cmp(&b.name));
        statuses
    }

    pub async fn status_text(&self) -> String {
        let statuses = self.statuses().await;
        if statuses.is_empty() {
            return "Plugins: none loaded".to_string();
        }
        let mut lines = Vec::with_capacity(statuses.len() + 1);
        lines.push("Plugins status:".to_string());
        for status in statuses {
            let state = if status.enabled {
                "enabled"
            } else {
                "disabled"
            };
            let forced = if status.forced_disabled {
                " (forced by config)"
            } else {
                ""
            };
            let last_error = status
                .last_error
                .as_deref()
                .unwrap_or("none")
                .replace('\n', " ");
            lines.push(format!(
                "- {} v{}: {}{} | cmd={} evt={} fail={} timeout={} | last_error={}",
                status.name,
                status.version,
                state,
                forced,
                status.command_calls,
                status.event_calls,
                status.failures,
                status.timeout_hits,
                last_error
            ));
        }
        lines.join("\n")
    }

    pub async fn set_enabled(&self, name: &str, enabled: bool) -> anyhow::Result<()> {
        let mut manager = self.inner.lock().await;
        let Some(entry) = manager.plugins.get_mut(name) else {
            anyhow::bail!("plugin not found: {name}");
        };
        if enabled && entry.forced_disabled {
            anyhow::bail!("plugin is forced disabled by config");
        }
        entry.enabled = enabled;
        drop(manager);
        Ok(())
    }

    pub async fn reload_named(&self, name: &str, disabled: &[String]) -> anyhow::Result<()> {
        self.reload_plugin(name, disabled).await;
        let manager = self.inner.lock().await;
        if manager.plugins.contains_key(name) {
            Ok(())
        } else {
            anyhow::bail!("plugin not found: {name}")
        }
    }

    pub async fn dispatch_tg_command(
        &self,
        command: &str,
        args: &[String],
        ctx: TgCommandContext,
    ) -> bool {
        let context = normalized_tg_context(&ctx);
        self.dispatch_command(command, args, &context).await
    }

    pub async fn dispatch_tt_command(
        &self,
        command: &str,
        args: &[String],
        ctx: TtCommandContext,
    ) -> bool {
        let context = normalized_tt_context(&ctx);
        self.dispatch_command(command, args, &context).await
    }

    async fn dispatch_command(&self, command: &str, args: &[String], context: &Value) -> bool {
        let mut manager = self.inner.lock().await;
        let error_window = manager.error_window;
        let error_threshold = manager.error_threshold;
        let mut handled = false;
        for (plugin_name, plugin) in &mut manager.plugins {
            if !plugin.enabled {
                continue;
            }
            plugin.command_calls = plugin.command_calls.saturating_add(1);
            match plugin.runtime.dispatch_command(command, args, context) {
                Ok(true) => handled = true,
                Ok(false) => {}
                Err(error) => {
                    plugin.failures = plugin.failures.saturating_add(1);
                    if error.to_string().contains("timeout") {
                        plugin.timeout_hits = plugin.timeout_hits.saturating_add(1);
                    }
                    plugin.last_error = Some(error.to_string());
                    tracing::error!(plugin = %plugin_name, error = %error, "Plugin command failed");
                    if should_disable(&mut plugin.errors, error_window, error_threshold) {
                        plugin.enabled = false;
                        tracing::error!(
                            plugin = %plugin_name,
                            "Plugin disabled after runtime error threshold"
                        );
                    }
                }
            }
        }
        drop(manager);
        handled
    }

    pub async fn dispatch_event(&self, event: PluginEvent) {
        let event_value = event_envelope(&event.name, &event.source, &event.normalized, &event.raw);
        let mut manager = self.inner.lock().await;
        let error_window = manager.error_window;
        let error_threshold = manager.error_threshold;
        for (plugin_name, plugin) in &mut manager.plugins {
            if !plugin.enabled {
                continue;
            }
            plugin.event_calls = plugin.event_calls.saturating_add(1);
            if let Err(error) = plugin.runtime.dispatch_event(&event_value) {
                plugin.failures = plugin.failures.saturating_add(1);
                if error.to_string().contains("timeout") {
                    plugin.timeout_hits = plugin.timeout_hits.saturating_add(1);
                }
                plugin.last_error = Some(error.to_string());
                tracing::error!(plugin = %plugin_name, error = %error, "Plugin event dispatch failed");
                if should_disable(&mut plugin.errors, error_window, error_threshold) {
                    plugin.enabled = false;
                    tracing::error!(
                        plugin = %plugin_name,
                        "Plugin disabled after runtime error threshold"
                    );
                }
            }
        }
    }

    pub async fn reload_changed(&self, path: &Path, disabled: &[String]) {
        let Some(name) = plugin_name_from_path(&self.root, path) else {
            self.reload_all(disabled).await;
            return;
        };
        self.reload_plugin(&name, disabled).await;
    }

    pub async fn reload_all(&self, disabled: &[String]) {
        let entries = match std::fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(error) => {
                tracing::error!(error = %error, root = %self.root.display(), "Failed to read plugins directory");
                return;
            }
        };
        for entry in entries.flatten() {
            if !entry.file_type().map(|f| f.is_dir()).unwrap_or(false) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            self.reload_plugin(&name, disabled).await;
        }
    }

    async fn reload_plugin(&self, name: &str, disabled: &[String]) {
        let plugin_dir = self.root.join(name);
        let manifest_path = plugin_dir.join("plugin.toml");
        if !manifest_path.exists() {
            return;
        }
        let Some(manifest) = std::fs::read_to_string(&manifest_path)
            .ok()
            .and_then(|raw| toml::from_str::<PluginManifest>(&raw).ok())
        else {
            tracing::error!(plugin = %name, "Failed to parse plugin manifest");
            return;
        };
        let mut manager = self.inner.lock().await;
        let call_timeout = manager.call_timeout;
        let action_tx = manager.action_tx.clone();

        let runtime = match PluginRuntime::load(&plugin_dir, &manifest, action_tx, call_timeout) {
            Ok(runtime) => runtime,
            Err(error) => {
                tracing::error!(
                    plugin = %manifest.name,
                    error = %error,
                    "Plugin load failed, previous version remains active"
                );
                return;
            }
        };

        let enabled = manifest.enabled && !disabled.iter().any(|item| item == &manifest.name);
        let forced_disabled = disabled.iter().any(|item| item == &manifest.name);
        tracing::info!(
            plugin = %manifest.name,
            version = %manifest.version,
            enabled,
            "Plugin loaded"
        );
        let old_errors = manager
            .plugins
            .get(&manifest.name)
            .map(|entry| entry.errors.clone())
            .unwrap_or_default();
        manager.plugins.insert(
            manifest.name.clone(),
            PluginEntry {
                name: manifest.name.clone(),
                version: manifest.version,
                runtime,
                errors: old_errors,
                enabled,
                forced_disabled,
                command_calls: 0,
                event_calls: 0,
                failures: 0,
                timeout_hits: 0,
                last_error: None,
            },
        );
    }
}

fn spawn_metrics_logger(plugins: PluginManagerHandle, cancel_token: CancellationToken) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        loop {
            tokio::select! {
                () = cancel_token.cancelled() => break,
                _ = interval.tick() => {
                    let statuses = plugins.statuses().await;
                    for status in statuses {
                        tracing::info!(
                            target: "plugin",
                            plugin = %status.name,
                            version = %status.version,
                            enabled = status.enabled,
                            forced_disabled = status.forced_disabled,
                            command_calls = status.command_calls,
                            event_calls = status.event_calls,
                            failures = status.failures,
                            timeout_hits = status.timeout_hits,
                            "Plugin metrics snapshot"
                        );
                    }
                }
            }
        }
    });
}

async fn process_actions(
    event_bot: Option<Bot>,
    tx_tt_cmd: tokio::sync::mpsc::Sender<crate::core::types::TtCommand>,
    mut action_rx: UnboundedReceiver<PluginAction>,
    cancel_token: CancellationToken,
) {
    loop {
        tokio::select! {
            () = cancel_token.cancelled() => break,
            action = action_rx.recv() => {
                let Some(action) = action else {
                    break;
                };
                match action {
                    PluginAction::TgSend { chat_id, text, reply_to } => {
                        if let Some(bot) = &event_bot {
                            if let Some(reply_to) = reply_to {
                                let _ = bot.send_message(teloxide::types::ChatId(chat_id), text)
                                    .reply_to(teloxide::types::MessageId(reply_to))
                                    .await;
                            } else {
                                let _ = bot.send_message(teloxide::types::ChatId(chat_id), text).await;
                            }
                        }
                    }
                    PluginAction::Tt(command) => {
                        let _ = tx_tt_cmd.send(command).await;
                    }
                }
            }
        }
    }
}
