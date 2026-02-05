use crate::adapters::tg::presenter::keyboards::{back_btn, callback_button};
use crate::args;
use crate::core::callbacks::{AdminAction, CallbackAction, MuteAction, SubAction};
use crate::core::types::LanguageCode;
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use super::context::{SearchCandidate, SearchContext, SearchListType};

pub(super) async fn render_search_results(
    bot: &Bot,
    msg: &Message,
    ctx: &SearchContext,
    query: &str,
    candidates: &[SearchCandidate],
    lang: LanguageCode,
) -> ResponseResult<()> {
    let title = locales::get_text_or_log(
        lang.as_str(),
        locales::LocaleKey::ListSearchTitle,
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
    let (back_text, back_act) = back_btn(lang, locales::LocaleKey::BtnBackSearch, back_action);
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
