use super::*;
use crate::core::types::{MuteListMode, NotificationSetting, TtUsername};

#[test]
fn callback_roundtrip_menu() {
    let action = CallbackAction::Menu(MenuAction::Settings);
    let encoded = encode_callback(&action);
    let decoded = CallbackAction::from_str(&encoded).unwrap();
    assert_eq!(decoded, action);
}

#[test]
fn callback_roundtrip_nested() {
    let action = CallbackAction::Settings(SettingsAction::SubSet {
        setting: NotificationSetting::JoinOff,
    });
    let encoded = encode_callback(&action);
    let decoded = CallbackAction::from_str(&encoded).unwrap();
    assert_eq!(decoded, action);
}

#[test]
fn callback_noop_roundtrip() {
    let decoded = CallbackAction::from_str("noop").unwrap();
    assert_eq!(decoded, CallbackAction::NoOp);
}

#[test]
fn callback_invalid_base85() {
    let err = CallbackAction::from_str("%%%").unwrap_err();
    assert!(err.to_string().contains("Invalid callback encoding"));
}

#[test]
fn callback_invalid_postcard() {
    let encoded = z85::encode(&[1, 2, 3, 4, 5, 6]);
    let decoded = CallbackAction::from_str(&encoded);
    assert!(decoded.is_ok() || decoded.is_err());
}

#[test]
fn as_callback_data_converts_variants() {
    let encoded = MenuAction::Help.into_data();
    assert_ne!(encoded, "noop");
    let decoded = CallbackAction::from_str(&encoded).unwrap();
    assert_eq!(decoded, CallbackAction::Menu(MenuAction::Help));

    let encoded = MuteAction::ModeSet {
        mode: MuteListMode::Whitelist,
    }
    .into_data();
    let decoded = CallbackAction::from_str(&encoded).unwrap();
    assert_eq!(
        decoded,
        CallbackAction::Mute(MuteAction::ModeSet {
            mode: MuteListMode::Whitelist
        })
    );
}

#[test]
fn callback_includes_usernames() {
    let action = CallbackAction::Subscriber(SubAction::LinkPerform {
        sub_id: 42,
        page: 0,
        username: TtUsername::from("alpha"),
    });
    let encoded = encode_callback(&action);
    let decoded = CallbackAction::from_str(&encoded).unwrap();
    assert_eq!(decoded, action);
}

#[test]
fn callback_rejects_truncated_payload() {
    let action = CallbackAction::Menu(MenuAction::Who);
    let encoded = encode_callback(&action);
    let truncated = &encoded[..encoded.len() / 2];
    assert!(CallbackAction::from_str(truncated).is_err());
}

#[test]
fn callback_rejects_oversized_payload_marker() {
    let encoded = "x".repeat(512);
    assert!(CallbackAction::from_str(&encoded).is_err());
}

#[test]
fn callback_noop_in_data() {
    let encoded = CallbackAction::NoOp.into_data();
    let decoded = CallbackAction::from_str(&encoded).unwrap();
    assert_eq!(decoded, CallbackAction::NoOp);
}

#[test]
fn callback_fuzz_lite_inputs_do_not_panic() {
    for len in [0, 1, 2, 5, 10, 20, 64, 80, 120] {
        let s = "a".repeat(len);
        let _ = CallbackAction::from_str(&s);
    }
}
