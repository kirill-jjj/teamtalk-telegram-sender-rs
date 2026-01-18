pub mod commands;
pub mod events;
pub mod reports;

use crate::bootstrap::config::Config;
use crate::core::types::{BridgeEvent, LanguageCode, LiteUser, TtCommand};
use crate::infra::db::Database;
use crate::infra::locales;
use futures::StreamExt;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use teamtalk::Client;
use teamtalk::client::media::MediaPlayback;
use teamtalk::client::{ConnectParams, ConnectParamsOwned, ReconnectConfig, ReconnectHandler};
use teamtalk::types::{AudioPreprocessor, ChannelId, UserGender, UserStatus};
use teamtalk::types::{UserAccount, UserId};
use teamtalk::{AsyncClient, AsyncConfig};
use tokio::sync::mpsc::{Receiver, Sender};

pub(super) fn resolve_server_name(
    tt_config: &crate::bootstrap::config::TeamTalkConfig,
    real_name: Option<&str>,
) -> String {
    tt_config
        .server_name
        .as_deref()
        .filter(|s| !s.is_empty())
        .or_else(|| real_name.filter(|s| !s.is_empty()))
        .unwrap_or(&tt_config.host_name)
        .to_string()
}

pub(super) fn resolve_channel_name(
    client: &Client,
    channel_id: ChannelId,
    lang: LanguageCode,
) -> String {
    if channel_id.0 == 0 {
        return locales::get_text(lang.as_str(), "tt-root-channel-name", None);
    }
    match client.get_channel(channel_id) {
        Some(channel) if !channel.name.is_empty() => channel.name,
        Some(_) => locales::get_text(lang.as_str(), "tt-root-channel-name", None),
        None => "Unknown".to_string(),
    }
}

pub struct WorkerContext {
    pub config: Arc<Config>,
    pub online_users: Arc<RwLock<HashMap<i32, LiteUser>>>,
    pub online_users_by_username: Arc<RwLock<HashMap<String, i32>>>,
    pub user_accounts: Arc<RwLock<HashMap<String, UserAccount>>>,
    pub tx_bridge: tokio::sync::mpsc::Sender<BridgeEvent>,
    pub tx_tt_cmd: Sender<TtCommand>,
    pub db: Database,
    pub rt: tokio::runtime::Handle,
    pub bot_username: Option<String>,
    pub is_streaming: Arc<std::sync::atomic::AtomicBool>,
}

pub struct RunTeamtalkArgs {
    pub config: Arc<Config>,
    pub online_users: Arc<RwLock<HashMap<i32, LiteUser>>>,
    pub online_users_by_username: Arc<RwLock<HashMap<String, i32>>>,
    pub user_accounts: Arc<RwLock<HashMap<String, UserAccount>>>,
    pub tx_bridge: tokio::sync::mpsc::Sender<BridgeEvent>,
    pub rx_cmd: Receiver<TtCommand>,
    pub tx_cmd_clone: Sender<TtCommand>,
    pub db: Database,
    pub rt: tokio::runtime::Handle,
    pub bot_username: Option<String>,
    pub tx_init: std::sync::mpsc::Sender<Result<(), String>>,
}

struct StreamItem {
    stream_id: u64,
    channel_id: i32,
    file_path: String,
    duration_ms: u32,
    announce_text: Option<String>,
}

struct StreamState {
    queue: VecDeque<StreamItem>,
    current: Option<StreamItem>,
    seq: u64,
}

impl StreamState {
    const fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            current: None,
            seq: 0,
        }
    }
}

struct CommandCtx<'a> {
    client: &'a Client,
    worker: &'a WorkerContext,
    tx_cmd_clone: &'a Sender<TtCommand>,
    is_streaming: &'a Arc<std::sync::atomic::AtomicBool>,
    status_gender: UserGender,
    status_text: &'a str,
}

struct AsyncWorkerArgs {
    async_client: AsyncClient,
    ctx: WorkerContext,
    rx_cmd: Receiver<TtCommand>,
    tx_cmd_clone: Sender<TtCommand>,
    is_streaming: Arc<std::sync::atomic::AtomicBool>,
    status_gender: UserGender,
    status_text: String,
    reconnect_handler: ReconnectHandler,
    connect_params: ConnectParamsOwned,
    reconnect_check_interval_seconds: u64,
}

