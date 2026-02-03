use crate::adapters::tg::admin_logic::bans as bans_logic;
use crate::adapters::tg::admin_logic::subscriber_settings as subscriber_settings_logic;
use crate::adapters::tg::admin_logic::subscribers as subscribers_logic;
use crate::adapters::tg::keyboards::{back_btn, callback_button};
use crate::adapters::tg::settings_logic::{RenderMuteListArgs, RenderMuteListStringsArgs};
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{notify_admin_error, send_text_key};
use crate::app::services::subscriber_actions as subscriber_actions_service;
use crate::app::services::user_settings as user_settings_service;
use crate::args;
use crate::core::callbacks::{AdminAction, CallbackAction, MuteAction, SubAction};
use crate::core::types::{
    AdminErrorContext, LanguageCode, LiteUser, MuteListMode, TtCommand, TtUsername,
};
use crate::infra::db::types::BanEntry;
use crate::infra::locales;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use teamtalk::types::UserAccount;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, MessageId};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct SearchContext {
    pub message_id: MessageId,
    pub list_type: SearchListType,
}

#[derive(Debug, Clone)]
pub enum SearchListType {
    Kick,
    Ban,
    Unban,
    Subscribers,
    MuteServer {
        telegram_id: i64,
        mode: MuteListMode,
        page: usize,
    },
    MuteLocal {
        telegram_id: i64,
        mode: MuteListMode,
        page: usize,
    },
    SubMuteView {
        sub_id: i64,
        sub_page: usize,
        view_page: usize,
    },
    LinkList {
        sub_id: i64,
        page: usize,
    },
}

#[derive(Debug, Clone)]
struct SearchCandidate {
    label: String,
    match_key: String,
    action: CallbackAction,
}

pub fn new_search_contexts() -> Arc<Mutex<HashMap<ChatId, SearchContext>>> {
    Arc::new(Mutex::new(HashMap::new()))
}

pub async fn set_search_context(state: &AppState, chat_id: ChatId, ctx: SearchContext) {
    let mut map = state.search_contexts.lock().await;
    map.insert(chat_id, ctx);
}

pub async fn set_search_context_raw(
    search_contexts: &Arc<Mutex<HashMap<ChatId, SearchContext>>>,
    chat_id: ChatId,
    ctx: SearchContext,
) {
    let mut map = search_contexts.lock().await;
    map.insert(chat_id, ctx);
}

pub fn append_search_hint(text: &str, lang: LanguageCode) -> String {
    let hint = locales::get_text(lang.as_str(), "list-search-hint", None);
    format!("{text}\n\n{hint}")
}

pub async fn maybe_handle_search_message(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    lang: LanguageCode,
) -> ResponseResult<bool> {
    let Some(text) = msg.text() else {
        return Ok(false);
    };
    if text.trim().is_empty() {
        return Ok(false);
    }
    if text.trim_start().starts_with('/') {
        return Ok(false);
    }
    if msg.reply_to_message().is_some() {
        return Ok(false);
    }

    let ctx = {
        let map = state.search_contexts.lock().await;
        map.get(&msg.chat.id).cloned()
    };
    let Some(ctx) = ctx else {
        return Ok(false);
    };

    let query = text.trim();
    let matches = find_matches(state, &ctx.list_type, bot, query, lang).await?;

    if matches.is_empty() {
        let args = args!(query = query.to_string());
        let text = locales::get_text(lang.as_str(), "list-search-empty", args.as_ref());
        bot.send_message(msg.chat.id, text).reply_to(msg.id).await?;
        return Ok(true);
    }

    if matches.len() == 1 {
        let candidate = &matches[0];
        if handle_single_match(bot, msg, state, &ctx, candidate, lang).await? {
            return Ok(true);
        }
    }

    render_search_results(bot, msg, &ctx, query, &matches, lang).await?;
    Ok(true)
}

