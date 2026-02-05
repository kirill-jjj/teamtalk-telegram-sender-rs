use crate::adapters::tg::presenter::admin::bans::{edit_unban_list, send_unban_list};
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{
    answer_callback, answer_callback_empty, answer_cmd_error_callback, TgErrorReporter,
    telegram_id_from_callback_query,
};
use crate::app::services::tg_admin as tg_admin_service;
use crate::core::types::{AdminErrorContext, DbBanId, LanguageCode};
use crate::infra::locales;
use teloxide::prelude::*;

use super::lists::should_send_page;

async fn load_bans_or_reply(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    lang: LanguageCode,
) -> ResponseResult<Option<Vec<crate::infra::db::types::BanEntry>>> {
    match tg_admin_service::list_ban_entries(&state.db).await {
        Ok(entries) => Ok(Some(entries)),
        Err(err) => {
            tracing::error!(error = ?err, "Failed to load ban entries");
            answer_cmd_error_callback(bot, &q.id, lang, true).await?;
            Ok(None)
        }
    }
}

pub(super) async fn handle_unban_list(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    page: usize,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let Some(entries) = load_bans_or_reply(bot, q, state, lang).await? else {
        return Ok(());
    };
    if should_send_page(msg, page) {
        send_unban_list(
            bot,
            msg.chat.id,
            entries,
            &state.search_contexts,
            lang,
            0,
            None,
        )
        .await?;
    } else {
        edit_unban_list(bot, msg, entries, &state.search_contexts, lang, page).await?;
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
    let Some(admin_id) = telegram_id_from_callback_query(q, "handle_unban_perform") else {
        return Ok(());
    };
    let errors = TgErrorReporter::new(bot, &state.config, admin_id, lang);
    if errors
        .check_db_err(
            &q.id.0,
            tg_admin_service::remove_ban(&state.db, ban_db_id)
                .await
                .map_err(crate::app::services::tg_admin::AdminError::into_error),
            AdminErrorContext::Callback,
        )
        .await?
    {
        return Ok(());
    }
    answer_callback(
        bot,
        &q.id,
        locales::get_text(lang.as_str(), locales::LocaleKey::ToastUserUnbanned, None),
        false,
    )
    .await?;
    let Some(entries) = load_bans_or_reply(bot, q, state, lang).await? else {
        return Ok(());
    };
    edit_unban_list(bot, msg, entries, &state.search_contexts, lang, page).await
}
