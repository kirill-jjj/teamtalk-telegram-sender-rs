#![cfg(feature = "mock")]

use teamtalk::client::ffi;
use teamtalk::dispatch::EventSource;
use teamtalk::events::Event;
use teamtalk::mock::{MockClient, MockMessage, MockUserBuilder};
use teamtalk::types::{ChannelId, UserId, UserPresence, UserStatus};

#[test]
fn mock_message_text_roundtrip() {
    let msg = MockMessage::text(
        ffi::TextMsgType::MSGTYPE_USER,
        UserId(10),
        UserId(20),
        ChannelId(30),
        "alice",
        "hello",
    );
    let text = msg.text().unwrap();
    assert_eq!(text.from_id.0, 10);
    assert_eq!(text.to_id.0, 20);
    assert_eq!(text.channel_id.0, 30);
    assert_eq!(text.from_username, "alice");
    assert_eq!(text.text, "hello");
}

#[test]
fn mock_user_builder_fields() {
    let status = UserStatus {
        presence: UserPresence::Away,
        ..UserStatus::default()
    };
    let msg = MockUserBuilder::new(UserId(7))
        .username("bob")
        .nickname("b")
        .client_name("client")
        .ip_address("127.0.0.1")
        .channel_id(ChannelId(2))
        .status(status)
        .user_data(42)
        .user_type(2)
        .version(3)
        .build_for(Event::UserUpdate);
    let user = msg.user().unwrap();
    assert_eq!(user.id.0, 7);
    assert_eq!(user.username, "bob");
    assert_eq!(user.nickname, "b");
    assert_eq!(user.client_name, "client");
    assert_eq!(user.ip_address, "127.0.0.1");
    assert_eq!(user.channel_id.0, 2);
    assert_eq!(user.status.presence, UserPresence::Away);
    assert_eq!(user.user_data, 42);
    assert_eq!(user.user_type, 2);
    assert_eq!(user.version, 3);
}

#[test]
fn mock_client_queue_order() {
    let mut client = MockClient::new();
    client
        .push_event(Event::ConnectSuccess)
        .push_event(Event::ConnectFailed);
    assert_eq!(client.len(), 2);
    let first = client.poll(0).expect("first event");
    assert_eq!(first.0, Event::ConnectSuccess);
    let second = client.poll(0).expect("second event");
    assert_eq!(second.0, Event::ConnectFailed);
    assert!(client.is_empty());
}

#[test]
fn mock_message_text_fields_roundtrip_channel() {
    let message = MockMessage::text(
        teamtalk_sys::TextMsgType::MSGTYPE_CHANNEL,
        UserId(1),
        UserId(2),
        ChannelId(10),
        "alice",
        "hello",
    );
    let text = message.text().expect("text message");
    assert_eq!(text.from_id.0, 1);
    assert_eq!(text.to_id.0, 2);
    assert_eq!(text.channel_id.0, 10);
    assert_eq!(text.from_username, "alice");
    assert_eq!(text.text, "hello");
}

#[test]
fn mock_message_user_builder_fields_joined() {
    let user = MockUserBuilder::new(UserId(7))
        .username("bob")
        .nickname("bobby")
        .client_name("client")
        .ip_address("127.0.0.1")
        .channel_id(ChannelId(3));
    let msg = user.build_for(Event::UserJoined);
    let user = msg.user().expect("user");
    assert_eq!(user.id.0, 7);
    assert_eq!(user.username, "bob");
    assert_eq!(user.nickname, "bobby");
    assert_eq!(user.client_name, "client");
    assert_eq!(user.ip_address, "127.0.0.1");
    assert_eq!(user.channel_id.0, 3);
}
