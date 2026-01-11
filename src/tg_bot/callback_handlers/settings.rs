use crate::args;
use crate::locales;
use crate::tg_bot::callbacks_types::{CallbackAction, SettingsAction};
use crate::tg_bot::settings_logic::{
    send_main_settings_edit, send_mute_menu, send_notif_settings, send_sub_settings,
};
use crate::tg_bot::state::AppState;
use crate::tg_bot::utils::check_db_err;
use crate::types::{LanguageCode, MuteListMode, NotificationSetting};
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

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
                vec![InlineKeyboardButton::callback(
                    "🇷🇺 Русский",
                    CallbackAction::Settings(SettingsAction::LangSet {
                        lang: LanguageCode::Ru,
                    })
                    .to_string(),
                )],
                vec![InlineKeyboardButton::callback(
                    "🇬🇧 English",
                    CallbackAction::Settings(SettingsAction::LangSet {
                        lang: LanguageCode::En,
                    })
                    .to_string(),
                )],
                vec![InlineKeyboardButton::callback(
                    locales::get_text(lang.as_str(), "btn-back-settings", None),
                    CallbackAction::Settings(SettingsAction::Main).to_string(),
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
                "admin-error-context-callback",
                lang,
            )
            .await?
            {
                return Ok(());
            }
            bot.answer_callback_query(q.id)
                .text(locales::get_text(
                    new_lang.as_str(),
                    "toast-lang-updated",
                    None,
                ))
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
                "admin-error-context-callback",
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
            bot.answer_callback_query(q.id)
                .text(locales::get_text(
                    lang.as_str(),
                    "resp-sub-updated",
                    args!(text = setting_text).as_ref(),
                ))
                .await?;
            send_sub_settings(&bot, &msg, db, telegram_id, lang).await?;
        }
        SettingsAction::NotifSelect => {
            send_notif_settings(&bot, &msg, db, telegram_id, lang).await?;
        }
        SettingsAction::NoonToggle => {
            let user_settings = match db.get_or_create_user(telegram_id, LanguageCode::En).await {
                Ok(u) => u,
                Err(e) => {
                    check_db_err(
                        &bot,
                        &q.id.0,
                        Err(e),
                        config,
                        telegram_id,
                        "admin-error-context-callback",
                        lang,
                    )
                    .await?;
                    return Ok(());
                }
            };

            if user_settings.teamtalk_username.is_none() {
                bot.answer_callback_query(q.id)
                    .text(locales::get_text(
                        lang.as_str(),
                        "cmd-fail-noon-guest",
                        None,
                    ))
                    .show_alert(true)
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

                    if let Err(e) = bot
                        .answer_callback_query(q.id)
                        .text(locales::get_text(
                            lang.as_str(),
                            "resp-noon-updated",
                            args!(status = status).as_ref(),
                        ))
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
                        "admin-error-context-callback",
                        lang,
                    )
                    .await?;
                }
            }
        }
        SettingsAction::MuteManage => {
            match db.get_or_create_user(telegram_id, LanguageCode::En).await {
                Ok(u) => {
                    let mode = MuteListMode::try_from(u.mute_list_mode.as_str())
                        .unwrap_or(MuteListMode::Blacklist);
                    send_mute_menu(&bot, &msg, lang, mode).await?;
                }
                Err(e) => {
                    check_db_err(
                        &bot,
                        &q.id.0,
                        Err(e),
                        config,
                        telegram_id,
                        "admin-error-context-callback",
                        lang,
                    )
                    .await?;
                }
            }
        }
    }
    Ok(())
}
