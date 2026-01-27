#![allow(clippy::pedantic, clippy::nursery)]

pub mod commands;
pub mod events;
pub mod reports;

use crate::bootstrap::config::Config;
use crate::core::types::{BridgeEvent, LanguageCode, LiteUser, TtCommand};
use crate::infra::db::Database;
use crate::infra::locales;
use futures_util::StreamExt;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use teamtalk::Client;
use teamtalk::client::media::MediaPlayback;
use teamtalk::client::{ConnectParams, ReconnectConfig, ReconnectHandler};
use teamtalk::types::{AudioPreprocessor, ChannelId, UserStatus};
use teamtalk::types::{UserAccount, UserId};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;

pub(super) fn resolve_server_name(
    tt_config: &crate::bootstrap::config::TeamTalkConfig,
    real_name: Option<&str>,
) -> String {
    tt_config
        .server_name
        .as_deref()
        .filter(|s| !s.is_empty())
        .or(real_name.filter(|s| !s.is_empty()))
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
    pub bot_username: Option<String>,
    pub tx_init: oneshot::Sender<Result<(), String>>,
}

struct StreamItem {
    stream_id: u64,
    channel_id: i32,
    file_path: String,
    duration_ms: u32,
    announce_text: Option<String>,
}

pub async fn run_teamtalk_worker(args: RunTeamtalkArgs) {
    let RunTeamtalkArgs {
        config,
        online_users,
        online_users_by_username,
        user_accounts,
        tx_bridge,
        mut rx_cmd,
        tx_cmd_clone,
        db,
        bot_username,
        tx_init,
    } = args;
    let tt_config = &config.teamtalk;
    let reconnect_retry_seconds = config.operational_parameters.tt_reconnect_retry;
    let reconnect_check_interval_seconds =
        config.operational_parameters.tt_reconnect_check_interval;

    let ctx = WorkerContext {
        config: config.clone(),
        online_users: online_users.clone(),
        online_users_by_username,
        user_accounts,
        tx_bridge,
        tx_tt_cmd: tx_cmd_clone.clone(),
        db,
        bot_username,
        is_streaming: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    };
    let is_streaming = ctx.is_streaming.clone();

    let client = match Client::new() {
        Ok(c) => {
            let _ = tx_init.send(Ok(()));
            c
        }
        Err(e) => {
            let err_msg = format!("Failed to initialize TeamTalk SDK: {}", e);
            tracing::error!(error = %e, "Failed to initialize TeamTalk SDK");
            let _ = tx_init.send(Err(err_msg));
            return;
        }
    };
    let mut ready_time: Option<std::time::Instant> = None;
    let mut is_connected = false;
    let mut stream_queue: VecDeque<StreamItem> = VecDeque::new();
    let mut current_stream: Option<StreamItem> = None;
    let mut stream_seq: u64 = 0;
    let status_gender = config.general.gender.to_user_gender();
    let set_streaming_status = |client: &Client, streaming: bool| {
        let status = UserStatus {
            gender: status_gender,
            streaming,
            ..UserStatus::default()
        };
        client.set_status(status, &config.teamtalk.status_text);
    };

    let mut reconnect_handler = ReconnectHandler::new(ReconnectConfig {
        min_delay: Duration::from_millis(200),
        max_delay: Duration::from_secs(60),
        max_attempts: u32::MAX,
        stability_threshold: Duration::from_secs(10),
    });

    let connect_params = ConnectParams {
        host: &tt_config.host_name,
        tcp: tt_config.port as i32,
        udp: tt_config.port as i32,
        encrypted: tt_config.encrypted,
    };

    tracing::info!(
        component = "tt_worker",
        host = %tt_config.host_name,
        port = tt_config.port,
        encrypted = tt_config.encrypted,
        reconnect_retry_seconds,
        reconnect_check_interval_seconds,
        "Connecting to TeamTalk"
    );

    if let Err(e) = client.connect(
        connect_params.host,
        connect_params.tcp,
        connect_params.udp,
        connect_params.encrypted,
    ) {
        tracing::error!(
            host = %connect_params.host,
            port = connect_params.tcp,
            encrypted = connect_params.encrypted,
            error = %e,
            "TeamTalk connect failed"
        );
    }

    let is_streaming_for_start = is_streaming.clone();
    let start_next = |client: &Client,
                      queue: &mut VecDeque<StreamItem>,
                      current: &mut Option<StreamItem>,
                      tx_cmd: &Sender<TtCommand>| {
        if current.is_some() {
            return;
        }
        while let Some(mut item) = queue.pop_front() {
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
                tracing::error!(
                    file_path = %item.file_path,
                    "Failed to start streaming"
                );
                let delete_path = item.file_path.clone();
                tokio::task::spawn_blocking(move || {
                    let _ = std::fs::remove_file(&delete_path);
                });
                continue;
            }
            is_streaming_for_start.store(true, std::sync::atomic::Ordering::Relaxed);
            let stream_id = item.stream_id;
            let delete_path = item.file_path.clone();
            let duration_ms = item.duration_ms;
            let tx_cmd_for_stop = tx_cmd.clone();
            tokio::task::spawn_local(async move {
                tokio::time::sleep(Duration::from_millis(duration_ms as u64)).await;
                let _ = tx_cmd_for_stop
                    .send(TtCommand::StopStreamingIf { stream_id })
                    .await;

                tokio::time::sleep(Duration::from_millis(10_000)).await;
                let mut attempts = 0;
                loop {
                    let delete_path_attempt = delete_path.clone();
                    let res = tokio::task::spawn_blocking(move || {
                        std::fs::remove_file(delete_path_attempt)
                    })
                    .await;

                    match res {
                        Ok(Ok(())) => break,
                        Ok(Err(e)) => {
                            attempts += 1;
                            if attempts >= 10 {
                                tracing::error!(
                                    file_path = %delete_path,
                                    error = %e,
                                    "Failed to delete streamed file"
                                );
                                break;
                            }
                            tokio::time::sleep(Duration::from_secs(30)).await;
                        }
                        Err(e) => {
                            tracing::error!(
                                file_path = %delete_path,
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
    };

    let mut async_client = client.into_async_with_config(teamtalk::AsyncConfig::new().buffer(256));
    let shutdown = loop {
        tokio::select! {
            maybe_cmd = rx_cmd.recv() => {
                let Some(cmd) = maybe_cmd else {
                    break true;
                };
                match cmd {
                    TtCommand::Shutdown => {
                        break true;
                    }
                    TtCommand::Broadcast { text } => {
                        async_client.with_client_mut(|client_ref| {
                            client_ref.send_to_all(&text);
                        });
                    }
                    TtCommand::ReplyToUser { user_id, text } => {
                        async_client.with_client_mut(|client_ref| {
                            client_ref.send_to_user(UserId(user_id), &text);
                        });
                    }
                    TtCommand::SendToChannel { channel_id, text } => {
                        async_client.with_client_mut(|client_ref| {
                            client_ref.send_to_channel(ChannelId(channel_id), &text);
                        });
                    }
                    TtCommand::EnqueueStream {
                        channel_id,
                        file_path,
                        duration_ms,
                        announce_text,
                    } => {
                        stream_seq = stream_seq.wrapping_add(1);
                        stream_queue.push_back(StreamItem {
                            stream_id: stream_seq,
                            channel_id,
                            file_path,
                            duration_ms,
                            announce_text,
                        });
                        async_client.with_client_mut(|client_ref| {
                            start_next(
                                client_ref,
                                &mut stream_queue,
                                &mut current_stream,
                                &tx_cmd_clone,
                            );
                        });
                    }
                    TtCommand::StopStreamingIf { stream_id } => {
                        if current_stream
                            .as_ref()
                            .map(|s| s.stream_id == stream_id)
                            .unwrap_or(false)
                        {
                            async_client.with_client_mut(|client_ref| {
                                client_ref.stop_streaming();
                            });
                            let is_streaming = is_streaming.clone();
                            let tx_cmd_for_stop = tx_cmd_clone.clone();
                            tokio::task::spawn_local(async move {
                                tokio::time::sleep(Duration::from_secs(2)).await;
                                if is_streaming.load(std::sync::atomic::Ordering::Relaxed) {
                                    let _ = tx_cmd_for_stop
                                        .send(TtCommand::SetStreamingStatus { streaming: false })
                                        .await;
                                }
                            });
                            current_stream = None;
                            async_client.with_client_mut(|client_ref| {
                                start_next(
                                    client_ref,
                                    &mut stream_queue,
                                    &mut current_stream,
                                    &tx_cmd_clone,
                                );
                            });
                        }
                    }
                    TtCommand::SkipStream => {
                        if current_stream.is_some() {
                            async_client.with_client_mut(|client_ref| {
                                client_ref.stop_streaming();
                            });
                            let is_streaming = is_streaming.clone();
                            let tx_cmd_for_stop = tx_cmd_clone.clone();
                            tokio::task::spawn_local(async move {
                                tokio::time::sleep(Duration::from_secs(2)).await;
                                if is_streaming.load(std::sync::atomic::Ordering::Relaxed) {
                                    let _ = tx_cmd_for_stop
                                        .send(TtCommand::SetStreamingStatus { streaming: false })
                                        .await;
                                }
                            });
                            current_stream = None;
                        }
                        async_client.with_client_mut(|client_ref| {
                            start_next(
                                client_ref,
                                &mut stream_queue,
                                &mut current_stream,
                                &tx_cmd_clone,
                            );
                        });
                    }
                    TtCommand::SetStreamingStatus { streaming } => {
                        if !streaming {
                            is_streaming.store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                        async_client.with_client_mut(|client_ref| {
                            set_streaming_status(client_ref, streaming);
                        });
                    }
                    TtCommand::KickUser { user_id } => {
                        async_client.with_client_mut(|client_ref| {
                            client_ref.kick_user(UserId(user_id), teamtalk::types::ChannelId(0));
                        });
                    }
                    TtCommand::BanUser { user_id } => {
                        async_client.with_client_mut(|client_ref| {
                            client_ref.ban_user(UserId(user_id), client_ref.my_channel_id());
                        });
                    }
                    TtCommand::Who {
                        chat_id,
                        lang,
                        reply_to,
                    } => {
                        async_client.with_client(|client_ref| {
                            reports::handle_who_command(client_ref, &ctx, chat_id, lang, reply_to);
                        });
                    }
                    TtCommand::LoadAccounts => {
                        tracing::info!(
                            component = "tt_worker",
                            "Requesting full user accounts list"
                        );
                        async_client.with_client_mut(|client_ref| {
                            client_ref.list_user_accounts(0, 1000);
                        });
                    }
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
                        msg,
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
            }
        }
    };

    if shutdown {
        tracing::info!(component = "tt_worker", "Shutdown requested");
        if current_stream.is_some() {
            tracing::info!(component = "tt_worker", "Stopping active stream");
            async_client.with_client_mut(|client_ref| {
                client_ref.stop_streaming();
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
