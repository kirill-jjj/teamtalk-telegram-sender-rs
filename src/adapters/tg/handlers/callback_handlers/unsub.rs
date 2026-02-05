use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{
    TgErrorReporter, answer_callback, answer_callback_empty, answer_cmd_error_callback,
    telegram_id_from_callback_query,
};
use crate::app::services::tg_basic as tg_basic_service;
use crate::core::callbacks::UnsubAction;
use crate::core::types::{AdminErrorContext, LanguageCode};
use crate::infra::locales;
use teloxide::prelude::*;

pub async fn handle_unsub_action(
    bot: Bot,
    q: CallbackQuery,
    state: &AppState,
    action: UnsubAction,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let Some(telegram_id) = telegram_id_from_callback_query(&q, "handle_unsub_action") else {
        return Ok(());
    };
    let Some(teloxide::types::MaybeInaccessibleMessage::Regular(msg)) = q.message else {
        return Ok(());
    };
    let db = &state.db;
    let errors = TgErrorReporter::new(&bot, &state.config, telegram_id, lang);

    match action {
        UnsubAction::Confirm => {
            if let Err(e) = tg_basic_service::unsubscribe(db, telegram_id).await {
                tracing::error!(
                    telegram_id = telegram_id.as_i64(),
                    error = %e,
                    "Failed to unsubscribe user"
                );
                errors
                    .notify(AdminErrorContext::Callback, &e.to_string())
                    .await;
                answer_cmd_error_callback(&bot, &q.id, lang, false).await?;
                return Ok(());
            }
            bot.edit_message_text(
                msg.chat.id,
                msg.id,
                locales::get_text(lang.as_str(), locales::LocaleKey::CmdSuccessUnsub, None),
            )
            .await?;
            answer_callback(
                &bot,
                &q.id,
                locales::get_text(lang.as_str(), locales::LocaleKey::CmdSuccessUnsub, None),
                false,
            )
            .await?;
        }
        UnsubAction::Cancel => {
            bot.edit_message_text(
                msg.chat.id,
                msg.id,
                locales::get_text(lang.as_str(), locales::LocaleKey::UnsubCancelled, None),
            )
            .await?;
            answer_callback_empty(&bot, &q.id).await?;
        }
    }
    Ok(())
}
