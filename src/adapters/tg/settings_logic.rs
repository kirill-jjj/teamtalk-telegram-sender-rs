use crate::adapters::tg::keyboards::{
    back_btn, back_button, back_button_keyboard, callback_button, create_user_list_keyboard,
};
use crate::app::services::user_settings as user_settings_service;
use crate::args;
use crate::core::callbacks::{CallbackAction, MuteAction, SettingsAction};
use crate::core::types::{LanguageCode, MuteListMode, NotificationSetting, TtUsername};
use crate::infra::db::Database;
use crate::infra::locales;
use teamtalk::types::UserAccount;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardMarkup, ParseMode};

pub async fn send_main_settings(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let text = locales::get_text(lang.as_str(), "settings-title", None);
    let keyboard = main_settings_keyboard(lang);
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
            locales::get_text(lang.as_str(), "btn-lang", None),
            CallbackAction::Settings(SettingsAction::LangSelect),
        )],
        vec![callback_button(
            locales::get_text(lang.as_str(), "btn-sub-settings", None),
            CallbackAction::Settings(SettingsAction::SubSelect),
        )],
        vec![callback_button(
            locales::get_text(lang.as_str(), "btn-notif-settings", None),
            CallbackAction::Settings(SettingsAction::NotifSelect),
        )],
    ])
}

