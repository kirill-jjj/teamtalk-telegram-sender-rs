use crate::adapters::tg::presenter::settings::send_mute_menu;
use crate::adapters::tg::utils::check_db_err;
use crate::app::services::tg_settings as tg_settings_service;
use crate::core::types::{AdminErrorContext, LanguageCode, TelegramId};
use teloxide::prelude::*;

use super::AppState;

pub(super) async fn handle_mute_manage(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    match tg_settings_service::load_settings(&state.db, telegram_id, LanguageCode::En).await {
        Ok(u) => {
            let mode = u.mute_list_mode;
            let has_guest = state.config.teamtalk.guest_username.is_some();
            send_mute_menu(bot, msg, lang, mode, has_guest).await?;
        }
        Err(e) => {
            check_db_err(
                bot,
                &q.id.0,
                Err(e),
                &state.config,
                telegram_id,
                AdminErrorContext::Callback,
                lang,
            )
            .await?;
        }
    }
    Ok(())
}
