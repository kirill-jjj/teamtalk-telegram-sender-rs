//! Core data structures and constants used by the SDK.
use teamtalk_sys as ffi;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
/// Strongly typed user id.
pub struct UserId(pub i32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
/// Strongly typed channel id.
pub struct ChannelId(pub i32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
/// Strongly typed remote file id.
pub struct FileId(pub i32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
/// Strongly typed transfer id.
pub struct TransferId(pub i32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
/// Strongly typed command id.
pub struct CommandId(pub i32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
/// Strongly typed client id.
pub struct ClientId(pub u64);

/// Reserved local user id.
pub const LOCAL_USER_ID: UserId = UserId(0);
/// Reserved local transmit user id.
pub const LOCAL_TX_USER_ID: UserId = UserId(4098);
/// Reserved muxed audio user id.
pub const MUXED_USER_ID: UserId = UserId(4097);

impl CommandId {
    pub fn raw(self) -> i32 {
        self.0
    }
}

impl From<i32> for CommandId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// Speex narrowband minimum bitrate.
pub const SPEEX_NB_MIN_BITRATE: i32 = 2150;
/// Speex narrowband maximum bitrate.
pub const SPEEX_NB_MAX_BITRATE: i32 = 24600;
/// Speex wideband minimum bitrate.
pub const SPEEX_WB_MIN_BITRATE: i32 = 3950;
/// Speex wideband maximum bitrate.
pub const SPEEX_WB_MAX_BITRATE: i32 = 42200;
/// Speex ultra-wideband minimum bitrate.
pub const SPEEX_UWB_MIN_BITRATE: i32 = 4150;
/// Speex ultra-wideband maximum bitrate.
pub const SPEEX_UWB_MAX_BITRATE: i32 = 44000;

/// Opus minimum bitrate.
pub const OPUS_MIN_BITRATE: i32 = 6000;
/// Opus maximum bitrate.
pub const OPUS_MAX_BITRATE: i32 = 510000;
/// Opus minimum frame size in ms.
pub const OPUS_MIN_FRAMESIZE: i32 = 2;
/// Opus maximum frame size in ms.
pub const OPUS_MAX_FRAMESIZE: i32 = 60;

/// Maximum WebRTC gain value.
pub const WEBRTC_GAIN_MAX: f32 = 49.9;

/// Desktop input: left mouse button.
pub const MOUSE_LEFT: u32 = 0x1000;
/// Desktop input: right mouse button.
pub const MOUSE_RIGHT: u32 = 0x1001;
/// Desktop input: middle mouse button.
pub const MOUSE_MIDDLE: u32 = 0x1002;

/// TeamTalk fixed string length.
pub const TT_STRLEN: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Subscription mask for user streams.
pub struct Subscriptions(u32);

impl Subscriptions {
    /// No subscriptions.
    pub const NONE: u32 = 0x00000000;
    /// User messages subscription flag.
    pub const USER_MSG: u32 = 0x00000001;
    /// Channel messages subscription flag.
    pub const CHANNEL_MSG: u32 = 0x00000002;
    /// Broadcast messages subscription flag.
    pub const BROADCAST_MSG: u32 = 0x00000004;
    /// Custom messages subscription flag.
    pub const CUSTOM_MSG: u32 = 0x00000008;
    /// Voice subscription flag.
    pub const VOICE: u32 = 0x00000010;
    /// Video capture subscription flag.
    pub const VIDEOCAPTURE: u32 = 0x00000020;
    /// Desktop subscription flag.
    pub const DESKTOP: u32 = 0x00000040;
    /// Desktop input subscription flag.
    pub const DESKTOPINPUT: u32 = 0x00000080;
    /// Media file subscription flag.
    pub const MEDIAFILE: u32 = 0x00000100;
    /// Intercept user messages flag.
    pub const INTERCEPT_USER_MSG: u32 = 0x00010000;
    /// Intercept channel messages flag.
    pub const INTERCEPT_CHANNEL_MSG: u32 = 0x00020000;
    /// Intercept custom messages flag.
    pub const INTERCEPT_CUSTOM_MSG: u32 = 0x00080000;
    /// Intercept voice streams flag.
    pub const INTERCEPT_VOICE: u32 = 0x00100000;
    /// Intercept video capture streams flag.
    pub const INTERCEPT_VIDEOCAPTURE: u32 = 0x00200000;
    /// Intercept desktop streams flag.
    pub const INTERCEPT_DESKTOP: u32 = 0x00400000;
    /// Intercept media file streams flag.
    pub const INTERCEPT_MEDIAFILE: u32 = 0x01000000;
    /// All subscription flags.
    pub const ALL: u32 = 0xFFFFFFFF;

    /// Creates an empty subscription mask.
    pub fn new() -> Self {
        Self(Self::NONE)
    }
    /// Creates a subscription mask with all flags set.
    pub fn all() -> Self {
        Self(Self::ALL)
    }
    /// Creates a subscription mask with voice and media file streams.
    pub fn all_audio() -> Self {
        Self(Self::VOICE | Self::MEDIAFILE)
    }
    /// Creates a subscription mask with user, channel, broadcast, and custom messages.
    pub fn all_text() -> Self {
        Self(Self::USER_MSG | Self::CHANNEL_MSG | Self::BROADCAST_MSG | Self::CUSTOM_MSG)
    }
    /// Creates a subscription mask with desktop control streams.
    pub fn all_control() -> Self {
        Self(Self::DESKTOP | Self::DESKTOPINPUT)
    }
    /// Creates a subscription mask from raw bits.
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }
    /// Adds a subscription flag.
    pub fn add(&mut self, flag: u32) {
        self.0 |= flag;
    }
    /// Removes a subscription flag.
    pub fn remove(&mut self, flag: u32) {
        self.0 &= !flag;
    }
    /// Returns true if the flag is present.
    pub fn has(&self, flag: u32) -> bool {
        (self.0 & flag) != 0
    }
    /// Returns the raw bitmask.
    pub fn raw(&self) -> u32 {
        self.0
    }
}

/// Bitmask of user state flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UserState(u32);

impl UserState {
    /// No user state flags set.
    pub const NONE: u32 = 0x00000000;
    /// User is talking.
    pub const VOICE: u32 = 0x00000001;
    /// User voice is muted.
    pub const MUTE_VOICE: u32 = 0x00000002;
    /// User media file is muted.
    pub const MUTE_MEDIAFILE: u32 = 0x00000004;
    /// User has desktop sharing active.
    pub const DESKTOP: u32 = 0x00000008;
    /// User has video capture active.
    pub const VIDEOCAPTURE: u32 = 0x00000010;
    /// User is streaming media file audio.
    pub const MEDIAFILE_AUDIO: u32 = 0x00000020;
    /// User is streaming media file video.
    pub const MEDIAFILE_VIDEO: u32 = 0x00000040;

    /// Creates a user state from raw bits.
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }
    /// Returns true if the user is talking.
    pub fn is_talking(&self) -> bool {
        (self.0 & Self::VOICE) != 0
    }
    /// Returns true if the user is muted.
    pub fn is_muted(&self) -> bool {
        (self.0 & Self::MUTE_VOICE) != 0
    }
    /// Returns true if the user has video.
    pub fn has_video(&self) -> bool {
        (self.0 & Self::VIDEOCAPTURE) != 0
    }
    /// Returns the raw bitmask.
    pub fn raw(&self) -> u32 {
        self.0
    }
}

/// Channel type bitmask.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChannelType(u32);

impl ChannelType {
    /// Default channel type.
    pub const DEFAULT: u32 = 0x0000;
    /// Permanent channel flag.
    pub const PERMANENT: u32 = 0x0001;
    /// Solo transmit flag.
    pub const SOLO_TRANSMIT: u32 = 0x0002;
    /// Classroom flag.
    pub const CLASSROOM: u32 = 0x0004;
    /// Operator receive-only flag.
    pub const OPERATOR_RECVONLY: u32 = 0x0008;
    /// Disable voice activation flag.
    pub const NO_VOICEACTIVATION: u32 = 0x0010;
    /// Disable recording flag.
    pub const NO_RECORDING: u32 = 0x0020;
    /// Hidden channel flag.
    pub const HIDDEN: u32 = 0x0040;

    /// Creates from raw bits.
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }
    /// Returns the raw bitmask.
    pub fn raw(&self) -> u32 {
        self.0
    }
}

/// Presence status of a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UserPresence {
    #[default]
    Available,
    Away,
    Question,
}

/// Gender metadata for a user profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UserGender {
    Male,
    Female,
    #[default]
    Neutral,
}

