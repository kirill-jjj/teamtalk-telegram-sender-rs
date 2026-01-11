use crate::locales;
use crate::tg_bot::callbacks_types::UnsubAction;
use crate::tg_bot::state::AppState;
use teloxide::prelude::*;

pub async fn handle_unsub_action(
    bot: Bot,
    q: CallbackQuery,
    state: AppState,
    action: UnsubAction,
    lang: &str,
) -> ResponseResult<()> {
    let msg = match q.message {
        Some(teloxide::types::MaybeInaccessibleMessage::Regular(m)) => m,
        _ => return Ok(()),
    };
    let telegram_id = q.from.id.0 as i64;
    let db = &state.db;

    match action {
        UnsubAction::Confirm => {
            if let Err(e) = db.delete_user_profile(telegram_id).await {
                tracing::error!("Failed to unsubscribe user {}: {}", telegram_id, e);
                bot.answer_callback_query(q.id)
                    .text("Database error")
                    .await?;
                return Ok(());
            }
            bot.edit_message_text(
                msg.chat.id,
                msg.id,
                locales::get_text(lang, "cmd-success-unsub", None),
            )
            .await?;
            bot.answer_callback_query(q.id)
                .text(locales::get_text(lang, "cmd-success-unsub", None))
                .await?;
        }
        UnsubAction::Cancel => {
            bot.edit_message_text(
                msg.chat.id,
                msg.id,
                locales::get_text(lang, "unsub-cancelled", None),
            )
            .await?;
            bot.answer_callback_query(q.id).await?;
        }
    }
    Ok(())
}
