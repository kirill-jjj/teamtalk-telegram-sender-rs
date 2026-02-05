use crate::adapters::tg::presenter::keyboards::callback_button;
use crate::core::callbacks::{CallbackAction, SettingsAction};
use crate::core::types::LanguageCode;
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::types::{InlineKeyboardMarkup, ParseMode};

pub async fn send_main_settings(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    lang: LanguageCode,
    reply_to: Option<teloxide::types::MessageId>,
) -> ResponseResult<()> {
    let text = locales::get_text(lang.as_str(), locales::LocaleKey::SettingsTitle, None);
    let keyboard = main_settings_keyboard(lang);
    let req = bot
        .send_message(chat_id, text)
        .reply_markup(keyboard)
        .parse_mode(ParseMode::Html);
    if let Some(reply_to) = reply_to {
        req.reply_to(reply_to).await?;
    } else {
        req.await?;
    }
    Ok(())
}

pub async fn send_main_settings_edit(
    bot: &Bot,
    msg: &Message,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let text = locales::get_text(lang.as_str(), locales::LocaleKey::SettingsTitle, None);
    let keyboard = main_settings_keyboard(lang);
    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

fn main_settings_keyboard(lang: LanguageCode) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![callback_button(
            locales::get_text(lang.as_str(), locales::LocaleKey::BtnLang, None),
            CallbackAction::Settings(SettingsAction::LangSelect),
        )],
        vec![callback_button(
            locales::get_text(lang.as_str(), locales::LocaleKey::BtnSubSettings, None),
            CallbackAction::Settings(SettingsAction::SubSelect),
        )],
        vec![callback_button(
            locales::get_text(lang.as_str(), locales::LocaleKey::BtnNotifSettings, None),
            CallbackAction::Settings(SettingsAction::NotifSelect),
        )],
        vec![callback_button(
            locales::get_text(lang.as_str(), locales::LocaleKey::BtnQueueSettings, None),
            CallbackAction::Settings(SettingsAction::QueueMenu),
        )],
    ])
}
