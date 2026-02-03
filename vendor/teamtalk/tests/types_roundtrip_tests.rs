use pretty_assertions::assert_eq;
use proptest::prelude::*;
use teamtalk::client::ffi;
use teamtalk::types::{
    AbusePrevention, AudioCodec, AudioConfig, AudioFormat, AudioPreprocessor, Channel, ChannelType,
    ClientKeepAlive, EncryptionContext, FileTransfer, FileTransferStatus, JitterConfig, OpusCodec,
    SpeexCodec, SpeexDSP, SpeexVBRCodec, Subscriptions, UserAccount, UserAccountBuilder,
    UserGender, UserPresence, UserState, UserStatus, VideoCodec, VideoFormat, WebRTCConfig,
};
use teamtalk::utils::strings::to_string;

proptest! {
    #[test]
    fn subscriptions_raw_roundtrip(raw in any::<u32>()) {
        let subs = Subscriptions::from_raw(raw);
        prop_assert_eq!(subs.raw(), raw);
    }

    #[test]
    fn user_state_raw_roundtrip(raw in any::<u32>()) {
        let state = UserState::from_raw(raw);
        prop_assert_eq!(state.raw(), raw);
    }

    #[test]
    fn channel_type_raw_roundtrip(raw in any::<u32>()) {
        let ct = ChannelType::from_raw(raw);
        prop_assert_eq!(ct.raw(), raw);
    }

    #[test]
    fn user_status_roundtrip(
        presence in 0u8..=2,
        gender in 0u8..=2,
        video in any::<bool>(),
        desktop in any::<bool>(),
        streaming in any::<bool>(),
        media_paused in any::<bool>(),
    ) {
        let presence = match presence {
            1 => UserPresence::Away,
            2 => UserPresence::Question,
            _ => UserPresence::Available,
        };
        let gender = match gender {
            1 => UserGender::Female,
            2 => UserGender::Neutral,
            _ => UserGender::Male,
        };
        let status = UserStatus {
            presence,
            gender,
            video,
            desktop,
            streaming,
            media_paused,
        };
        let bits = status.to_bits();
        let roundtrip = UserStatus::from_bits(bits);
        prop_assert_eq!(roundtrip, status);
    }
}

#[test]
fn user_account_builder_fields() {
    let account = UserAccountBuilder::new("alice")
        .password("secret")
        .user_type(2)
        .rights(7)
        .build();
    assert_eq!(account.username, "alice");
    assert_eq!(account.password, "secret");
    assert_eq!(account.user_type, 2);
    assert_eq!(account.user_rights, 7);
}

#[test]
fn user_account_to_from_ffi() {
    let mut account = UserAccount::builder("bob")
        .password("pass")
        .user_type(3)
        .rights(5)
        .build();
    account.user_data = 42;
    account.abuse_prevent = AbusePrevention {
        commands_limit: 2,
        commands_interval_ms: 500,
    };
    let raw = account.to_ffi();
    let parsed = UserAccount::from(raw);
    assert_eq!(parsed.username, "bob");
    assert_eq!(parsed.user_type, 3);
    assert_eq!(parsed.user_rights, 5);
    assert_eq!(parsed.user_data, 42);
    assert_eq!(parsed.abuse_prevent.commands_limit, 2);
    assert_eq!(parsed.abuse_prevent.commands_interval_ms, 500);
}

#[test]
fn audio_preprocessor_speex_roundtrip() {
    let cfg = SpeexDSP {
        enable_agc: true,
        gain_level: 10,
        max_inc_db_sec: 2,
        max_dec_db_sec: 3,
        max_gain_db: 9,
        enable_denoise: true,
        max_noise_suppress_db: 12,
        enable_aec: true,
        echo_suppress: 7,
        echo_suppress_active: 8,
    };
    let ap = AudioPreprocessor::Speex(cfg);
    let raw = ap.to_ffi();
    let parsed = AudioPreprocessor::from(raw);
    match parsed {
        AudioPreprocessor::Speex(s) => {
            assert_eq!(s.enable_agc, cfg.enable_agc);
            assert_eq!(s.gain_level, cfg.gain_level);
            assert_eq!(s.max_inc_db_sec, cfg.max_inc_db_sec);
            assert_eq!(s.max_dec_db_sec, cfg.max_dec_db_sec);
            assert_eq!(s.max_gain_db, cfg.max_gain_db);
            assert_eq!(s.enable_denoise, cfg.enable_denoise);
            assert_eq!(s.max_noise_suppress_db, cfg.max_noise_suppress_db);
            assert_eq!(s.enable_aec, cfg.enable_aec);
            assert_eq!(s.echo_suppress, cfg.echo_suppress);
            assert_eq!(s.echo_suppress_active, cfg.echo_suppress_active);
        }
        _ => panic!("expected speex preprocessor"),
    }
}

