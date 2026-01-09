use crate::config::Config;
use crate::db::Database;
use crate::types::{LiteUser, TtCommand};
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use teamtalk::types::UserAccount;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub online_users: Arc<DashMap<i32, LiteUser>>,
    pub user_accounts: Arc<DashMap<String, UserAccount>>,
    pub tx_tt: Sender<TtCommand>,
    pub config: Arc<Config>,
}
