//! Audio device and audio stream APIs.
use super::Client;
use crate::events::Event;
use crate::types::{AudioPreprocessor, SoundDevice, UserId};
use std::io::Write;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use teamtalk_sys as ffi;

/// Audio device selection preset.
#[derive(Debug, Clone, Copy)]
pub struct AudioDeviceProfile {
    pub input_id: i32,
    pub output_id: i32,
    pub duplex: bool,
}

impl AudioDeviceProfile {
    /// Creates a split input/output profile.
    pub fn split(input_id: i32, output_id: i32) -> Self {
        Self {
            input_id,
            output_id,
            duplex: false,
        }
    }

    /// Creates a duplex input/output profile.
    pub fn duplex(input_id: i32, output_id: i32) -> Self {
        Self {
            input_id,
            output_id,
            duplex: true,
        }
    }
}
impl Client {
    /// Returns available sound devices.
    pub fn get_sound_devices(&self) -> Vec<SoundDevice> {
        let mut count: i32 = 0;
        unsafe {
            ffi::api().TT_GetSoundDevices(std::ptr::null_mut(), &mut count);
            let mut devices = vec![std::mem::zeroed::<ffi::SoundDevice>(); count as usize];
            if ffi::api().TT_GetSoundDevices(devices.as_mut_ptr(), &mut count) == 1 {
                devices.into_iter().map(SoundDevice::from).collect()
            } else {
                vec![]
            }
        }
    }

    /// Returns default input and output device ids.
    pub fn get_default_sound_devices(&self) -> (i32, i32) {
        let mut input: i32 = 0;
        let mut output: i32 = 0;
        unsafe {
            ffi::api().TT_GetDefaultSoundDevices(&mut input, &mut output);
        }
        (input, output)
    }

    /// Returns default input and output device ids for a sound system.
    pub fn get_default_sound_devices_ex(&self, system: ffi::SoundSystem) -> (i32, i32) {
        let mut input: i32 = 0;
        let mut output: i32 = 0;
        unsafe {
            ffi::api().TT_GetDefaultSoundDevicesEx(system, &mut input, &mut output);
        }
        (input, output)
    }

    /// Restarts the global sound system.
    pub fn restart_sound_system(&self) -> bool {
        unsafe { ffi::api().TT_RestartSoundSystem() == 1 }
    }

    /// Initializes the sound input device by id.
    pub fn init_sound_input_device(&self, device_id: i32) -> bool {
        unsafe { ffi::api().TT_InitSoundInputDevice(self.ptr.0, device_id) == 1 }
    }

    /// Initializes the sound output device by id.
    pub fn init_sound_output_device(&self, device_id: i32) -> bool {
        unsafe { ffi::api().TT_InitSoundOutputDevice(self.ptr.0, device_id) == 1 }
    }

    /// Initializes a shared input device.
    pub fn init_sound_input_shared_device(&self, rate: i32, chans: i32, frame: i32) -> bool {
        unsafe { ffi::api().TT_InitSoundInputSharedDevice(rate, chans, frame) == 1 }
    }

    /// Initializes a shared output device.
    pub fn init_sound_output_shared_device(&self, rate: i32, chans: i32, frame: i32) -> bool {
        unsafe { ffi::api().TT_InitSoundOutputSharedDevice(rate, chans, frame) == 1 }
    }

    /// Initializes duplex input/output devices.
    pub fn init_sound_duplex_devices(&self, in_id: i32, out_id: i32) -> bool {
        unsafe { ffi::api().TT_InitSoundDuplexDevices(self.ptr.0, in_id, out_id) == 1 }
    }

    /// Returns an audio profile using default input and output devices.
    pub fn default_audio_profile(&self) -> AudioDeviceProfile {
        let (input, output) = self.get_default_sound_devices();
        AudioDeviceProfile::split(input, output)
    }

    /// Applies an audio profile to the client.
    pub fn apply_audio_profile(&self, profile: AudioDeviceProfile) -> bool {
        if profile.duplex {
            self.init_sound_duplex_devices(profile.input_id, profile.output_id)
        } else {
            self.init_sound_input_device(profile.input_id)
                && self.init_sound_output_device(profile.output_id)
        }
    }

    /// Closes the sound input device.
    pub fn close_sound_input_device(&self) -> bool {
        unsafe { ffi::api().TT_CloseSoundInputDevice(self.ptr.0) == 1 }
    }

