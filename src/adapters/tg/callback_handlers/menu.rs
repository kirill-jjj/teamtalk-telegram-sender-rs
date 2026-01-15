use crate::adapters::tg::keyboards::confirm_cancel_keyboard;
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{answer_callback_empty, notify_admin_error};
use crate::core::callbacks::{CallbackAction, MenuAction, UnsubAction};
use crate::core::types::{AdminErrorContext, LanguageCode, TtCommand};
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
    let Some(teloxide::types::MaybeInaccessibleMessage::Regular(msg)) = q.message else {
        return Ok(());
    };
    let chat_id = msg.chat.id;

    match action {
        MenuAction::Who => {
            if let Err(e) = state.tx_tt.send(TtCommand::Who {
                chat_id: chat_id.0,
                lang,
            }) {
                tracing::error!(error = %e, "Failed to send TT who command");
                notify_admin_error(
                    &bot,
                    &state.config,
                    tg_user_id_i64(q.from.id.0),
                    AdminErrorContext::TtCommand,
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

fn tg_user_id_i64(user_id: u64) -> i64 {
    i64::try_from(user_id).unwrap_or(i64::MAX)
}
