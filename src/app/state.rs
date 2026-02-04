use crate::core::types::{LanguageCode, LiteUser, TtChannelName, TtUsername};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use teamtalk::types::UserAccount;
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub struct StateHandle {
    tx: mpsc::Sender<StateCmd>,
}

enum StateCmd {
    GetOnlineUsers {
        resp: oneshot::Sender<Vec<LiteUser>>,
    },
    GetOnlineUserById {
        user_id: i32,
        resp: oneshot::Sender<Option<LiteUser>>,
    },
    GetUserIdByUsername {
        username: TtUsername,
        resp: oneshot::Sender<Option<i32>>,
    },
    IsUsernameOnline {
        username: TtUsername,
        resp: oneshot::Sender<bool>,
    },
    UpsertOnlineUser {
        user: LiteUser,
    },
    UpdateUserUsername {
        user_id: i32,
        username: TtUsername,
    },
    UpdateUserNickname {
        user_id: i32,
        nickname: String,
    },
    UpdateUserChannel {
        user_id: i32,
        channel_name: TtChannelName,
    },
    RemoveOnlineUser {
        user_id: i32,
        resp: Option<oneshot::Sender<Option<LiteUser>>>,
    },
    ClearOnlineUsers,
    GetUserAccounts {
        resp: oneshot::Sender<Vec<UserAccount>>,
    },
    UpsertUserAccount {
        account: UserAccount,
    },
    ClearUserAccounts,
    PreloadLangCache {
        cache: HashMap<TtUsername, LanguageCode>,
    },
    PreloadTgCache {
        cache: HashMap<TtUsername, i64>,
    },
    GetLangCached {
        username: TtUsername,
        resp: oneshot::Sender<Option<LanguageCode>>,
    },
    GetTgCached {
        username: TtUsername,
        resp: oneshot::Sender<Option<i64>>,
    },
    SetLangCached {
        username: TtUsername,
        lang: LanguageCode,
    },
    SetTgCached {
        username: TtUsername,
        tg_id: i64,
    },
    GetCacheStats {
        resp: oneshot::Sender<TtCacheStatsSnapshot>,
    },
}

impl StateHandle {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1024);
        tokio::spawn(run_state(rx));
        Self { tx }
    }

    pub fn notify_upsert_online_user(&self, user: LiteUser) {
        let _ = self.tx.try_send(StateCmd::UpsertOnlineUser { user });
    }

    pub fn notify_update_user_username(&self, user_id: i32, username: TtUsername) {
        let _ = self
            .tx
            .try_send(StateCmd::UpdateUserUsername { user_id, username });
    }

    pub fn notify_update_user_nickname(&self, user_id: i32, nickname: String) {
        let _ = self
            .tx
            .try_send(StateCmd::UpdateUserNickname { user_id, nickname });
    }

    pub fn notify_update_user_channel(&self, user_id: i32, channel_name: TtChannelName) {
        let _ = self.tx.try_send(StateCmd::UpdateUserChannel {
            user_id,
            channel_name,
        });
    }

    pub fn notify_clear_online_users(&self) {
        let _ = self.tx.try_send(StateCmd::ClearOnlineUsers);
    }

    pub fn notify_clear_user_accounts(&self) {
        let _ = self.tx.try_send(StateCmd::ClearUserAccounts);
    }

    pub fn notify_upsert_user_account(&self, account: UserAccount) {
        let _ = self.tx.try_send(StateCmd::UpsertUserAccount { account });
    }

    pub async fn online_users_sorted(&self) -> Vec<LiteUser> {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(StateCmd::GetOnlineUsers { resp: tx })
            .await
            .is_err()
        {
            return Vec::new();
        }
        rx.await.unwrap_or_default()
    }

    pub async fn online_user_by_id(&self, user_id: i32) -> Option<LiteUser> {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(StateCmd::GetOnlineUserById { user_id, resp: tx })
            .await
            .is_err()
        {
            return None;
        }
        rx.await.ok().flatten()
    }

    pub async fn user_id_by_username(&self, username: &TtUsername) -> Option<i32> {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(StateCmd::GetUserIdByUsername {
                username: username.clone(),
                resp: tx,
            })
            .await
            .is_err()
        {
            return None;
        }
        rx.await.ok().flatten()
    }

    pub async fn is_username_online(&self, username: &TtUsername) -> bool {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(StateCmd::IsUsernameOnline {
                username: username.clone(),
                resp: tx,
            })
            .await
            .is_err()
        {
            return false;
        }
        rx.await.unwrap_or(false)
    }

    pub async fn remove_online_user(&self, user_id: i32) -> Option<LiteUser> {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(StateCmd::RemoveOnlineUser {
                user_id,
                resp: Some(tx),
            })
            .await
            .is_err()
        {
            return None;
        }
        rx.await.ok().flatten()
    }

    pub async fn user_accounts_sorted(&self) -> Vec<UserAccount> {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(StateCmd::GetUserAccounts { resp: tx })
            .await
            .is_err()
        {
            return Vec::new();
        }
        rx.await.unwrap_or_default()
    }

    pub fn preload_lang_cache(&self, cache: HashMap<TtUsername, LanguageCode>) {
        let _ = self.tx.try_send(StateCmd::PreloadLangCache { cache });
    }

    pub fn preload_tg_cache(&self, cache: HashMap<TtUsername, i64>) {
        let _ = self.tx.try_send(StateCmd::PreloadTgCache { cache });
    }

    pub async fn get_lang_cached(&self, username: &TtUsername) -> Option<LanguageCode> {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(StateCmd::GetLangCached {
                username: username.clone(),
                resp: tx,
            })
            .await
            .is_err()
        {
            return None;
        }
        rx.await.ok().flatten()
    }

    pub async fn get_tg_cached(&self, username: &TtUsername) -> Option<i64> {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(StateCmd::GetTgCached {
                username: username.clone(),
                resp: tx,
            })
            .await
            .is_err()
        {
            return None;
        }
        rx.await.ok().flatten()
    }

    pub fn set_lang_cached(&self, username: TtUsername, lang: LanguageCode) {
        let _ = self.tx.try_send(StateCmd::SetLangCached { username, lang });
    }

    pub fn set_tg_cached(&self, username: TtUsername, tg_id: i64) {
        let _ = self.tx.try_send(StateCmd::SetTgCached { username, tg_id });
    }

    pub async fn cache_stats(&self) -> TtCacheStatsSnapshot {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(StateCmd::GetCacheStats { resp: tx })
            .await
            .is_err()
        {
            return TtCacheStatsSnapshot::default();
        }
        rx.await.unwrap_or_default()
    }
}

