use crate::app::services::tg_admin as tg_admin_service;
use crate::app::services::{
    bans as bans_service, mute_lists as mute_lists_service, subscribers as subscribers_service,
    user_settings as user_settings_service,
};
use crate::app::state::StateHandle;
use crate::core::types::{LanguageCode, MuteListMode, TtUsername};
use crate::infra::db::Database;
pub async fn list_online_users(state: &StateHandle) -> Vec<crate::core::types::LiteUser> {
    match tg_admin_service::list_online_users(state).await {
        Ok(users) => users,
        Err(err) => {
            tracing::error!(error = ?err, "Failed to list online users");
            Vec::new()
        }
    }
}

pub async fn list_ban_entries(db: &Database) -> Vec<crate::infra::db::types::BanEntry> {
    match bans_service::get_banned_users(db).await {
        Ok(entries) => entries,
        Err(err) => {
            tracing::error!(error = ?err, "Failed to list ban entries");
            Vec::new()
        }
    }
}

pub async fn list_subscribers(db: &Database) -> Vec<crate::infra::db::types::SubscriberInfo> {
    match subscribers_service::get_subscribers(db).await {
        Ok(subs) => subs,
        Err(err) => {
            tracing::error!(error = ?err, "Failed to list subscribers");
            Vec::new()
        }
    }
}

pub async fn list_user_accounts(state: &StateHandle) -> Vec<teamtalk::types::UserAccount> {
    match state.user_accounts_sorted().await {
        Ok(accounts) => accounts,
        Err(err) => {
            tracing::error!(error = ?err, "Failed to list TeamTalk accounts");
            Vec::new()
        }
    }
}

pub async fn list_muted_users(
    db: &Database,
    telegram_id: crate::core::types::TelegramId,
    mode: MuteListMode,
) -> Vec<TtUsername> {
    match mute_lists_service::get_muted_users_list(db, telegram_id, mode).await {
        Ok(items) => items,
        Err(err) => {
            tracing::error!(
                telegram_id = telegram_id.as_i64(),
                error = %err,
                "Failed to list muted users"
            );
            Vec::new()
        }
    }
}

pub async fn resolve_mute_mode(
    db: &Database,
    telegram_id: crate::core::types::TelegramId,
    lang: LanguageCode,
) -> MuteListMode {
    match user_settings_service::get_or_create(db, telegram_id, lang).await {
        Ok(settings) => settings.mute_list_mode,
        Err(err) => {
            tracing::error!(
                telegram_id = telegram_id.as_i64(),
                error = %err,
                "Failed to resolve mute mode, using blacklist"
            );
            MuteListMode::Blacklist
        }
    }
}
