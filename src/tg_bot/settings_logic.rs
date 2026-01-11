use crate::args;
use crate::db::Database;
use crate::locales;
use crate::tg_bot::callbacks_types::{CallbackAction, MuteAction, SettingsAction};
use crate::tg_bot::keyboards::create_user_list_keyboard;
use crate::types::{LanguageCode, MuteListMode, NotificationSetting};
use teamtalk::types::UserAccount;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};

pub async fn send_main_settings(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let text = locales::get_text(lang.as_str(), "settings-title", None);
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-lang", None),
            CallbackAction::Settings(SettingsAction::LangSelect).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-sub-settings", None),
            CallbackAction::Settings(SettingsAction::SubSelect).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-notif-settings", None),
            CallbackAction::Settings(SettingsAction::NotifSelect).to_string(),
        )],
    ]);
    bot.send_message(chat_id, text)
        .reply_markup(keyboard)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn send_main_settings_edit(
    bot: &Bot,
    msg: &Message,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let text = locales::get_text(lang.as_str(), "settings-title", None);
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-lang", None),
            CallbackAction::Settings(SettingsAction::LangSelect).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-sub-settings", None),
            CallbackAction::Settings(SettingsAction::SubSelect).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-notif-settings", None),
            CallbackAction::Settings(SettingsAction::NotifSelect).to_string(),
        )],
    ]);
    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn send_sub_settings(
    bot: &Bot,
    msg: &Message,
    db: &Database,
    telegram_id: i64,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let settings = match db.get_or_create_user(telegram_id, LanguageCode::En).await {
        Ok(s) => {
            tracing::debug!(
                "[UI] Fetched settings for {}: enabled={}",
                telegram_id,
                s.not_on_online_enabled
            );
            s
        }
        Err(e) => {
            tracing::error!("Failed to get or create user {}: {}", telegram_id, e);
            bot.edit_message_text(
                msg.chat.id,
                msg.id,
                locales::get_text(lang.as_str(), "cmd-error", None),
            )
            .await?;
            return Ok(());
        }
    };
    let current_notif = NotificationSetting::try_from(settings.notification_settings.as_str())
        .unwrap_or(NotificationSetting::All);

    let check_icon = locales::get_text(lang.as_str(), "icon-check-simple", None);
    let mk = |ns: NotificationSetting| {
        if ns == current_notif {
            check_icon.clone()
        } else {
            "".to_string()
        }
    };

    let btn_all = locales::get_text(
        lang.as_str(),
        "btn-sub-all",
        args!(marker = mk(NotificationSetting::All)).as_ref(),
    );
    let btn_join = locales::get_text(
        lang.as_str(),
        "btn-sub-join",
        args!(marker = mk(NotificationSetting::LeaveOff)).as_ref(),
    );
    let btn_leave = locales::get_text(
        lang.as_str(),
        "btn-sub-leave",
        args!(marker = mk(NotificationSetting::JoinOff)).as_ref(),
    );
    let btn_none = locales::get_text(
        lang.as_str(),
        "btn-sub-none",
        args!(marker = mk(NotificationSetting::None)).as_ref(),
    );

    let mk_act = |val: NotificationSetting| {
        CallbackAction::Settings(SettingsAction::SubSet { setting: val }).to_string()
    };

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            btn_all,
            mk_act(NotificationSetting::All),
        )],
        vec![InlineKeyboardButton::callback(
            btn_join,
            mk_act(NotificationSetting::LeaveOff),
        )],
        vec![InlineKeyboardButton::callback(
            btn_leave,
            mk_act(NotificationSetting::JoinOff),
        )],
        vec![InlineKeyboardButton::callback(
            btn_none,
            mk_act(NotificationSetting::None),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-back-settings", None),
            CallbackAction::Settings(SettingsAction::Main).to_string(),
        )],
    ]);

    bot.edit_message_text(
        msg.chat.id,
        msg.id,
        locales::get_text(lang.as_str(), "btn-sub-settings", None),
    )
    .reply_markup(keyboard)
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

pub async fn send_notif_settings(
    bot: &Bot,
    msg: &Message,
    db: &Database,
    telegram_id: i64,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let settings = match db.get_or_create_user(telegram_id, LanguageCode::En).await {
        Ok(s) => {
            tracing::debug!(
                "[UI] Fetched settings for {}: enabled={}",
                telegram_id,
                s.not_on_online_enabled
            );
            s
        }
        Err(e) => {
            tracing::error!("Failed to get or create user {}: {}", telegram_id, e);
            bot.edit_message_text(
                msg.chat.id,
                msg.id,
                locales::get_text(lang.as_str(), "cmd-error", None),
            )
            .await?;
            return Ok(());
        }
    };
    let status_text = if settings.not_on_online_enabled {
        locales::get_text(lang.as_str(), "status-enabled", None)
    } else {
        locales::get_text(lang.as_str(), "status-disabled", None)
    };
    let noon_text = locales::get_text(
        lang.as_str(),
        "btn-noon",
        args!(status = status_text).as_ref(),
    );

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            noon_text,
            CallbackAction::Settings(SettingsAction::NoonToggle).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-mute-manage", None),
            CallbackAction::Settings(SettingsAction::MuteManage).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-back-settings", None),
            CallbackAction::Settings(SettingsAction::Main).to_string(),
        )],
    ]);

    bot.edit_message_text(
        msg.chat.id,
        msg.id,
        locales::get_text(lang.as_str(), "notif-settings-title", None),
    )
    .reply_markup(keyboard)
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

