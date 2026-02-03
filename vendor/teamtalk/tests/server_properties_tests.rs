use teamtalk::client::ffi;
use teamtalk::types::ServerProperties;
use teamtalk::utils::strings::ToTT;

fn copy_tt(src: &str, dst: &mut [ffi::TTCHAR]) {
    let tt = src.tt();
    let len = tt.len().min(dst.len());
    dst[..len].copy_from_slice(&tt[..len]);
}

#[test]
fn server_properties_from_ffi() {
    let mut raw = ffi::ServerProperties {
        nMaxUsers: 5,
        nMaxLoginAttempts: 2,
        nMaxLoginsPerIPAddress: 3,
        nMaxVoiceTxPerSecond: 4,
        nMaxVideoCaptureTxPerSecond: 6,
        nMaxMediaFileTxPerSecond: 7,
        nMaxDesktopTxPerSecond: 8,
        nMaxTotalTxPerSecond: 9,
        bAutoSave: 1,
        nTcpPort: 10,
        nUdpPort: 11,
        nUserTimeout: 12,
        nLoginDelayMSec: 13,
        uServerLogEvents: 14,
        ..Default::default()
    };
    copy_tt("srv", &mut raw.szServerName);
    copy_tt("motd", &mut raw.szMOTD);
    copy_tt("raw", &mut raw.szMOTDRaw);
    copy_tt("ver", &mut raw.szServerVersion);
    copy_tt("proto", &mut raw.szServerProtocolVersion);
    copy_tt("token", &mut raw.szAccessToken);
    let props = ServerProperties::from(raw);
    assert_eq!(props.name, "srv");
    assert_eq!(props.motd, "motd");
    assert_eq!(props.motd_raw, "raw");
    assert_eq!(props.version, "ver");
    assert_eq!(props.protocol_version, "proto");
    assert_eq!(props.access_token, "token");
    assert_eq!(props.max_users, 5);
    assert_eq!(props.max_login_attempts, 2);
    assert_eq!(props.max_logins_per_ip, 3);
    assert_eq!(props.max_voice_tx, 4);
    assert_eq!(props.max_video_tx, 6);
    assert_eq!(props.max_media_tx, 7);
    assert_eq!(props.max_desktop_tx, 8);
    assert_eq!(props.max_total_tx, 9);
    assert!(props.auto_save);
    assert_eq!(props.tcp_port, 10);
    assert_eq!(props.udp_port, 11);
    assert_eq!(props.user_timeout, 12);
    assert_eq!(props.login_delay, 13);
    assert_eq!(props.log_events, 14);
}
