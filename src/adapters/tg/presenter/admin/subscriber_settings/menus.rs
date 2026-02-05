use crate::adapters::tg::presenter::keyboards::{back_button, callback_button};
use crate::args;
use crate::core::callbacks::{CallbackAction, SubAction};
use crate::core::types::{LanguageCode, NotificationSetting, TelegramId};
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardMarkup;

pub async fn send_sub_lang_menu(
    bot: &Bot,
    msg: &Message,
    lang: LanguageCode,
    target_id: TelegramId,
    return_page: usize,
) -> ResponseResult<()> {
    let args = args!(id = target_id.to_string());
    let text = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::SubLangTitle,
        args.as_ref(),
    );

    let mk_btn = |lbl: &str, l_code: &str| {
        callback_button(
            lbl,
            CallbackAction::Subscriber(SubAction::LangSet {
                sub_id: target_id,
                page: return_page,
                lang: match l_code {
                    "ru" => LanguageCode::Ru,
                    _ => LanguageCode::En,
                },
            }),
        )
    };

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![mk_btn("🇷🇺 Русский", "ru")],
        vec![mk_btn("🇬🇧 English", "en")],
        vec![back_button(
            lang,
            locales::LocaleKey::BtnBackUserActions,
            CallbackAction::Subscriber(SubAction::Details {
                sub_id: target_id,
                page: return_page,
            }),
        )],
    ]);

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

pub async fn send_sub_notif_menu(
    bot: &Bot,
    msg: &Message,
    lang: LanguageCode,
    target_id: TelegramId,
    return_page: usize,
) -> ResponseResult<()> {
    let args = args!(id = target_id.to_string());
    let text = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::SubNotifTitle,
        args.as_ref(),
    );

    let marker_args = args!(marker = "");

    let btn_all = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::BtnSubAll,
        marker_args.as_ref(),
    );
    let btn_join = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::BtnSubJoin,
        marker_args.as_ref(),
    );
    let btn_leave = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::BtnSubLeave,
        marker_args.as_ref(),
    );
    let btn_none = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::BtnSubNone,
        marker_args.as_ref(),
    );

    let mk_act = |val: NotificationSetting| {
        CallbackAction::Subscriber(SubAction::NotifSet {
            sub_id: target_id,
            page: return_page,
            val,
        })
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
            locales::LocaleKey::BtnBackUserActions,
            CallbackAction::Subscriber(SubAction::Details {
                sub_id: target_id,
                page: return_page,
            }),
        )],
    ]);

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}
