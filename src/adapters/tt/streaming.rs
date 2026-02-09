use super::context::WorkerContext;
use crate::core::types::{RecordingFileFormat, TtChannelId, TtCommand};
use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use teamtalk::Client;
use teamtalk::types::{ChannelId, Subscriptions, UserId};
use tokio::sync::mpsc::Sender;

pub struct StreamItem {
    pub stream_id: u64,
    pub channel_id: TtChannelId,
    pub file_path: PathBuf,
    pub duration_ms: u32,
    pub announce_text: Option<String>,
}

pub struct ActiveRecording {
    pub channel_id: TtChannelId,
    pub file_path: PathBuf,
    pub notify_chat: Option<crate::core::types::TgChatId>,
    pub auto_subscribe_audio: bool,
    pub auto_subscribed_users: HashSet<UserId>,
}

pub type StartNextFn = dyn Fn(&Client, &mut VecDeque<StreamItem>, &mut Option<StreamItem>, &Sender<TtCommand>)
    + Send
    + Sync;

pub type SetStreamingStatusFn = dyn Fn(&Client, bool) + Send + Sync;

pub struct HandleCmdCtx<'a> {
    pub async_client: &'a mut teamtalk::AsyncClient,
    pub stream_seq: &'a mut u64,
    pub stream_queue: &'a mut VecDeque<StreamItem>,
    pub current_stream: &'a mut Option<StreamItem>,
    pub tx_cmd: &'a Sender<TtCommand>,
    pub is_streaming: &'a Arc<std::sync::atomic::AtomicBool>,
    pub worker_ctx: &'a WorkerContext,
    pub start_next: &'a StartNextFn,
    pub set_streaming_status: &'a SetStreamingStatusFn,
    pub recording: &'a mut Option<ActiveRecording>,
}

pub fn handle_cmd(cmd: TtCommand, ctx: &mut HandleCmdCtx<'_>) -> bool {
    match cmd {
        TtCommand::Shutdown => return true,
        TtCommand::Broadcast { text } => send_broadcast(ctx, &text),
        TtCommand::ReplyToUser { user_id, text } => send_user_reply(ctx, user_id, &text),
        TtCommand::SendToChannel { channel_id, text } => {
            send_channel_message(ctx, channel_id, &text);
        }
        TtCommand::EnqueueStream {
            channel_id,
            file_path,
            duration_ms,
            announce_text,
        } => enqueue_stream(ctx, channel_id, file_path, duration_ms, announce_text),
        TtCommand::StopStreamingIf { stream_id } => stop_streaming_if(ctx, stream_id),
        TtCommand::SkipStream => skip_stream(ctx),
        TtCommand::SetStreamingStatus { streaming } => set_streaming_status(ctx, streaming),
        TtCommand::KickUser { user_id } => kick_user(ctx, user_id),
        TtCommand::BanUser { user_id } => ban_user(ctx, user_id),
        TtCommand::Who {
            chat_id,
            lang,
            reply_to,
        } => send_who(ctx, chat_id, lang, reply_to),
        TtCommand::LoadAccounts => request_accounts(ctx),
        TtCommand::StartRecording { request } => start_recording(ctx, &request),
        TtCommand::StopRecording { request } => stop_recording(ctx, request),
        TtCommand::SyncRecordingSubscription { user_id } => {
            sync_recording_subscription(ctx, user_id);
        }
    }
    false
}

fn send_broadcast(ctx: &mut HandleCmdCtx<'_>, text: &str) {
    ctx.async_client.with_client_mut(|client_ref| {
        client_ref.send_to_all(text);
    });
}

fn send_user_reply(ctx: &mut HandleCmdCtx<'_>, user_id: crate::core::types::TtUserId, text: &str) {
    ctx.async_client.with_client_mut(|client_ref| {
        client_ref.send_to_user(UserId(user_id.as_i32()), text);
    });
}

