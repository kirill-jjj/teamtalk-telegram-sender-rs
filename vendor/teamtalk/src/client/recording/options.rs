use crate::types::{AudioCodec, ChannelId};
use std::time::Duration;
use teamtalk_sys as ffi;

#[derive(Clone, Debug)]
pub enum RecordingSampleFormat {
    PcmS16Le,
    WavS16Le,
}

/// Target for a managed recording session.
#[derive(Clone, Debug)]
pub enum RecordingTarget {
    /// Record the current channel audio (server codec).
    Channel(ChannelId),
    /// Record the channel you are currently joined to.
    CurrentChannel,
    /// Record muxed streams using a specific codec.
    Streams {
        stream_types: u32,
        codec: AudioCodec,
    },
    /// Record muxed audio using a specific codec.
    Muxed { codec: AudioCodec },
}

/// Configuration for managed recordings.
#[derive(Clone, Debug)]
pub struct RecordingOptions {
    /// Output file template. Use `{index}` to control segment numbering.
    pub template: String,
    /// Audio file format for the SDK muxed recorder.
    pub format: ffi::AudioFileFormat,
    /// First segment index to use.
    pub start_index: u32,
    /// Rotate after the given duration.
    pub max_duration: Option<Duration>,
    /// Rotate after the file exceeds the given size in bytes.
    pub max_size_bytes: Option<u64>,
    /// Rotate when the current channel changes.
    pub rotate_on_channel_change: bool,
    /// Rotate when the channel codec changes.
    pub rotate_on_codec_change: bool,
}

impl RecordingOptions {
    /// Creates options with a template and format.
    pub fn new(template: impl Into<String>, format: ffi::AudioFileFormat) -> Self {
        Self {
            template: template.into(),
            format,
            start_index: 1,
            max_duration: None,
            max_size_bytes: None,
            rotate_on_channel_change: true,
            rotate_on_codec_change: true,
        }
    }

    /// Sets a maximum segment duration.
    pub fn with_max_duration(mut self, duration: Duration) -> Self {
        self.max_duration = Some(duration);
        self
    }

    /// Sets a maximum segment size in bytes.
    pub fn with_max_size_bytes(mut self, size: u64) -> Self {
        self.max_size_bytes = Some(size);
        self
    }
}

pub(crate) fn segment_path(template: &str, index: u32) -> String {
    if template.contains("{index}") {
        return template.replace("{index}", &index.to_string());
    }

    let base = template.to_string();
    if let Some(pos) = base.rfind('.') {
        let (stem, ext) = base.split_at(pos);
        format!("{stem}.part{index}{ext}")
    } else {
        format!("{base}.part{index}")
    }
}

#[cfg(test)]
mod tests {
    use super::{RecordingOptions, segment_path};
    use std::time::Duration;
    use teamtalk_sys as ffi;

    #[test]
    fn segment_path_replaces_index() {
        let path = segment_path("out_{index}.wav", 5);
        assert_eq!(path, "out_5.wav");
    }

    #[test]
    fn segment_path_inserts_before_extension() {
        let path = segment_path("out.wav", 3);
        assert_eq!(path, "out.part3.wav");
    }

    #[test]
    fn segment_path_appends_when_no_extension() {
        let path = segment_path("out", 2);
        assert_eq!(path, "out.part2");
    }

    #[test]
    fn recording_options_builder_fields() {
        let options =
            RecordingOptions::new("out_{index}.wav", ffi::AudioFileFormat::AFF_WAVE_FORMAT)
                .with_max_duration(Duration::from_secs(5))
                .with_max_size_bytes(1024);
        assert_eq!(options.template, "out_{index}.wav");
        assert_eq!(options.start_index, 1);
        assert_eq!(options.max_duration, Some(Duration::from_secs(5)));
        assert_eq!(options.max_size_bytes, Some(1024));
        assert!(options.rotate_on_channel_change);
        assert!(options.rotate_on_codec_change);
    }
}
