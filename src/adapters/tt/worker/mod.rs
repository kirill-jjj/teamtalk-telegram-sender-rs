mod driver;
mod setup;

use super::context::RunTeamtalkArgs;
use crate::app::services::tt_follow;
use crate::core::types::TtCommand;
use std::collections::VecDeque;

use driver::{DriverCtx, run_driver};

pub async fn run_teamtalk_worker(args: RunTeamtalkArgs) {
    let tt_host_name = args.config.teamtalk.host_name.clone();
    let Ok(tt_port) = i32::try_from(args.config.teamtalk.port) else {
        tracing::error!(
            component = "tt_worker",
            port = args.config.teamtalk.port,
            "Invalid TeamTalk port"
        );
        return;
    };
    let tt_encrypted = args.config.teamtalk.encrypted;
    let tt_status_text = args.config.teamtalk.status_text.clone();
    let reconnect_retry_seconds = args.config.operational_parameters.tt_reconnect_retry;
    let reconnect_check_interval_seconds = args
        .config
        .operational_parameters
        .tt_reconnect_check_interval;
    let ctx = setup::build_worker_context(&args);
    let permanent_override = tt_follow::load_permanent_override(&args.db).await;
    {
        let mut follow = ctx
            .follow_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        follow.permanent_override = permanent_override;
    }

    let mut rx_cmd = args.rx_cmd;
    let tx_cmd_clone = args.tx_cmd_clone.clone();
    let client = args.client;
    let tx_init = args.tx_init;

    let is_streaming = ctx.is_streaming.clone();
    let services = ctx.services();
    setup::preload_caches(&services).await;
    setup::spawn_cache_refresh(ctx.state.clone(), services.clone());

    let _ = tx_init.send(Ok(()));
    let ready_time: Option<std::time::Instant> = None;
    let is_connected = false;
    let stream_queue = VecDeque::new();
    let current_stream = None;
    let recording = None;
    let stream_seq = 0;
    let set_streaming_status = setup::build_set_streaming_status(
        args.config.general.gender.to_user_gender(),
        tt_status_text,
    );

    let reconnect_handler = setup::build_reconnect_handler();

    setup::connect_teamtalk(
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
    let start_next = setup::build_start_next(is_streaming.clone());

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
        recording,
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

    if let Err(err) = driver.await {
        tracing::error!(error = %err, "TeamTalk worker driver task failed");
    }
    cmd_forwarder.abort();

    if let Err(err) = cmd_forwarder.await {
        tracing::debug!(error = %err, "TeamTalk command forwarder task stopped");
    }
}
