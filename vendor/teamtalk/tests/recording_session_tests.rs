#![cfg(feature = "mock")]

use std::fs;
use std::sync::Arc;
use std::time::Duration;

use teamtalk::client::Client;
use teamtalk::client::backend::MockBackend;
use teamtalk::client::recording::{RecordingOptions, RecordingSession};
use teamtalk::events::Error;
use teamtalk::types::{AudioCodec, Channel, ChannelId};
use teamtalk_sys as ffi;

fn test_channel(id: ChannelId, codec: AudioCodec) -> Channel {
    let mut channel = Channel::builder("test").codec(codec).build();
    channel.id = id;
    channel
}

#[test]
fn recording_session_rotates_on_size() {
    let backend = Arc::new(MockBackend::new());
    backend.set_channel(test_channel(ChannelId(1), AudioCodec::default()));
    backend.set_my_channel_id(ChannelId(1));
    let client = Client::with_backend(backend).expect("client");
    let options =
        RecordingOptions::new("session_{index}.wav", ffi::AudioFileFormat::AFF_WAVE_FORMAT)
            .with_max_size_bytes(1);
    let mut session =
        RecordingSession::start_channel(&client, ChannelId(1), options).expect("session");
    let first_path = session.current_path().expect("path").to_string();
    fs::write(&first_path, [1u8]).expect("write");
    let rotated = session.rotate_if_needed().expect("rotate");
    assert!(rotated);
    assert_eq!(session.segments().len(), 2);
    let second_path = session.current_path().expect("path").to_string();
    assert_ne!(first_path, second_path);
    let _ = fs::remove_file(first_path);
    let _ = fs::remove_file(second_path);
}

#[test]
fn recording_session_pause_resume_adds_segment() {
    let backend = Arc::new(MockBackend::new());
    backend.set_channel(test_channel(ChannelId(2), AudioCodec::default()));
    backend.set_my_channel_id(ChannelId(2));
    let client = Client::with_backend(backend).expect("client");
    let options = RecordingOptions::new("pause_{index}.wav", ffi::AudioFileFormat::AFF_WAVE_FORMAT);
    let mut session =
        RecordingSession::start_channel(&client, ChannelId(2), options).expect("session");
    assert!(session.is_active());
    assert!(session.pause());
    assert!(!session.is_active());
    let before = session.segments().len();
    session.resume().expect("resume");
    assert!(session.is_active());
    assert_eq!(session.segments().len(), before + 1);
}

#[test]
fn start_current_channel_requires_channel() {
    let backend = Arc::new(MockBackend::new());
    backend.set_my_channel_id(ChannelId(0));
    let client = Client::with_backend(backend).expect("client");
    let options =
        RecordingOptions::new("current_{index}.wav", ffi::AudioFileFormat::AFF_WAVE_FORMAT)
            .with_max_duration(Duration::from_millis(1));
    let result = RecordingSession::start_current_channel(&client, options);
    match result {
        Err(Error::CommandFailed { message, .. }) => {
            assert!(message.contains("Not joined"));
        }
        Err(other) => panic!("unexpected error: {other:?}"),
        Ok(_) => panic!("expected error"),
    }
}
