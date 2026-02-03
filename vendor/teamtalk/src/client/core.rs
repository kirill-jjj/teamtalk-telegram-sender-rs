//! Core client type and message wrapper.
use crate::events::{ConnectionState, Error, Event, Result};
#[cfg(feature = "scripts")]
use crate::extensions::scripts::ScriptManager;
use crate::types::ClientId;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
pub use teamtalk_sys as ffi;

use super::bus;
use super::cache;
use super::hooks;

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug)]
pub struct TTPtr(pub *mut ffi::TTInstance);

unsafe impl Send for TTPtr {}
unsafe impl Sync for TTPtr {}

pub struct Client {
    /// Optional client name used by the SDK.
    pub name: Option<String>,
    pub(crate) ptr: TTPtr,
    pub(crate) id: ClientId,
    pub(crate) backend: Arc<dyn super::backend::TeamTalkBackend>,
    pub(crate) label: Mutex<Option<String>>,
    pub(crate) state: Mutex<ConnectionState>,
    pub(crate) hooks: Mutex<hooks::ClientHooks>,
    pub(crate) bus: Mutex<bus::EventBus>,
    #[cfg(feature = "scripts")]
    pub(crate) scripts: Mutex<Option<ScriptManager>>,
    pub(crate) auto_reconnect: Mutex<AutoReconnectState>,
    pub(crate) cache: Mutex<cache::CacheState>,
}

unsafe impl Send for Client {}
unsafe impl Sync for Client {}

/// A split interface for handling client events (polling).
pub struct ClientEvents(pub Arc<Client>);

impl ClientEvents {
    /// Polls the client for the next event.
    pub fn poll(&self, timeout_ms: i32) -> Option<(Event, Message)> {
        self.0.poll(timeout_ms)
    }
}

/// A split interface for issuing client commands.
#[derive(Clone)]
pub struct ClientCommands(pub Arc<Client>);

impl std::ops::Deref for ClientCommands {
    type Target = Client;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Client {
    /// Creates a new polling client and loads the SDK.
    pub fn new() -> Result<Self> {
        crate::init()?;
        let backend: Arc<dyn super::backend::TeamTalkBackend> =
            Arc::new(super::backend::FfiBackend);
        let ptr = backend.init_poll();
        if ptr.is_null() {
            Err(Error::InitFailed)
        } else {
            Ok(Self {
                name: None,
                ptr: TTPtr(ptr),
                id: ClientId(NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed)),
                backend,
                label: Mutex::new(None),
                state: Mutex::new(ConnectionState::Idle),
                hooks: Mutex::new(hooks::ClientHooks::default()),
                bus: Mutex::new(bus::EventBus::default()),
                #[cfg(feature = "scripts")]
                scripts: Mutex::new(None),
                auto_reconnect: Mutex::new(AutoReconnectState::default()),
                cache: Mutex::new(cache::CacheState::default()),
            })
        }
    }

    /// Splits the client into event polling and command execution parts.
    pub fn split(self) -> (ClientEvents, ClientCommands) {
        let shared = Arc::new(self);
        (ClientEvents(shared.clone()), ClientCommands(shared))
    }

