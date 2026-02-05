use super::context::{RunTeamtalkArgs, WorkerContext};
use super::events;
use super::streaming::{HandleCmdCtx, StreamItem, handle_cmd};
use crate::app::services::tt_cache as tt_cache_service;
use crate::core::types::TtCommand;
use futures_util::StreamExt;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use teamtalk::Client;
use teamtalk::client::media::MediaFilePlayback;
use teamtalk::client::{ConnectParams, ReconnectConfig, ReconnectHandler};
use teamtalk::types::{ChannelId, UserStatus};
use tokio::sync::Semaphore;
use tokio::sync::mpsc::Sender;
use tokio::time::interval;

struct DriverCtx {
    async_client: teamtalk::AsyncClient,
    cmd_rx: tokio::sync::mpsc::Receiver<TtCommand>,
    stream_seq: u64,
    stream_queue: VecDeque<StreamItem>,
    current_stream: Option<StreamItem>,
    tx_cmd: Sender<TtCommand>,
    is_streaming: Arc<std::sync::atomic::AtomicBool>,
    ctx: WorkerContext,
    start_next: Arc<super::streaming::StartNextFn>,
    set_streaming_status: Arc<super::streaming::SetStreamingStatusFn>,
    tt_host_name: String,
    tt_port: i32,
    tt_encrypted: bool,
    reconnect_handler: ReconnectHandler,
    ready_time: Option<std::time::Instant>,
    is_connected: bool,
}

pub async fn run_teamtalk_worker(args: RunTeamtalkArgs) {
    let RunTeamtalkArgs {
        config,
        state,
        tx_bridge,
        mut rx_cmd,
        tx_cmd_clone,
        db,
        bot_username,
        client,
        tx_init,
    } = args;
    let tt_host_name = config.teamtalk.host_name.clone();
    let Ok(tt_port) = i32::try_from(config.teamtalk.port) else {
        tracing::error!(
            component = "tt_worker",
            port = config.teamtalk.port,
            "Invalid TeamTalk port"
        );
        return;
    };
    let tt_encrypted = config.teamtalk.encrypted;
    let tt_status_text = config.teamtalk.status_text.clone();
    let reconnect_retry_seconds = config.operational_parameters.tt_reconnect_retry;
    let reconnect_check_interval_seconds =
        config.operational_parameters.tt_reconnect_check_interval;

    let ctx = WorkerContext {
        config: config.clone(),
        state,
        tx_bridge,
        tx_tt_cmd: tx_cmd_clone.clone(),
        db,
        bot_username,
        is_streaming: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        tt_msg_sem: Arc::new(Semaphore::new(8)),
    };
    let is_streaming = ctx.is_streaming.clone();
    let services = ctx.services();
    preload_caches(&services).await;
    spawn_cache_refresh(ctx.state.clone(), services.clone());

    let _ = tx_init.send(Ok(()));
    let ready_time: Option<std::time::Instant> = None;
    let is_connected = false;
    let stream_queue: VecDeque<StreamItem> = VecDeque::new();
    let current_stream: Option<StreamItem> = None;
    let stream_seq: u64 = 0;
    let set_streaming_status =
        build_set_streaming_status(config.general.gender.to_user_gender(), tt_status_text);

    let reconnect_handler = build_reconnect_handler();

    connect_teamtalk(
        &client,
        &tt_host_name,
        tt_port,
        tt_encrypted,
        reconnect_retry_seconds,
        reconnect_check_interval_seconds,
    );

    let tt_host_name_for_driver = tt_host_name.clone();
    let tt_port_for_driver = tt_port;
    let tt_encrypted_for_driver = tt_encrypted;
    let start_next = build_start_next(is_streaming.clone());

    let async_client = client.into_async_with_config(teamtalk::AsyncConfig::new().buffer(1024));
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel::<TtCommand>(1024);
    let cmd_forwarder = tokio::task::spawn_local(async move {
        while let Some(cmd) = rx_cmd.recv().await {
            if cmd_tx.send(cmd).await.is_err() {
                break;
            }
        }
    });

    let driver = tokio::task::spawn_local(run_driver(DriverCtx {
        async_client,
        cmd_rx,
        stream_seq,
        stream_queue,
        current_stream,
        tx_cmd: tx_cmd_clone,
        is_streaming,
        ctx,
        start_next,
        set_streaming_status,
        tt_host_name: tt_host_name_for_driver.as_str().to_string(),
        tt_port: tt_port_for_driver,
        tt_encrypted: tt_encrypted_for_driver,
        reconnect_handler,
        ready_time,
        is_connected,
    }));

    let _ = driver.await;
    cmd_forwarder.abort();

    let _ = cmd_forwarder.await;
}

