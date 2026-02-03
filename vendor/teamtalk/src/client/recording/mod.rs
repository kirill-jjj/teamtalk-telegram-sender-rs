//! Recording APIs for channels and streams.

mod options;
mod raw;
mod session;
mod synced;
mod user;

pub use options::{RecordingOptions, RecordingSampleFormat, RecordingTarget};
pub use session::RecordingSession;
pub use synced::{
    SilencePolicy, SyncedUserRecording, SyncedUserRecordingBus, SyncedUserRecordingOptions,
    SyncedUserRecordingSession,
};
pub use user::{UserRecordingOptions, UserRecordingSession};

pub use raw::RecordSession;
