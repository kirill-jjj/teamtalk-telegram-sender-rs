use crate::args;
use crate::db::Database;
use crate::locales;
use crate::tg_bot::callbacks_types::{CallbackAction, MuteAction, SettingsAction};
use crate::tg_bot::keyboards::create_user_list_keyboard;
use crate::types::NotificationSetting;
use teamtalk::types::UserAccount;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};

pub async fn send_main_settings(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    lang: &str,
) -> ResponseResult<()> {
    let text = locales::get_text(lang, "settings-title", None);
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang, "btn-lang", None),
            CallbackAction::Settings(SettingsAction::LangSelect).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang, "btn-sub-settings", None),
            CallbackAction::Settings(SettingsAction::SubSelect).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang, "btn-notif-settings", None),
            CallbackAction::Settings(SettingsAction::NotifSelect).to_string(),
        )],
    ]);
    bot.send_message(chat_id, text)
        .reply_markup(keyboard)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn send_main_settings_edit(bot: &Bot, msg: &Message, lang: &str) -> ResponseResult<()> {
    let text = locales::get_text(lang, "settings-title", None);
    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang, "btn-lang", None),
            CallbackAction::Settings(SettingsAction::LangSelect).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang, "btn-sub-settings", None),
            CallbackAction::Settings(SettingsAction::SubSelect).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang, "btn-notif-settings", None),
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
    lang: &str,
) -> ResponseResult<()> {
    let settings = match db.get_or_create_user(telegram_id, "en").await {
        Ok(s) => {
            log::debug!(
                "[UI] Fetched settings for {}: enabled={}",
                telegram_id,
                s.not_on_online_enabled
            );
            s
        }
        Err(e) => {
            log::error!("Failed to get or create user {}: {}", telegram_id, e);
            bot.edit_message_text(
                msg.chat.id,
                msg.id,
                locales::get_text(lang, "cmd-error", None),
            )
            .await?;
            return Ok(());
        }
    };
    let current_notif = NotificationSetting::from(settings.notification_settings.as_str());

    let check_icon = locales::get_text(lang, "icon-check-simple", None);
    let mk = |ns: NotificationSetting| {
        if ns == current_notif {
            check_icon.clone()
        } else {
            "".to_string()
        }
    };

    let btn_all = locales::get_text(
        lang,
        "btn-sub-all",
        args!(marker = mk(NotificationSetting::All)).as_ref(),
    );
    let btn_join = locales::get_text(
        lang,
        "btn-sub-join",
        args!(marker = mk(NotificationSetting::LeaveOff)).as_ref(),
    );
    let btn_leave = locales::get_text(
        lang,
        "btn-sub-leave",
        args!(marker = mk(NotificationSetting::JoinOff)).as_ref(),
    );
    let btn_none = locales::get_text(
        lang,
        "btn-sub-none",
        args!(marker = mk(NotificationSetting::None)).as_ref(),
    );

    let mk_act = |val: &str| {
        CallbackAction::Settings(SettingsAction::SubSet {
            setting: val.to_string(),
        })
        .to_string()
    };

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(btn_all, mk_act("all"))],
        vec![InlineKeyboardButton::callback(
            btn_join,
            mk_act("leave_off"),
        )],
        vec![InlineKeyboardButton::callback(
            btn_leave,
            mk_act("join_off"),
        )],
        vec![InlineKeyboardButton::callback(btn_none, mk_act("none"))],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang, "btn-back-settings", None),
            CallbackAction::Settings(SettingsAction::Main).to_string(),
        )],
    ]);

    bot.edit_message_text(
        msg.chat.id,
        msg.id,
        locales::get_text(lang, "btn-sub-settings", None),
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
    lang: &str,
) -> ResponseResult<()> {
    let settings = match db.get_or_create_user(telegram_id, "en").await {
        Ok(s) => {
            log::debug!(
                "[UI] Fetched settings for {}: enabled={}",
                telegram_id,
                s.not_on_online_enabled
            );
            s
        }
        Err(e) => {
            log::error!("Failed to get or create user {}: {}", telegram_id, e);
            bot.edit_message_text(
                msg.chat.id,
                msg.id,
                locales::get_text(lang, "cmd-error", None),
            )
            .await?;
            return Ok(());
        }
    };
    let status_text = if settings.not_on_online_enabled {
        locales::get_text(lang, "status-enabled", None)
    } else {
        locales::get_text(lang, "status-disabled", None)
    };
    let noon_text = locales::get_text(lang, "btn-noon", args!(status = status_text).as_ref());

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            noon_text,
            CallbackAction::Settings(SettingsAction::NoonToggle).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang, "btn-mute-manage", None),
            CallbackAction::Settings(SettingsAction::MuteManage).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang, "btn-back-settings", None),
            CallbackAction::Settings(SettingsAction::Main).to_string(),
        )],
    ]);

    bot.edit_message_text(
        msg.chat.id,
        msg.id,
        locales::get_text(lang, "notif-settings-title", None),
    )
    .reply_markup(keyboard)
    .parse_mode(ParseMode::Html)
    .await?;
    Ok(())
}