pub fn run_teamtalk_thread(args: RunTeamtalkArgs) {
    let RunTeamtalkArgs {
        config,
        online_users,
        online_users_by_username,
        user_accounts,
        tx_bridge,
        rx_cmd,
        tx_cmd_clone,
        db,
        rt,
        bot_username,
        tx_init,
    } = args;

    let tt_config = &config.teamtalk;
    let reconnect_retry_seconds = config.operational_parameters.tt_reconnect_retry;
    let reconnect_check_interval_seconds =
        config.operational_parameters.tt_reconnect_check_interval;

    let ctx = WorkerContext {
        config: config.clone(),
        online_users,
        online_users_by_username,
        user_accounts,
        tx_bridge,
        tx_tt_cmd: tx_cmd_clone.clone(),
        db,
        rt,
        bot_username,
        is_streaming: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    };
    let is_streaming = ctx.is_streaming.clone();

    let Some(client) = init_client(&tx_init) else {
        return;
    };

    let status_gender = parse_status_gender(&config.general.gender);
    let status_text = config.teamtalk.status_text.clone();

    let reconnect_handler = build_reconnect_handler();
    let connect_params = build_connect_params(tt_config);

    tracing::info!(
        component = "tt_worker",
        host = %tt_config.host_name,
        port = tt_config.port,
        encrypted = tt_config.encrypted,
        reconnect_retry_seconds,
        reconnect_check_interval_seconds,
        "Connecting to TeamTalk"
    );

    let async_client = client.into_async_with_config(AsyncConfig::new().poll_timeout_ms(100));
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_time()
        .build()
        .expect("failed to build TT runtime");

    let args = AsyncWorkerArgs {
        async_client,
        ctx,
        rx_cmd,
        tx_cmd_clone,
        is_streaming,
        status_gender,
        status_text,
        reconnect_handler,
        connect_params,
        reconnect_check_interval_seconds,
    };

    rt.block_on(run_async_worker(args));
}

async fn run_async_worker(mut args: AsyncWorkerArgs) {
    let mut ready_time: Option<std::time::Instant> = None;
    let mut is_connected = false;
    let mut stream_state = StreamState::new();
    let mut shutdown = false;
    let mut reconnect_tick = tokio::time::interval(Duration::from_secs(
        args.reconnect_check_interval_seconds.max(1),
    ));

    args.async_client.with_client(|client| {
        let params = connect_params_ref(&args.connect_params);
        connect_to_teamtalk(client, &params);
    });

    loop {
        tokio::select! {
            _ = reconnect_tick.tick() => {
                if !is_connected {
                    args.async_client.with_client(|client| {
                        let params = connect_params_ref(&args.connect_params);
                        client.handle_reconnect(&params, &mut args.reconnect_handler);
                    });
                }
            }
            cmd = args.rx_cmd.recv() => {
                match cmd {
                    Some(cmd) => {
                        args.async_client.with_client(|client| {
                            let cmd_ctx = CommandCtx {
                                client,
                                worker: &args.ctx,
                                tx_cmd_clone: &args.tx_cmd_clone,
                                is_streaming: &args.is_streaming,
                                status_gender: args.status_gender,
                                status_text: &args.status_text,
                            };
                            handle_command(&cmd_ctx, &mut stream_state, &mut shutdown, cmd);
                        });
                    }
                    None => shutdown = true,
                }
            }
            maybe_event = args.async_client.next() => {
                let Some((event, msg)) = maybe_event else {
                    break;
                };
                if stream_state.current.is_some() && matches!(event, teamtalk::events::Event::CmdProcessing) {
                    continue;
                }
                args.async_client.with_client(|client| {
                    events::handle_sdk_event(
                        client,
                        &args.ctx,
                        event,
                        &msg,
                        &mut is_connected,
                        &mut args.reconnect_handler,
                        &mut ready_time,
                    );
                });
            }
        }
        if shutdown {
            break;
        }
    }

    args.async_client.stop();
    args.async_client.with_client(|client| {
        shutdown_teamtalk(client, stream_state.current.is_some());
        let _ = client.disconnect();
    });
}

fn init_client(tx_init: &std::sync::mpsc::Sender<Result<(), String>>) -> Option<Client> {
    match Client::new() {
        Ok(c) => {
            if let Err(e) = tx_init.send(Ok(())) {
                tracing::error!(error = %e, "Failed to signal TT init success");
            }
            Some(c)
        }
        Err(e) => {
            let err_msg = format!("Failed to initialize TeamTalk SDK: {e}");
            tracing::error!(error = %e, "Failed to initialize TeamTalk SDK");
            if let Err(send_err) = tx_init.send(Err(err_msg)) {
                tracing::error!(error = %send_err, "Failed to signal TT init failure");
            }
            None
        }
    }
}

