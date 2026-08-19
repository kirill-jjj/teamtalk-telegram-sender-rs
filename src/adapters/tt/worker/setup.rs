use crate::adapters::tt::context::{RunTeamtalkArgs, WorkerContext};
use crate::adapters::tt::streaming;
use crate::adapters::tt::streaming::StreamItem;
use crate::app::services::tt_cache as tt_cache_service;
use crate::core::types::TtCommand;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use teamtalk::client::ReconnectConfig;
use teamtalk::client::media::MediaFilePlayback;
use teamtalk::events::Event;
use teamtalk::types::{ChannelId, UserStatus};
use teamtalk::{Client, LoginParams};
use tokio::sync::Semaphore;
use tokio::sync::mpsc::Sender;
use tokio::time::interval;

pub(super) fn spawn_cache_refresh(
    state: crate::app::state::StateHandle,
    services: crate::app::services::tt_context::TtServiceContext,
) {
    tokio::task::spawn_local(async move {
        let mut tick = interval(Duration::from_mins(5));
        loop {
            tick.tick().await;
            if !tt_cache_service::preload_all_ctx(&services).await {
                tracing::warn!(component = "tt_worker", "Failed to refresh TT user caches");
            }
            let cache_stats = match state.cache_stats().await {
                Ok(cache_stats) => cache_stats,
                Err(err) => {
                    tracing::error!(component = "tt_worker", error = %err, "Failed to read cache stats");
                    continue;
                }
            };
            tracing::info!(
                component = "tt_worker",
                lang_hits = cache_stats.lang_hits,
                lang_misses = cache_stats.lang_misses,
                tg_hits = cache_stats.tg_hits,
                tg_misses = cache_stats.tg_misses,
                "TT user cache stats"
            );
        }
    });
}

pub(super) fn spawn_afk_monitor(ctx: WorkerContext) {
    let configured = ctx.config.afk_notifications.poll_interval_seconds;
    let interval_seconds = configured.max(1);
    if configured == 0 {
        tracing::warn!(
            component = "tt_worker",
            configured_poll_interval_seconds = configured,
            used_poll_interval_seconds = interval_seconds,
            "AFK poll interval cannot be 0; falling back to 1 second"
        );
    }
    let poll_interval = Duration::from_secs(interval_seconds);
    tokio::task::spawn_local(async move {
        let mut tick = interval(poll_interval);
        loop {
            tick.tick().await;
            let snapshot = {
                let state = ctx.afk_state.lock().await;
                state
                    .iter()
                    .map(|(k, v)| (*k, v.clone()))
                    .collect::<Vec<_>>()
            };

            let now = std::time::Instant::now();
            let mut to_notify: Vec<(
                crate::core::types::TtUserId,
                crate::core::types::TelegramId,
                crate::core::types::TtNickname,
            )> = Vec::new();
            for (user_id, afk) in snapshot {
                if !afk.is_away {
                    continue;
                }
                if afk.username.as_str().trim().is_empty() {
                    continue;
                }
                let elapsed = afk.away_since.map(|v| now.saturating_duration_since(v));
                let Some(elapsed) = elapsed else {
                    continue;
                };

                let recipients = match ctx.db.get_afk_recipients_for_username(&afk.username).await {
                    Ok(items) if !items.is_empty() => items,
                    Ok(_) => continue,
                    Err(err) => {
                        tracing::error!(
                            component = "tt_worker",
                            tt_username = %afk.username,
                            error = %err,
                            "Failed to load AFK recipients in monitor"
                        );
                        continue;
                    }
                };
                for recipient in recipients {
                    if afk.notified_recipients.contains(&recipient.telegram_id) {
                        continue;
                    }
                    let threshold_seconds = recipient.threshold_minutes.max(1).cast_unsigned() * 60;
                    let cooldown_seconds = recipient.cooldown_seconds.max(0).cast_unsigned();
                    let effective = Duration::from_secs(threshold_seconds.max(cooldown_seconds));
                    if elapsed >= effective {
                        to_notify.push((user_id, recipient.telegram_id, afk.nickname.clone()));
                    }
                }
            }

            if to_notify.is_empty() {
                continue;
            }

            {
                let mut state = ctx.afk_state.lock().await;
                for (user_id, recipient, _) in &to_notify {
                    if let Some(s) = state.get_mut(user_id) {
                        s.notified_recipients.insert(*recipient);
                    }
                }
            }

            for (_, recipient, nickname) in to_notify {
                let _ = ctx
                    .tx_bridge
                    .send(crate::core::types::BridgeEvent::AfkStatus {
                        recipient,
                        nickname,
                        is_afk: true,
                    })
                    .await;
            }
        }
    });
}

pub(super) async fn preload_caches(services: &crate::app::services::tt_context::TtServiceContext) {
    if !tt_cache_service::preload_all_ctx(services).await {
        tracing::warn!(component = "tt_worker", "Failed to preload TT user caches");
    }
}

pub(super) fn build_start_next(
    is_streaming: Arc<std::sync::atomic::AtomicBool>,
) -> Arc<streaming::StartNextFn> {
    Arc::new(
        move |client: &Client,
              queue: &mut VecDeque<StreamItem>,
              current: &mut Option<StreamItem>,
              tx_cmd: &Sender<TtCommand>| {
            start_next_stream(client, queue, current, tx_cmd, &is_streaming);
        },
    )
}

