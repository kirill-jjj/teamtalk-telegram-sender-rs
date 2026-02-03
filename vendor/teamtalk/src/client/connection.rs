//! Connection and reconnect helpers.
use super::Client;
use crate::events::ConnectionState;
use crate::utils::{ToTT, backoff::ExponentialBackoff};
use std::env;
use std::time::{Duration, Instant};
use teamtalk_sys as ffi;

/// Reconnect policy configuration.
#[derive(Clone)]
pub struct ReconnectConfig {
    pub max_attempts: u32,
    pub min_delay: Duration,
    pub max_delay: Duration,
    pub stability_threshold: Duration,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_attempts: u32::MAX,
            min_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(60),
            stability_threshold: Duration::from_secs(10),
        }
    }
}

pub struct ReconnectHandler {
    pub config: ReconnectConfig,
    backoff: ExponentialBackoff,
    attempts: u32,
    last_attempt: Option<Instant>,
    connected_at: Option<Instant>,
}

impl ReconnectHandler {
    /// Creates a new reconnect handler.
    pub fn new(config: ReconnectConfig) -> Self {
        let backoff = ExponentialBackoff::new(config.min_delay, config.max_delay, 1.6, 1.0);
        Self {
            config,
            backoff,
            attempts: 0,
            last_attempt: None,
            connected_at: None,
        }
    }

    /// Marks the client as connected.
    pub fn mark_connected(&mut self) {
        self.connected_at = Some(Instant::now());
    }

    /// Marks the client as disconnected.
    pub fn mark_disconnected(&mut self) {
        if let Some(at) = self.connected_at
            && at.elapsed() >= self.config.stability_threshold
        {
            self.attempts = 0;
            self.backoff.reset();
        }
        self.connected_at = None;
    }

    /// Returns true when a reconnect attempt is allowed.
    pub fn can_attempt(&self) -> bool {
        if self.attempts >= self.config.max_attempts {
            return false;
        }
        match self.last_attempt {
            Some(last) => last.elapsed() >= self.backoff.current_delay(),
            None => true,
        }
    }

    /// Records a reconnect attempt.
    pub fn record_attempt(&mut self) {
        self.last_attempt = Some(Instant::now());
        self.attempts += 1;
        self.backoff.next_delay();
    }

    /// Returns the current backoff delay.
    pub fn current_delay(&self) -> Duration {
        self.backoff.current_delay()
    }

    /// Returns the number of attempts.
    pub fn attempts(&self) -> u32 {
        self.attempts
    }
}

#[derive(Debug, Clone)]
/// Borrowed connection parameters.
pub struct ConnectParams<'a> {
    pub host: &'a str,
    pub tcp: i32,
    pub udp: i32,
    pub encrypted: bool,
}

#[derive(Debug, Clone)]
/// Owned connection parameters.
pub struct ConnectParamsOwned {
    pub host: String,
    pub tcp: i32,
    pub udp: i32,
    pub encrypted: bool,
}

impl ConnectParamsOwned {
    pub fn new(host: impl Into<String>, tcp: i32, udp: i32, encrypted: bool) -> Self {
        Self {
            host: host.into(),
            tcp,
            udp,
            encrypted,
        }
    }

    pub fn from_env() -> Self {
        let host = env::var("TT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let tcp = env::var("TT_TCP")
            .ok()
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(10333);
        let udp = env::var("TT_UDP")
            .ok()
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(10333);
        let encrypted = env::var("TT_ENCRYPTED")
            .ok()
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
            .unwrap_or(false);
        Self::new(host, tcp, udp, encrypted)
    }
}

impl<'a> From<&ConnectParams<'a>> for ConnectParamsOwned {
    fn from(params: &ConnectParams<'a>) -> Self {
        Self::new(params.host, params.tcp, params.udp, params.encrypted)
    }
}

impl Client {
    /// Enables automatic reconnection using the provided config.
    pub fn enable_auto_reconnect(&self, config: ReconnectConfig) {
        let mut auto = self.auto_reconnect.lock().unwrap();
        auto.enabled = true;
        auto.handler = Some(ReconnectHandler::new(config));
    }

    /// Disables automatic reconnection.
    pub fn disable_auto_reconnect(&self) {
        let mut auto = self.auto_reconnect.lock().unwrap();
        auto.enabled = false;
        auto.handler = None;
    }

    /// Returns true if automatic reconnection is enabled.
    pub fn auto_reconnect_enabled(&self) -> bool {
        self.auto_reconnect.lock().unwrap().enabled
    }

    /// Stores connection parameters for automatic reconnection.
    pub fn set_reconnect_params(&self, params: ConnectParamsOwned) {
        self.auto_reconnect.lock().unwrap().params = Some(params);
    }

    /// Returns the stored reconnection parameters, if any.
    pub fn reconnect_params(&self) -> Option<ConnectParamsOwned> {
        self.auto_reconnect.lock().unwrap().params.clone()
    }

    /// Returns the last remembered channel, if any.
    pub fn last_channel(&self) -> Option<crate::types::ChannelId> {
        self.auto_reconnect.lock().unwrap().last_channel
    }

    /// Clears the remembered channel.
    pub fn clear_last_channel(&self) {
        self.auto_reconnect.lock().unwrap().last_channel = None;
    }

