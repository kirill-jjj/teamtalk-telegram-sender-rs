use crate::adapters::tg::callback_handlers::{admin, menu, mute, settings, subscriber, unsub};
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::notify_admin_error;
use crate::app::services::subscription as subscriptions_service;
use crate::app::services::user_settings as user_settings_service;
use crate::core::callbacks::CallbackAction;
use crate::core::types::{AdminErrorContext, LanguageCode};
use crate::infra::locales;
use std::str::FromStr;
use teloxide::prelude::*;
use teloxide::types::MaybeInaccessibleMessage;

pub async fn answer_callback(bot: Bot, q: CallbackQuery, state: AppState) -> ResponseResult<()> {
    let query_id = q.id.clone();
    let telegram_id = q.from.id.0 as i64;
    let callback_data_str = q.data.clone().unwrap_or_default();

    let db = &state.db;
    let config = &state.config;
    let default_lang =
        LanguageCode::from_str_or_default(&config.general.default_lang, LanguageCode::En);

    let _msg = match &q.message {
        Some(MaybeInaccessibleMessage::Regular(m)) => m,
        _ => return Ok(()),
    };

    let user_settings =
        match user_settings_service::get_or_create(db, telegram_id, default_lang).await {
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
                    AdminErrorContext::Callback,
                    &e.to_string(),
                    default_lang,
                )
                .await;
                bot.answer_callback_query(q.id)
                    .text(locales::get_text(default_lang.as_str(), "cmd-error", None))
                    .show_alert(true)
                    .await?;
                return Ok(());
            }
        };
    let lang = LanguageCode::from_str_or_default(&user_settings.language_code, default_lang);

    match subscriptions_service::is_subscribed(db, telegram_id).await {
        Ok(true) => {}
        Ok(false) => {
            bot.answer_callback_query(query_id)
                .text(locales::get_text(lang.as_str(), "cmd-not-subscribed", None))
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
                AdminErrorContext::Subscription,
                &e.to_string(),
                lang,
            )
            .await;
            bot.answer_callback_query(query_id)
                .text(locales::get_text(lang.as_str(), "cmd-error", None))
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