    /// Closes the sound output device.
    pub fn close_sound_output_device(&self) -> bool {
        unsafe { ffi::api().TT_CloseSoundOutputDevice(self.ptr.0) == 1 }
    }

    /// Closes duplex input/output devices.
    pub fn close_sound_duplex_devices(&self) -> bool {
        unsafe { ffi::api().TT_CloseSoundDuplexDevices(self.ptr.0) == 1 }
    }

    /// Returns current sound input level.
    pub fn get_sound_input_level(&self) -> i32 {
        unsafe { ffi::api().TT_GetSoundInputLevel(self.ptr.0) }
    }

    /// Sets the sound input gain level.
    pub fn set_sound_input_gain_level(&self, level: i32) -> bool {
        unsafe { ffi::api().TT_SetSoundInputGainLevel(self.ptr.0, level) == 1 }
    }

    /// Returns the sound input gain level.
    pub fn get_sound_input_gain_level(&self) -> i32 {
        unsafe { ffi::api().TT_GetSoundInputGainLevel(self.ptr.0) }
    }

    /// Sets output volume.
    pub fn set_sound_output_volume(&self, volume: i32) -> bool {
        unsafe { ffi::api().TT_SetSoundOutputVolume(self.ptr.0, volume) == 1 }
    }

    /// Returns output volume.
    pub fn get_sound_output_volume(&self) -> i32 {
        unsafe { ffi::api().TT_GetSoundOutputVolume(self.ptr.0) }
    }

    /// Mutes or unmutes output audio.
    pub fn set_sound_output_mute(&self, mute: bool) -> bool {
        unsafe { ffi::api().TT_SetSoundOutputMute(self.ptr.0, if mute { 1 } else { 0 }) == 1 }
    }

    /// Mutes or unmutes a user stream.
    pub fn set_user_mute(&self, user_id: UserId, stream_type: ffi::StreamType, mute: bool) -> bool {
        unsafe {
            ffi::api().TT_SetUserMute(self.ptr.0, user_id.0, stream_type, if mute { 1 } else { 0 })
                == 1
        }
    }

    /// Sets the user audio stream buffer size.
    pub fn set_user_audio_stream_buffer_size(
        &self,
        user_id: UserId,
        stream_type: ffi::StreamType,
        msec: i32,
    ) -> bool {
        unsafe {
            let st = stream_type as u32;
            ffi::api().TT_SetUserAudioStreamBufferSize(self.ptr.0, user_id.0, st, msec) == 1
        }
    }

    /// Sets stopped playback delay for a user stream.
    pub fn set_user_stopped_playback_delay(
        &self,
        user_id: UserId,
        stream_type: ffi::StreamType,
        msec: i32,
    ) -> bool {
        unsafe {
            ffi::api().TT_SetUserStoppedPlaybackDelay(self.ptr.0, user_id.0, stream_type, msec) == 1
        }
    }

    /// Enables or disables voice transmission.
    pub fn enable_voice_transmission(&self, enable: bool) -> bool {
        unsafe {
            ffi::api().TT_EnableVoiceTransmission(self.ptr.0, if enable { 1 } else { 0 }) == 1
        }
    }

    /// Enables or disables voice activation.
    pub fn enable_voice_activation(&self, enable: bool) -> bool {
        unsafe { ffi::api().TT_EnableVoiceActivation(self.ptr.0, if enable { 1 } else { 0 }) == 1 }
    }

    /// Sets the voice activation level.
    pub fn set_voice_activation_level(&self, level: i32) -> bool {
        unsafe { ffi::api().TT_SetVoiceActivationLevel(self.ptr.0, level) == 1 }
    }

    /// Returns the voice activation level.
    pub fn get_voice_activation_level(&self) -> i32 {
        unsafe { ffi::api().TT_GetVoiceActivationLevel(self.ptr.0) }
    }

    /// Sets the voice activation stop delay.
    pub fn set_voice_activation_stop_delay(&self, delay: i32) -> bool {
        unsafe { ffi::api().TT_SetVoiceActivationStopDelay(self.ptr.0, delay) == 1 }
    }

    /// Returns the voice activation stop delay.
    pub fn get_voice_activation_stop_delay(&self) -> i32 {
        unsafe { ffi::api().TT_GetVoiceActivationStopDelay(self.ptr.0) }
    }

