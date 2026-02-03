use teamtalk::client::ffi;
use teamtalk::types::{
    AudioCodec, AudioConfig, AudioPreprocessor, Channel, ChannelId, ChannelType, ClientFlags,
    MessageTarget, OpusCodec, SpeexCodec, SpeexVBRCodec, TextMessage, UserGender, UserId,
    UserPresence, UserState, UserStatus,
};

#[test]
fn client_flags_has() {
    let flags = ClientFlags::from_raw(ClientFlags::CONNECTED | ClientFlags::AUTHORIZED);
    assert!(flags.has(ClientFlags::CONNECTED));
    assert!(flags.has(ClientFlags::AUTHORIZED));
    assert!(!flags.has(ClientFlags::TX_VOICE));
}

#[test]
fn message_target_from_text_message() {
    let raw = ffi::TextMessage {
        nFromUserID: 10,
        nToUserID: 20,
        nChannelID: 30,
        nMsgType: ffi::TextMsgType::MSGTYPE_USER,
        ..Default::default()
    };
    let msg = TextMessage::from(raw);
    let target = MessageTarget::from(&msg);
    assert!(matches!(target, MessageTarget::User(UserId(10))));
}

#[test]
fn audio_preprocessor_none_roundtrip() {
    let ap = AudioPreprocessor::None;
    let raw = ap.to_ffi();
    let parsed = AudioPreprocessor::from(raw);
    assert!(matches!(parsed, AudioPreprocessor::None));
}

#[test]
fn channel_builder_defaults() {
    let channel = Channel::builder("room").build();
    assert_eq!(channel.id, ChannelId(0));
    assert_eq!(channel.parent_id, ChannelId(0));
    assert_eq!(channel.channel_type.raw(), ChannelType::DEFAULT);
    assert!(channel.transmit_users.is_empty());
}

#[test]
fn audio_codec_none_roundtrip() {
    let codec = AudioCodec::None;
    let raw = codec.to_ffi();
    let parsed = AudioCodec::from(raw);
    assert!(matches!(parsed, AudioCodec::None));
}

#[test]
fn audio_config_defaults() {
    let cfg = AudioConfig::default();
    assert!(!cfg.enable_agc);
    assert_eq!(cfg.gain_level, 0);
}

#[test]
fn user_state_flags_composition() {
    let state = UserState::from_raw(UserState::VOICE | UserState::VIDEOCAPTURE);
    assert!(state.is_talking());
    assert!(state.has_video());
    assert!(!state.is_muted());
}

#[test]
fn audio_codec_fields_match() {
    let speex = AudioCodec::Speex(SpeexCodec {
        bandmode: 1,
        quality: 3,
        tx_interval_msec: 20,
        stereo_playback: true,
    });
    let speex_vbr = AudioCodec::SpeexVBR(SpeexVBRCodec {
        bandmode: 2,
        quality: 4,
        bitrate: 12_000,
        max_bitrate: 16_000,
        dtx: true,
        tx_interval_msec: 40,
        stereo_playback: false,
    });
    let opus = AudioCodec::Opus(OpusCodec {
        sample_rate: 48_000,
        channels: 2,
        application: 2049,
        complexity: 5,
        fec: false,
        dtx: true,
        bitrate: 64_000,
        vbr: true,
        vbr_constraint: true,
        tx_interval_msec: 20,
        frame_size_msec: 10,
    });
    let speex_parsed = AudioCodec::from(speex.to_ffi());
    let speex_vbr_parsed = AudioCodec::from(speex_vbr.to_ffi());
    let opus_parsed = AudioCodec::from(opus.to_ffi());
    assert_eq!(speex_parsed, speex);
    assert_eq!(speex_vbr_parsed, speex_vbr);
    assert_eq!(opus_parsed, opus);
}

#[test]
fn user_status_gender_precedence() {
    let bits = 0x1000 | 0x100 | 0x200;
    let status = UserStatus::from_bits(bits);
    assert_eq!(status.gender, UserGender::Neutral);
    assert!(status.video);
    assert_eq!(status.presence, UserPresence::Available);
}