    /// Connects and remembers the parameters for automatic reconnection.
    pub fn connect_remember(
        &self,
        host: &str,
        tcp: i32,
        udp: i32,
        encrypted: bool,
    ) -> Result<(), crate::events::Error> {
        self.set_reconnect_params(ConnectParamsOwned::new(host, tcp, udp, encrypted));
        self.connect(host, tcp, udp, encrypted)
    }

    /// Connects using the provided parameters.
    pub fn connect_with_params(
        &self,
        params: &ConnectParamsOwned,
    ) -> Result<(), crate::events::Error> {
        self.connect(&params.host, params.tcp, params.udp, params.encrypted)
    }

    pub fn connect_from_env(&self) -> Result<(), crate::events::Error> {
        let params = ConnectParamsOwned::from_env();
        self.connect_remember(&params.host, params.tcp, params.udp, params.encrypted)
    }

    /// Connects to a TeamTalk server.
    pub fn connect(
        &self,
        host: &str,
        tcp: i32,
        udp: i32,
        encrypted: bool,
    ) -> Result<(), crate::events::Error> {
        let ok = unsafe {
            ffi::api().TT_Connect(
                self.ptr.0,
                host.tt().as_ptr(),
                tcp,
                udp,
                0,
                0,
                if encrypted { 1 } else { 0 },
            ) == 1
        };
        if ok {
            self.set_connection_state(ConnectionState::Connecting);
            Ok(())
        } else {
            Err(crate::events::Error::ConnectFailed)
        }
    }

    /// Connects without encryption.
    pub fn connect_auto(&self, host: &str, tcp: i32, udp: i32) -> Result<(), crate::events::Error> {
        self.connect(host, tcp, udp, false)
    }

    /// Returns true when the client is connected.
    pub fn is_connected(&self) -> bool {
        let flags = unsafe { ffi::api().TT_GetFlags(self.ptr.0) };
        (flags & ffi::ClientFlag::CLIENT_CONNECTED as u32) != 0
    }

    /// Returns true when the client is attempting to connect.
    pub fn is_connecting(&self) -> bool {
        let flags = unsafe { ffi::api().TT_GetFlags(self.ptr.0) };
        (flags & ffi::ClientFlag::CLIENT_CONNECTING as u32) != 0
    }

    /// Handles reconnect logic using provided parameters.
    pub fn handle_reconnect(&self, params: &ConnectParams, handler: &mut ReconnectHandler) -> bool {
        if handler.can_attempt() {
            let _ = self.disconnect();
            handler.record_attempt();
            let _ = self.connect(params.host, params.tcp, params.udp, params.encrypted);
        }

        true
    }

    /// Connects with a custom system id string.
    pub fn connect_sys_id(
        &self,
        host: &str,
        tcp: i32,
        udp: i32,
        encrypted: bool,
        sys_id: &str,
    ) -> Result<(), crate::events::Error> {
        let ok = unsafe {
            ffi::api().TT_ConnectSysID(
                self.ptr.0,
                host.tt().as_ptr(),
                tcp,
                udp,
                0,
                0,
                if encrypted { 1 } else { 0 },
                sys_id.tt().as_ptr(),
            ) == 1
        };
        if ok {
            self.set_connection_state(ConnectionState::Connecting);
            Ok(())
        } else {
            Err(crate::events::Error::ConnectFailed)
        }
    }

    /// Connects with a custom bind IP.
    pub fn connect_ex(
        &self,
        host: &str,
        tcp: i32,
        udp: i32,
        bind_ip: &str,
        encrypted: bool,
    ) -> Result<(), crate::events::Error> {
        let ok = unsafe {
            ffi::api().TT_ConnectEx(
                self.ptr.0,
                host.tt().as_ptr(),
                tcp,
                udp,
                bind_ip.tt().as_ptr(),
                0,
                0,
                if encrypted { 1 } else { 0 },
            ) == 1
        };
        if ok {
            self.set_connection_state(ConnectionState::Connecting);
            Ok(())
        } else {
            Err(crate::events::Error::ConnectFailed)
        }
    }

    /// Disconnects from the server.
    pub fn disconnect(&self) -> Result<(), crate::events::Error> {
        if unsafe { ffi::api().TT_Disconnect(self.ptr.0) == 1 } {
            self.set_connection_state(ConnectionState::Disconnected);
            Ok(())
        } else {
            Err(crate::events::Error::CommandFailed {
                code: -1,
                message: "Disconnect failed".to_string(),
            })
        }
    }

    /// Sets client keep-alive parameters.
    pub fn set_client_keep_alive(
        &self,
        keep_alive: &crate::types::ClientKeepAlive,
    ) -> Result<(), crate::events::Error> {
        if unsafe { ffi::api().TT_SetClientKeepAlive(self.ptr.0, &keep_alive.to_ffi()) == 1 } {
            Ok(())
        } else {
            Err(crate::events::Error::CommandFailed {
                code: -1,
                message: "Set keep-alive failed".to_string(),
            })
        }
    }

    /// Returns client keep-alive parameters.
    pub fn get_client_keep_alive(&self) -> Option<crate::types::ClientKeepAlive> {
        let mut raw = unsafe { std::mem::zeroed::<ffi::ClientKeepAlive>() };
        if unsafe { ffi::api().TT_GetClientKeepAlive(self.ptr.0, &mut raw) } == 1 {
            Some(crate::types::ClientKeepAlive::from(raw))
        } else {
            None
        }
    }
}
