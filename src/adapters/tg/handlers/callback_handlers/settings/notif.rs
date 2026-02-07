use crate::adapters::tg::presenter::settings::send_notif_settings;
use crate::adapters::tg::utils::{TgErrorReporter, answer_callback, cmd_error_text};
use crate::app::services::tg_admin as tg_admin_service;
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
            bot.edit_message_text(msg.chat.id, msg.id, cmd_error_text(lang))
                .await?;
            return Ok(());
        }
    };
    let is_admin = tg_admin_service::is_admin(&state.db, &state.config, telegram_id).await;
    let admin_sub_events_enabled = if is_admin {
        match tg_settings_service::admin_sub_events_enabled(&state.db, telegram_id).await {
            Ok(enabled) => Some(enabled),
            Err(e) => {
                tracing::error!(telegram_id = telegram_id.as_i64(), error = %e, "Failed to load admin subscription events setting");
                None
            }
        }
    } else {
        None
    };
    send_notif_settings(
        bot,
        msg,
        lang,
        settings.not_on_online_enabled,
        admin_sub_events_enabled,
    )
    .await
}

pub(super) async fn handle_noon_toggle(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let errors = TgErrorReporter::new(bot, &state.config, telegram_id, lang);
    let user_settings =
        match tg_settings_service::load_settings(&state.db, telegram_id, LanguageCode::En).await {
            Ok(u) => u,
            Err(e) => {
                errors
                    .check_db_err(&q.id.0, Err(e), AdminErrorContext::Callback)
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
                        errors
                            .check_db_err(&q.id.0, Err(e), AdminErrorContext::Callback)
                            .await?;
                        return Ok(());
                    }
                };
            let is_admin = tg_admin_service::is_admin(&state.db, &state.config, telegram_id).await;
            let admin_sub_events_enabled = if is_admin {
                match tg_settings_service::admin_sub_events_enabled(&state.db, telegram_id).await {
                    Ok(enabled) => Some(enabled),
                    Err(e) => {
                        tracing::error!(telegram_id = telegram_id.as_i64(), error = %e, "Failed to load admin subscription events setting");
                        None
                    }
                }
            } else {
                None
            };
            send_notif_settings(
                bot,
                msg,
                lang,
                settings.not_on_online_enabled,
                admin_sub_events_enabled,
            )
            .await?;
        }
        Err(e) => {
            errors
                .check_db_err(&q.id.0, Err(e), AdminErrorContext::Callback)
                .await?;
        }
    }
    Ok(())
}

pub(super) async fn handle_admin_sub_events_toggle(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    if !tg_admin_service::is_admin(&state.db, &state.config, telegram_id).await {
        answer_callback(
            bot,
            &q.id,
            locales::get_text(lang.as_str(), locales::LocaleKey::CmdUnauth, None),
            true,
        )
        .await?;
        return Ok(());
    }

    let errors = TgErrorReporter::new(bot, &state.config, telegram_id, lang);
    let current = match tg_settings_service::admin_sub_events_enabled(&state.db, telegram_id).await
    {
        Ok(value) => value,
        Err(e) => {
            errors
                .check_db_err(&q.id.0, Err(e), AdminErrorContext::Callback)
                .await?;
            return Ok(());
        }
    };
    let new_value = !current;
    if let Err(e) =
        tg_settings_service::set_admin_sub_events_enabled(&state.db, telegram_id, new_value).await
    {
        errors
            .check_db_err(&q.id.0, Err(e), AdminErrorContext::Callback)
            .await?;
        return Ok(());
    }

    let status = if new_value {
        locales::get_text(lang.as_str(), locales::LocaleKey::StatusEnabled, None)
    } else {
        locales::get_text(lang.as_str(), locales::LocaleKey::StatusDisabled, None)
    };
    answer_callback(
        bot,
        &q.id,
        locales::get_text(
            lang.as_str(),
            locales::LocaleKey::RespAdminSubEventsUpdated,
            args!(status = status).as_ref(),
        ),
        false,
    )
    .await?;

    let settings =
        match tg_settings_service::load_settings(&state.db, telegram_id, LanguageCode::En).await {
            Ok(s) => s,
            Err(e) => {
                errors
                    .check_db_err(&q.id.0, Err(e), AdminErrorContext::Callback)
                    .await?;
                return Ok(());
            }
        };
    send_notif_settings(
        bot,
        msg,
        lang,
        settings.not_on_online_enabled,
        Some(new_value),
    )
    .await?;
    Ok(())
}
