use crate::locales;
use crate::tg_bot::callback_handlers::{admin, menu, mute, settings, subscriber, unsub};
use crate::tg_bot::callbacks_types::CallbackAction;
use crate::tg_bot::state::AppState;
use crate::tg_bot::utils::notify_admin_error;
use std::str::FromStr;
use teloxide::prelude::*;
use teloxide::types::MaybeInaccessibleMessage;

pub async fn answer_callback(bot: Bot, q: CallbackQuery, state: AppState) -> ResponseResult<()> {
    let query_id = q.id.clone();
    let telegram_id = q.from.id.0 as i64;
    let callback_data_str = q.data.clone().unwrap_or_default();

    let db = &state.db;
    let config = &state.config;

    let _msg = match &q.message {
        Some(MaybeInaccessibleMessage::Regular(m)) => m,
        _ => return Ok(()),
    };

    let user_settings = match db
        .get_or_create_user(telegram_id, &config.general.default_lang)
        .await
    {
        Ok(settings) => settings,
        Err(e) => {
            tracing::error!(
                "Failed to get/create user {} in callback: {}",
                telegram_id,
                e
            );
            notify_admin_error(
                &bot,
                config,
                telegram_id,
                "admin-error-context-callback",
                &e.to_string(),
                &config.general.default_lang,
            )
            .await;
            bot.answer_callback_query(q.id)
                .text(locales::get_text(
                    &config.general.default_lang,
                    "cmd-error",
                    None,
                ))
                .show_alert(true)
                .await?;
            return Ok(());
        }
    };
    let lang = &user_settings.language_code;

    match db.is_subscribed(telegram_id).await {
        Ok(true) => {}
        Ok(false) => {
            bot.answer_callback_query(query_id)
                .text(locales::get_text(lang, "cmd-not-subscribed", None))
                .show_alert(true)
                .await?;
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Failed to check subscription for {}: {}", telegram_id, e);
            notify_admin_error(
                &bot,
                config,
                telegram_id,
                "admin-error-context-subscription",
                &e.to_string(),
                lang,
            )
            .await;
            bot.answer_callback_query(query_id)
                .text(locales::get_text(lang, "cmd-error", None))
                .show_alert(true)
                .await?;
            return Ok(());
        }
    }

    let action = match CallbackAction::from_str(&callback_data_str) {
        Ok(a) => a,
        Err(e) => {
            tracing::warn!(
                "Unknown or legacy callback data '{}': {}",
                callback_data_str,
                e
            );
            return Ok(());
        }
    };

    match action {
        CallbackAction::Menu(menu_act) => {
            menu::handle_menu(bot, q, state, menu_act, lang).await?;
        }
        CallbackAction::Admin(admin_act) => {
            admin::handle_admin(bot, q, state, admin_act, lang).await?;
        }
        CallbackAction::Settings(sett_act) => {
            settings::handle_settings(bot, q, state, sett_act, lang).await?;
        }
        CallbackAction::Subscriber(sub_act) => {
            subscriber::handle_subscriber_actions(bot, q, state, sub_act, lang).await?;
        }
        CallbackAction::Mute(mute_act) => {
            mute::handle_mute(bot, q, state, mute_act, lang).await?;
        }
        CallbackAction::Unsub(unsub_act) => {
            unsub::handle_unsub_action(bot, q, state, unsub_act, lang).await?;
        }
        CallbackAction::NoOp => {
            bot.answer_callback_query(q.id).await?;
        }
    }

    Ok(())
}
