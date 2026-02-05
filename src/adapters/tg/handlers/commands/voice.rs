use crate::adapters::tg::state::AppState;
use crate::core::types::TtCommand;
use std::time::{SystemTime, UNIX_EPOCH};
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::Voice;
use tokio::fs::File;

#[derive(Debug, thiserror::Error)]
pub enum StreamVoiceError {
    #[error("Telegram request failed: {0}")]
    Telegram(#[from] teloxide::RequestError),
    #[error("Telegram download failed: {0}")]
    Download(#[from] teloxide::DownloadError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to send TeamTalk stream command: {0}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<TtCommand>),
}

pub async fn stream_voice(
    bot: &Bot,
    state: &AppState,
    announce: Option<(crate::core::types::TtChannelId, String)>,
    voice: &Voice,
) -> Result<(), StreamVoiceError> {
    let file_info = bot.get_file(voice.file.id.clone()).await?;
    let mut temp_path = std::env::temp_dir();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    temp_path.push(format!("tg-voice-{}-{}.ogg", voice.file.id, now));
    let mut dst = File::create(&temp_path).await?;
    bot.download_file(&file_info.path, &mut dst).await?;

    let duration_ms = voice.duration.seconds().saturating_mul(1000);
    let (channel_id, announce_text) = announce.map_or_else(
        || (crate::core::types::TtChannelId::from(0), None),
        |(id, text)| (id, Some(text)),
    );
    state
        .tx_tt
        .send(TtCommand::EnqueueStream {
            channel_id,
            file_path: temp_path,
            duration_ms,
            announce_text,
        })
        .await?;
    Ok(())
}
