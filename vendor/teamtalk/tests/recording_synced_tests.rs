use teamtalk::RecordingSampleFormat;
use teamtalk::client::recording::{SilencePolicy, SyncedUserRecordingOptions};

#[test]
fn synced_user_recording_options_builder_sets_fields() {
    let options = SyncedUserRecordingOptions::new("out")
        .with_format(RecordingSampleFormat::PcmS16Le)
        .with_file_vars("%username%")
        .with_stream_types(123)
        .with_tick_interval(std::time::Duration::from_millis(150))
        .with_default_audio_format(48_000, 2)
        .with_subscribe_audio(false)
        .with_silence_policy(SilencePolicy::OnlyWhileConnected);

    assert_eq!(options.folder, "out");
    assert_eq!(options.file_vars, "%username%".to_string());
    assert_eq!(options.stream_types, 123);
    assert_eq!(options.tick_interval, std::time::Duration::from_millis(150));
    assert_eq!(options.default_sample_rate, Some(48_000));
    assert_eq!(options.default_channels, Some(2));
    assert!(!options.subscribe_audio);
    assert!(matches!(
        options.silence_policy,
        SilencePolicy::OnlyWhileConnected
    ));
}
