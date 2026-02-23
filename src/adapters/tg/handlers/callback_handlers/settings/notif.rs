use crate::adapters::tg::presenter::keyboards::{
    back_btn, callback_button, create_user_list_keyboard,
};
use crate::adapters::tg::presenter::settings::send_notif_settings;
use crate::adapters::tg::utils::{TgErrorReporter, answer_callback, cmd_error_text};
use crate::app::services::tg_admin as tg_admin_service;
use crate::app::services::tg_search_actions as tg_search_actions_service;
use crate::app::services::tg_settings as tg_settings_service;
use crate::args;
use crate::core::callbacks::{CallbackAction, SettingsAction};
use crate::core::types::{AdminErrorContext, AfkListMode, LanguageCode, TelegramId, TtUsername};
use crate::infra::locales;
use teloxide_ng::prelude::*;

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
    let is_admin = tg_admin_service::is_admin(&state.db, &state.config, telegram_id).await;
    let admin_sub_events_enabled = if is_admin {
        match tg_settings_service::admin_sub_events_enabled(
            &state.db,
            telegram_id,
            state.config.general.default_lang,
        )
        .await
        {
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

pub(super) async fn handle_afk_menu(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    render_afk_menu(bot, msg, state, telegram_id, lang).await
}

pub(super) async fn handle_afk_toggle(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let resolved = match state.db.resolve_afk_settings_for_user(telegram_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(telegram_id = telegram_id.as_i64(), error = %e, "AFK toggle read failed");
            return Ok(());
        }
    };
    if let Err(e) = state
        .db
        .set_afk_enabled(telegram_id, !resolved.enabled)
        .await
    {
        tracing::error!(telegram_id = telegram_id.as_i64(), error = %e, "AFK toggle write failed");
        return Ok(());
    }
    answer_callback(
        bot,
        &q.id,
        locales::get_text(lang.as_str(), locales::LocaleKey::ToastAfkUpdated, None),
        false,
    )
    .await?;
    render_afk_menu(bot, msg, state, telegram_id, lang).await
}

pub(super) async fn handle_afk_threshold_step(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
    delta: i32,
) -> ResponseResult<()> {
    let resolved = match state.db.resolve_afk_settings_for_user(telegram_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(telegram_id = telegram_id.as_i64(), error = %e, "AFK threshold read failed");
            return Ok(());
        }
    };
    let mut value = resolved.threshold_minutes + i64::from(delta);
    value = value.clamp(1, 1440);
    if let Err(e) = state.db.set_afk_threshold_minutes(telegram_id, value).await {
        tracing::error!(telegram_id = telegram_id.as_i64(), error = %e, "AFK threshold write failed");
        return Ok(());
    }
    answer_callback(
        bot,
        &q.id,
        locales::get_text(
            lang.as_str(),
            locales::LocaleKey::ToastAfkThresholdUpdated,
            None,
        ),
        false,
    )
    .await?;
    render_afk_menu(bot, msg, state, telegram_id, lang).await
}

pub(super) async fn handle_afk_cooldown_step(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
    delta: i32,
) -> ResponseResult<()> {
    let resolved = match state.db.resolve_afk_settings_for_user(telegram_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(telegram_id = telegram_id.as_i64(), error = %e, "AFK cooldown read failed");
            return Ok(());
        }
    };
    let mut value = resolved.cooldown_seconds + i64::from(delta);
    value = value.clamp(0, 3600);
    if let Err(e) = state.db.set_afk_cooldown_seconds(telegram_id, value).await {
        tracing::error!(telegram_id = telegram_id.as_i64(), error = %e, "AFK cooldown write failed");
        return Ok(());
    }
    answer_callback(
        bot,
        &q.id,
        locales::get_text(
            lang.as_str(),
            locales::LocaleKey::ToastAfkCooldownUpdated,
            None,
        ),
        false,
    )
    .await?;
    render_afk_menu(bot, msg, state, telegram_id, lang).await
}

