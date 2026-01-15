use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{answer_callback, answer_callback_empty, notify_admin_error};
use crate::app::services::subscription as subscription_service;
use crate::core::callbacks::UnsubAction;
use crate::core::types::{AdminErrorContext, LanguageCode};
use crate::infra::locales;
use teloxide::prelude::*;

pub async fn handle_unsub_action(
    bot: Bot,
    q: CallbackQuery,
    state: AppState,
    action: UnsubAction,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let Some(teloxide::types::MaybeInaccessibleMessage::Regular(msg)) = q.message else {
        return Ok(());
    };
    let telegram_id = tg_user_id_i64(q.from.id.0);
    let db = &state.db;
    let config = &state.config;

    match action {
        UnsubAction::Confirm => {
            if let Err(e) = subscription_service::unsubscribe(db, telegram_id).await {
                tracing::error!(
                    telegram_id,
                    error = %e,
                    "Failed to unsubscribe user"
                );
                notify_admin_error(
                    &bot,
                    config,
                    telegram_id,
                    AdminErrorContext::Callback,
                    &e.to_string(),
                    lang,
                )
                .await;
                answer_callback(
                    &bot,
                    &q.id,
                    locales::get_text(lang.as_str(), "cmd-error", None),
                    false,
                )
                .await?;
                return Ok(());
            }
            bot.edit_message_text(
                msg.chat.id,
                msg.id,
                locales::get_text(lang.as_str(), "cmd-success-unsub", None),
            )
            .await?;
            answer_callback(
                &bot,
                &q.id,
                locales::get_text(lang.as_str(), "cmd-success-unsub", None),
                false,
            )
            .await?;
        }
        UnsubAction::Cancel => {
            bot.edit_message_text(
                msg.chat.id,
                msg.id,
                locales::get_text(lang.as_str(), "unsub-cancelled", None),
            )
            .await?;
            answer_callback_empty(&bot, &q.id).await?;
        }
    }
    Ok(())
}

fn tg_user_id_i64(user_id: u64) -> i64 {
    i64::try_from(user_id).unwrap_or(i64::MAX)
}
