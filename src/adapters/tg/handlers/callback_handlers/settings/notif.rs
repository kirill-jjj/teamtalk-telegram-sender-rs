use crate::adapters::tg::presenter::settings::send_notif_settings;
use crate::adapters::tg::utils::{answer_callback, check_db_err, cmd_error_text};
use crate::app::services::tg_settings as tg_settings_service;
use crate::args;
use crate::core::types::{AdminErrorContext, LanguageCode, TelegramId};
use crate::infra::locales;
use teloxide::prelude::*;

use super::AppState;

pub(super) async fn handle_notif_select(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let settings = match tg_settings_service::load_settings(
        &state.db,
        telegram_id,
        LanguageCode::En,
    )
    .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(telegram_id = telegram_id.as_i64(), error = %e, "Failed to load settings");
            bot.edit_message_text(
                msg.chat.id,
                msg.id,
                cmd_error_text(lang),
            )
            .await?;
            return Ok(());
        }
    };
    send_notif_settings(bot, msg, lang, settings.not_on_online_enabled).await
}

pub(super) async fn handle_noon_toggle(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let user_settings =
        match tg_settings_service::load_settings(&state.db, telegram_id, LanguageCode::En).await {
            Ok(u) => u,
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
                return Ok(());
            }
        };

    if user_settings.teamtalk_username.is_none() {
        answer_callback(
            bot,
            &q.id,
            locales::get_text(lang.as_str(), locales::LocaleKey::CmdFailNoonGuest, None),
            true,
        )
        .await?;
        return Ok(());
    }

    match tg_settings_service::toggle_noon(&state.db, telegram_id).await {
        Ok(new_val) => {
            let status = if new_val {
                locales::get_text(lang.as_str(), locales::LocaleKey::StatusEnabled, None)
            } else {
                locales::get_text(lang.as_str(), locales::LocaleKey::StatusDisabled, None)
            };
            if let Err(e) = answer_callback(
                bot,
                &q.id,
                locales::get_text(
                    lang.as_str(),
                    locales::LocaleKey::RespNoonUpdated,
                    args!(status = status).as_ref(),
                ),
                false,
            )
            .await
            {
                tracing::error!(error = %e, "Failed to send noon update callback");
            }

            let settings =
                match tg_settings_service::load_settings(&state.db, telegram_id, LanguageCode::En)
                    .await
                {
                    Ok(s) => s,
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
                        return Ok(());
                    }
                };
            send_notif_settings(bot, msg, lang, settings.not_on_online_enabled).await?;
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
