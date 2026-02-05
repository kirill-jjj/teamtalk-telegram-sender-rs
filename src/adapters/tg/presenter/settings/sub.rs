use crate::adapters::tg::presenter::keyboards::{back_button, callback_button};
use crate::args;
use crate::core::callbacks::{CallbackAction, SettingsAction};
use crate::core::types::{LanguageCode, NotificationSetting};
use crate::infra::locales;
use crate::infra::locales::LocaleKey;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardMarkup, ParseMode};

pub async fn send_sub_settings(
    bot: &Bot,
    msg: &Message,
    lang: LanguageCode,
    current_notif: NotificationSetting,
) -> ResponseResult<()> {
    let check_icon = locales::get_text(lang.as_str(), locales::LocaleKey::IconCheckSimple, None);
    let mk = |ns: NotificationSetting| {
        if ns == current_notif {
            check_icon.clone()
        } else {
            String::new()
        }
    };

    let btn_all = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::BtnSubAll,
        args!(marker = mk(NotificationSetting::All)).as_ref(),
    );
    let btn_join = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::BtnSubJoin,
        args!(marker = mk(NotificationSetting::LeaveOff)).as_ref(),
    );
    let btn_leave = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::BtnSubLeave,
        args!(marker = mk(NotificationSetting::JoinOff)).as_ref(),
    );
    let btn_none = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::BtnSubNone,
        args!(marker = mk(NotificationSetting::None)).as_ref(),
    );

    let mk_act = |val: NotificationSetting| {
        CallbackAction::Settings(SettingsAction::SubSet { setting: val })
    };

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![callback_button(btn_all, mk_act(NotificationSetting::All))],
        vec![callback_button(
            btn_join,
            mk_act(NotificationSetting::LeaveOff),
        )],
        vec![callback_button(
            btn_leave,
            mk_act(NotificationSetting::JoinOff),
        )],
        vec![callback_button(btn_none, mk_act(NotificationSetting::None))],
        vec![back_button(
            lang,
            LocaleKey::BtnBackSettings,
            CallbackAction::Settings(SettingsAction::Main),
        )],
    ]);

    bot.edit_message_text(
        msg.chat.id,
        msg.id,
        locales::get_text(lang.as_str(), locales::LocaleKey::BtnSubSettings, None),
    )
    .reply_markup(keyboard)
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}