    #[cfg(windows)]
    /// Creates a client bound to a Windows message window.
    ///
    /// # Safety
    /// - `hwnd` must be a valid window handle for the lifetime of the client.
    /// - `msg` must be a valid message ID routed to `hwnd`.
    /// - The caller must ensure the window's message loop stays alive while the
    ///   client is in use.
    pub unsafe fn with_hwnd(hwnd: ffi::HWND, msg: u32) -> Result<Self> {
        crate::init()?;
        let backend: Arc<dyn super::backend::TeamTalkBackend> =
            Arc::new(super::backend::FfiBackend);
        let ptr = backend.init_hwnd(hwnd, msg);
        if ptr.is_null() {
            Err(Error::InitFailed)
        } else {
            Ok(Self {
                name: None,
                ptr: TTPtr(ptr),
                id: ClientId(NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed)),
                backend,
                label: Mutex::new(None),
                state: Mutex::new(ConnectionState::Idle),
                hooks: Mutex::new(hooks::ClientHooks::default()),
                bus: Mutex::new(bus::EventBus::default()),
                #[cfg(feature = "scripts")]
                scripts: Mutex::new(None),
                auto_reconnect: Mutex::new(AutoReconnectState::default()),
                cache: Mutex::new(cache::CacheState::default()),
            })
        }
    }

    #[cfg(windows)]
    /// Swaps the window handle used by the client.
    ///
    /// # Safety
    /// - `hwnd` must be a valid window handle for the lifetime of the client.
    /// - The previous window handle must no longer be in use by this client.
    pub unsafe fn swap_hwnd(&self, hwnd: ffi::HWND) -> bool {
        unsafe { ffi::api().TT_SwapTeamTalkHWND(self.ptr.0, hwnd) == 1 }
    }

    pub(crate) fn backend(&self) -> &dyn super::backend::TeamTalkBackend {
        self.backend.as_ref()
    }

    #[cfg(feature = "mock")]
    pub fn with_backend(backend: Arc<dyn super::backend::TeamTalkBackend>) -> Result<Self> {
        let ptr = backend.init_poll();
        if ptr.is_null() {
            Err(Error::InitFailed)
        } else {
            Ok(Self {
                name: None,
                ptr: TTPtr(ptr),
                id: ClientId(NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed)),
                backend,
                label: Mutex::new(None),
                state: Mutex::new(ConnectionState::Idle),
                hooks: Mutex::new(hooks::ClientHooks::default()),
                bus: Mutex::new(bus::EventBus::default()),
                #[cfg(feature = "scripts")]
                scripts: Mutex::new(None),
                auto_reconnect: Mutex::new(AutoReconnectState::default()),
                cache: Mutex::new(cache::CacheState::default()),
            })
        }
    }

    /// Sets the client name used for login.
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets a human-friendly label for the client instance.
    pub fn with_label(self, label: &str) -> Self {
        *self.label.lock().unwrap() = Some(label.to_string());
        self
    }

    /// Returns the client instance id.
    pub fn id(&self) -> ClientId {
        self.id
    }

    /// Returns the client label, if set.
    pub fn label(&self) -> Option<String> {
        self.label.lock().unwrap().clone()
    }

    /// Sets or clears the client label.
    pub fn set_label(&self, label: Option<&str>) {
        *self.label.lock().unwrap() = label.map(|value| value.to_string());
    }

    /// Returns the current connection state.
    pub fn connection_state(&self) -> ConnectionState {
        *self.state.lock().unwrap()
    }

    /// Creates a subscription for a specific event type.
    pub fn on_event(&self, event: Event) -> bus::SubscriptionBuilder<'_> {
        bus::SubscriptionBuilder::new(self, Some(event))
    }

    /// Creates a subscription for all events.
    pub fn on_any(&self) -> bus::SubscriptionBuilder<'_> {
        bus::SubscriptionBuilder::new(self, None)
    }

    /// Removes an event subscription.
    pub fn unsubscribe_event(&self, id: bus::EventSubscriptionId) -> bool {
        self.bus.lock().unwrap().unsubscribe(id)
    }

    /// Clears all event subscriptions.
    pub fn clear_event_subscriptions(&self) {
        self.bus.lock().unwrap().clear();
    }

    /// Removes all subscriptions in the specified group.
    pub fn unsubscribe_event_group(&self, group: impl AsRef<str>) -> usize {
        let group = bus::EventSubscriptionGroup::new(group.as_ref());
        self.bus.lock().unwrap().unsubscribe_group(&group)
    }

    /// Returns the number of active event subscriptions.
    pub fn event_subscription_count(&self) -> usize {
        self.bus.lock().unwrap().len()
    }

    /// Replaces the current hook set.
    pub fn set_hooks(&self, hooks: hooks::ClientHooks) {
        *self.hooks.lock().unwrap() = hooks;
    }

    /// Clears all hooks.
    pub fn clear_hooks(&self) {
        *self.hooks.lock().unwrap() = hooks::ClientHooks::default();
    }

    #[cfg(feature = "scripts")]
    pub fn enable_scripts(&self) {
        let mut scripts = self.scripts.lock().unwrap();
        if scripts.is_none() {
            *scripts = Some(ScriptManager::new());
        }
    }

    #[cfg(feature = "scripts")]
    pub fn set_script_manager(&self, manager: ScriptManager) {
        *self.scripts.lock().unwrap() = Some(manager);
    }

    #[cfg(feature = "scripts")]
    pub fn clear_scripts(&self) {
        *self.scripts.lock().unwrap() = None;
    }

    #[cfg(feature = "scripts")]
    pub fn scripts_mut<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut ScriptManager) -> R,
    {
        self.scripts.lock().unwrap().as_mut().map(f)
    }

    pub(crate) fn set_connection_state(&self, state: ConnectionState) {
        *self.state.lock().unwrap() = state;
    }

    pub(crate) fn invoke_hooks(&self, event: crate::events::Event, msg: &Message) {
        self.hooks.lock().unwrap().fire(self, event, msg);
    }

    pub(crate) fn dispatch_bus(&self, event: crate::events::Event, msg: &Message) {
        self.bus.lock().unwrap().dispatch(self, event, msg);
    }

    #[cfg(feature = "scripts")]
    pub(crate) fn dispatch_scripts(&self, event: crate::events::Event, msg: &Message) {
        if let Some(manager) = self.scripts.lock().unwrap().as_ref() {
            let _ = manager.handle_event(event, msg);
        }
    }

    pub(crate) fn invoke_joined_hook(&self, channel_id: crate::types::ChannelId) {
        self.hooks.lock().unwrap().fire_joined(self, channel_id);
    }

    pub(crate) fn handle_auto_reconnect(&self) {
        if *self.state.lock().unwrap() != ConnectionState::Disconnected {
            return;
        }

        let mut auto = self.auto_reconnect.lock().unwrap();
        if !auto.enabled {
            return;
        }

        let params: super::connection::ConnectParamsOwned = match auto.params.as_ref() {
            Some(params) => params.clone(),
            None => return,
        };

        let handler: &mut super::connection::ReconnectHandler = match auto.handler.as_mut() {
            Some(handler) => handler,
            None => return,
        };

        if handler.can_attempt() {
            let attempt = handler.attempts() + 1;
            let delay = handler.current_delay();
            let before_event = Event::BeforeReconnect { attempt, delay };
            let msg = Message::from_raw(before_event, unsafe {
                std::mem::zeroed::<ffi::TTMessage>()
            });
            drop(auto);

            self.invoke_hooks(before_event, &msg);

            let mut auto = self.auto_reconnect.lock().unwrap();
            if let Some(handler) = auto.handler.as_mut() {
                handler.record_attempt();
            }
            drop(auto);

            self.invoke_hooks(Event::Reconnecting { attempt, delay }, &msg);
            let _ = self.connect(&params.host, params.tcp, params.udp, params.encrypted);
        } else {
            let attempts = handler.attempts();
            let failed_event = Event::ReconnectFailed { attempts };
            let msg = Message::from_raw(failed_event, unsafe {
                std::mem::zeroed::<ffi::TTMessage>()
            });
            drop(auto);
            self.invoke_hooks(failed_event, &msg);

            let mut auto = self.auto_reconnect.lock().unwrap();
            auto.enabled = false;
            auto.handler = None;
        }
    }

    pub(crate) fn handle_auto_login(&self) {
        if *self.state.lock().unwrap() != ConnectionState::Connected {
            return;
        }

        let auto = self.auto_reconnect.lock().unwrap();
        if !auto.enabled {
            return;
        }

        let params: super::users::LoginParams = match auto.login.as_ref() {
            Some(params) => params.clone(),
            None => return,
        };
        drop(auto);

        let _ = self.login(
            &params.nickname,
            &params.username,
            &params.password,
            &params.client_name,
        );
    }

    pub(crate) fn handle_auto_join(&self) {
        if *self.state.lock().unwrap() != ConnectionState::LoggedIn {
            return;
        }

        let auto = self.auto_reconnect.lock().unwrap();
        if !auto.enabled {
            return;
        }

        let channel = match auto.last_channel {
            Some(channel) => channel,
            None => return,
        };
        drop(auto);

        let _ = self.join_channel(channel, "");
    }

    /// Sends a debug input tone to the SDK.
    pub fn dbg_set_input_tone(&self, stream_types: u32, freq: i32) -> bool {
        unsafe { ffi::api().TT_DBG_SetSoundInputTone(self.ptr.0, stream_types, freq) == 1 }
    }

    /// Writes a debug tone into an audio file.
    pub fn dbg_write_audio_file_tone(&self, file_path: &str, freq: i32) -> bool {
        let mut info = unsafe { std::mem::zeroed::<ffi::MediaFileInfo>() };
        let p = crate::utils::ToTT::tt(file_path);
        unsafe {
            std::ptr::copy_nonoverlapping(
                p.as_ptr(),
                info.szFileName.as_mut_ptr(),
                p.len().min(511),
            );
            ffi::api().TT_DBG_WriteAudioFileTone(&info, freq) == 1
        }
    }

    /// Returns the SDK-reported size for a TeamTalk type.
    pub fn dbg_sizeof(n_type: ffi::TTType) -> i32 {
        unsafe { ffi::api().TT_DBG_SIZEOF(n_type) }
    }

    /// Returns a data pointer for a TeamTalk message.
    pub fn dbg_get_data_ptr(msg: &mut ffi::TTMessage) -> *mut std::ffi::c_void {
        unsafe { ffi::api().TT_DBG_GETDATAPTR(msg) }
    }
}

