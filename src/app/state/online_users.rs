use crate::core::types::{LiteUser, TtChannelName, TtNickname, TtUserId, TtUsername};
use std::collections::{HashMap, HashSet};

pub struct OnlineUsersStore {
    online_users: HashMap<TtUserId, LiteUser>,
    online_users_by_username: HashMap<TtUsername, HashSet<TtUserId>>,
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
        self.online_users_by_username
            .get(username)
            .and_then(|ids| ids.iter().next().copied())
    }

    pub fn is_username_online(&self, username: &TtUsername) -> bool {
        self.online_users_by_username
            .get(username)
            .is_some_and(|ids| !ids.is_empty())
    }

    pub fn upsert_user(&mut self, user: LiteUser) {
        let user_id = user.id;
        let username = user.username.clone();

        if let Some(existing) = self.online_users.get(&user_id).cloned()
            && !existing.username.as_str().is_empty()
        {
            self.remove_username_ref(&existing.username, existing.id);
        }

        if !username.as_str().is_empty() {
            self.online_users_by_username
                .entry(username)
                .or_default()
                .insert(user_id);
        }
        self.online_users.insert(user_id, user);
    }

    pub fn update_user_username(&mut self, user_id: TtUserId, username: TtUsername) {
        let old_username = self.online_users.get(&user_id).map(|u| u.username.clone());

        if let Some(old_username) = old_username
            && !old_username.as_str().is_empty()
        {
            self.remove_username_ref(&old_username, user_id);
        }

        if !username.as_str().is_empty() {
            self.online_users_by_username
                .entry(username.clone())
                .or_default()
                .insert(user_id);
        }

        if let Some(existing) = self.online_users.get_mut(&user_id) {
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
            self.remove_username_ref(&user.username, user_id);
        }
        removed
    }

    pub fn clear(&mut self) {
        self.online_users.clear();
        self.online_users_by_username.clear();
    }

    fn remove_username_ref(&mut self, username: &TtUsername, user_id: TtUserId) {
        if let Some(ids) = self.online_users_by_username.get_mut(username) {
            ids.remove(&user_id);
            if ids.is_empty() {
                self.online_users_by_username.remove(username);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OnlineUsersStore;
    use crate::core::types::{LiteUser, TtChannelName, TtNickname, TtUserId, TtUsername};

    fn mk_user(id: i32, username: &str, nickname: &str) -> LiteUser {
        LiteUser {
            id: TtUserId::from(id),
            username: TtUsername::from(username),
            nickname: TtNickname::from(nickname),
            channel_name: TtChannelName::from(String::from("root")),
        }
    }

    #[test]
    fn username_stays_online_until_last_session_leaves() {
        let mut store = OnlineUsersStore::new();
        store.upsert_user(mk_user(1, "kirill", "Kirill phone"));
        store.upsert_user(mk_user(2, "kirill", "Kirill desktop"));

        assert!(store.is_username_online(&TtUsername::from("kirill")));

        store.remove_user(TtUserId::from(1));
        assert!(store.is_username_online(&TtUsername::from("kirill")));

        store.remove_user(TtUserId::from(2));
        assert!(!store.is_username_online(&TtUsername::from("kirill")));
    }

    #[test]
    fn update_username_removes_old_mapping_only_for_that_session() {
        let mut store = OnlineUsersStore::new();
        store.upsert_user(mk_user(1, "shared", "one"));
        store.upsert_user(mk_user(2, "shared", "two"));

        store.update_user_username(TtUserId::from(1), TtUsername::from("new_name"));

        assert!(store.is_username_online(&TtUsername::from("shared")));
        assert!(store.is_username_online(&TtUsername::from("new_name")));
    }
}
