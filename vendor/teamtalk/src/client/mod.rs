//! Core client type and message wrapper.
pub use teamtalk_sys as ffi;

pub mod audio;
#[cfg(feature = "mock")]
pub mod backend;
#[cfg(not(feature = "mock"))]
pub(crate) mod backend;
pub mod bus;
pub mod cache;
pub mod channels;
pub mod connection;
pub mod core;
pub mod desktop;
pub mod encryption;
pub mod files;
pub mod hooks;
#[cfg(windows)]
pub mod hotkeys;
pub mod manager;
pub mod media;
pub mod recording;
pub mod registry;
pub mod server;
pub mod system;
pub mod users;
pub mod video;

pub use bus::{EventContext, EventSubscriptionGroup, EventSubscriptionId, SubscriptionBuilder};
pub use cache::ServerInfo;
pub use connection::{ConnectParams, ConnectParamsOwned, ReconnectConfig, ReconnectHandler};
pub use core::{Client, ClientCommands, ClientEvents, Message};
pub use hooks::ClientHooks;
pub use manager::{ClientEvent, ClientHealth, ClientManager};
pub use registry::{ClientInfo, ClientRegistry};
