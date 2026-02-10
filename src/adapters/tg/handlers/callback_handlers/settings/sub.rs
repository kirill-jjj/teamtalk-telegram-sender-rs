use crate::adapters::tg::presenter::settings::send_sub_settings;
use crate::adapters::tg::utils::{TgErrorReporter, answer_callback, cmd_error_text};
use crate::app::services::tg_settings as tg_settings_service;
use crate::args;
use crate::core::types::{AdminErrorContext, LanguageCode, NotificationSetting, TelegramId};
use crate::infra::locales;
use teloxide_ng::prelude::*;

use super::AppState;

pub(super) async fn handle_sub_select(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let settings = match tg_settings_service::load_settings(
        &state.db,
        telegram_id,
        state.config.general.default_lang,
    )
    .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(telegram_id = telegram_id.as_i64(), error = %e, "Failed to load settings");
            bot.edit_message_text(msg.chat.id, msg.id, cmd_error_text(lang))
                .await?;
            return Ok(());
        }
    };
    send_sub_settings(bot, msg, lang, settings.notification_settings).await
}

pub(super) async fn handle_sub_set(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
    setting: NotificationSetting,
) -> ResponseResult<()> {
    let errors = TgErrorReporter::new(bot, &state.config, telegram_id, lang);
    if errors
        .check_db_err(
            &q.id.0,
            tg_settings_service::update_notifications(&state.db, telegram_id, setting.clone())
                .await,
            AdminErrorContext::Callback,
        )
        .await?
    {
        return Ok(());
    }
    let text_key = match setting {
        NotificationSetting::All => locales::LocaleKey::BtnSubAll,
        NotificationSetting::JoinOff => locales::LocaleKey::BtnSubLeave,
        NotificationSetting::LeaveOff => locales::LocaleKey::BtnSubJoin,
        NotificationSetting::None => locales::LocaleKey::BtnSubNone,
    };
    let setting_text = locales::get_text(lang.as_str(), text_key, args!(marker = "").as_ref());
    answer_callback(
        bot,
        &q.id,
        locales::get_text(
            lang.as_str(),
            locales::LocaleKey::RespSubUpdated,
            args!(text = setting_text).as_ref(),
        ),
        false,
    )
    .await?;
    send_sub_settings(bot, msg, lang, setting).await
}
