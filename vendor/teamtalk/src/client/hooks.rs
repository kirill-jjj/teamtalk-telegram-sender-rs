use super::{Client, Message};
use crate::events::Event;
use crate::types::{ChannelId, TextMessage, User};

type EventHook = Box<dyn FnMut(&Client, Event, &Message) + Send>;
type ClientHook = Box<dyn FnMut(&Client) + Send>;
type ChannelHook = Box<dyn FnMut(&Client, ChannelId) + Send>;
type UserHook = Box<dyn FnMut(&Client, User) + Send>;
type TextHook = Box<dyn FnMut(&Client, TextMessage) + Send>;
type MessageHook = Box<dyn FnMut(&Client, &Message) + Send>;

/// Event hooks for reacting to client activity.
#[derive(Default)]
pub struct ClientHooks {
    on_event: Option<EventHook>,
    on_connect_success: Option<ClientHook>,
    on_connect_failed: Option<ClientHook>,
    on_connect_crypt_error: Option<ClientHook>,
    on_connect_max_payload_updated: Option<MessageHook>,
    on_connection_lost: Option<ClientHook>,
    on_cmd_processing: Option<MessageHook>,
    on_cmd_error: Option<MessageHook>,
    on_cmd_success: Option<MessageHook>,
    on_logged_in: Option<ClientHook>,
    on_logged_out: Option<ClientHook>,
    on_myself_kicked: Option<MessageHook>,
    on_user_logged_in: Option<UserHook>,
    on_user_logged_out: Option<UserHook>,
    on_user_update: Option<UserHook>,
    on_joined: Option<ChannelHook>,
    on_user_joined: Option<UserHook>,
    on_user_left: Option<UserHook>,
    on_text_message: Option<TextHook>,
    on_channel_created: Option<MessageHook>,
    on_channel_updated: Option<MessageHook>,
    on_channel_removed: Option<MessageHook>,
    on_server_update: Option<MessageHook>,
    on_server_statistics: Option<MessageHook>,
    on_file_new: Option<MessageHook>,
    on_file_remove: Option<MessageHook>,
    on_user_account: Option<MessageHook>,
    on_banned_user: Option<MessageHook>,
    on_user_account_created: Option<MessageHook>,
    on_user_account_removed: Option<MessageHook>,
    on_user_state_change: Option<MessageHook>,
    on_video_capture_frame: Option<MessageHook>,
    on_media_file_video: Option<MessageHook>,
    on_desktop_window: Option<MessageHook>,
    on_desktop_cursor: Option<MessageHook>,
    on_desktop_input: Option<MessageHook>,
    on_user_record_media_file: Option<MessageHook>,
    on_audio_block: Option<MessageHook>,
    on_internal_error: Option<MessageHook>,
    on_voice_activation: Option<MessageHook>,
    on_hotkey: Option<MessageHook>,
    on_hotkey_test: Option<MessageHook>,
    on_file_transfer: Option<MessageHook>,
    on_desktop_window_transfer: Option<MessageHook>,
    on_stream_media_file: Option<MessageHook>,
    on_local_media_file: Option<MessageHook>,
    on_audio_input: Option<MessageHook>,
    on_user_first_voice_stream_packet: Option<MessageHook>,
    on_sound_device_added: Option<MessageHook>,
    on_sound_device_removed: Option<MessageHook>,
    on_sound_device_unplugged: Option<MessageHook>,
    on_sound_device_new_default_input: Option<MessageHook>,
    on_sound_device_new_default_output: Option<MessageHook>,
    on_sound_device_new_default_input_com_device: Option<MessageHook>,
    on_sound_device_new_default_output_com_device: Option<MessageHook>,
    on_before_reconnect: Option<MessageHook>,
    on_reconnecting: Option<MessageHook>,
    on_after_reconnect: Option<MessageHook>,
    on_reconnect_failed: Option<MessageHook>,
}

impl ClientHooks {
    /// Registers a handler for every event.
    pub fn on_event(mut self, hook: impl FnMut(&Client, Event, &Message) + Send + 'static) -> Self {
        self.on_event = Some(Box::new(hook));
        self
    }