/// User status flags and presence metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UserStatus {
    pub presence: UserPresence,
    pub gender: UserGender,
    pub video: bool,
    pub desktop: bool,
    pub streaming: bool,
    pub media_paused: bool,
}

impl UserStatus {
    /// Creates a status from raw bit flags.
    pub fn from_bits(bits: u32) -> Self {
        let presence = match bits & 0xFF {
            1 => UserPresence::Away,
            2 => UserPresence::Question,
            _ => UserPresence::Available,
        };
        let gender = if (bits & 0x1000) != 0 {
            UserGender::Neutral
        } else if (bits & 0x100) != 0 {
            UserGender::Female
        } else {
            UserGender::Male
        };
        Self {
            presence,
            gender,
            video: (bits & 0x200) != 0,
            desktop: (bits & 0x400) != 0,
            streaming: (bits & 0x800) != 0,
            media_paused: (bits & 0x2000) != 0,
        }
    }
    /// Converts the status into raw bit flags.
    pub fn to_bits(&self) -> u32 {
        let mut bits = match self.presence {
            UserPresence::Available => 0,
            UserPresence::Away => 1,
            UserPresence::Question => 2,
        };
        match self.gender {
            UserGender::Male => {}
            UserGender::Female => bits |= 0x100,
            UserGender::Neutral => bits |= 0x1000,
        }
        if self.video {
            bits |= 0x200;
        }
        if self.desktop {
            bits |= 0x400;
        }
        if self.streaming {
            bits |= 0x800;
        }
        if self.media_paused {
            bits |= 0x2000;
        }
        bits
    }
}

/// Speex DSP preprocessing settings.
#[derive(Debug, Clone, Copy, Default)]
pub struct SpeexDSP {
    pub enable_agc: bool,
    pub gain_level: i32,
    pub max_inc_db_sec: i32,
    pub max_dec_db_sec: i32,
    pub max_gain_db: i32,
    pub enable_denoise: bool,
    pub max_noise_suppress_db: i32,
    pub enable_aec: bool,
    pub echo_suppress: i32,
    pub echo_suppress_active: i32,
}

impl From<ffi::SpeexDSP> for SpeexDSP {
    fn from(d: ffi::SpeexDSP) -> Self {
        Self {
            enable_agc: d.bEnableAGC != 0,
            gain_level: d.nGainLevel,
            max_inc_db_sec: d.nMaxIncDBSec,
            max_dec_db_sec: d.nMaxDecDBSec,
            max_gain_db: d.nMaxGainDB,
            enable_denoise: d.bEnableDenoise != 0,
            max_noise_suppress_db: d.nMaxNoiseSuppressDB,
            enable_aec: d.bEnableEchoCancellation != 0,
            echo_suppress: d.nEchoSuppress,
            echo_suppress_active: d.nEchoSuppressActive,
        }
    }
}

impl SpeexDSP {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::SpeexDSP {
        ffi::SpeexDSP {
            bEnableAGC: self.enable_agc as i32,
            nGainLevel: self.gain_level,
            nMaxIncDBSec: self.max_inc_db_sec,
            nMaxDecDBSec: self.max_dec_db_sec,
            nMaxGainDB: self.max_gain_db,
            bEnableDenoise: self.enable_denoise as i32,
            nMaxNoiseSuppressDB: self.max_noise_suppress_db,
            bEnableEchoCancellation: self.enable_aec as i32,
            nEchoSuppress: self.echo_suppress,
            nEchoSuppressActive: self.echo_suppress_active,
        }
    }
}

/// WebRTC audio preprocessing settings.
#[derive(Debug, Clone, Copy, Default)]
pub struct WebRTCConfig {
    pub preamplifier_enable: bool,
    pub preamplifier_gain: f32,
    pub aec_enable: bool,
    pub ns_enable: bool,
    pub ns_level: i32,
    pub agc2_enable: bool,
    pub agc2_gain_db: f32,
}

impl From<ffi::WebRTCAudioPreprocessor> for WebRTCConfig {
    fn from(w: ffi::WebRTCAudioPreprocessor) -> Self {
        Self {
            preamplifier_enable: w.preamplifier.bEnable != 0,
            preamplifier_gain: w.preamplifier.fFixedGainFactor,
            aec_enable: w.echocanceller.bEnable != 0,
            ns_enable: w.noisesuppression.bEnable != 0,
            ns_level: w.noisesuppression.nLevel,
            agc2_enable: w.gaincontroller2.bEnable != 0,
            agc2_gain_db: w.gaincontroller2.fixeddigital.fGainDB,
        }
    }
}

impl WebRTCConfig {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::WebRTCAudioPreprocessor {
        let mut raw = ffi::WebRTCAudioPreprocessor::default();
        raw.preamplifier.bEnable = self.preamplifier_enable as i32;
        raw.preamplifier.fFixedGainFactor = self.preamplifier_gain;
        raw.echocanceller.bEnable = self.aec_enable as i32;
        raw.noisesuppression.bEnable = self.ns_enable as i32;
        raw.noisesuppression.nLevel = self.ns_level;
        raw.gaincontroller2.bEnable = self.agc2_enable as i32;
        raw.gaincontroller2.fixeddigital.fGainDB = self.agc2_gain_db;
        raw
    }
}

/// Audio preprocessing configuration.
#[derive(Debug, Clone, Copy, Default)]
pub enum AudioPreprocessor {
    #[default]
    None,
    Speex(SpeexDSP),
    WebRTC(WebRTCConfig),
}

impl From<ffi::AudioPreprocessor> for AudioPreprocessor {
    fn from(p: ffi::AudioPreprocessor) -> Self {
        match p.nPreprocessor {
            ffi::AudioPreprocessorType::SPEEXDSP_AUDIOPREPROCESSOR => {
                Self::Speex(SpeexDSP::from(unsafe { p.__bindgen_anon_1.speexdsp }))
            }
            ffi::AudioPreprocessorType::WEBRTC_AUDIOPREPROCESSOR => {
                Self::WebRTC(WebRTCConfig::from(unsafe { p.__bindgen_anon_1.webrtc }))
            }
            _ => Self::None,
        }
    }
}

impl AudioPreprocessor {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::AudioPreprocessor {
        let mut raw = ffi::AudioPreprocessor::default();
        match self {
            Self::None => raw.nPreprocessor = ffi::AudioPreprocessorType::NO_AUDIOPREPROCESSOR,
            Self::Speex(s) => {
                raw.nPreprocessor = ffi::AudioPreprocessorType::SPEEXDSP_AUDIOPREPROCESSOR;
                raw.__bindgen_anon_1.speexdsp = s.to_ffi();
            }
            Self::WebRTC(w) => {
                raw.nPreprocessor = ffi::AudioPreprocessorType::WEBRTC_AUDIOPREPROCESSOR;
                raw.__bindgen_anon_1.webrtc = w.to_ffi();
            }
        }
        raw
    }
}

/// Jitter control configuration.
#[derive(Debug, Clone, Copy, Default)]
pub struct JitterConfig {
    pub fixed_delay_ms: i32,
    pub use_adaptive: bool,
    pub max_adaptive_delay_ms: i32,
    pub active_adaptive_delay_ms: i32,
}

