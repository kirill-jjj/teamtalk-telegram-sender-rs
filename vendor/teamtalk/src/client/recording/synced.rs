use crate::events::{Error, Event, Result};
use crate::types::{Subscriptions, User, UserId};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use teamtalk_sys as ffi;

use super::super::{Client, Message};
use super::RecordingSampleFormat;
use crate::AudioBlockView;

#[derive(Clone, Debug)]
pub enum SilencePolicy {
    Always,
    OnlyWhileConnected,
    OnlyWhileTalking,
}

#[derive(Clone, Debug)]
pub struct SyncedUserRecordingOptions {
    pub folder: String,
    pub file_vars: String,
    pub format: RecordingSampleFormat,
    pub stream_types: u32,
    pub tick_interval: Duration,
    pub subscribe_audio: bool,
    pub default_sample_rate: Option<i32>,
    pub default_channels: Option<i32>,
    pub silence_policy: SilencePolicy,
}

impl SyncedUserRecordingOptions {
    pub fn new(folder: impl Into<String>) -> Self {
        Self {
            folder: folder.into(),
            file_vars: "user-%user_id%-%username%".to_string(),
            format: RecordingSampleFormat::PcmS16Le,
            stream_types: ffi::StreamType::STREAMTYPE_VOICE as u32,
            tick_interval: Duration::from_millis(250),
            subscribe_audio: true,
            default_sample_rate: None,
            default_channels: None,
            silence_policy: SilencePolicy::Always,
        }
    }

    pub fn with_format(mut self, format: RecordingSampleFormat) -> Self {
        self.format = format;
        self
    }

    pub fn with_file_vars(mut self, vars: impl Into<String>) -> Self {
        self.file_vars = vars.into();
        self
    }

    pub fn with_stream_types(mut self, types: u32) -> Self {
        self.stream_types = types;
        self
    }

    pub fn with_tick_interval(mut self, interval: Duration) -> Self {
        self.tick_interval = interval;
        self
    }

    pub fn with_default_audio_format(mut self, sample_rate: i32, channels: i32) -> Self {
        self.default_sample_rate = Some(sample_rate);
        self.default_channels = Some(channels);
        self
    }

    pub fn with_subscribe_audio(mut self, enabled: bool) -> Self {
        self.subscribe_audio = enabled;
        self
    }

    pub fn with_silence_policy(mut self, policy: SilencePolicy) -> Self {
        self.silence_policy = policy;
        self
    }
}

pub struct SyncedUserRecordingSession {
    options: SyncedUserRecordingOptions,
    start: Instant,
    users: HashMap<UserId, UserTrack>,
    last_tick: Instant,
}

impl SyncedUserRecordingSession {
    pub fn start(client: &Client, options: SyncedUserRecordingOptions) -> Result<Self> {
        fs::create_dir_all(&options.folder).map_err(|e| Error::IoError {
            message: e.to_string(),
        })?;

        let mut session = Self {
            options,
            start: Instant::now(),
            users: HashMap::new(),
            last_tick: Instant::now(),
        };

        session.attach_existing_users(client)?;
        Ok(session)
    }

    pub fn tick(&mut self) -> Result<()> {
        if self.options.tick_interval > Duration::ZERO
            && self.last_tick.elapsed() < self.options.tick_interval
        {
            return Ok(());
        }
        self.last_tick = Instant::now();
        if matches!(self.options.silence_policy, SilencePolicy::OnlyWhileTalking) {
            return Ok(());
        }
        let elapsed = self.start.elapsed();
        for track in self.users.values_mut() {
            track.pad_to(elapsed)?;
        }
        Ok(())
    }

