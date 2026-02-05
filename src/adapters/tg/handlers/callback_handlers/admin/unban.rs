use crate::adapters::tg::presenter::admin::bans::{edit_unban_list, send_unban_list};
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{answer_callback, answer_callback_empty, check_db_err};
use crate::app::services::tg_admin as tg_admin_service;
use crate::core::types::{AdminErrorContext, DbBanId, LanguageCode, TelegramId};
use crate::infra::locales;
use teloxide::prelude::*;

use super::lists::should_send_page;

pub(super) async fn handle_unban_list(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    page: usize,
    lang: LanguageCode,
) -> ResponseResult<()> {
    if should_send_page(msg, page) {
        send_unban_list(
            bot,
            msg.chat.id,
            tg_admin_service::list_ban_entries(&state.db)
                .await
                .unwrap_or_default(),
            &state.search_contexts,
            lang,
            0,
            None,
        )
        .await?;
    } else {
        edit_unban_list(
            bot,
            msg,
            tg_admin_service::list_ban_entries(&state.db)
                .await
                .unwrap_or_default(),
            &state.search_contexts,
            lang,
            page,
        )
        .await?;
    }
    answer_callback_empty(bot, &q.id).await
}

pub(super) async fn handle_unban_perform(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    ban_db_id: DbBanId,
    page: usize,
    lang: LanguageCode,
) -> ResponseResult<()> {
    if check_db_err(
        bot,
        &q.id.0,
        tg_admin_service::remove_ban(&state.db, ban_db_id)
            .await
            .map_err(crate::app::services::tg_admin::AdminError::into_error),
        &state.config,
        tg_user_id_i64(q.from.id.0),
        AdminErrorContext::Callback,
        lang,
    )
    .await?
    {
        return Ok(());
    }
    answer_callback(
        bot,
        &q.id,
        locales::get_text_or_log(lang.as_str(), locales::LocaleKey::ToastUserUnbanned, None),
        false,
    )
    .await?;
    edit_unban_list(
        bot,
        msg,
        tg_admin_service::list_ban_entries(&state.db)
            .await
            .unwrap_or_default(),
        &state.search_contexts,
        lang,
        page,
    )
    .await
}

fn tg_user_id_i64(user_id: u64) -> TelegramId {
    TelegramId::from(i64::try_from(user_id).unwrap_or(i64::MAX))
}
