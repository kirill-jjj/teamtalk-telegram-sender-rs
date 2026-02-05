use crate::adapters::tg::state::AppState;
use crate::args;
use crate::core::types::LanguageCode;
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;

mod actions;
mod context;
mod matching;
mod render;

pub use context::{
    SearchContext, SearchListType, append_search_hint, new_search_contexts, set_search_context,
    set_search_context_raw,
};

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
    let matches = matching::find_matches(state, &ctx.list_type, bot, query, lang).await?;

    if matches.is_empty() {
        let args = args!(query = query.to_string());
        let text = locales::get_text(
            lang.as_str(),
            locales::LocaleKey::ListSearchEmpty,
            args.as_ref(),
        );
        bot.send_message(msg.chat.id, text).reply_to(msg.id).await?;
        return Ok(true);
    }

    if matches.len() == 1 {
        let candidate = &matches[0];
        if actions::handle_single_match(bot, msg, state, &ctx, candidate, lang).await? {
            return Ok(true);
        }
    }

    render::render_search_results(bot, msg, &ctx, query, &matches, lang).await?;
    Ok(true)
}
