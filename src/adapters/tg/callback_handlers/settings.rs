use crate::adapters::tg::keyboards::{back_button, callback_button};
use crate::adapters::tg::settings_logic::{
    send_main_settings_edit, send_mute_menu, send_notif_settings, send_sub_settings,
};
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{answer_callback, check_db_err};
use crate::app::services::user_settings as user_settings_service;
use crate::args;
use crate::core::callbacks::{CallbackAction, SettingsAction};
use crate::core::types::{AdminErrorContext, LanguageCode, NotificationSetting};
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardMarkup;

pub async fn handle_settings(
    bot: Bot,
    q: CallbackQuery,
    state: AppState,
    action: SettingsAction,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let msg = match q.message {
        Some(teloxide::types::MaybeInaccessibleMessage::Regular(m)) => m,
        _ => return Ok(()),
    };
    let chat_id = msg.chat.id;
    let telegram_id = q.from.id.0 as i64;
    let db = &state.db;
    let config = &state.config;

    match action {
        SettingsAction::Main => {
            send_main_settings_edit(&bot, &msg, lang).await?;
        }
        SettingsAction::LangSelect => {
            let keyboard = InlineKeyboardMarkup::new(vec![
                vec![callback_button(
                    "🇷🇺 Русский",
                    CallbackAction::Settings(SettingsAction::LangSet {
                        lang: LanguageCode::Ru,
                    })
                    .to_string(),
                )],
                vec![callback_button(
                    "🇬🇧 English",
                    CallbackAction::Settings(SettingsAction::LangSet {
                        lang: LanguageCode::En,
                    })
                    .to_string(),
                )],
                vec![back_button(
                    lang,
                    "btn-back-settings",
                    CallbackAction::Settings(SettingsAction::Main),
                )],
            ]);
            bot.edit_message_text(
                chat_id,
                msg.id,
                locales::get_text(lang.as_str(), "msg-choose-lang", None),
            )
            .reply_markup(keyboard)
            .await?;
        }
        SettingsAction::LangSet { lang: new_lang } => {
            if check_db_err(
                &bot,
                &q.id.0,
                db.update_language(telegram_id, new_lang).await,
                config,
                telegram_id,
                AdminErrorContext::Callback,
                lang,
            )
            .await?
            {
                return Ok(());
            }
            answer_callback(
                &bot,
                &q.id,
                locales::get_text(new_lang.as_str(), "toast-lang-updated", None),
                false,
            )
            .await?;
            send_main_settings_edit(&bot, &msg, new_lang).await?;
        }
        SettingsAction::SubSelect => {
            send_sub_settings(&bot, &msg, db, telegram_id, lang).await?;
        }
        SettingsAction::SubSet { setting } => {
            let res = db
                .update_notification_setting(telegram_id, setting.clone())
                .await;
            if check_db_err(
                &bot,
                &q.id.0,
                res,
                config,
                telegram_id,
                AdminErrorContext::Callback,
                lang,
            )
            .await?
            {
                return Ok(());
            }

            let text_key = match setting {
                NotificationSetting::All => "btn-sub-all",
                NotificationSetting::JoinOff => "btn-sub-join",
                NotificationSetting::LeaveOff => "btn-sub-leave",
                NotificationSetting::None => "btn-sub-none",
            };
            let setting_text =
                locales::get_text(lang.as_str(), text_key, args!(marker = "").as_ref());
            answer_callback(
                &bot,
                &q.id,
                locales::get_text(
                    lang.as_str(),
                    "resp-sub-updated",
                    args!(text = setting_text).as_ref(),
                ),
                false,
            )
            .await?;
            send_sub_settings(&bot, &msg, db, telegram_id, lang).await?;
        }
        SettingsAction::NotifSelect => {
            send_notif_settings(&bot, &msg, db, telegram_id, lang).await?;
        }
        SettingsAction::NoonToggle => {
            let user_settings =
                match user_settings_service::get_or_create(db, telegram_id, LanguageCode::En).await
                {
                    Ok(u) => u,
                    Err(e) => {
                        check_db_err(
                            &bot,
                            &q.id.0,
                            Err(e),
                            config,
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
                    &bot,
                    &q.id,
                    locales::get_text(lang.as_str(), "cmd-fail-noon-guest", None),
                    true,
                )
                .await?;
                return Ok(());
            }

            match db.toggle_noon(telegram_id).await {
                Ok(new_val) => {
                    let status = if new_val {
                        locales::get_text(lang.as_str(), "status-enabled", None)
                    } else {
                        locales::get_text(lang.as_str(), "status-disabled", None)
                    };

                    if let Err(e) = answer_callback(
                        &bot,
                        &q.id,
                        locales::get_text(
                            lang.as_str(),
                            "resp-noon-updated",
                            args!(status = status).as_ref(),
                        ),
                        false,
                    )
                    .await
                    {
                        tracing::error!("Failed to send noon update callback: {}", e);
                    }

                    if let Err(e) = send_notif_settings(&bot, &msg, db, telegram_id, lang).await
                        && !e.to_string().contains("message is not modified")
                    {
                        return Err(e);
                    }
                }
                Err(e) => {
                    check_db_err(
                        &bot,
                        &q.id.0,
                        Err(e),
                        config,
                        telegram_id,
                        AdminErrorContext::Callback,
                        lang,
                    )
                    .await?;
                }
            }
        }
        SettingsAction::MuteManage => {
            match user_settings_service::get_or_create(db, telegram_id, LanguageCode::En).await {
                Ok(u) => {
                    let mode = user_settings_service::parse_mute_list_mode(&u.mute_list_mode);
                    send_mute_menu(&bot, &msg, lang, mode).await?;
                }
                Err(e) => {
                    check_db_err(
                        &bot,
                        &q.id.0,
                        Err(e),
                        config,
                        telegram_id,
                        AdminErrorContext::Callback,
                        lang,
                    )
                    .await?;
                }
            }
        }
    }
    Ok(())
}