pub(super) async fn handle_afk_mode_set(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
    mode: AfkListMode,
) -> ResponseResult<()> {
    if let Err(e) = state.db.set_afk_list_mode(telegram_id, mode).await {
        tracing::error!(telegram_id = telegram_id.as_i64(), error = %e, "AFK mode write failed");
        return Ok(());
    }
    answer_callback(
        bot,
        &q.id,
        locales::get_text(lang.as_str(), locales::LocaleKey::ToastAfkModeUpdated, None),
        false,
    )
    .await?;
    render_afk_menu(bot, msg, state, telegram_id, lang).await
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_afk_list(
    bot: &Bot,
    _q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
    mode: AfkListMode,
    page: usize,
) -> ResponseResult<()> {
    let accounts = tg_search_actions_service::list_user_accounts(&state.state).await;
    let tracked = state
        .db
        .get_afk_tracked_users(telegram_id, mode)
        .await
        .unwrap_or_default();
    let tracked_set: std::collections::HashSet<_> = tracked.into_iter().collect();

    let keyboard = create_user_list_keyboard(
        &accounts,
        page,
        |acc| {
            let username = TtUsername::new(acc.username.clone());
            let checked = tracked_set.contains(&username);
            let icon = if checked { "✅" } else { "☑️" };
            (
                format!("{icon} {}", acc.username),
                CallbackAction::Settings(SettingsAction::AfkListToggle {
                    mode,
                    username,
                    page,
                }),
            )
        },
        |p| CallbackAction::Settings(SettingsAction::AfkList { mode, page: p }),
        Some(back_btn(
            lang,
            locales::LocaleKey::BtnBackNotif,
            CallbackAction::Settings(SettingsAction::AfkMenu),
        )),
        lang,
    );

    let title = match mode {
        AfkListMode::Blacklist => locales::get_text(
            lang.as_str(),
            locales::LocaleKey::AfkListTitleBlacklist,
            None,
        ),
        AfkListMode::Whitelist => locales::get_text(
            lang.as_str(),
            locales::LocaleKey::AfkListTitleWhitelist,
            None,
        ),
        AfkListMode::None => {
            locales::get_text(lang.as_str(), locales::LocaleKey::AfkListTitleNone, None)
        }
    };
    bot.edit_message_text(msg.chat.id, msg.id, title)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_afk_list_toggle(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
    mode: AfkListMode,
    username: TtUsername,
    page: usize,
) -> ResponseResult<()> {
    if matches!(mode, AfkListMode::None) {
        answer_callback(
            bot,
            &q.id,
            locales::get_text(
                lang.as_str(),
                locales::LocaleKey::ToastAfkSwitchModeFirst,
                None,
            ),
            true,
        )
        .await?;
        return Ok(());
    }
    if let Err(e) = state
        .db
        .toggle_afk_tracked_user(telegram_id, mode, &username)
        .await
    {
        tracing::error!(telegram_id = telegram_id.as_i64(), error = %e, "AFK list toggle failed");
        return Ok(());
    }
    answer_callback(
        bot,
        &q.id,
        locales::get_text(lang.as_str(), locales::LocaleKey::ToastAfkListUpdated, None),
        false,
    )
    .await?;
    handle_afk_list(bot, q, state, msg, telegram_id, lang, mode, page).await
}

pub(super) async fn handle_afk_overrides(
    bot: &Bot,
    _q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
    page: usize,
) -> ResponseResult<()> {
    let overrides = state
        .db
        .list_afk_threshold_overrides(telegram_id)
        .await
        .unwrap_or_default();
    let map: std::collections::HashMap<_, _> = overrides.into_iter().collect();
    let accounts = tg_search_actions_service::list_user_accounts(&state.state).await;
    let keyboard = create_user_list_keyboard(
        &accounts,
        page,
        |acc| {
            let username = TtUsername::new(acc.username.clone());
            if let Some(minutes) = map.get(&username).copied() {
                (
                    format!("🧩 {username} ({minutes}m)"),
                    CallbackAction::Settings(SettingsAction::AfkOverrideDelete { username, page }),
                )
            } else {
                (
                    format!("➕ {username} (10m)"),
                    CallbackAction::Settings(SettingsAction::AfkOverrideSetPreset {
                        username,
                        minutes: 10,
                        page,
                    }),
                )
            }
        },
        |p| CallbackAction::Settings(SettingsAction::AfkOverrides { page: p }),
        Some(back_btn(
            lang,
            locales::LocaleKey::BtnBackNotif,
            CallbackAction::Settings(SettingsAction::AfkMenu),
        )),
        lang,
    );

    bot.edit_message_text(
        msg.chat.id,
        msg.id,
        locales::get_text(lang.as_str(), locales::LocaleKey::AfkOverridesTitle, None),
    )
    .reply_markup(keyboard)
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_afk_override_delete(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
    username: TtUsername,
    page: usize,
) -> ResponseResult<()> {
    if let Err(e) = state
        .db
        .delete_afk_threshold_override(telegram_id, &username)
        .await
    {
        tracing::error!(telegram_id = telegram_id.as_i64(), error = %e, "AFK override delete failed");
        return Ok(());
    }
    answer_callback(
        bot,
        &q.id,
        locales::get_text(
            lang.as_str(),
            locales::LocaleKey::ToastAfkOverrideDeleted,
            None,
        ),
        false,
    )
    .await?;
    handle_afk_overrides(bot, q, state, msg, telegram_id, lang, page).await
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn handle_afk_override_set_preset(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: TelegramId,
    lang: LanguageCode,
    username: TtUsername,
    minutes: i64,
    page: usize,
) -> ResponseResult<()> {
    if let Err(e) = state
        .db
        .set_afk_threshold_override(telegram_id, &username, minutes)
        .await
    {
        tracing::error!(telegram_id = telegram_id.as_i64(), error = %e, "AFK override set failed");
        return Ok(());
    }
    answer_callback(
        bot,
        &q.id,
        locales::get_text(
            lang.as_str(),
            locales::LocaleKey::ToastAfkOverrideUpdated,
            args!(user = username.to_string(), minutes = minutes).as_ref(),
        ),
        false,
    )
    .await?;
    handle_afk_overrides(bot, q, state, msg, telegram_id, lang, page).await
}

#[allow(clippy::too_many_lines)]
async fn render_afk_menu(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let resolved = match state.db.resolve_afk_settings_for_user(telegram_id).await {
        Ok(value) => value,
        Err(err) => {
            tracing::error!(
                telegram_id = telegram_id.as_i64(),
                error = %err,
                "Failed to resolve AFK settings"
            );
            bot.edit_message_text(msg.chat.id, msg.id, cmd_error_text(lang))
                .await?;
            return Ok(());
        }
    };
    let status = if resolved.enabled {
        locales::get_text(lang.as_str(), locales::LocaleKey::StatusEnabled, None)
    } else {
        locales::get_text(lang.as_str(), locales::LocaleKey::StatusDisabled, None)
    };
    let mode = resolved.list_mode.to_string();
    let text = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::AfkSettingsTitle,
        args!(
            status = status.clone(),
            threshold = resolved.threshold_minutes,
            mode = mode,
            cooldown = resolved.cooldown_seconds
        )
        .as_ref(),
    );
    let keyboard = teloxide_ng::types::InlineKeyboardMarkup::new(vec![
        vec![callback_button(
            locales::get_text(
                lang.as_str(),
                locales::LocaleKey::BtnAfkToggle,
                args!(status = status.clone()).as_ref(),
            ),
            CallbackAction::Settings(SettingsAction::AfkToggle),
        )],
        vec![
            callback_button(
                locales::get_text(
                    lang.as_str(),
                    locales::LocaleKey::BtnAfkThresholdMinus,
                    None,
                ),
                CallbackAction::Settings(SettingsAction::AfkThresholdStep { delta: -1 }),
            ),
            callback_button(
                locales::get_text(lang.as_str(), locales::LocaleKey::BtnAfkThresholdPlus, None),
                CallbackAction::Settings(SettingsAction::AfkThresholdStep { delta: 1 }),
            ),
        ],
        vec![
            callback_button(
                locales::get_text(lang.as_str(), locales::LocaleKey::BtnAfkCooldownMinus, None),
                CallbackAction::Settings(SettingsAction::AfkCooldownStep { delta: -5 }),
            ),
            callback_button(
                locales::get_text(lang.as_str(), locales::LocaleKey::BtnAfkCooldownPlus, None),
                CallbackAction::Settings(SettingsAction::AfkCooldownStep { delta: 5 }),
            ),
        ],
        vec![
            callback_button(
                locales::get_text(
                    lang.as_str(),
                    locales::LocaleKey::BtnAfkModeNone,
                    args!(
                        marker = if matches!(resolved.list_mode, AfkListMode::None) {
                            "?"
                        } else {
                            "??"
                        }
                    )
                    .as_ref(),
                ),
                CallbackAction::Settings(SettingsAction::AfkModeSet {
                    mode: AfkListMode::None,
                }),
            ),
            callback_button(
                locales::get_text(
                    lang.as_str(),
                    locales::LocaleKey::BtnAfkModeBlacklist,
                    args!(
                        marker = if matches!(resolved.list_mode, AfkListMode::Blacklist) {
                            "?"
                        } else {
                            "??"
                        }
                    )
                    .as_ref(),
                ),
                CallbackAction::Settings(SettingsAction::AfkModeSet {
                    mode: AfkListMode::Blacklist,
                }),
            ),
            callback_button(
                locales::get_text(
                    lang.as_str(),
                    locales::LocaleKey::BtnAfkModeWhitelist,
                    args!(
                        marker = if matches!(resolved.list_mode, AfkListMode::Whitelist) {
                            "?"
                        } else {
                            "??"
                        }
                    )
                    .as_ref(),
                ),
                CallbackAction::Settings(SettingsAction::AfkModeSet {
                    mode: AfkListMode::Whitelist,
                }),
            ),
        ],
        vec![callback_button(
            locales::get_text(
                lang.as_str(),
                locales::LocaleKey::BtnAfkManageBlacklist,
                None,
            ),
            CallbackAction::Settings(SettingsAction::AfkList {
                mode: AfkListMode::Blacklist,
                page: 0,
            }),
        )],
        vec![callback_button(
            locales::get_text(
                lang.as_str(),
                locales::LocaleKey::BtnAfkManageWhitelist,
                None,
            ),
            CallbackAction::Settings(SettingsAction::AfkList {
                mode: AfkListMode::Whitelist,
                page: 0,
            }),
        )],
        vec![callback_button(
            locales::get_text(
                lang.as_str(),
                locales::LocaleKey::BtnAfkManageOverrides,
                None,
            ),
            CallbackAction::Settings(SettingsAction::AfkOverrides { page: 0 }),
        )],
        vec![crate::adapters::tg::presenter::keyboards::back_button(
            lang,
            locales::LocaleKey::BtnBackNotif,
            CallbackAction::Settings(SettingsAction::NotifSelect),
        )],
    ]);
    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
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
    let user_settings = match tg_settings_service::load_settings(
        &state.db,
        telegram_id,
        state.config.general.default_lang,
    )
    .await
    {
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

            let settings = match tg_settings_service::load_settings(
                &state.db,
                telegram_id,
                state.config.general.default_lang,
            )
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
                match tg_settings_service::admin_sub_events_enabled(
                    &state.db,
                    telegram_id,
                    state.config.general.default_lang,
                )
                .await
                {
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
    let current = match tg_settings_service::admin_sub_events_enabled(
        &state.db,
        telegram_id,
        state.config.general.default_lang,
    )
    .await
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

    let settings = match tg_settings_service::load_settings(
        &state.db,
        telegram_id,
        state.config.general.default_lang,
    )
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
