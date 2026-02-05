use crate::adapters::tg::presenter::keyboards::{back_button, callback_button};
use crate::args;
use crate::core::callbacks::{CallbackAction, SubAction};
use crate::core::types::{LanguageCode, MuteListMode, TelegramId};
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardMarkup;

pub async fn send_sub_mute_mode_menu(
    bot: &Bot,
    msg: &Message,
    lang: LanguageCode,
    target_id: TelegramId,
    return_page: usize,
) -> ResponseResult<()> {
    let args = args!(id = target_id.to_string());
    let text = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::SubModeTitle,
        args.as_ref(),
    );

    let bl_text = locales::get_text(lang.as_str(), locales::LocaleKey::ModeBlacklist, None);
    let wl_text = locales::get_text(lang.as_str(), locales::LocaleKey::ModeWhitelist, None);

    let mk_act = |mode: MuteListMode| {
        CallbackAction::Subscriber(SubAction::ModeSet {
            sub_id: target_id,
            page: return_page,
            mode,
        })
    };

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![callback_button(bl_text, mk_act(MuteListMode::Blacklist))],
        vec![callback_button(wl_text, mk_act(MuteListMode::Whitelist))],
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
