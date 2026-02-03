use teamtalk::client::ffi;
use teamtalk::types::{ClientStatistics, ServerStatistics, UserStatistics};

#[test]
fn client_statistics_from_ffi() {
    let raw = ffi::ClientStatistics {
        nUdpPingTimeMs: 1,
        nTcpPingTimeMs: 2,
        nUdpBytesSent: 3,
        nUdpBytesRecv: 4,
        nVoiceBytesSent: 5,
        nVoiceBytesRecv: 6,
        nVideoCaptureBytesSent: 7,
        nVideoCaptureBytesRecv: 8,
        nMediaFileAudioBytesSent: 9,
        nMediaFileAudioBytesRecv: 10,
        nMediaFileVideoBytesSent: 11,
        nMediaFileVideoBytesRecv: 12,
        nDesktopBytesSent: 13,
        nDesktopBytesRecv: 14,
        ..Default::default()
    };
    let stats = ClientStatistics::from(raw);
    assert_eq!(stats.udp_ping, 1);
    assert_eq!(stats.tcp_ping, 2);
    assert_eq!(stats.udp_sent, 3);
    assert_eq!(stats.udp_recv, 4);
    assert_eq!(stats.voice_sent, 5);
    assert_eq!(stats.voice_recv, 6);
    assert_eq!(stats.video_sent, 7);
    assert_eq!(stats.video_recv, 8);
    assert_eq!(stats.media_audio_sent, 9);
    assert_eq!(stats.media_audio_recv, 10);
    assert_eq!(stats.media_video_sent, 11);
    assert_eq!(stats.media_video_recv, 12);
    assert_eq!(stats.desktop_sent, 13);
    assert_eq!(stats.desktop_recv, 14);
}

#[test]
fn server_statistics_from_ffi() {
    let raw = ffi::ServerStatistics {
        nTotalBytesTX: 1,
        nTotalBytesRX: 2,
        nVoiceBytesTX: 3,
        nVoiceBytesRX: 4,
        nVideoCaptureBytesTX: 5,
        nVideoCaptureBytesRX: 6,
        nMediaFileBytesTX: 7,
        nMediaFileBytesRX: 8,
        nDesktopBytesTX: 9,
        nDesktopBytesRX: 10,
        nUsersServed: 11,
        nUsersPeak: 12,
        nFilesTx: 13,
        nFilesRx: 14,
        nUptimeMSec: 15,
    };
    let stats = ServerStatistics::from(raw);
    assert_eq!(stats.total_tx, 1);
    assert_eq!(stats.total_rx, 2);
    assert_eq!(stats.voice_tx, 3);
    assert_eq!(stats.voice_rx, 4);
    assert_eq!(stats.video_tx, 5);
    assert_eq!(stats.video_rx, 6);
    assert_eq!(stats.media_tx, 7);
    assert_eq!(stats.media_rx, 8);
    assert_eq!(stats.desktop_tx, 9);
    assert_eq!(stats.desktop_rx, 10);
    assert_eq!(stats.users_served, 11);
    assert_eq!(stats.users_peak, 12);
    assert_eq!(stats.files_tx, 13);
    assert_eq!(stats.files_rx, 14);
    assert_eq!(stats.uptime_ms, 15);
}

#[test]
fn user_statistics_from_ffi() {
    let raw = ffi::UserStatistics {
        nVoicePacketsRecv: 1,
        nVoicePacketsLost: 2,
        nVideoCapturePacketsRecv: 3,
        nVideoCaptureFramesRecv: 4,
        nVideoCaptureFramesLost: 5,
        nVideoCaptureFramesDropped: 6,
        nMediaFileAudioPacketsRecv: 7,
        nMediaFileAudioPacketsLost: 8,
        nMediaFileVideoPacketsRecv: 9,
        nMediaFileVideoFramesRecv: 10,
        nMediaFileVideoFramesLost: 11,
        nMediaFileVideoFramesDropped: 12,
    };
    let stats = UserStatistics::from(raw);
    assert_eq!(stats.voice_recv, 1);
    assert_eq!(stats.voice_lost, 2);
    assert_eq!(stats.video_recv, 3);
    assert_eq!(stats.video_frames_recv, 4);
    assert_eq!(stats.video_frames_lost, 5);
    assert_eq!(stats.video_frames_dropped, 6);
    assert_eq!(stats.media_audio_recv, 7);
    assert_eq!(stats.media_audio_lost, 8);
    assert_eq!(stats.media_video_recv, 9);
    assert_eq!(stats.media_video_frames_recv, 10);
    assert_eq!(stats.media_video_frames_lost, 11);
    assert_eq!(stats.media_video_frames_dropped, 12);
}
