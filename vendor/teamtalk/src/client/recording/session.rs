use super::super::Client;
use super::options::{RecordingOptions, RecordingTarget, segment_path};
use crate::client::Message;
use crate::events::{Error, Event, Result};
use crate::types::{AudioCodec, ChannelId};
use std::fs;
use std::time::Instant;

/// Managed recording session with pause/resume and segmentation support.
pub struct RecordingSession<'a> {
    client: &'a Client,
    target: RecordingTarget,
    options: RecordingOptions,
    active: bool,
    next_index: u32,
    current_path: Option<String>,
    segments: Vec<String>,
    segment_started_at: Option<Instant>,
    last_channel_id: Option<ChannelId>,
    last_codec: Option<AudioCodec>,
}

impl<'a> RecordingSession<'a> {
    /// Starts a managed recording session for a channel.
    pub fn start_channel(
        client: &'a Client,
        channel_id: ChannelId,
        options: RecordingOptions,
    ) -> Result<Self> {
        let mut session = Self {
            client,
            target: RecordingTarget::Channel(channel_id),
            options,
            active: false,
            next_index: 0,
            current_path: None,
            segments: Vec::new(),
            segment_started_at: None,
            last_channel_id: None,
            last_codec: None,
        };
        session.start_segment()?;
        Ok(session)
    }

    /// Starts a managed recording session for muxed streams.
    pub fn start_streams(
        client: &'a Client,
        stream_types: u32,
        codec: AudioCodec,
        options: RecordingOptions,
    ) -> Result<Self> {
        let mut session = Self {
            client,
            target: RecordingTarget::Streams {
                stream_types,
                codec,
            },
            options,
            active: false,
            next_index: 0,
            current_path: None,
            segments: Vec::new(),
            segment_started_at: None,
            last_channel_id: None,
            last_codec: None,
        };
        session.start_segment()?;
        Ok(session)
    }

    /// Starts a managed recording session for muxed audio.
    pub fn start_muxed(
        client: &'a Client,
        codec: AudioCodec,
        options: RecordingOptions,
    ) -> Result<Self> {
        let mut session = Self {
            client,
            target: RecordingTarget::Muxed { codec },
            options,
            active: false,
            next_index: 0,
            current_path: None,
            segments: Vec::new(),
            segment_started_at: None,
            last_channel_id: None,
            last_codec: None,
        };
        session.start_segment()?;
        Ok(session)
    }

    /// Starts a managed recording session for the current channel.
    pub fn start_current_channel(client: &'a Client, options: RecordingOptions) -> Result<Self> {
        let mut session = Self {
            client,
            target: RecordingTarget::CurrentChannel,
            options,
            active: false,
            next_index: 0,
            current_path: None,
            segments: Vec::new(),
            segment_started_at: None,
            last_channel_id: None,
            last_codec: None,
        };
        session.start_segment()?;
        Ok(session)
    }

    /// Returns true if the recording is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Returns the current output path, if any.
    pub fn current_path(&self) -> Option<&str> {
        self.current_path.as_deref()
    }

    /// Returns a list of recorded segment paths.
    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    /// Pauses the recording by stopping the current segment.
    pub fn pause(&mut self) -> bool {
        if !self.active {
            return true;
        }
        let ok = self.stop_active();
        if ok {
            self.active = false;
            self.current_path = None;
            self.segment_started_at = None;
        }
        ok
    }

    /// Resumes recording by creating a new segment.
    pub fn resume(&mut self) -> Result<bool> {
        if self.active {
            return Ok(true);
        }
        self.start_segment()?;
        Ok(self.active)
    }

    /// Stops the recording permanently.
    pub fn stop(mut self) -> bool {
        if self.active {
            let ok = self.stop_active();
            self.active = false;
            ok
        } else {
            true
        }
    }

    /// Forces a new segment without changing the target.
    pub fn segment(&mut self) -> Result<bool> {
        let _ = self.pause();
        self.resume()
    }

    /// Updates the channel target and starts a new segment.
    pub fn switch_channel(&mut self, channel_id: ChannelId) -> Result<bool> {
        self.target = RecordingTarget::Channel(channel_id);
        self.segment()
    }

