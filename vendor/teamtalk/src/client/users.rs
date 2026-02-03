//! User management APIs.
use super::Client;
use crate::types::{
    ChannelId, MessageTarget, Subscriptions, User, UserAccount, UserId, UserStatistics, UserStatus,
};
use crate::utils::ToTT;
use std::env;
use teamtalk_sys as ffi;

/// Stored login parameters for automatic login.
#[derive(Debug, Clone)]
pub struct LoginParams {
    pub nickname: String,
    pub username: String,
    pub password: String,
    pub client_name: String,
}

impl LoginParams {
    pub fn new(
        nickname: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
        client_name: impl Into<String>,
    ) -> Self {
        Self {
            nickname: nickname.into(),
            username: username.into(),
            password: password.into(),
            client_name: client_name.into(),
        }
    }

    pub fn from_env() -> Self {
        let nickname = env::var("TT_NICK").unwrap_or_default();
        let username = env::var("TT_USER").unwrap_or_default();
        let password = env::var("TT_PASS").unwrap_or_default();
        let client_name = env::var("TT_CLIENT").unwrap_or_default();
        Self::new(nickname, username, password, client_name)
    }
}

impl Client {
    /// Logs in to the server.
    pub fn login(&self, nickname: &str, username: &str, password: &str, client_name: &str) -> i32 {
        let cmd_id =
            self.backend()
                .do_login_ex(self.ptr.0, nickname, username, password, client_name);
        if cmd_id > 0 {
            self.set_connection_state(crate::events::ConnectionState::LoggingIn);
        }
        cmd_id
    }

    /// Stores login parameters for automatic login.
    pub fn set_login_params(&self, params: LoginParams) {
        self.auto_reconnect.lock().unwrap().login = Some(params);
    }

    /// Returns stored login parameters, if any.
    pub fn login_params(&self) -> Option<LoginParams> {
        self.auto_reconnect.lock().unwrap().login.clone()
    }

    /// Logs in using stored login parameters.
    pub fn login_with_params(&self) -> Result<i32, crate::events::Error> {
        let params = self
            .login_params()
            .ok_or(crate::events::Error::MissingLoginParams)?;
        Ok(self.login(
            &params.nickname,
            &params.username,
            &params.password,
            &params.client_name,
        ))
    }

    /// Stores login parameters and immediately logs in.
    pub fn login_remember(
        &self,
        nickname: &str,
        username: &str,
        password: &str,
        client_name: &str,
    ) -> i32 {
        let params = LoginParams::new(nickname, username, password, client_name);
        self.set_login_params(params);
        self.login(nickname, username, password, client_name)
    }

    pub fn login_from_env(&self) -> i32 {
        let params = LoginParams::from_env();
        self.set_login_params(params.clone());
        self.login(
            &params.nickname,
            &params.username,
            &params.password,
            &params.client_name,
        )
    }

    /// Logs out from the server.
    pub fn logout(&self) -> i32 {
        let cmd_id = self.backend().do_logout(self.ptr.0);
        if cmd_id > 0 {
            self.set_connection_state(crate::events::ConnectionState::Connected);
        }
        cmd_id
    }

    /// Returns the current user id.
    pub fn my_id(&self) -> UserId {
        UserId(self.backend().get_my_user_id(self.ptr.0))
    }

    /// Returns the account of the current user.
    pub fn get_my_user_account(&self) -> Option<UserAccount> {
        let mut raw = unsafe { std::mem::zeroed::<ffi::UserAccount>() };
        if unsafe { ffi::api().TT_GetMyUserAccount(self.ptr.0, &mut raw) } == 1 {
            Some(UserAccount::from(raw))
        } else {
            None
        }
    }

    /// Returns the user type of the current user.
    pub fn get_my_user_type(&self) -> u32 {
        unsafe { ffi::api().TT_GetMyUserType(self.ptr.0) }
    }

    /// Returns the user rights of the current user.
    pub fn get_my_user_rights(&self) -> u32 {
        unsafe { ffi::api().TT_GetMyUserRights(self.ptr.0) }
    }

    /// Requests user data for the current user.
    pub fn get_my_user_data(&self) -> i32 {
        unsafe { ffi::api().TT_GetMyUserData(self.ptr.0) }
    }

    /// Changes the current nickname.
    pub fn change_nickname(&self, nick: &str) -> i32 {
        unsafe { ffi::api().TT_DoChangeNickname(self.ptr.0, nick.tt().as_ptr()) }
    }

    /// Sets the status and status message.
    pub fn set_status(&self, status: UserStatus, msg: &str) -> i32 {
        unsafe {
            ffi::api().TT_DoChangeStatus(self.ptr.0, status.to_bits() as i32, msg.tt().as_ptr())
        }
    }

    /// Updates only the status message.
    pub fn set_status_message(&self, msg: &str) -> i32 {
        let mut user = unsafe { std::mem::zeroed::<ffi::User>() };
        let my_id = self.my_id();
        let bits = if self.backend().get_user(self.ptr.0, my_id.0, &mut user) {
            user.nStatusMode as u32
        } else {
            UserStatus::default().to_bits()
        };
        self.backend()
            .do_change_status(self.ptr.0, bits as i32, msg)
    }

