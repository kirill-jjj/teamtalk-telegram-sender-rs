//! Event dispatcher built on top of `Client::poll`.
use crate::client::{Client, ConnectParams, Message, ReconnectConfig, ReconnectHandler};
use crate::events::Event;
use std::mem;

/// Owned connection parameters for reconnect workflows.
#[derive(Clone)]
pub struct ConnectParamsOwned {
    pub host: String,
    pub tcp: i32,
    pub udp: i32,
    pub encrypted: bool,
}

impl ConnectParamsOwned {
    /// Creates owned connection parameters.
    pub fn new(host: impl Into<String>, tcp: i32, udp: i32, encrypted: bool) -> Self {
        Self {
            host: host.into(),
            tcp,
            udp,
            encrypted,
        }
    }

    /// Returns a borrowed `ConnectParams` view.
    pub fn as_params(&self) -> ConnectParams<'_> {
        ConnectParams {
            host: &self.host,
            tcp: self.tcp,
            udp: self.udp,
            encrypted: self.encrypted,
        }
    }
}

impl From<ConnectParams<'_>> for ConnectParamsOwned {
    fn from(params: ConnectParams<'_>) -> Self {
        Self::new(params.host, params.tcp, params.udp, params.encrypted)
    }
}

#[derive(Clone)]
/// Reconnect settings for dispatch flows.
pub struct ReconnectSettings {
    pub params: ConnectParamsOwned,
    pub config: ReconnectConfig,
    pub extra_events: Vec<Event>,
}

impl ReconnectSettings {
    /// Creates reconnect settings with default event set.
    pub fn new(params: ConnectParamsOwned, config: ReconnectConfig) -> Self {
        Self {
            params,
            config,
            extra_events: Vec::new(),
        }
    }

    /// Adds extra events which should trigger reconnection.
    pub fn with_extra_events(mut self, extra_events: Vec<Event>) -> Self {
        self.extra_events = extra_events;
        self
    }
}

#[derive(Clone)]
/// Configuration for the dispatcher runtime.
pub struct ClientConfig {
    pub poll_timeout_ms: i32,
    pub reconnect: Option<ReconnectSettings>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            poll_timeout_ms: 100,
            reconnect: None,
        }
    }
}

impl ClientConfig {
    /// Creates a configuration with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the poll timeout in milliseconds.
    pub fn poll_timeout_ms(mut self, timeout_ms: i32) -> Self {
        self.poll_timeout_ms = timeout_ms;
        self
    }

    /// Enables reconnect using provided connection parameters.
    pub fn reconnect(mut self, params: ConnectParamsOwned, config: ReconnectConfig) -> Self {
        self.reconnect = Some(ReconnectSettings::new(params, config));
        self
    }

    /// Enables reconnect and adds extra events which trigger reconnect.
    pub fn reconnect_with_events(
        mut self,
        params: ConnectParamsOwned,
        config: ReconnectConfig,
        extra_events: Vec<Event>,
    ) -> Self {
        self.reconnect =
            Some(ReconnectSettings::new(params, config).with_extra_events(extra_events));
        self
    }

    pub fn without_reconnect(mut self) -> Self {
        self.reconnect = None;
        self
    }
}

/// Controls dispatcher loop flow.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DispatchFlow {
    Continue,
    Stop,
}

/// Event context passed into dispatcher handlers.
#[derive(Clone, Copy)]
pub struct EventContext<'a> {
    event: Event,
    message: &'a Message,
    client: Option<&'a Client>,
}

impl<'a> EventContext<'a> {
    /// Returns the event.
    pub fn event(&self) -> Event {
        self.event
    }

    /// Returns the raw message.
    pub fn message(&self) -> &Message {
        self.message
    }

    /// Returns the client if the source provides one.
    pub fn client(&self) -> Option<&Client> {
        self.client
    }
}

