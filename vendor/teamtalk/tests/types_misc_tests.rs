use teamtalk::client::ffi;
use teamtalk::types::{
    AudioInputProgress, BannedUser, ChannelId, FileTransfer, TransferId, VideoFrame,
};
use teamtalk::utils::strings::ToTT;

fn copy_tt(src: &str, dst: &mut [ffi::TTCHAR]) {
    let tt = src.tt();
    let len = tt.len().min(dst.len());
    dst[..len].copy_from_slice(&tt[..len]);
}

#[test]
fn audio_input_progress_from_ffi() {
    let raw = ffi::AudioInputProgress {
        nStreamID: 7,
        uQueueMSec: 10,
        uElapsedMSec: 20,
    };
    let progress = AudioInputProgress::from(raw);
    assert_eq!(progress.stream_id, 7);
    assert_eq!(progress.queue_ms, 10);
    assert_eq!(progress.elapsed_ms, 20);
}

#[test]
fn video_frame_from_ffi() {
    let raw = ffi::VideoFrame {
        nWidth: 640,
        nHeight: 480,
        nStreamID: 3,
        bKeyFrame: 1,
        frameBuffer: std::ptr::null_mut(),
        nFrameBufferSize: 123,
    };
    let frame = VideoFrame::from(raw);
    assert_eq!(frame.width, 640);
    assert_eq!(frame.height, 480);
    assert_eq!(frame.stream_id, 3);
    assert!(frame.key_frame);
    assert_eq!(frame.buf_len, 123);
}

#[test]
fn banned_user_from_ffi() {
    let mut raw = ffi::BannedUser {
        uBanTypes: 5,
        ..Default::default()
    };
    copy_tt("1.1.1.1", &mut raw.szIPAddress);
    copy_tt("/root", &mut raw.szChannelPath);
    copy_tt("nick", &mut raw.szNickname);
    copy_tt("user", &mut raw.szUsername);
    copy_tt("now", &mut raw.szBanTime);
    copy_tt("owner", &mut raw.szOwner);
    let user = BannedUser::from(raw);
    assert_eq!(user.ip, "1.1.1.1");
    assert_eq!(user.channel_path, "/root");
    assert_eq!(user.nickname, "nick");
    assert_eq!(user.username, "user");
    assert_eq!(user.ban_time, "now");
    assert_eq!(user.owner, "owner");
    assert_eq!(user.ban_types, 5);
}

#[test]
fn file_transfer_from_ffi() {
    let mut raw = ffi::FileTransfer {
        nStatus: ffi::FileTransferStatus::FILETRANSFER_ACTIVE,
        nTransferID: 7,
        nChannelID: 9,
        nFileSize: 100,
        nTransferred: 25,
        bInbound: 1,
        ..Default::default()
    };
    copy_tt("local", &mut raw.szLocalFilePath);
    copy_tt("remote", &mut raw.szRemoteFileName);
    let transfer = FileTransfer::from(raw);
    assert_eq!(transfer.id, TransferId(7));
    assert_eq!(transfer.channel_id, ChannelId(9));
    assert_eq!(transfer.size, 100);
    assert_eq!(transfer.transferred, 25);
    assert!(transfer.inbound);
    assert_eq!(transfer.local_path, "local");
    assert_eq!(transfer.remote_name, "remote");
}
