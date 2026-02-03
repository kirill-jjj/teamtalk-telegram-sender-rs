#![cfg(feature = "mock")]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use teamtalk::Event;
use teamtalk::client::ffi;
use teamtalk::dispatch::{DispatchFlow, Dispatcher};
use teamtalk::mock::{MockClient, MockMessage, MockUserBuilder};
use teamtalk::types::{ChannelId, UserId};

#[test]
fn dispatcher_dispatches_events() {
    let mut mock = MockClient::new();
    let user = MockUserBuilder::new(UserId(1))
        .username("alice")
        .nickname("a");
    mock.push_user_joined(user);
    let msg = MockMessage::text(
        ffi::TextMsgType::MSGTYPE_USER,
        UserId(1),
        UserId(2),
        ChannelId(3),
        "alice",
        "hi",
    );
    mock.push_text_message(msg);

    let joined = Arc::new(AtomicUsize::new(0));
    let messages = Arc::new(AtomicUsize::new(0));
    let any = Arc::new(AtomicUsize::new(0));

    let joined_c = Arc::clone(&joined);
    let messages_c = Arc::clone(&messages);
    let any_c = Arc::clone(&any);

    let mut dispatcher = Dispatcher::new(mock)
        .on_user_joined(move |ctx| {
            let user = ctx.message().user().unwrap();
            assert_eq!(user.id.0, 1);
            joined_c.fetch_add(1, Ordering::SeqCst);
            DispatchFlow::Continue
        })
        .on_text_message(move |ctx| {
            let text = ctx.message().text().unwrap();
            assert_eq!(text.text, "hi");
            messages_c.fetch_add(1, Ordering::SeqCst);
            DispatchFlow::Continue
        })
        .on_any(move |_| {
            any_c.fetch_add(1, Ordering::SeqCst);
            DispatchFlow::Continue
        });

    dispatcher.step(0);
    dispatcher.step(0);
    dispatcher.step(0);

    assert_eq!(joined.load(Ordering::SeqCst), 1);
    assert_eq!(messages.load(Ordering::SeqCst), 1);
    assert_eq!(any.load(Ordering::SeqCst), 2);
}

#[test]
fn dispatcher_stop_flow() {
    let mut mock = MockClient::new();
    mock.push_event(Event::ConnectFailed);
    let mut dispatcher = Dispatcher::new(mock).on_connect_failed(|_| DispatchFlow::Stop);
    let flow = dispatcher.step(0);
    assert!(matches!(flow, DispatchFlow::Stop));
}