#[derive(Default)]
pub(crate) struct AutoReconnectState {
    pub(crate) enabled: bool,
    pub(crate) handler: Option<super::connection::ReconnectHandler>,
    pub(crate) params: Option<super::connection::ConnectParamsOwned>,
    pub(crate) last_channel: Option<crate::types::ChannelId>,
    pub(crate) login: Option<super::users::LoginParams>,
}

/// Wrapper around a raw TeamTalk message with its originating event.
pub struct Message {
    event: crate::events::Event,
    raw: ffi::TTMessage,
}

impl Message {
    /// Wraps a raw TeamTalk message.
    pub(crate) fn from_raw(event: crate::events::Event, raw: ffi::TTMessage) -> Self {
        Self { event, raw }
    }

    /// Returns the originating event for this message.
    pub fn event(&self) -> crate::events::Event {
        self.event
    }

    /// Returns the source user id for the message.
    pub fn source(&self) -> i32 {
        self.raw.nSource
    }

    /// Returns the text message payload if present.
    pub fn text(&self) -> Option<crate::types::TextMessage> {
        if matches!(self.event, crate::events::Event::TextMessage) {
            unsafe {
                Some(crate::types::TextMessage::from(
                    self.raw.__bindgen_anon_1.textmessage,
                ))
            }
        } else {
            None
        }
    }