pub async fn send_mute_menu(
    bot: &Bot,
    msg: &Message,
    lang: &str,
    current_mode: &str,
) -> ResponseResult<()> {
    let mode_desc_key = if current_mode == "blacklist" {
        "mute-mode-blacklist"
    } else {
        "mute-mode-whitelist"
    };
    let mode_desc = locales::get_text(lang, mode_desc_key, None);
    let args = args!(mode_desc = mode_desc);
    let text = locales::get_text(lang, "mute-title", args.as_ref());

    let icon_checked = locales::get_text(lang, "icon-checked", None);
    let icon_unchecked = locales::get_text(lang, "icon-unchecked", None);

    let bl_marker = if current_mode == "blacklist" {
        &icon_checked
    } else {
        &icon_unchecked
    };
    let wl_marker = if current_mode == "whitelist" {
        &icon_checked
    } else {
        &icon_unchecked
    };

    let btn_bl_text = locales::get_text(
        lang,
        "btn-mode-blacklist",
        args!(marker = bl_marker).as_ref(),
    );
    let btn_wl_text = locales::get_text(
        lang,
        "btn-mode-whitelist",
        args!(marker = wl_marker).as_ref(),
    );

    let current_mode_display = if current_mode == "blacklist" {
        locales::get_text(lang, "mode-blacklist", None)
    } else {
        locales::get_text(lang, "mode-whitelist", None)
    };

    let btn_manage_text = locales::get_text(
        lang,
        "btn-manage-list",
        args!(mode = current_mode_display).as_ref(),
    );

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                btn_bl_text,
                CallbackAction::Mute(MuteAction::ModeSet {
                    mode: "blacklist".to_string(),
                })
                .to_string(),
            ),
            InlineKeyboardButton::callback(
                btn_wl_text,
                CallbackAction::Mute(MuteAction::ModeSet {
                    mode: "whitelist".to_string(),
                })
                .to_string(),
            ),
        ],
        vec![InlineKeyboardButton::callback(
            btn_manage_text,
            CallbackAction::Mute(MuteAction::List { page: 0 }).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang, "btn-mute-server-list", None),
            CallbackAction::Mute(MuteAction::ServerList { page: 0 }).to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang, "btn-back-notif", None),
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
    lang: &str,
    accounts: &[UserAccount],
    page: usize,
    title_key: &str,
    guest_username: Option<&str>,
) -> ResponseResult<()> {
    let muted_users: Vec<String> = db
        .get_muted_users_list(telegram_id)
        .await
        .unwrap_or_default();
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
                locales::get_text(lang, "display-guest-account", None)
            } else {
                acc.username.clone()
            };

            let args = args!(name = display_name);
            let display_text = locales::get_text(lang, icon_key, args.as_ref());
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
            locales::get_text(lang, "btn-back-mute", None),
            CallbackAction::Settings(SettingsAction::MuteManage),
        )),
        lang,
    );

    let text = locales::get_text(lang, title_key, None);
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
    lang: &str,
    items: &[String],
    page: usize,
    _is_server_list: bool,
    title_key: &str,
    guest_username: Option<&str>,
) -> ResponseResult<()> {
    if items.is_empty() {
        let text = locales::get_text(lang, "list-mute-empty", None);
        let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
            locales::get_text(lang, "btn-back-mute", None),
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
                locales::get_text(lang, "display-guest-account", None)
            } else {
                username.clone()
            };

            let args = args!(name = display_name);
            let display_text = locales::get_text(lang, "item-status-muted", args.as_ref());
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
            locales::get_text(lang, "btn-back-mute", None),
            CallbackAction::Settings(SettingsAction::MuteManage),
        )),
        lang,
    );

    let user_name = format!("{}", _telegram_id);
    let args = args!(name = user_name);
    let text = locales::get_text(lang, title_key, args.as_ref());

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}
