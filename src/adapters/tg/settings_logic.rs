use crate::adapters::tg::keyboards::{
    back_btn, back_button, back_button_keyboard, callback_button, create_user_list_keyboard,
};
use crate::adapters::tg::search::append_search_hint;
use crate::app::services::reply_queue as reply_queue_service;
use crate::app::services::user_settings as user_settings_service;
use crate::args;
use crate::core::callbacks::{CallbackAction, MuteAction, SettingsAction};
use crate::core::types::TelegramId;
use crate::core::types::{LanguageCode, MuteListMode, NotificationSetting, TtUsername};
use crate::infra::db::Database;
use crate::infra::locales;
use crate::infra::locales::LocaleKey;
use teamtalk::types::UserAccount;
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

pub async fn send_sub_settings(
    bot: &Bot,
    msg: &Message,
    db: &Database,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let settings =
        match user_settings_service::get_or_create(db, telegram_id, LanguageCode::En).await {
            Ok(s) => {
                tracing::debug!(
                    component = "ui",
                    telegram_id = telegram_id.as_i64(),
                    enabled = s.not_on_online_enabled,
                    "Fetched settings"
                );
                s
            }
            Err(e) => {
                tracing::error!(
                    telegram_id = telegram_id.as_i64(),
                    error = %e,
                    "Failed to get or create user"
                );
                bot.edit_message_text(
                    msg.chat.id,
                    msg.id,
                    locales::get_text(lang.as_str(), locales::LocaleKey::CmdError, None),
                )
                .await?;
                return Ok(());
            }
        };
    let current_notif = settings.notification_settings;

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

pub async fn send_notif_settings(
    bot: &Bot,
    msg: &Message,
    db: &Database,
    telegram_id: TelegramId,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let settings =
        match user_settings_service::get_or_create(db, telegram_id, LanguageCode::En).await {
            Ok(s) => {
                tracing::debug!(
                    component = "ui",
                    telegram_id = telegram_id.as_i64(),
                    enabled = s.not_on_online_enabled,
                    "Fetched settings"
                );
                s
            }
            Err(e) => {
                tracing::error!(
                    telegram_id = telegram_id.as_i64(),
                    error = %e,
                    "Failed to get or create user"
                );
                bot.edit_message_text(
                    msg.chat.id,
                    msg.id,
                    locales::get_text(lang.as_str(), locales::LocaleKey::CmdError, None),
                )
                .await?;
                return Ok(());
            }
        };
    let status_text = if settings.not_on_online_enabled {
        locales::get_text(lang.as_str(), locales::LocaleKey::StatusEnabled, None)
    } else {
        locales::get_text(lang.as_str(), locales::LocaleKey::StatusDisabled, None)
    };
    let noon_text = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::BtnNoon,
        args!(status = status_text).as_ref(),
    );

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![callback_button(
            noon_text,
            CallbackAction::Settings(SettingsAction::NoonToggle),
        )],
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

pub async fn send_queue_settings(
    bot: &Bot,
    msg: &Message,
    db: &Database,
    telegram_id: TelegramId,
    lang: LanguageCode,
    is_admin: bool,
) -> ResponseResult<()> {
    let settings = match user_settings_service::get_or_create(db, telegram_id, LanguageCode::En)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(telegram_id = telegram_id.as_i64(), error = %e, "Failed to get or create user");
            bot.edit_message_text(
                msg.chat.id,
                msg.id,
                locales::get_text(lang.as_str(), locales::LocaleKey::CmdError, None),
            )
            .await?;
            return Ok(());
        }
    };

    if settings.teamtalk_username.is_none() {
        bot.edit_message_text(
            msg.chat.id,
            msg.id,
            locales::get_text(lang.as_str(), locales::LocaleKey::CmdQueueNoLink, None),
        )
        .await?;
        return Ok(());
    }

    let user_status = if settings.reply_queue_enabled {
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

    if is_admin {
        let global_enabled = reply_queue_service::get_reply_queue_global_enabled(db)
            .await
            .unwrap_or(false);
        let global_status = if global_enabled {
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

    if is_admin {
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

const fn mute_mode_desc_key(current_mode: &MuteListMode) -> LocaleKey {
    match current_mode {
        MuteListMode::Blacklist => LocaleKey::MuteModeBlacklist,
        MuteListMode::Whitelist => LocaleKey::MuteModeWhitelist,
    }
}

fn mute_menu_text(lang: LanguageCode, current_mode: &MuteListMode, has_guest: bool) -> String {
    let mode_desc = locales::get_text(lang.as_str(), mute_mode_desc_key(current_mode), None);
    let guest_note = if has_guest {
        locales::get_text(lang.as_str(), LocaleKey::MuteGuestNote, None)
    } else {
        String::new()
    };
    let args = args!(mode_desc = mode_desc, guest_note = guest_note);
    locales::get_text(lang.as_str(), LocaleKey::MuteTitle, args.as_ref())
}

fn mute_menu_keyboard(lang: LanguageCode, current_mode: &MuteListMode) -> InlineKeyboardMarkup {
    let icon_checked = locales::get_text(lang.as_str(), LocaleKey::IconChecked, None);
    let icon_unchecked = locales::get_text(lang.as_str(), LocaleKey::IconUnchecked, None);

    let bl_marker = if current_mode == &MuteListMode::Blacklist {
        icon_checked.as_str()
    } else {
        icon_unchecked.as_str()
    };
    let wl_marker = if current_mode == &MuteListMode::Whitelist {
        icon_checked.as_str()
    } else {
        icon_unchecked.as_str()
    };

    let btn_blacklist_text = locales::get_text(
        lang.as_str(),
        LocaleKey::BtnModeBlacklist,
        args!(marker = bl_marker).as_ref(),
    );
    let btn_whitelist_text = locales::get_text(
        lang.as_str(),
        LocaleKey::BtnModeWhitelist,
        args!(marker = wl_marker).as_ref(),
    );

    let btn_manage_blacklist =
        locales::get_text(lang.as_str(), LocaleKey::BtnManageBlacklist, None);
    let btn_manage_whitelist =
        locales::get_text(lang.as_str(), LocaleKey::BtnManageWhitelist, None);
    let btn_server_blacklist =
        locales::get_text(lang.as_str(), LocaleKey::BtnMuteServerListBlacklist, None);
    let btn_server_whitelist =
        locales::get_text(lang.as_str(), LocaleKey::BtnMuteServerListWhitelist, None);

    InlineKeyboardMarkup::new(vec![
        vec![
            callback_button(
                btn_blacklist_text,
                CallbackAction::Mute(MuteAction::ModeSet {
                    mode: MuteListMode::Blacklist,
                }),
            ),
            callback_button(
                btn_whitelist_text,
                CallbackAction::Mute(MuteAction::ModeSet {
                    mode: MuteListMode::Whitelist,
                }),
            ),
        ],
        vec![callback_button(
            btn_manage_blacklist,
            CallbackAction::Mute(MuteAction::List {
                mode: MuteListMode::Blacklist,
                page: 0,
            }),
        )],
        vec![callback_button(
            btn_manage_whitelist,
            CallbackAction::Mute(MuteAction::List {
                mode: MuteListMode::Whitelist,
                page: 0,
            }),
        )],
        vec![callback_button(
            btn_server_blacklist,
            CallbackAction::Mute(MuteAction::ServerList {
                mode: MuteListMode::Blacklist,
                page: 0,
            }),
        )],
        vec![callback_button(
            btn_server_whitelist,
            CallbackAction::Mute(MuteAction::ServerList {
                mode: MuteListMode::Whitelist,
                page: 0,
            }),
        )],
        vec![back_button(
            lang,
            LocaleKey::BtnBackNotif,
            CallbackAction::Settings(SettingsAction::NotifSelect),
        )],
    ])
}

pub async fn send_mute_menu(
    bot: &Bot,
    msg: &Message,
    lang: LanguageCode,
    current_mode: MuteListMode,
    has_guest: bool,
) -> ResponseResult<()> {
    let text = mute_menu_text(lang, &current_mode, has_guest);
    let keyboard = mute_menu_keyboard(lang, &current_mode);

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
    pub telegram_id: TelegramId,
    pub lang: LanguageCode,
    pub accounts: &'a [UserAccount],
    pub page: usize,
    pub title_key: LocaleKey,
    pub guest_username: Option<&'a str>,
    pub mode: MuteListMode,
}

pub struct RenderMuteListStringsArgs<'a> {
    pub bot: &'a Bot,
    pub msg: &'a Message,
    pub lang: LanguageCode,
    pub items: &'a [TtUsername],
    pub page: usize,
    pub title_key: LocaleKey,
    pub guest_username: Option<&'a str>,
    pub mode: MuteListMode,
}

pub async fn render_mute_list(args: RenderMuteListArgs<'_>) -> ResponseResult<()> {
    let muted_users: Vec<TtUsername> = match args
        .db
        .get_muted_users_list(args.telegram_id, args.mode.clone())
        .await
    {
        Ok(list) => list,
        Err(e) => {
            tracing::error!(
                telegram_id = args.telegram_id.as_i64(),
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
            let is_muted = match args.mode {
                MuteListMode::Blacklist | MuteListMode::Whitelist => {
                    muted_set.contains(&TtUsername::new(acc.username.clone()))
                }
            };
            let icon_key = match (args.mode.clone(), is_muted) {
                (MuteListMode::Blacklist, true) => LocaleKey::ItemStatusBlacklistIn,
                (MuteListMode::Blacklist, false) => LocaleKey::ItemStatusBlacklistOut,
                (MuteListMode::Whitelist, true) => LocaleKey::ItemStatusWhitelistIn,
                (MuteListMode::Whitelist, false) => LocaleKey::ItemStatusWhitelistOut,
            };

            let display_name = if Some(acc.username.as_str()) == args.guest_username {
                locales::get_text(
                    args.lang.as_str(),
                    locales::LocaleKey::DisplayGuestAccount,
                    None,
                )
            } else {
                acc.username.clone()
            };

            let fmt_args = args!(name = display_name);
            let display_text = locales::get_text(args.lang.as_str(), icon_key, fmt_args.as_ref());
            (
                display_text,
                CallbackAction::Mute(MuteAction::ServerToggle {
                    mode: args.mode.clone(),
                    username: TtUsername::new(acc.username.clone()),
                    page: args.page,
                }),
            )
        },
        |p| {
            CallbackAction::Mute(MuteAction::ServerList {
                mode: args.mode.clone(),
                page: p,
            })
        },
        Some(back_btn(
            args.lang,
            LocaleKey::BtnBackMute,
            CallbackAction::Settings(SettingsAction::MuteManage),
        )),
        args.lang,
    );

    let base = locales::get_text(args.lang.as_str(), args.title_key, None);
    let text = append_search_hint(&base, args.lang);
    args.bot
        .edit_message_text(args.msg.chat.id, args.msg.id, text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

pub async fn render_mute_list_strings(args: RenderMuteListStringsArgs<'_>) -> ResponseResult<()> {
    if args.items.is_empty() {
        let text = locales::get_text(args.lang.as_str(), locales::LocaleKey::ListMuteEmpty, None);
        let keyboard = back_button_keyboard(
            args.lang,
            LocaleKey::BtnBackMute,
            CallbackAction::Settings(SettingsAction::MuteManage),
        );
        args.bot
            .edit_message_text(args.msg.chat.id, args.msg.id, text)
            .reply_markup(keyboard)
            .await?;
        return Ok(());
    }

    let mut sorted_items = args.items.to_vec();
    sorted_items.sort_by_key(|a| a.as_str().to_lowercase());

    let keyboard = create_user_list_keyboard(
        &sorted_items,
        args.page,
        |username| {
            let display_name = if Some(username.as_str()) == args.guest_username {
                locales::get_text(
                    args.lang.as_str(),
                    locales::LocaleKey::DisplayGuestAccount,
                    None,
                )
            } else {
                username.to_string()
            };

            let fmt_args = args!(name = display_name);
            let icon_key = match args.mode {
                MuteListMode::Blacklist => LocaleKey::ItemStatusBlacklistIn,
                MuteListMode::Whitelist => LocaleKey::ItemStatusWhitelistIn,
            };
            let display_text = locales::get_text(args.lang.as_str(), icon_key, fmt_args.as_ref());
            (
                display_text,
                CallbackAction::Mute(MuteAction::Toggle {
                    mode: args.mode.clone(),
                    username: username.clone(),
                    page: args.page,
                }),
            )
        },
        |p| {
            CallbackAction::Mute(MuteAction::List {
                mode: args.mode.clone(),
                page: p,
            })
        },
        Some(back_btn(
            args.lang,
            LocaleKey::BtnBackMute,
            CallbackAction::Settings(SettingsAction::MuteManage),
        )),
        args.lang,
    );

    let base = locales::get_text(args.lang.as_str(), args.title_key, None);
    let text = append_search_hint(&base, args.lang);

    args.bot
        .edit_message_text(args.msg.chat.id, args.msg.id, text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}
