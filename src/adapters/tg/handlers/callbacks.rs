use crate::adapters::tg::handlers::callback_handlers::{
    admin, menu, mute, settings, subscriber, unsub,
};
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{notify_admin_error, telegram_id_from_user_id};
use crate::app::services::tg_callbacks as tg_callbacks_service;
use crate::core::callbacks::CallbackAction;
use crate::core::types::{AdminErrorContext, LanguageCode, TelegramId};
use crate::infra::locales;
use std::str::FromStr;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::MaybeInaccessibleMessage;

pub async fn answer_callback(
    bot: Bot,
    q: CallbackQuery,
    state: Arc<AppState>,
) -> ResponseResult<()> {
    let query_id = q.id.clone();
    let Some(telegram_id) = telegram_id_from_user_id(q.from.id.0, "answer_callback") else {
        return Ok(());
    };
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
    let action = match parse_action(&callback_data_str) {
        Ok(action) => action,
        Err(error) => {
            handle_invalid_callback_data(
                &bot,
                config,
                query_id,
                telegram_id,
                lang,
                callback_data_str.as_str(),
                &error.to_string(),
            )
            .await?;
            return Ok(());
        }
    };
    dispatch_action(bot, q, state.as_ref(), action, lang).await?;

    Ok(())
}

async fn load_user_lang(
    bot: &Bot,
    db: &crate::infra::db::Database,
    config: &crate::bootstrap::config::Config,
    telegram_id: TelegramId,
    default_lang: LanguageCode,
    query_id: teloxide::types::CallbackQueryId,
) -> ResponseResult<LanguageCode> {
    match tg_callbacks_service::load_user_lang(db, telegram_id, default_lang).await {
        Ok(lang) => Ok(lang),
        Err(err) => {
            let error = err.into_error();
            tracing::error!(
                telegram_id = telegram_id.as_i64(),
                error = %error,
                "Failed to get/create user in callback"
            );
            notify_admin_error(
                bot,
                config,
                telegram_id,
                AdminErrorContext::Callback,
                &error.to_string(),
                default_lang,
            )
            .await;
            bot.answer_callback_query(query_id)
                .text(locales::get_text(
                    default_lang.as_str(),
                    locales::LocaleKey::CmdError,
                    None,
                ))
                .show_alert(true)
                .await?;
            Ok(default_lang)
        }
    }
}

async fn ensure_subscribed(
    bot: &Bot,
    db: &crate::infra::db::Database,
    config: &crate::bootstrap::config::Config,
    telegram_id: TelegramId,
    lang: LanguageCode,
    query_id: teloxide::types::CallbackQueryId,
) -> ResponseResult<bool> {
    match tg_callbacks_service::ensure_subscribed(db, telegram_id).await {
        Ok(true) => Ok(true),
        Ok(false) => {
            bot.answer_callback_query(query_id)
                .text(locales::get_text(
                    lang.as_str(),
                    locales::LocaleKey::CmdNotSubscribed,
                    None,
                ))
                .show_alert(true)
                .await?;
            Ok(false)
        }
        Err(err) => {
            let error = err.into_error();
            tracing::error!(
                telegram_id = telegram_id.as_i64(),
                error = %error,
                "Failed to check subscription"
            );
            notify_admin_error(
                bot,
                config,
                telegram_id,
                AdminErrorContext::Subscription,
                &error.to_string(),
                lang,
            )
            .await;
            bot.answer_callback_query(query_id)
                .text(locales::get_text(
                    lang.as_str(),
                    locales::LocaleKey::CmdError,
                    None,
                ))
                .show_alert(true)
                .await?;
            Ok(false)
        }
    }
}

fn parse_action(callback_data_str: &str) -> Result<CallbackAction, anyhow::Error> {
    CallbackAction::from_str(callback_data_str)
}

async fn handle_invalid_callback_data(
    bot: &Bot,
    config: &crate::bootstrap::config::Config,
    query_id: teloxide::types::CallbackQueryId,
    telegram_id: TelegramId,
    lang: LanguageCode,
    callback_data: &str,
    error: &str,
) -> ResponseResult<()> {
    tracing::warn!(
        telegram_id = telegram_id.as_i64(),
        callback_data = %callback_data,
        error = %error,
        "Invalid callback data"
    );
    notify_admin_error(
        bot,
        config,
        telegram_id,
        AdminErrorContext::Callback,
        &format!("Invalid callback data: {error}; payload={callback_data}"),
        lang,
    )
    .await;
    bot.answer_callback_query(query_id)
        .text("Invalid button data. Please reopen the menu.")
        .show_alert(true)
        .await?;
    Ok(())
}

async fn dispatch_action(
    bot: Bot,
    q: CallbackQuery,
    state: &AppState,
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