impl From<ffi::JitterConfig> for JitterConfig {
    fn from(c: ffi::JitterConfig) -> Self {
        Self {
            fixed_delay_ms: c.nFixedDelayMSec,
            use_adaptive: c.bUseAdativeDejitter != 0,
            max_adaptive_delay_ms: c.nMaxAdaptiveDelayMSec,
            active_adaptive_delay_ms: c.nActiveAdaptiveDelayMSec,
        }
    }
}

impl JitterConfig {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::JitterConfig {
        ffi::JitterConfig {
            nFixedDelayMSec: self.fixed_delay_ms,
            bUseAdativeDejitter: self.use_adaptive as i32,
            nMaxAdaptiveDelayMSec: self.max_adaptive_delay_ms,
            nActiveAdaptiveDelayMSec: self.active_adaptive_delay_ms,
        }
    }
}

/// Video format configuration.
#[derive(Debug, Clone, Copy)]
pub struct VideoFormat {
    pub width: i32,
    pub height: i32,
    pub fps_numerator: i32,
    pub fps_denominator: i32,
    pub fourcc: ffi::FourCC,
}

impl Default for VideoFormat {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            fps_numerator: 0,
            fps_denominator: 0,
            fourcc: ffi::FourCC::FOURCC_NONE,
        }
    }
}

impl From<ffi::VideoFormat> for VideoFormat {
    fn from(f: ffi::VideoFormat) -> Self {
        Self {
            width: f.nWidth,
            height: f.nHeight,
            fps_numerator: f.nFPS_Numerator,
            fps_denominator: f.nFPS_Denominator,
            fourcc: f.picFourCC,
        }
    }
}

impl VideoFormat {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::VideoFormat {
        ffi::VideoFormat {
            nWidth: self.width,
            nHeight: self.height,
            nFPS_Numerator: self.fps_numerator,
            nFPS_Denominator: self.fps_denominator,
            picFourCC: self.fourcc,
        }
    }
}

/// Video codec configuration.
#[derive(Debug, Clone, Copy, Default)]
pub struct VideoCodec {
    pub bitrate: i32,
    pub deadline: u32,
}

impl From<ffi::VideoCodec> for VideoCodec {
    fn from(c: ffi::VideoCodec) -> Self {
        Self {
            bitrate: unsafe {
                c.__bindgen_anon_1
                    .webm_vp8
                    .__bindgen_anon_1
                    .nRcTargetBitrate
            },
            deadline: unsafe { c.__bindgen_anon_1.webm_vp8.nEncodeDeadline },
        }
    }
}

impl VideoCodec {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::VideoCodec {
        let mut raw = ffi::VideoCodec {
            nCodec: ffi::Codec::WEBM_VP8_CODEC,
            ..Default::default()
        };
        raw.__bindgen_anon_1.webm_vp8.nEncodeDeadline = self.deadline;
        raw.__bindgen_anon_1
            .webm_vp8
            .__bindgen_anon_1
            .nRcTargetBitrate = self.bitrate;
        raw
    }
}

/// TLS encryption context settings.
#[derive(Debug, Clone, Default)]
pub struct EncryptionContext {
    pub cert_file: String,
    pub key_file: String,
    pub ca_file: String,
    pub ca_dir: String,
    pub verify_peer: bool,
    pub verify_client_once: bool,
    pub verify_depth: i32,
}

impl EncryptionContext {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::EncryptionContext {
        let mut raw = ffi::EncryptionContext::default();
        let cert = crate::utils::ToTT::tt(&self.cert_file);
        let key = crate::utils::ToTT::tt(&self.key_file);
        let ca = crate::utils::ToTT::tt(&self.ca_file);
        let cadir = crate::utils::ToTT::tt(&self.ca_dir);
        unsafe {
            let n_len = cert.len().min(511);
            std::ptr::copy_nonoverlapping(cert.as_ptr(), raw.szCertificateFile.as_mut_ptr(), n_len);
            let k_len = key.len().min(511);
            std::ptr::copy_nonoverlapping(key.as_ptr(), raw.szPrivateKeyFile.as_mut_ptr(), k_len);
            let ca_len = ca.len().min(511);
            std::ptr::copy_nonoverlapping(ca.as_ptr(), raw.szCAFile.as_mut_ptr(), ca_len);
            let cd_len = cadir.len().min(511);
            std::ptr::copy_nonoverlapping(cadir.as_ptr(), raw.szCADir.as_mut_ptr(), cd_len);
        }
        raw.bVerifyPeer = self.verify_peer as i32;
        raw.bVerifyClientOnce = self.verify_client_once as i32;
        raw.nVerifyDepth = self.verify_depth;
        raw
    }
}

/// Keep-alive configuration for client connections.
#[derive(Debug, Clone, Copy, Default)]
pub struct ClientKeepAlive {
    pub lost_ms: i32,
    pub tcp_interval_ms: i32,
    pub udp_interval_ms: i32,
    pub udp_rtx_ms: i32,
    pub udp_connect_rtx_ms: i32,
    pub udp_timeout_ms: i32,
}

impl From<ffi::ClientKeepAlive> for ClientKeepAlive {
    fn from(c: ffi::ClientKeepAlive) -> Self {
        Self {
            lost_ms: c.nConnectionLostMSec,
            tcp_interval_ms: c.nTcpKeepAliveIntervalMSec,
            udp_interval_ms: c.nUdpKeepAliveIntervalMSec,
            udp_rtx_ms: c.nUdpKeepAliveRTXMSec,
            udp_connect_rtx_ms: c.nUdpConnectRTXMSec,
            udp_timeout_ms: c.nUdpConnectTimeoutMSec,
        }
    }
}

impl ClientKeepAlive {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::ClientKeepAlive {
        ffi::ClientKeepAlive {
            nConnectionLostMSec: self.lost_ms,
            nTcpKeepAliveIntervalMSec: self.tcp_interval_ms,
            nUdpKeepAliveIntervalMSec: self.udp_interval_ms,
            nUdpKeepAliveRTXMSec: self.udp_rtx_ms,
            nUdpConnectRTXMSec: self.udp_connect_rtx_ms,
            nUdpConnectTimeoutMSec: self.udp_timeout_ms,
        }
    }
}

/// Abuse prevention configuration.
#[derive(Debug, Clone, Copy, Default)]
pub struct AbusePrevention {
    pub commands_limit: i32,
    pub commands_interval_ms: i32,
}

impl From<ffi::AbusePrevention> for AbusePrevention {
    fn from(a: ffi::AbusePrevention) -> Self {
        Self {
            commands_limit: a.nCommandsLimit,
            commands_interval_ms: a.nCommandsIntervalMSec,
        }
    }
}

impl AbusePrevention {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::AbusePrevention {
        ffi::AbusePrevention {
            nCommandsLimit: self.commands_limit,
            nCommandsIntervalMSec: self.commands_interval_ms,
        }
    }
}

/// Audio format description.
#[derive(Debug, Clone, Copy)]
pub struct AudioFormat {
    pub format: ffi::AudioFileFormat,
    pub sample_rate: i32,
    pub channels: i32,
}

impl Default for AudioFormat {
    fn default() -> Self {
        Self {
            format: ffi::AudioFileFormat::AFF_NONE,
            sample_rate: 0,
            channels: 0,
        }
    }
}

impl From<ffi::AudioFormat> for AudioFormat {
    fn from(f: ffi::AudioFormat) -> Self {
        Self {
            format: f.nAudioFmt,
            sample_rate: f.nSampleRate,
            channels: f.nChannels,
        }
    }
}

impl AudioFormat {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::AudioFormat {
        ffi::AudioFormat {
            nAudioFmt: self.format,
            nSampleRate: self.sample_rate,
            nChannels: self.channels,
        }
    }
}

/// Video frame metadata.
pub struct VideoFrame {
    pub width: i32,
    pub height: i32,
    pub stream_id: i32,
    pub key_frame: bool,
    pub buf: *mut std::ffi::c_void,
    pub buf_len: i32,
}