    /// Registers a handler for successful connections.
    pub fn on_connect_success(mut self, hook: impl FnMut(&Client) + Send + 'static) -> Self {
        self.on_connect_success = Some(Box::new(hook));
        self
    }

    /// Registers a handler for failed connections.
    pub fn on_connect_failed(mut self, hook: impl FnMut(&Client) + Send + 'static) -> Self {
        self.on_connect_failed = Some(Box::new(hook));
        self
    }

    /// Registers a handler for connection encryption errors.
    pub fn on_connect_crypt_error(mut self, hook: impl FnMut(&Client) + Send + 'static) -> Self {
        self.on_connect_crypt_error = Some(Box::new(hook));
        self
    }

    /// Registers a handler for max payload updates.
    pub fn on_connect_max_payload_updated(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_connect_max_payload_updated = Some(Box::new(hook));
        self
    }

    /// Registers a handler for connection loss.
    pub fn on_connection_lost(mut self, hook: impl FnMut(&Client) + Send + 'static) -> Self {
        self.on_connection_lost = Some(Box::new(hook));
        self
    }

    /// Registers a handler for command processing notifications.
    pub fn on_cmd_processing(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_cmd_processing = Some(Box::new(hook));
        self
    }

    /// Registers a handler for command errors.
    pub fn on_cmd_error(mut self, hook: impl FnMut(&Client, &Message) + Send + 'static) -> Self {
        self.on_cmd_error = Some(Box::new(hook));
        self
    }

    /// Registers a handler for command success notifications.
    pub fn on_cmd_success(mut self, hook: impl FnMut(&Client, &Message) + Send + 'static) -> Self {
        self.on_cmd_success = Some(Box::new(hook));
        self
    }

    /// Registers a handler for successful login.
    pub fn on_logged_in(mut self, hook: impl FnMut(&Client) + Send + 'static) -> Self {
        self.on_logged_in = Some(Box::new(hook));
        self
    }

    /// Registers a handler for logout.
    pub fn on_logged_out(mut self, hook: impl FnMut(&Client) + Send + 'static) -> Self {
        self.on_logged_out = Some(Box::new(hook));
        self
    }

    /// Registers a handler for being kicked.
    pub fn on_myself_kicked(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_myself_kicked = Some(Box::new(hook));
        self
    }

    /// Registers a handler for user login events.
    pub fn on_user_logged_in(mut self, hook: impl FnMut(&Client, User) + Send + 'static) -> Self {
        self.on_user_logged_in = Some(Box::new(hook));
        self
    }

    /// Registers a handler for user logout events.
    pub fn on_user_logged_out(mut self, hook: impl FnMut(&Client, User) + Send + 'static) -> Self {
        self.on_user_logged_out = Some(Box::new(hook));
        self
    }

    /// Registers a handler for user updates.
    pub fn on_user_update(mut self, hook: impl FnMut(&Client, User) + Send + 'static) -> Self {
        self.on_user_update = Some(Box::new(hook));
        self
    }

    /// Registers a handler for channel joins.
    pub fn on_joined(mut self, hook: impl FnMut(&Client, ChannelId) + Send + 'static) -> Self {
        self.on_joined = Some(Box::new(hook));
        self
    }

    /// Registers a handler for any user join event.
    pub fn on_user_joined(mut self, hook: impl FnMut(&Client, User) + Send + 'static) -> Self {
        self.on_user_joined = Some(Box::new(hook));
        self
    }

    /// Registers a handler for any user leave event.
    pub fn on_user_left(mut self, hook: impl FnMut(&Client, User) + Send + 'static) -> Self {
        self.on_user_left = Some(Box::new(hook));
        self
    }

    /// Registers a handler for channel or user text messages.
    pub fn on_text_message(
        mut self,
        hook: impl FnMut(&Client, TextMessage) + Send + 'static,
    ) -> Self {
        self.on_text_message = Some(Box::new(hook));
        self
    }