#[allow(clippy::too_many_lines)]
async fn find_matches(
    state: &AppState,
    list_type: &SearchListType,
    bot: &Bot,
    query: &str,
    lang: LanguageCode,
) -> ResponseResult<Vec<SearchCandidate>> {
    let normalized_query = query.to_lowercase();
    let mut candidates = match list_type {
        SearchListType::Kick => {
            let users = sorted_online_users(&state.online_users);
            users
                .into_iter()
                .map(|u| SearchCandidate {
                    label: u.nickname.clone(),
                    match_key: format!("{} {}", u.nickname, u.username),
                    action: CallbackAction::Admin(AdminAction::KickPerform { user_id: u.id }),
                })
                .collect()
        }
        SearchListType::Ban => {
            let users = sorted_online_users(&state.online_users);
            users
                .into_iter()
                .map(|u| SearchCandidate {
                    label: u.nickname.clone(),
                    match_key: format!("{} {}", u.nickname, u.username),
                    action: CallbackAction::Admin(AdminAction::BanPerform { user_id: u.id }),
                })
                .collect()
        }
        SearchListType::Unban => {
            let entries = state.db.get_banned_users().await.unwrap_or_default();
            entries
                .into_iter()
                .filter_map(ban_entry_candidate)
                .collect::<Vec<_>>()
        }
        SearchListType::Subscribers => {
            let subs = state.db.get_subscribers().await.unwrap_or_default();
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
                .collect()
        }
        SearchListType::MuteServer { mode, page, .. } => {
            let accounts = load_accounts(&state.user_accounts);
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
                            page: *page,
                        }),
                    }
                })
                .collect()
        }
        SearchListType::MuteLocal {
            telegram_id,
            mode,
            page,
        } => {
            let muted = state
                .db
                .get_muted_users_list(*telegram_id, mode.clone())
                .await
                .unwrap_or_default();
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
                            page: *page,
                        }),
                    }
                })
                .collect()
        }
        SearchListType::SubMuteView {
            sub_id,
            sub_page,
            view_page,
        } => {
            let settings = user_settings_service::get_or_create(&state.db, *sub_id, lang)
                .await
                .ok();
            let mode = settings
                .as_ref()
                .map_or(MuteListMode::Blacklist, |s| s.mute_list_mode.clone());
            let muted = state
                .db
                .get_muted_users_list(*sub_id, mode)
                .await
                .unwrap_or_default();
            muted
                .into_iter()
                .map(|username| {
                    let label = username.to_string();
                    let match_key = label.clone();
                    SearchCandidate {
                        label,
                        match_key,
                        action: CallbackAction::Subscriber(SubAction::MuteView {
                            sub_id: *sub_id,
                            page: *sub_page,
                            view_page: *view_page,
                        }),
                    }
                })
                .collect()
        }
        SearchListType::LinkList { sub_id, page } => {
            let accounts = load_accounts(&state.user_accounts);
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
                            sub_id: *sub_id,
                            page: *page,
                            username,
                        }),
                    }
                })
                .collect()
        }
    };

    candidates.retain(|c| c.match_key.to_lowercase().contains(&normalized_query));
    candidates.sort_by(|a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase()));
    Ok(candidates)
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

fn format_display_subscriber(display_name: &str, tt_username: Option<&TtUsername>) -> String {
    let mut parts = vec![display_name.to_string()];
    if let Some(tt) = tt_username {
        parts.push(format!("TT: {tt}"));
    }
    parts.join(", ")
}

