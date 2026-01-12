use crate::bootstrap::config::Config;
use crate::core::types::{LiteUser, TtCommand};
use crate::infra::db::Database;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::mpsc::Sender;
use teamtalk::types::UserAccount;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub online_users: Arc<RwLock<HashMap<i32, LiteUser>>>,
    pub user_accounts: Arc<RwLock<HashMap<String, UserAccount>>>,
    pub tx_tt: Sender<TtCommand>,
    pub config: Arc<Config>,
    pub shutdown_tx: tokio::sync::watch::Sender<bool>,
}
