use crate::adapters::tg::handlers::search::append_search_hint;
use crate::adapters::tg::presenter::keyboards::{
    back_btn, back_button_keyboard, callback_button, create_user_list_keyboard,
};
use crate::args;
use crate::core::callbacks::{CallbackAction, MuteAction, SettingsAction};
use crate::core::types::{LanguageCode, MuteListMode, TtUsername};
use crate::infra::locales;
use crate::infra::locales::LocaleKey;
use teamtalk::types::UserAccount;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardMarkup, ParseMode};

const fn mute_mode_desc_key(current_mode: &MuteListMode) -> LocaleKey {
    match current_mode {
        MuteListMode::Blacklist => LocaleKey::MuteModeBlacklist,
        MuteListMode::Whitelist => LocaleKey::MuteModeWhitelist,
    }
}

fn mute_menu_text(lang: LanguageCode, current_mode: &MuteListMode, has_guest: bool) -> String {
    let mode_desc = locales::get_text_or_log(lang.as_str(), mute_mode_desc_key(current_mode), None);
    let guest_note = if has_guest {
        locales::get_text_or_log(lang.as_str(), LocaleKey::MuteGuestNote, None)
    } else {
        String::new()
    };
    let args = args!(mode_desc = mode_desc, guest_note = guest_note);
    locales::get_text_or_log(lang.as_str(), LocaleKey::MuteTitle, args.as_ref())
}

fn mute_menu_keyboard(lang: LanguageCode, current_mode: &MuteListMode) -> InlineKeyboardMarkup {
    let icon_checked = locales::get_text_or_log(lang.as_str(), LocaleKey::IconChecked, None);
    let icon_unchecked = locales::get_text_or_log(lang.as_str(), LocaleKey::IconUnchecked, None);

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

    let btn_blacklist_text = locales::get_text_or_log(
        lang.as_str(),
        LocaleKey::BtnModeBlacklist,
        args!(marker = bl_marker).as_ref(),
    );
    let btn_whitelist_text = locales::get_text_or_log(
        lang.as_str(),
        LocaleKey::BtnModeWhitelist,
        args!(marker = wl_marker).as_ref(),
    );

    let btn_manage_blacklist =
        locales::get_text_or_log(lang.as_str(), LocaleKey::BtnManageBlacklist, None);
    let btn_manage_whitelist =
        locales::get_text_or_log(lang.as_str(), LocaleKey::BtnManageWhitelist, None);
    let btn_server_blacklist =
        locales::get_text_or_log(lang.as_str(), LocaleKey::BtnMuteServerListBlacklist, None);
    let btn_server_whitelist =
        locales::get_text_or_log(lang.as_str(), LocaleKey::BtnMuteServerListWhitelist, None);

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
        vec![crate::adapters::tg::presenter::keyboards::back_button(
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
    pub lang: LanguageCode,
    pub accounts: &'a [UserAccount],
    pub page: usize,
    pub title_key: LocaleKey,
    pub guest_username: Option<&'a str>,
    pub mode: MuteListMode,
    pub muted_users: &'a [TtUsername],
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
    let muted_set: std::collections::HashSet<_> = args.muted_users.iter().cloned().collect();

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
                locales::get_text_or_log(
                    args.lang.as_str(),
                    locales::LocaleKey::DisplayGuestAccount,
                    None,
                )
            } else {
                acc.username.clone()
            };

            let fmt_args = args!(name = display_name);
            let display_text = locales::get_text_or_log(args.lang.as_str(), icon_key, fmt_args.as_ref());
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

    let base = locales::get_text_or_log(args.lang.as_str(), args.title_key, None);
    let text = append_search_hint(&base, args.lang);
    args.bot
        .edit_message_text(args.msg.chat.id, args.msg.id, text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

pub async fn render_mute_list_strings(args: RenderMuteListStringsArgs<'_>) -> ResponseResult<()> {
    if args.items.is_empty() {
        let text = locales::get_text_or_log(args.lang.as_str(), locales::LocaleKey::ListMuteEmpty, None);
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
                locales::get_text_or_log(
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
            let display_text = locales::get_text_or_log(args.lang.as_str(), icon_key, fmt_args.as_ref());
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

    let base = locales::get_text_or_log(args.lang.as_str(), args.title_key, None);
    let text = append_search_hint(&base, args.lang);

    args.bot
        .edit_message_text(args.msg.chat.id, args.msg.id, text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}