fn build_reconnect_handler() -> ReconnectHandler {
    ReconnectHandler::new(ReconnectConfig {
        min_delay: Duration::from_millis(200),
        max_delay: Duration::from_secs(60),
        max_attempts: u32::MAX,
        stability_threshold: Duration::from_secs(10),
    })
}

fn build_connect_params(
    tt_config: &crate::bootstrap::config::TeamTalkConfig,
) -> ConnectParamsOwned {
    let port = i32::try_from(tt_config.port).unwrap_or_else(|_| {
        tracing::error!(port = tt_config.port, "Invalid TeamTalk port");
        0
    });
    ConnectParamsOwned::new(&tt_config.host_name, port, port, tt_config.encrypted)
}

fn connect_params_ref(params: &ConnectParamsOwned) -> ConnectParams<'_> {
    ConnectParams {
        host: &params.host,
        tcp: params.tcp,
        udp: params.udp,
        encrypted: params.encrypted,
    }
}

fn connect_to_teamtalk(client: &Client, params: &ConnectParams<'_>) {
    if let Err(e) = client.connect(params.host, params.tcp, params.udp, params.encrypted) {
        tracing::error!(
            host = %params.host,
            port = params.tcp,
            encrypted = params.encrypted,
            error = %e,
            "TeamTalk connect failed"
        );
    }
}

fn parse_status_gender(raw: &str) -> UserGender {
    match raw.trim().to_lowercase().as_str() {
        "male" => UserGender::Male,
        "female" => UserGender::Female,
        _ => UserGender::Neutral,
    }
}

fn handle_command(
    cmd_ctx: &CommandCtx<'_>,
    stream_state: &mut StreamState,
    shutdown: &mut bool,
    cmd: TtCommand,
) {
    match cmd {
        TtCommand::Shutdown => {
            *shutdown = true;
        }
        TtCommand::ReplyToUser { user_id, text } => {
            cmd_ctx.client.send_to_user(UserId(user_id), &text);
        }
        TtCommand::SendToChannel { channel_id, text } => {
            cmd_ctx.client.send_to_channel(ChannelId(channel_id), &text);
        }
        TtCommand::EnqueueStream {
            channel_id,
            file_path,
            duration_ms,
            announce_text,
        } => {
            stream_state.seq = stream_state.seq.wrapping_add(1);
            stream_state.queue.push_back(StreamItem {
                stream_id: stream_state.seq,
                channel_id,
                file_path,
                duration_ms,
                announce_text,
            });
            start_next_stream(
                cmd_ctx.client,
                stream_state,
                cmd_ctx.tx_cmd_clone,
                cmd_ctx.is_streaming,
            );
        }
        TtCommand::StopStreamingIf { stream_id } => {
            stop_stream_if_current(
                cmd_ctx.client,
                stream_state,
                stream_id,
                cmd_ctx.tx_cmd_clone,
                cmd_ctx.is_streaming,
            );
        }
        TtCommand::SkipStream => {
            skip_stream(
                cmd_ctx.client,
                stream_state,
                cmd_ctx.tx_cmd_clone,
                cmd_ctx.is_streaming,
            );
        }
        TtCommand::SetStreamingStatus { streaming } => {
            if !streaming {
                cmd_ctx
                    .is_streaming
                    .store(false, std::sync::atomic::Ordering::Relaxed);
            }
            apply_streaming_status(
                cmd_ctx.client,
                cmd_ctx.status_gender,
                cmd_ctx.status_text,
                streaming,
            );
        }
        TtCommand::KickUser { user_id } => {
            cmd_ctx
                .client
                .kick_user(UserId(user_id), teamtalk::types::ChannelId(0));
        }
        TtCommand::BanUser { user_id } => {
            cmd_ctx
                .client
                .ban_user(UserId(user_id), cmd_ctx.client.my_channel_id());
        }
        TtCommand::Who {
            chat_id,
            lang,
            reply_to,
        } => {
            reports::handle_who_command(cmd_ctx.client, cmd_ctx.worker, chat_id, lang, reply_to);
        }
        TtCommand::LoadAccounts => {
            tracing::info!(
                component = "tt_worker",
                "Requesting full user accounts list"
            );
            cmd_ctx.client.list_user_accounts(0, 1000);
        }
    }
}