pub async fn send_mute_menu(
    bot: &Bot,
    msg: &Message,
    lang: LanguageCode,
    current_mode: MuteListMode,
) -> ResponseResult<()> {
    let mode_desc_key = match current_mode {
        MuteListMode::Blacklist => "mute-mode-blacklist",
        MuteListMode::Whitelist => "mute-mode-whitelist",
    };
    let mode_desc = locales::get_text(lang.as_str(), mode_desc_key, None);
    let args = args!(mode_desc = mode_desc);
    let text = locales::get_text(lang.as_str(), "mute-title", args.as_ref());

    let icon_checked = locales::get_text(lang.as_str(), "icon-checked", None);
    let icon_unchecked = locales::get_text(lang.as_str(), "icon-unchecked", None);

    let bl_marker = if current_mode == MuteListMode::Blacklist {
        &icon_checked
    } else {
        &icon_unchecked
    };
    let wl_marker = if current_mode == MuteListMode::Whitelist {
        &icon_checked
    } else {
        &icon_unchecked
    };

    let btn_bl_text = locales::get_text(
        lang.as_str(),
        "btn-mode-blacklist",
        args!(marker = bl_marker).as_ref(),
    );
    let btn_wl_text = locales::get_text(
        lang.as_str(),
        "btn-mode-whitelist",
        args!(marker = wl_marker).as_ref(),
    );

    let current_mode_display = match current_mode {
        MuteListMode::Blacklist => locales::get_text(lang.as_str(), "mode-blacklist", None),
        MuteListMode::Whitelist => locales::get_text(lang.as_str(), "mode-whitelist", None),
    };

    let btn_manage_text = locales::get_text(
        lang.as_str(),
        "btn-manage-list",
        args!(mode = current_mode_display).as_ref(),
    );

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                btn_bl_text,
                CallbackAction::Mute(MuteAction::ModeSet {
                    mode: MuteListMode::Blacklist,
                })
                .to_string(),
            ),
            InlineKeyboardButton::callback(
                btn_wl_text,
                CallbackAction::Mute(MuteAction::ModeSet {
                    mode: MuteListMode::Whitelist,
                })
                .to_string(),
            ),
        ],
        vec![InlineKeyboardButton::callback(
            btn_manage_text,
            CallbackAction::Mute(MuteAction::List { page: 0 }).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-mute-server-list", None),
            CallbackAction::Mute(MuteAction::ServerList { page: 0 }).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-back-notif", None),
            CallbackAction::Settings(SettingsAction::NotifSelect).to_string(),
        )],
    ]);

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn render_mute_list(
    bot: &Bot,
    msg: &Message,
    db: &Database,
    telegram_id: i64,
    lang: LanguageCode,
    accounts: &[UserAccount],
    page: usize,
    title_key: &str,
    guest_username: Option<&str>,
) -> ResponseResult<()> {
    let muted_users: Vec<String> = match db.get_muted_users_list(telegram_id).await {
        Ok(list) => list,
        Err(e) => {
            tracing::error!("Failed to load muted users for {}: {}", telegram_id, e);
            Vec::new()
        }
    };
    let muted_set: std::collections::HashSet<_> = muted_users.into_iter().collect();

    let keyboard = create_user_list_keyboard(
        accounts,
        page,
        |acc| {
            let is_muted = muted_set.contains(&acc.username);
            let icon_key = if is_muted {
                "item-status-muted"
            } else {
                "item-status-unmuted"
            };

            let display_name = if Some(acc.username.as_str()) == guest_username {
                locales::get_text(lang.as_str(), "display-guest-account", None)
            } else {
                acc.username.clone()
            };

            let args = args!(name = display_name);
            let display_text = locales::get_text(lang.as_str(), icon_key, args.as_ref());
            (
                display_text,
                CallbackAction::Mute(MuteAction::ServerToggle {
                    username: acc.username.clone(),
                    page,
                }),
            )
        },
        |p| CallbackAction::Mute(MuteAction::ServerList { page: p }),
        Some((
            locales::get_text(lang.as_str(), "btn-back-mute", None),
            CallbackAction::Settings(SettingsAction::MuteManage),
        )),
        lang,
    );

    let text = locales::get_text(lang.as_str(), title_key, None);
    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn render_mute_list_strings(
    bot: &Bot,
    msg: &Message,
    _telegram_id: i64,
    lang: LanguageCode,
    items: &[String],
    page: usize,
    _is_server_list: bool,
    title_key: &str,
    guest_username: Option<&str>,
) -> ResponseResult<()> {
    if items.is_empty() {
        let text = locales::get_text(lang.as_str(), "list-mute-empty", None);
        let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
            locales::get_text(lang.as_str(), "btn-back-mute", None),
            CallbackAction::Settings(SettingsAction::MuteManage).to_string(),
        )]]);
        bot.edit_message_text(msg.chat.id, msg.id, text)
            .reply_markup(keyboard)
            .await?;
        return Ok(());
    }

    let mut sorted_items = items.to_vec();
    sorted_items.sort_by_key(|a| a.to_lowercase());

    let keyboard = create_user_list_keyboard(
        &sorted_items,
        page,
        |username| {
            let display_name = if Some(username.as_str()) == guest_username {
                locales::get_text(lang.as_str(), "display-guest-account", None)
            } else {
                username.clone()
            };

            let args = args!(name = display_name);
            let display_text = locales::get_text(lang.as_str(), "item-status-muted", args.as_ref());
            (
                display_text,
                CallbackAction::Mute(MuteAction::Toggle {
                    username: username.clone(),
                    page,
                }),
            )
        },
        |p| CallbackAction::Mute(MuteAction::List { page: p }),
        Some((
            locales::get_text(lang.as_str(), "btn-back-mute", None),
            CallbackAction::Settings(SettingsAction::MuteManage),
        )),
        lang,
    );

    let user_name = format!("{}", _telegram_id);
    let args = args!(name = user_name);
    let text = locales::get_text(lang.as_str(), title_key, args.as_ref());

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}