    /// Rotates the segment when duration or size limits are reached.
    pub fn rotate_if_needed(&mut self) -> Result<bool> {
        if !self.active {
            return Ok(false);
        }

        if let Some(max_duration) = self.options.max_duration
            && let Some(started) = self.segment_started_at
            && started.elapsed() >= max_duration
        {
            return self.segment();
        }

        if let Some(max_size) = self.options.max_size_bytes
            && let Some(path) = self.current_path.as_ref()
        {
            let len = fs::metadata(path)
                .map(|m| m.len())
                .map_err(|e| Error::IoError {
                    message: e.to_string(),
                })?;
            if len >= max_size {
                return self.segment();
            }
        }

        Ok(false)
    }

    /// Applies smart rotation based on channel or codec changes.
    pub fn handle_event(&mut self, event: Event, message: &Message) -> Result<bool> {
        if !self.active {
            return Ok(false);
        }

        match self.target {
            RecordingTarget::CurrentChannel => {
                if let Some(rotated) = self.rotate_for_channel_change()? {
                    return Ok(rotated);
                }
            }
            RecordingTarget::Channel(_) => {
                if matches!(event, Event::ChannelUpdated)
                    && let Some(channel) = message.channel()
                    && Some(channel.id) == self.last_channel_id
                    && self.maybe_rotate_for_codec(channel.audio_codec)?
                {
                    return Ok(true);
                }
            }
            _ => {}
        }

        Ok(false)
    }

    fn stop_active(&self) -> bool {
        match self.target {
            RecordingTarget::Channel(id) => self.client.stop_recording_channel(id.0),
            RecordingTarget::CurrentChannel => {
                if let Some(id) = self.last_channel_id.or_else(|| self.current_channel_id()) {
                    self.client.stop_recording_channel(id.0)
                } else {
                    true
                }
            }
            RecordingTarget::Streams { .. } | RecordingTarget::Muxed { .. } => {
                self.client.stop_recording()
            }
        }
    }

    fn start_segment(&mut self) -> Result<()> {
        if self.next_index == 0 {
            self.next_index = self.options.start_index.max(1);
        }
        let path = segment_path(&self.options.template, self.next_index);
        self.next_index = self.next_index.saturating_add(1);

        let ok = match self.target.clone() {
            RecordingTarget::Channel(id) => {
                self.last_channel_id = Some(id);
                self.last_codec = self.channel_codec(id);
                self.client
                    .start_recording_channel(id.0, &path, self.options.format)
            }
            RecordingTarget::CurrentChannel => {
                let id = self
                    .current_channel_id()
                    .ok_or_else(|| Error::CommandFailed {
                        code: -1,
                        message: "Not joined to a channel".to_string(),
                    })?;
                self.last_channel_id = Some(id);
                self.last_codec = self.channel_codec(id);
                self.client
                    .start_recording_channel(id.0, &path, self.options.format)
            }
            RecordingTarget::Streams {
                stream_types,
                codec,
            } => self.client.start_recording_streams(
                stream_types,
                &codec,
                &path,
                self.options.format,
            ),
            RecordingTarget::Muxed { codec } => {
                self.client
                    .start_recording_muxed(&codec, &path, self.options.format)
            }
        };

        if ok {
            self.active = true;
            self.current_path = Some(path.clone());
            self.segments.push(path);
            self.segment_started_at = Some(Instant::now());
            Ok(())
        } else {
            Err(Error::CommandFailed {
                code: -1,
                message: "Recording start failed".to_string(),
            })
        }
    }

    fn current_channel_id(&self) -> Option<ChannelId> {
        let id = self.client.my_channel_id();
        if id.0 == 0 { None } else { Some(id) }
    }

    fn channel_codec(&self, id: ChannelId) -> Option<AudioCodec> {
        self.client
            .get_channel(id)
            .map(|channel| channel.audio_codec)
    }

    fn rotate_for_channel_change(&mut self) -> Result<Option<bool>> {
        let current = self.current_channel_id();
        if current.is_none() {
            return Ok(None);
        }
        let Some(current) = current else {
            return Ok(Some(false));
        };
        if self.options.rotate_on_channel_change && self.last_channel_id != Some(current) {
            let rotated = self.segment()?;
            return Ok(Some(rotated));
        }
        if let Some(codec) = self.channel_codec(current)
            && self.maybe_rotate_for_codec(codec)?
        {
            return Ok(Some(true));
        }
        Ok(Some(false))
    }

    fn maybe_rotate_for_codec(&mut self, codec: AudioCodec) -> Result<bool> {
        if self.options.rotate_on_codec_change && self.last_codec != Some(codec) {
            self.last_codec = Some(codec);
            return self.segment();
        }
        Ok(false)
    }
}
