//! Windows mixer control APIs.
#[cfg(windows)]
use super::Client;
#[cfg(windows)]
use crate::types::TT_STRLEN;
#[cfg(windows)]
use crate::utils::strings::tt_buf;
#[cfg(windows)]
use teamtalk_sys as ffi;

#[cfg(windows)]
impl Client {
    /// Returns the number of mixer devices.
    pub fn get_mixer_count(&self) -> i32 {
        unsafe { ffi::api().TT_Mixer_GetMixerCount() }
    }

    /// Returns the mixer name by index.
    pub fn get_mixer_name(&self, index: i32) -> String {
        let mut buf = tt_buf::<TT_STRLEN>();
        unsafe {
            if ffi::api().TT_Mixer_GetMixerName(index, buf.as_mut_ptr()) == 1 {
                crate::utils::strings::to_string(&buf)
            } else {
                String::new()
            }
        }
    }

    /// Mutes or unmutes mixer output.
    pub fn set_mixer_output_mute(&self, wave_id: i32, control: ffi::MixerControl, mute: bool) -> bool {
        unsafe { ffi::api().TT_Mixer_SetWaveOutMute(wave_id, control, mute as i32) == 1 }
    }

    /// Returns mixer output mute state.
    pub fn get_mixer_output_mute(&self, wave_id: i32, control: ffi::MixerControl) -> i32 {
        unsafe { ffi::api().TT_Mixer_GetWaveOutMute(wave_id, control) }
    }

    /// Sets mixer output volume.
    pub fn set_mixer_output_volume(&self, wave_id: i32, control: ffi::MixerControl, vol: i32) -> bool {
        unsafe { ffi::api().TT_Mixer_SetWaveOutVolume(wave_id, control, vol) == 1 }
    }

    /// Returns mixer output volume.
    pub fn get_mixer_output_volume(&self, wave_id: i32, control: ffi::MixerControl) -> i32 {
        unsafe { ffi::api().TT_Mixer_GetWaveOutVolume(wave_id, control) }
    }

    /// Mutes or unmutes mixer input.
    pub fn set_mixer_input_mute(&self, wave_id: i32, mute: bool) -> bool {
        unsafe { ffi::api().TT_Mixer_SetWaveInMute(wave_id, mute as i32) == 1 }
    }

    /// Returns mixer input mute state.
    pub fn get_mixer_input_mute(&self, wave_id: i32) -> i32 {
        unsafe { ffi::api().TT_Mixer_GetWaveInMute(wave_id) }
    }

    /// Returns mixer input device name.
    pub fn get_mixer_wave_in_name(&self, wave_id: i32) -> String {
        let mut buf = tt_buf::<TT_STRLEN>();
        unsafe {
            if ffi::api().TT_Mixer_GetWaveInName(wave_id, buf.as_mut_ptr()) == 1 {
                crate::utils::strings::to_string(&buf)
            } else {
                String::new()
            }
        }
    }

    /// Returns mixer output device name.
    pub fn get_mixer_wave_out_name(&self, wave_id: i32) -> String {
        let mut buf = tt_buf::<TT_STRLEN>();
        unsafe {
            if ffi::api().TT_Mixer_GetWaveOutName(wave_id, buf.as_mut_ptr()) == 1 {
                crate::utils::strings::to_string(&buf)
            } else {
                String::new()
            }
        }
    }

    /// Enables or disables mixer input boost.
    pub fn set_mixer_input_boost(&self, wave_id: i32, enable: bool) -> bool {
        unsafe { ffi::api().TT_Mixer_SetWaveInBoost(wave_id, enable as i32) == 1 }
    }

    /// Returns mixer input boost state.
    pub fn get_mixer_input_boost(&self, wave_id: i32) -> i32 {
        unsafe { ffi::api().TT_Mixer_GetWaveInBoost(wave_id) }
    }

    /// Returns the number of mixer input controls.
    pub fn get_mixer_input_control_count(&self, wave_id: i32) -> i32 {
        unsafe { ffi::api().TT_Mixer_GetWaveInControlCount(wave_id) }
    }

    /// Returns a mixer input control name.
    pub fn get_mixer_input_control_name(&self, wave_id: i32, index: i32) -> String {
        let mut buf = tt_buf::<TT_STRLEN>();
        unsafe {
            if ffi::api().TT_Mixer_GetWaveInControlName(wave_id, index, buf.as_mut_ptr()) == 1 {
                crate::utils::strings::to_string(&buf)
            } else {
                String::new()
            }
        }
    }

    /// Selects a mixer input control.
    pub fn set_mixer_input_control_selected(&self, wave_id: i32, index: i32) -> bool {
        unsafe { ffi::api().TT_Mixer_SetWaveInControlSelected(wave_id, index) == 1 }
    }

    /// Returns whether a mixer input control is selected.
    pub fn get_mixer_input_control_selected(&self, wave_id: i32, index: i32) -> bool {
        unsafe { ffi::api().TT_Mixer_GetWaveInControlSelected(wave_id, index) == 1 }
    }
}