fn spawn_cache_refresh(
    state: crate::app::state::StateHandle,
    services: crate::app::services::tt_context::TtServiceContext,
) {
    tokio::task::spawn_local(async move {
        let mut tick = interval(Duration::from_secs(300));
        loop {
            tick.tick().await;
            let _ = tt_cache_service::preload_all_ctx(&services).await;
            let cache_stats = state.cache_stats().await.unwrap_or_default();
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

async fn preload_caches(services: &crate::app::services::tt_context::TtServiceContext) {
    if !tt_cache_service::preload_all_ctx(services).await {
        tracing::warn!(component = "tt_worker", "Failed to preload TT user caches");
    }
}

fn build_start_next(
    is_streaming: Arc<std::sync::atomic::AtomicBool>,
) -> Arc<super::streaming::StartNextFn> {
    Arc::new(
        move |client: &Client,
              queue: &mut VecDeque<StreamItem>,
              current: &mut Option<StreamItem>,
              tx_cmd: &Sender<TtCommand>| {
            start_next_stream(client, queue, current, tx_cmd, &is_streaming);
        },
    )
}

fn build_set_streaming_status(
    status_gender: teamtalk::types::UserGender,
    status_text: String,
) -> Arc<super::streaming::SetStreamingStatusFn> {
    Arc::new(move |client: &Client, streaming: bool| {
        let status = UserStatus {
            gender: status_gender,
            streaming,
            ..UserStatus::default()
        };
        client.set_status(status, &status_text);
    })
}

fn build_reconnect_handler() -> ReconnectHandler {
    ReconnectHandler::new(ReconnectConfig {
        min_delay: Duration::from_millis(200),
        max_delay: Duration::from_secs(60),
        max_attempts: u32::MAX,
        stability_threshold: Duration::from_secs(10),
    })
}

fn connect_teamtalk(
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
    if let Err(e) = client.connect(host.as_str(), port, port, encrypted) {
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

            tokio::time::sleep(Duration::from_millis(10_000)).await;
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

async fn run_driver(ctx: DriverCtx) {
    let DriverCtx {
        mut async_client,
        mut cmd_rx,
        mut stream_seq,
        mut stream_queue,
        mut current_stream,
        tx_cmd,
        is_streaming,
        ctx,
        start_next,
        set_streaming_status,
        tt_host_name,
        tt_port,
        tt_encrypted,
        mut reconnect_handler,
        mut ready_time,
        mut is_connected,
    } = ctx;
    let connect_params = ConnectParams {
        host: tt_host_name.as_str(),
        tcp: tt_port,
        udp: tt_port,
        encrypted: tt_encrypted,
    };
    let shutdown = loop {
        tokio::select! {
            maybe_cmd = cmd_rx.recv() => {
                let Some(cmd) = maybe_cmd else {
                    break true;
                };
                let mut handle_ctx = HandleCmdCtx {
                    async_client: &mut async_client,
                    stream_seq: &mut stream_seq,
                    stream_queue: &mut stream_queue,
                    current_stream: &mut current_stream,
                    tx_cmd: &tx_cmd,
                    is_streaming: &is_streaming,
                    worker_ctx: &ctx,
                    start_next: start_next.as_ref(),
                    set_streaming_status: set_streaming_status.as_ref(),
                };
                if handle_cmd(cmd, &mut handle_ctx) {
                    break true;
                }
            }
            maybe_event = async_client.next() => {
                let Some((event, msg)) = maybe_event else {
                    break true;
                };

                if current_stream.is_some() && matches!(event, teamtalk::events::Event::CmdProcessing) {
                    continue;
                }

                async_client.with_client(|client_ref| {
                    events::handle_sdk_event(
                        client_ref,
                        &ctx,
                        event,
                        &msg,
                        &mut is_connected,
                        &mut reconnect_handler,
                        &mut ready_time,
                    );
                });

                if !is_connected {
                    async_client.with_client_mut(|client_ref| {
                        client_ref.handle_reconnect(&connect_params, &mut reconnect_handler);
                    });
                }
                let mut shutdown_now = false;
                while let Ok(cmd) = cmd_rx.try_recv() {
                    let mut handle_ctx = HandleCmdCtx {
                        async_client: &mut async_client,
                        stream_seq: &mut stream_seq,
                        stream_queue: &mut stream_queue,
                        current_stream: &mut current_stream,
                        tx_cmd: &tx_cmd,
                        is_streaming: &is_streaming,
                        worker_ctx: &ctx,
                        start_next: start_next.as_ref(),
                        set_streaming_status: set_streaming_status.as_ref(),
                    };
                    if handle_cmd(cmd, &mut handle_ctx) {
                        shutdown_now = true;
                        break;
                    }
                }
                if shutdown_now {
                    break true;
                }
            }
        }
    };

    shutdown_driver(async_client, current_stream.is_some(), shutdown);
}

fn shutdown_driver(mut async_client: teamtalk::AsyncClient, has_stream: bool, shutdown: bool) {
    if shutdown {
        tracing::info!(component = "tt_worker", "Shutdown requested");
        if has_stream {
            tracing::info!(component = "tt_worker", "Stopping active stream");
            async_client.with_client_mut(|client_ref| {
                client_ref.stop_streaming_media_file_to_channel();
            });
        }
        tracing::info!(component = "tt_worker", "Logging out");
        async_client.with_client_mut(|client_ref| {
            client_ref.logout();
        });
    }

    if let Some(client) = async_client.into_client() {
        tracing::info!(component = "tt_worker", "Disconnecting");
        let _ = client.disconnect();
    }
}