async fn run_state(mut rx: mpsc::Receiver<StateCmd>) {
    let mut store = StateStore::new();
    while let Some(cmd) = rx.recv().await {
        store.handle(cmd);
    }
}

struct StateStore {
    online_users: HashMap<i32, LiteUser>,
    online_users_by_username: HashMap<TtUsername, i32>,
    user_accounts: HashMap<TtUsername, UserAccount>,
    tt_lang_cache: HashMap<TtUsername, LanguageCode>,
    tt_tg_cache: HashMap<TtUsername, i64>,
    stats: TtCacheStats,
}

impl StateStore {
    fn new() -> Self {
        Self {
            online_users: HashMap::new(),
            online_users_by_username: HashMap::new(),
            user_accounts: HashMap::new(),
            tt_lang_cache: HashMap::new(),
            tt_tg_cache: HashMap::new(),
            stats: TtCacheStats::default(),
        }
    }

    fn handle(&mut self, cmd: StateCmd) {
        match cmd {
            StateCmd::GetOnlineUsers { resp } => self.handle_get_online_users(resp),
            StateCmd::GetOnlineUserById { user_id, resp } => {
                let _ = resp.send(self.online_users.get(&user_id).cloned());
            }
            StateCmd::GetUserIdByUsername { username, resp } => {
                let _ = resp.send(self.online_users_by_username.get(&username).copied());
            }
            StateCmd::IsUsernameOnline { username, resp } => {
                let _ = resp.send(self.online_users_by_username.contains_key(&username));
            }
            StateCmd::UpsertOnlineUser { user } => self.handle_upsert_online_user(user),
            StateCmd::UpdateUserUsername { user_id, username } => {
                self.handle_update_user_username(user_id, username);
            }
            StateCmd::UpdateUserNickname { user_id, nickname } => {
                if let Some(existing) = self.online_users.get_mut(&user_id) {
                    existing.nickname = nickname;
                }
            }
            StateCmd::UpdateUserChannel {
                user_id,
                channel_name,
            } => {
                if let Some(existing) = self.online_users.get_mut(&user_id) {
                    existing.channel_name = channel_name;
                }
            }
            StateCmd::RemoveOnlineUser { user_id, resp } => {
                let removed = self.online_users.remove(&user_id);
                if let Some(user) = &removed
                    && !user.username.as_str().is_empty()
                {
                    self.online_users_by_username.remove(&user.username);
                }
                if let Some(resp) = resp {
                    let _ = resp.send(removed);
                }
            }
            StateCmd::ClearOnlineUsers => self.handle_clear_online_users(),
            StateCmd::GetUserAccounts { resp } => self.handle_get_user_accounts(resp),
            StateCmd::UpsertUserAccount { account } => self.handle_upsert_user_account(account),
            StateCmd::ClearUserAccounts => {
                self.user_accounts.clear();
            }
            StateCmd::PreloadLangCache { cache } => {
                self.tt_lang_cache = cache;
            }
            StateCmd::PreloadTgCache { cache } => {
                self.tt_tg_cache = cache;
            }
            StateCmd::GetLangCached { username, resp } => {
                let val = self.tt_lang_cache.get(&username).copied();
                if val.is_some() {
                    self.stats.lang_hits.fetch_add(1, Ordering::Relaxed);
                } else {
                    self.stats.lang_misses.fetch_add(1, Ordering::Relaxed);
                }
                let _ = resp.send(val);
            }
            StateCmd::GetTgCached { username, resp } => {
                let val = self.tt_tg_cache.get(&username).copied();
                if val.is_some() {
                    self.stats.tg_hits.fetch_add(1, Ordering::Relaxed);
                } else {
                    self.stats.tg_misses.fetch_add(1, Ordering::Relaxed);
                }
                let _ = resp.send(val);
            }
            StateCmd::SetLangCached { username, lang } => {
                if self.tt_lang_cache.len() > 5000 {
                    self.tt_lang_cache.clear();
                }
                self.tt_lang_cache.insert(username, lang);
            }
            StateCmd::SetTgCached { username, tg_id } => {
                if self.tt_tg_cache.len() > 5000 {
                    self.tt_tg_cache.clear();
                }
                self.tt_tg_cache.insert(username, tg_id);
            }
            StateCmd::GetCacheStats { resp } => {
                let snapshot = TtCacheStatsSnapshot {
                    lang_hits: self.stats.lang_hits.load(Ordering::Relaxed),
                    lang_misses: self.stats.lang_misses.load(Ordering::Relaxed),
                    tg_hits: self.stats.tg_hits.load(Ordering::Relaxed),
                    tg_misses: self.stats.tg_misses.load(Ordering::Relaxed),
                };
                let _ = resp.send(snapshot);
            }
        }
    }