#[allow(clippy::too_many_lines)]
async fn handle_single_match(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    ctx: &SearchContext,
    candidate: &SearchCandidate,
    lang: LanguageCode,
) -> ResponseResult<bool> {
    match &ctx.list_type {
        SearchListType::Kick => {
            let CallbackAction::Admin(AdminAction::KickPerform { user_id }) = &candidate.action
            else {
                return Ok(false);
            };
            send_tt_command(
                bot,
                msg,
                state,
                TtCommand::KickUser { user_id: *user_id },
                lang,
            )
            .await?;
            Ok(true)
        }
        SearchListType::Ban => {
            let CallbackAction::Admin(AdminAction::BanPerform { user_id }) = &candidate.action
            else {
                return Ok(false);
            };
            send_tt_command(
                bot,
                msg,
                state,
                TtCommand::BanUser { user_id: *user_id },
                lang,
            )
            .await?;
            Ok(true)
        }
        SearchListType::Unban => {
            let CallbackAction::Admin(AdminAction::UnbanPerform { ban_db_id, page }) =
                &candidate.action
            else {
                return Ok(false);
            };
            if let Err(e) = state.db.remove_ban_by_id(*ban_db_id).await {
                notify_admin_error(
                    bot,
                    &state.config,
                    tg_user_id_i64(msg.from.as_ref().map_or(0, |u| u.id.0)),
                    AdminErrorContext::Callback,
                    &e.to_string(),
                    lang,
                )
                .await;
                return Ok(true);
            }
            send_text_key(bot, msg.chat.id, lang, "toast-user-unbanned", Some(msg.id)).await?;
            bans_logic::edit_unban_list(bot, msg, &state.db, &state.search_contexts, lang, *page)
                .await?;
            Ok(true)
        }
        SearchListType::Subscribers => {
            let CallbackAction::Subscriber(SubAction::Details { sub_id, page }) = &candidate.action
            else {
                return Ok(false);
            };
            subscribers_logic::send_subscriber_details(bot, msg, &state.db, lang, *sub_id, *page)
                .await?;
            Ok(true)
        }
        SearchListType::MuteServer {
            telegram_id,
            mode,
            page,
        } => {
            let CallbackAction::Mute(MuteAction::ServerToggle { username, .. }) = &candidate.action
            else {
                return Ok(false);
            };
            toggle_mute_and_render(
                bot,
                msg,
                state,
                ToggleMuteArgs {
                    telegram_id: *telegram_id,
                    mode: mode.clone(),
                    username: username.clone(),
                    page: *page,
                    server_list: true,
                    lang,
                },
            )
            .await?;
            Ok(true)
        }
        SearchListType::MuteLocal {
            telegram_id,
            mode,
            page,
        } => {
            let CallbackAction::Mute(MuteAction::Toggle { username, .. }) = &candidate.action
            else {
                return Ok(false);
            };
            toggle_mute_and_render(
                bot,
                msg,
                state,
                ToggleMuteArgs {
                    telegram_id: *telegram_id,
                    mode: mode.clone(),
                    username: username.clone(),
                    page: *page,
                    server_list: false,
                    lang,
                },
            )
            .await?;
            Ok(true)
        }
        SearchListType::SubMuteView { .. } => Ok(false),
        SearchListType::LinkList { sub_id, page } => {
            let CallbackAction::Subscriber(SubAction::LinkPerform { username, .. }) =
                &candidate.action
            else {
                return Ok(false);
            };
            if let Err(e) = subscriber_actions_service::link_tt(&state.db, *sub_id, username).await
            {
                notify_admin_error(
                    bot,
                    &state.config,
                    tg_user_id_i64(msg.from.as_ref().map_or(0, |u| u.id.0)),
                    AdminErrorContext::Callback,
                    &e.to_string(),
                    lang,
                )
                .await;
                return Ok(true);
            }
            let args = args!(user = username.to_string());
            let text = locales::get_text(lang.as_str(), "toast-account-linked", args.as_ref());
            bot.send_message(msg.chat.id, text).reply_to(msg.id).await?;
            subscriber_settings_logic::send_sub_manage_tt_menu(
                bot, msg, &state.db, lang, *sub_id, *page,
            )
            .await?;
            Ok(true)
        }
    }
}

struct ToggleMuteArgs {
    telegram_id: i64,
    mode: MuteListMode,
    username: TtUsername,
    page: usize,
    server_list: bool,
    lang: LanguageCode,
}

async fn toggle_mute_and_render(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    args: ToggleMuteArgs,
) -> ResponseResult<()> {
    if let Err(e) = state
        .db
        .toggle_muted_user(args.telegram_id, args.mode.clone(), &args.username)
        .await
    {
        notify_admin_error(
            bot,
            &state.config,
            args.telegram_id,
            AdminErrorContext::Callback,
            &e.to_string(),
            args.lang,
        )
        .await;
        return Ok(());
    }

    let fmt_args = args!(user = args.username.to_string(), action = "toggled");
    let text = locales::get_text(args.lang.as_str(), "toast-user-muted", fmt_args.as_ref());
    bot.send_message(msg.chat.id, text).reply_to(msg.id).await?;

    if args.server_list {
        let accounts = load_accounts(&state.user_accounts);
        let guest_username = state
            .config
            .teamtalk
            .guest_username
            .as_ref()
            .map(TtUsername::as_str);
        let render_args = RenderMuteListArgs {
            bot,
            msg,
            db: &state.db,
            telegram_id: args.telegram_id,
            lang: args.lang,
            accounts: &accounts,
            page: args.page,
            title_key: "list-all-accs-title",
            guest_username,
            mode: args.mode,
        };
        crate::adapters::tg::settings_logic::render_mute_list(render_args).await?;
    } else {
        let muted = state
            .db
            .get_muted_users_list(args.telegram_id, args.mode.clone())
            .await
            .unwrap_or_default();
        let guest_username = state
            .config
            .teamtalk
            .guest_username
            .as_ref()
            .map(TtUsername::as_str);
        let render_args = RenderMuteListStringsArgs {
            bot,
            msg,
            lang: args.lang,
            items: &muted,
            page: args.page,
            title_key: "list-mute-title",
            guest_username,
            mode: args.mode,
        };
        crate::adapters::tg::settings_logic::render_mute_list_strings(render_args).await?;
    }
    Ok(())
}

