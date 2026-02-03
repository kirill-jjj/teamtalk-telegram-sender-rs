use teamtalk::types::{
    Channel, ChannelId, ChannelType, MessageTarget, Subscriptions, UserGender, UserId,
    UserPresence, UserState, UserStatus,
};

#[test]
fn user_status_roundtrip() {
    let status = UserStatus {
        presence: UserPresence::Away,
        gender: UserGender::Female,
        video: true,
        desktop: true,
        streaming: false,
        media_paused: true,
    };
    let bits = status.to_bits();
    let parsed = UserStatus::from_bits(bits);
    assert_eq!(parsed.presence, status.presence);
    assert_eq!(parsed.gender, status.gender);
    assert_eq!(parsed.video, status.video);
    assert_eq!(parsed.desktop, status.desktop);
    assert_eq!(parsed.streaming, status.streaming);
    assert_eq!(parsed.media_paused, status.media_paused);
}

#[test]
fn user_state_flags() {
    let state =
        UserState::from_raw(UserState::VOICE | UserState::MUTE_VOICE | UserState::VIDEOCAPTURE);
    assert!(state.is_talking());
    assert!(state.is_muted());
    assert!(state.has_video());
}

#[test]
fn subscriptions_add_remove() {
    let mut subs = Subscriptions::new();
    subs.add(Subscriptions::USER_MSG);
    assert!(subs.has(Subscriptions::USER_MSG));
    subs.remove(Subscriptions::USER_MSG);
    assert!(!subs.has(Subscriptions::USER_MSG));
}

#[test]
fn subscriptions_presets() {
    let audio = Subscriptions::all_audio();
    assert!(audio.has(Subscriptions::VOICE));
    assert!(audio.has(Subscriptions::MEDIAFILE));
    let text = Subscriptions::all_text();
    assert!(text.has(Subscriptions::USER_MSG));
    assert!(text.has(Subscriptions::CHANNEL_MSG));
    assert!(text.has(Subscriptions::BROADCAST_MSG));
    assert!(text.has(Subscriptions::CUSTOM_MSG));
    let control = Subscriptions::all_control();
    assert!(control.has(Subscriptions::DESKTOP));
    assert!(control.has(Subscriptions::DESKTOPINPUT));
}

#[test]
fn channel_builder_sets_fields() {
    let channel = Channel::builder("room")
        .topic("topic")
        .max_users(42)
        .channel_type(ChannelType::from_raw(ChannelType::HIDDEN))
        .build();
    assert_eq!(channel.name, "room");
    assert_eq!(channel.topic, "topic");
    assert_eq!(channel.max_users, 42);
    assert_eq!(channel.channel_type.raw(), ChannelType::HIDDEN);
}

#[test]
fn message_target_from_ids() {
    let user = MessageTarget::from(UserId(1));
    match user {
        MessageTarget::User(id) => assert_eq!(id.0, 1),
        _ => panic!("expected user target"),
    }
    let channel = MessageTarget::from(ChannelId(2));
    match channel {
        MessageTarget::Channel(id) => assert_eq!(id.0, 2),
        _ => panic!("expected channel target"),
    }
}
