#[cfg(feature = "scripts")]
use crate::client::Message;
#[cfg(feature = "scripts")]
use crate::events::Event;
#[cfg(feature = "scripts")]
use crate::types::{
    Channel, FileTransfer, ServerProperties, ServerStatistics, TextMessage, User, UserAccount,
};
#[cfg(feature = "scripts")]
use mlua::{HookTriggers, Lua, Table, Value, VmState};
#[cfg(feature = "scripts")]
use std::collections::HashMap;
#[cfg(feature = "scripts")]
use std::fs;
#[cfg(feature = "scripts")]
use std::path::{Path, PathBuf};
#[cfg(feature = "scripts")]
use std::time::{Duration, Instant};

#[cfg(feature = "scripts")]
pub struct ScriptManager {
    lua: Lua,
    scripts: HashMap<String, ScriptEntry>,
    max_exec_time: Option<Duration>,
    hook_instruction_count: u32,
    init_error: Option<mlua::Error>,
}

#[cfg(feature = "scripts")]
struct ScriptEntry {
    path: PathBuf,
    globals: Vec<String>,
}

#[cfg(feature = "scripts")]
impl ScriptManager {
    pub fn new() -> Self {
        let mut manager = Self {
            lua: Lua::new(),
            scripts: HashMap::new(),
            max_exec_time: None,
            hook_instruction_count: 50_000,
            init_error: None,
        };
        if let Err(err) = manager.register_builtin_api() {
            manager.init_error = Some(err);
        }
        manager
    }

    pub fn set_timeout(&mut self, max_exec_time: Duration) {
        self.max_exec_time = Some(max_exec_time);
    }

    pub fn clear_timeout(&mut self) {
        self.max_exec_time = None;
    }

    pub fn set_hook_instruction_count(&mut self, count: u32) {
        self.hook_instruction_count = count.max(1);
    }

    pub fn load_script(&mut self, name: &str, path: impl AsRef<Path>) -> mlua::Result<()> {
        self.ensure_ready()?;
        let path = path.as_ref().to_path_buf();
        let contents = fs::read_to_string(&path)?;
        let globals_before = self.collect_global_keys()?;
        let result = self.with_script_name(name, || {
            self.with_timeout(|| self.lua.load(&contents).exec())
        });
        if let Err(err) = result {
            return Err(self.wrap_error(name, "load_script", err));
        }
        let globals_after = self.collect_global_keys()?;
        let globals = diff_globals(&globals_before, &globals_after);
        self.scripts
            .insert(name.to_string(), ScriptEntry { path, globals });
        Ok(())
    }

    pub fn reload_script(&mut self, name: &str) -> mlua::Result<()> {
        self.ensure_ready()?;
        let path = self
            .scripts
            .get(name)
            .ok_or_else(|| mlua::Error::RuntimeError("script not found".into()))?
            .path
            .clone();
        self.unload_script(name)?;
        self.load_script(name, path)
    }

    pub fn unload_script(&mut self, name: &str) -> mlua::Result<()> {
        self.ensure_ready()?;
        let entry = self
            .scripts
            .remove(name)
            .ok_or_else(|| mlua::Error::RuntimeError("script not found".into()))?;
        let globals = self.lua.globals();
        for key in entry.globals {
            let _ = globals.raw_remove(key);
        }
        self.remove_registered_commands(name)?;
        Ok(())
    }

    pub fn call_command(&self, command: &str, args: &[String]) -> mlua::Result<bool> {
        self.ensure_ready()?;
        let globals = self.lua.globals();
        let handlers: Value = globals.get("commands")?;
        let handlers = match handlers {
            Value::Table(table) => table,
            _ => return Ok(false),
        };
        let func: Value = handlers.get(command)?;
        let func = match func {
            Value::Function(func) => func,
            _ => return Ok(false),
        };
        let args_table = self.lua.create_table()?;
        for (idx, arg) in args.iter().enumerate() {
            args_table.set(idx + 1, arg.clone())?;
        }
        let result = self
            .with_timeout(|| func.call::<bool>(args_table))
            .map_err(|err| self.wrap_error(command, "call_command", err))?;
        Ok(result)
    }

