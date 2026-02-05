use crate::adapters::tg::handlers::callback_handlers::{
    admin, menu, mute, settings, subscriber, unsub,
};
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::notify_admin_error;
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
    dispatch_action(bot, q, state.as_ref(), action, lang).await?;

    Ok(())
}

fn tg_user_id_i64(user_id: u64) -> TelegramId {
    TelegramId::from(i64::try_from(user_id).unwrap_or(i64::MAX))
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
