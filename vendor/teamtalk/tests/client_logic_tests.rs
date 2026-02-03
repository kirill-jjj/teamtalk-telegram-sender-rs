#![cfg(feature = "mock")]

use std::sync::Arc;

use teamtalk::client::Client;
use teamtalk::client::backend::MockBackend;
use teamtalk::client::ffi;
use teamtalk::events::ConnectionState;
use teamtalk::types::{Channel, ChannelId, UserPresence, UserStatus};
use teamtalk::utils::strings::to_string;

fn test_channel(id: i32, name: &str) -> Channel {
    let mut channel = Channel::builder(name).build();
    channel.id = ChannelId(id);
    channel
}

#[test]
fn login_with_params_sets_state_and_records_login() {
    let backend = Arc::new(MockBackend::new());
    backend.set_login_result(42);
    let client = Client::with_backend(backend.clone()).expect("client");
    client.set_login_params(teamtalk::client::users::LoginParams::new(
        "nick", "user", "pass", "client",
    ));

    let cmd_id = client.login_with_params().expect("login");

    assert_eq!(cmd_id, 42);
    assert_eq!(client.connection_state(), ConnectionState::LoggingIn);
    assert_eq!(
        backend.last_login(),
        Some((
            "nick".to_string(),
            "user".to_string(),
            "pass".to_string(),
            "client".to_string()
        ))
    );
}

#[test]
fn login_with_params_requires_login_params() {
    let backend = Arc::new(MockBackend::new());
    let client = Client::with_backend(backend).expect("client");

    let err = client.login_with_params().expect_err("missing params");

    assert!(matches!(err, teamtalk::events::Error::MissingLoginParams));
}

#[test]
fn login_from_env_uses_env() {
    let backend = Arc::new(MockBackend::new());
    backend.set_login_result(7);
    let client = Client::with_backend(backend.clone()).expect("client");

    let original_nick = std::env::var("TT_NICK").ok();
    let original_user = std::env::var("TT_USER").ok();
    let original_pass = std::env::var("TT_PASS").ok();
    let original_client = std::env::var("TT_CLIENT").ok();

    unsafe {
        std::env::set_var("TT_NICK", "nick-env");
        std::env::set_var("TT_USER", "user-env");
        std::env::set_var("TT_PASS", "pass-env");
        std::env::set_var("TT_CLIENT", "client-env");
    }

    let cmd_id = client.login_from_env();

    assert_eq!(cmd_id, 7);
    assert_eq!(
        backend.last_login(),
        Some((
            "nick-env".to_string(),
            "user-env".to_string(),
            "pass-env".to_string(),
            "client-env".to_string()
        ))
    );

    match original_nick {
        Some(value) => unsafe { std::env::set_var("TT_NICK", value) },
        None => unsafe { std::env::remove_var("TT_NICK") },
    }
    match original_user {
        Some(value) => unsafe { std::env::set_var("TT_USER", value) },
        None => unsafe { std::env::remove_var("TT_USER") },
    }
    match original_pass {
        Some(value) => unsafe { std::env::set_var("TT_PASS", value) },
        None => unsafe { std::env::remove_var("TT_PASS") },
    }
    match original_client {
        Some(value) => unsafe { std::env::set_var("TT_CLIENT", value) },
        None => unsafe { std::env::remove_var("TT_CLIENT") },
    }
}

#[test]
fn join_channel_sets_state_when_successful() {
    let backend = Arc::new(MockBackend::new());
    backend.set_join_result(11);
    backend.set_channel(test_channel(1, "main"));

    let client = Client::with_backend(backend).expect("client");
    let cmd_id = client.join_channel(ChannelId(1), "");

    assert_eq!(cmd_id, 11);
    assert_eq!(
        client.connection_state(),
        ConnectionState::Joining(ChannelId(1))
    );
}

#[test]
fn join_channel_does_not_change_state_on_failure() {
    let backend = Arc::new(MockBackend::new());
    backend.set_join_result(0);
    let client = Client::with_backend(backend).expect("client");

    let cmd_id = client.join_channel(ChannelId(1), "");

    assert_eq!(cmd_id, 0);
    assert_eq!(client.connection_state(), ConnectionState::Idle);
}

#[test]
fn send_text_builds_expected_message() {
    let backend = Arc::new(MockBackend::new());
    let client = Client::with_backend(backend.clone()).expect("client");

    let cmd_id = client.send_to_user(teamtalk::types::UserId(99), "hello");

    assert_eq!(cmd_id, 1);
    let msg = backend.last_text_message().expect("text message");
    assert_eq!(msg.nMsgType, ffi::TextMsgType::MSGTYPE_USER);
    assert_eq!(msg.nToUserID, 99);
    assert_eq!(to_string(&msg.szMessage), "hello");
}

#[test]
fn set_status_message_uses_current_status_when_available() {
    let backend = Arc::new(MockBackend::new());
    backend.set_my_user_id(42);
    let mut user = unsafe { std::mem::zeroed::<ffi::User>() };
    let status = UserStatus {
        presence: UserPresence::Away,
        ..UserStatus::default()
    };
    user.nStatusMode = status.to_bits() as i32;
    backend.set_user(user);

    let client = Client::with_backend(backend.clone()).expect("client");
    let cmd_id = client.set_status_message("ready");

    assert_eq!(cmd_id, 1);
    assert_eq!(
        backend.last_status(),
        Some((status.to_bits() as i32, "ready".to_string()))
    );
}

#[test]
fn set_status_message_uses_default_when_user_missing() {
    let backend = Arc::new(MockBackend::new());
    backend.set_my_user_id(42);
    let client = Client::with_backend(backend.clone()).expect("client");

    let cmd_id = client.set_status_message("fallback");

    assert_eq!(cmd_id, 1);
    assert_eq!(
        backend.last_status(),
        Some((
            UserStatus::default().to_bits() as i32,
            "fallback".to_string()
        ))
    );
}