impl From<ffi::VideoFrame> for VideoFrame {
    fn from(f: ffi::VideoFrame) -> Self {
        Self {
            width: f.nWidth,
            height: f.nHeight,
            stream_id: f.nStreamID,
            key_frame: f.bKeyFrame != 0,
            buf: f.frameBuffer,
            buf_len: f.nFrameBufferSize,
        }
    }
}

/// Audio input progress information.
#[derive(Debug, Clone, Copy, Default)]
pub struct AudioInputProgress {
    pub stream_id: i32,
    pub queue_ms: u32,
    pub elapsed_ms: u32,
}

impl From<ffi::AudioInputProgress> for AudioInputProgress {
    fn from(p: ffi::AudioInputProgress) -> Self {
        Self {
            stream_id: p.nStreamID,
            queue_ms: p.uQueueMSec,
            elapsed_ms: p.uElapsedMSec,
        }
    }
}

/// SDK error message payload.
#[derive(Debug, Clone, Default)]
pub struct ErrorMessage {
    pub code: i32,
    pub message: String,
}

impl From<ffi::ClientErrorMsg> for ErrorMessage {
    fn from(e: ffi::ClientErrorMsg) -> Self {
        Self {
            code: e.nErrorNo,
            message: crate::utils::strings::to_string(&e.szErrorMsg),
        }
    }
}

/// User state snapshot.
#[derive(Debug, Clone, Default)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub nickname: String,
    pub user_data: i32,
    pub user_type: u32,
    pub ip_address: String,
    pub version: u32,
    pub channel_id: ChannelId,
    pub status: UserStatus,
    pub status_msg: String,
    pub state: UserState,
    pub local_subscriptions: Subscriptions,
    pub peer_subscriptions: Subscriptions,
    pub media_storage_dir: String,
    pub volume_voice: i32,
    pub volume_media: i32,
    pub stopped_delay_voice: i32,
    pub stopped_delay_media: i32,
    pub sound_position_voice: [f32; 3],
    pub sound_position_media: [f32; 3],
    pub stereo_playback_voice: [bool; 2],
    pub stereo_playback_media: [bool; 2],
    pub buf_ms_voice: i32,
    pub buf_ms_media: i32,
    pub active_adaptive_delay: i32,
    pub client_name: String,
}

impl From<ffi::User> for User {
    fn from(u: ffi::User) -> Self {
        Self {
            id: UserId(u.nUserID),
            username: crate::utils::strings::to_string(&u.szUsername),
            nickname: crate::utils::strings::to_string(&u.szNickname),
            user_data: u.nUserData,
            user_type: u.uUserType,
            ip_address: crate::utils::strings::to_string(&u.szIPAddress),
            version: u.uVersion,
            channel_id: ChannelId(u.nChannelID),
            status: UserStatus::from_bits(u.nStatusMode as u32),
            status_msg: crate::utils::strings::to_string(&u.szStatusMsg),
            state: UserState::from_raw(u.uUserState),
            local_subscriptions: Subscriptions::from_raw(u.uLocalSubscriptions),
            peer_subscriptions: Subscriptions::from_raw(u.uPeerSubscriptions),
            media_storage_dir: crate::utils::strings::to_string(&u.szMediaStorageDir),
            volume_voice: u.nVolumeVoice,
            volume_media: u.nVolumeMediaFile,
            stopped_delay_voice: u.nStoppedDelayVoice,
            stopped_delay_media: u.nStoppedDelayMediaFile,
            sound_position_voice: u.soundPositionVoice,
            sound_position_media: u.soundPositionMediaFile,
            stereo_playback_voice: [u.stereoPlaybackVoice[0] != 0, u.stereoPlaybackVoice[1] != 0],
            stereo_playback_media: [
                u.stereoPlaybackMediaFile[0] != 0,
                u.stereoPlaybackMediaFile[1] != 0,
            ],
            buf_ms_voice: u.nBufferMSecVoice,
            buf_ms_media: u.nBufferMSecMediaFile,
            active_adaptive_delay: u.nActiveAdaptiveDelayMSec,
            client_name: crate::utils::strings::to_string(&u.szClientName),
        }
    }
}

/// Speex audio codec configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpeexCodec {
    pub bandmode: i32,
    pub quality: i32,
    pub tx_interval_msec: i32,
    pub stereo_playback: bool,
}

impl From<ffi::SpeexCodec> for SpeexCodec {
    fn from(c: ffi::SpeexCodec) -> Self {
        Self {
            bandmode: c.nBandmode,
            quality: c.nQuality,
            tx_interval_msec: c.nTxIntervalMSec,
            stereo_playback: c.bStereoPlayback != 0,
        }
    }
}

impl SpeexCodec {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::SpeexCodec {
        ffi::SpeexCodec {
            nBandmode: self.bandmode,
            nQuality: self.quality,
            nTxIntervalMSec: self.tx_interval_msec,
            bStereoPlayback: self.stereo_playback as i32,
        }
    }
}

/// Speex VBR audio codec configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpeexVBRCodec {
    pub bandmode: i32,
    pub quality: i32,
    pub bitrate: i32,
    pub max_bitrate: i32,
    pub dtx: bool,
    pub tx_interval_msec: i32,
    pub stereo_playback: bool,
}

impl From<ffi::SpeexVBRCodec> for SpeexVBRCodec {
    fn from(c: ffi::SpeexVBRCodec) -> Self {
        Self {
            bandmode: c.nBandmode,
            quality: c.nQuality,
            bitrate: c.nBitRate,
            max_bitrate: c.nMaxBitRate,
            dtx: c.bDTX != 0,
            tx_interval_msec: c.nTxIntervalMSec,
            stereo_playback: c.bStereoPlayback != 0,
        }
    }
}

impl SpeexVBRCodec {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::SpeexVBRCodec {
        ffi::SpeexVBRCodec {
            nBandmode: self.bandmode,
            nQuality: self.quality,
            nBitRate: self.bitrate,
            nMaxBitRate: self.max_bitrate,
            bDTX: self.dtx as i32,
            nTxIntervalMSec: self.tx_interval_msec,
            bStereoPlayback: self.stereo_playback as i32,
        }
    }
}

/// Opus audio codec configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpusCodec {
    pub sample_rate: i32,
    pub channels: i32,
    pub application: i32,
    pub complexity: i32,
    pub fec: bool,
    pub dtx: bool,
    pub bitrate: i32,
    pub vbr: bool,
    pub vbr_constraint: bool,
    pub tx_interval_msec: i32,
    pub frame_size_msec: i32,
}

impl From<ffi::OpusCodec> for OpusCodec {
    fn from(c: ffi::OpusCodec) -> Self {
        Self {
            sample_rate: c.nSampleRate,
            channels: c.nChannels,
            application: c.nApplication,
            complexity: c.nComplexity,
            fec: c.bFEC != 0,
            dtx: c.bDTX != 0,
            bitrate: c.nBitRate,
            vbr: c.bVBR != 0,
            vbr_constraint: c.bVBRConstraint != 0,
            tx_interval_msec: c.nTxIntervalMSec,
            frame_size_msec: c.nFrameSizeMSec,
        }
    }
}

impl OpusCodec {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::OpusCodec {
        ffi::OpusCodec {
            nSampleRate: self.sample_rate,
            nChannels: self.channels,
            nApplication: self.application,
            nComplexity: self.complexity,
            bFEC: self.fec as i32,
            bDTX: self.dtx as i32,
            nBitRate: self.bitrate,
            bVBR: self.vbr as i32,
            bVBRConstraint: self.vbr_constraint as i32,
            nTxIntervalMSec: self.tx_interval_msec,
            nFrameSizeMSec: self.frame_size_msec,
        }
    }
}

