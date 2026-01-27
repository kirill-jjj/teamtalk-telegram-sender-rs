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
    let telegram_id = tg_user_id_i64(q.from.id.0);
    let callback_data_str = q.data.clone().unwrap_or_default();

    let db = &state.db;
    let config = &state.config;
    let default_lang = config.general.default_lang;

    let Some(MaybeInaccessibleMessage::Regular(_msg)) = &q.message else {
        return Ok(());
    };

    let lang = load_user_lang(&bot, db, config, telegram_id, default_lang, q.id.clone()).await?;
    if !ensure_subscribed(&bot, db, config, telegram_id, lang, query_id.clone()).await? {
        return Ok(());
    }
    let action = parse_action(&callback_data_str);
    dispatch_action(bot, q, state, action, lang).await?;

    Ok(())
}

fn tg_user_id_i64(user_id: u64) -> i64 {
    i64::try_from(user_id).unwrap_or(i64::MAX)
}

async fn load_user_lang(
    bot: &Bot,
    db: &crate::infra::db::Database,
    config: &crate::bootstrap::config::Config,
    telegram_id: i64,
    default_lang: LanguageCode,
    query_id: teloxide::types::CallbackQueryId,
) -> ResponseResult<LanguageCode> {
    let user_settings =
        match user_settings_service::get_or_create(db, telegram_id, default_lang).await {
            Ok(settings) => settings,
            Err(e) => {
                tracing::error!(
                    telegram_id,
                    error = %e,
                    "Failed to get/create user in callback"
                );
                notify_admin_error(
                    bot,
                    config,
                    telegram_id,
                    AdminErrorContext::Callback,
                    &e.to_string(),
                    default_lang,
                )
                .await;
                bot.answer_callback_query(query_id)
                    .text(locales::get_text(default_lang.as_str(), "cmd-error", None))
                    .show_alert(true)
                    .await?;
                return Ok(default_lang);
            }
        };
    Ok(LanguageCode::from_str_or_default(
        &user_settings.language_code,
        default_lang,
    ))
}

async fn ensure_subscribed(
    bot: &Bot,
    db: &crate::infra::db::Database,
    config: &crate::bootstrap::config::Config,
    telegram_id: i64,
    lang: LanguageCode,
    query_id: teloxide::types::CallbackQueryId,
) -> ResponseResult<bool> {
    match subscriptions_service::is_subscribed(db, telegram_id).await {
        Ok(true) => Ok(true),
        Ok(false) => {
            bot.answer_callback_query(query_id)
                .text(locales::get_text(lang.as_str(), "cmd-not-subscribed", None))
                .show_alert(true)
                .await?;
            Ok(false)
        }
        Err(e) => {
            tracing::error!(
                telegram_id,
                error = %e,
                "Failed to check subscription"
            );
            notify_admin_error(
                bot,
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
            Ok(false)
        }
    }
}

fn parse_action(callback_data_str: &str) -> CallbackAction {
    match CallbackAction::from_str(callback_data_str) {
        Ok(action) => action,
        Err(e) => {
            tracing::warn!(
                callback_data = %callback_data_str,
                error = %e,
                "Unknown or legacy callback data"
            );
            CallbackAction::NoOp
        }
    }
}

async fn dispatch_action(
    bot: Bot,
    q: CallbackQuery,
    state: AppState,
    action: CallbackAction,
    lang: LanguageCode,
) -> ResponseResult<()> {
    match action {
        CallbackAction::Menu(menu_act) => menu::handle_menu(bot, q, state, menu_act, lang).await,
        CallbackAction::Admin(admin_act) => {
            admin::handle_admin(bot, q, state, admin_act, lang).await
        }
        CallbackAction::Settings(sett_act) => {
            settings::handle_settings(bot, q, state, sett_act, lang).await
        }
        CallbackAction::Subscriber(sub_act) => {
            subscriber::handle_subscriber_actions(bot, q, state, sub_act, lang).await
        }
        CallbackAction::Mute(mute_act) => mute::handle_mute(bot, q, state, mute_act, lang).await,
        CallbackAction::Unsub(unsub_act) => {
            unsub::handle_unsub_action(bot, q, state, unsub_act, lang).await
        }
        CallbackAction::NoOp => {
            bot.answer_callback_query(q.id).await?;
            Ok(())
        }
    }
}
