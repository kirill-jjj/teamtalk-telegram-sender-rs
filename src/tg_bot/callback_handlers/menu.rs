use crate::locales;
use crate::tg_bot::callbacks_types::{CallbackAction, MenuAction, UnsubAction};
use crate::tg_bot::state::AppState;
use crate::types::TtCommand;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};

pub async fn handle_menu(
    bot: Bot,
    q: CallbackQuery,
    state: AppState,
    action: MenuAction,
    lang: &str,
) -> ResponseResult<()> {
    let msg = match q.message {
        Some(teloxide::types::MaybeInaccessibleMessage::Regular(m)) => m,
        _ => return Ok(()),
    };
    let chat_id = msg.chat.id;

    match action {
        MenuAction::Who => {
            state
                .tx_tt
                .send(TtCommand::Who {
                    chat_id: chat_id.0,
                    lang: lang.to_string(),
                })
                .ok();
            bot.answer_callback_query(q.id).await?;
        }
        MenuAction::Help => {
            bot.send_message(chat_id, locales::get_text(lang, "help-text", None))
                .parse_mode(ParseMode::Html)
                .await?;
            bot.answer_callback_query(q.id).await?;
        }
        MenuAction::Unsub => {
            let text = locales::get_text(lang, "unsub-confirm-text", None);
            let keyboard = InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::callback(
                    locales::get_text(lang, "btn-yes", None),
                    CallbackAction::Unsub(UnsubAction::Confirm).to_string(),
                ),
                InlineKeyboardButton::callback(
                    locales::get_text(lang, "btn-no", None),
                    CallbackAction::Unsub(UnsubAction::Cancel).to_string(),
                ),
            ]]);

            bot.send_message(chat_id, text)
                .reply_markup(keyboard)
                .await?;

            bot.answer_callback_query(q.id).await?;
        }
        MenuAction::Settings => {}
    }
    Ok(())
}
