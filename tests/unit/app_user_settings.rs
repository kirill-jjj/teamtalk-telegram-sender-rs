use super::*;

#[test]
fn parse_notification_setting_fallback() {
    assert_eq!(parse_notification_setting(""), NotificationSetting::All);
    assert_eq!(
        parse_notification_setting("unknown"),
        NotificationSetting::All
    );
}

#[test]
fn parse_mute_list_mode_fallback() {
    assert_eq!(parse_mute_list_mode(""), MuteListMode::Blacklist);
    assert_eq!(parse_mute_list_mode("unknown"), MuteListMode::Blacklist);
}
