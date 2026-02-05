use crate::adapters::tg::presenter::admin::subscribers::{
    edit_subscribers_list, prepare_display_list, send_subscribers_list,
};
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::answer_callback_empty;
use crate::app::services::tg_admin as tg_admin_service;
use crate::core::types::LanguageCode;
use teloxide::prelude::*;

use super::lists::should_send_page;

pub(super) async fn handle_subs_list(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    page: usize,
    lang: LanguageCode,
) -> ResponseResult<()> {
    if should_send_page(msg, page) {
        send_subscribers_list(
            bot,
            msg.chat.id,
            prepare_display_list(
                bot,
                tg_admin_service::list_subscribers(&state.db)
                    .await
                    .unwrap_or_default(),
            )
            .await,
            &state.search_contexts,
            lang,
            0,
            None,
        )
        .await?;
    } else {
        edit_subscribers_list(
            bot,
            msg,
            prepare_display_list(
                bot,
                tg_admin_service::list_subscribers(&state.db)
                    .await
                    .unwrap_or_default(),
            )
            .await,
            &state.search_contexts,
            lang,
            page,
        )
        .await?;
    }
    answer_callback_empty(bot, &q.id).await
}
