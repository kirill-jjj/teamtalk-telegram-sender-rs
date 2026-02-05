use crate::adapters::tg::presenter::settings::{
    QueueAdminStatus, QueueLinkStatus, QueueSettingsView, QueueToggleStatus, send_queue_settings,
};
use crate::adapters::tg::utils::{answer_callback, check_db_err};
use crate::app::services::tg_admin as tg_admin_service;
use crate::app::services::tg_queue_settings as tg_queue_settings_service;
use crate::args;
use crate::core::types::{AdminErrorContext, LanguageCode, TelegramId};
use crate::infra::locales;
use teloxide::prelude::*;

use super::AppState;
use crate::infra::db::types::UserSettings;

async fn load_settings_or_reply(
    bot: &Bot,
    q_id: &str,
    state: &AppState,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> ResponseResult<Option<UserSettings>> {
    match tg_queue_settings_service::load_settings(&state.db, telegram_id, LanguageCode::En).await {
        Ok(s) => Ok(Some(s)),
        Err(e) => {
            check_db_err(
                bot,
                q_id,
                Err(e),
                &state.config,
                telegram_id,
                AdminErrorContext::Callback,
                lang,
            )
            .await?;
            Ok(None)
        }
    }
}

async fn global_enabled_or_reply(
    bot: &Bot,
    q_id: &str,
    state: &AppState,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> ResponseResult<Option<bool>> {
    match tg_queue_settings_service::global_enabled(&state.db).await {
        Ok(val) => Ok(Some(val)),
        Err(e) => {
            check_db_err(
                bot,
                q_id,
                Err(e),
                &state.config,
                telegram_id,
                AdminErrorContext::Callback,
                lang,
            )
            .await?;
            Ok(None)
        }
    }
}

pub(super) async fn handle_queue_menu(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    telegram_id: TelegramId,
    lang: LanguageCode,
    is_admin: bool,
) -> ResponseResult<()> {
    let settings = match tg_queue_settings_service::load_settings(
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
                locales::get_text_or_log(lang.as_str(), locales::LocaleKey::CmdError, None),
            )
            .await?;
            return Ok(());
        }
    };
    let global_enabled = tg_queue_settings_service::global_enabled(&state.db)
        .await
        .unwrap_or(false);
    send_queue_settings(
        bot,
        msg,
        lang,
        QueueSettingsView {
            link: if settings.teamtalk_username.is_some() {
                QueueLinkStatus::Linked
            } else {
                QueueLinkStatus::Unlinked
            },
            user: if settings.reply_queue_enabled {
                QueueToggleStatus::Enabled
            } else {
                QueueToggleStatus::Disabled
            },
            global: if global_enabled {
                QueueToggleStatus::Enabled
            } else {
                QueueToggleStatus::Disabled
            },
            admin: if is_admin {
                QueueAdminStatus::Admin
            } else {
                QueueAdminStatus::User
            },
        },
    )
    .await
}

pub(super) async fn handle_queue_toggle_user(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let Some(settings) = load_settings_or_reply(bot, &q.id.0, state, telegram_id, lang).await?
    else {
        return Ok(());
    };

    if settings.teamtalk_username.is_none() {
        answer_callback(
            bot,
            &q.id,
            locales::get_text_or_log(lang.as_str(), locales::LocaleKey::CmdQueueNoLink, None),
            true,
        )
        .await?;
        return Ok(());
    }

    let Some(global_enabled) =
        global_enabled_or_reply(bot, &q.id.0, state, telegram_id, lang).await?
    else {
        return Ok(());
    };
    if !global_enabled {
        answer_callback(
            bot,
            &q.id,
            locales::get_text_or_log(
                lang.as_str(),
                locales::LocaleKey::RespQueueGlobalDisabledUser,
                None,
            ),
            true,
        )
        .await?;
        return Ok(());
    }

    let new_val = !settings.reply_queue_enabled;
    if let Err(e) =
        tg_queue_settings_service::set_user_enabled(&state.db, telegram_id, new_val).await
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
        locales::LocaleKey::RespQueueUserEnabled
    } else {
        locales::LocaleKey::RespQueueUserDisabled
    };
    answer_callback(
        bot,
        &q.id,
        locales::get_text_or_log(lang.as_str(), status_key, None),
        false,
    )
    .await?;

    let is_admin = tg_admin_service::is_admin(&state.db, &state.config, telegram_id).await;
    let global_enabled = tg_queue_settings_service::global_enabled(&state.db)
        .await
        .unwrap_or(false);
    send_queue_settings(
        bot,
        msg,
        lang,
        QueueSettingsView {
            link: if settings.teamtalk_username.is_some() {
                QueueLinkStatus::Linked
            } else {
                QueueLinkStatus::Unlinked
            },
            user: if new_val {
                QueueToggleStatus::Enabled
            } else {
                QueueToggleStatus::Disabled
            },
            global: if global_enabled {
                QueueToggleStatus::Enabled
            } else {
                QueueToggleStatus::Disabled
            },
            admin: if is_admin {
                QueueAdminStatus::Admin
            } else {
                QueueAdminStatus::User
            },
        },
    )
    .await
}

