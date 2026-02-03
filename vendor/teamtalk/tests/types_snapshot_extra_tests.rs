use insta::assert_debug_snapshot;
use teamtalk::client::ffi;
use teamtalk::types::{ErrorMessage, ServerProperties, TextMessage, User};
use teamtalk::utils::strings::ToTT;

fn copy_tt(src: &str, dst: &mut [ffi::TTCHAR]) {
    let tt = src.tt();
    let len = tt.len().min(dst.len());
    dst[..len].copy_from_slice(&tt[..len]);
}

#[test]
fn user_snapshot() {
    let mut raw = ffi::User {
        nUserID: 1,
        nChannelID: 2,
        nStatusMode: 2,
        uUserState: 0,
        ..Default::default()
    };
    copy_tt("user", &mut raw.szUsername);
    copy_tt("nick", &mut raw.szNickname);
    let user = User::from(raw);
    assert_debug_snapshot!(user);
}

#[test]
fn server_properties_snapshot() {
    let mut raw = ffi::ServerProperties {
        nMaxUsers: 10,
        bAutoSave: 1,
        ..Default::default()
    };
    copy_tt("srv", &mut raw.szServerName);
    copy_tt("motd", &mut raw.szMOTD);
    copy_tt("raw", &mut raw.szMOTDRaw);
    let props = ServerProperties::from(raw);
    assert_debug_snapshot!(props);
}

#[test]
fn text_message_snapshot() {
    let mut raw = ffi::TextMessage {
        nMsgType: ffi::TextMsgType::MSGTYPE_USER,
        nFromUserID: 10,
        nToUserID: 20,
        nChannelID: 30,
        ..Default::default()
    };
    copy_tt("alice", &mut raw.szFromUsername);
    copy_tt("hello", &mut raw.szMessage);
    let msg = TextMessage::from(raw);
    assert_debug_snapshot!(msg);
}

#[test]
fn error_message_snapshot() {
    let mut raw = ffi::ClientErrorMsg {
        nErrorNo: -1,
        ..Default::default()
    };
    copy_tt("fail", &mut raw.szErrorMsg);
    let err = ErrorMessage::from(raw);
    assert_debug_snapshot!(err);
}
