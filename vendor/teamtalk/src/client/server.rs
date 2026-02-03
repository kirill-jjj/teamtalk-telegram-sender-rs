//! Server management APIs.
use super::Client;
use crate::types::{ChannelId, ServerProperties, User, UserId};
use crate::utils::ToTT;
use teamtalk_sys as ffi;

impl Client {
    /// Returns current server properties.
    pub fn get_server_properties(&self) -> Option<ServerProperties> {
        let mut raw = unsafe { std::mem::zeroed::<ffi::ServerProperties>() };
        if unsafe { ffi::api().TT_GetServerProperties(self.ptr.0, &mut raw) } == 1 {
            Some(ServerProperties::from(raw))
        } else {
            None
        }
    }

    /// Returns all users on the server.
    pub fn get_server_users(&self) -> Vec<User> {
        let mut count: i32 = 0;
        unsafe {
            ffi::api().TT_GetServerUsers(self.ptr.0, std::ptr::null_mut(), &mut count);
            let mut users = vec![std::mem::zeroed::<ffi::User>(); count as usize];
            if ffi::api().TT_GetServerUsers(self.ptr.0, users.as_mut_ptr(), &mut count) == 1 {
                users.into_iter().map(User::from).collect()
            } else {
                vec![]
            }
        }
    }

    /// Bans an IP address.
    pub fn ban_ip(&self, ip: &str, ban_type: i32) -> i32 {
        unsafe { ffi::api().TT_DoBanIPAddress(self.ptr.0, ip.tt().as_ptr(), ban_type) }
    }

    /// Returns client statistics.
    pub fn get_client_statistics(&self) -> Option<crate::types::ClientStatistics> {
        let mut raw = unsafe { std::mem::zeroed::<ffi::ClientStatistics>() };
        if unsafe { ffi::api().TT_GetClientStatistics(self.ptr.0, &mut raw) } == 1 {
            Some(crate::types::ClientStatistics::from(raw))
        } else {
            None
        }
    }

    /// Requests a list of bans.
    pub fn list_bans(&self, channel_id: ChannelId, index: i32, count: i32) -> i32 {
        unsafe { ffi::api().TT_DoListBans(self.ptr.0, channel_id.0, index, count) }
    }

    /// Updates server properties.
    pub fn update_server(&self, props: &ServerProperties) -> i32 {
        unsafe { ffi::api().TT_DoUpdateServer(self.ptr.0, &props.to_ffi()) }
    }

    /// Saves the server configuration.
    pub fn save_server_config(&self) -> i32 {
        unsafe { ffi::api().TT_DoSaveConfig(self.ptr.0) }
    }

    /// Returns the root channel ID.
    pub fn get_root_channel_id(&self) -> ChannelId {
        ChannelId(unsafe { ffi::api().TT_GetRootChannelID(self.ptr.0) })
    }

    /// Requests server statistics.
    pub fn query_server_stats(&self) -> i32 {
        unsafe { ffi::api().TT_DoQueryServerStats(self.ptr.0) }
    }

    /// Queries the max payload for a user.
    pub fn query_max_payload(&self, user_id: UserId) -> bool {
        unsafe { ffi::api().TT_QueryMaxPayload(self.ptr.0, user_id.0) == 1 }
    }

    /// Pumps a message into the Windows message loop (Windows only).
    #[cfg(windows)]
    pub fn pump_message(&self, event: ffi::ClientEvent, id: i32) -> bool {
        unsafe { ffi::api().TT_PumpMessage(self.ptr.0, event, id) == 1 }
    }

    /// Quits the TeamTalk client (for standalone apps).
    pub fn quit(&self) -> i32 {
        unsafe { ffi::api().TT_DoQuit(self.ptr.0) }
    }
}