    /// Sets the audio preprocessor configuration.
    pub fn set_audio_preprocessor(&self, preprocessor: &AudioPreprocessor) -> bool {
        unsafe { ffi::api().TT_SetSoundInputPreprocessEx(self.ptr.0, &preprocessor.to_ffi()) == 1 }
    }

    /// Returns the audio preprocessor configuration.
    pub fn get_audio_preprocessor(&self) -> Option<AudioPreprocessor> {
        let mut raw = unsafe { std::mem::zeroed::<ffi::AudioPreprocessor>() };
        if unsafe { ffi::api().TT_GetSoundInputPreprocessEx(self.ptr.0, &mut raw) } == 1 {
            Some(AudioPreprocessor::from(raw))
        } else {
            None
        }
    }

    /// Sets sound device effects.
    pub fn set_device_effects(&self, effects: &ffi::SoundDeviceEffects) -> bool {
        unsafe { ffi::api().TT_SetSoundDeviceEffects(self.ptr.0, effects) == 1 }
    }

    /// Returns sound device effects.
    pub fn get_device_effects(&self) -> Option<ffi::SoundDeviceEffects> {
        let mut raw = unsafe { std::mem::zeroed::<ffi::SoundDeviceEffects>() };
        if unsafe { ffi::api().TT_GetSoundDeviceEffects(self.ptr.0, &mut raw) } == 1 {
            Some(raw)
        } else {
            None
        }
    }

    /// Enables or disables 3D sound positioning.
    pub fn enable_3d_sound(&self, enable: bool) -> bool {
        unsafe {
            ffi::api().TT_Enable3DSoundPositioning(self.ptr.0, if enable { 1 } else { 0 }) == 1
        }
    }

    /// Automatically positions users in 3D space.
    pub fn auto_position_users(&self) -> bool {
        unsafe { ffi::api().TT_AutoPositionUsers(self.ptr.0) == 1 }
    }

    /// Sets a user's 3D position.
    pub fn set_user_position(
        &self,
        user_id: UserId,
        stream_type: ffi::StreamType,
        x: f32,
        y: f32,
        z: f32,
    ) -> bool {
        unsafe { ffi::api().TT_SetUserPosition(self.ptr.0, user_id.0, stream_type, x, y, z) == 1 }
    }

    /// Sets a user's stereo playback.
    pub fn set_user_stereo(
        &self,
        user_id: UserId,
        stream_type: ffi::StreamType,
        left: bool,
        right: bool,
    ) -> bool {
        unsafe {
            ffi::api().TT_SetUserStereo(
                self.ptr.0,
                user_id.0,
                stream_type,
                left as i32,
                right as i32,
            ) == 1
        }
    }

    /// Enables audio block events for a user.
    pub fn enable_audio_block_event(&self, user_id: UserId, types: u32, enable: bool) -> bool {
        unsafe {
            ffi::api().TT_EnableAudioBlockEvent(
                self.ptr.0,
                user_id.0,
                types,
                if enable { 1 } else { 0 },
            ) == 1
        }
    }

    /// Sets jitter control for a user.
    pub fn set_user_jitter_control(&self, user_id: UserId, delay: i32) -> bool {
        let mut cfg = unsafe { std::mem::zeroed::<ffi::JitterConfig>() };
        cfg.nFixedDelayMSec = delay;
        unsafe {
            ffi::api().TT_SetUserJitterControl(
                self.ptr.0,
                user_id.0,
                ffi::StreamType::STREAMTYPE_VOICE,
                &cfg,
            ) == 1
        }
    }

    /// Enables audio block events with a custom format.
    pub fn enable_audio_block_event_ex(
        &self,
        user_id: UserId,
        types: u32,
        format: Option<&ffi::AudioFormat>,
        enable: bool,
    ) -> bool {
        let fmt_ptr = format.map_or(std::ptr::null(), |f| f);
        unsafe {
            ffi::api().TT_EnableAudioBlockEventEx(
                self.ptr.0,
                user_id.0,
                types,
                fmt_ptr,
                if enable { 1 } else { 0 },
            ) == 1
        }
    }