fn apply_streaming_status(client: &Client, gender: UserGender, status_text: &str, streaming: bool) {
    let status = UserStatus {
        gender,
        streaming,
        ..UserStatus::default()
    };
    client.set_status(status, status_text);
}

fn start_next_stream(
    client: &Client,
    stream_state: &mut StreamState,
    tx_cmd: &Sender<TtCommand>,
    is_streaming: &Arc<std::sync::atomic::AtomicBool>,
) {
    if stream_state.current.is_some() {
        return;
    }
    while let Some(mut item) = stream_state.queue.pop_front() {
        let channel_id = if item.channel_id == 0 {
            client.my_channel_id().0
        } else {
            item.channel_id
        };
        if let Some(text) = item.announce_text.take() {
            client.send_to_channel(ChannelId(channel_id), &text);
        }
        let playback = MediaPlayback {
            offset_ms: 0,
            paused: false,
            preprocessor: AudioPreprocessor::None,
        };
        let started = client.start_streaming_ex(&item.file_path, &playback, None);
        if !started {
            tracing::error!(file_path = %item.file_path, "Failed to start streaming");
            let delete_path = item.file_path.clone();
            std::thread::spawn(move || {
                let _ = std::fs::remove_file(&delete_path);
            });
            continue;
        }
        is_streaming.store(true, std::sync::atomic::Ordering::Relaxed);
        let stream_id = item.stream_id;
        let delete_path = item.file_path.clone();
        let duration_ms = item.duration_ms;
        let tx_cmd_for_stop = tx_cmd.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(u64::from(duration_ms)));
            let _ = tx_cmd_for_stop.blocking_send(TtCommand::StopStreamingIf { stream_id });
            std::thread::sleep(Duration::from_millis(10_000));
            let mut attempts = 0;
            loop {
                match std::fs::remove_file(&delete_path) {
                    Ok(()) => break,
                    Err(e) => {
                        attempts += 1;
                        if attempts >= 10 {
                            tracing::error!(
                                file_path = %delete_path,
                                error = %e,
                                "Failed to delete streamed file"
                            );
                            break;
                        }
                        std::thread::sleep(Duration::from_secs(30));
                    }
                }
            }
        });
        stream_state.current = Some(item);
        break;
    }
}

fn stop_stream_if_current(
    client: &Client,
    stream_state: &mut StreamState,
    stream_id: u64,
    tx_cmd_clone: &Sender<TtCommand>,
    is_streaming: &Arc<std::sync::atomic::AtomicBool>,
) {
    if stream_state
        .current
        .as_ref()
        .is_some_and(|s| s.stream_id == stream_id)
    {
        stop_current_stream(client, stream_state, tx_cmd_clone, is_streaming);
        start_next_stream(client, stream_state, tx_cmd_clone, is_streaming);
    }
}

fn skip_stream(
    client: &Client,
    stream_state: &mut StreamState,
    tx_cmd_clone: &Sender<TtCommand>,
    is_streaming: &Arc<std::sync::atomic::AtomicBool>,
) {
    if stream_state.current.is_some() {
        stop_current_stream(client, stream_state, tx_cmd_clone, is_streaming);
    }
    start_next_stream(client, stream_state, tx_cmd_clone, is_streaming);
}

fn stop_current_stream(
    client: &Client,
    stream_state: &mut StreamState,
    tx_cmd_clone: &Sender<TtCommand>,
    is_streaming: &Arc<std::sync::atomic::AtomicBool>,
) {
    client.stop_streaming();
    let is_streaming = is_streaming.clone();
    let tx_cmd_for_stop = tx_cmd_clone.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(2));
        if is_streaming.load(std::sync::atomic::Ordering::Relaxed) {
            let _ =
                tx_cmd_for_stop.blocking_send(TtCommand::SetStreamingStatus { streaming: false });
        }
    });
    stream_state.current = None;
}

fn shutdown_teamtalk(client: &Client, has_stream: bool) {
    tracing::info!(component = "tt_worker", "Disconnecting");
    if has_stream {
        tracing::info!(component = "tt_worker", "Stopping active stream");
        client.stop_streaming();
    }
    tracing::info!(component = "tt_worker", "Logging out");
    client.logout();
}

#[cfg(test)]
#[path = "../../../tests/unit/tt_mod.rs"]
mod tests;
