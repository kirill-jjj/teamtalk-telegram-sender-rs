use crate::events::Event;
use crate::types::ClientId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub id: ClientId,
    pub label: Option<String>,
    pub state: crate::events::ConnectionState,
    pub last_event: Option<Event>,
    pub last_event_at: Option<SystemTime>,
}

#[derive(Clone, Default)]
pub struct ClientRegistry {
    inner: Arc<Mutex<HashMap<ClientId, ClientInfo>>>,
}

impl ClientRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, client: &crate::client::Client) {
        let info = ClientInfo {
            id: client.id(),
            label: client.label(),
            state: client.connection_state(),
            last_event: None,
            last_event_at: None,
        };
        let mut map = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        map.insert(info.id, info);
    }

    pub fn unregister(&self, id: ClientId) {
        let mut map = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        map.remove(&id);
    }

    pub fn update_event(&self, client: &crate::client::Client, event: Event) {
        let mut map = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let entry = map.entry(client.id()).or_insert(ClientInfo {
            id: client.id(),
            label: client.label(),
            state: client.connection_state(),
            last_event: None,
            last_event_at: None,
        });
        entry.label = client.label();
        entry.state = client.connection_state();
        entry.last_event = Some(event);
        entry.last_event_at = Some(SystemTime::now());
    }

    pub fn update_snapshot(&self, client: &crate::client::Client) {
        let mut map = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let entry = map.entry(client.id()).or_insert(ClientInfo {
            id: client.id(),
            label: client.label(),
            state: client.connection_state(),
            last_event: None,
            last_event_at: None,
        });
        entry.label = client.label();
        entry.state = client.connection_state();
    }

    pub fn list(&self) -> Vec<ClientInfo> {
        let map = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        map.values().cloned().collect()
    }

    pub fn get(&self, id: ClientId) -> Option<ClientInfo> {
        let map = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        map.get(&id).cloned()
    }
}
