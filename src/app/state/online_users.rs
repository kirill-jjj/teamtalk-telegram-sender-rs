use crate::core::types::{LiteUser, TtChannelName, TtNickname, TtUserId, TtUsername};
use std::collections::HashMap;

pub struct OnlineUsersStore {
    online_users: HashMap<TtUserId, LiteUser>,
    online_users_by_username: HashMap<TtUsername, TtUserId>,
}

impl OnlineUsersStore {
    pub fn new() -> Self {
        Self {
            online_users: HashMap::new(),
            online_users_by_username: HashMap::new(),
        }
    }

    pub fn get_users_sorted(&self) -> Vec<LiteUser> {
        let mut users: Vec<LiteUser> = self.online_users.values().cloned().collect();
        users.sort_by(|a, b| {
            a.nickname
                .as_str()
                .to_lowercase()
                .cmp(&b.nickname.as_str().to_lowercase())
        });
        users
    }

    pub fn get_by_id(&self, user_id: TtUserId) -> Option<LiteUser> {
        self.online_users.get(&user_id).cloned()
    }

    pub fn get_id_by_username(&self, username: &TtUsername) -> Option<TtUserId> {
        self.online_users_by_username.get(username).copied()
    }

    pub fn is_username_online(&self, username: &TtUsername) -> bool {
        self.online_users_by_username.contains_key(username)
    }

    pub fn upsert_user(&mut self, user: LiteUser) {
        if !user.username.as_str().is_empty() {
            self.online_users_by_username
                .insert(user.username.clone(), user.id);
        }
        self.online_users.insert(user.id, user);
    }

    pub fn update_user_username(&mut self, user_id: TtUserId, username: TtUsername) {
        if let Some(existing) = self.online_users.get_mut(&user_id) {
            if !existing.username.as_str().is_empty() {
                self.online_users_by_username.remove(&existing.username);
            }
            if !username.as_str().is_empty() {
                self.online_users_by_username
                    .insert(username.clone(), user_id);
            }
            existing.username = username;
        }
    }

    pub fn update_user_nickname(&mut self, user_id: TtUserId, nickname: TtNickname) {
        if let Some(existing) = self.online_users.get_mut(&user_id) {
            existing.nickname = nickname;
        }
    }

    pub fn update_user_channel(&mut self, user_id: TtUserId, channel_name: TtChannelName) {
        if let Some(existing) = self.online_users.get_mut(&user_id) {
            existing.channel_name = channel_name;
        }
    }

    pub fn remove_user(&mut self, user_id: TtUserId) -> Option<LiteUser> {
        let removed = self.online_users.remove(&user_id);
        if let Some(user) = &removed
            && !user.username.as_str().is_empty()
        {
            self.online_users_by_username.remove(&user.username);
        }
        removed
    }

    pub fn clear(&mut self) {
        self.online_users.clear();
        self.online_users_by_username.clear();
    }
}
