use crate::core::types::{LiteUser, TtChannelName, TtUsername};
use std::collections::HashMap;
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
}

async fn run_state(mut rx: mpsc::Receiver<StateCmd>) {
    let mut online_users: HashMap<i32, LiteUser> = HashMap::new();
    let mut online_users_by_username: HashMap<TtUsername, i32> = HashMap::new();
    let mut user_accounts: HashMap<TtUsername, UserAccount> = HashMap::new();

    while let Some(cmd) = rx.recv().await {
        match cmd {
            StateCmd::GetOnlineUsers { resp } => {
                let mut users: Vec<LiteUser> = online_users.values().cloned().collect();
                users.sort_by(|a, b| a.nickname.to_lowercase().cmp(&b.nickname.to_lowercase()));
                let _ = resp.send(users);
            }
            StateCmd::GetOnlineUserById { user_id, resp } => {
                let _ = resp.send(online_users.get(&user_id).cloned());
            }
            StateCmd::GetUserIdByUsername { username, resp } => {
                let _ = resp.send(online_users_by_username.get(&username).copied());
            }
            StateCmd::IsUsernameOnline { username, resp } => {
                let _ = resp.send(online_users_by_username.contains_key(&username));
            }
            StateCmd::UpsertOnlineUser { user } => {
                if !user.username.as_str().is_empty() {
                    online_users_by_username.insert(user.username.clone(), user.id);
                }
                online_users.insert(user.id, user);
            }
            StateCmd::UpdateUserUsername { user_id, username } => {
                if let Some(existing) = online_users.get_mut(&user_id) {
                    if !existing.username.as_str().is_empty() {
                        online_users_by_username.remove(&existing.username);
                    }
                    if !username.as_str().is_empty() {
                        online_users_by_username.insert(username.clone(), user_id);
                    }
                    existing.username = username;
                }
            }
            StateCmd::UpdateUserNickname { user_id, nickname } => {
                if let Some(existing) = online_users.get_mut(&user_id) {
                    existing.nickname = nickname;
                }
            }
            StateCmd::UpdateUserChannel {
                user_id,
                channel_name,
            } => {
                if let Some(existing) = online_users.get_mut(&user_id) {
                    existing.channel_name = channel_name;
                }
            }
            StateCmd::RemoveOnlineUser { user_id, resp } => {
                let removed = online_users.remove(&user_id);
                if let Some(user) = &removed
                    && !user.username.as_str().is_empty()
                {
                    online_users_by_username.remove(&user.username);
                }
                if let Some(resp) = resp {
                    let _ = resp.send(removed);
                }
            }
            StateCmd::ClearOnlineUsers => {
                online_users.clear();
                online_users_by_username.clear();
            }
            StateCmd::GetUserAccounts { resp } => {
                let mut accounts: Vec<UserAccount> = user_accounts.values().cloned().collect();
                accounts.sort_by(|a, b| a.username.to_lowercase().cmp(&b.username.to_lowercase()));
                let _ = resp.send(accounts);
            }
            StateCmd::UpsertUserAccount { account } => {
                if !account.username.is_empty() {
                    user_accounts.insert(TtUsername::new(account.username.clone()), account);
                }
            }
            StateCmd::ClearUserAccounts => {
                user_accounts.clear();
            }
        }
    }
}