pub(super) fn build_set_streaming_status(
    status_gender: teamtalk::types::UserGender,
    status_text: String,
) -> Arc<streaming::SetStreamingStatusFn> {
    Arc::new(move |client: &Client, streaming: bool| {
        let status = UserStatus {
            gender: status_gender,
            streaming,
            ..UserStatus::default()
        };
        client.set_status(status, &status_text);
    })
}

pub(super) fn configure_auto_reconnect(client: &Client, config: &crate::bootstrap::config::Config) {
    client.enable_auto_reconnect_with_events(
        ReconnectConfig {
            min_delay: Duration::from_millis(200),
            max_delay: Duration::from_mins(1),
            max_attempts: u32::MAX,
            stability_threshold: Duration::from_secs(10),
        },
        vec![Event::MySelfKicked],
    );

    client.set_login_params(LoginParams::new(
        config.teamtalk.nick_name.as_str(),
        config.teamtalk.user_name.as_str(),
        config.teamtalk.password.as_str(),
        config.teamtalk.client_name.as_str(),
    ));
}

pub(super) fn connect_teamtalk(
    client: &Client,
    host: &crate::core::types::TtHostName,
    port: i32,
    encrypted: bool,
    reconnect_retry_seconds: u64,
    reconnect_check_interval_seconds: u64,
) {
    tracing::info!(
        component = "tt_worker",
        host = %host,
        port,
        encrypted,
        reconnect_retry_seconds,
        reconnect_check_interval_seconds,
        "Connecting to TeamTalk"
    );
    if let Err(e) = client.connect_remember(host.as_str(), port, port, encrypted) {
        tracing::error!(
            host = %host,
            port,
            encrypted,
            error = %e,
            "TeamTalk connect failed"
        );
    }
}

fn start_next_stream(
    client: &Client,
    queue: &mut VecDeque<StreamItem>,
    current: &mut Option<StreamItem>,
    tx_cmd: &Sender<TtCommand>,
    is_streaming: &Arc<std::sync::atomic::AtomicBool>,
) {
    if current.is_some() {
        return;
    }
    while let Some(mut item) = queue.pop_front() {
        let channel_id = if item.channel_id.as_i32() == 0 {
            client.my_channel_id().0
        } else {
            item.channel_id.as_i32()
        };
        if let Some(text) = item.announce_text.take() {
            client.send_to_channel(ChannelId(channel_id), &text);
        }
        let playback = MediaFilePlayback {
            offset_ms: 0,
            paused: false,
        };
        let file_path = item.file_path.to_string_lossy();
        let started =
            client.start_streaming_media_file_to_channel_ex(file_path.as_ref(), &playback, None);
        if !started {
            tracing::error!(
                file_path = %item.file_path.display(),
                "Failed to start streaming"
            );
            let delete_path = item.file_path.clone();
            tokio::task::spawn_blocking(move || {
                let _ = std::fs::remove_file(&delete_path);
            });
            continue;
        }
        is_streaming.store(true, std::sync::atomic::Ordering::Relaxed);
        let stream_id = item.stream_id;
        let delete_path = item.file_path.clone();
        let duration_ms = item.duration_ms;
        let tx_cmd_for_stop = tx_cmd.clone();
        tokio::task::spawn_local(async move {
            tokio::time::sleep(Duration::from_millis(u64::from(duration_ms))).await;
            let _ = tx_cmd_for_stop
                .send(TtCommand::StopStreamingIf { stream_id })
                .await;

            tokio::time::sleep(Duration::from_secs(10)).await;
            let mut attempts = 0;
            loop {
                let delete_path_attempt = delete_path.clone();
                let res =
                    tokio::task::spawn_blocking(move || std::fs::remove_file(delete_path_attempt))
                        .await;

                match res {
                    Ok(Ok(())) => break,
                    Ok(Err(e)) => {
                        attempts += 1;
                        if attempts >= 10 {
                            tracing::error!(
                                file_path = %delete_path.display(),
                                error = %e,
                                "Failed to delete streamed file"
                            );
                            break;
                        }
                        tokio::time::sleep(Duration::from_secs(30)).await;
                    }
                    Err(e) => {
                        tracing::error!(
                            file_path = %delete_path.display(),
                            error = %e,
                            "Failed to join blocking file delete task"
                        );
                        break;
                    }
                }
            }
        });
        *current = Some(item);
        break;
    }
}

pub(super) fn build_worker_context(args: &RunTeamtalkArgs) -> WorkerContext {
    WorkerContext {
        config: args.config.clone(),
        state: args.state.clone(),
        tx_bridge: args.tx_bridge.clone(),
        tx_tt_cmd: args.tx_cmd_clone.clone(),
        db: args.db.clone(),
        bot_username: args.bot_username.clone(),
        is_streaming: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        tt_msg_sem: Arc::new(Semaphore::new(8)),
        tt_bridge_disabled_reply_state: Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::new(),
        )),
        afk_state: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        plugins: args.plugins.clone(),
    }
}
