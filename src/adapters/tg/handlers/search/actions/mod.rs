mod admin;
mod mute;
mod subscribers;

use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::telegram_id_from_user;
use crate::core::types::{LanguageCode, TelegramId};
use teloxide_ng::prelude::*;

use super::context::{SearchCandidate, SearchContext, SearchListType};

pub(super) async fn handle_single_match(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    ctx: &SearchContext,
    candidate: &SearchCandidate,
    lang: LanguageCode,
) -> ResponseResult<bool> {
    match &ctx.list_type {
        SearchListType::Kick => admin::handle_kick(bot, msg, state, candidate, lang).await,
        SearchListType::Ban => admin::handle_ban(bot, msg, state, candidate, lang).await,
        SearchListType::Unban => admin::handle_unban(bot, msg, state, candidate, lang).await,
        SearchListType::Subscribers => {
            subscribers::handle_subscribers(bot, msg, state, candidate, lang).await
        }
        SearchListType::MuteServer {
            telegram_id,
            mode,
            page,
        } => {
            mute::handle_mute(mute::HandleMuteArgs {
                bot,
                msg,
                state,
                candidate,
                telegram_id: *telegram_id,
                mode: mode.clone(),
                page: *page,
                server_list: true,
                lang,
            })
            .await
        }
        SearchListType::MuteLocal {
            telegram_id,
            mode,
            page,
        } => {
            mute::handle_mute(mute::HandleMuteArgs {
                bot,
                msg,
                state,
                candidate,
                telegram_id: *telegram_id,
                mode: mode.clone(),
                page: *page,
                server_list: false,
                lang,
            })
            .await
        }
        SearchListType::SubMuteView { .. } => Ok(false),
        SearchListType::LinkList { sub_id, page } => {
            subscribers::handle_link_list(bot, msg, state, candidate, *sub_id, *page, lang).await
        }
    }
}

fn requester_id_from_message(msg: &Message, context: &'static str) -> Option<TelegramId> {
    let user = msg.from.as_ref()?;
    telegram_id_from_user(user, context)
}