#[test]
fn audio_preprocessor_webrtc_roundtrip() {
    let cfg = WebRTCConfig {
        preamplifier_enable: true,
        preamplifier_gain: 1.5,
        aec_enable: true,
        ns_enable: true,
        ns_level: 2,
        agc2_enable: true,
        agc2_gain_db: 3.2,
    };
    let ap = AudioPreprocessor::WebRTC(cfg);
    let raw = ap.to_ffi();
    let parsed = AudioPreprocessor::from(raw);
    match parsed {
        AudioPreprocessor::WebRTC(w) => {
            assert_eq!(w.preamplifier_enable, cfg.preamplifier_enable);
            assert_eq!(w.preamplifier_gain, cfg.preamplifier_gain);
            assert_eq!(w.aec_enable, cfg.aec_enable);
            assert_eq!(w.ns_enable, cfg.ns_enable);
            assert_eq!(w.ns_level, cfg.ns_level);
            assert_eq!(w.agc2_enable, cfg.agc2_enable);
            assert_eq!(w.agc2_gain_db, cfg.agc2_gain_db);
        }
        _ => panic!("expected webrtc preprocessor"),
    }
}

#[test]
fn jitter_config_roundtrip() {
    let cfg = JitterConfig {
        fixed_delay_ms: 10,
        use_adaptive: true,
        max_adaptive_delay_ms: 50,
        active_adaptive_delay_ms: 12,
    };
    let raw = cfg.to_ffi();
    let parsed = JitterConfig::from(raw);
    assert_eq!(parsed.fixed_delay_ms, cfg.fixed_delay_ms);
    assert_eq!(parsed.use_adaptive, cfg.use_adaptive);
    assert_eq!(parsed.max_adaptive_delay_ms, cfg.max_adaptive_delay_ms);
    assert_eq!(
        parsed.active_adaptive_delay_ms,
        cfg.active_adaptive_delay_ms
    );
}

#[test]
fn video_format_roundtrip() {
    let fmt = VideoFormat {
        width: 640,
        height: 480,
        fps_numerator: 30,
        fps_denominator: 1,
        fourcc: ffi::FourCC::FOURCC_I420,
    };
    let raw = fmt.to_ffi();
    let parsed = VideoFormat::from(raw);
    assert_eq!(parsed.width, fmt.width);
    assert_eq!(parsed.height, fmt.height);
    assert_eq!(parsed.fps_numerator, fmt.fps_numerator);
    assert_eq!(parsed.fps_denominator, fmt.fps_denominator);
    assert_eq!(parsed.fourcc, fmt.fourcc);
}

#[test]
fn video_codec_roundtrip() {
    let codec = VideoCodec {
        bitrate: 64_000,
        deadline: 42,
    };
    let raw = codec.to_ffi();
    let parsed = VideoCodec::from(raw);
    assert_eq!(parsed.bitrate, codec.bitrate);
    assert_eq!(parsed.deadline, codec.deadline);
}

#[test]
fn audio_format_roundtrip() {
    let fmt = AudioFormat {
        format: ffi::AudioFileFormat::AFF_WAVE_FORMAT,
        sample_rate: 48_000,
        channels: 2,
    };
    let raw = fmt.to_ffi();
    let parsed = AudioFormat::from(raw);
    assert_eq!(parsed.format, fmt.format);
    assert_eq!(parsed.sample_rate, fmt.sample_rate);
    assert_eq!(parsed.channels, fmt.channels);
}

#[test]
fn client_keepalive_roundtrip() {
    let keepalive = ClientKeepAlive {
        lost_ms: 1000,
        tcp_interval_ms: 2000,
        udp_interval_ms: 3000,
        udp_rtx_ms: 4000,
        udp_connect_rtx_ms: 5000,
        udp_timeout_ms: 6000,
    };
    let raw = keepalive.to_ffi();
    let parsed = ClientKeepAlive::from(raw);
    assert_eq!(parsed.lost_ms, keepalive.lost_ms);
    assert_eq!(parsed.tcp_interval_ms, keepalive.tcp_interval_ms);
    assert_eq!(parsed.udp_interval_ms, keepalive.udp_interval_ms);
    assert_eq!(parsed.udp_rtx_ms, keepalive.udp_rtx_ms);
    assert_eq!(parsed.udp_connect_rtx_ms, keepalive.udp_connect_rtx_ms);
    assert_eq!(parsed.udp_timeout_ms, keepalive.udp_timeout_ms);
}

#[test]
fn audio_codec_roundtrip_speex() {
    let codec = AudioCodec::Speex(SpeexCodec {
        bandmode: 1,
        quality: 5,
        tx_interval_msec: 40,
        stereo_playback: true,
    });
    let raw = codec.to_ffi();
    let parsed = AudioCodec::from(raw);
    assert_eq!(parsed, codec);
}

