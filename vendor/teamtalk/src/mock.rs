//! Mock event source for tests and offline simulations.
use crate::client::{Client, Message};
use crate::dispatch::EventSource;
use crate::events::Event;
use crate::types::{ChannelId, UserId, UserState, UserStatus};
use crate::utils::ToTT;
use std::collections::VecDeque;
use teamtalk_sys as ffi;

/// In-memory event queue implementing `EventSource`.
pub struct MockClient {
    queue: VecDeque<(Event, Message)>,
}

impl MockClient {
    /// Creates an empty mock client.
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// Pushes a raw event and message pair.
    pub fn push(&mut self, event: Event, message: Message) -> &mut Self {
        self.queue.push_back((event, message));
        self
    }

    /// Pushes an event with an empty message.
    pub fn push_event(&mut self, event: Event) -> &mut Self {
        self.push(event, MockMessage::empty())
    }

    /// Pushes a `UserJoined` event.
    pub fn push_user_joined(&mut self, user: MockUserBuilder) -> &mut Self {
        self.push(Event::UserJoined, user.build_for(Event::UserJoined))
    }

    /// Pushes a `UserUpdate` event.
    pub fn push_user_update(&mut self, user: MockUserBuilder) -> &mut Self {
        self.push(Event::UserUpdate, user.build_for(Event::UserUpdate))
    }

    /// Pushes a `TextMessage` event.
    pub fn push_text_message(&mut self, message: Message) -> &mut Self {
        self.push(Event::TextMessage, message)
    }

    /// Returns the number of queued events.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Returns true when no events are queued.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

impl Default for MockClient {
    fn default() -> Self {
        Self::new()
    }
}

impl EventSource for MockClient {
    fn poll(&mut self, _timeout_ms: i32) -> Option<(Event, Message)> {
        self.queue.pop_front()
    }

    fn client(&self) -> Option<&Client> {
        None
    }
}

/// Helpers for constructing mock `Message` instances.
pub struct MockMessage;

impl MockMessage {
    /// Returns an empty message instance.
    pub fn empty() -> Message {
        Message::from_raw(Event::None, unsafe { std::mem::zeroed() })
    }

    /// Builds a text message instance.
    pub fn text(
        msg_type: ffi::TextMsgType,
        from_id: UserId,
        to_id: UserId,
        channel_id: ChannelId,
        from_username: &str,
        text: &str,
    ) -> Message {
        let mut msg = unsafe { std::mem::zeroed::<ffi::TextMessage>() };
        msg.nMsgType = msg_type;
        msg.nFromUserID = from_id.0;
        msg.nToUserID = to_id.0;
        msg.nChannelID = channel_id.0;
        write_tt(&mut msg.szFromUsername, from_username);
        write_tt(&mut msg.szMessage, text);
        message_from_text(msg, from_id.0)
    }
}

/// Builder for mock user messages.
pub struct MockUserBuilder {
    user: ffi::User,
}

impl MockUserBuilder {
    /// Creates a new builder with the provided user id.
    pub fn new(id: UserId) -> Self {
        let mut user = unsafe { std::mem::zeroed::<ffi::User>() };
        user.nUserID = id.0;
        Self { user }
    }

    /// Sets the username field.
    pub fn username(mut self, username: &str) -> Self {
        write_tt(&mut self.user.szUsername, username);
        self
    }

    /// Sets the nickname field.
    pub fn nickname(mut self, nickname: &str) -> Self {
        write_tt(&mut self.user.szNickname, nickname);
        self
    }

    /// Sets the client name field.
    pub fn client_name(mut self, client_name: &str) -> Self {
        write_tt(&mut self.user.szClientName, client_name);
        self
    }

    /// Sets the IP address field.
    pub fn ip_address(mut self, ip_address: &str) -> Self {
        write_tt(&mut self.user.szIPAddress, ip_address);
        self
    }

    /// Sets the channel id field.
    pub fn channel_id(mut self, channel_id: ChannelId) -> Self {
        self.user.nChannelID = channel_id.0;
        self
    }

    /// Sets the status field.
    pub fn status(mut self, status: UserStatus) -> Self {
        self.user.nStatusMode = status.to_bits() as i32;
        self
    }

    /// Sets the state field.
    pub fn state(mut self, state: UserState) -> Self {
        self.user.uUserState = state.raw();
        self
    }

    /// Sets the user data field.
    pub fn user_data(mut self, user_data: i32) -> Self {
        self.user.nUserData = user_data;
        self
    }

    /// Sets the user type field.
    pub fn user_type(mut self, user_type: u32) -> Self {
        self.user.uUserType = user_type;
        self
    }

    /// Sets the protocol version field.
    pub fn version(mut self, version: u32) -> Self {
        self.user.uVersion = version;
        self
    }

    /// Builds the message.
    pub fn build_for(self, event: Event) -> Message {
        message_from_user(self.user, event)
    }
}

fn write_tt(dst: &mut [ffi::TTCHAR], value: &str) {
    dst.fill(0);
    let tt = value.tt();
    let len = tt.len().min(dst.len());
    dst[..len].copy_from_slice(&tt[..len]);
    if len == dst.len() {
        dst[dst.len() - 1] = 0;
    }
}

fn message_from_user(user: ffi::User, event: Event) -> Message {
    let mut msg = unsafe { std::mem::zeroed::<ffi::TTMessage>() };
    msg.nSource = user.nUserID;
    msg.ttType = ffi::TTType::__USER;
    msg.__bindgen_anon_1.user = user;
    Message::from_raw(event, msg)
}

fn message_from_text(text: ffi::TextMessage, source: i32) -> Message {
    let mut msg = unsafe { std::mem::zeroed::<ffi::TTMessage>() };
    msg.nSource = source;
    msg.ttType = ffi::TTType::__TEXTMESSAGE;
    msg.__bindgen_anon_1.textmessage = text;
    Message::from_raw(Event::TextMessage, msg)
}