    /// Registers a handler for channel creation events.
    pub fn on_channel_created(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_channel_created = Some(Box::new(hook));
        self
    }

    /// Registers a handler for channel update events.
    pub fn on_channel_updated(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_channel_updated = Some(Box::new(hook));
        self
    }

    /// Registers a handler for channel removal events.
    pub fn on_channel_removed(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_channel_removed = Some(Box::new(hook));
        self
    }

    /// Registers a handler for server updates.
    pub fn on_server_update(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_server_update = Some(Box::new(hook));
        self
    }

    /// Registers a handler for server statistics updates.
    pub fn on_server_statistics(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_server_statistics = Some(Box::new(hook));
        self
    }

    /// Registers a handler for new file events.
    pub fn on_file_new(mut self, hook: impl FnMut(&Client, &Message) + Send + 'static) -> Self {
        self.on_file_new = Some(Box::new(hook));
        self
    }

    /// Registers a handler for file removal events.
    pub fn on_file_remove(mut self, hook: impl FnMut(&Client, &Message) + Send + 'static) -> Self {
        self.on_file_remove = Some(Box::new(hook));
        self
    }

    /// Registers a handler for user account events.
    pub fn on_user_account(mut self, hook: impl FnMut(&Client, &Message) + Send + 'static) -> Self {
        self.on_user_account = Some(Box::new(hook));
        self
    }

    /// Registers a handler for banned user events.
    pub fn on_banned_user(mut self, hook: impl FnMut(&Client, &Message) + Send + 'static) -> Self {
        self.on_banned_user = Some(Box::new(hook));
        self
    }

    /// Registers a handler for user account creation events.
    pub fn on_user_account_created(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_user_account_created = Some(Box::new(hook));
        self
    }

    /// Registers a handler for user account removal events.
    pub fn on_user_account_removed(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_user_account_removed = Some(Box::new(hook));
        self
    }

    /// Registers a handler for user state changes.
    pub fn on_user_state_change(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_user_state_change = Some(Box::new(hook));
        self
    }

    /// Registers a handler for video capture frames.
    pub fn on_video_capture_frame(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_video_capture_frame = Some(Box::new(hook));
        self
    }

    /// Registers a handler for media file video frames.
    pub fn on_media_file_video(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_media_file_video = Some(Box::new(hook));
        self
    }

    /// Registers a handler for desktop window updates.
    pub fn on_desktop_window(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_desktop_window = Some(Box::new(hook));
        self
    }

    /// Registers a handler for desktop cursor updates.
    pub fn on_desktop_cursor(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_desktop_cursor = Some(Box::new(hook));
        self
    }

    /// Registers a handler for desktop input updates.
    pub fn on_desktop_input(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_desktop_input = Some(Box::new(hook));
        self
    }

    /// Registers a handler for recorded media file events.
    pub fn on_user_record_media_file(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_user_record_media_file = Some(Box::new(hook));
        self
    }

    /// Registers a handler for audio block events.
    pub fn on_audio_block(mut self, hook: impl FnMut(&Client, &Message) + Send + 'static) -> Self {
        self.on_audio_block = Some(Box::new(hook));
        self
    }

    /// Registers a handler for internal error events.
    pub fn on_internal_error(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_internal_error = Some(Box::new(hook));
        self
    }

    /// Registers a handler for voice activation events.
    pub fn on_voice_activation(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_voice_activation = Some(Box::new(hook));
        self
    }

    /// Registers a handler for hotkey events.
    pub fn on_hotkey(mut self, hook: impl FnMut(&Client, &Message) + Send + 'static) -> Self {
        self.on_hotkey = Some(Box::new(hook));
        self
    }

    /// Registers a handler for hotkey test events.
    pub fn on_hotkey_test(mut self, hook: impl FnMut(&Client, &Message) + Send + 'static) -> Self {
        self.on_hotkey_test = Some(Box::new(hook));
        self
    }

    /// Registers a handler for file transfer events.
    pub fn on_file_transfer(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_file_transfer = Some(Box::new(hook));
        self
    }

