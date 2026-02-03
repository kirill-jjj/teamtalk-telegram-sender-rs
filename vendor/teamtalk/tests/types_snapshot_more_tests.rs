use insta::assert_debug_snapshot;
use teamtalk::types::{AudioCodec, Channel, ChannelType, OpusCodec};

#[test]
fn channel_builder_snapshot() {
    let mut channel = Channel::builder("room")
        .topic("topic")
        .channel_type(ChannelType::from_raw(
            ChannelType::HIDDEN | ChannelType::PERMANENT,
        ))
        .max_users(10)
        .build();
    channel.audio_codec = AudioCodec::Opus(OpusCodec {
        sample_rate: 48_000,
        channels: 2,
        application: 2049,
        complexity: 10,
        fec: true,
        dtx: false,
        bitrate: 64_000,
        vbr: true,
        vbr_constraint: false,
        tx_interval_msec: 20,
        frame_size_msec: 10,
    });
    assert_debug_snapshot!(channel);
}
