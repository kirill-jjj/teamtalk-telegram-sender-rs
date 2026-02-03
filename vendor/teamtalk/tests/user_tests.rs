use teamtalk::client::ffi;
use teamtalk::types::{ChannelId, Subscriptions, User, UserGender, UserId, UserPresence};
use teamtalk::utils::strings::ToTT;

fn copy_tt(src: &str, dst: &mut [ffi::TTCHAR]) {
    let tt = src.tt();
    let len = tt.len().min(dst.len());
    dst[..len].copy_from_slice(&tt[..len]);
}

#[test]
fn user_from_ffi_sets_fields() {
    let mut raw = ffi::User {
        nUserID: 7,
        nChannelID: 3,
        nUserData: 9,
        uUserType: 2,
        uVersion: 11,
        uUserState: 0,
        uLocalSubscriptions: Subscriptions::VOICE,
        uPeerSubscriptions: Subscriptions::USER_MSG,
        nStatusMode: 1,
        nVolumeVoice: 10,
        nVolumeMediaFile: 20,
        nStoppedDelayVoice: 1,
        nStoppedDelayMediaFile: 2,
        nBufferMSecVoice: 30,
        nBufferMSecMediaFile: 40,
        nActiveAdaptiveDelayMSec: 50,
        stereoPlaybackVoice: [1, 0],
        stereoPlaybackMediaFile: [0, 1],
        ..Default::default()
    };
    copy_tt("user", &mut raw.szUsername);
    copy_tt("nick", &mut raw.szNickname);
    copy_tt("127.0.0.1", &mut raw.szIPAddress);
    copy_tt("status", &mut raw.szStatusMsg);
    copy_tt("media", &mut raw.szMediaStorageDir);
    copy_tt("client", &mut raw.szClientName);

    let user = User::from(raw);
    assert_eq!(user.id, UserId(7));
    assert_eq!(user.channel_id, ChannelId(3));
    assert_eq!(user.user_data, 9);
    assert_eq!(user.user_type, 2);
    assert_eq!(user.version, 11);
    assert_eq!(user.username, "user");
    assert_eq!(user.nickname, "nick");
    assert_eq!(user.ip_address, "127.0.0.1");
    assert_eq!(user.status_msg, "status");
    assert_eq!(user.media_storage_dir, "media");
    assert_eq!(user.client_name, "client");
    assert_eq!(user.status.presence, UserPresence::Away);
    assert_eq!(user.status.gender, UserGender::Male);
    assert_eq!(user.local_subscriptions.raw(), Subscriptions::VOICE);
    assert_eq!(user.peer_subscriptions.raw(), Subscriptions::USER_MSG);
    assert_eq!(user.volume_voice, 10);
    assert_eq!(user.volume_media, 20);
    assert_eq!(user.stopped_delay_voice, 1);
    assert_eq!(user.stopped_delay_media, 2);
    assert_eq!(user.buf_ms_voice, 30);
    assert_eq!(user.buf_ms_media, 40);
    assert_eq!(user.active_adaptive_delay, 50);
    assert_eq!(user.stereo_playback_voice, [true, false]);
    assert_eq!(user.stereo_playback_media, [false, true]);
}