/// Audio codec selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AudioCodec {
    #[default]
    None,
    Speex(SpeexCodec),
    SpeexVBR(SpeexVBRCodec),
    Opus(OpusCodec),
}

impl From<ffi::AudioCodec> for AudioCodec {
    fn from(c: ffi::AudioCodec) -> Self {
        match c.nCodec {
            ffi::Codec::SPEEX_CODEC => unsafe {
                Self::Speex(SpeexCodec::from(c.__bindgen_anon_1.speex))
            },
            ffi::Codec::SPEEX_VBR_CODEC => unsafe {
                Self::SpeexVBR(SpeexVBRCodec::from(c.__bindgen_anon_1.speex_vbr))
            },
            ffi::Codec::OPUS_CODEC => unsafe {
                Self::Opus(OpusCodec::from(c.__bindgen_anon_1.opus))
            },
            _ => Self::None,
        }
    }
}

impl AudioCodec {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::AudioCodec {
        let mut raw = ffi::AudioCodec::default();
        match self {
            Self::None => raw.nCodec = ffi::Codec::NO_CODEC,
            Self::Speex(c) => {
                raw.nCodec = ffi::Codec::SPEEX_CODEC;
                raw.__bindgen_anon_1.speex = c.to_ffi();
            }
            Self::SpeexVBR(c) => {
                raw.nCodec = ffi::Codec::SPEEX_VBR_CODEC;
                raw.__bindgen_anon_1.speex_vbr = c.to_ffi();
            }
            Self::Opus(c) => {
                raw.nCodec = ffi::Codec::OPUS_CODEC;
                raw.__bindgen_anon_1.opus = c.to_ffi();
            }
        }
        raw
    }
}

/// Audio input configuration.
#[derive(Debug, Clone, Copy, Default)]
pub struct AudioConfig {
    pub enable_agc: bool,
    pub gain_level: i32,
}

impl From<ffi::AudioConfig> for AudioConfig {
    fn from(c: ffi::AudioConfig) -> Self {
        Self {
            enable_agc: c.bEnableAGC != 0,
            gain_level: c.nGainLevel,
        }
    }
}

impl AudioConfig {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::AudioConfig {
        ffi::AudioConfig {
            bEnableAGC: self.enable_agc as i32,
            nGainLevel: self.gain_level,
        }
    }
}

/// Channel definition and configuration.
#[derive(Debug, Clone)]
pub struct Channel {
    pub id: ChannelId,
    pub parent_id: ChannelId,
    pub name: String,
    pub topic: String,
    pub channel_type: ChannelType,
    pub has_password: bool,
    pub user_data: i32,
    pub disk_quota: i64,
    pub max_users: i32,
    pub audio_codec: AudioCodec,
    pub audio_cfg: AudioConfig,
    pub transmit_users: Vec<(UserId, u32)>,
    pub transmit_users_queue: Vec<UserId>,
    pub queue_delay_ms: i32,
    pub timeout_voice_ms: i32,
    pub timeout_media_ms: i32,
}

impl Channel {
    /// Creates a channel builder with a name.
    pub fn builder(name: &str) -> ChannelBuilder {
        ChannelBuilder::new(name)
    }
}

/// Builder for channel configuration.
pub struct ChannelBuilder {
    inner: Channel,
}

impl ChannelBuilder {
    /// Creates a new builder with the channel name.
    pub fn new(name: &str) -> Self {
        Self {
            inner: Channel {
                id: ChannelId(0),
                parent_id: ChannelId(0),
                name: name.to_string(),
                topic: String::new(),
                channel_type: ChannelType::default(),
                has_password: false,
                user_data: 0,
                disk_quota: 0,
                max_users: 0,
                audio_codec: AudioCodec::default(),
                audio_cfg: AudioConfig::default(),
                transmit_users: Vec::new(),
                transmit_users_queue: Vec::new(),
                queue_delay_ms: 0,
                timeout_voice_ms: 0,
                timeout_media_ms: 0,
            },
        }
    }

    /// Sets the parent channel id.
    pub fn parent(mut self, id: ChannelId) -> Self {
        self.inner.parent_id = id;
        self
    }

    /// Sets the channel topic.
    pub fn topic(mut self, topic: &str) -> Self {
        self.inner.topic = topic.to_string();
        self
    }

    /// Sets the channel type flags.
    pub fn channel_type(mut self, t: ChannelType) -> Self {
        self.inner.channel_type = t;
        self
    }

    /// Sets the maximum number of users.
    pub fn max_users(mut self, max: i32) -> Self {
        self.inner.max_users = max;
        self
    }

    /// Sets the audio codec configuration.
    pub fn codec(mut self, codec: AudioCodec) -> Self {
        self.inner.audio_codec = codec;
        self
    }

    /// Builds the channel.
    pub fn build(self) -> Channel {
        self.inner
    }
}

impl From<ffi::Channel> for Channel {
    fn from(c: ffi::Channel) -> Self {
        let mut transmit_users = Vec::new();
        for i in 0..128 {
            let id = c.transmitUsers[i][0];
            if id == 0 {
                break;
            }
            transmit_users.push((UserId(id), c.transmitUsers[i][1] as u32));
        }
        let transmit_users_queue = c
            .transmitUsersQueue
            .iter()
            .take_while(|&&id| id != 0)
            .map(|&id| UserId(id))
            .collect();
        Self {
            id: ChannelId(c.nChannelID),
            parent_id: ChannelId(c.nParentID),
            name: crate::utils::strings::to_string(&c.szName),
            topic: crate::utils::strings::to_string(&c.szTopic),
            channel_type: ChannelType::from_raw(c.uChannelType),
            has_password: c.bPassword != 0,
            user_data: c.nUserData,
            disk_quota: c.nDiskQuota,
            max_users: c.nMaxUsers,
            audio_codec: AudioCodec::from(c.audiocodec),
            audio_cfg: AudioConfig::from(c.audiocfg),
            transmit_users,
            transmit_users_queue,
            queue_delay_ms: c.nTransmitUsersQueueDelayMSec,
            timeout_voice_ms: c.nTimeOutTimerVoiceMSec,
            timeout_media_ms: c.nTimeOutTimerMediaFileMSec,
        }
    }
}

impl Channel {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::Channel {
        let mut raw = ffi::Channel {
            nChannelID: self.id.0,
            nParentID: self.parent_id.0,
            ..Default::default()
        };
        let n = crate::utils::ToTT::tt(&self.name);
        let t = crate::utils::ToTT::tt(&self.topic);
        unsafe {
            let n_len = n.len().min(511);
            std::ptr::copy_nonoverlapping(n.as_ptr(), raw.szName.as_mut_ptr(), n_len);
            let t_len = t.len().min(511);
            std::ptr::copy_nonoverlapping(t.as_ptr(), raw.szTopic.as_mut_ptr(), t_len);
        }
        raw.uChannelType = self.channel_type.raw();
        raw.nUserData = self.user_data;
        raw.nDiskQuota = self.disk_quota;
        raw.nMaxUsers = self.max_users;
        raw.audiocodec = self.audio_codec.to_ffi();
        raw.audiocfg = self.audio_cfg.to_ffi();
        raw.nTransmitUsersQueueDelayMSec = self.queue_delay_ms;
        raw.nTimeOutTimerVoiceMSec = self.timeout_voice_ms;
        raw.nTimeOutTimerMediaFileMSec = self.timeout_media_ms;
        for (i, (uid, types)) in self.transmit_users.iter().take(128).enumerate() {
            raw.transmitUsers[i][0] = uid.0;
            raw.transmitUsers[i][1] = *types as i32;
        }
        for (i, id) in self.transmit_users_queue.iter().take(16).enumerate() {
            raw.transmitUsersQueue[i] = id.0;
        }
        raw
    }
}

/// Text message payload.
#[derive(Debug, Clone)]
pub struct TextMessage {
    pub msg_type: ffi::TextMsgType,
    pub from_id: UserId,
    pub from_username: String,
    pub to_id: UserId,
    pub channel_id: ChannelId,
    pub text: String,
    pub more: bool,
}