    pub fn register_fn<A, R, F>(&mut self, name: &str, func: F) -> mlua::Result<()>
    where
        F: for<'lua> Fn(&'lua Lua, A) -> mlua::Result<R> + Send + 'static,
        A: mlua::FromLuaMulti,
        R: mlua::IntoLuaMulti,
    {
        self.ensure_ready()?;
        let f = self.lua.create_function(func)?;
        let globals = self.lua.globals();
        globals.set(name, f)?;
        Ok(())
    }

    pub fn handle_event(&self, event: Event, message: &Message) -> mlua::Result<bool> {
        self.ensure_ready()?;
        let globals = self.lua.globals();
        let mut handled = false;
        if let Ok(Value::Function(func)) = globals.get::<Value>("on_event") {
            let event_table = self.event_table(event, message)?;
            let result = self
                .with_timeout(|| func.call::<bool>(event_table))
                .map_err(|err| self.wrap_error(event_name(event), "on_event", err))?;
            handled |= result;
        }
        if let Ok(Value::Table(table)) = globals.get::<Value>("events") {
            let key = event_name(event);
            if let Ok(Value::Function(func)) = table.get::<Value>(key) {
                let event_table = self.event_table(event, message)?;
                let result = self
                    .with_timeout(|| func.call::<bool>(event_table))
                    .map_err(|err| self.wrap_error(key, "event", err))?;
                handled |= result;
            }
        }
        Ok(handled)
    }

    fn with_timeout<F, R>(&self, func: F) -> mlua::Result<R>
    where
        F: FnOnce() -> mlua::Result<R>,
    {
        let max_exec_time = match self.max_exec_time {
            Some(max_exec_time) => max_exec_time,
            None => return func(),
        };
        let start = Instant::now();
        let triggers = HookTriggers::new().every_nth_instruction(self.hook_instruction_count);
        self.lua.set_hook(triggers, move |_lua, _debug| {
            if start.elapsed() > max_exec_time {
                Err(mlua::Error::RuntimeError("script timeout".into()))
            } else {
                Ok(VmState::Continue)
            }
        })?;
        let result = func();
        let _ = self
            .lua
            .set_hook(HookTriggers::new(), |_lua, _debug| Ok(VmState::Continue));
        result
    }

    fn with_script_name<F, R>(&self, name: &str, func: F) -> mlua::Result<R>
    where
        F: FnOnce() -> mlua::Result<R>,
    {
        let globals = self.lua.globals();
        globals.set("_SCRIPT_NAME", name)?;
        let result = func();
        let _ = globals.raw_remove("_SCRIPT_NAME");
        result
    }

    fn collect_global_keys(&self) -> mlua::Result<Vec<String>> {
        self.ensure_ready()?;
        let globals = self.lua.globals();
        let mut keys = Vec::new();
        for pair in globals.pairs::<Value, Value>() {
            let (key, _) = pair?;
            if let Value::String(value) = key {
                keys.push(value.to_str()?.to_string());
            }
        }
        Ok(keys)
    }

    fn register_builtin_api(&mut self) -> mlua::Result<()> {
        let func = self
            .lua
            .create_function(|lua, (name, func): (String, mlua::Function)| {
                let globals = lua.globals();
                let commands = match globals.get::<Value>("commands")? {
                    Value::Table(table) => table,
                    _ => {
                        let table = lua.create_table()?;
                        globals.set("commands", table.clone())?;
                        table
                    }
                };
                commands.set(name.clone(), func)?;
                if let Ok(Value::String(script)) = globals.get::<Value>("_SCRIPT_NAME") {
                    let by_script = match globals.get::<Value>("__tt_commands_by_script")? {
                        Value::Table(table) => table,
                        _ => {
                            let table = lua.create_table()?;
                            globals.set("__tt_commands_by_script", table.clone())?;
                            table
                        }
                    };
                    let key = script.to_str()?.to_string();
                    let list = match by_script.get::<Value>(key.clone())? {
                        Value::Table(table) => table,
                        _ => {
                            let table = lua.create_table()?;
                            by_script.set(key, table.clone())?;
                            table
                        }
                    };
                    let idx = list.len()? + 1;
                    list.set(idx, name)?;
                }
                Ok(())
            })?;
        let globals = self.lua.globals();
        globals.set("register_command", func)?;
        Ok(())
    }

    fn remove_registered_commands(&self, name: &str) -> mlua::Result<()> {
        self.ensure_ready()?;
        let globals = self.lua.globals();
        let by_script = match globals.get::<Value>("__tt_commands_by_script")? {
            Value::Table(table) => table,
            _ => return Ok(()),
        };
        let list = match by_script.get::<Value>(name) {
            Ok(Value::Table(table)) => table,
            _ => return Ok(()),
        };
        let commands = match globals.get::<Value>("commands")? {
            Value::Table(table) => table,
            _ => return Ok(()),
        };
        for pair in list.sequence_values::<String>() {
            let cmd = pair?;
            let _ = commands.raw_remove(cmd);
        }
        let _ = by_script.raw_remove(name);
        Ok(())
    }

    fn wrap_error(&self, name: &str, context: &str, err: mlua::Error) -> mlua::Error {
        mlua::Error::RuntimeError(format!("lua {context} error ({name}): {err}"))
    }

    fn ensure_ready(&self) -> mlua::Result<()> {
        if let Some(err) = &self.init_error {
            return Err(mlua::Error::RuntimeError(format!("lua init error: {err}")));
        }
        Ok(())
    }

    fn event_table(&self, event: Event, message: &Message) -> mlua::Result<Table> {
        let table = self.lua.create_table()?;
        table.set("type", event_name(event))?;
        table.set("source", message.source())?;
        match event {
            Event::TextMessage => {
                if let Some(text) = message.text() {
                    table.set("text", self.text_table(&text)?)?;
                }
            }
            Event::UserLoggedIn
            | Event::UserLoggedOut
            | Event::UserUpdate
            | Event::UserJoined
            | Event::UserLeft
            | Event::UserStateChange
            | Event::UserFirstVoiceStreamPacket => {
                if let Some(user) = message.user() {
                    table.set("user", self.user_table(&user)?)?;
                }
            }
            Event::ChannelCreated | Event::ChannelUpdated | Event::ChannelRemoved => {
                if let Some(channel) = message.channel() {
                    table.set("channel", self.channel_table(&channel)?)?;
                }
            }
            Event::ServerUpdate => {
                if let Some(props) = message.server_properties() {
                    table.set("server_properties", self.server_properties_table(&props)?)?;
                }
            }
            Event::ServerStatistics => {
                if let Some(stats) = message.server_statistics() {
                    table.set("server_statistics", self.server_statistics_table(&stats)?)?;
                }
            }
            Event::FileTransfer => {
                if let Some(file_transfer) = message.file_transfer() {
                    table.set("file_transfer", self.file_transfer_table(&file_transfer)?)?;
                }
            }
            Event::UserAccount | Event::UserAccountCreated | Event::UserAccountRemoved => {
                if let Some(account) = message.account() {
                    table.set("account", self.account_table(&account)?)?;
                }
            }
            Event::None
            | Event::ConnectSuccess
            | Event::ConnectCryptError
            | Event::ConnectFailed
            | Event::ConnectionLost
            | Event::ConnectMaxPayloadUpdated
            | Event::CmdProcessing
            | Event::CmdError
            | Event::CmdSuccess
            | Event::MySelfLoggedIn
            | Event::MySelfLoggedOut
            | Event::MySelfKicked
            | Event::FileNew
            | Event::FileRemove
            | Event::BannedUser
            | Event::VideoCaptureFrame
            | Event::MediaFileVideo
            | Event::DesktopWindow
            | Event::DesktopCursor
            | Event::DesktopInput
            | Event::UserRecordMediaFile
            | Event::AudioBlock
            | Event::InternalError
            | Event::VoiceActivation
            | Event::Hotkey
            | Event::HotkeyTest
            | Event::DesktopWindowTransfer
            | Event::StreamMediaFile
            | Event::LocalMediaFile
            | Event::AudioInput
            | Event::SoundDeviceAdded
            | Event::SoundDeviceRemoved
            | Event::SoundDeviceUnplugged
            | Event::SoundDeviceNewDefaultInput
            | Event::SoundDeviceNewDefaultOutput
            | Event::SoundDeviceNewDefaultInputComDevice
            | Event::SoundDeviceNewDefaultOutputComDevice
            | Event::BeforeReconnect { .. }
            | Event::Reconnecting { .. }
            | Event::AfterReconnect { .. }
            | Event::ReconnectFailed { .. }
            | Event::Unknown(_) => {}
        }
        Ok(table)
    }

    fn text_table(&self, msg: &TextMessage) -> mlua::Result<Table> {
        let table = self.lua.create_table()?;
        table.set("msg_type", msg.msg_type as i32)?;
        table.set("from_id", msg.from_id.0)?;
        table.set("from_username", msg.from_username.clone())?;
        table.set("to_id", msg.to_id.0)?;
        table.set("channel_id", msg.channel_id.0)?;
        table.set("text", msg.text.clone())?;
        table.set("more", msg.more)?;
        Ok(table)
    }

    fn user_table(&self, user: &User) -> mlua::Result<Table> {
        let table = self.lua.create_table()?;
        table.set("id", user.id.0)?;
        table.set("username", user.username.clone())?;
        table.set("nickname", user.nickname.clone())?;
        table.set("user_data", user.user_data)?;
        table.set("user_type", user.user_type as i64)?;
        table.set("ip_address", user.ip_address.clone())?;
        table.set("version", user.version as i64)?;
        table.set("channel_id", user.channel_id.0)?;
        table.set("status_mode", user.status.to_bits() as i64)?;
        table.set("status_msg", user.status_msg.clone())?;
        table.set("state", user.state.raw() as i64)?;
        table.set("local_subscriptions", user.local_subscriptions.raw() as i64)?;
        table.set("peer_subscriptions", user.peer_subscriptions.raw() as i64)?;
        table.set("media_storage_dir", user.media_storage_dir.clone())?;
        table.set("volume_voice", user.volume_voice)?;
        table.set("volume_media", user.volume_media)?;
        table.set("stopped_delay_voice", user.stopped_delay_voice)?;
        table.set("stopped_delay_media", user.stopped_delay_media)?;
        table.set("sound_position_voice", user.sound_position_voice.to_vec())?;
        table.set("sound_position_media", user.sound_position_media.to_vec())?;
        table.set(
            "stereo_playback_voice",
            vec![user.stereo_playback_voice[0], user.stereo_playback_voice[1]],
        )?;
        table.set(
            "stereo_playback_media",
            vec![user.stereo_playback_media[0], user.stereo_playback_media[1]],
        )?;
        table.set("buf_ms_voice", user.buf_ms_voice)?;
        table.set("buf_ms_media", user.buf_ms_media)?;
        table.set("active_adaptive_delay", user.active_adaptive_delay)?;
        table.set("client_name", user.client_name.clone())?;
        Ok(table)
    }

    fn channel_table(&self, channel: &Channel) -> mlua::Result<Table> {
        let table = self.lua.create_table()?;
        table.set("id", channel.id.0)?;
        table.set("parent_id", channel.parent_id.0)?;
        table.set("name", channel.name.clone())?;
        table.set("topic", channel.topic.clone())?;
        table.set("channel_type", channel.channel_type.raw() as i64)?;
        table.set("has_password", channel.has_password)?;
        table.set("user_data", channel.user_data)?;
        table.set("disk_quota", channel.disk_quota)?;
        table.set("max_users", channel.max_users)?;
        table.set("queue_delay_ms", channel.queue_delay_ms)?;
        table.set("timeout_voice_ms", channel.timeout_voice_ms)?;
        table.set("timeout_media_ms", channel.timeout_media_ms)?;
        let tx = self.lua.create_table()?;
        for (idx, (user_id, stream_type)) in channel.transmit_users.iter().enumerate() {
            let entry = self.lua.create_table()?;
            entry.set("user_id", user_id.0)?;
            entry.set("stream_type", *stream_type as i64)?;
            tx.set(idx + 1, entry)?;
        }
        table.set("transmit_users", tx)?;
        table.set(
            "transmit_users_queue",
            channel
                .transmit_users_queue
                .iter()
                .map(|id| id.0)
                .collect::<Vec<_>>(),
        )?;
        Ok(table)
    }

    fn file_transfer_table(&self, transfer: &FileTransfer) -> mlua::Result<Table> {
        let table = self.lua.create_table()?;
        table.set("id", transfer.id.0)?;
        table.set("channel_id", transfer.channel_id.0)?;
        table.set("local_path", transfer.local_path.clone())?;
        table.set("remote_name", transfer.remote_name.clone())?;
        table.set("size", transfer.size)?;
        table.set("transferred", transfer.transferred)?;
        table.set("status", transfer.status as i64)?;
        table.set("inbound", transfer.inbound)?;
        Ok(table)
    }

    fn server_properties_table(&self, props: &ServerProperties) -> mlua::Result<Table> {
        let table = self.lua.create_table()?;
        table.set("name", props.name.clone())?;
        table.set("motd", props.motd.clone())?;
        table.set("motd_raw", props.motd_raw.clone())?;
        table.set("max_users", props.max_users)?;
        table.set("max_login_attempts", props.max_login_attempts)?;
        table.set("max_logins_per_ip", props.max_logins_per_ip)?;
        table.set("max_voice_tx", props.max_voice_tx)?;
        table.set("max_video_tx", props.max_video_tx)?;
        table.set("max_media_tx", props.max_media_tx)?;
        table.set("max_desktop_tx", props.max_desktop_tx)?;
        table.set("max_total_tx", props.max_total_tx)?;
        table.set("user_timeout", props.user_timeout)?;
        table.set("auto_save", props.auto_save)?;
        table.set("tcp_port", props.tcp_port)?;
        table.set("udp_port", props.udp_port)?;
        table.set("version", props.version.clone())?;
        table.set("protocol_version", props.protocol_version.clone())?;
        table.set("login_delay", props.login_delay)?;
        table.set("access_token", props.access_token.clone())?;
        table.set("log_events", props.log_events as i64)?;
        Ok(table)
    }

    fn server_statistics_table(&self, stats: &ServerStatistics) -> mlua::Result<Table> {
        let table = self.lua.create_table()?;
        table.set("total_tx", stats.total_tx)?;
        table.set("total_rx", stats.total_rx)?;
        table.set("voice_tx", stats.voice_tx)?;
        table.set("voice_rx", stats.voice_rx)?;
        table.set("video_tx", stats.video_tx)?;
        table.set("video_rx", stats.video_rx)?;
        table.set("media_tx", stats.media_tx)?;
        table.set("media_rx", stats.media_rx)?;
        table.set("desktop_tx", stats.desktop_tx)?;
        table.set("desktop_rx", stats.desktop_rx)?;
        table.set("users_served", stats.users_served)?;
        table.set("users_peak", stats.users_peak)?;
        table.set("files_tx", stats.files_tx)?;
        table.set("files_rx", stats.files_rx)?;
        table.set("uptime_ms", stats.uptime_ms)?;
        Ok(table)
    }

    fn account_table(&self, account: &UserAccount) -> mlua::Result<Table> {
        let table = self.lua.create_table()?;
        table.set("username", account.username.clone())?;
        table.set("password", account.password.clone())?;
        table.set("user_type", account.user_type as i64)?;
        table.set("user_rights", account.user_rights as i64)?;
        table.set("note", account.note.clone())?;
        table.set("init_channel", account.init_channel.clone())?;
        table.set("user_data", account.user_data)?;
        table.set(
            "auto_operator_channels",
            account
                .auto_operator_channels
                .iter()
                .map(|id| id.0)
                .collect::<Vec<_>>(),
        )?;
        table.set("audio_codec_bps_limit", account.audio_codec_bps_limit)?;
        Ok(table)
    }
}

