#![doc = include_str!("../README.md")]
use std::path::Path;

pub mod client;
pub mod events;
pub mod extensions;
pub mod loader;
pub mod types;
pub mod utils;

#[cfg(feature = "async")]
pub mod async_api;
#[cfg(feature = "dispatch")]
pub mod dispatch;
#[cfg(feature = "logging")]
pub mod logging;
#[cfg(feature = "mock")]
pub mod mock;

#[cfg(feature = "async")]
pub use async_api::{AsyncClient, AsyncConfig};
pub use client::audio::AudioDeviceProfile;
pub use client::audio::{
    AudioBlockSink, AudioBlockSubscription, AudioBlockView, CallbackSink, UdpSink, WriterSink,
};
pub use client::recording::{
    RecordSession, RecordingOptions, RecordingSampleFormat, RecordingSession, RecordingTarget,
    SilencePolicy, SyncedUserRecording, SyncedUserRecordingBus, SyncedUserRecordingOptions,
    SyncedUserRecordingSession, UserRecordingOptions, UserRecordingSession,
};
pub use client::users::LoginParams;
pub use client::{
    Client, ClientCommands, ClientEvent, ClientEvents, ClientHealth, ClientHooks, ClientInfo,
    ClientManager, ClientRegistry, EventContext, EventSubscriptionId, Message, ReconnectConfig,
    ServerInfo,
};
#[cfg(feature = "dispatch")]
pub use dispatch::{
    ClientConfig, ConnectParamsOwned, DispatchFlow, Dispatcher,
    EventContext as DispatchEventContext, ReconnectSettings,
};
pub use events::{ConnectionState, Error, Event, Result};
#[cfg(feature = "mock")]
pub use mock::{MockClient, MockMessage, MockUserBuilder};
pub use types::{ClientId, MessageBuilder};

/// Initializes the TeamTalk SDK by loading the runtime DLL from the default location.
pub fn init() -> Result<()> {
    let dll_path = loader::find_or_download_dll().map_err(|_| Error::InitFailed)?;
    let dll_path = dll_path.to_str().ok_or(Error::InitFailed)?;
    teamtalk_sys::load(dll_path).map_err(|_| Error::InitFailed)?;
    Ok(())
}

/// Initializes the TeamTalk SDK using a custom DLL path.
pub fn init_with_path<P: AsRef<Path>>(path: P) -> Result<()> {
    teamtalk_sys::load(path.as_ref().to_str().ok_or(Error::InitFailed)?)
        .map_err(|_| Error::InitFailed)?;
    Ok(())
}
