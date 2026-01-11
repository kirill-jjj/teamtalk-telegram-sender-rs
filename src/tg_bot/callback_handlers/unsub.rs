use crate::locales;
use crate::tg_bot::callbacks_types::UnsubAction;
use crate::tg_bot::state::AppState;
use crate::tg_bot::utils::{answer_callback, answer_callback_empty, notify_admin_error};
use crate::types::LanguageCode;
use teloxide::prelude::*;

pub async fn handle_unsub_action(
    bot: Bot,
    q: CallbackQuery,
    state: AppState,
    action: UnsubAction,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let msg = match q.message {
        Some(teloxide::types::MaybeInaccessibleMessage::Regular(m)) => m,
        _ => return Ok(()),
    };
    let telegram_id = q.from.id.0 as i64;
    let db = &state.db;
    let config = &state.config;

    match action {
        UnsubAction::Confirm => {
            if let Err(e) = db.delete_user_profile(telegram_id).await {
                tracing::error!("Failed to unsubscribe user {}: {}", telegram_id, e);
                notify_admin_error(
                    &bot,
                    config,
                    telegram_id,
                    "admin-error-context-callback",
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