    fn handle_get_online_users(&self, resp: oneshot::Sender<Vec<LiteUser>>) {
        let mut users: Vec<LiteUser> = self.online_users.values().cloned().collect();
        users.sort_by(|a, b| a.nickname.to_lowercase().cmp(&b.nickname.to_lowercase()));
        let _ = resp.send(users);
    }

    fn handle_upsert_online_user(&mut self, user: LiteUser) {
        if !user.username.as_str().is_empty() {
            self.online_users_by_username
                .insert(user.username.clone(), user.id);
        }
        self.online_users.insert(user.id, user);
    }

    fn handle_update_user_username(&mut self, user_id: i32, username: TtUsername) {
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

    fn handle_clear_online_users(&mut self) {
        self.online_users.clear();
        self.online_users_by_username.clear();
    }

    fn handle_get_user_accounts(&self, resp: oneshot::Sender<Vec<UserAccount>>) {
        let mut accounts: Vec<UserAccount> = self.user_accounts.values().cloned().collect();
        accounts.sort_by(|a, b| a.username.to_lowercase().cmp(&b.username.to_lowercase()));
        let _ = resp.send(accounts);
    }

    fn handle_upsert_user_account(&mut self, account: UserAccount) {
        if !account.username.is_empty() {
            self.user_accounts
                .insert(TtUsername::new(account.username.clone()), account);
        }
    }
}

#[derive(Default)]
struct TtCacheStats {
    lang_hits: AtomicU64,
    lang_misses: AtomicU64,
    tg_hits: AtomicU64,
    tg_misses: AtomicU64,
}

#[derive(Default, Clone, Copy)]
pub struct TtCacheStatsSnapshot {
    pub lang_hits: u64,
    pub lang_misses: u64,
    pub tg_hits: u64,
    pub tg_misses: u64,
}