    /// Kicks a user from a channel.
    pub fn kick_user(&self, user_id: UserId, channel_id: ChannelId) -> i32 {
        unsafe { ffi::api().TT_DoKickUser(self.ptr.0, user_id.0, channel_id.0) }
    }

    /// Bans a user from a channel.
    pub fn ban_user(&self, user_id: UserId, channel_id: ChannelId) -> i32 {
        unsafe { ffi::api().TT_DoBanUser(self.ptr.0, user_id.0, channel_id.0) }
    }

    /// Bans a user with custom ban types.
    pub fn ban_user_ex(&self, user_id: UserId, ban_types: u32) -> i32 {
        unsafe { ffi::api().TT_DoBanUserEx(self.ptr.0, user_id.0, ban_types) }
    }

    /// Removes a ban by IP address.
    pub fn unban_user(&self, ip: &str, channel_id: ChannelId) -> i32 {
        unsafe { ffi::api().TT_DoUnBanUser(self.ptr.0, ip.tt().as_ptr(), channel_id.0) }
    }

    /// Sends a text message to a target.
    pub fn send_text<T: Into<MessageTarget>>(&self, target: T, text: &str) -> i32 {
        let mut msg = unsafe { std::mem::zeroed::<ffi::TextMessage>() };
        match target.into() {
            MessageTarget::User(id) => {
                msg.nMsgType = ffi::TextMsgType::MSGTYPE_USER;
                msg.nToUserID = id.0;
            }
            MessageTarget::Channel(id) => {
                msg.nMsgType = ffi::TextMsgType::MSGTYPE_CHANNEL;
                msg.nChannelID = id.0;
            }
            MessageTarget::Broadcast => {
                msg.nMsgType = ffi::TextMsgType::MSGTYPE_BROADCAST;
            }
        }
        let t = text.tt();
        unsafe {
            let len = t.len().min(511);
            std::ptr::copy_nonoverlapping(t.as_ptr(), msg.szMessage.as_mut_ptr(), len);
            self.backend().do_text_message(self.ptr.0, &msg)
        }
    }

    /// Sends a text message to a user.
    pub fn send_to_user(&self, user_id: UserId, text: &str) -> i32 {
        self.send_text(MessageTarget::User(user_id), text)
    }

    /// Sends a text message to a channel.
    pub fn send_to_channel(&self, channel_id: ChannelId, text: &str) -> i32 {
        self.send_text(MessageTarget::Channel(channel_id), text)
    }

    /// Sends a text message to all users.
    pub fn send_to_all(&self, text: &str) -> i32 {
        self.send_text(MessageTarget::Broadcast, text)
    }

    /// Adds a user to the ban list.
    pub fn ban(&self, banned_user: &crate::types::BannedUser) -> i32 {
        unsafe { ffi::api().TT_DoBan(self.ptr.0, &banned_user.to_ffi()) }
    }

    /// Removes a user from the ban list.
    pub fn unban_ex(&self, banned_user: &crate::types::BannedUser) -> i32 {
        unsafe { ffi::api().TT_DoUnBanUserEx(self.ptr.0, &banned_user.to_ffi()) }
    }

    /// Returns a user by id.
    pub fn get_user(&self, user_id: UserId) -> Option<User> {
        let mut raw = unsafe { std::mem::zeroed::<ffi::User>() };
        if unsafe { ffi::api().TT_GetUser(self.ptr.0, user_id.0, &mut raw) } == 1 {
            Some(User::from(raw))
        } else {
            None
        }
    }

    /// Returns a user by username.
    pub fn get_user_by_username(&self, username: &str) -> Option<User> {
        let mut raw = unsafe { std::mem::zeroed::<ffi::User>() };
        if unsafe { ffi::api().TT_GetUserByUsername(self.ptr.0, username.tt().as_ptr(), &mut raw) }
            == 1
        {
            Some(User::from(raw))
        } else {
            None
        }
    }

    /// Returns user statistics by id.
    pub fn get_user_statistics(&self, user_id: UserId) -> Option<UserStatistics> {
        let mut raw = unsafe { std::mem::zeroed::<ffi::UserStatistics>() };
        if unsafe { ffi::api().TT_GetUserStatistics(self.ptr.0, user_id.0, &mut raw) } == 1 {
            Some(UserStatistics::from(raw))
        } else {
            None
        }
    }

    /// Requests a list of user accounts.
    pub fn list_user_accounts(&self, index: i32, count: i32) -> i32 {
        unsafe { ffi::api().TT_DoListUserAccounts(self.ptr.0, index, count) }
    }

    /// Creates a user account.
    pub fn create_user_account(&self, account: &UserAccount) -> i32 {
        unsafe { ffi::api().TT_DoNewUserAccount(self.ptr.0, &account.to_ffi()) }
    }

    /// Deletes a user account by username.
    pub fn delete_user_account(&self, username: &str) -> i32 {
        unsafe { ffi::api().TT_DoDeleteUserAccount(self.ptr.0, username.tt().as_ptr()) }
    }

