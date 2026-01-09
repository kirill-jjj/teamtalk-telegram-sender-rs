use crate::args;
use crate::locales;
use crate::tg_bot::callbacks_types::{CallbackAction, SettingsAction};
use crate::tg_bot::settings_logic::{
    send_main_settings_edit, send_mute_menu, send_notif_settings, send_sub_settings,
};
use crate::tg_bot::state::AppState;
use crate::types::NotificationSetting;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub async fn handle_settings(
    bot: Bot,
    q: CallbackQuery,
    state: AppState,
    action: SettingsAction,
    lang: &str,
) -> ResponseResult<()> {
    let msg = match q.message {
        Some(teloxide::types::MaybeInaccessibleMessage::Regular(m)) => m,
        _ => return Ok(()),
    };
    let chat_id = msg.chat.id;
    let telegram_id = q.from.id.0 as i64;
    let db = &state.db;

    match action {
        SettingsAction::Main => {
            send_main_settings_edit(&bot, &msg, lang).await?;
        }
        SettingsAction::LangSelect => {
            let keyboard = InlineKeyboardMarkup::new(vec![
                vec![InlineKeyboardButton::callback(
                    "🇷🇺 Русский",
                    CallbackAction::Settings(SettingsAction::LangSet {
                        lang: "ru".to_string(),
                    })
                    .to_string(),
                )],
                vec![InlineKeyboardButton::callback(
                    "🇬🇧 English",
                    CallbackAction::Settings(SettingsAction::LangSet {
                        lang: "en".to_string(),
                    })
                    .to_string(),
                )],
                vec![InlineKeyboardButton::callback(
                    locales::get_text(lang, "btn-back-settings", None),
                    CallbackAction::Settings(SettingsAction::Main).to_string(),
                )],
            ]);
            bot.edit_message_text(
                chat_id,
                msg.id,
                locales::get_text(lang, "msg-choose-lang", None),
            )
            .reply_markup(keyboard)
            .await?;
        }
        SettingsAction::LangSet { lang: new_lang } => {
            db.update_language(telegram_id, &new_lang).await.ok();
            bot.answer_callback_query(q.id)
                .text(locales::get_text(&new_lang, "toast-lang-updated", None))
                .await?;
            send_main_settings_edit(&bot, &msg, &new_lang).await?;
        }
        SettingsAction::SubSelect => {
            send_sub_settings(&bot, &msg, db, telegram_id, lang).await?;
        }
        SettingsAction::SubSet { setting } => {
            let new_setting = NotificationSetting::from(setting.as_str());
            db.update_notification_setting(telegram_id, new_setting.clone())
                .await
                .ok();

            let text_key = match new_setting {
                NotificationSetting::All => "btn-sub-all",
                NotificationSetting::JoinOff => "btn-sub-join",
                NotificationSetting::LeaveOff => "btn-sub-leave",
                NotificationSetting::None => "btn-sub-none",
            };
            let setting_text = locales::get_text(lang, text_key, args!(marker = "").as_ref());
            bot.answer_callback_query(q.id)
                .text(locales::get_text(
                    lang,
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
            let user_settings = db.get_or_create_user(telegram_id, "en").await.ok();
            if let Some(u) = user_settings {
                if u.teamtalk_username.is_none() {
                    bot.answer_callback_query(q.id)
                        .text(locales::get_text(lang, "cmd-fail-noon-guest", None))
                        .show_alert(true)
                        .await?;
                    return Ok(());
                }
                match db.toggle_noon(telegram_id).await {
                    Ok(new_val) => {
                        let status = if new_val {
                            locales::get_text(lang, "status-enabled", None)
                        } else {
                            locales::get_text(lang, "status-disabled", None)
                        };

                        let _ = bot
                            .answer_callback_query(q.id)
                            .text(locales::get_text(
                                lang,
                                "resp-noon-updated",
                                args!(status = status).as_ref(),
                            ))
                            .await;

                        if let Err(e) = send_notif_settings(&bot, &msg, db, telegram_id, lang).await
                            && !e.to_string().contains("message is not modified")
                        {
                            return Err(e);
                        }
                    }
                    Err(e) => {
                        log::error!("DB error in toggle_noon: {}", e);
                        bot.answer_callback_query(q.id)
                            .text(locales::get_text(lang, "cmd-error", None))
                            .show_alert(true)
                            .await?;
                    }
                }
            }
        }
        SettingsAction::MuteManage => {
            if let Ok(u) = db.get_or_create_user(telegram_id, "en").await {
                send_mute_menu(&bot, &msg, lang, &u.mute_list_mode).await?;
            }
        }
    }
    Ok(())
}