async fn send_tt_command(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    cmd: TtCommand,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let admin_id = tg_user_id_i64(msg.from.as_ref().map_or(0, |u| u.id.0));
    if let Err(e) = state.tx_tt.send(cmd).await {
        notify_admin_error(
            bot,
            &state.config,
            admin_id,
            AdminErrorContext::TtCommand,
            &e.to_string(),
            lang,
        )
        .await;
    }
    send_text_key(bot, msg.chat.id, lang, "toast-command-sent", Some(msg.id)).await?;
    Ok(())
}

async fn render_search_results(
    bot: &Bot,
    msg: &Message,
    ctx: &SearchContext,
    query: &str,
    candidates: &[SearchCandidate],
    lang: LanguageCode,
) -> ResponseResult<()> {
    let title = locales::get_text(
        lang.as_str(),
        "list-search-title",
        args!(query = query.to_string()).as_ref(),
    );
    let back_action = back_action(&ctx.list_type);
    let keyboard = search_results_keyboard(candidates, back_action, lang);
    bot.edit_message_text(msg.chat.id, ctx.message_id, title)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

fn search_results_keyboard(
    candidates: &[SearchCandidate],
    back_action: CallbackAction,
    lang: LanguageCode,
) -> InlineKeyboardMarkup {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    for candidate in candidates.iter().take(10) {
        rows.push(vec![callback_button(
            candidate.label.clone(),
            candidate.action.clone(),
        )]);
    }
    let (back_text, back_act) = back_btn(lang, "btn-back-search", back_action);
    rows.push(vec![callback_button(back_text, back_act)]);
    InlineKeyboardMarkup::new(rows)
}

fn back_action(list_type: &SearchListType) -> CallbackAction {
    match list_type {
        SearchListType::Kick => CallbackAction::Admin(AdminAction::KickList { page: 0 }),
        SearchListType::Ban => CallbackAction::Admin(AdminAction::BanList { page: 0 }),
        SearchListType::Unban => CallbackAction::Admin(AdminAction::UnbanList { page: 0 }),
        SearchListType::Subscribers => CallbackAction::Admin(AdminAction::SubsList { page: 0 }),
        SearchListType::MuteServer { mode, .. } => CallbackAction::Mute(MuteAction::ServerList {
            mode: mode.clone(),
            page: 0,
        }),
        SearchListType::MuteLocal { mode, .. } => CallbackAction::Mute(MuteAction::List {
            mode: mode.clone(),
            page: 0,
        }),
        SearchListType::SubMuteView {
            sub_id, sub_page, ..
        } => CallbackAction::Subscriber(SubAction::MuteView {
            sub_id: *sub_id,
            page: *sub_page,
            view_page: 0,
        }),
        SearchListType::LinkList { sub_id, page } => {
            CallbackAction::Subscriber(SubAction::LinkList {
                sub_id: *sub_id,
                page: *page,
                list_page: 0,
            })
        }
    }
}

fn sorted_online_users(online_users: &RwLock<HashMap<i32, LiteUser>>) -> Vec<LiteUser> {
    let mut users: Vec<LiteUser> = online_users
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .values()
        .cloned()
        .collect();
    users.sort_by(|a, b| a.nickname.to_lowercase().cmp(&b.nickname.to_lowercase()));
    users
}

fn load_accounts(
    user_accounts: &Arc<RwLock<HashMap<TtUsername, UserAccount>>>,
) -> Vec<UserAccount> {
    let mut accounts: Vec<UserAccount> = user_accounts
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .values()
        .cloned()
        .collect();
    accounts.sort_by(|a, b| a.username.to_lowercase().cmp(&b.username.to_lowercase()));
    accounts
}

fn tg_user_id_i64(user_id: u64) -> i64 {
    i64::try_from(user_id).unwrap_or(i64::MAX)
}