#[cfg(feature = "scripts")]
impl Default for ScriptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "scripts")]
fn event_name(event: Event) -> &'static str {
    match event {
        Event::None => "None",
        Event::ConnectSuccess => "ConnectSuccess",
        Event::ConnectCryptError => "ConnectCryptError",
        Event::ConnectFailed => "ConnectFailed",
        Event::ConnectionLost => "ConnectionLost",
        Event::ConnectMaxPayloadUpdated => "ConnectMaxPayloadUpdated",
        Event::CmdProcessing => "CmdProcessing",
        Event::CmdError => "CmdError",
        Event::CmdSuccess => "CmdSuccess",
        Event::MySelfLoggedIn => "MySelfLoggedIn",
        Event::MySelfLoggedOut => "MySelfLoggedOut",
        Event::MySelfKicked => "MySelfKicked",
        Event::UserLoggedIn => "UserLoggedIn",
        Event::UserLoggedOut => "UserLoggedOut",
        Event::UserUpdate => "UserUpdate",
        Event::UserJoined => "UserJoined",
        Event::UserLeft => "UserLeft",
        Event::TextMessage => "TextMessage",
        Event::ChannelCreated => "ChannelCreated",
        Event::ChannelUpdated => "ChannelUpdated",
        Event::ChannelRemoved => "ChannelRemoved",
        Event::ServerUpdate => "ServerUpdate",
        Event::ServerStatistics => "ServerStatistics",
        Event::FileNew => "FileNew",
        Event::FileRemove => "FileRemove",
        Event::UserAccount => "UserAccount",
        Event::BannedUser => "BannedUser",
        Event::UserAccountCreated => "UserAccountCreated",
        Event::UserAccountRemoved => "UserAccountRemoved",
        Event::UserStateChange => "UserStateChange",
        Event::VideoCaptureFrame => "VideoCaptureFrame",
        Event::MediaFileVideo => "MediaFileVideo",
        Event::DesktopWindow => "DesktopWindow",
        Event::DesktopCursor => "DesktopCursor",
        Event::DesktopInput => "DesktopInput",
        Event::UserRecordMediaFile => "UserRecordMediaFile",
        Event::AudioBlock => "AudioBlock",
        Event::InternalError => "InternalError",
        Event::VoiceActivation => "VoiceActivation",
        Event::Hotkey => "Hotkey",
        Event::HotkeyTest => "HotkeyTest",
        Event::FileTransfer => "FileTransfer",
        Event::DesktopWindowTransfer => "DesktopWindowTransfer",
        Event::StreamMediaFile => "StreamMediaFile",
        Event::LocalMediaFile => "LocalMediaFile",
        Event::AudioInput => "AudioInput",
        Event::UserFirstVoiceStreamPacket => "UserFirstVoiceStreamPacket",
        Event::SoundDeviceAdded => "SoundDeviceAdded",
        Event::SoundDeviceRemoved => "SoundDeviceRemoved",
        Event::SoundDeviceUnplugged => "SoundDeviceUnplugged",
        Event::SoundDeviceNewDefaultInput => "SoundDeviceNewDefaultInput",
        Event::SoundDeviceNewDefaultOutput => "SoundDeviceNewDefaultOutput",
        Event::SoundDeviceNewDefaultInputComDevice => "SoundDeviceNewDefaultInputComDevice",
        Event::SoundDeviceNewDefaultOutputComDevice => "SoundDeviceNewDefaultOutputComDevice",
        Event::BeforeReconnect { .. } => "BeforeReconnect",
        Event::Reconnecting { .. } => "Reconnecting",
        Event::AfterReconnect { .. } => "AfterReconnect",
        Event::ReconnectFailed { .. } => "ReconnectFailed",
        Event::Unknown(_) => "Unknown",
    }
}

#[cfg(feature = "scripts")]
fn diff_globals(before: &[String], after: &[String]) -> Vec<String> {
    let mut added = Vec::new();
    let excluded = [
        "_SCRIPT_NAME",
        "register_command",
        "commands",
        "__tt_commands_by_script",
    ];
    for key in after {
        if excluded.iter().any(|value| *value == key) {
            continue;
        }
        if !before.iter().any(|existing| existing == key) {
            added.push(key.clone());
        }
    }
    added
}