    pub fn handle_event(&mut self, client: &Client, event: Event, message: &Message) -> Result<()> {
        match event {
            Event::UserJoined => {
                if let Some(user) = message.user() {
                    self.start_user(client, user.id, Some(user))?;
                }
            }
            Event::UserLeft => {
                let user_id = message
                    .user()
                    .map(|u| u.id)
                    .unwrap_or(UserId(message.source()));
                if user_id.0 > 0 {
                    self.stop_user(client, user_id);
                }
            }
            Event::AudioBlock => {
                let user_id = UserId(message.source());
                if user_id.0 > 0 {
                    self.on_audio_block(client, user_id)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn attach_existing_users(&mut self, client: &Client) -> Result<()> {
        let channel_id = client.my_channel_id();
        let users = client.get_channel_users(channel_id);
        for user in users {
            self.start_user(client, user.id, Some(user))?;
        }
        Ok(())
    }

    fn start_user(&mut self, client: &Client, user_id: UserId, user: Option<User>) -> Result<()> {
        if self.users.contains_key(&user_id) {
            return Ok(());
        }
        if self.options.subscribe_audio {
            let _ = client.subscribe(user_id, Subscriptions::all_audio());
        }
        let track = UserTrack::new(
            &self.options.folder,
            &self.options.file_vars,
            self.options.format.clone(),
            user_id,
            user,
            self.options.default_sample_rate,
            self.options.default_channels,
        )?;
        client.enable_audio_block_event(user_id, self.options.stream_types, true);
        self.users.insert(user_id, track);
        Ok(())
    }

    fn stop_user(&mut self, client: &Client, user_id: UserId) {
        client.enable_audio_block_event(user_id, self.options.stream_types, false);
        if self.options.subscribe_audio {
            let _ = client.unsubscribe_all_from_user(user_id);
        }
        self.users.remove(&user_id);
    }

    fn on_audio_block(&mut self, client: &Client, user_id: UserId) -> Result<()> {
        let Some(ptr) = client.acquire_user_audio_block(self.options.stream_types, user_id) else {
            return Ok(());
        };
        let block = unsafe { &*ptr };
        let Some(view) = AudioBlockView::from_block(block) else {
            unsafe {
                let _ = client.release_user_audio_block(ptr);
            }
            return Ok(());
        };

        if !self.users.contains_key(&user_id) {
            self.start_user(client, user_id, None)?;
        }

        let elapsed = self.start.elapsed();
        if let Some(track) = self.users.get_mut(&user_id) {
            track.ensure_format(view.sample_rate, view.channels)?;
            track.pad_to(elapsed)?;
            track.write_block(&view)?;
        }
        unsafe {
            let _ = client.release_user_audio_block(ptr);
        }
        Ok(())
    }
}

pub struct SyncedUserRecording {
    session: SyncedUserRecordingSession,
}

impl SyncedUserRecording {
    pub fn start(client: &Client, options: SyncedUserRecordingOptions) -> Result<Self> {
        Ok(Self {
            session: SyncedUserRecordingSession::start(client, options)?,
        })
    }

    pub fn tick(&mut self) -> Result<()> {
        self.session.tick()
    }

    pub fn handle_event(&mut self, client: &Client, event: Event, message: &Message) -> Result<()> {
        self.session.handle_event(client, event, message)
    }
}

pub struct SyncedUserRecordingBus<'a> {
    client: &'a Client,
    group: String,
}

impl<'a> SyncedUserRecordingBus<'a> {
    pub fn attach(
        session: Arc<Mutex<SyncedUserRecordingSession>>,
        client: &'a Client,
        group: impl Into<String>,
    ) -> Self {
        let group_name = group.into();
        let group_filter = group_name.clone();
        let _id = client
            .on_any()
            .group(group_filter)
            .filter(|ctx| {
                matches!(
                    ctx.event(),
                    Event::UserJoined | Event::UserLeft | Event::AudioBlock
                )
            })
            .subscribe(move |ctx| {
                if let Ok(mut session) = session.lock() {
                    let _ = session.handle_event(ctx.client(), ctx.event(), ctx.message());
                }
            });
        Self {
            client,
            group: group_name,
        }
    }
}

impl Drop for SyncedUserRecordingBus<'_> {
    fn drop(&mut self) {
        let _ = self.client.unsubscribe_event_group(&self.group);
    }
}

struct UserTrack {
    writer: TrackWriter,
    samples_written: u64,
    sample_rate: Option<i32>,
    channels: Option<i32>,
}

impl UserTrack {
    fn new(
        folder: &str,
        file_vars: &str,
        format: RecordingSampleFormat,
        user_id: UserId,
        user: Option<User>,
        default_sample_rate: Option<i32>,
        default_channels: Option<i32>,
    ) -> Result<Self> {
        let username = user
            .map(|u| u.username)
            .unwrap_or_else(|| "unknown".to_string());
        let filename = render_vars(file_vars, user_id, &username);
        let path = Path::new(folder).join(filename);
        let mut writer = TrackWriter::new(path, format)?;
        let sample_rate = default_sample_rate;
        let channels = default_channels;
        if let (Some(rate), Some(ch)) = (sample_rate, channels) {
            writer.init(rate, ch)?;
        }
        Ok(Self {
            writer,
            samples_written: 0,
            sample_rate,
            channels,
        })
    }

    fn ensure_format(&mut self, sample_rate: i32, channels: i32) -> Result<()> {
        if self.sample_rate.is_none() {
            self.sample_rate = Some(sample_rate);
            self.channels = Some(channels);
            self.writer.init(sample_rate, channels)?;
        }
        Ok(())
    }

    fn pad_to(&mut self, elapsed: Duration) -> Result<()> {
        let sample_rate = match self.sample_rate {
            Some(rate) => rate,
            None => return Ok(()),
        };
        let channels = self.channels.unwrap_or(1) as u64;
        let target_samples = (elapsed.as_secs_f64() * sample_rate as f64) as u64;
        if target_samples > self.samples_written {
            let missing = target_samples - self.samples_written;
            self.writer.write_silence(missing, channels)?;
            self.samples_written = target_samples;
        }
        Ok(())
    }

    fn write_block(&mut self, block: &AudioBlockView<'_>) -> Result<()> {
        self.writer.write_pcm(block.data)?;
        self.samples_written = self.samples_written.saturating_add(block.samples as u64);
        Ok(())
    }
}

enum TrackWriter {
    Pcm(File),
    Wav(WavWriter),
}

impl TrackWriter {
    fn new(path: PathBuf, format: RecordingSampleFormat) -> Result<Self> {
        let file = File::create(path).map_err(|e| Error::IoError {
            message: e.to_string(),
        })?;
        Ok(match format {
            RecordingSampleFormat::PcmS16Le => TrackWriter::Pcm(file),
            RecordingSampleFormat::WavS16Le => TrackWriter::Wav(WavWriter::new(file)),
        })
    }

    fn init(&mut self, sample_rate: i32, channels: i32) -> Result<()> {
        match self {
            TrackWriter::Pcm(_) => Ok(()),
            TrackWriter::Wav(writer) => writer.init(sample_rate, channels),
        }
    }

    fn write_pcm(&mut self, data: &[i16]) -> Result<()> {
        let bytes = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data))
        };
        match self {
            TrackWriter::Pcm(file) => file.write_all(bytes).map_err(|e| Error::IoError {
                message: e.to_string(),
            }),
            TrackWriter::Wav(writer) => writer.write(bytes),
        }
    }

    fn write_silence(&mut self, samples: u64, channels: u64) -> Result<()> {
        let count = samples.saturating_mul(channels) as usize;
        let buf = vec![0i16; count];
        self.write_pcm(&buf)
    }
}