    /// Subscribes to audio blocks for a user and streams them into the sink.
    pub fn stream_audio_blocks<S>(
        &self,
        user_id: UserId,
        types: u32,
        sink: S,
    ) -> AudioBlockSubscription<'_>
    where
        S: AudioBlockSink + Send + 'static,
    {
        let _ = self.enable_audio_block_event(user_id, types, true);
        self.subscribe_audio_blocks(user_id, types, sink)
    }

    /// Subscribes to audio blocks with a custom format.
    pub fn stream_audio_blocks_ex<S>(
        &self,
        user_id: UserId,
        types: u32,
        format: Option<&ffi::AudioFormat>,
        sink: S,
    ) -> AudioBlockSubscription<'_>
    where
        S: AudioBlockSink + Send + 'static,
    {
        let _ = self.enable_audio_block_event_ex(user_id, types, format, true);
        self.subscribe_audio_blocks(user_id, types, sink)
    }

    #[allow(clippy::too_many_arguments)]
    /// Starts a sound loopback test with additional options.
    pub fn start_sound_loopback_test_ex(
        &self,
        in_id: i32,
        out_id: i32,
        rate: i32,
        chans: i32,
        duplex: bool,
        preprocessor: Option<&AudioPreprocessor>,
        effects: Option<&ffi::SoundDeviceEffects>,
    ) -> *mut ffi::TTSoundLoop {
        let prep_raw = preprocessor.map(|p| p.to_ffi());
        let prep_ptr = prep_raw.as_ref().map_or(std::ptr::null(), |p| p);
        let eff_ptr = effects.map_or(std::ptr::null(), |e| e);
        unsafe {
            ffi::api().TT_StartSoundLoopbackTestEx(
                in_id,
                out_id,
                rate,
                chans,
                if duplex { 1 } else { 0 },
                prep_ptr,
                eff_ptr,
            )
        }
    }

    /// Returns jitter control settings for a user.
    pub fn get_user_jitter_control(&self, user_id: UserId) -> Option<crate::types::JitterConfig> {
        let mut raw = unsafe { std::mem::zeroed::<ffi::JitterConfig>() };
        if unsafe {
            ffi::api().TT_GetUserJitterControl(
                self.ptr.0,
                user_id.0,
                ffi::StreamType::STREAMTYPE_VOICE,
                &mut raw,
            ) == 1
        } {
            Some(crate::types::JitterConfig::from(raw))
        } else {
            None
        }
    }

    /// Acquires an audio block for a user.
    pub fn acquire_user_audio_block(
        &self,
        types: u32,
        user_id: UserId,
    ) -> Option<*mut ffi::AudioBlock> {
        unsafe {
            let ptr = ffi::api().TT_AcquireUserAudioBlock(self.ptr.0, types, user_id.0);
            if ptr.is_null() { None } else { Some(ptr) }
        }
    }

    /// Inserts an audio block into the mixer.
    pub fn insert_audio_block(&self, block: &ffi::AudioBlock) -> bool {
        unsafe { ffi::api().TT_InsertAudioBlock(self.ptr.0, block) == 1 }
    }

    /// Releases a previously acquired audio block.
    ///
    /// # Safety
    /// - `block` must be a pointer returned by `acquire_user_audio_block`.
    /// - The block must not be released more than once.
    /// - The pointer must not be used after release.
    pub unsafe fn release_user_audio_block(&self, block: *mut ffi::AudioBlock) -> bool {
        if block.is_null() {
            return false;
        }
        unsafe { ffi::api().TT_ReleaseUserAudioBlock(self.ptr.0, block) == 1 }
    }

    /// Sets a user's stream volume.
    pub fn set_user_volume(
        &self,
        user_id: UserId,
        stream_type: ffi::StreamType,
        volume: i32,
    ) -> bool {
        unsafe { ffi::api().TT_SetUserVolume(self.ptr.0, user_id.0, stream_type, volume) == 1 }
    }

    /// Starts a sound loopback test.
    pub fn start_sound_loopback_test(
        &self,
        in_id: i32,
        out_id: i32,
        rate: i32,
        chans: i32,
        duplex: bool,
    ) -> *mut ffi::TTSoundLoop {
        unsafe {
            ffi::api().TT_StartSoundLoopbackTest(
                in_id,
                out_id,
                rate,
                chans,
                if duplex { 1 } else { 0 },
                std::ptr::null(),
            )
        }
    }

