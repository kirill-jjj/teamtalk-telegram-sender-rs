use crate::adapters::tg::presenter::keyboards::confirm_cancel_keyboard;
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{
    answer_callback_empty, telegram_id_from_callback_query, TgErrorReporter,
};
use crate::core::callbacks::{CallbackAction, MenuAction, UnsubAction};
use crate::core::types::{AdminErrorContext, LanguageCode, TtCommand};
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub async fn handle_menu(
    bot: Bot,
    q: CallbackQuery,
    state: &AppState,
    action: MenuAction,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let admin_id = telegram_id_from_callback_query(&q, "handle_menu");
    let Some(teloxide::types::MaybeInaccessibleMessage::Regular(msg)) = q.message else {
        return Ok(());
    };
    let chat_id = msg.chat.id;
    let errors = admin_id.map(|id| TgErrorReporter::new(&bot, &state.config, id, lang));

    match action {
        MenuAction::Who => {
            if let Err(e) = state
                .tx_tt
                .send(TtCommand::Who {
                    chat_id: crate::core::types::TgChatId::from(chat_id.0),
                    lang,
                    reply_to: None,
                })
                .await
            {
                tracing::error!(error = %e, "Failed to send TT who command");
                if let Some(errors) = &errors {
                    errors.notify(AdminErrorContext::TtCommand, &e.to_string()).await;
                }
            }
            answer_callback_empty(&bot, &q.id).await?;
        }
        MenuAction::Help => {
            bot.send_message(
                chat_id,
                locales::get_text(lang.as_str(), locales::LocaleKey::HelpText, None),
            )
            .parse_mode(ParseMode::Html)
            .await?;
            answer_callback_empty(&bot, &q.id).await?;
        }
        MenuAction::Unsub => {
            let text = locales::get_text(lang.as_str(), locales::LocaleKey::UnsubConfirmText, None);
            let keyboard = confirm_cancel_keyboard(
                lang,
                locales::LocaleKey::BtnYes,
                CallbackAction::Unsub(UnsubAction::Confirm),
                locales::LocaleKey::BtnNo,
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