    /// Subscribes to a user's streams.
    pub fn subscribe(&self, user_id: UserId, mask: Subscriptions) -> i32 {
        unsafe { ffi::api().TT_DoSubscribe(self.ptr.0, user_id.0, mask.raw()) }
    }

    /// Unsubscribes from a user's streams.
    pub fn unsubscribe(&self, user_id: UserId, mask: Subscriptions) -> i32 {
        unsafe { ffi::api().TT_DoUnsubscribe(self.ptr.0, user_id.0, mask.raw()) }
    }

    /// Unsubscribes from all streams for a user.
    pub fn unsubscribe_all_from_user(&self, user_id: UserId) -> i32 {
        unsafe { ffi::api().TT_DoUnsubscribe(self.ptr.0, user_id.0, Subscriptions::ALL) }
    }

    /// Unsubscribes from all streams for all users.
    pub fn unsubscribe_all(&self) -> i32 {
        unsafe { ffi::api().TT_DoUnsubscribe(self.ptr.0, 0, Subscriptions::ALL) }
    }

    pub fn channel_op_ex(
        &self,
        user_id: UserId,
        channel_id: ChannelId,
        password: &str,
        make_op: bool,
    ) -> i32 {
        unsafe {
            ffi::api().TT_DoChannelOpEx(
                self.ptr.0,
                user_id.0,
                channel_id.0,
                password.tt().as_ptr(),
                if make_op { 1 } else { 0 },
            )
        }
    }

    pub fn set_channel_operator(
        &self,
        user_id: UserId,
        channel_id: ChannelId,
        make_op: bool,
    ) -> i32 {
        self.channel_op_ex(user_id, channel_id, "", make_op)
    }

    pub fn set_user_operator(&self, user_id: UserId, channel_id: ChannelId, make_op: bool) -> i32 {
        self.set_channel_operator(user_id, channel_id, make_op)
    }

    pub fn set_user_operator_ex(
        &self,
        user_id: UserId,
        channel_id: ChannelId,
        password: &str,
        make_op: bool,
    ) -> i32 {
        self.channel_op_ex(user_id, channel_id, password, make_op)
    }

    pub fn set_user_text_mute(&self, user_id: UserId, mute: bool) -> i32 {
        if mute {
            self.unsubscribe(user_id, Subscriptions::all_text())
        } else {
            self.subscribe(user_id, Subscriptions::all_text())
        }
    }

    /// Grants operator status to a user in a channel.
    pub fn op_user(&self, user_id: UserId, channel_id: ChannelId) -> i32 {
        self.set_channel_operator(user_id, channel_id, true)
    }

    /// Revokes operator status from a user in a channel.
    pub fn deop_user(&self, user_id: UserId, channel_id: ChannelId) -> i32 {
        self.set_channel_operator(user_id, channel_id, false)
    }

    /// Mutes text messages from a user (local subscription).
    pub fn mute_user_text(&self, user_id: UserId) -> i32 {
        self.set_user_text_mute(user_id, true)
    }

    /// Unmutes text messages from a user (local subscription).
    pub fn unmute_user_text(&self, user_id: UserId) -> i32 {
        self.set_user_text_mute(user_id, false)
    }

    /// Grants operator status to a user in a channel (with password).
    pub fn op_user_ex(&self, user_id: UserId, channel_id: ChannelId, password: &str) -> i32 {
        self.set_user_operator_ex(user_id, channel_id, password, true)
    }

    /// Revokes operator status from a user in a channel (with password).
    pub fn deop_user_ex(&self, user_id: UserId, channel_id: ChannelId, password: &str) -> i32 {
        self.set_user_operator_ex(user_id, channel_id, password, false)
    }

    /// Mutes voice streams from a user (local subscription).
    pub fn mute_user_voice(&self, user_id: UserId) -> i32 {
        self.unsubscribe(user_id, Subscriptions::from_raw(Subscriptions::VOICE))
    }

    /// Unmutes voice streams from a user (local subscription).
    pub fn unmute_user_voice(&self, user_id: UserId) -> i32 {
        self.subscribe(user_id, Subscriptions::from_raw(Subscriptions::VOICE))
    }

    /// Mutes media file streams from a user (local subscription).
    pub fn mute_user_media(&self, user_id: UserId) -> i32 {
        self.unsubscribe(user_id, Subscriptions::from_raw(Subscriptions::MEDIAFILE))
    }

    /// Unmutes media file streams from a user (local subscription).
    pub fn unmute_user_media(&self, user_id: UserId) -> i32 {
        self.subscribe(user_id, Subscriptions::from_raw(Subscriptions::MEDIAFILE))
    }

    /// Returns the current user's subscription mask.
    pub fn my_subscriptions(&self) -> Subscriptions {
        let mut user = unsafe { std::mem::zeroed::<ffi::User>() };
        let my_id = self.my_id();
        if unsafe { ffi::api().TT_GetUser(self.ptr.0, my_id.0, &mut user) } == 1 {
            Subscriptions::from_raw(user.uLocalSubscriptions)
        } else {
            Subscriptions::new()
        }
    }
}
