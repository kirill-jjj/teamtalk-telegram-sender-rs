//! User-centric media storage and recording APIs.
use crate::Client;
use crate::types::UserId;
use crate::utils::ToTT;
use teamtalk_sys as ffi;

/// Configuration for per-user media recording.
#[derive(Clone, Debug)]
pub struct UserRecordingOptions {
    pub folder: String,
    pub file_vars: String,
    pub format: ffi::AudioFileFormat,
    pub stop_delay_ms: i32,
}

impl UserRecordingOptions {
    pub fn new(
        folder: impl Into<String>,
        file_vars: impl Into<String>,
        format: ffi::AudioFileFormat,
    ) -> Self {
        Self {
            folder: folder.into(),
            file_vars: file_vars.into(),
            format,
            stop_delay_ms: 0,
        }
    }

    pub fn with_stop_delay(mut self, delay_ms: i32) -> Self {
        self.stop_delay_ms = delay_ms;
        self
    }
}

/// Guard that manages per-user recording configuration.
pub struct UserRecordingSession<'a> {
    client: &'a Client,
    user_id: UserId,
    active: bool,
}

impl Client {
    /// Configures per-user media recording.
    pub fn set_user_media_storage(
        &self,
        user_id: UserId,
        folder: &str,
        file_vars: &str,
        format: ffi::AudioFileFormat,
    ) -> bool {
        unsafe {
            ffi::api().TT_SetUserMediaStorageDir(
                self.ptr.0,
                user_id.0,
                folder.tt().as_ptr(),
                file_vars.tt().as_ptr(),
                format,
            ) == 1
        }
    }

    /// Configures per-user media recording with a stop delay.
    pub fn set_user_media_storage_ex(
        &self,
        user_id: UserId,
        folder: &str,
        file_vars: &str,
        format: ffi::AudioFileFormat,
        stop_delay_ms: i32,
    ) -> bool {
        unsafe {
            ffi::api().TT_SetUserMediaStorageDirEx(
                self.ptr.0,
                user_id.0,
                folder.tt().as_ptr(),
                file_vars.tt().as_ptr(),
                format,
                stop_delay_ms,
            ) == 1
        }
    }

    /// Disables per-user media recording.
    pub fn clear_user_media_storage(&self, user_id: UserId) -> bool {
        self.set_user_media_storage_ex(user_id, "", "", ffi::AudioFileFormat::AFF_NONE, 0)
    }
}

impl<'a> UserRecordingSession<'a> {
    pub fn start(
        client: &'a Client,
        user_id: UserId,
        options: UserRecordingOptions,
    ) -> Option<Self> {
        let ok = client.set_user_media_storage_ex(
            user_id,
            &options.folder,
            &options.file_vars,
            options.format,
            options.stop_delay_ms,
        );
        if ok {
            Some(Self {
                client,
                user_id,
                active: true,
            })
        } else {
            None
        }
    }

    pub fn stop(mut self) -> bool {
        let ok = self.client.clear_user_media_storage(self.user_id);
        self.active = false;
        ok
    }
}

impl Drop for UserRecordingSession<'_> {
    fn drop(&mut self) {
        if self.active {
            let _ = self.client.clear_user_media_storage(self.user_id);
        }
    }
}
