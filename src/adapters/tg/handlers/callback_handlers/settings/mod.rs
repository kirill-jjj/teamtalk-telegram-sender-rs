use crate::adapters::tg::presenter::settings::send_main_settings_edit;
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::telegram_id_from_callback_query;
use crate::app::services::tg_admin as tg_admin_service;
use crate::core::callbacks::SettingsAction;
use crate::core::types::LanguageCode;
use teloxide_ng::prelude::*;

mod lang;
mod mute;
mod notif;
mod queue;
mod sub;

#[allow(clippy::too_many_lines)]
pub async fn handle_settings(
    bot: Bot,
    q: CallbackQuery,
    state: &AppState,
    action: SettingsAction,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let Some(teloxide_ng::types::MaybeInaccessibleMessage::Regular(ref msg)) = q.message else {
        return Ok(());
    };
    let msg = msg.as_ref();
    let Some(telegram_id) = telegram_id_from_callback_query(&q, "handle_settings") else {
        return Ok(());
    };

    match action {
        SettingsAction::Main => {
            send_main_settings_edit(&bot, msg, lang).await?;
        }
        SettingsAction::LangSelect => {
            lang::handle_lang_select(&bot, msg, lang).await?;
        }
        SettingsAction::LangSet { lang: new_lang } => {
            lang::handle_lang_set(&bot, &q, state, msg, telegram_id, lang, new_lang).await?;
        }
        SettingsAction::SubSelect => {
            sub::handle_sub_select(&bot, msg, state, telegram_id, lang).await?;
        }
        SettingsAction::SubSet { setting } => {
            sub::handle_sub_set(&bot, &q, state, msg, telegram_id, lang, setting).await?;
        }
        SettingsAction::NotifSelect => {
            notif::handle_notif_select(&bot, msg, state, telegram_id, lang).await?;
        }
        SettingsAction::AfkMenu => {
            notif::handle_afk_menu(&bot, msg, state, telegram_id, lang).await?;
        }
        SettingsAction::NoonToggle => {
            notif::handle_noon_toggle(&bot, &q, state, msg, telegram_id, lang).await?;
        }
        SettingsAction::AdminSubEventsToggle => {
            notif::handle_admin_sub_events_toggle(&bot, &q, state, msg, telegram_id, lang).await?;
        }
        SettingsAction::MuteManage => {
            mute::handle_mute_manage(&bot, &q, state, msg, telegram_id, lang).await?;
        }
        SettingsAction::QueueMenu => {
            let is_admin = tg_admin_service::is_admin(&state.db, &state.config, telegram_id).await;
            queue::handle_queue_menu(&bot, msg, state, telegram_id, lang, is_admin).await?;
        }
        SettingsAction::QueueToggleUser => {
            queue::handle_queue_toggle_user(&bot, &q, state, msg, telegram_id, lang).await?;
        }
        SettingsAction::QueueToggleGlobal => {
            queue::handle_queue_toggle_global(&bot, &q, state, msg, telegram_id, lang).await?;
        }
        SettingsAction::QueueClearSelf => {
            queue::handle_queue_clear_self(&bot, &q, state, msg, telegram_id, lang).await?;
        }
        SettingsAction::QueueClearAll => {
            queue::handle_queue_clear_all(&bot, &q, state, msg, telegram_id, lang).await?;
        }
        SettingsAction::AfkToggle => {
            notif::handle_afk_toggle(&bot, &q, state, msg, telegram_id, lang).await?;
        }
        SettingsAction::AfkThresholdStep { delta } => {
            notif::handle_afk_threshold_step(&bot, &q, state, msg, telegram_id, lang, delta)
                .await?;
        }
        SettingsAction::AfkCooldownStep { delta } => {
            notif::handle_afk_cooldown_step(&bot, &q, state, msg, telegram_id, lang, delta).await?;
        }
        SettingsAction::AfkModeSet { mode } => {
            notif::handle_afk_mode_set(&bot, &q, state, msg, telegram_id, lang, mode).await?;
        }
        SettingsAction::AfkList { mode, page } => {
            notif::handle_afk_list(&bot, &q, state, msg, telegram_id, lang, mode, page).await?;
        }
        SettingsAction::AfkListToggle {
            mode,
            username,
            page,
        } => {
            notif::handle_afk_list_toggle(
                &bot,
                &q,
                state,
                msg,
                telegram_id,
                lang,
                mode,
                username,
                page,
            )
            .await?;
        }
        SettingsAction::AfkOverrides { page } => {
            notif::handle_afk_overrides(&bot, &q, state, msg, telegram_id, lang, page).await?;
        }
        SettingsAction::AfkOverrideDelete { username, page } => {
            notif::handle_afk_override_delete(
                &bot,
                &q,
                state,
                msg,
                telegram_id,
                lang,
                username,
                page,
            )
            .await?;
        }
        SettingsAction::AfkOverrideSetPreset {
            username,
            minutes,
            page,
        } => {
            notif::handle_afk_override_set_preset(
                &bot,
                &q,
                state,
                msg,
                telegram_id,
                lang,
                username,
                minutes,
                page,
            )
            .await?;
        }
    }
    Ok(())
}