impl From<ffi::TextMessage> for TextMessage {
    fn from(m: ffi::TextMessage) -> Self {
        Self {
            msg_type: m.nMsgType,
            from_id: UserId(m.nFromUserID),
            from_username: crate::utils::strings::to_string(&m.szFromUsername),
            to_id: UserId(m.nToUserID),
            channel_id: ChannelId(m.nChannelID),
            text: unsafe { crate::utils::strings::from_tt(m.szMessage.as_ptr()) },
            more: m.bMore != 0,
        }
    }
}

impl TextMessage {
    pub fn send_to_user(client: &crate::client::Client, user_id: UserId, text: &str) -> CommandId {
        MessageBuilder::new(user_id).text(text).send_cmd(client)
    }

    pub fn send_to_channel(
        client: &crate::client::Client,
        channel_id: ChannelId,
        text: &str,
    ) -> CommandId {
        MessageBuilder::new(channel_id).text(text).send_cmd(client)
    }

    pub fn send_broadcast(client: &crate::client::Client, text: &str) -> CommandId {
        MessageBuilder::new(MessageTarget::Broadcast)
            .text(text)
            .send_cmd(client)
    }

    /// Sends a private reply to the sender of this message.
    pub fn send_private(&self, client: &crate::client::Client, text: &str) -> CommandId {
        MessageBuilder::new(self.from_id)
            .text(text)
            .send_cmd(client)
    }
}

/// Destination for sending text messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageTarget {
    User(UserId),
    Channel(ChannelId),
    Broadcast,
}

impl From<UserId> for MessageTarget {
    fn from(id: UserId) -> Self {
        Self::User(id)
    }
}
impl From<ChannelId> for MessageTarget {
    fn from(id: ChannelId) -> Self {
        Self::Channel(id)
    }
}
impl From<&TextMessage> for MessageTarget {
    fn from(m: &TextMessage) -> Self {
        Self::User(m.from_id)
    }
}

/// Builder for outgoing text messages.
pub struct MessageBuilder {
    target: MessageTarget,
    text: String,
}

impl MessageBuilder {
    /// Creates a new builder for the target.
    pub fn new(target: impl Into<MessageTarget>) -> Self {
        Self {
            target: target.into(),
            text: String::new(),
        }
    }

    /// Sets the message body.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Sends the message using the provided client.
    pub fn send(self, client: &crate::client::Client) -> i32 {
        client.send_text(self.target, &self.text)
    }

    /// Sends the message and wraps the result in a CommandId.
    pub fn send_cmd(self, client: &crate::client::Client) -> CommandId {
        CommandId(self.send(client))
    }
}

/// Sound device description.
pub struct SoundDevice {
    pub id: i32,
    pub name: String,
    pub system: ffi::SoundSystem,
    pub device_uid: String,
    pub max_input_channels: i32,
    pub max_output_channels: i32,
    pub input_sample_rates: Vec<i32>,
    pub output_sample_rates: Vec<i32>,
    pub default_sample_rate: i32,
    pub features: u32,
}

