//! Media file playback and streaming APIs.
use super::Client;
use crate::types::{UserId, VideoCodec};
use crate::utils::ToTT;
use teamtalk_sys as ffi;

/// Controls media file playback behavior.
#[derive(Debug, Clone, Default)]
pub struct MediaFilePlayback {
    pub offset_ms: u32,
    pub paused: bool,
}

impl MediaFilePlayback {
    pub fn to_ffi(&self) -> ffi::MediaFilePlayback {
        ffi::MediaFilePlayback {
            uOffsetMSec: self.offset_ms,
            bPaused: self.paused as i32,
            ..Default::default()
        }
    }
}

impl Client {
    /// Starts streaming a media file to the channel.
    pub fn start_streaming_media_file_to_channel(
        &self,
        file_path: &str,
        codec: Option<&VideoCodec>,
    ) -> bool {
        let codec_ptr = codec.map_or(std::ptr::null(), |c| &c.to_ffi());
        unsafe {
            ffi::api().TT_StartStreamingMediaFileToChannel(
                self.ptr.0,
                file_path.tt().as_ptr(),
                codec_ptr,
            ) == 1
        }
    }

    /// Starts streaming a media file with advanced playback options.
    pub fn start_streaming_media_file_to_channel_ex(
        &self,
        file_path: &str,
        playback: &MediaFilePlayback,
        codec: Option<&VideoCodec>,
    ) -> bool {
        let codec_ptr = codec.map_or(std::ptr::null(), |c| &c.to_ffi());
        unsafe {
            ffi::api().TT_StartStreamingMediaFileToChannelEx(
                self.ptr.0,
                file_path.tt().as_ptr(),
                &playback.to_ffi(),
                codec_ptr,
            ) == 1
        }
    }

    /// Updates the currently streaming media file info.
    pub fn update_streaming_media_file_to_channel(
        &self,
        playback: &MediaFilePlayback,
        codec: Option<&VideoCodec>,
    ) -> bool {
        let codec_ptr = codec.map_or(std::ptr::null(), |c| &c.to_ffi());
        unsafe {
            ffi::api().TT_UpdateStreamingMediaFileToChannel(
                self.ptr.0,
                &playback.to_ffi(),
                codec_ptr,
            ) == 1
        }
    }

    /// Stops media file streaming.
    pub fn stop_streaming_media_file_to_channel(&self) -> bool {
        unsafe { ffi::api().TT_StopStreamingMediaFileToChannel(self.ptr.0) == 1 }
    }

    /// Initializes local media playback.
    pub fn init_local_playback(&self, file_path: &str, playback: &MediaFilePlayback) -> i32 {
        unsafe {
            ffi::api().TT_InitLocalPlayback(self.ptr.0, file_path.tt().as_ptr(), &playback.to_ffi())
        }
    }

    /// Updates local playback info.
    pub fn update_local_playback(&self, session_id: i32, playback: &MediaFilePlayback) -> bool {
        unsafe {
            ffi::api().TT_UpdateLocalPlayback(self.ptr.0, session_id, &playback.to_ffi()) == 1
        }
    }

    /// Stops local playback.
    pub fn stop_local_playback(&self, session_id: i32) -> bool {
        unsafe { ffi::api().TT_StopLocalPlayback(self.ptr.0, session_id) == 1 }
    }

    /// Acquires a media video frame for a user.
    pub fn acquire_user_media_video_frame(&self, user_id: UserId) -> Option<*mut ffi::VideoFrame> {
        unsafe {
            let ptr = ffi::api().TT_AcquireUserMediaVideoFrame(self.ptr.0, user_id.0);
            if ptr.is_null() { None } else { Some(ptr) }
        }
    }

    /// Releases a previously acquired media video frame.
    ///
    /// # Safety
    /// - `frame` must be a pointer returned by `acquire_user_media_video_frame`.
    /// - The frame must not be released more than once.
    /// - The pointer must not be used after release.
    pub unsafe fn release_user_media_video_frame(&self, frame: *mut ffi::VideoFrame) -> bool {
        if frame.is_null() {
            return false;
        }
        unsafe { ffi::api().TT_ReleaseUserMediaVideoFrame(self.ptr.0, frame) == 1 }
    }
}
