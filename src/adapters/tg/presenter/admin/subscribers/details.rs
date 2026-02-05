use crate::adapters::tg::presenter::admin::utils::format_tg_user;
use crate::adapters::tg::presenter::keyboards::{back_button, callback_button};
use crate::args;
use crate::core::callbacks::{AdminAction, CallbackAction, SubAction};
use crate::core::types::{LanguageCode, MuteListMode, NotificationSetting, TelegramId};
use crate::infra::db::types::UserSettings;
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardMarkup, ParseMode};

pub struct SubscriberDetailsArgs<'a> {
    pub bot: &'a Bot,
    pub msg: &'a Message,
    pub lang: LanguageCode,
    pub sub_id: TelegramId,
    pub return_page: usize,
    pub is_main_admin: bool,
    pub admin_chat_id: TelegramId,
    pub settings: UserSettings,
    pub is_admin: bool,
}

pub async fn send_subscriber_details(args: SubscriberDetailsArgs<'_>) -> ResponseResult<()> {
    let display_name = (args
        .bot
        .get_chat(teloxide::types::ChatId(args.sub_id.as_i64()))
        .await)
        .map_or_else(|_| args.sub_id.to_string(), |chat| format_tg_user(&chat));

    let text =
        build_subscriber_details_text(args.lang, &args.settings, display_name, args.is_admin);
    let keyboard = build_subscriber_details_keyboard(
        args.lang,
        args.sub_id,
        args.return_page,
        args.is_admin,
        args.is_main_admin,
        args.admin_chat_id,
    );

    args.bot
        .edit_message_text(args.msg.chat.id, args.msg.id, text)
        .reply_markup(keyboard)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub const fn default_user_settings(sub_id: TelegramId) -> UserSettings {
    UserSettings {
        telegram_id: sub_id,
        language_code: LanguageCode::En,
        notification_settings: NotificationSetting::All,
        mute_list_mode: MuteListMode::Blacklist,
        teamtalk_username: None,
        not_on_online_enabled: false,
        not_on_online_confirmed: false,
        reply_queue_enabled: false,
    }
}

fn build_subscriber_details_text(
    lang: LanguageCode,
    settings: &UserSettings,
    display_name: String,
    is_admin: bool,
) -> String {
    let notif_setting = settings.notification_settings.clone();
    let notif_text = match notif_setting {
        NotificationSetting::All => locales::get_text(
            lang.as_str(),
            locales::LocaleKey::BtnSubAll,
            args!(marker = "").as_ref(),
        ),
        NotificationSetting::JoinOff => locales::get_text(
            lang.as_str(),
            locales::LocaleKey::BtnSubLeave,
            args!(marker = "").as_ref(),
        ),
        NotificationSetting::LeaveOff => locales::get_text(
            lang.as_str(),
            locales::LocaleKey::BtnSubJoin,
            args!(marker = "").as_ref(),
        ),
        NotificationSetting::None => locales::get_text(
            lang.as_str(),
            locales::LocaleKey::BtnSubNone,
            args!(marker = "").as_ref(),
        ),
    };

    let mute_mode = settings.mute_list_mode.clone();
    let mode_text = match mute_mode {
        MuteListMode::Blacklist => {
            locales::get_text(lang.as_str(), locales::LocaleKey::ModeBlacklist, None)
        }
        MuteListMode::Whitelist => {
            locales::get_text(lang.as_str(), locales::LocaleKey::ModeWhitelist, None)
        }
    };
    let sub_lang = settings.language_code;

    let tt_user = settings.teamtalk_username.as_ref().map_or_else(
        || locales::get_text(lang.as_str(), locales::LocaleKey::ValNone, None),
        ToString::to_string,
    );
    let admin_label = if is_admin {
        locales::get_text(lang.as_str(), locales::LocaleKey::ValYes, None)
    } else {
        locales::get_text(lang.as_str(), locales::LocaleKey::ValNo, None)
    };
    let args = args!(
        name = display_name,
        tt_user = tt_user,
        lang = sub_lang.as_str(),
        noon = if settings.not_on_online_enabled {
            locales::get_text(lang.as_str(), locales::LocaleKey::StatusEnabled, None)
        } else {
            locales::get_text(lang.as_str(), locales::LocaleKey::StatusDisabled, None)
        },
        notif = notif_text,
        mode = mode_text,
        admin = admin_label
    );

    locales::get_text(
        lang.as_str(),
        locales::LocaleKey::SubDetailsTitle,
        args.as_ref(),
    )
}

fn build_subscriber_details_keyboard(
    lang: LanguageCode,
    sub_id: TelegramId,
    return_page: usize,
    is_admin: bool,
    is_main_admin: bool,
    admin_chat_id: TelegramId,
) -> InlineKeyboardMarkup {
    let btn = |text_key: locales::LocaleKey, action: SubAction| {
        callback_button(
            locales::get_text(lang.as_str(), text_key, None),
            CallbackAction::Subscriber(action),
        )
    };

    let mut rows = vec![
        vec![btn(
            locales::LocaleKey::BtnSubDelete,
            SubAction::Delete {
                sub_id,
                page: return_page,
            },
        )],
        vec![btn(
            locales::LocaleKey::BtnSubBan,
            SubAction::Ban {
                sub_id,
                page: return_page,
            },
        )],
    ];

    if is_main_admin && sub_id != admin_chat_id {
        let (key, action) = if is_admin {
            (
                locales::LocaleKey::BtnSubAdminRemove,
                SubAction::AdminRemoveConfirm {
                    sub_id,
                    page: return_page,
                },
            )
        } else {
            (
                locales::LocaleKey::BtnSubAdminAdd,
                SubAction::AdminAddConfirm {
                    sub_id,
                    page: return_page,
                },
            )
        };
        rows.push(vec![btn(key, action)]);
    }

    rows.extend([
        vec![btn(
            locales::LocaleKey::BtnSubManageTt,
            SubAction::ManageTt {
                sub_id,
                page: return_page,
            },
        )],
        vec![btn(
            locales::LocaleKey::BtnSubLang,
            SubAction::LangMenu {
                sub_id,
                page: return_page,
            },
        )],
        vec![btn(
            locales::LocaleKey::BtnSubNoon,
            SubAction::NoonToggle {
                sub_id,
                page: return_page,
            },
        )],
        vec![btn(
            locales::LocaleKey::BtnSubNotif,
            SubAction::NotifMenu {
                sub_id,
                page: return_page,
            },
        )],
        vec![btn(
            locales::LocaleKey::BtnSubMuteMode,
            SubAction::ModeMenu {
                sub_id,
                page: return_page,
            },
        )],
        vec![btn(
            locales::LocaleKey::BtnSubViewMute,
            SubAction::MuteView {
                sub_id,
                page: return_page,
                view_page: 0,
            },
        )],
        vec![back_button(
            lang,
            locales::LocaleKey::BtnBackSubs,
            CallbackAction::Admin(AdminAction::SubsList { page: return_page }),
        )],
    ]);
    InlineKeyboardMarkup::new(rows)
}
