use crate::adapters::tg::presenter::keyboards::{back_button, callback_button};
use crate::adapters::tg::presenter::settings::send_main_settings_edit;
use crate::adapters::tg::utils::{TgErrorReporter, answer_callback};
use crate::app::services::tg_settings as tg_settings_service;
use crate::core::callbacks::{CallbackAction, SettingsAction};
use crate::core::types::{AdminErrorContext, LanguageCode, TelegramId};
use crate::infra::locales;
use teloxide_ng::prelude::*;
use teloxide_ng::types::InlineKeyboardMarkup;

use super::AppState;

pub(super) async fn handle_lang_select(
    bot: &Bot,
    msg: &Message,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![callback_button(
            "Русский",
            CallbackAction::Settings(SettingsAction::LangSet {
                lang: LanguageCode::Ru,
            }),
        )],
        vec![callback_button(
            "English",
            CallbackAction::Settings(SettingsAction::LangSet {
                lang: LanguageCode::En,
            }),
        )],
        vec![back_button(
            lang,
            locales::LocaleKey::BtnBackSettings,
            CallbackAction::Settings(SettingsAction::Main),
        )],
    ]);
    bot.edit_message_text(
        msg.chat.id,
        msg.id,
        locales::get_text(lang.as_str(), locales::LocaleKey::MsgChooseLang, None),
    )
    .reply_markup(keyboard)
    .await?;
    Ok(())
}

pub(super) async fn handle_lang_set(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
    new_lang: LanguageCode,
) -> ResponseResult<()> {
    let errors = TgErrorReporter::new(bot, &state.config, telegram_id, lang);
    if errors
        .check_db_err(
            &q.id.0,
            tg_settings_service::update_language(&state.db, telegram_id, new_lang).await,
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
            new_lang.as_str(),
            locales::LocaleKey::ToastLangUpdated,
            None,
        ),
        false,
    )
    .await?;
    send_main_settings_edit(bot, msg, new_lang).await
}
