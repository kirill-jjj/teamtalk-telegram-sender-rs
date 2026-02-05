use crate::core::types::TtUsername;
use std::collections::HashMap;
use teamtalk::types::UserAccount;

pub struct UserAccountsStore {
    user_accounts: HashMap<TtUsername, UserAccount>,
}

impl UserAccountsStore {
    pub fn new() -> Self {
        Self {
            user_accounts: HashMap::new(),
        }
    }

    pub fn get_sorted(&self) -> Vec<UserAccount> {
        let mut accounts: Vec<UserAccount> = self.user_accounts.values().cloned().collect();
        accounts.sort_by(|a, b| a.username.to_lowercase().cmp(&b.username.to_lowercase()));
        accounts
    }

    pub fn upsert(&mut self, account: UserAccount) {
        if !account.username.is_empty() {
            self.user_accounts
                .insert(TtUsername::new(account.username.clone()), account);
        }
    }

    pub fn clear(&mut self) {
        self.user_accounts.clear();
    }
}
