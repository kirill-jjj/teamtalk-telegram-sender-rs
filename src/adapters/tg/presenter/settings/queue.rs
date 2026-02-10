use crate::adapters::tg::presenter::keyboards::{back_button, callback_button};
use crate::args;
use crate::core::callbacks::{CallbackAction, SettingsAction};
use crate::core::types::LanguageCode;
use crate::infra::locales;
use crate::infra::locales::LocaleKey;
use teloxide_ng::prelude::*;
use teloxide_ng::types::{InlineKeyboardMarkup, ParseMode};

pub enum QueueLinkStatus {
    Linked,
    Unlinked,
}

pub enum QueueToggleStatus {
    Enabled,
    Disabled,
}

pub enum QueueAdminStatus {
    Admin,
    User,
}

pub struct QueueSettingsView {
    pub link: QueueLinkStatus,
    pub user: QueueToggleStatus,
    pub global: QueueToggleStatus,
    pub admin: QueueAdminStatus,
}

pub async fn send_queue_settings(
    bot: &Bot,
    msg: &Message,
    lang: LanguageCode,
    view: QueueSettingsView,
) -> ResponseResult<()> {
    if matches!(view.link, QueueLinkStatus::Unlinked) {
        bot.edit_message_text(
            msg.chat.id,
            msg.id,
            locales::get_text(lang.as_str(), locales::LocaleKey::CmdQueueNoLink, None),
        )
        .await?;
        return Ok(());
    }

    let user_status = if matches!(view.user, QueueToggleStatus::Enabled) {
        locales::get_text(lang.as_str(), locales::LocaleKey::StatusEnabled, None)
    } else {
        locales::get_text(lang.as_str(), locales::LocaleKey::StatusDisabled, None)
    };
    let user_btn = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::BtnQueueUserToggle,
        args!(status = user_status).as_ref(),
    );

    let mut rows = vec![vec![callback_button(
        user_btn,
        CallbackAction::Settings(SettingsAction::QueueToggleUser),
    )]];

    if matches!(view.admin, QueueAdminStatus::Admin) {
        let global_status = if matches!(view.global, QueueToggleStatus::Enabled) {
            locales::get_text(lang.as_str(), locales::LocaleKey::StatusEnabled, None)
        } else {
            locales::get_text(lang.as_str(), locales::LocaleKey::StatusDisabled, None)
        };
        let global_btn = locales::get_text(
            lang.as_str(),
            locales::LocaleKey::BtnQueueGlobalToggle,
            args!(status = global_status).as_ref(),
        );
        rows.push(vec![callback_button(
            global_btn,
            CallbackAction::Settings(SettingsAction::QueueToggleGlobal),
        )]);
    }

    rows.push(vec![callback_button(
        locales::get_text(lang.as_str(), locales::LocaleKey::BtnQueueClear, None),
        CallbackAction::Settings(SettingsAction::QueueClearSelf),
    )]);

    if matches!(view.admin, QueueAdminStatus::Admin) {
        rows.push(vec![callback_button(
            locales::get_text(lang.as_str(), locales::LocaleKey::BtnQueueClearAll, None),
            CallbackAction::Settings(SettingsAction::QueueClearAll),
        )]);
    }

    rows.push(vec![back_button(
        lang,
        LocaleKey::BtnBackSettings,
        CallbackAction::Settings(SettingsAction::Main),
    )]);

    let keyboard = InlineKeyboardMarkup::new(rows);
    bot.edit_message_text(
        msg.chat.id,
        msg.id,
        locales::get_text(lang.as_str(), locales::LocaleKey::QueueSettingsTitle, None),
    )
    .reply_markup(keyboard)
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}
