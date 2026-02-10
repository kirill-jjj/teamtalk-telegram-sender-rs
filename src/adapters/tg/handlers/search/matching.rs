use crate::adapters::tg::presenter::admin::subscribers as subscribers_logic;
use crate::adapters::tg::state::AppState;
use crate::app::services::tg_search as tg_search_service;
use crate::core::callbacks::{AdminAction, CallbackAction, MuteAction, SubAction};
use crate::core::types::{LanguageCode, MuteListMode, TtUsername};
use crate::infra::db::types::BanEntry;
use teloxide_ng::prelude::{Bot, ResponseResult};

use super::context::{SearchCandidate, SearchListType, format_display_subscriber};

pub(super) async fn find_matches(
    state: &AppState,
    list_type: &SearchListType,
    bot: &Bot,
    query: &str,
    lang: LanguageCode,
) -> ResponseResult<Vec<SearchCandidate>> {
    let normalized_query = query.to_lowercase();
    let mut candidates: Vec<SearchCandidate> = match list_type {
        SearchListType::Kick => candidates_kick(state).await,
        SearchListType::Ban => candidates_ban(state).await,
        SearchListType::Unban => candidates_unban(state).await,
        SearchListType::Subscribers => candidates_subscribers(state, bot).await,
        SearchListType::MuteServer { mode, page, .. } => {
            candidates_mute_server(state, mode, *page).await
        }
        SearchListType::MuteLocal {
            telegram_id,
            mode,
            page,
        } => candidates_mute_local(state, *telegram_id, mode, *page).await,
        SearchListType::SubMuteView {
            sub_id,
            sub_page,
            view_page,
        } => candidates_sub_mute_view(state, *sub_id, *sub_page, *view_page, lang).await,
        SearchListType::LinkList { sub_id, page } => {
            candidates_link_list(state, *sub_id, *page).await
        }
    };

    candidates.retain(|c| c.match_key.to_lowercase().contains(&normalized_query));
    candidates.sort_by(|a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase()));
    Ok(candidates)
}

async fn candidates_kick(state: &AppState) -> Vec<SearchCandidate> {
    let users = tg_search_service::list_online_users(&state.state).await;
    users
        .into_iter()
        .map(|u| SearchCandidate {
            label: u.nickname.as_str().to_string(),
            match_key: format!("{} {}", u.nickname.as_str(), u.username),
            action: CallbackAction::Admin(AdminAction::KickPerform { user_id: u.id }),
        })
        .collect::<Vec<_>>()
}

async fn candidates_ban(state: &AppState) -> Vec<SearchCandidate> {
    let users = tg_search_service::list_online_users(&state.state).await;
    users
        .into_iter()
        .map(|u| SearchCandidate {
            label: u.nickname.as_str().to_string(),
            match_key: format!("{} {}", u.nickname.as_str(), u.username),
            action: CallbackAction::Admin(AdminAction::BanPerform { user_id: u.id }),
        })
        .collect::<Vec<_>>()
}

async fn candidates_unban(state: &AppState) -> Vec<SearchCandidate> {
    let entries = tg_search_service::list_ban_entries(&state.db).await;
    entries
        .into_iter()
        .filter_map(ban_entry_candidate)
        .collect::<Vec<_>>()
}

async fn candidates_subscribers(state: &AppState, bot: &Bot) -> Vec<SearchCandidate> {
    let subs = tg_search_service::list_subscribers(&state.db).await;
    let display_list = subscribers_logic::prepare_display_list(bot, subs).await;
    display_list
        .into_iter()
        .map(|s| {
            let mut match_key = s.display_name.clone();
            if let Some(tt) = &s.tt_username {
                match_key.push(' ');
                match_key.push_str(tt.as_str());
            }
            SearchCandidate {
                label: format_display_subscriber(&s.display_name, s.tt_username.as_ref()),
                match_key,
                action: CallbackAction::Subscriber(SubAction::Details {
                    sub_id: s.telegram_id,
                    page: 0,
                }),
            }
        })
        .collect::<Vec<_>>()
}

async fn candidates_mute_server(
    state: &AppState,
    mode: &MuteListMode,
    page: usize,
) -> Vec<SearchCandidate> {
    let accounts = tg_search_service::list_user_accounts(&state.state).await;
    accounts
        .into_iter()
        .map(|acc| {
            let username = TtUsername::new(acc.username);
            let label = username.to_string();
            let match_key = label.clone();
            SearchCandidate {
                label,
                match_key,
                action: CallbackAction::Mute(MuteAction::ServerToggle {
                    mode: mode.clone(),
                    username,
                    page,
                }),
            }
        })
        .collect::<Vec<_>>()
}

async fn candidates_mute_local(
    state: &AppState,
    telegram_id: crate::core::types::TelegramId,
    mode: &MuteListMode,
    page: usize,
) -> Vec<SearchCandidate> {
    let muted = tg_search_service::list_muted_users(&state.db, telegram_id, mode.clone()).await;
    muted
        .into_iter()
        .map(|username| {
            let label = username.to_string();
            let match_key = label.clone();
            SearchCandidate {
                label,
                match_key,
                action: CallbackAction::Mute(MuteAction::Toggle {
                    mode: mode.clone(),
                    username,
                    page,
                }),
            }
        })
        .collect::<Vec<_>>()
}

async fn candidates_sub_mute_view(
    state: &AppState,
    sub_id: crate::core::types::TelegramId,
    sub_page: usize,
    view_page: usize,
    lang: LanguageCode,
) -> Vec<SearchCandidate> {
    let mode = tg_search_service::resolve_mute_mode(&state.db, sub_id, lang).await;
    let muted = tg_search_service::list_muted_users(&state.db, sub_id, mode.clone()).await;
    muted
        .into_iter()
        .map(|username| {
            let label = username.to_string();
            let match_key = label.clone();
            SearchCandidate {
                label,
                match_key,
                action: CallbackAction::Subscriber(SubAction::MuteView {
                    sub_id,
                    page: sub_page,
                    view_page,
                }),
            }
        })
        .collect::<Vec<_>>()
}

async fn candidates_link_list(
    state: &AppState,
    sub_id: crate::core::types::TelegramId,
    page: usize,
) -> Vec<SearchCandidate> {
    let accounts = tg_search_service::list_user_accounts(&state.state).await;
    accounts
        .into_iter()
        .map(|acc| {
            let username = TtUsername::new(acc.username);
            let label = username.to_string();
            let match_key = label.clone();
            SearchCandidate {
                label,
                match_key,
                action: CallbackAction::Subscriber(SubAction::LinkPerform {
                    sub_id,
                    page,
                    username,
                }),
            }
        })
        .collect::<Vec<_>>()
}

fn ban_entry_candidate(entry: BanEntry) -> Option<SearchCandidate> {
    let username = entry.teamtalk_username?;
    let label = username.to_string();
    let match_key = label.clone();
    Some(SearchCandidate {
        label,
        match_key,
        action: CallbackAction::Admin(AdminAction::UnbanPerform {
            ban_db_id: entry.id,
            page: 0,
        }),
    })
}
