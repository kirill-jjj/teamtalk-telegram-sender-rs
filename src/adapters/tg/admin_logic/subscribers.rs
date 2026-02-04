use crate::adapters::tg::admin_logic::utils::format_tg_user;
use crate::adapters::tg::keyboards::{
    back_btn, back_button, callback_button, create_user_list_keyboard,
};
use crate::adapters::tg::search::{
    SearchContext, SearchListType, append_search_hint, set_search_context_raw,
};
use crate::args;
use crate::core::callbacks::{AdminAction, CallbackAction, MenuAction, SubAction};
use crate::core::types::{LanguageCode, MuteListMode, NotificationSetting, TelegramId, TtUsername};
use crate::infra::db::Database;
use crate::infra::db::types::UserSettings;
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::types::{InlineKeyboardMarkup, ParseMode};

#[derive(Clone)]
pub struct SubDisplayInfo {
    pub telegram_id: TelegramId,
    pub display_name: String,
    pub tt_username: Option<TtUsername>,
}

pub async fn send_subscribers_list(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    db: &Database,
    search_contexts: &std::sync::Arc<
        tokio::sync::Mutex<
            std::collections::HashMap<
                teloxide::types::ChatId,
                crate::adapters::tg::search::SearchContext,
            >,
        >,
    >,
    lang: LanguageCode,
    page: usize,
    reply_to: Option<teloxide::types::MessageId>,
) -> ResponseResult<()> {
    let subs = match db.get_subscribers().await {
        Ok(list) => list,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load subscribers");
            Vec::new()
        }
    };

    if subs.is_empty() {
        let req = bot.send_message(
            chat_id,
            locales::get_text(lang.as_str(), locales::LocaleKey::ListSubsEmpty, None),
        );
        if let Some(reply_to) = reply_to {
            req.reply_to(reply_to).await?;
        } else {
            req.await?;
        }
        return Ok(());
    }

    let display_list = prepare_display_list(bot, subs).await;

    let keyboard = create_user_list_keyboard(
        &display_list,
        page,
        |s| {
            let mut parts = vec![s.display_name.clone()];
            if let Some(tt) = &s.tt_username {
                parts.push(format!("TT: {tt}"));
            }
            let name = parts.join(", ");
            (
                name,
                CallbackAction::Subscriber(SubAction::Details {
                    sub_id: s.telegram_id,
                    page,
                }),
            )
        },
        |p| CallbackAction::Admin(AdminAction::SubsList { page: p }),
        Some(back_btn(
            lang,
            locales::LocaleKey::BtnBackMenu,
            CallbackAction::Menu(MenuAction::Who),
        )),
        lang,
    );

    let base = locales::get_text(lang.as_str(), locales::LocaleKey::ListSubsTitle, None);
    let text = append_search_hint(&base, lang);
    let req = bot.send_message(chat_id, text).reply_markup(keyboard);
    if let Some(reply_to) = reply_to {
        let msg = req.reply_to(reply_to).await?;
        set_search_context_raw(
            search_contexts,
            msg.chat.id,
            SearchContext {
                message_id: msg.id,
                list_type: SearchListType::Subscribers,
            },
        )
        .await;
    } else {
        let msg = req.await?;
        set_search_context_raw(
            search_contexts,
            msg.chat.id,
            SearchContext {
                message_id: msg.id,
                list_type: SearchListType::Subscribers,
            },
        )
        .await;
    }
    Ok(())
}

