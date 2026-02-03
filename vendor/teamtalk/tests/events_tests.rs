use teamtalk::Event;
use teamtalk::client::ffi;

#[test]
fn reconnect_needed_matches() {
    assert!(Event::ConnectionLost.is_reconnect_needed());
    assert!(Event::ConnectFailed.is_reconnect_needed());
    assert!(Event::ConnectCryptError.is_reconnect_needed());
    assert!(!Event::CmdSuccess.is_reconnect_needed());
}

#[test]
fn reconnect_needed_with_extra() {
    let extra = [Event::ServerUpdate, Event::UserJoined];
    assert!(Event::ServerUpdate.is_reconnect_needed_with(&extra));
    assert!(!Event::TextMessage.is_reconnect_needed_with(&extra));
}

#[test]
fn ffi_event_mapping() {
    let event = Event::from(ffi::ClientEvent::CLIENTEVENT_CON_SUCCESS);
    assert!(matches!(event, Event::ConnectSuccess));
}
