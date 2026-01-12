use crate::adapters::tg::keyboards::confirm_cancel_keyboard;
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{answer_callback_empty, notify_admin_error};
use crate::core::callbacks::{CallbackAction, MenuAction, UnsubAction};
use crate::core::types::LanguageCode;
use crate::core::types::TtCommand;
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub async fn handle_menu(
    bot: Bot,
    q: CallbackQuery,
    state: AppState,
    action: MenuAction,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let msg = match q.message {
        Some(teloxide::types::MaybeInaccessibleMessage::Regular(m)) => m,
        _ => return Ok(()),
    };
    let chat_id = msg.chat.id;

    match action {
        MenuAction::Who => {
            if let Err(e) = state.tx_tt.send(TtCommand::Who {
                chat_id: chat_id.0,
                lang,
            }) {
                tracing::error!("Failed to send TT who command: {}", e);
                notify_admin_error(
                    &bot,
                    &state.config,
                    q.from.id.0 as i64,
                    "admin-error-context-tt-command",
                    &e.to_string(),
                    lang,
                )
                .await;
            }
            answer_callback_empty(&bot, &q.id).await?;
        }
        MenuAction::Help => {
            bot.send_message(chat_id, locales::get_text(lang.as_str(), "help-text", None))
                .parse_mode(ParseMode::Html)
                .await?;
            answer_callback_empty(&bot, &q.id).await?;
        }
        MenuAction::Unsub => {
            let text = locales::get_text(lang.as_str(), "unsub-confirm-text", None);
            let keyboard = confirm_cancel_keyboard(
                lang,
                "btn-yes",
                CallbackAction::Unsub(UnsubAction::Confirm),
                "btn-no",
                CallbackAction::Unsub(UnsubAction::Cancel),
            );

            bot.send_message(chat_id, text)
                .reply_markup(keyboard)
                .await?;

            answer_callback_empty(&bot, &q.id).await?;
        }
        MenuAction::Settings => {}
    }
    Ok(())
}
