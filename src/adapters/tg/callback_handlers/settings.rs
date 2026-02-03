use crate::adapters::tg::keyboards::{back_button, callback_button};
use crate::adapters::tg::settings_logic::{
    send_main_settings_edit, send_mute_menu, send_notif_settings, send_queue_settings,
    send_sub_settings,
};
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{answer_callback, check_db_err};
use crate::app::services::reply_queue as reply_queue_service;
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
    let Some(teloxide::types::MaybeInaccessibleMessage::Regular(ref msg)) = q.message else {
        return Ok(());
    };
    let msg = msg.as_ref();
    let telegram_id = tg_user_id_i64(q.from.id.0);

    match action {
        SettingsAction::Main => {
            send_main_settings_edit(&bot, msg, lang).await?;
        }
        SettingsAction::LangSelect => {
            handle_lang_select(&bot, msg, lang).await?;
        }
        SettingsAction::LangSet { lang: new_lang } => {
            handle_lang_set(&bot, &q, &state, msg, telegram_id, lang, new_lang).await?;
        }
        SettingsAction::SubSelect => {
            send_sub_settings(&bot, msg, &state.db, telegram_id, lang).await?;
        }
        SettingsAction::SubSet { setting } => {
            handle_sub_set(&bot, &q, &state, msg, telegram_id, lang, setting).await?;
        }
        SettingsAction::NotifSelect => {
            send_notif_settings(&bot, msg, &state.db, telegram_id, lang).await?;
        }
        SettingsAction::NoonToggle => {
            handle_noon_toggle(&bot, &q, &state, msg, telegram_id, lang).await?;
        }
        SettingsAction::MuteManage => {
            handle_mute_manage(&bot, &q, &state, msg, telegram_id, lang).await?;
        }
        SettingsAction::QueueMenu => {
            let is_admin = is_admin(&state.db, &state.config, telegram_id).await;
            send_queue_settings(&bot, msg, &state.db, telegram_id, lang, is_admin).await?;
        }
        SettingsAction::QueueToggleUser => {
            handle_queue_toggle_user(&bot, &q, &state, msg, telegram_id, lang).await?;
        }
        SettingsAction::QueueToggleGlobal => {
            handle_queue_toggle_global(&bot, &q, &state, msg, telegram_id, lang).await?;
        }
        SettingsAction::QueueClearSelf => {
            handle_queue_clear_self(&bot, &q, &state, msg, telegram_id, lang).await?;
        }
        SettingsAction::QueueClearAll => {
            handle_queue_clear_all(&bot, &q, &state, msg, telegram_id, lang).await?;
        }
    }
    Ok(())
}

async fn handle_lang_select(bot: &Bot, msg: &Message, lang: LanguageCode) -> ResponseResult<()> {
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
            "btn-back-settings",
            CallbackAction::Settings(SettingsAction::Main),
        )],
    ]);
    bot.edit_message_text(
        msg.chat.id,
        msg.id,
        locales::get_text(lang.as_str(), "msg-choose-lang", None),
    )
    .reply_markup(keyboard)
    .await?;
    Ok(())
}

async fn handle_lang_set(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: i64,
    lang: LanguageCode,
    new_lang: LanguageCode,
) -> ResponseResult<()> {
    if check_db_err(
        bot,
        &q.id.0,
        state.db.update_language(telegram_id, new_lang).await,
        &state.config,
        telegram_id,
        AdminErrorContext::Callback,
        lang,
    )
    .await?
    {
        return Ok(());
    }
    answer_callback(
        bot,
        &q.id,
        locales::get_text(new_lang.as_str(), "toast-lang-updated", None),
        false,
    )
    .await?;
    send_main_settings_edit(bot, msg, new_lang).await
}

async fn handle_sub_set(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: i64,
    lang: LanguageCode,
    setting: NotificationSetting,
) -> ResponseResult<()> {
    if check_db_err(
        bot,
        &q.id.0,
        state
            .db
            .update_notification_setting(telegram_id, setting.clone())
            .await,
        &state.config,
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
        NotificationSetting::JoinOff => "btn-sub-leave",
        NotificationSetting::LeaveOff => "btn-sub-join",
        NotificationSetting::None => "btn-sub-none",
    };
    let setting_text = locales::get_text(lang.as_str(), text_key, args!(marker = "").as_ref());
    answer_callback(
        bot,
        &q.id,
        locales::get_text(
            lang.as_str(),
            "resp-sub-updated",
            args!(text = setting_text).as_ref(),
        ),
        false,
    )
    .await?;
    send_sub_settings(bot, msg, &state.db, telegram_id, lang).await
}