struct WavWriter {
    file: File,
    data_bytes: u32,
    sample_rate: i32,
    channels: i32,
}

impl WavWriter {
    fn new(file: File) -> Self {
        Self {
            file,
            data_bytes: 0,
            sample_rate: 0,
            channels: 0,
        }
    }

    fn init(&mut self, sample_rate: i32, channels: i32) -> Result<()> {
        self.sample_rate = sample_rate;
        self.channels = channels;
        self.write_header(0)
    }

    fn write(&mut self, data: &[u8]) -> Result<()> {
        self.file.write_all(data).map_err(|e| Error::IoError {
            message: e.to_string(),
        })?;
        self.data_bytes = self.data_bytes.saturating_add(data.len() as u32);
        Ok(())
    }

    fn write_header(&mut self, data_bytes: u32) -> Result<()> {
        let byte_rate = self.sample_rate as u32 * self.channels as u32 * 2;
        let block_align = self.channels as u16 * 2;
        let mut header = Vec::with_capacity(44);
        header.extend_from_slice(b"RIFF");
        header.extend_from_slice(&(36 + data_bytes).to_le_bytes());
        header.extend_from_slice(b"WAVEfmt ");
        header.extend_from_slice(&(16u32).to_le_bytes());
        header.extend_from_slice(&(1u16).to_le_bytes());
        header.extend_from_slice(&(self.channels as u16).to_le_bytes());
        header.extend_from_slice(&(self.sample_rate as u32).to_le_bytes());
        header.extend_from_slice(&byte_rate.to_le_bytes());
        header.extend_from_slice(&block_align.to_le_bytes());
        header.extend_from_slice(&(16u16).to_le_bytes());
        header.extend_from_slice(b"data");
        header.extend_from_slice(&data_bytes.to_le_bytes());
        self.file.write_all(&header).map_err(|e| Error::IoError {
            message: e.to_string(),
        })
    }
}

impl Drop for WavWriter {
    fn drop(&mut self) {
        let _ = self.file.seek(SeekFrom::Start(0));
        let _ = self.write_header(self.data_bytes);
    }
}

fn render_vars(template: &str, user_id: UserId, username: &str) -> String {
    template
        .replace("%user_id%", &user_id.0.to_string())
        .replace("%username%", username)
}