pub async fn send_sub_settings(
    bot: &Bot,
    msg: &Message,
    db: &Database,
    telegram_id: i64,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let settings =
        match user_settings_service::get_or_create(db, telegram_id, LanguageCode::En).await {
            Ok(s) => {
                tracing::debug!(
                    component = "ui",
                    telegram_id,
                    enabled = s.not_on_online_enabled,
                    "Fetched settings"
                );
                s
            }
            Err(e) => {
                tracing::error!(
                    telegram_id,
                    error = %e,
                    "Failed to get or create user"
                );
                bot.edit_message_text(
                    msg.chat.id,
                    msg.id,
                    locales::get_text(lang.as_str(), "cmd-error", None),
                )
                .await?;
                return Ok(());
            }
        };
    let current_notif =
        user_settings_service::parse_notification_setting(&settings.notification_settings);

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
        args!(marker = mk(NotificationSetting::JoinOff)).as_ref(),
    );
    let btn_leave = locales::get_text(
        lang.as_str(),
        "btn-sub-leave",
        args!(marker = mk(NotificationSetting::LeaveOff)).as_ref(),
    );
    let btn_none = locales::get_text(
        lang.as_str(),
        "btn-sub-none",
        args!(marker = mk(NotificationSetting::None)).as_ref(),
    );

    let mk_act = |val: NotificationSetting| {
        CallbackAction::Settings(SettingsAction::SubSet { setting: val })
    };

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![callback_button(btn_all, mk_act(NotificationSetting::All))],
        vec![callback_button(
            btn_join,
            mk_act(NotificationSetting::JoinOff),
        )],
        vec![callback_button(
            btn_leave,
            mk_act(NotificationSetting::LeaveOff),
        )],
        vec![callback_button(btn_none, mk_act(NotificationSetting::None))],
        vec![back_button(
            lang,
            "btn-back-settings",
            CallbackAction::Settings(SettingsAction::Main),
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
    let settings =
        match user_settings_service::get_or_create(db, telegram_id, LanguageCode::En).await {
            Ok(s) => {
                tracing::debug!(
                    component = "ui",
                    telegram_id,
                    enabled = s.not_on_online_enabled,
                    "Fetched settings"
                );
                s
            }
            Err(e) => {
                tracing::error!(
                    telegram_id,
                    error = %e,
                    "Failed to get or create user"
                );
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
        vec![callback_button(
            noon_text,
            CallbackAction::Settings(SettingsAction::NoonToggle),
        )],
        vec![callback_button(
            locales::get_text(lang.as_str(), "btn-mute-manage", None),
            CallbackAction::Settings(SettingsAction::MuteManage),
        )],
        vec![back_button(
            lang,
            "btn-back-settings",
            CallbackAction::Settings(SettingsAction::Main),
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
            callback_button(
                btn_bl_text,
                CallbackAction::Mute(MuteAction::ModeSet {
                    mode: MuteListMode::Blacklist,
                }),
            ),
            callback_button(
                btn_wl_text,
                CallbackAction::Mute(MuteAction::ModeSet {
                    mode: MuteListMode::Whitelist,
                }),
            ),
        ],
        vec![callback_button(
            btn_manage_text,
            CallbackAction::Mute(MuteAction::List { page: 0 }),
        )],
        vec![callback_button(
            locales::get_text(lang.as_str(), "btn-mute-server-list", None),
            CallbackAction::Mute(MuteAction::ServerList { page: 0 }),
        )],
        vec![back_button(
            lang,
            "btn-back-notif",
            CallbackAction::Settings(SettingsAction::NotifSelect),
        )],
    ]);

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub struct RenderMuteListArgs<'a> {
    pub bot: &'a Bot,
    pub msg: &'a Message,
    pub db: &'a Database,
    pub telegram_id: i64,
    pub lang: LanguageCode,
    pub accounts: &'a [UserAccount],
    pub page: usize,
    pub title_key: &'a str,
    pub guest_username: Option<&'a str>,
}

pub struct RenderMuteListStringsArgs<'a> {
    pub bot: &'a Bot,
    pub msg: &'a Message,
    pub lang: LanguageCode,
    pub items: &'a [String],
    pub page: usize,
    pub title_key: &'a str,
    pub guest_username: Option<&'a str>,
}

pub async fn render_mute_list(args: RenderMuteListArgs<'_>) -> ResponseResult<()> {
    let muted_users: Vec<String> = match args.db.get_muted_users_list(args.telegram_id).await {
        Ok(list) => list,
        Err(e) => {
            tracing::error!(
                telegram_id = args.telegram_id,
                error = %e,
                "Failed to load muted users"
            );
            Vec::new()
        }
    };
    let muted_set: std::collections::HashSet<_> = muted_users.into_iter().collect();

    let keyboard = create_user_list_keyboard(
        args.accounts,
        args.page,
        |acc| {
            let is_muted = muted_set.contains(&acc.username);
            let icon_key = if is_muted {
                "item-status-muted"
            } else {
                "item-status-unmuted"
            };

            let display_name = if Some(acc.username.as_str()) == args.guest_username {
                locales::get_text(args.lang.as_str(), "display-guest-account", None)
            } else {
                acc.username.clone()
            };

            let fmt_args = args!(name = display_name);
            let display_text = locales::get_text(args.lang.as_str(), icon_key, fmt_args.as_ref());
            (
                display_text,
                CallbackAction::Mute(MuteAction::ServerToggle {
                    username: TtUsername::new(acc.username.clone()),
                    page: args.page,
                }),
            )
        },
        |p| CallbackAction::Mute(MuteAction::ServerList { page: p }),
        Some(back_btn(
            args.lang,
            "btn-back-mute",
            CallbackAction::Settings(SettingsAction::MuteManage),
        )),
        args.lang,
    );

    let text = locales::get_text(args.lang.as_str(), args.title_key, None);
    args.bot
        .edit_message_text(args.msg.chat.id, args.msg.id, text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

pub async fn render_mute_list_strings(args: RenderMuteListStringsArgs<'_>) -> ResponseResult<()> {
    if args.items.is_empty() {
        let text = locales::get_text(args.lang.as_str(), "list-mute-empty", None);
        let keyboard = back_button_keyboard(
            args.lang,
            "btn-back-mute",
            CallbackAction::Settings(SettingsAction::MuteManage),
        );
        args.bot
            .edit_message_text(args.msg.chat.id, args.msg.id, text)
            .reply_markup(keyboard)
            .await?;
        return Ok(());
    }

    let mut sorted_items = args.items.to_vec();
    sorted_items.sort_by_key(|a| a.to_lowercase());

    let keyboard = create_user_list_keyboard(
        &sorted_items,
        args.page,
        |username| {
            let display_name = if Some(username.as_str()) == args.guest_username {
                locales::get_text(args.lang.as_str(), "display-guest-account", None)
            } else {
                username.clone()
            };

            let fmt_args = args!(name = display_name);
            let display_text =
                locales::get_text(args.lang.as_str(), "item-status-muted", fmt_args.as_ref());
            (
                display_text,
                CallbackAction::Mute(MuteAction::Toggle {
                    username: TtUsername::new(username.clone()),
                    page: args.page,
                }),
            )
        },
        |p| CallbackAction::Mute(MuteAction::List { page: p }),
        Some(back_btn(
            args.lang,
            "btn-back-mute",
            CallbackAction::Settings(SettingsAction::MuteManage),
        )),
        args.lang,
    );

    let text = locales::get_text(args.lang.as_str(), args.title_key, None);

    args.bot
        .edit_message_text(args.msg.chat.id, args.msg.id, text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}
