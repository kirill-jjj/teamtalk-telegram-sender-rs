use crate::core::types::{LanguageCode, MuteListMode, TelegramId, TtUsername};
use std::collections::HashMap;
use std::sync::Arc;
use teloxide::prelude::ChatId;
use teloxide::types::MessageId;
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
        telegram_id: TelegramId,
        mode: MuteListMode,
        page: usize,
    },
    MuteLocal {
        telegram_id: TelegramId,
        mode: MuteListMode,
        page: usize,
    },
    SubMuteView {
        sub_id: TelegramId,
        sub_page: usize,
        view_page: usize,
    },
    LinkList {
        sub_id: TelegramId,
        page: usize,
    },
}

#[derive(Debug, Clone)]
pub(super) struct SearchCandidate {
    pub label: String,
    pub match_key: String,
    pub action: crate::core::callbacks::CallbackAction,
}

pub fn new_search_contexts() -> Arc<Mutex<HashMap<ChatId, SearchContext>>> {
    Arc::new(Mutex::new(HashMap::new()))
}

pub async fn set_search_context(
    state: &crate::adapters::tg::state::AppState,
    chat_id: ChatId,
    ctx: SearchContext,
) {
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
    let hint = crate::infra::locales::get_text(
        lang.as_str(),
        crate::infra::locales::LocaleKey::ListSearchHint,
        None,
    );
    format!("{text}\n\n{hint}")
}

pub(super) fn format_display_subscriber(
    display_name: &str,
    tt_username: Option<&TtUsername>,
) -> String {
    let mut parts = vec![display_name.to_string()];
    if let Some(tt) = tt_username {
        parts.push(format!("TT: {tt}"));
    }
    parts.join(", ")
}
