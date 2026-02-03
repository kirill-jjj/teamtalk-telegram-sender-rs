use teamtalk::client::ffi;
use teamtalk::client::recording::UserRecordingOptions;

#[test]
fn user_recording_options_builder_sets_delay() {
    let options = UserRecordingOptions::new(
        "folder",
        "%username%",
        ffi::AudioFileFormat::AFF_WAVE_FORMAT,
    )
    .with_stop_delay(1500);
    assert_eq!(options.folder, "folder");
    assert_eq!(options.file_vars, "%username%");
    assert_eq!(options.format, ffi::AudioFileFormat::AFF_WAVE_FORMAT);
    assert_eq!(options.stop_delay_ms, 1500);
}
