use teamtalk::{
    RecordingSampleFormat, SilencePolicy, SyncedUserRecordingOptions, SyncedUserRecordingSession,
};

fn main() -> teamtalk::Result<()> {
    let client = teamtalk::Client::new()?;
    let options = SyncedUserRecordingOptions::new("recordings/users")
        .with_format(RecordingSampleFormat::PcmS16Le)
        .with_silence_policy(SilencePolicy::Always);
    let mut recorder = SyncedUserRecordingSession::start(&client, options)?;

    loop {
        if let Some((event, message)) = client.poll(50) {
            recorder.handle_event(&client, event, &message)?;
        }
        recorder.tick()?;
    }
}
