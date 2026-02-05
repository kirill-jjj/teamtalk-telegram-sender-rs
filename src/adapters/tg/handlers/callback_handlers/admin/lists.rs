use crate::adapters::tg::handlers::search::{
    SearchContext, SearchListType, append_search_hint, set_search_context,
};
use crate::adapters::tg::presenter::keyboards::create_user_list_keyboard;
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{answer_callback, answer_callback_empty};
use crate::app::services::tg_admin as tg_admin_service;
use crate::args;
use crate::core::callbacks::{AdminAction, CallbackAction};
use crate::core::types::LanguageCode;
use crate::infra::locales;
use teloxide::prelude::*;

pub(super) async fn handle_kick_list(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    page: usize,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let users = match tg_admin_service::list_online_users(&state.state).await {
        Ok(users) => users,
        Err(err) => {
            tracing::error!(error = ?err, "Failed to list online users for kick list");
            answer_callback(
                bot,
                &q.id,
                locales::get_text(lang.as_str(), locales::LocaleKey::CmdError, None),
                true,
            )
            .await?;
            return Ok(());
        }
    };
    let args = args!(server = state.config.teamtalk.display_name().to_string());
    let base = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::ListKickTitle,
        args.as_ref(),
    );
    let title = append_search_hint(&base, lang);
    let keyboard = create_user_list_keyboard(
        &users,
        page,
        |u| {
            (
                u.nickname.as_str().to_string(),
                CallbackAction::Admin(AdminAction::KickPerform { user_id: u.id }),
            )
        },
        |p| CallbackAction::Admin(AdminAction::KickList { page: p }),
        None,
        lang,
    );
    let message_id = send_or_edit_list(bot, msg, page, title, keyboard).await?;
    set_search_context(
        state,
        msg.chat.id,
        SearchContext {
            message_id,
            list_type: SearchListType::Kick,
        },
    )
    .await;
    answer_callback_empty(bot, &q.id).await
}

pub(super) async fn handle_ban_list(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    page: usize,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let users = match tg_admin_service::list_online_users(&state.state).await {
        Ok(users) => users,
        Err(err) => {
            tracing::error!(error = ?err, "Failed to list online users for ban list");
            answer_callback(
                bot,
                &q.id,
                locales::get_text(lang.as_str(), locales::LocaleKey::CmdError, None),
                true,
            )
            .await?;
            return Ok(());
        }
    };
    let args = args!(server = state.config.teamtalk.display_name().to_string());
    let base = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::ListBanTitle,
        args.as_ref(),
    );
    let title = append_search_hint(&base, lang);
    let keyboard = create_user_list_keyboard(
        &users,
        page,
        |u| {
            (
                u.nickname.as_str().to_string(),
                CallbackAction::Admin(AdminAction::BanPerform { user_id: u.id }),
            )
        },
        |p| CallbackAction::Admin(AdminAction::BanList { page: p }),
        None,
        lang,
    );
    let message_id = send_or_edit_list(bot, msg, page, title, keyboard).await?;
    set_search_context(
        state,
        msg.chat.id,
        SearchContext {
            message_id,
            list_type: SearchListType::Ban,
        },
    )
    .await;
    answer_callback_empty(bot, &q.id).await
}

async fn send_or_edit_list(
    bot: &Bot,
    msg: &Message,
    page: usize,
    title: String,
    keyboard: teloxide::types::InlineKeyboardMarkup,
) -> ResponseResult<teloxide::types::MessageId> {
    if should_send_page(msg, page) {
        let sent = bot
            .send_message(msg.chat.id, title)
            .reply_markup(keyboard)
            .await?;
        return Ok(sent.id);
    }
    bot.edit_message_text(msg.chat.id, msg.id, title)
        .reply_markup(keyboard)
        .await?;
    Ok(msg.id)
}

pub(super) fn should_send_page(msg: &Message, page: usize) -> bool {
    page == 0 && !msg.text().unwrap_or("").contains("Page")
}