/// Event source abstraction for the dispatcher.
pub trait EventSource {
    /// Polls for the next event.
    fn poll(&mut self, timeout_ms: i32) -> Option<(Event, Message)>;
    /// Returns the underlying client if available.
    fn client(&self) -> Option<&Client>;
}

impl EventSource for Client {
    fn poll(&mut self, timeout_ms: i32) -> Option<(Event, Message)> {
        Client::poll(self, timeout_ms)
    }

    fn client(&self) -> Option<&Client> {
        Some(self)
    }
}

impl EventSource for &Client {
    fn poll(&mut self, timeout_ms: i32) -> Option<(Event, Message)> {
        (*self).poll(timeout_ms)
    }

    fn client(&self) -> Option<&Client> {
        Some(*self)
    }
}

type HandlerFn = Box<dyn for<'a> FnMut(EventContext<'a>) -> DispatchFlow + Send>;

struct HandlerEntry {
    event: Option<Event>,
    handler: HandlerFn,
}

impl HandlerEntry {
    fn matches(&self, event: &Event) -> bool {
        match &self.event {
            Some(e) => mem::discriminant(e) == mem::discriminant(event),
            None => true,
        }
    }
}

struct ReconnectState {
    params: ConnectParamsOwned,
    handler: ReconnectHandler,
    extra_events: Vec<Event>,
}

impl ReconnectState {
    fn new(settings: ReconnectSettings) -> Self {
        Self {
            handler: ReconnectHandler::new(settings.config),
            params: settings.params,
            extra_events: settings.extra_events,
        }
    }

    fn on_event(&mut self, client: Option<&Client>, event: &Event) {
        if matches!(event, Event::ConnectSuccess) {
            self.handler.mark_connected();
        }
        if event.is_reconnect_needed_with(&self.extra_events) {
            self.handler.mark_disconnected();
            if let Some(client) = client {
                let params = self.params.as_params();
                client.handle_reconnect(&params, &mut self.handler);
            }
        }
    }
}

pub struct Dispatcher<S: EventSource> {
    source: S,
    handlers: Vec<HandlerEntry>,
    poll_timeout_ms: i32,
    reconnect: Option<ReconnectState>,
    stop: bool,
}

impl<S: EventSource> Dispatcher<S> {
    /// Creates a dispatcher with default configuration.
    pub fn new(source: S) -> Self {
        Self::with_config(source, ClientConfig::default())
    }

    /// Creates a dispatcher with a custom configuration.
    pub fn with_config(source: S, config: ClientConfig) -> Self {
        let reconnect = config.reconnect.map(ReconnectState::new);
        Self {
            source,
            handlers: Vec::new(),
            poll_timeout_ms: config.poll_timeout_ms,
            reconnect,
            stop: false,
        }
    }

    /// Returns the underlying event source.
    pub fn source(&self) -> &S {
        &self.source
    }

    /// Returns a mutable reference to the event source.
    pub fn source_mut(&mut self) -> &mut S {
        &mut self.source
    }