fn send_channel_message(ctx: &mut HandleCmdCtx<'_>, channel_id: TtChannelId, text: &str) {
    ctx.async_client.with_client_mut(|client_ref| {
        client_ref.send_to_channel(ChannelId(channel_id.as_i32()), text);
    });
}

fn enqueue_stream(
    ctx: &mut HandleCmdCtx<'_>,
    channel_id: TtChannelId,
    file_path: PathBuf,
    duration_ms: u32,
    announce_text: Option<String>,
) {
    *ctx.stream_seq = ctx.stream_seq.wrapping_add(1);
    ctx.stream_queue.push_back(StreamItem {
        stream_id: *ctx.stream_seq,
        channel_id,
        file_path,
        duration_ms,
        announce_text,
    });
    ctx.async_client.with_client_mut(|client_ref| {
        (ctx.start_next)(client_ref, ctx.stream_queue, ctx.current_stream, ctx.tx_cmd);
    });
}

fn stop_streaming_if(ctx: &mut HandleCmdCtx<'_>, stream_id: u64) {
    if ctx
        .current_stream
        .as_ref()
        .is_some_and(|s| s.stream_id == stream_id)
    {
        stop_current_stream(ctx);
    }
}

fn skip_stream(ctx: &mut HandleCmdCtx<'_>) {
    if ctx.current_stream.is_some() {
        stop_current_stream(ctx);
    }
    ctx.async_client.with_client_mut(|client_ref| {
        (ctx.start_next)(client_ref, ctx.stream_queue, ctx.current_stream, ctx.tx_cmd);
    });
}

fn stop_current_stream(ctx: &mut HandleCmdCtx<'_>) {
    ctx.async_client.with_client_mut(|client_ref| {
        client_ref.stop_streaming_media_file_to_channel();
    });
    let is_streaming = ctx.is_streaming.clone();
    let tx_cmd_for_stop = ctx.tx_cmd.clone();
    tokio::task::spawn_local(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        if is_streaming.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = tx_cmd_for_stop
                .send(TtCommand::SetStreamingStatus { streaming: false })
                .await;
        }
    });
    *ctx.current_stream = None;
}