    /// Registers a handler for desktop window transfer events.
    pub fn on_desktop_window_transfer(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_desktop_window_transfer = Some(Box::new(hook));
        self
    }

    /// Registers a handler for stream media file events.
    pub fn on_stream_media_file(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_stream_media_file = Some(Box::new(hook));
        self
    }

    /// Registers a handler for local media file events.
    pub fn on_local_media_file(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_local_media_file = Some(Box::new(hook));
        self
    }

    /// Registers a handler for audio input events.
    pub fn on_audio_input(mut self, hook: impl FnMut(&Client, &Message) + Send + 'static) -> Self {
        self.on_audio_input = Some(Box::new(hook));
        self
    }

    /// Registers a handler for first voice stream packet events.
    pub fn on_user_first_voice_stream_packet(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_user_first_voice_stream_packet = Some(Box::new(hook));
        self
    }

    /// Registers a handler for sound device added events.
    pub fn on_sound_device_added(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_sound_device_added = Some(Box::new(hook));
        self
    }

    /// Registers a handler for sound device removed events.
    pub fn on_sound_device_removed(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_sound_device_removed = Some(Box::new(hook));
        self
    }

    /// Registers a handler for sound device unplugged events.
    pub fn on_sound_device_unplugged(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_sound_device_unplugged = Some(Box::new(hook));
        self
    }

    /// Registers a handler for default sound input device changes.
    pub fn on_sound_device_new_default_input(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_sound_device_new_default_input = Some(Box::new(hook));
        self
    }

    /// Registers a handler for default sound output device changes.
    pub fn on_sound_device_new_default_output(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_sound_device_new_default_output = Some(Box::new(hook));
        self
    }

    /// Registers a handler for default sound input communications device changes.
    pub fn on_sound_device_new_default_input_com_device(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_sound_device_new_default_input_com_device = Some(Box::new(hook));
        self
    }

    /// Registers a handler for default sound output communications device changes.
    pub fn on_sound_device_new_default_output_com_device(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_sound_device_new_default_output_com_device = Some(Box::new(hook));
        self
    }

    /// Registers a handler for reconnecting notifications.
    pub fn on_reconnecting(mut self, hook: impl FnMut(&Client, &Message) + Send + 'static) -> Self {
        self.on_reconnecting = Some(Box::new(hook));
        self
    }

    /// Registers a handler before an automatic reconnect attempt.
    pub fn on_before_reconnect(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_before_reconnect = Some(Box::new(hook));
        self
    }

    /// Registers a handler after an automatic reconnect succeeds.
    pub fn on_after_reconnect(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_after_reconnect = Some(Box::new(hook));
        self
    }

    /// Registers a handler when automatic reconnect gives up.
    pub fn on_reconnect_failed(
        mut self,
        hook: impl FnMut(&Client, &Message) + Send + 'static,
    ) -> Self {
        self.on_reconnect_failed = Some(Box::new(hook));
        self
    }