    /// Returns the channel payload if present.
    pub fn channel(&self) -> Option<crate::types::Channel> {
        if matches!(
            self.event,
            crate::events::Event::ChannelCreated
                | crate::events::Event::ChannelUpdated
                | crate::events::Event::ChannelRemoved
        ) {
            unsafe {
                Some(crate::types::Channel::from(
                    self.raw.__bindgen_anon_1.channel,
                ))
            }
        } else {
            None
        }
    }

    /// Returns the server properties payload if present.
    pub fn server_properties(&self) -> Option<crate::types::ServerProperties> {
        if matches!(self.event, crate::events::Event::ServerUpdate) {
            unsafe {
                Some(crate::types::ServerProperties::from(
                    self.raw.__bindgen_anon_1.serverproperties,
                ))
            }
        } else {
            None
        }
    }

    /// Returns the server statistics payload if present.
    pub fn server_statistics(&self) -> Option<crate::types::ServerStatistics> {
        if matches!(self.event, crate::events::Event::ServerStatistics) {
            unsafe {
                Some(crate::types::ServerStatistics::from(
                    self.raw.__bindgen_anon_1.serverstatistics,
                ))
            }
        } else {
            None
        }
    }

    /// Returns the file transfer payload if present.
    pub fn file_transfer(&self) -> Option<crate::types::FileTransfer> {
        if matches!(self.event, crate::events::Event::FileTransfer) {
            unsafe {
                Some(crate::types::FileTransfer::from(
                    self.raw.__bindgen_anon_1.filetransfer,
                ))
            }
        } else {
            None
        }
    }

    /// Returns the user payload if present.
    pub fn user(&self) -> Option<crate::types::User> {
        if matches!(
            self.event,
            crate::events::Event::UserLoggedIn
                | crate::events::Event::UserLoggedOut
                | crate::events::Event::UserUpdate
                | crate::events::Event::UserJoined
                | crate::events::Event::UserLeft
                | crate::events::Event::UserStateChange
                | crate::events::Event::UserFirstVoiceStreamPacket
        ) {
            unsafe { Some(crate::types::User::from(self.raw.__bindgen_anon_1.user)) }
        } else {
            None
        }
    }

    /// Returns the user account payload if present.
    pub fn account(&self) -> Option<crate::types::UserAccount> {
        if matches!(
            self.event,
            crate::events::Event::UserAccount
                | crate::events::Event::UserAccountCreated
                | crate::events::Event::UserAccountRemoved
        ) {
            unsafe {
                Some(crate::types::UserAccount::from(
                    self.raw.__bindgen_anon_1.useraccount,
                ))
            }
        } else {
            None
        }
    }

    /// Returns the raw TeamTalk message.
    pub fn raw(&self) -> &ffi::TTMessage {
        &self.raw
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.backend.close(self.ptr.0);
    }
}

impl Client {
    /// Returns the raw TeamTalk instance pointer.
    pub fn raw_ptr(&self) -> *mut ffi::TTInstance {
        self.ptr.0
    }

    /// Returns the SDK version string.
    pub fn version() -> String {
        let _ = crate::init();
        unsafe {
            let ptr = ffi::api().TT_GetVersion();
            if ptr.is_null() {
                "Unknown".to_string()
            } else {
                crate::utils::strings::from_tt(ptr)
            }
        }
    }

