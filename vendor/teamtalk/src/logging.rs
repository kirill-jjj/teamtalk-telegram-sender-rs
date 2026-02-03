//! Tracing integration for client events.
use crate::client::Message;
use crate::events::Event;

/// Logs an event and its source id using `tracing::debug!`.
pub fn event(event: &Event, message: &Message) {
    tracing::debug!(?event, source = message.source());
}