pub async fn edit_subscribers_list(
    bot: &Bot,
    msg: &Message,
    db: &Database,
    search_contexts: &std::sync::Arc<
        tokio::sync::Mutex<
            std::collections::HashMap<
                teloxide::types::ChatId,
                crate::adapters::tg::search::SearchContext,
            >,
        >,
    >,
    lang: LanguageCode,
    page: usize,
) -> ResponseResult<()> {
    let subs = match db.get_subscribers().await {
        Ok(list) => list,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load subscribers");
            Vec::new()
        }
    };

    if subs.is_empty() {
        bot.edit_message_text(
            msg.chat.id,
            msg.id,
            locales::get_text(lang.as_str(), locales::LocaleKey::ListSubsEmpty, None),
        )
        .await?;
        return Ok(());
    }

    let display_list = prepare_display_list(bot, subs).await;

    let keyboard = create_user_list_keyboard(
        &display_list,
        page,
        |s| {
            let mut parts = vec![s.display_name.clone()];
            if let Some(tt) = &s.tt_username {
                parts.push(format!("TT: {tt}"));
            }
            let name = parts.join(", ");
            (
                name,
                CallbackAction::Subscriber(SubAction::Details {
                    sub_id: s.telegram_id,
                    page,
                }),
            )
        },
        |p| CallbackAction::Admin(AdminAction::SubsList { page: p }),
        Some(back_btn(
            lang,
            locales::LocaleKey::BtnBackMenu,
            CallbackAction::Menu(MenuAction::Who),
        )),
        lang,
    );

    let base = locales::get_text(lang.as_str(), locales::LocaleKey::ListSubsTitle, None);
    let text = append_search_hint(&base, lang);
    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .await?;
    set_search_context_raw(
        search_contexts,
        msg.chat.id,
        SearchContext {
            message_id: msg.id,
            list_type: SearchListType::Subscribers,
        },
    )
    .await;
    Ok(())
}

pub async fn prepare_display_list(
    bot: &Bot,
    subs: Vec<crate::infra::db::types::SubscriberInfo>,
) -> Vec<SubDisplayInfo> {
    let mut display_list = Vec::new();
    for sub in subs {
        let display_name = match bot
            .get_chat(teloxide::types::ChatId(sub.telegram_id.as_i64()))
            .await
        {
            Ok(chat) => format_tg_user(&chat),
            Err(e) => {
                tracing::error!(
                    telegram_id = sub.telegram_id.as_i64(),
                    error = %e,
                    "Failed to load Telegram user"
                );
                sub.telegram_id.to_string()
            }
        };
        display_list.push(SubDisplayInfo {
            telegram_id: sub.telegram_id,
            display_name,
            tt_username: sub.teamtalk_username,
        });
    }
    display_list.sort_by(|a, b| {
        a.display_name
            .to_lowercase()
            .cmp(&b.display_name.to_lowercase())
    });
    display_list
}

pub struct SubscriberDetailsArgs<'a> {
    pub bot: &'a Bot,
    pub msg: &'a Message,
    pub db: &'a Database,
    pub lang: LanguageCode,
    pub sub_id: TelegramId,
    pub return_page: usize,
    pub is_main_admin: bool,
    pub admin_chat_id: TelegramId,
}

pub async fn send_subscriber_details(args: SubscriberDetailsArgs<'_>) -> ResponseResult<()> {
    let settings = load_subscriber_settings(args.db, args.sub_id).await;

    let display_name = (args
        .bot
        .get_chat(teloxide::types::ChatId(args.sub_id.as_i64()))
        .await)
        .map_or_else(|_| args.sub_id.to_string(), |chat| format_tg_user(&chat));

    let is_admin = match args.db.get_all_admins().await {
        Ok(admins) => admins.contains(&args.sub_id),
        Err(e) => {
            tracing::error!(
                sub_id = args.sub_id.as_i64(),
                error = %e,
                "Failed to load admins list"
            );
            false
        }
    };
    let text = build_subscriber_details_text(args.lang, &settings, display_name, is_admin);
    let keyboard = build_subscriber_details_keyboard(
        args.lang,
        args.sub_id,
        args.return_page,
        is_admin,
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

async fn load_subscriber_settings(db: &Database, sub_id: TelegramId) -> UserSettings {
    db.get_or_create_user(sub_id, LanguageCode::En)
        .await
        .unwrap_or_else(|e| {
            tracing::error!(
                sub_id = sub_id.as_i64(),
                error = %e,
                "Failed to load subscriber settings"
            );
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
        })
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
