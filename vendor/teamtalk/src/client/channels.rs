//! Channel management APIs.
use super::Client;
use crate::events::ConnectionState;
use crate::types::{Channel, ChannelId, UserId};
use crate::utils::ToTT;
use teamtalk_sys as ffi;

impl Client {
    /// Returns available channels from the server.
    pub fn get_server_channels(&self) -> Vec<Channel> {
        let mut count: i32 = 0;
        unsafe {
            ffi::api().TT_GetServerChannels(self.ptr.0, std::ptr::null_mut(), &mut count);
            let mut channels = vec![std::mem::zeroed::<ffi::Channel>(); count as usize];
            if ffi::api().TT_GetServerChannels(self.ptr.0, channels.as_mut_ptr(), &mut count) == 1 {
                channels.into_iter().map(Channel::from).collect()
            } else {
                vec![]
            }
        }
    }

    /// Returns a channel by id.
    pub fn get_channel(&self, id: ChannelId) -> Option<Channel> {
        self.backend().get_channel(self.ptr.0, id.0)
    }

    /// Returns a channel path by id.
    pub fn get_channel_path(&self, id: ChannelId) -> Option<String> {
        use crate::types::TT_STRLEN;
        use crate::utils::strings::tt_buf;
        let mut buf = tt_buf::<TT_STRLEN>();
        unsafe {
            if ffi::api().TT_GetChannelPath(self.ptr.0, id.0, buf.as_mut_ptr()) == 1 {
                Some(crate::utils::strings::to_string(&buf))
            } else {
                None
            }
        }
    }

    /// Returns a channel id from a path.
    pub fn get_channel_id_from_path(&self, path: &str) -> ChannelId {
        ChannelId(unsafe { ffi::api().TT_GetChannelIDFromPath(self.ptr.0, path.tt().as_ptr()) })
    }

    /// Joins a channel.
    pub fn join_channel(&self, id: ChannelId, password: &str) -> i32 {
        let cmd_id = self
            .backend()
            .do_join_channel_by_id(self.ptr.0, id.0, password);
        if cmd_id > 0 {
            self.auto_reconnect.lock().unwrap().last_channel = Some(id);
            self.set_connection_state(ConnectionState::Joining(id));
        }
        cmd_id
    }

    /// Joins a channel by id without a password.
    pub fn join_channel_unprotected(&self, channel_id: ChannelId) -> i32 {
        self.join_channel(channel_id, "")
    }

    /// Joins a channel path.
    pub fn join_channel_path(&self, path: &str, password: &str) -> i32 {
        let id = self.get_channel_id_from_path(path);
        if id.0 > 0 {
            self.join_channel(id, password)
        } else {
            0
        }
    }

    /// Joins a channel path without a password.
    pub fn join_channel_path_unprotected(&self, path: &str) -> i32 {
        self.join_channel_path(path, "")
    }

    /// Leaves the current channel.
    pub fn leave_channel(&self) -> i32 {
        let cmd_id = self.backend().do_leave_channel(self.ptr.0);
        if cmd_id > 0 {
            self.auto_reconnect.lock().unwrap().last_channel = None;
        }
        cmd_id
    }

    /// Creates a new channel.
    pub fn make_channel(&self, channel: &Channel) -> i32 {
        unsafe { ffi::api().TT_DoMakeChannel(self.ptr.0, &channel.to_ffi()) }
    }

    /// Updates an existing channel.
    pub fn update_channel(&self, channel: &Channel) -> i32 {
        unsafe { ffi::api().TT_DoUpdateChannel(self.ptr.0, &channel.to_ffi()) }
    }

    /// Removes a channel.
    pub fn remove_channel(&self, id: ChannelId) -> i32 {
        unsafe { ffi::api().TT_DoRemoveChannel(self.ptr.0, id.0) }
    }

    /// Moves a user to a different channel.
    pub fn move_user(&self, user_id: UserId, channel_id: ChannelId) -> i32 {
        unsafe { ffi::api().TT_DoMoveUser(self.ptr.0, user_id.0, channel_id.0) }
    }

    /// Checks if a user is an operator in a channel.
    pub fn is_channel_operator(&self, user_id: UserId, channel_id: ChannelId) -> bool {
        unsafe { ffi::api().TT_IsChannelOperator(self.ptr.0, user_id.0, channel_id.0) == 1 }
    }

    /// Joins the root channel.
    pub fn join_root(&self) -> i32 {
        let root = ChannelId(unsafe { ffi::api().TT_GetRootChannelID(self.ptr.0) });
        self.join_channel(root, "")
    }

    /// Leaves the current channel and joins the root channel.
    pub fn leave_to_root(&self) -> i32 {
        let _ = self.leave_channel();
        self.join_root()
    }

    /// Returns the channel ID where the current user is.
    pub fn my_channel_id(&self) -> ChannelId {
        self.backend().get_my_channel_id(self.ptr.0)
    }

    /// Returns users in a channel.
    pub fn get_channel_users(&self, channel_id: ChannelId) -> Vec<crate::types::User> {
        let mut count: i32 = 0;
        unsafe {
            ffi::api().TT_GetChannelUsers(
                self.ptr.0,
                channel_id.0,
                std::ptr::null_mut(),
                &mut count,
            );
            let mut users = vec![std::mem::zeroed::<ffi::User>(); count as usize];
            if ffi::api().TT_GetChannelUsers(
                self.ptr.0,
                channel_id.0,
                users.as_mut_ptr(),
                &mut count,
            ) == 1
            {
                users.into_iter().map(crate::types::User::from).collect()
            } else {
                vec![]
            }
        }
    }
}