#[test]
fn audio_codec_roundtrip_speex_vbr() {
    let codec = AudioCodec::SpeexVBR(SpeexVBRCodec {
        bandmode: 2,
        quality: 4,
        bitrate: 12_000,
        max_bitrate: 24_000,
        dtx: true,
        tx_interval_msec: 60,
        stereo_playback: false,
    });
    let raw = codec.to_ffi();
    let parsed = AudioCodec::from(raw);
    assert_eq!(parsed, codec);
}

#[test]
fn audio_codec_roundtrip_opus() {
    let codec = AudioCodec::Opus(OpusCodec {
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
    let raw = codec.to_ffi();
    let parsed = AudioCodec::from(raw);
    assert_eq!(parsed, codec);
}

#[test]
fn audio_config_roundtrip() {
    let cfg = AudioConfig {
        enable_agc: true,
        gain_level: 7,
    };
    let raw = cfg.to_ffi();
    let parsed = AudioConfig::from(raw);
    assert_eq!(parsed.enable_agc, cfg.enable_agc);
    assert_eq!(parsed.gain_level, cfg.gain_level);
}

#[test]
fn channel_to_from_ffi() {
    let mut channel = Channel::builder("room")
        .topic("topic")
        .channel_type(ChannelType::from_raw(
            ChannelType::HIDDEN | ChannelType::PERMANENT,
        ))
        .max_users(42)
        .build();
    channel.user_data = 7;
    channel.disk_quota = 9;
    channel.transmit_users = vec![(teamtalk::types::UserId(1), 2)];
    channel.transmit_users_queue = vec![teamtalk::types::UserId(3)];
    let raw = channel.to_ffi();
    let parsed = Channel::from(raw);
    assert_eq!(parsed.name, "room");
    assert_eq!(parsed.topic, "topic");
    assert_eq!(parsed.max_users, 42);
    assert_eq!(parsed.channel_type.raw(), channel.channel_type.raw());
    assert_eq!(parsed.user_data, 7);
    assert_eq!(parsed.disk_quota, 9);
    assert_eq!(parsed.transmit_users.len(), 1);
    assert_eq!(parsed.transmit_users_queue.len(), 1);
}
#[test]
fn encryption_context_to_ffi_copies_fields() {
    let ctx = EncryptionContext {
        cert_file: "cert.pem".to_string(),
        key_file: "key.pem".to_string(),
        ca_file: "ca.pem".to_string(),
        ca_dir: "certs".to_string(),
        verify_peer: true,
        verify_client_once: false,
        verify_depth: 3,
    };
    let raw = ctx.to_ffi();
    assert_eq!(to_string(&raw.szCertificateFile), "cert.pem");
    assert_eq!(to_string(&raw.szPrivateKeyFile), "key.pem");
    assert_eq!(to_string(&raw.szCAFile), "ca.pem");
    assert_eq!(to_string(&raw.szCADir), "certs");
    assert_eq!(raw.bVerifyPeer, 1);
    assert_eq!(raw.bVerifyClientOnce, 0);
    assert_eq!(raw.nVerifyDepth, 3);
}

#[test]
fn server_properties_to_ffi_copies_fields() {
    let props = teamtalk::types::ServerProperties {
        name: "srv".to_string(),
        motd: String::new(),
        motd_raw: "motd".to_string(),
        max_users: 0,
        max_login_attempts: 0,
        max_logins_per_ip: 0,
        max_voice_tx: 0,
        max_video_tx: 0,
        max_media_tx: 0,
        max_desktop_tx: 0,
        max_total_tx: 0,
        auto_save: false,
        tcp_port: 0,
        udp_port: 0,
        user_timeout: 0,
        version: String::new(),
        protocol_version: String::new(),
        login_delay: 0,
        access_token: String::new(),
        log_events: 7,
    };
    let raw = props.to_ffi();
    assert_eq!(to_string(&raw.szServerName), "srv");
    assert_eq!(to_string(&raw.szMOTDRaw), "motd");
    assert_eq!(raw.uServerLogEvents, 7);
}

#[test]
fn file_transfer_progress_values() {
    let zero = FileTransfer {
        status: FileTransferStatus::Closed,
        id: teamtalk::types::TransferId(1),
        channel_id: teamtalk::types::ChannelId(2),
        local_path: String::new(),
        remote_name: String::new(),
        size: 0,
        transferred: 0,
        inbound: true,
    };
    assert_eq!(zero.progress(), 0.0);
    let transfer = FileTransfer {
        size: 100,
        transferred: 30,
        ..zero
    };
    assert_eq!(transfer.progress(), 0.3);
}