async fn handle_noon_toggle(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: i64,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let user_settings = match user_settings_service::get_or_create(
        &state.db,
        telegram_id,
        LanguageCode::En,
    )
    .await
    {
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
            locales::get_text(lang.as_str(), "cmd-fail-noon-guest", None),
            true,
        )
        .await?;
        return Ok(());
    }

    match state.db.toggle_noon(telegram_id).await {
        Ok(new_val) => {
            let status = if new_val {
                locales::get_text(lang.as_str(), "status-enabled", None)
            } else {
                locales::get_text(lang.as_str(), "status-disabled", None)
            };
            if let Err(e) = answer_callback(
                bot,
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
                tracing::error!(error = %e, "Failed to send noon update callback");
            }

            send_notif_settings(bot, msg, &state.db, telegram_id, lang).await?;
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

async fn handle_mute_manage(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: i64,
    lang: LanguageCode,
) -> ResponseResult<()> {
    match user_settings_service::get_or_create(&state.db, telegram_id, LanguageCode::En).await {
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

fn tg_user_id_i64(user_id: u64) -> i64 {
    i64::try_from(user_id).unwrap_or(i64::MAX)
}

async fn handle_queue_toggle_user(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: i64,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let settings = match user_settings_service::get_or_create(
        &state.db,
        telegram_id,
        LanguageCode::En,
    )
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

    if settings.teamtalk_username.is_none() {
        answer_callback(
            bot,
            &q.id,
            locales::get_text(lang.as_str(), "cmd-queue-no-link", None),
            true,
        )
        .await?;
        return Ok(());
    }

    let global_enabled = match reply_queue_service::get_reply_queue_global_enabled(&state.db).await
    {
        Ok(val) => val,
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
    if !global_enabled {
        answer_callback(
            bot,
            &q.id,
            locales::get_text(lang.as_str(), "resp-queue-global-disabled-user", None),
            true,
        )
        .await?;
        return Ok(());
    }

    let new_val = !settings.reply_queue_enabled;
    if let Err(e) =
        reply_queue_service::set_reply_queue_user_enabled(&state.db, telegram_id, new_val).await
    {
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

    let status_key = if new_val {
        "resp-queue-user-enabled"
    } else {
        "resp-queue-user-disabled"
    };
    answer_callback(
        bot,
        &q.id,
        locales::get_text(lang.as_str(), status_key, None),
        false,
    )
    .await?;

    let is_admin = is_admin(&state.db, &state.config, telegram_id).await;
    send_queue_settings(bot, msg, &state.db, telegram_id, lang, is_admin).await
}

async fn handle_queue_toggle_global(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: i64,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let is_admin = is_admin(&state.db, &state.config, telegram_id).await;
    if !is_admin {
        answer_callback(
            bot,
            &q.id,
            locales::get_text(lang.as_str(), "cmd-unauth", None),
            true,
        )
        .await?;
        return Ok(());
    }
    let current = reply_queue_service::get_reply_queue_global_enabled(&state.db)
        .await
        .unwrap_or(false);
    let new_val = !current;
    if let Err(e) = reply_queue_service::set_reply_queue_global_enabled(&state.db, new_val).await {
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
    let status_key = if new_val {
        "resp-queue-global-enabled"
    } else {
        "resp-queue-global-disabled"
    };
    answer_callback(
        bot,
        &q.id,
        locales::get_text(lang.as_str(), status_key, None),
        false,
    )
    .await?;
    send_queue_settings(bot, msg, &state.db, telegram_id, lang, is_admin).await
}

async fn handle_queue_clear_self(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: i64,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let settings = match user_settings_service::get_or_create(
        &state.db,
        telegram_id,
        LanguageCode::En,
    )
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
    let Some(tt_username) = settings.teamtalk_username else {
        answer_callback(
            bot,
            &q.id,
            locales::get_text(lang.as_str(), "cmd-queue-no-link", None),
            true,
        )
        .await?;
        return Ok(());
    };
    let cleared = match state.db.clear_reply_queue_for_user(&tt_username).await {
        Ok(count) => count,
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
    answer_callback(
        bot,
        &q.id,
        locales::get_text(
            lang.as_str(),
            "resp-queue-cleared",
            args!(count = cleared).as_ref(),
        ),
        false,
    )
    .await?;
    let is_admin = is_admin(&state.db, &state.config, telegram_id).await;
    send_queue_settings(bot, msg, &state.db, telegram_id, lang, is_admin).await
}

async fn handle_queue_clear_all(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: i64,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let is_admin = is_admin(&state.db, &state.config, telegram_id).await;
    if !is_admin {
        answer_callback(
            bot,
            &q.id,
            locales::get_text(lang.as_str(), "cmd-unauth", None),
            true,
        )
        .await?;
        return Ok(());
    }
    let cleared = match state.db.clear_reply_queue_all().await {
        Ok(count) => count,
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
    answer_callback(
        bot,
        &q.id,
        locales::get_text(
            lang.as_str(),
            "resp-queue-cleared-all",
            args!(count = cleared).as_ref(),
        ),
        false,
    )
    .await?;
    send_queue_settings(bot, msg, &state.db, telegram_id, lang, is_admin).await
}

async fn is_admin(
    db: &crate::infra::db::Database,
    config: &crate::bootstrap::config::Config,
    telegram_id: i64,
) -> bool {
    if telegram_id == config.telegram.admin_chat_id {
        return true;
    }
    match db.get_all_admins().await {
        Ok(admins) => admins.contains(&telegram_id),
        Err(e) => {
            tracing::error!(error = %e, "Failed to load admin list");
            false
        }
    }
}
