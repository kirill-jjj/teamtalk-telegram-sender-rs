//! Event and error types emitted by the TeamTalk client.
use crate::types::ChannelId;
use std::time::Duration;
use teamtalk_sys as ffi;

/// Client event emitted by `Client::poll`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    None,
    ConnectSuccess,
    ConnectCryptError,
    ConnectFailed,
    ConnectionLost,
    ConnectMaxPayloadUpdated,
    CmdProcessing,
    CmdError,
    CmdSuccess,
    MySelfLoggedIn,
    MySelfLoggedOut,
    MySelfKicked,
    UserLoggedIn,
    UserLoggedOut,
    UserUpdate,
    UserJoined,
    UserLeft,
    TextMessage,
    ChannelCreated,
    ChannelUpdated,
    ChannelRemoved,
    ServerUpdate,
    ServerStatistics,
    FileNew,
    FileRemove,
    UserAccount,
    BannedUser,
    UserAccountCreated,
    UserAccountRemoved,
    UserStateChange,
    VideoCaptureFrame,
    MediaFileVideo,
    DesktopWindow,
    DesktopCursor,
    DesktopInput,
    UserRecordMediaFile,
    AudioBlock,
    InternalError,
    VoiceActivation,
    Hotkey,
    HotkeyTest,
    FileTransfer,
    DesktopWindowTransfer,
    StreamMediaFile,
    LocalMediaFile,
    AudioInput,
    UserFirstVoiceStreamPacket,
    SoundDeviceAdded,
    SoundDeviceRemoved,
    SoundDeviceUnplugged,
    SoundDeviceNewDefaultInput,
    SoundDeviceNewDefaultOutput,
    SoundDeviceNewDefaultInputComDevice,
    SoundDeviceNewDefaultOutputComDevice,
    BeforeReconnect { attempt: u32, delay: Duration },
    Reconnecting { attempt: u32, delay: Duration },
    AfterReconnect { attempt: u32 },
    ReconnectFailed { attempts: u32 },
    Unknown(ffi::ClientEvent),
}

/// Client connection state derived from commands and events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    #[default]
    Idle,
    Connecting,
    Connected,
    LoggingIn,
    LoggedIn,
    Joining(ChannelId),
    Joined(ChannelId),
    Disconnected,
}

