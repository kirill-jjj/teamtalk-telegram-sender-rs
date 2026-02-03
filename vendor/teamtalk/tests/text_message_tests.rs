use teamtalk::client::ffi;
use teamtalk::types::{MessageTarget, TextMessage, UserId};
use teamtalk::utils::strings::ToTT;

fn copy_tt(src: &str, dst: &mut [ffi::TTCHAR]) {
    let tt = src.tt();
    let len = tt.len().min(dst.len());
    dst[..len].copy_from_slice(&tt[..len]);
}

#[test]
fn text_message_from_ffi_fields() {
    let mut raw = ffi::TextMessage {
        nMsgType: ffi::TextMsgType::MSGTYPE_USER,
        nFromUserID: 11,
        nToUserID: 22,
        nChannelID: 33,
        bMore: 1,
        ..Default::default()
    };
    copy_tt("alice", &mut raw.szFromUsername);
    copy_tt("hello", &mut raw.szMessage);
    let msg = TextMessage::from(raw);
    assert_eq!(msg.from_id, UserId(11));
    assert_eq!(msg.to_id, UserId(22));
    assert_eq!(msg.channel_id.0, 33);
    assert_eq!(msg.from_username, "alice");
    assert_eq!(msg.text, "hello");
    assert!(msg.more);
    assert!(matches!(
        MessageTarget::from(&msg),
        MessageTarget::User(UserId(11))
    ));
}
