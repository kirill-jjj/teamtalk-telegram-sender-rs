use crate::adapters::tt::WorkerContext;
use teamtalk::client::ffi;
use teamtalk::types::UserStatus;
use teamtalk::{Client, Message};

use super::parse_gender_cfg;

pub(super) fn handle_stream_media_file(client: &Client, ctx: &WorkerContext, msg: &Message) {
    let raw = msg.raw();
    let info = unsafe { teamtalk::types::MediaFileInfo::from(raw.__bindgen_anon_1.mediafileinfo) };
    let gender = parse_gender_cfg(ctx.config.general.gender);
    match info.status {
        ffi::MediaFileStatus::MFS_CLOSED
        | ffi::MediaFileStatus::MFS_ERROR
        | ffi::MediaFileStatus::MFS_FINISHED
        | ffi::MediaFileStatus::MFS_ABORTED => {
            client.stop_streaming_media_file_to_channel();
            ctx.is_streaming
                .store(false, std::sync::atomic::Ordering::Relaxed);
            let status = UserStatus {
                gender,
                streaming: false,
                ..UserStatus::default()
            };
            client.set_status(status, &ctx.config.teamtalk.status_text);
        }
        ffi::MediaFileStatus::MFS_PAUSED => {
            if ctx.is_streaming.load(std::sync::atomic::Ordering::Relaxed) {
                let status = UserStatus {
                    gender,
                    streaming: true,
                    media_paused: true,
                    ..UserStatus::default()
                };
                client.set_status(status, &ctx.config.teamtalk.status_text);
            }
        }
        ffi::MediaFileStatus::MFS_STARTED | ffi::MediaFileStatus::MFS_PLAYING => {
            if ctx.is_streaming.load(std::sync::atomic::Ordering::Relaxed) {
                let status = UserStatus {
                    gender,
                    streaming: true,
                    ..UserStatus::default()
                };
                client.set_status(status, &ctx.config.teamtalk.status_text);
            }
        }
    }
}
