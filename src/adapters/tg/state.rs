use crate::app::plugins::PluginManagerHandle;
use crate::app::state::StateHandle;
use crate::bootstrap::config::Config;
use crate::core::types::TtCommand;
use crate::infra::db::Database;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub state: StateHandle,
    pub search_contexts: Arc<
        Mutex<
            HashMap<teloxide::types::ChatId, crate::adapters::tg::handlers::search::SearchContext>,
        >,
    >,
    pub tx_tt: Sender<TtCommand>,
    pub plugins: PluginManagerHandle,
    pub config: Arc<Config>,
    pub cancel_token: tokio_util::sync::CancellationToken,
}