impl From<ffi::ClientEvent> for Event {
    fn from(event: ffi::ClientEvent) -> Self {
        match event {
            ffi::ClientEvent::CLIENTEVENT_NONE => Event::None,
            ffi::ClientEvent::CLIENTEVENT_CON_SUCCESS => Event::ConnectSuccess,
            ffi::ClientEvent::CLIENTEVENT_CON_CRYPT_ERROR => Event::ConnectCryptError,
            ffi::ClientEvent::CLIENTEVENT_CON_FAILED => Event::ConnectFailed,
            ffi::ClientEvent::CLIENTEVENT_CON_LOST => Event::ConnectionLost,
            ffi::ClientEvent::CLIENTEVENT_CON_MAX_PAYLOAD_UPDATED => {
                Event::ConnectMaxPayloadUpdated
            }
            ffi::ClientEvent::CLIENTEVENT_CMD_PROCESSING => Event::CmdProcessing,
            ffi::ClientEvent::CLIENTEVENT_CMD_ERROR => Event::CmdError,
            ffi::ClientEvent::CLIENTEVENT_CMD_SUCCESS => Event::CmdSuccess,
            ffi::ClientEvent::CLIENTEVENT_CMD_MYSELF_LOGGEDIN => Event::MySelfLoggedIn,
            ffi::ClientEvent::CLIENTEVENT_CMD_MYSELF_LOGGEDOUT => Event::MySelfLoggedOut,
            ffi::ClientEvent::CLIENTEVENT_CMD_MYSELF_KICKED => Event::MySelfKicked,
            ffi::ClientEvent::CLIENTEVENT_CMD_USER_LOGGEDIN => Event::UserLoggedIn,
            ffi::ClientEvent::CLIENTEVENT_CMD_USER_LOGGEDOUT => Event::UserLoggedOut,
            ffi::ClientEvent::CLIENTEVENT_CMD_USER_UPDATE => Event::UserUpdate,
            ffi::ClientEvent::CLIENTEVENT_CMD_USER_JOINED => Event::UserJoined,
            ffi::ClientEvent::CLIENTEVENT_CMD_USER_LEFT => Event::UserLeft,
            ffi::ClientEvent::CLIENTEVENT_CMD_USER_TEXTMSG => Event::TextMessage,
            ffi::ClientEvent::CLIENTEVENT_CMD_CHANNEL_NEW => Event::ChannelCreated,
            ffi::ClientEvent::CLIENTEVENT_CMD_CHANNEL_UPDATE => Event::ChannelUpdated,
            ffi::ClientEvent::CLIENTEVENT_CMD_CHANNEL_REMOVE => Event::ChannelRemoved,
            ffi::ClientEvent::CLIENTEVENT_CMD_SERVER_UPDATE => Event::ServerUpdate,
            ffi::ClientEvent::CLIENTEVENT_CMD_SERVERSTATISTICS => Event::ServerStatistics,
            ffi::ClientEvent::CLIENTEVENT_CMD_FILE_NEW => Event::FileNew,
            ffi::ClientEvent::CLIENTEVENT_CMD_FILE_REMOVE => Event::FileRemove,
            ffi::ClientEvent::CLIENTEVENT_CMD_USERACCOUNT => Event::UserAccount,
            ffi::ClientEvent::CLIENTEVENT_CMD_BANNEDUSER => Event::BannedUser,
            ffi::ClientEvent::CLIENTEVENT_CMD_USERACCOUNT_NEW => Event::UserAccountCreated,
            ffi::ClientEvent::CLIENTEVENT_CMD_USERACCOUNT_REMOVE => Event::UserAccountRemoved,
            ffi::ClientEvent::CLIENTEVENT_USER_STATECHANGE => Event::UserStateChange,
            ffi::ClientEvent::CLIENTEVENT_USER_VIDEOCAPTURE => Event::VideoCaptureFrame,
            ffi::ClientEvent::CLIENTEVENT_USER_MEDIAFILE_VIDEO => Event::MediaFileVideo,
            ffi::ClientEvent::CLIENTEVENT_USER_DESKTOPWINDOW => Event::DesktopWindow,
            ffi::ClientEvent::CLIENTEVENT_USER_DESKTOPCURSOR => Event::DesktopCursor,
            ffi::ClientEvent::CLIENTEVENT_USER_DESKTOPINPUT => Event::DesktopInput,
            ffi::ClientEvent::CLIENTEVENT_USER_RECORD_MEDIAFILE => Event::UserRecordMediaFile,
            ffi::ClientEvent::CLIENTEVENT_USER_AUDIOBLOCK => Event::AudioBlock,
            ffi::ClientEvent::CLIENTEVENT_INTERNAL_ERROR => Event::InternalError,
            ffi::ClientEvent::CLIENTEVENT_VOICE_ACTIVATION => Event::VoiceActivation,
            ffi::ClientEvent::CLIENTEVENT_HOTKEY => Event::Hotkey,
            ffi::ClientEvent::CLIENTEVENT_HOTKEY_TEST => Event::HotkeyTest,
            ffi::ClientEvent::CLIENTEVENT_FILETRANSFER => Event::FileTransfer,
            ffi::ClientEvent::CLIENTEVENT_DESKTOPWINDOW_TRANSFER => Event::DesktopWindowTransfer,
            ffi::ClientEvent::CLIENTEVENT_STREAM_MEDIAFILE => Event::StreamMediaFile,
            ffi::ClientEvent::CLIENTEVENT_LOCAL_MEDIAFILE => Event::LocalMediaFile,
            ffi::ClientEvent::CLIENTEVENT_AUDIOINPUT => Event::AudioInput,
            ffi::ClientEvent::CLIENTEVENT_USER_FIRSTVOICESTREAMPACKET => {
                Event::UserFirstVoiceStreamPacket
            }
            ffi::ClientEvent::CLIENTEVENT_SOUNDDEVICE_ADDED => Event::SoundDeviceAdded,
            ffi::ClientEvent::CLIENTEVENT_SOUNDDEVICE_REMOVED => Event::SoundDeviceRemoved,
            ffi::ClientEvent::CLIENTEVENT_SOUNDDEVICE_UNPLUGGED => Event::SoundDeviceUnplugged,
            ffi::ClientEvent::CLIENTEVENT_SOUNDDEVICE_NEW_DEFAULT_INPUT => {
                Event::SoundDeviceNewDefaultInput
            }
            ffi::ClientEvent::CLIENTEVENT_SOUNDDEVICE_NEW_DEFAULT_OUTPUT => {
                Event::SoundDeviceNewDefaultOutput
            }
            ffi::ClientEvent::CLIENTEVENT_SOUNDDEVICE_NEW_DEFAULT_INPUT_COMDEVICE => {
                Event::SoundDeviceNewDefaultInputComDevice
            }
            ffi::ClientEvent::CLIENTEVENT_SOUNDDEVICE_NEW_DEFAULT_OUTPUT_COMDEVICE => {
                Event::SoundDeviceNewDefaultOutputComDevice
            }
            #[allow(unreachable_patterns)]
            c => Event::Unknown(c),
        }
    }
}

impl Event {
    /// Returns true when the event indicates a reconnect should be attempted.
    pub fn is_reconnect_needed(&self) -> bool {
        matches!(
            self,
            Event::ConnectionLost | Event::ConnectFailed | Event::ConnectCryptError
        )
    }

    /// Returns true when the event indicates a reconnect should be attempted,
    /// including any additional custom events.
    pub fn is_reconnect_needed_with(&self, extra: &[Event]) -> bool {
        if self.is_reconnect_needed() {
            return true;
        }
        for extra_event in extra {
            if std::mem::discriminant(self) == std::mem::discriminant(extra_event) {
                return true;
            }
        }
        false
    }
}

/// Error type used across TeamTalk operations.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Init failed")]
    InitFailed,
    #[error("Command failed: {code} ({message})")]
    CommandFailed { code: i32, message: String },
    #[error("Connection failed")]
    ConnectFailed,
    #[error("Auth failed")]
    AuthFailed,
    #[error("Invalid parameter")]
    InvalidParam,
    #[error("Missing reconnect parameters")]
    MissingReconnectParams,
    #[error("Missing login parameters")]
    MissingLoginParams,
    #[error("SDK Error: {code} ({message})")]
    ClientError { code: i32, message: String },
    #[error("IO error: {message}")]
    IoError { message: String },
}

/// Convenience result type for TeamTalk operations.
pub type Result<T> = std::result::Result<T, Error>;
