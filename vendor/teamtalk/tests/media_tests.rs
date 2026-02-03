use teamtalk::client::ffi;
use teamtalk::types::{MediaFileInfo, RemoteFile, SoundDevice};
use teamtalk::utils::strings::ToTT;

fn copy_tt(src: &str, dst: &mut [ffi::TTCHAR]) {
    let tt = src.tt();
    let len = tt.len().min(dst.len());
    dst[..len].copy_from_slice(&tt[..len]);
}

#[test]
fn media_file_info_from_ffi() {
    let mut raw = ffi::MediaFileInfo {
        nStatus: ffi::MediaFileStatus::MFS_FINISHED,
        ..Default::default()
    };
    raw.audioFmt.nAudioFmt = ffi::AudioFileFormat::AFF_WAVE_FORMAT;
    raw.audioFmt.nSampleRate = 48_000;
    raw.audioFmt.nChannels = 2;
    raw.videoFmt.nWidth = 640;
    raw.videoFmt.nHeight = 480;
    raw.uDurationMSec = 100;
    raw.uElapsedMSec = 25;
    copy_tt("file.wav", &mut raw.szFileName);
    let info = MediaFileInfo::from(raw);
    assert_eq!(info.status, ffi::MediaFileStatus::MFS_FINISHED);
    assert_eq!(info.name, "file.wav");
    assert_eq!(info.audio_fmt.nSampleRate, 48_000);
    assert_eq!(info.audio_fmt.nChannels, 2);
    assert_eq!(info.video_fmt.nWidth, 640);
    assert_eq!(info.video_fmt.nHeight, 480);
    assert_eq!(info.duration_ms, 100);
    assert_eq!(info.elapsed_ms, 25);
}

#[test]
fn media_file_info_snapshot() {
    let mut raw = ffi::MediaFileInfo {
        nStatus: ffi::MediaFileStatus::MFS_ERROR,
        ..Default::default()
    };
    raw.audioFmt.nAudioFmt = ffi::AudioFileFormat::AFF_WAVE_FORMAT;
    raw.audioFmt.nSampleRate = 44_100;
    raw.audioFmt.nChannels = 1;
    raw.videoFmt.nWidth = 320;
    raw.videoFmt.nHeight = 240;
    raw.uDurationMSec = 10;
    raw.uElapsedMSec = 2;
    copy_tt("snap.wav", &mut raw.szFileName);
    let info = MediaFileInfo::from(raw);
    insta::assert_debug_snapshot!(info);
}

#[test]
fn remote_file_from_ffi() {
    let mut raw = ffi::RemoteFile {
        nChannelID: 3,
        nFileID: 4,
        nFileSize: 1024,
        ..Default::default()
    };
    copy_tt("data.bin", &mut raw.szFileName);
    copy_tt("owner", &mut raw.szUsername);
    copy_tt("time", &mut raw.szUploadTime);
    let file = RemoteFile::from(raw);
    assert_eq!(file.channel_id.0, 3);
    assert_eq!(file.id.0, 4);
    assert_eq!(file.size, 1024);
    assert_eq!(file.name, "data.bin");
    assert_eq!(file.owner, "owner");
    assert_eq!(file.upload_time, "time");
}

#[test]
fn sound_device_from_ffi() {
    let mut raw = ffi::SoundDevice {
        nDeviceID: 10,
        nSoundSystem: ffi::SoundSystem::SOUNDSYSTEM_DSOUND,
        nMaxInputChannels: 2,
        nMaxOutputChannels: 2,
        nDefaultSampleRate: 48_000,
        uSoundDeviceFeatures: 5,
        ..Default::default()
    };
    raw.inputSampleRates[0] = 48_000;
    raw.inputSampleRates[1] = 0;
    raw.outputSampleRates[0] = 48_000;
    raw.outputSampleRates[1] = 0;
    copy_tt("dev", &mut raw.szDeviceName);
    copy_tt("uid", &mut raw.szDeviceID);
    let device = SoundDevice::from(raw);
    assert_eq!(device.id, 10);
    assert_eq!(device.system, ffi::SoundSystem::SOUNDSYSTEM_DSOUND);
    assert_eq!(device.max_input_channels, 2);
    assert_eq!(device.max_output_channels, 2);
    assert_eq!(device.default_sample_rate, 48_000);
    assert_eq!(device.features, 5);
    assert_eq!(device.name, "dev");
    assert_eq!(device.device_uid, "uid");
    assert_eq!(device.input_sample_rates, vec![48_000]);
    assert_eq!(device.output_sample_rates, vec![48_000]);
}
