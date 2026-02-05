use crate::app::services::tg_admin as tg_admin_service;
use crate::app::services::{
    bans as bans_service, mute_lists as mute_lists_service, subscribers as subscribers_service,
    user_settings as user_settings_service,
};
use crate::app::state::StateHandle;
use crate::core::types::{LanguageCode, MuteListMode, TtUsername};
use crate::infra::db::Database;
pub async fn list_online_users(state: &StateHandle) -> Vec<crate::core::types::LiteUser> {
    tg_admin_service::list_online_users(state)
        .await
        .unwrap_or_default()
}

pub async fn list_ban_entries(db: &Database) -> Vec<crate::infra::db::types::BanEntry> {
    bans_service::get_banned_users(db).await.unwrap_or_default()
}

pub async fn list_subscribers(db: &Database) -> Vec<crate::infra::db::types::SubscriberInfo> {
    subscribers_service::get_subscribers(db)
        .await
        .unwrap_or_default()
}

pub async fn list_user_accounts(state: &StateHandle) -> Vec<teamtalk::types::UserAccount> {
    state.user_accounts_sorted().await.unwrap_or_default()
}

pub async fn list_muted_users(
    db: &Database,
    telegram_id: crate::core::types::TelegramId,
    mode: MuteListMode,
) -> Vec<TtUsername> {
    mute_lists_service::get_muted_users_list(db, telegram_id, mode)
        .await
        .unwrap_or_default()
}

pub async fn resolve_mute_mode(
    db: &Database,
    telegram_id: crate::core::types::TelegramId,
    lang: LanguageCode,
) -> MuteListMode {
    user_settings_service::get_or_create(db, telegram_id, lang)
        .await
        .ok()
        .map_or(MuteListMode::Blacklist, |s| s.mute_list_mode)
}
