#![cfg(feature = "mock")]

use std::sync::Arc;

use teamtalk::client::Client;
use teamtalk::client::backend::MockBackend;
use teamtalk::client::registry::ClientRegistry;
use teamtalk::events::Event;

#[test]
fn register_update_and_unregister() {
    let backend = Arc::new(MockBackend::new());
    let client = Client::with_backend(backend).expect("client");
    client.set_label(Some("test-client"));

    let registry = ClientRegistry::new();
    registry.register(&client);
    let stored = registry.get(client.id()).expect("registered");
    assert_eq!(stored.label.as_deref(), Some("test-client"));

    registry.update_event(&client, Event::ConnectSuccess);
    let updated = registry.get(client.id()).expect("updated");
    assert_eq!(updated.last_event, Some(Event::ConnectSuccess));
    assert!(updated.last_event_at.is_some());

    registry.unregister(client.id());
    assert!(registry.get(client.id()).is_none());
}
