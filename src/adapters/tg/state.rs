use crate::bootstrap::config::Config;
use crate::core::types::{LiteUser, TtCommand};
use crate::infra::db::Database;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use teamtalk::types::UserAccount;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub online_users: Arc<RwLock<HashMap<i32, LiteUser>>>,
    pub online_users_by_username: Arc<RwLock<HashMap<String, i32>>>,
    pub user_accounts: Arc<RwLock<HashMap<String, UserAccount>>>,
    pub search_contexts:
        Arc<Mutex<HashMap<teloxide::types::ChatId, crate::adapters::tg::search::SearchContext>>>,
    pub tx_tt: Sender<TtCommand>,
    pub config: Arc<Config>,
    pub cancel_token: tokio_util::sync::CancellationToken,
}
