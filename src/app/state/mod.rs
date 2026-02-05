use crate::core::types::{
    LanguageCode, LiteUser, TelegramId, TtChannelName, TtNickname, TtUserId, TtUsername,
};
use teamtalk::types::UserAccount;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

mod cache;
mod online_users;
mod user_accounts;

pub use cache::CacheStatsSnapshot as TtCacheStatsSnapshot;

#[derive(Clone)]
pub struct StateHandle {
    tx: mpsc::Sender<StateCmd>,
}

pub type StateResult<T> = Result<T, StateError>;

#[derive(Debug, Error)]
pub enum StateError {
    #[error("state channel closed")]
    ChannelClosed,
    #[error("state response dropped")]
    ResponseDropped,
}

enum StateCmd {
    GetOnlineUsers {
        resp: oneshot::Sender<Vec<LiteUser>>,
    },
    GetOnlineUserById {
        user_id: TtUserId,
        resp: oneshot::Sender<Option<LiteUser>>,
    },
    GetUserIdByUsername {
        username: TtUsername,
        resp: oneshot::Sender<Option<TtUserId>>,
    },
    IsUsernameOnline {
        username: TtUsername,
        resp: oneshot::Sender<bool>,
    },
    UpsertOnlineUser {
        user: LiteUser,
    },
    UpdateUserUsername {
        user_id: TtUserId,
        username: TtUsername,
    },
    UpdateUserNickname {
        user_id: TtUserId,
        nickname: TtNickname,
    },
    UpdateUserChannel {
        user_id: TtUserId,
        channel_name: TtChannelName,
    },
    RemoveOnlineUser {
        user_id: TtUserId,
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
        cache: std::collections::HashMap<TtUsername, LanguageCode>,
    },
    PreloadTgCache {
        cache: std::collections::HashMap<TtUsername, TelegramId>,
    },
    GetLangCached {
        username: TtUsername,
        resp: oneshot::Sender<Option<LanguageCode>>,
    },
    GetTgCached {
        username: TtUsername,
        resp: oneshot::Sender<Option<TelegramId>>,
    },
    SetLangCached {
        username: TtUsername,
        lang: LanguageCode,
    },
    SetTgCached {
        username: TtUsername,
        tg_id: TelegramId,
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

    pub fn notify_update_user_username(&self, user_id: TtUserId, username: TtUsername) {
        let _ = self
            .tx
            .try_send(StateCmd::UpdateUserUsername { user_id, username });
    }

    pub fn notify_update_user_nickname(&self, user_id: TtUserId, nickname: TtNickname) {
        let _ = self
            .tx
            .try_send(StateCmd::UpdateUserNickname { user_id, nickname });
    }

    pub fn notify_update_user_channel(&self, user_id: TtUserId, channel_name: TtChannelName) {
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

    pub async fn online_users_sorted(&self) -> StateResult<Vec<LiteUser>> {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(StateCmd::GetOnlineUsers { resp: tx })
            .await
            .is_err()
        {
            return Err(StateError::ChannelClosed);
        }
        rx.await.map_err(|_| StateError::ResponseDropped)
    }

    pub async fn online_user_by_id(&self, user_id: TtUserId) -> StateResult<Option<LiteUser>> {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(StateCmd::GetOnlineUserById { user_id, resp: tx })
            .await
            .is_err()
        {
            return Err(StateError::ChannelClosed);
        }
        rx.await.map_err(|_| StateError::ResponseDropped)
    }

    pub async fn user_id_by_username(
        &self,
        username: &TtUsername,
    ) -> StateResult<Option<TtUserId>> {
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
            return Err(StateError::ChannelClosed);
        }
        rx.await.map_err(|_| StateError::ResponseDropped)
    }

    pub async fn is_username_online(&self, username: &TtUsername) -> StateResult<bool> {
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
            return Err(StateError::ChannelClosed);
        }
        rx.await.map_err(|_| StateError::ResponseDropped)
    }