    /// Closes a sound loopback test handle.
    ///
    /// # Safety
    /// - `loopback` must be a pointer returned by `start_sound_loopback_test`.
    /// - The handle must not be closed more than once.
    pub unsafe fn close_sound_loopback_test(&self, loopback: *mut ffi::TTSoundLoop) -> bool {
        if loopback.is_null() {
            return false;
        }
        unsafe { ffi::api().TT_CloseSoundLoopbackTest(loopback) == 1 }
    }

    fn subscribe_audio_blocks<S>(
        &self,
        user_id: UserId,
        types: u32,
        sink: S,
    ) -> AudioBlockSubscription<'_>
    where
        S: AudioBlockSink + Send + 'static,
    {
        let sink = Arc::new(Mutex::new(sink));
        let sink_ref = Arc::clone(&sink);
        let subscription_id = self
            .on_event(Event::AudioBlock)
            .filter_user(user_id)
            .subscribe(move |ctx| {
                let client = ctx.client();
                let Some(ptr) = client.acquire_user_audio_block(types, user_id) else {
                    return;
                };
                let block = unsafe { &*ptr };
                if let Some(view) = AudioBlockView::from_block(block)
                    && let Ok(mut sink) = sink_ref.lock()
                {
                    sink.handle(&view);
                }
                unsafe {
                    let _ = client.release_user_audio_block(ptr);
                }
            });

        AudioBlockSubscription {
            client: self,
            subscription_id,
            user_id,
            stream_types: types,
        }
    }
}

/// View of raw audio block data.
pub struct AudioBlockView<'a> {
    pub sample_rate: i32,
    pub channels: i32,
    pub stream_types: u32,
    pub samples: i32,
    pub data: &'a [i16],
}

impl<'a> AudioBlockView<'a> {
    pub fn from_block(block: &'a ffi::AudioBlock) -> Option<Self> {
        if block.lpRawAudio.is_null() || block.nSamples <= 0 || block.nChannels <= 0 {
            return None;
        }
        let count = block.nSamples.saturating_mul(block.nChannels) as usize;
        let ptr = block.lpRawAudio as *const i16;
        let data = unsafe { std::slice::from_raw_parts(ptr, count) };
        Some(Self {
            sample_rate: block.nSampleRate,
            channels: block.nChannels,
            stream_types: block.uStreamTypes,
            samples: block.nSamples,
            data,
        })
    }
}

/// Sink for streaming audio blocks.
pub trait AudioBlockSink {
    fn handle(&mut self, block: &AudioBlockView<'_>);
}

/// Sink backed by a callback.
pub struct CallbackSink<F>(pub F);

impl<F> AudioBlockSink for CallbackSink<F>
where
    F: FnMut(&AudioBlockView<'_>),
{
    fn handle(&mut self, block: &AudioBlockView<'_>) {
        (self.0)(block);
    }
}

/// Sink that writes raw PCM data into a writer.
pub struct WriterSink<W> {
    writer: W,
}

impl<W> WriterSink<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W> AudioBlockSink for WriterSink<W>
where
    W: Write,
{
    fn handle(&mut self, block: &AudioBlockView<'_>) {
        let bytes = unsafe {
            std::slice::from_raw_parts(
                block.data.as_ptr() as *const u8,
                std::mem::size_of_val(block.data),
            )
        };
        let _ = self.writer.write_all(bytes);
    }
}

/// Sink that sends raw PCM data over UDP.
pub struct UdpSink {
    socket: UdpSocket,
}

impl UdpSink {
    pub fn connect(addr: &str) -> std::io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(addr)?;
        Ok(Self { socket })
    }
}

impl AudioBlockSink for UdpSink {
    fn handle(&mut self, block: &AudioBlockView<'_>) {
        let bytes = unsafe {
            std::slice::from_raw_parts(
                block.data.as_ptr() as *const u8,
                std::mem::size_of_val(block.data),
            )
        };
        let _ = self.socket.send(bytes);
    }
}

/// Guard that cleans up an audio block subscription.
pub struct AudioBlockSubscription<'a> {
    client: &'a Client,
    subscription_id: crate::client::EventSubscriptionId,
    user_id: UserId,
    stream_types: u32,
}

impl Drop for AudioBlockSubscription<'_> {
    fn drop(&mut self) {
        let _ = self
            .client
            .enable_audio_block_event(self.user_id, self.stream_types, false);
        let _ = self.client.unsubscribe_event(self.subscription_id);
    }
}