    /// Adds a handler for a specific event.
    pub fn add_handler<F>(&mut self, event: Event, handler: F)
    where
        F: for<'a> FnMut(EventContext<'a>) -> DispatchFlow + Send + 'static,
    {
        self.handlers.push(HandlerEntry {
            event: Some(event),
            handler: Box::new(handler),
        });
    }

    /// Adds a handler which receives all events.
    pub fn add_handler_any<F>(&mut self, handler: F)
    where
        F: for<'a> FnMut(EventContext<'a>) -> DispatchFlow + Send + 'static,
    {
        self.handlers.push(HandlerEntry {
            event: None,
            handler: Box::new(handler),
        });
    }

    /// Adds a handler and returns the dispatcher for chaining.
    pub fn on_event<F>(mut self, event: Event, handler: F) -> Self
    where
        F: for<'a> FnMut(EventContext<'a>) -> DispatchFlow + Send + 'static,
    {
        self.add_handler(event, handler);
        self
    }

    /// Adds a handler for all events and returns the dispatcher for chaining.
    pub fn on_any<F>(mut self, handler: F) -> Self
    where
        F: for<'a> FnMut(EventContext<'a>) -> DispatchFlow + Send + 'static,
    {
        self.add_handler_any(handler);
        self
    }

    /// Adds a handler for user join events.
    pub fn on_user_joined<F>(self, handler: F) -> Self
    where
        F: for<'a> FnMut(EventContext<'a>) -> DispatchFlow + Send + 'static,
    {
        self.on_event(Event::UserJoined, handler)
    }

    /// Adds a handler for user left events.
    pub fn on_user_left<F>(self, handler: F) -> Self
    where
        F: for<'a> FnMut(EventContext<'a>) -> DispatchFlow + Send + 'static,
    {
        self.on_event(Event::UserLeft, handler)
    }

    /// Adds a handler for text messages.
    pub fn on_text_message<F>(self, handler: F) -> Self
    where
        F: for<'a> FnMut(EventContext<'a>) -> DispatchFlow + Send + 'static,
    {
        self.on_event(Event::TextMessage, handler)
    }

    /// Adds a handler for connection success events.
    pub fn on_connect_success<F>(self, handler: F) -> Self
    where
        F: for<'a> FnMut(EventContext<'a>) -> DispatchFlow + Send + 'static,
    {
        self.on_event(Event::ConnectSuccess, handler)
    }

    /// Adds a handler for connection lost events.
    pub fn on_connection_lost<F>(self, handler: F) -> Self
    where
        F: for<'a> FnMut(EventContext<'a>) -> DispatchFlow + Send + 'static,
    {
        self.on_event(Event::ConnectionLost, handler)
    }

    /// Adds a handler for connection failure events.
    pub fn on_connect_failed<F>(self, handler: F) -> Self
    where
        F: for<'a> FnMut(EventContext<'a>) -> DispatchFlow + Send + 'static,
    {
        self.on_event(Event::ConnectFailed, handler)
    }

    /// Adds a handler for command error events.
    pub fn on_command_error<F>(self, handler: F) -> Self
    where
        F: for<'a> FnMut(EventContext<'a>) -> DispatchFlow + Send + 'static,
    {
        self.on_event(Event::CmdError, handler)
    }

    /// Requests the dispatcher loop to stop.
    pub fn stop(&mut self) {
        self.stop = true;
    }

    /// Runs the dispatcher loop with the configured timeout.
    pub fn run(&mut self) -> DispatchFlow {
        self.run_with_timeout(self.poll_timeout_ms)
    }

    /// Runs the dispatcher loop with an explicit timeout.
    pub fn run_with_timeout(&mut self, timeout_ms: i32) -> DispatchFlow {
        while !self.stop {
            let flow = self.step(timeout_ms);
            if flow == DispatchFlow::Stop {
                self.stop = true;
            }
        }
        DispatchFlow::Stop
    }

    /// Performs one poll/dispatch step.
    pub fn step(&mut self, timeout_ms: i32) -> DispatchFlow {
        match self.source.poll(timeout_ms) {
            Some((event, message)) => self.process_event(event, message),
            None => DispatchFlow::Continue,
        }
    }

    fn process_event(&mut self, event: Event, message: Message) -> DispatchFlow {
        let client = self.source.client();
        if let Some(reconnect) = self.reconnect.as_mut() {
            reconnect.on_event(client, &event);
        }
        #[cfg(feature = "logging")]
        crate::logging::event(&event, &message);
        let ctx = EventContext {
            event,
            message: &message,
            client,
        };
        let mut flow = DispatchFlow::Continue;
        for handler in self.handlers.iter_mut() {
            if handler.matches(&event) && (handler.handler)(ctx) == DispatchFlow::Stop {
                flow = DispatchFlow::Stop;
            }
        }
        flow
    }
}