    pub async fn remove_online_user(&self, user_id: TtUserId) -> StateResult<Option<LiteUser>> {
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
            return Err(StateError::ChannelClosed);
        }
        rx.await.map_err(|_| StateError::ResponseDropped)
    }

    pub async fn user_accounts_sorted(&self) -> StateResult<Vec<UserAccount>> {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(StateCmd::GetUserAccounts { resp: tx })
            .await
            .is_err()
        {
            return Err(StateError::ChannelClosed);
        }
        rx.await.map_err(|_| StateError::ResponseDropped)
    }

    pub fn preload_lang_cache(&self, cache: std::collections::HashMap<TtUsername, LanguageCode>) {
        let _ = self.tx.try_send(StateCmd::PreloadLangCache { cache });
    }

    pub fn preload_tg_cache(&self, cache: std::collections::HashMap<TtUsername, TelegramId>) {
        let _ = self.tx.try_send(StateCmd::PreloadTgCache { cache });
    }

    pub async fn get_lang_cached(
        &self,
        username: &TtUsername,
    ) -> StateResult<Option<LanguageCode>> {
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
            return Err(StateError::ChannelClosed);
        }
        rx.await.map_err(|_| StateError::ResponseDropped)
    }

    pub async fn get_tg_cached(&self, username: &TtUsername) -> StateResult<Option<TelegramId>> {
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
            return Err(StateError::ChannelClosed);
        }
        rx.await.map_err(|_| StateError::ResponseDropped)
    }

    pub fn set_lang_cached(&self, username: TtUsername, lang: LanguageCode) {
        let _ = self.tx.try_send(StateCmd::SetLangCached { username, lang });
    }

    pub fn set_tg_cached(&self, username: TtUsername, tg_id: TelegramId) {
        let _ = self.tx.try_send(StateCmd::SetTgCached { username, tg_id });
    }

    pub async fn cache_stats(&self) -> StateResult<TtCacheStatsSnapshot> {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(StateCmd::GetCacheStats { resp: tx })
            .await
            .is_err()
        {
            return Err(StateError::ChannelClosed);
        }
        rx.await.map_err(|_| StateError::ResponseDropped)
    }
}

async fn run_state(mut rx: mpsc::Receiver<StateCmd>) {
    let mut store = StateStore::new();
    while let Some(cmd) = rx.recv().await {
        store.handle(cmd);
    }
}

struct StateStore {
    online_users: online_users::OnlineUsersStore,
    user_accounts: user_accounts::UserAccountsStore,
    cache: cache::CacheStore,
}

impl StateStore {
    fn new() -> Self {
        Self {
            online_users: online_users::OnlineUsersStore::new(),
            user_accounts: user_accounts::UserAccountsStore::new(),
            cache: cache::CacheStore::new(),
        }
    }

    fn handle(&mut self, cmd: StateCmd) {
        match cmd {
            StateCmd::GetOnlineUsers { resp } => {
                let _ = resp.send(self.online_users.get_users_sorted());
            }
            StateCmd::GetOnlineUserById { user_id, resp } => {
                let _ = resp.send(self.online_users.get_by_id(user_id));
            }
            StateCmd::GetUserIdByUsername { username, resp } => {
                let _ = resp.send(self.online_users.get_id_by_username(&username));
            }
            StateCmd::IsUsernameOnline { username, resp } => {
                let _ = resp.send(self.online_users.is_username_online(&username));
            }
            StateCmd::UpsertOnlineUser { user } => self.online_users.upsert_user(user),
            StateCmd::UpdateUserUsername { user_id, username } => {
                self.online_users.update_user_username(user_id, username);
            }
            StateCmd::UpdateUserNickname { user_id, nickname } => {
                self.online_users.update_user_nickname(user_id, nickname);
            }
            StateCmd::UpdateUserChannel {
                user_id,
                channel_name,
            } => {
                self.online_users.update_user_channel(user_id, channel_name);
            }
            StateCmd::RemoveOnlineUser { user_id, resp } => {
                let removed = self.online_users.remove_user(user_id);
                if let Some(resp) = resp {
                    let _ = resp.send(removed);
                }
            }
            StateCmd::ClearOnlineUsers => self.online_users.clear(),
            StateCmd::GetUserAccounts { resp } => {
                let _ = resp.send(self.user_accounts.get_sorted());
            }
            StateCmd::UpsertUserAccount { account } => self.user_accounts.upsert(account),
            StateCmd::ClearUserAccounts => self.user_accounts.clear(),
            StateCmd::PreloadLangCache { cache } => self.cache.preload_lang(cache),
            StateCmd::PreloadTgCache { cache } => self.cache.preload_tg(cache),
            StateCmd::GetLangCached { username, resp } => {
                let val = self.cache.get_lang(&username);
                let _ = resp.send(val);
            }
            StateCmd::GetTgCached { username, resp } => {
                let val = self.cache.get_tg(&username);
                let _ = resp.send(val);
            }
            StateCmd::SetLangCached { username, lang } => self.cache.set_lang(username, lang),
            StateCmd::SetTgCached { username, tg_id } => self.cache.set_tg(username, tg_id),
            StateCmd::GetCacheStats { resp } => {
                let _ = resp.send(self.cache.snapshot());
            }
        }
    }
}
