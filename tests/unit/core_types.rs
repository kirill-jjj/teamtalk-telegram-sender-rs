use super::*;

#[test]
fn language_code_parse_and_display() {
    assert_eq!(LanguageCode::try_from("en").unwrap(), LanguageCode::En);
    assert_eq!(LanguageCode::try_from("RU").unwrap(), LanguageCode::Ru);
    assert!(LanguageCode::try_from("xx").is_err());
    assert_eq!(
        LanguageCode::from_str_or_default("xx", LanguageCode::Ru),
        LanguageCode::Ru
    );
    assert_eq!(LanguageCode::En.to_string(), "en");
}

#[test]
fn notification_setting_roundtrip() {
    for (raw, expected) in [
        ("all", NotificationSetting::All),
        ("join_off", NotificationSetting::JoinOff),
        ("leave_off", NotificationSetting::LeaveOff),
        ("none", NotificationSetting::None),
    ] {
        assert_eq!(NotificationSetting::try_from(raw).unwrap(), expected);
        assert_eq!(expected.to_string(), raw);
    }
    assert!(NotificationSetting::try_from("nope").is_err());
}

#[test]
fn mute_list_mode_roundtrip() {
    assert_eq!(
        MuteListMode::try_from("blacklist").unwrap(),
        MuteListMode::Blacklist
    );
    assert_eq!(
        MuteListMode::try_from("whitelist").unwrap(),
        MuteListMode::Whitelist
    );
    assert!(MuteListMode::try_from("invalid").is_err());
    assert_eq!(MuteListMode::Blacklist.to_string(), "blacklist");
    assert_eq!(MuteListMode::Whitelist.to_string(), "whitelist");
}

#[test]
fn deeplink_action_roundtrip() {
    assert_eq!(
        DeeplinkAction::try_from("subscribe").unwrap(),
        DeeplinkAction::Subscribe
    );
    assert_eq!(
        DeeplinkAction::try_from("UNSUBSCRIBE").unwrap(),
        DeeplinkAction::Unsubscribe
    );
    assert!(DeeplinkAction::try_from("other").is_err());
    assert_eq!(DeeplinkAction::Subscribe.to_string(), "subscribe");
    assert_eq!(DeeplinkAction::Unsubscribe.to_string(), "unsubscribe");
}

#[test]
fn tt_username_helpers() {
    let name = TtUsername::new("tester");
    assert_eq!(name.as_str(), "tester");
    assert_eq!(name.to_string(), "tester");
    let from_str: TtUsername = "user".into();
    assert_eq!(from_str.as_ref(), "user");
    let from_string: TtUsername = String::from("bob").into();
    assert_eq!(from_string.as_ref(), "bob");
}
