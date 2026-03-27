use crate::adapters::tt::context::WorkerContext;
use crate::adapters::tt::handlers::events;
use crate::adapters::tt::streaming;
use crate::adapters::tt::streaming::{ActiveRecording, HandleCmdCtx, StreamItem, handle_cmd};
use crate::core::types::TtCommand;
use futures_util::StreamExt;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub(super) struct DriverCtx {
    pub(super) async_client: teamtalk::AsyncClient,
    pub(super) cmd_rx: tokio::sync::mpsc::Receiver<TtCommand>,
    pub(super) stream_seq: u64,
    pub(super) stream_queue: VecDeque<StreamItem>,
    pub(super) current_stream: Option<StreamItem>,
    pub(super) recording: Option<ActiveRecording>,
    pub(super) tx_cmd: Sender<TtCommand>,
    pub(super) is_streaming: Arc<std::sync::atomic::AtomicBool>,
    pub(super) ctx: WorkerContext,
    pub(super) start_next: Arc<streaming::StartNextFn>,
    pub(super) set_streaming_status: Arc<streaming::SetStreamingStatusFn>,
    pub(super) ready_time: Option<std::time::Instant>,
}

pub(super) async fn run_driver(ctx: DriverCtx) {
    let DriverCtx {
        mut async_client,
        mut cmd_rx,
        mut stream_seq,
        mut stream_queue,
        mut current_stream,
        mut recording,
        tx_cmd,
        is_streaming,
        ctx,
        start_next,
        set_streaming_status,
        mut ready_time,
    } = ctx;
    let shutdown = loop {
        tokio::select! {
            received_cmd = cmd_rx.recv() => {
                let Some(cmd) = received_cmd else {
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
                    recording: &mut recording,
                };
                if handle_cmd(cmd, &mut handle_ctx) {
                    break true;
                }
            }
            sdk_event = async_client.next() => {
                let Some((event, msg)) = sdk_event else {
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
                        &mut ready_time,
                    );
                });
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
                        recording: &mut recording,
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

fn shutdown_driver(async_client: teamtalk::AsyncClient, has_stream: bool, shutdown: bool) {
    if shutdown {
        tracing::info!(component = "tt_worker", "Shutdown requested");
        if has_stream {
            tracing::info!(component = "tt_worker", "Stopping active stream");
            async_client.with_client(|client_ref| {
                client_ref.stop_streaming_media_file_to_channel();
            });
        }
        tracing::info!(component = "tt_worker", "Logging out");
        async_client.with_client(|client_ref| {
            client_ref.logout();
        });
    }

    if let Some(client) = async_client.into_client() {
        tracing::info!(component = "tt_worker", "Disconnecting");
        let _ = client.disconnect();
    }
}