    pub(crate) fn fire(&mut self, client: &Client, event: Event, msg: &Message) {
        match event {
            Event::ConnectSuccess => {
                if let Some(hook) = self.on_connect_success.as_mut() {
                    hook(client);
                }
            }
            Event::ConnectFailed => {
                if let Some(hook) = self.on_connect_failed.as_mut() {
                    hook(client);
                }
            }
            Event::ConnectCryptError => {
                if let Some(hook) = self.on_connect_crypt_error.as_mut() {
                    hook(client);
                }
            }
            Event::ConnectMaxPayloadUpdated => {
                if let Some(hook) = self.on_connect_max_payload_updated.as_mut() {
                    hook(client, msg);
                }
            }
            Event::ConnectionLost => {
                if let Some(hook) = self.on_connection_lost.as_mut() {
                    hook(client);
                }
            }
            Event::CmdProcessing => {
                if let Some(hook) = self.on_cmd_processing.as_mut() {
                    hook(client, msg);
                }
            }
            Event::CmdError => {
                if let Some(hook) = self.on_cmd_error.as_mut() {
                    hook(client, msg);
                }
            }
            Event::CmdSuccess => {
                if let Some(hook) = self.on_cmd_success.as_mut() {
                    hook(client, msg);
                }
            }
            Event::MySelfLoggedIn => {
                if let Some(hook) = self.on_logged_in.as_mut() {
                    hook(client);
                }
            }
            Event::MySelfLoggedOut => {
                if let Some(hook) = self.on_logged_out.as_mut() {
                    hook(client);
                }
            }
            Event::MySelfKicked => {
                if let Some(hook) = self.on_myself_kicked.as_mut() {
                    hook(client, msg);
                }
            }
            Event::UserLoggedIn => {
                if let Some(user) = msg.user()
                    && let Some(hook) = self.on_user_logged_in.as_mut()
                {
                    hook(client, user);
                }
            }
            Event::UserLoggedOut => {
                if let Some(user) = msg.user()
                    && let Some(hook) = self.on_user_logged_out.as_mut()
                {
                    hook(client, user);
                }
            }
            Event::UserUpdate => {
                if let Some(user) = msg.user()
                    && let Some(hook) = self.on_user_update.as_mut()
                {
                    hook(client, user);
                }
            }
            Event::UserJoined => {
                if let Some(user) = msg.user()
                    && let Some(hook) = self.on_user_joined.as_mut()
                {
                    hook(client, user);
                }
            }
            Event::UserLeft => {
                if let Some(user) = msg.user()
                    && let Some(hook) = self.on_user_left.as_mut()
                {
                    hook(client, user);
                }
            }
            Event::TextMessage => {
                if let Some(text) = msg.text()
                    && let Some(hook) = self.on_text_message.as_mut()
                {
                    hook(client, text);
                }
            }
            Event::ChannelCreated => {
                if let Some(hook) = self.on_channel_created.as_mut() {
                    hook(client, msg);
                }
            }
            Event::ChannelUpdated => {
                if let Some(hook) = self.on_channel_updated.as_mut() {
                    hook(client, msg);
                }
            }
            Event::ChannelRemoved => {
                if let Some(hook) = self.on_channel_removed.as_mut() {
                    hook(client, msg);
                }
            }
            Event::ServerUpdate => {
                if let Some(hook) = self.on_server_update.as_mut() {
                    hook(client, msg);
                }
            }
            Event::ServerStatistics => {
                if let Some(hook) = self.on_server_statistics.as_mut() {
                    hook(client, msg);
                }
            }
            Event::FileNew => {
                if let Some(hook) = self.on_file_new.as_mut() {
                    hook(client, msg);
                }
            }
            Event::FileRemove => {
                if let Some(hook) = self.on_file_remove.as_mut() {
                    hook(client, msg);
                }
            }
            Event::UserAccount => {
                if let Some(hook) = self.on_user_account.as_mut() {
                    hook(client, msg);
                }
            }
            Event::BannedUser => {
                if let Some(hook) = self.on_banned_user.as_mut() {
                    hook(client, msg);
                }
            }
            Event::UserAccountCreated => {
                if let Some(hook) = self.on_user_account_created.as_mut() {
                    hook(client, msg);
                }
            }
            Event::UserAccountRemoved => {
                if let Some(hook) = self.on_user_account_removed.as_mut() {
                    hook(client, msg);
                }
            }
            Event::UserStateChange => {
                if let Some(hook) = self.on_user_state_change.as_mut() {
                    hook(client, msg);
                }
            }
            Event::VideoCaptureFrame => {
                if let Some(hook) = self.on_video_capture_frame.as_mut() {
                    hook(client, msg);
                }
            }
            Event::MediaFileVideo => {
                if let Some(hook) = self.on_media_file_video.as_mut() {
                    hook(client, msg);
                }
            }
            Event::DesktopWindow => {
                if let Some(hook) = self.on_desktop_window.as_mut() {
                    hook(client, msg);
                }
            }
            Event::DesktopCursor => {
                if let Some(hook) = self.on_desktop_cursor.as_mut() {
                    hook(client, msg);
                }
            }
            Event::DesktopInput => {
                if let Some(hook) = self.on_desktop_input.as_mut() {
                    hook(client, msg);
                }
            }
            Event::UserRecordMediaFile => {
                if let Some(hook) = self.on_user_record_media_file.as_mut() {
                    hook(client, msg);
                }
            }
            Event::AudioBlock => {
                if let Some(hook) = self.on_audio_block.as_mut() {
                    hook(client, msg);
                }
            }
            Event::InternalError => {
                if let Some(hook) = self.on_internal_error.as_mut() {
                    hook(client, msg);
                }
            }
            Event::VoiceActivation => {
                if let Some(hook) = self.on_voice_activation.as_mut() {
                    hook(client, msg);
                }
            }
            Event::Hotkey => {
                if let Some(hook) = self.on_hotkey.as_mut() {
                    hook(client, msg);
                }
            }
            Event::HotkeyTest => {
                if let Some(hook) = self.on_hotkey_test.as_mut() {
                    hook(client, msg);
                }
            }
            Event::FileTransfer => {
                if let Some(hook) = self.on_file_transfer.as_mut() {
                    hook(client, msg);
                }
            }
            Event::DesktopWindowTransfer => {
                if let Some(hook) = self.on_desktop_window_transfer.as_mut() {
                    hook(client, msg);
                }
            }
            Event::StreamMediaFile => {
                if let Some(hook) = self.on_stream_media_file.as_mut() {
                    hook(client, msg);
                }
            }
            Event::LocalMediaFile => {
                if let Some(hook) = self.on_local_media_file.as_mut() {
                    hook(client, msg);
                }
            }
            Event::AudioInput => {
                if let Some(hook) = self.on_audio_input.as_mut() {
                    hook(client, msg);
                }
            }
            Event::UserFirstVoiceStreamPacket => {
                if let Some(hook) = self.on_user_first_voice_stream_packet.as_mut() {
                    hook(client, msg);
                }
            }
            Event::SoundDeviceAdded => {
                if let Some(hook) = self.on_sound_device_added.as_mut() {
                    hook(client, msg);
                }
            }
            Event::SoundDeviceRemoved => {
                if let Some(hook) = self.on_sound_device_removed.as_mut() {
                    hook(client, msg);
                }
            }
            Event::SoundDeviceUnplugged => {
                if let Some(hook) = self.on_sound_device_unplugged.as_mut() {
                    hook(client, msg);
                }
            }
            Event::SoundDeviceNewDefaultInput => {
                if let Some(hook) = self.on_sound_device_new_default_input.as_mut() {
                    hook(client, msg);
                }
            }
            Event::SoundDeviceNewDefaultOutput => {
                if let Some(hook) = self.on_sound_device_new_default_output.as_mut() {
                    hook(client, msg);
                }
            }
            Event::SoundDeviceNewDefaultInputComDevice => {
                if let Some(hook) = self.on_sound_device_new_default_input_com_device.as_mut() {
                    hook(client, msg);
                }
            }
            Event::SoundDeviceNewDefaultOutputComDevice => {
                if let Some(hook) = self.on_sound_device_new_default_output_com_device.as_mut() {
                    hook(client, msg);
                }
            }
            Event::BeforeReconnect { .. } => {
                if let Some(hook) = self.on_before_reconnect.as_mut() {
                    hook(client, msg);
                }
            }
            Event::Reconnecting { .. } => {
                if let Some(hook) = self.on_reconnecting.as_mut() {
                    hook(client, msg);
                }
            }
            Event::AfterReconnect { .. } => {
                if let Some(hook) = self.on_after_reconnect.as_mut() {
                    hook(client, msg);
                }
            }
            Event::ReconnectFailed { .. } => {
                if let Some(hook) = self.on_reconnect_failed.as_mut() {
                    hook(client, msg);
                }
            }
            _ => {}
        }

        if let Some(hook) = self.on_event.as_mut() {
            hook(client, event, msg);
        }
    }

    pub(crate) fn fire_joined(&mut self, client: &Client, channel_id: ChannelId) {
        if let Some(hook) = self.on_joined.as_mut() {
            hook(client, channel_id);
        }
    }
}
