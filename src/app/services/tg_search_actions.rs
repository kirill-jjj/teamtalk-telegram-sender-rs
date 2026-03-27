use crate::app::services::admin_bans as admin_bans_service;
use crate::app::services::mute_lists as mute_lists_service;
use crate::app::services::subscriber_actions as subscriber_actions_service;
use crate::app::services::subscribers as subscribers_service;
use crate::app::services::tg_search as tg_search_service;
use crate::app::services::user_settings as user_settings_service;
use crate::app::state::StateHandle;
use crate::core::types::{LanguageCode, MuteListMode, TelegramId, TtUsername};
use crate::infra::db::Database;
use anyhow::Result;

pub async fn remove_ban(db: &Database, ban_db_id: crate::core::types::DbBanId) -> Result<()> {
    admin_bans_service::remove_ban_by_id(db, ban_db_id).await
}

pub async fn load_subscriber_details(
    db: &Database,
    sub_id: TelegramId,
    lang: LanguageCode,
) -> Result<crate::app::services::subscribers::SubscriberDetails> {
    subscribers_service::get_subscriber_details(db, sub_id, lang).await
}

pub async fn link_tt(db: &Database, sub_id: TelegramId, username: &TtUsername) -> Result<()> {
    subscriber_actions_service::link_tt(db, sub_id, username).await
}

pub async fn load_user_settings(
    db: &Database,
    sub_id: TelegramId,
    lang: LanguageCode,
) -> Result<crate::infra::db::types::UserSettings> {
    user_settings_service::get_or_create(db, sub_id, lang).await
}

pub async fn toggle_mute(
    db: &Database,
    telegram_id: TelegramId,
    mode: MuteListMode,
    username: &TtUsername,
) -> Result<()> {
    mute_lists_service::toggle_muted_user(db, telegram_id, mode, username).await
}

pub async fn list_muted_users(
    db: &Database,
    telegram_id: TelegramId,
    mode: MuteListMode,
) -> Vec<TtUsername> {
    tg_search_service::list_muted_users(db, telegram_id, mode).await
}

pub async fn list_user_accounts(state: &StateHandle) -> Vec<teamtalk::types::UserAccount> {
    tg_search_service::list_user_accounts(state).await
}