impl From<ffi::SoundDevice> for SoundDevice {
    fn from(d: ffi::SoundDevice) -> Self {
        Self {
            id: d.nDeviceID,
            name: crate::utils::strings::to_string(&d.szDeviceName),
            system: d.nSoundSystem,
            device_uid: crate::utils::strings::to_string(&d.szDeviceID),
            max_input_channels: d.nMaxInputChannels,
            max_output_channels: d.nMaxOutputChannels,
            input_sample_rates: d
                .inputSampleRates
                .iter()
                .take_while(|&&r| r != 0)
                .cloned()
                .collect(),
            output_sample_rates: d
                .outputSampleRates
                .iter()
                .take_while(|&&r| r != 0)
                .cloned()
                .collect(),
            default_sample_rate: d.nDefaultSampleRate,
            features: d.uSoundDeviceFeatures,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// File transfer status.
pub enum FileTransferStatus {
    Closed,
    Error,
    Active,
    Finished,
}

impl From<ffi::FileTransferStatus> for FileTransferStatus {
    fn from(s: ffi::FileTransferStatus) -> Self {
        match s {
            ffi::FileTransferStatus::FILETRANSFER_CLOSED => Self::Closed,
            ffi::FileTransferStatus::FILETRANSFER_ERROR => Self::Error,
            ffi::FileTransferStatus::FILETRANSFER_ACTIVE => Self::Active,
            ffi::FileTransferStatus::FILETRANSFER_FINISHED => Self::Finished,
        }
    }
}

/// File transfer information.
pub struct FileTransfer {
    pub status: FileTransferStatus,
    pub id: TransferId,
    pub channel_id: ChannelId,
    pub local_path: String,
    pub remote_name: String,
    pub size: i64,
    pub transferred: i64,
    pub inbound: bool,
}

impl FileTransfer {
    /// Returns transfer progress as a 0.0-1.0 fraction.
    pub fn progress(&self) -> f32 {
        if self.size == 0 {
            0.0
        } else {
            self.transferred as f32 / self.size as f32
        }
    }
}

impl From<ffi::FileTransfer> for FileTransfer {
    fn from(t: ffi::FileTransfer) -> Self {
        Self {
            status: FileTransferStatus::from(t.nStatus),
            id: TransferId(t.nTransferID),
            channel_id: ChannelId(t.nChannelID),
            local_path: crate::utils::strings::to_string(&t.szLocalFilePath),
            remote_name: crate::utils::strings::to_string(&t.szRemoteFileName),
            size: t.nFileSize,
            transferred: t.nTransferred,
            inbound: t.bInbound != 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
/// Remote file metadata.
pub struct RemoteFile {
    pub channel_id: ChannelId,
    pub id: FileId,
    pub name: String,
    pub size: i64,
    pub owner: String,
    pub upload_time: String,
}

impl From<ffi::RemoteFile> for RemoteFile {
    fn from(f: ffi::RemoteFile) -> Self {
        Self {
            channel_id: ChannelId(f.nChannelID),
            id: FileId(f.nFileID),
            name: crate::utils::strings::to_string(&f.szFileName),
            size: f.nFileSize,
            owner: crate::utils::strings::to_string(&f.szUsername),
            upload_time: crate::utils::strings::to_string(&f.szUploadTime),
        }
    }
}

#[derive(Debug, Clone)]
/// Media file information.
pub struct MediaFileInfo {
    pub status: ffi::MediaFileStatus,
    pub name: String,
    pub audio_fmt: ffi::AudioFormat,
    pub video_fmt: ffi::VideoFormat,
    pub duration_ms: u32,
    pub elapsed_ms: u32,
}

impl From<ffi::MediaFileInfo> for MediaFileInfo {
    fn from(i: ffi::MediaFileInfo) -> Self {
        Self {
            status: i.nStatus,
            name: crate::utils::strings::to_string(&i.szFileName),
            audio_fmt: i.audioFmt,
            video_fmt: i.videoFmt,
            duration_ms: i.uDurationMSec,
            elapsed_ms: i.uElapsedMSec,
        }
    }
}

/// Server properties snapshot.
#[derive(Debug, Clone)]
pub struct ServerProperties {
    pub name: String,
    pub motd: String,
    pub motd_raw: String,
    pub max_users: i32,
    pub max_login_attempts: i32,
    pub max_logins_per_ip: i32,
    pub max_voice_tx: i32,
    pub max_video_tx: i32,
    pub max_media_tx: i32,
    pub max_desktop_tx: i32,
    pub max_total_tx: i32,
    pub auto_save: bool,
    pub tcp_port: i32,
    pub udp_port: i32,
    pub user_timeout: i32,
    pub version: String,
    pub protocol_version: String,
    pub login_delay: i32,
    pub access_token: String,
    pub log_events: u32,
}

impl From<ffi::ServerProperties> for ServerProperties {
    fn from(p: ffi::ServerProperties) -> Self {
        Self {
            name: crate::utils::strings::to_string(&p.szServerName),
            motd: crate::utils::strings::to_string(&p.szMOTD),
            motd_raw: crate::utils::strings::to_string(&p.szMOTDRaw),
            max_users: p.nMaxUsers,
            max_login_attempts: p.nMaxLoginAttempts,
            max_logins_per_ip: p.nMaxLoginsPerIPAddress,
            max_voice_tx: p.nMaxVoiceTxPerSecond,
            max_video_tx: p.nMaxVideoCaptureTxPerSecond,
            max_media_tx: p.nMaxMediaFileTxPerSecond,
            max_desktop_tx: p.nMaxDesktopTxPerSecond,
            max_total_tx: p.nMaxTotalTxPerSecond,
            auto_save: p.bAutoSave != 0,
            tcp_port: p.nTcpPort,
            udp_port: p.nUdpPort,
            user_timeout: p.nUserTimeout,
            version: crate::utils::strings::to_string(&p.szServerVersion),
            protocol_version: crate::utils::strings::to_string(&p.szServerProtocolVersion),
            login_delay: p.nLoginDelayMSec,
            access_token: crate::utils::strings::to_string(&p.szAccessToken),
            log_events: p.uServerLogEvents,
        }
    }
}

impl ServerProperties {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::ServerProperties {
        let mut raw = ffi::ServerProperties::default();
        let name = crate::utils::ToTT::tt(&self.name);
        let motd_raw = crate::utils::ToTT::tt(&self.motd_raw);
        unsafe {
            let n_len = name.len().min(511);
            std::ptr::copy_nonoverlapping(name.as_ptr(), raw.szServerName.as_mut_ptr(), n_len);
            let m_len = motd_raw.len().min(511);
            std::ptr::copy_nonoverlapping(motd_raw.as_ptr(), raw.szMOTDRaw.as_mut_ptr(), m_len);
        }
        raw.nMaxUsers = self.max_users;
        raw.nMaxLoginAttempts = self.max_login_attempts;
        raw.nMaxLoginsPerIPAddress = self.max_logins_per_ip;
        raw.nMaxVoiceTxPerSecond = self.max_voice_tx;
        raw.nMaxVideoCaptureTxPerSecond = self.max_video_tx;
        raw.nMaxMediaFileTxPerSecond = self.max_media_tx;
        raw.nMaxDesktopTxPerSecond = self.max_desktop_tx;
        raw.nMaxTotalTxPerSecond = self.max_total_tx;
        raw.bAutoSave = self.auto_save as i32;
        raw.nTcpPort = self.tcp_port;
        raw.nUdpPort = self.udp_port;
        raw.nUserTimeout = self.user_timeout;
        raw.nLoginDelayMSec = self.login_delay;
        raw.uServerLogEvents = self.log_events;
        raw
    }
}

/// Client statistics snapshot.
pub struct ClientStatistics {
    pub udp_ping: i32,
    pub tcp_ping: i32,
    pub udp_sent: i64,
    pub udp_recv: i64,
    pub voice_sent: i64,
    pub voice_recv: i64,
    pub video_sent: i64,
    pub video_recv: i64,
    pub media_audio_sent: i64,
    pub media_audio_recv: i64,
    pub media_video_sent: i64,
    pub media_video_recv: i64,
    pub desktop_sent: i64,
    pub desktop_recv: i64,
}

impl From<ffi::ClientStatistics> for ClientStatistics {
    fn from(s: ffi::ClientStatistics) -> Self {
        Self {
            udp_ping: s.nUdpPingTimeMs,
            tcp_ping: s.nTcpPingTimeMs,
            udp_sent: s.nUdpBytesSent,
            udp_recv: s.nUdpBytesRecv,
            voice_sent: s.nVoiceBytesSent,
            voice_recv: s.nVoiceBytesRecv,
            video_sent: s.nVideoCaptureBytesSent,
            video_recv: s.nVideoCaptureBytesRecv,
            media_audio_sent: s.nMediaFileAudioBytesSent,
            media_audio_recv: s.nMediaFileAudioBytesRecv,
            media_video_sent: s.nMediaFileVideoBytesSent,
            media_video_recv: s.nMediaFileVideoBytesRecv,
            desktop_sent: s.nDesktopBytesSent,
            desktop_recv: s.nDesktopBytesRecv,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Client flag bitmask.
pub struct ClientFlags(u32);

impl ClientFlags {
    /// Client is closed.
    pub const CLOSED: u32 = 0x00000000;
    /// Sound input is ready.
    pub const SNDINPUT_READY: u32 = 0x00000001;
    /// Sound output is ready.
    pub const SNDOUTPUT_READY: u32 = 0x00000002;
    /// Sound input/output duplex is ready.
    pub const SNDINOUTPUT_DUPLEX: u32 = 0x00000004;
    /// Voice activation is enabled.
    pub const SNDINPUT_VOICEACTIVATED: u32 = 0x00000008;
    /// Voice input is active.
    pub const SNDINPUT_VOICEACTIVE: u32 = 0x00000010;
    /// Output is muted.
    pub const SNDOUTPUT_MUTE: u32 = 0x00000020;
    /// Auto 3D positioning is enabled.
    pub const SNDOUTPUT_AUTO3DPOSITION: u32 = 0x00000040;
    /// Video capture is ready.
    pub const VIDEOCAPTURE_READY: u32 = 0x00000080;
    /// Transmitting voice.
    pub const TX_VOICE: u32 = 0x00000100;
    /// Transmitting video.
    pub const TX_VIDEOCAPTURE: u32 = 0x00000200;
    /// Transmitting desktop.
    pub const TX_DESKTOP: u32 = 0x00000400;
    /// Desktop sharing is active.
    pub const DESKTOP_ACTIVE: u32 = 0x00000800;
    /// Muxing audio file.
    pub const MUX_AUDIOFILE: u32 = 0x00001000;
    /// Currently connecting.
    pub const CONNECTING: u32 = 0x00002000;
    /// Connected.
    pub const CONNECTED: u32 = 0x00004000;
    /// Authorized.
    pub const AUTHORIZED: u32 = 0x00008000;
    /// Streaming audio.
    pub const STREAM_AUDIO: u32 = 0x00010000;
    /// Streaming video.
    pub const STREAM_VIDEO: u32 = 0x00020000;

    /// Creates from raw bits.
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }
    /// Returns true if the flag is present.
    pub fn has(&self, flag: u32) -> bool {
        (self.0 & flag) != 0
    }
    /// Returns the raw bitmask.
    pub fn raw(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Default)]
/// User account definition.
pub struct UserAccount {
    pub username: String,
    pub password: String,
    pub user_type: u32,
    pub user_rights: u32,
    pub note: String,
    pub init_channel: String,
    pub user_data: i32,
    pub auto_operator_channels: Vec<ChannelId>,
    pub audio_codec_bps_limit: i32,
    pub abuse_prevent: AbusePrevention,
}

impl UserAccount {
    /// Creates a user account builder.
    pub fn builder(username: &str) -> UserAccountBuilder {
        UserAccountBuilder::new(username)
    }
}

/// Builder for user account configuration.
pub struct UserAccountBuilder {
    inner: UserAccount,
}

impl UserAccountBuilder {
    /// Creates a new builder with the username.
    pub fn new(username: &str) -> Self {
        Self {
            inner: UserAccount {
                username: username.to_string(),
                password: String::new(),
                user_type: 0,
                user_rights: 0,
                note: String::new(),
                init_channel: String::new(),
                user_data: 0,
                auto_operator_channels: Vec::new(),
                audio_codec_bps_limit: 0,
                abuse_prevent: AbusePrevention::default(),
            },
        }
    }

    /// Sets the account password.
    pub fn password(mut self, pass: &str) -> Self {
        self.inner.password = pass.to_string();
        self
    }

    /// Sets the user type.
    pub fn user_type(mut self, t: u32) -> Self {
        self.inner.user_type = t;
        self
    }

    /// Sets the user rights.
    pub fn rights(mut self, r: u32) -> Self {
        self.inner.user_rights = r;
        self
    }

    /// Builds the user account.
    pub fn build(self) -> UserAccount {
        self.inner
    }
}

impl UserAccount {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::UserAccount {
        let mut raw = ffi::UserAccount::default();
        let u = crate::utils::ToTT::tt(&self.username);
        let p = crate::utils::ToTT::tt(&self.password);
        let n = crate::utils::ToTT::tt(&self.note);
        let c = crate::utils::ToTT::tt(&self.init_channel);
        unsafe {
            let u_len = u.len().min(511);
            std::ptr::copy_nonoverlapping(u.as_ptr(), raw.szUsername.as_mut_ptr(), u_len);
            let p_len = p.len().min(511);
            std::ptr::copy_nonoverlapping(p.as_ptr(), raw.szPassword.as_mut_ptr(), p_len);
            let n_len = n.len().min(511);
            std::ptr::copy_nonoverlapping(n.as_ptr(), raw.szNote.as_mut_ptr(), n_len);
            let c_len = c.len().min(511);
            std::ptr::copy_nonoverlapping(c.as_ptr(), raw.szInitChannel.as_mut_ptr(), c_len);
        }
        raw.uUserType = self.user_type;
        raw.uUserRights = self.user_rights;
        raw.nUserData = self.user_data;
        raw.nAudioCodecBpsLimit = self.audio_codec_bps_limit;
        raw.abusePrevent = self.abuse_prevent.to_ffi();
        for (i, cid) in self.auto_operator_channels.iter().take(16).enumerate() {
            raw.autoOperatorChannels[i] = cid.0;
        }
        raw
    }
}

impl From<ffi::UserAccount> for UserAccount {
    fn from(a: ffi::UserAccount) -> Self {
        Self {
            username: crate::utils::strings::to_string(&a.szUsername),
            password: String::new(),
            user_type: a.uUserType,
            user_rights: a.uUserRights,
            note: crate::utils::strings::to_string(&a.szNote),
            init_channel: crate::utils::strings::to_string(&a.szInitChannel),
            user_data: a.nUserData,
            auto_operator_channels: a
                .autoOperatorChannels
                .iter()
                .take_while(|&&c| c != 0)
                .map(|&c| ChannelId(c))
                .collect(),
            audio_codec_bps_limit: a.nAudioCodecBpsLimit,
            abuse_prevent: AbusePrevention::from(a.abusePrevent),
        }
    }
}

#[derive(Debug, Clone, Default)]
/// Banned user entry.
pub struct BannedUser {
    pub ip: String,
    pub channel_path: String,
    pub nickname: String,
    pub username: String,
    pub ban_time: String,
    pub ban_types: u32,
    pub owner: String,
}

impl From<ffi::BannedUser> for BannedUser {
    fn from(b: ffi::BannedUser) -> Self {
        Self {
            ip: crate::utils::strings::to_string(&b.szIPAddress),
            channel_path: crate::utils::strings::to_string(&b.szChannelPath),
            nickname: crate::utils::strings::to_string(&b.szNickname),
            username: crate::utils::strings::to_string(&b.szUsername),
            ban_time: crate::utils::strings::to_string(&b.szBanTime),
            ban_types: b.uBanTypes,
            owner: crate::utils::strings::to_string(&b.szOwner),
        }
    }
}

impl BannedUser {
    /// Converts to the raw TeamTalk struct.
    pub fn to_ffi(&self) -> ffi::BannedUser {
        let mut raw = ffi::BannedUser::default();
        let ip = crate::utils::ToTT::tt(&self.ip);
        let chan = crate::utils::ToTT::tt(&self.channel_path);
        let nick = crate::utils::ToTT::tt(&self.nickname);
        let user = crate::utils::ToTT::tt(&self.username);
        let owner = crate::utils::ToTT::tt(&self.owner);
        unsafe {
            let ip_len = ip.len().min(511);
            std::ptr::copy_nonoverlapping(ip.as_ptr(), raw.szIPAddress.as_mut_ptr(), ip_len);
            let chan_len = chan.len().min(511);
            std::ptr::copy_nonoverlapping(chan.as_ptr(), raw.szChannelPath.as_mut_ptr(), chan_len);
            let nick_len = nick.len().min(511);
            std::ptr::copy_nonoverlapping(nick.as_ptr(), raw.szNickname.as_mut_ptr(), nick_len);
            let user_len = user.len().min(511);
            std::ptr::copy_nonoverlapping(user.as_ptr(), raw.szUsername.as_mut_ptr(), user_len);
            let owner_len = owner.len().min(511);
            std::ptr::copy_nonoverlapping(owner.as_ptr(), raw.szOwner.as_mut_ptr(), owner_len);
        }
        raw.uBanTypes = self.ban_types;
        raw
    }
}

/// User statistics snapshot.
pub struct UserStatistics {
    pub voice_recv: i64,
    pub voice_lost: i64,
    pub video_recv: i64,
    pub video_frames_recv: i64,
    pub video_frames_lost: i64,
    pub video_frames_dropped: i64,
    pub media_audio_recv: i64,
    pub media_audio_lost: i64,
    pub media_video_recv: i64,
    pub media_video_frames_recv: i64,
    pub media_video_frames_lost: i64,
    pub media_video_frames_dropped: i64,
}

impl From<ffi::UserStatistics> for UserStatistics {
    fn from(s: ffi::UserStatistics) -> Self {
        Self {
            voice_recv: s.nVoicePacketsRecv,
            voice_lost: s.nVoicePacketsLost,
            video_recv: s.nVideoCapturePacketsRecv,
            video_frames_recv: s.nVideoCaptureFramesRecv,
            video_frames_lost: s.nVideoCaptureFramesLost,
            video_frames_dropped: s.nVideoCaptureFramesDropped,
            media_audio_recv: s.nMediaFileAudioPacketsRecv,
            media_audio_lost: s.nMediaFileAudioPacketsLost,
            media_video_recv: s.nMediaFileVideoPacketsRecv,
            media_video_frames_recv: s.nMediaFileVideoFramesRecv,
            media_video_frames_lost: s.nMediaFileVideoFramesLost,
            media_video_frames_dropped: s.nMediaFileVideoFramesDropped,
        }
    }
}

/// Server statistics snapshot.
#[derive(Debug, Clone)]
pub struct ServerStatistics {
    pub total_tx: i64,
    pub total_rx: i64,
    pub voice_tx: i64,
    pub voice_rx: i64,
    pub video_tx: i64,
    pub video_rx: i64,
    pub media_tx: i64,
    pub media_rx: i64,
    pub desktop_tx: i64,
    pub desktop_rx: i64,
    pub users_served: i32,
    pub users_peak: i32,
    pub files_tx: i64,
    pub files_rx: i64,
    pub uptime_ms: i64,
}

impl From<ffi::ServerStatistics> for ServerStatistics {
    fn from(s: ffi::ServerStatistics) -> Self {
        Self {
            total_tx: s.nTotalBytesTX,
            total_rx: s.nTotalBytesRX,
            voice_tx: s.nVoiceBytesTX,
            voice_rx: s.nVoiceBytesRX,
            video_tx: s.nVideoCaptureBytesTX,
            video_rx: s.nVideoCaptureBytesRX,
            media_tx: s.nMediaFileBytesTX,
            media_rx: s.nMediaFileBytesRX,
            desktop_tx: s.nDesktopBytesTX,
            desktop_rx: s.nDesktopBytesRX,
            users_served: s.nUsersServed,
            users_peak: s.nUsersPeak,
            files_tx: s.nFilesTx,
            files_rx: s.nFilesRx,
            uptime_ms: s.nUptimeMSec,
        }
    }
}
