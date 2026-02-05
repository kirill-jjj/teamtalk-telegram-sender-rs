use crate::adapters::tg::presenter::settings::send_main_settings_edit;
use crate::adapters::tg::state::AppState;
use crate::app::services::tg_admin as tg_admin_service;
use crate::core::callbacks::SettingsAction;
use crate::core::types::{LanguageCode, TelegramId};
use teloxide::prelude::*;

mod lang;
mod mute;
mod notif;
mod queue;
mod sub;

pub async fn handle_settings(
    bot: Bot,
    q: CallbackQuery,
    state: &AppState,
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
        SettingsAction::NoonToggle => {
            notif::handle_noon_toggle(&bot, &q, state, msg, telegram_id, lang).await?;
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
    }
    Ok(())
}

fn tg_user_id_i64(user_id: u64) -> TelegramId {
    TelegramId::from(i64::try_from(user_id).unwrap_or(i64::MAX))
}