    /// Polls the client for the next event.
    pub fn poll(&self, timeout_ms: i32) -> Option<(Event, Message)> {
        let mut msg = unsafe { std::mem::zeroed::<ffi::TTMessage>() };
        let t = timeout_ms;
        if unsafe { ffi::api().TT_GetMessage(self.ptr.0, &mut msg, &t) } == 1 {
            let event = Event::from(msg.nClientEvent);
            let message = Message::from_raw(event, msg);
            self.update_state_for_event(event, &message);
            self.update_cache_for_event(event, &message);
            self.invoke_hooks(event, &message);
            self.dispatch_bus(event, &message);
            #[cfg(feature = "scripts")]
            self.dispatch_scripts(event, &message);
            self.handle_auto_reconnect();
            Some((event, message))
        } else {
            self.handle_auto_reconnect();
            None
        }
    }

    /// Polls until the predicate matches or the timeout expires.
    pub fn poll_until<F>(&self, timeout_ms: i32, mut predicate: F) -> Option<(Event, Message)>
    where
        F: FnMut(Event, &Message) -> bool,
    {
        use std::time::{Duration, Instant};
        if timeout_ms < 0 {
            loop {
                if let Some((event, msg)) = self.poll(timeout_ms)
                    && predicate(event, &msg)
                {
                    return Some((event, msg));
                }
            }
        }

        let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64);
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return None;
            }
            let wait_ms = remaining.as_millis().min(i32::MAX as u128) as i32;
            if let Some((event, msg)) = self.poll(wait_ms)
                && predicate(event, &msg)
            {
                return Some((event, msg));
            }
        }
    }

    /// Polls until a specific event arrives or the timeout expires.
    pub fn wait_for(&self, event: Event, timeout_ms: i32) -> Option<Message> {
        self.poll_until(timeout_ms, |incoming, _| incoming == event)
            .map(|(_, msg)| msg)
    }

    /// Polls until a specific event arrives or the timeout expires.
    pub fn poll_until_event(&self, event: Event, timeout_ms: i32) -> Option<Message> {
        self.wait_for(event, timeout_ms)
    }

    fn update_state_for_event(&self, event: Event, msg: &Message) {
        match event {
            Event::ConnectSuccess => {
                self.set_connection_state(ConnectionState::Connected);
                self.handle_auto_login();
                if self.auto_reconnect_enabled() {
                    let msg =
                        Message::from_raw(event, unsafe { std::mem::zeroed::<ffi::TTMessage>() });
                    let auto = self.auto_reconnect.lock().unwrap();
                    let attempts = auto.handler.as_ref().map(|h| h.attempts()).unwrap_or(0);
                    drop(auto);
                    if attempts > 0 {
                        self.invoke_hooks(Event::AfterReconnect { attempt: attempts }, &msg);
                    }
                }
            }
            Event::ConnectFailed | Event::ConnectionLost | Event::ConnectCryptError => {
                self.set_connection_state(ConnectionState::Disconnected)
            }
            Event::MySelfLoggedIn => {
                self.set_connection_state(ConnectionState::LoggedIn);
                self.handle_auto_join();
            }
            Event::MySelfLoggedOut => self.set_connection_state(ConnectionState::Connected),
            Event::UserJoined => {
                if let Some(user) = msg.user()
                    && user.id == self.my_id()
                {
                    self.set_connection_state(ConnectionState::Joined(user.channel_id));
                    self.invoke_joined_hook(user.channel_id);
                }
            }
            Event::UserLeft => {
                if let Some(user) = msg.user()
                    && user.id == self.my_id()
                {
                    self.set_connection_state(ConnectionState::LoggedIn);
                }
            }
            _ => {}
        }
    }

    /// Returns the current client flags.
    pub fn get_flags(&self) -> crate::types::ClientFlags {
        crate::types::ClientFlags::from_raw(unsafe { ffi::api().TT_GetFlags(self.ptr.0) })
    }

    /// Returns a human-readable error message for a TeamTalk error code.
    pub fn get_error_message(&self, code: i32) -> String {
        use crate::types::TT_STRLEN;
        use crate::utils::strings::tt_buf;
        let mut buf = tt_buf::<TT_STRLEN>();
        unsafe {
            ffi::api().TT_GetErrorMessage(code, buf.as_mut_ptr());
            crate::utils::strings::to_string(&buf)
        }
    }

    /// Builds a typed SDK error with the resolved message.
    pub fn client_error(&self, code: i32) -> crate::events::Error {
        crate::events::Error::ClientError {
            code,
            message: self.get_error_message(code),
        }
    }
}