pub(super) async fn handle_queue_toggle_global(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let is_admin = tg_admin_service::is_admin(&state.db, &state.config, telegram_id).await;
    if !is_admin {
        answer_callback(
            bot,
            &q.id,
            locales::get_text_or_log(lang.as_str(), locales::LocaleKey::CmdUnauth, None),
            true,
        )
        .await?;
        return Ok(());
    }
    let current = tg_queue_settings_service::global_enabled(&state.db)
        .await
        .unwrap_or(false);
    let new_val = !current;
    if let Err(e) = tg_queue_settings_service::set_global_enabled(&state.db, new_val).await {
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
        locales::LocaleKey::RespQueueGlobalEnabled
    } else {
        locales::LocaleKey::RespQueueGlobalDisabled
    };
    answer_callback(
        bot,
        &q.id,
        locales::get_text_or_log(lang.as_str(), status_key, None),
        false,
    )
    .await?;
    let Some(settings) = load_settings_or_reply(bot, &q.id.0, state, telegram_id, lang).await?
    else {
        return Ok(());
    };
    send_queue_settings(
        bot,
        msg,
        lang,
        QueueSettingsView {
            link: if settings.teamtalk_username.is_some() {
                QueueLinkStatus::Linked
            } else {
                QueueLinkStatus::Unlinked
            },
            user: if settings.reply_queue_enabled {
                QueueToggleStatus::Enabled
            } else {
                QueueToggleStatus::Disabled
            },
            global: if new_val {
                QueueToggleStatus::Enabled
            } else {
                QueueToggleStatus::Disabled
            },
            admin: if is_admin {
                QueueAdminStatus::Admin
            } else {
                QueueAdminStatus::User
            },
        },
    )
    .await
}

pub(super) async fn handle_queue_clear_self(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let Some(settings) = load_settings_or_reply(bot, &q.id.0, state, telegram_id, lang).await?
    else {
        return Ok(());
    };
    let has_link = settings.teamtalk_username.is_some();
    let Some(tt_username) = settings.teamtalk_username.as_ref() else {
        answer_callback(
            bot,
            &q.id,
            locales::get_text_or_log(lang.as_str(), locales::LocaleKey::CmdQueueNoLink, None),
            true,
        )
        .await?;
        return Ok(());
    };
    let tt_username = tt_username.clone();
    let cleared = match tg_queue_settings_service::clear_user(&state.db, &tt_username).await {
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
        locales::get_text_or_log(
            lang.as_str(),
            locales::LocaleKey::RespQueueCleared,
            args!(count = cleared).as_ref(),
        ),
        false,
    )
    .await?;
    let is_admin = tg_admin_service::is_admin(&state.db, &state.config, telegram_id).await;
    let global_enabled = tg_queue_settings_service::global_enabled(&state.db)
        .await
        .unwrap_or(false);
    send_queue_settings(
        bot,
        msg,
        lang,
        QueueSettingsView {
            link: if has_link {
                QueueLinkStatus::Linked
            } else {
                QueueLinkStatus::Unlinked
            },
            user: if settings.reply_queue_enabled {
                QueueToggleStatus::Enabled
            } else {
                QueueToggleStatus::Disabled
            },
            global: if global_enabled {
                QueueToggleStatus::Enabled
            } else {
                QueueToggleStatus::Disabled
            },
            admin: if is_admin {
                QueueAdminStatus::Admin
            } else {
                QueueAdminStatus::User
            },
        },
    )
    .await
}

pub(super) async fn handle_queue_clear_all(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let is_admin = tg_admin_service::is_admin(&state.db, &state.config, telegram_id).await;
    if !is_admin {
        answer_callback(
            bot,
            &q.id,
            locales::get_text_or_log(lang.as_str(), locales::LocaleKey::CmdUnauth, None),
            true,
        )
        .await?;
        return Ok(());
    }
    let cleared = match tg_queue_settings_service::clear_all(&state.db).await {
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
        locales::get_text_or_log(
            lang.as_str(),
            locales::LocaleKey::RespQueueClearedAll,
            args!(count = cleared).as_ref(),
        ),
        false,
    )
    .await?;
    let Some(settings) = load_settings_or_reply(bot, &q.id.0, state, telegram_id, lang).await?
    else {
        return Ok(());
    };
    let global_enabled = tg_queue_settings_service::global_enabled(&state.db)
        .await
        .unwrap_or(false);
    send_queue_settings(
        bot,
        msg,
        lang,
        QueueSettingsView {
            link: if settings.teamtalk_username.is_some() {
                QueueLinkStatus::Linked
            } else {
                QueueLinkStatus::Unlinked
            },
            user: if settings.reply_queue_enabled {
                QueueToggleStatus::Enabled
            } else {
                QueueToggleStatus::Disabled
            },
            global: if global_enabled {
                QueueToggleStatus::Enabled
            } else {
                QueueToggleStatus::Disabled
            },
            admin: if is_admin {
                QueueAdminStatus::Admin
            } else {
                QueueAdminStatus::User
            },
        },
    )
    .await
}
