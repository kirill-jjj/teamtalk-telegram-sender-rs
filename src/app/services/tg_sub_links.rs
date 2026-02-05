use crate::app::services::subscriber_actions as subscriber_actions_service;
use crate::app::services::user_settings as user_settings_service;
use crate::app::state::StateHandle;
use crate::core::types::{LanguageCode, TelegramId, TtCommand, TtUsername};
use crate::infra::db::Database;
use anyhow::Result;

pub async fn load_settings(
    db: &Database,
    sub_id: TelegramId,
    lang: LanguageCode,
) -> Result<crate::infra::db::types::UserSettings> {
    user_settings_service::get_or_create(db, sub_id, lang).await
}

pub async fn unlink_tt(db: &Database, sub_id: TelegramId) -> Result<()> {
    subscriber_actions_service::unlink_tt(db, sub_id).await
}

pub async fn link_tt(db: &Database, sub_id: TelegramId, username: &TtUsername) -> Result<()> {
    subscriber_actions_service::link_tt(db, sub_id, username).await
}

pub async fn load_accounts(
    tx_tt: &tokio::sync::mpsc::Sender<TtCommand>,
    state: &StateHandle,
) -> Vec<teamtalk::types::UserAccount> {
    let _ = tx_tt.send(TtCommand::LoadAccounts).await;
    state.user_accounts_sorted().await.unwrap_or_default()
}
