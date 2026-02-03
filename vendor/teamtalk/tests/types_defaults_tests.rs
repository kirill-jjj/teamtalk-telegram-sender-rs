use teamtalk::client::ffi;
use teamtalk::types::{
    AudioFormat, AudioPreprocessor, ChannelType, ClientFlags, Subscriptions, VideoFormat,
    WEBRTC_GAIN_MAX,
};

#[test]
fn audio_format_default_values() {
    let fmt = AudioFormat::default();
    assert_eq!(fmt.format, ffi::AudioFileFormat::AFF_NONE);
    assert_eq!(fmt.sample_rate, 0);
    assert_eq!(fmt.channels, 0);
}

#[test]
fn video_format_default_values() {
    let fmt = VideoFormat::default();
    assert_eq!(fmt.width, 0);
    assert_eq!(fmt.height, 0);
    assert_eq!(fmt.fps_numerator, 0);
    assert_eq!(fmt.fps_denominator, 0);
    assert_eq!(fmt.fourcc, ffi::FourCC::FOURCC_NONE);
}

#[test]
fn audio_preprocessor_default_is_none() {
    let ap = AudioPreprocessor::default();
    assert!(matches!(ap, AudioPreprocessor::None));
}

#[test]
fn client_flags_default_is_zero() {
    let flags = ClientFlags::default();
    assert_eq!(flags.raw(), 0);
    assert!(!flags.has(ClientFlags::CONNECTED));
}

#[test]
fn subscriptions_default_is_empty() {
    let subs = Subscriptions::default();
    assert_eq!(subs.raw(), Subscriptions::NONE);
}

#[test]
fn channel_type_default_is_default() {
    let ct = ChannelType::default();
    assert_eq!(ct.raw(), ChannelType::DEFAULT);
}

#[test]
fn webrtc_gain_max_constant() {
    const _: () = assert!(WEBRTC_GAIN_MAX > 0.0);
}
