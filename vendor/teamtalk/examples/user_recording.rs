use teamtalk::client::ffi::AudioFileFormat;
use teamtalk::types::UserId;
use teamtalk::{UserRecordingOptions, UserRecordingSession};

fn main() -> teamtalk::Result<()> {
    let client = teamtalk::Client::new()?;
    let options = UserRecordingOptions::new(
        "recordings/users",
        "user-%user_id%-%username%",
        AudioFileFormat::AFF_WAVE_FORMAT,
    );
    let _session = UserRecordingSession::start(&client, UserId(1), options);
    Ok(())
}
