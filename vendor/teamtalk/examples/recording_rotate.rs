use std::time::Duration;
use teamtalk::client::ffi::AudioFileFormat;
use teamtalk::{RecordingOptions, RecordingSession};

fn main() -> teamtalk::Result<()> {
    let client = teamtalk::Client::new()?;
    let channel_id = client.get_root_channel_id();

    let options = RecordingOptions::new(
        "recordings/session-{index}.wav",
        AudioFileFormat::AFF_WAVE_FORMAT,
    )
    .with_max_duration(Duration::from_secs(300))
    .with_max_size_bytes(50 * 1024 * 1024);
    let mut session = RecordingSession::start_channel(&client, channel_id, options)?;

    let _ = session.rotate_if_needed()?;
    let _ = session.stop();
    Ok(())
}
