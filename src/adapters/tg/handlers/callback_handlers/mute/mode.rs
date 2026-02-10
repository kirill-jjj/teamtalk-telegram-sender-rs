use crate::adapters::tg::presenter::settings::send_mute_menu;
use crate::adapters::tg::utils::{TgErrorReporter, answer_callback};
use crate::app::services::tg_sub_settings as tg_sub_settings_service;
use crate::args;
use crate::core::types::{AdminErrorContext, LanguageCode, MuteListMode, TelegramId};
use crate::infra::locales;
use teloxide_ng::prelude::*;

use super::AppState;

pub(super) async fn handle_mode_set(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
    mode: MuteListMode,
) -> ResponseResult<()> {
    let errors = TgErrorReporter::new(bot, &state.config, telegram_id, lang);
    if errors
        .check_db_err(
            &q.id.0,
            tg_sub_settings_service::update_mute_mode(&state.db, telegram_id, mode.clone()).await,
            AdminErrorContext::Callback,
        )
        .await?
    {
        return Ok(());
    }
    answer_callback(
        bot,
        &q.id,
        locales::get_text(
            lang.as_str(),
            locales::LocaleKey::ToastMuteModeSet,
            args!(mode = mode.to_string()).as_ref(),
        ),
        false,
    )
    .await?;
    let has_guest = state.config.teamtalk.guest_username.is_some();
    send_mute_menu(bot, msg, lang, mode, has_guest).await
}
