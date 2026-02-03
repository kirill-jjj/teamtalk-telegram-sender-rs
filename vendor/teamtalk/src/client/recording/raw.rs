use super::super::Client;
use crate::events::{Error, Result};
use crate::types::{AudioCodec, ChannelId};
use teamtalk_sys as ffi;

impl Client {
    /// Starts recording a muxed audio file using a codec.
    pub fn start_recording_muxed(
        &self,
        codec: &AudioCodec,
        file_path: &str,
        format: ffi::AudioFileFormat,
    ) -> bool {
        self.backend()
            .start_recording_muxed(self.ptr.0, codec, file_path, format)
    }

    /// Starts recording the specified channel.
    pub fn start_recording_channel(
        &self,
        channel_id: i32,
        file_path: &str,
        format: ffi::AudioFileFormat,
    ) -> bool {
        self.backend()
            .start_recording_channel(self.ptr.0, channel_id, file_path, format)
    }

    /// Starts recording a set of stream types.
    pub fn start_recording_streams(
        &self,
        stream_types: u32,
        codec: &AudioCodec,
        file_path: &str,
        format: ffi::AudioFileFormat,
    ) -> bool {
        self.backend()
            .start_recording_streams(self.ptr.0, stream_types, codec, file_path, format)
    }

    /// Stops recording a muxed audio file.
    pub fn stop_recording(&self) -> bool {
        self.backend().stop_recording(self.ptr.0)
    }

    /// Stops recording for a channel.
    pub fn stop_recording_channel(&self, channel_id: i32) -> bool {
        self.backend()
            .stop_recording_channel(self.ptr.0, channel_id)
    }
}

/// Guard that stops a channel recording when dropped.
pub struct RecordSession<'a> {
    client: &'a Client,
    channel_id: ChannelId,
    active: bool,
}

impl<'a> RecordSession<'a> {
    /// Starts recording a channel and returns a guard that stops on drop.
    pub fn start_channel(
        client: &'a Client,
        channel_id: ChannelId,
        file_path: &str,
        format: ffi::AudioFileFormat,
    ) -> Result<Self> {
        if client.start_recording_channel(channel_id.0, file_path, format) {
            Ok(Self {
                client,
                channel_id,
                active: true,
            })
        } else {
            Err(Error::CommandFailed {
                code: -1,
                message: "Recording start failed".to_string(),
            })
        }
    }

    /// Stops the recording and returns whether it succeeded.
    pub fn stop(mut self) -> bool {
        let ok = self.client.stop_recording_channel(self.channel_id.0);
        self.active = false;
        ok
    }
}

impl Drop for RecordSession<'_> {
    fn drop(&mut self) {
        if self.active {
            let _ = self.client.stop_recording_channel(self.channel_id.0);
        }
    }
}
