use teamtalk::client::ffi::AudioFileFormat;
use teamtalk::{Event, RecordingOptions, RecordingSession};

fn main() -> teamtalk::Result<()> {
    let client = teamtalk::Client::new()?;
    let channel_id = client.get_root_channel_id();

    let options = RecordingOptions::new(
        "recordings/session-{index}.wav",
        AudioFileFormat::AFF_WAVE_FORMAT,
    );
    let mut session = RecordingSession::start_channel(&client, channel_id, options)?;

    loop {
        if let Some((event, message)) = client.poll(100) {
            let _ = session.handle_event(event, &message)?;
            let _ = session.rotate_if_needed()?;
            if matches!(event, Event::ConnectionLost) {
                break;
            }
        }
    }
    let _ = session.stop();
    Ok(())
}
