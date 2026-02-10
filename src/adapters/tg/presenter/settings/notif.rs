use crate::adapters::tg::presenter::keyboards::{back_button, callback_button};
use crate::args;
use crate::core::callbacks::{CallbackAction, SettingsAction};
use crate::core::types::LanguageCode;
use crate::infra::locales;
use crate::infra::locales::LocaleKey;
use teloxide_ng::prelude::*;
use teloxide_ng::types::{InlineKeyboardMarkup, ParseMode};

pub async fn send_notif_settings(
    bot: &Bot,
    msg: &Message,
    lang: LanguageCode,
    not_on_online_enabled: bool,
    admin_sub_events_enabled: Option<bool>,
) -> ResponseResult<()> {
    let status_text = if not_on_online_enabled {
        locales::get_text(lang.as_str(), locales::LocaleKey::StatusEnabled, None)
    } else {
        locales::get_text(lang.as_str(), locales::LocaleKey::StatusDisabled, None)
    };
    let noon_text = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::BtnNoon,
        args!(status = status_text).as_ref(),
    );

    let mut rows = vec![vec![callback_button(
        noon_text,
        CallbackAction::Settings(SettingsAction::NoonToggle),
    )]];

    if let Some(enabled) = admin_sub_events_enabled {
        let status_text = if enabled {
            locales::get_text(lang.as_str(), locales::LocaleKey::StatusEnabled, None)
        } else {
            locales::get_text(lang.as_str(), locales::LocaleKey::StatusDisabled, None)
        };
        let admin_sub_events_text = locales::get_text(
            lang.as_str(),
            locales::LocaleKey::BtnAdminSubEvents,
            args!(status = status_text).as_ref(),
        );
        rows.push(vec![callback_button(
            admin_sub_events_text,
            CallbackAction::Settings(SettingsAction::AdminSubEventsToggle),
        )]);
    }

    rows.extend([
        vec![callback_button(
            locales::get_text(lang.as_str(), locales::LocaleKey::BtnMuteManage, None),
            CallbackAction::Settings(SettingsAction::MuteManage),
        )],
        vec![back_button(
            lang,
            LocaleKey::BtnBackSettings,
            CallbackAction::Settings(SettingsAction::Main),
        )],
    ]);
    let keyboard = InlineKeyboardMarkup::new(rows);

    bot.edit_message_text(
        msg.chat.id,
        msg.id,
        locales::get_text(lang.as_str(), locales::LocaleKey::NotifSettingsTitle, None),
    )
    .reply_markup(keyboard)
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}
