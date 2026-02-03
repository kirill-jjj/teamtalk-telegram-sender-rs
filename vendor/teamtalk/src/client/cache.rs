use super::Client;
use crate::events::Event;
use crate::types::{Channel, ChannelId, ServerProperties, ServerStatistics, User, UserId};
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CacheState {
    pub(crate) users: HashMap<UserId, User>,
    pub(crate) channels: HashMap<ChannelId, Channel>,
    pub(crate) server_props: Option<ServerProperties>,
    pub(crate) server_stats: Option<ServerStatistics>,
    pub(crate) users_auto: bool,
    pub(crate) channels_auto: bool,
}

/// Aggregated server snapshot from cached data.
#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub properties: Option<ServerProperties>,
    pub statistics: Option<ServerStatistics>,
    pub users: Vec<User>,
    pub channels: Vec<Channel>,
}

impl Client {
    /// Enables the user cache. When auto-sync is true, events update the cache.
    pub fn enable_user_cache(&self, auto_sync: bool) {
        let mut cache = self.cache.lock().unwrap();
        cache.users_auto = auto_sync;
    }

    /// Enables the channel cache. When auto-sync is true, events update the cache.
    pub fn enable_channel_cache(&self, auto_sync: bool) {
        let mut cache = self.cache.lock().unwrap();
        cache.channels_auto = auto_sync;
    }

    /// Clears the cached users.
    pub fn clear_user_cache(&self) {
        self.cache.lock().unwrap().users.clear();
    }

    /// Clears the cached channels.
    pub fn clear_channel_cache(&self) {
        self.cache.lock().unwrap().channels.clear();
    }

    /// Refreshes the user cache from the server.
    pub fn refresh_user_cache(&self) -> Vec<User> {
        let users = self.get_server_users();
        let mut cache = self.cache.lock().unwrap();
        cache.users = users.iter().map(|u| (u.id, u.clone())).collect();
        users
    }

    /// Refreshes the channel cache from the server.
    pub fn refresh_channel_cache(&self) -> Vec<Channel> {
        let channels = self.get_server_channels();
        let mut cache = self.cache.lock().unwrap();
        cache.channels = channels.iter().map(|c| (c.id, c.clone())).collect();
        channels
    }

    /// Returns a cached user, if present.
    pub fn cached_user(&self, user_id: UserId) -> Option<User> {
        self.cache.lock().unwrap().users.get(&user_id).cloned()
    }

    /// Returns all cached users.
    pub fn cached_users(&self) -> Vec<User> {
        self.cache.lock().unwrap().users.values().cloned().collect()
    }

    /// Returns a cached user by username, if present.
    pub fn cached_user_by_username(&self, username: &str) -> Option<User> {
        self.cache
            .lock()
            .unwrap()
            .users
            .values()
            .find(|u| u.username == username)
            .cloned()
    }

    /// Returns a cached channel, if present.
    pub fn cached_channel(&self, channel_id: ChannelId) -> Option<Channel> {
        self.cache
            .lock()
            .unwrap()
            .channels
            .get(&channel_id)
            .cloned()
    }

    /// Returns all cached channels.
    pub fn cached_channels(&self) -> Vec<Channel> {
        self.cache
            .lock()
            .unwrap()
            .channels
            .values()
            .cloned()
            .collect()
    }

    /// Returns a cached channel by name, if present.
    pub fn cached_channel_by_name(&self, name: &str) -> Option<Channel> {
        self.cache
            .lock()
            .unwrap()
            .channels
            .values()
            .find(|c| c.name == name)
            .cloned()
    }

    /// Returns a cached channel by path, if present.
    pub fn cached_channel_by_path(&self, path: &str) -> Option<Channel> {
        self.cache
            .lock()
            .unwrap()
            .channels
            .values()
            .find(|c| self.get_channel_path(c.id).as_deref() == Some(path))
            .cloned()
    }

    /// Returns the last cached server properties and statistics.
    pub fn server_info(&self) -> ServerInfo {
        let cache = self.cache.lock().unwrap();
        ServerInfo {
            properties: cache.server_props.clone(),
            statistics: cache.server_stats.clone(),
            users: cache.users.values().cloned().collect(),
            channels: cache.channels.values().cloned().collect(),
        }
    }

    pub(crate) fn update_cache_for_event(&self, event: Event, msg: &super::Message) {
        let mut cache = self.cache.lock().unwrap();
        match event {
            Event::UserLoggedIn | Event::UserUpdate | Event::UserJoined | Event::UserLeft => {
                if cache.users_auto
                    && let Some(user) = msg.user()
                {
                    cache.users.insert(user.id, user);
                }
            }
            Event::UserLoggedOut => {
                if cache.users_auto
                    && let Some(user) = msg.user()
                {
                    cache.users.remove(&user.id);
                }
            }
            Event::ChannelCreated | Event::ChannelUpdated => {
                if cache.channels_auto
                    && let Some(channel) = msg.channel()
                {
                    cache.channels.insert(channel.id, channel);
                }
            }
            Event::ChannelRemoved => {
                if cache.channels_auto
                    && let Some(channel) = msg.channel()
                {
                    cache.channels.remove(&channel.id);
                }
            }
            Event::ServerUpdate => {
                cache.server_props = msg.server_properties();
            }
            Event::ServerStatistics => {
                cache.server_stats = msg.server_statistics();
            }
            _ => {}
        }
    }
}