fn set_streaming_status(ctx: &mut HandleCmdCtx<'_>, streaming: bool) {
    if !streaming {
        ctx.is_streaming
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
    ctx.async_client.with_client_mut(|client_ref| {
        (ctx.set_streaming_status)(client_ref, streaming);
    });
}

fn kick_user(ctx: &mut HandleCmdCtx<'_>, user_id: crate::core::types::TtUserId) {
    ctx.async_client.with_client_mut(|client_ref| {
        client_ref.kick_user(UserId(user_id.as_i32()), teamtalk::types::ChannelId(0));
    });
}

fn ban_user(ctx: &mut HandleCmdCtx<'_>, user_id: crate::core::types::TtUserId) {
    ctx.async_client.with_client_mut(|client_ref| {
        client_ref.ban_user(UserId(user_id.as_i32()), client_ref.my_channel_id());
    });
}

fn send_who(
    ctx: &HandleCmdCtx<'_>,
    chat_id: crate::core::types::TgChatId,
    lang: crate::core::types::LanguageCode,
    reply_to: Option<crate::core::types::TgMessageId>,
) {
    ctx.async_client.with_client(|client_ref| {
        super::reports::handle_who_command(client_ref, ctx.worker_ctx, chat_id, lang, reply_to);
    });
}

fn request_accounts(ctx: &mut HandleCmdCtx<'_>) {
    tracing::info!(
        component = "tt_worker",
        "Requesting full user accounts list"
    );
    ctx.async_client.with_client_mut(|client_ref| {
        client_ref.list_user_accounts(0, 1000);
    });
}

fn start_recording(
    ctx: &mut HandleCmdCtx<'_>,
    request: &crate::core::types::RecordingStartRequest,
) {
    if ctx.recording.is_some() {
        tracing::warn!(component = "tt_worker", "Recording already active");
        return;
    }

    ctx.async_client.with_client_mut(|client_ref| {
        let channel_id = client_ref.my_channel_id().0;
        if channel_id <= 0 {
            tracing::warn!(
                component = "tt_worker",
                "Cannot start recording: bot is not in a channel"
            );
            return;
        }

        if let Some(parent) = request.output_path.parent()
            && !parent.as_os_str().is_empty()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            tracing::error!(
                component = "tt_worker",
                error = %error,
                "Failed to create recordings directory"
            );
            return;
        }
        let audio_format = match request.format {
            RecordingFileFormat::ChannelCodec => {
                teamtalk::client::ffi::AudioFileFormat::AFF_CHANNELCODEC_FORMAT
            }
            RecordingFileFormat::Wave => teamtalk::client::ffi::AudioFileFormat::AFF_WAVE_FORMAT,
            RecordingFileFormat::Mp3_128 => {
                teamtalk::client::ffi::AudioFileFormat::AFF_MP3_128KBIT_FORMAT
            }
        };

        let ok = client_ref.start_recording_channel(
            channel_id,
            request.output_path.to_string_lossy().as_ref(),
            audio_format,
        );
        if !ok {
            tracing::error!(component = "tt_worker", "Failed to start recording");
            return;
        }

        let mut auto_subscribed_users = HashSet::new();
        if request.auto_subscribe_audio {
            for user in client_ref.get_channel_users(ChannelId(channel_id)) {
                if user.id == client_ref.my_id() {
                    continue;
                }
                let _ = client_ref.subscribe(user.id, Subscriptions::all_audio());
                auto_subscribed_users.insert(user.id);
            }
        }

        *ctx.recording = Some(ActiveRecording {
            channel_id: TtChannelId::from(channel_id),
            file_path: request.output_path.clone(),
            notify_chat: request.notify_chat,
            auto_subscribe_audio: request.auto_subscribe_audio,
            auto_subscribed_users,
        });
        tracing::info!(
            component = "tt_worker",
            path = %request.output_path.display(),
            channel_id,
            "Recording started"
        );
    });
}

fn stop_recording(ctx: &mut HandleCmdCtx<'_>, request: crate::core::types::RecordingStopRequest) {
    let Some(active) = ctx.recording.take() else {
        tracing::warn!(component = "tt_worker", "Recording is not active");
        return;
    };

    let mut stop_ok = false;
    ctx.async_client.with_client_mut(|client_ref| {
        stop_ok = client_ref.stop_recording_channel(active.channel_id.as_i32());
        if active.auto_subscribe_audio {
            for user_id in &active.auto_subscribed_users {
                let _ = client_ref.unsubscribe(*user_id, Subscriptions::all_audio());
            }
        }
    });

    if !stop_ok {
        tracing::error!(component = "tt_worker", "Failed to stop recording");
        return;
    }

    let target_chat = request.notify_chat.or(active.notify_chat);
    if let Some(chat_id) = target_chat {
        let _ = ctx
            .worker_ctx
            .tx_bridge
            .try_send(crate::core::types::BridgeEvent::TgDocument {
                chat_id,
                file_path: active.file_path.clone(),
                caption: request.caption,
                delete_after_send: request.delete_after_send,
            });
    }

    tracing::info!(
        component = "tt_worker",
        path = %active.file_path.display(),
        "Recording stopped"
    );
}

fn sync_recording_subscription(ctx: &mut HandleCmdCtx<'_>, user_id: crate::core::types::TtUserId) {
    let Some(active) = ctx.recording.as_mut() else {
        return;
    };
    if !active.auto_subscribe_audio {
        return;
    }
    let user_id = UserId(user_id.as_i32());
    ctx.async_client.with_client_mut(|client_ref| {
        if user_id == client_ref.my_id() {
            return;
        }
        if active.auto_subscribed_users.contains(&user_id) {
            return;
        }
        let _ = client_ref.subscribe(user_id, Subscriptions::all_audio());
        active.auto_subscribed_users.insert(user_id);
    });
}
