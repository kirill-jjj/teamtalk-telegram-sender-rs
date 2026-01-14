use crate::adapters::tg::admin_logic::utils::format_tg_user;
use crate::adapters::tg::keyboards::{
    back_btn, back_button, callback_button, create_user_list_keyboard,
};
use crate::app::services::user_settings as user_settings_service;
use crate::args;
use crate::core::callbacks::{AdminAction, CallbackAction, MenuAction, SubAction};
use crate::core::types::{LanguageCode, MuteListMode, NotificationSetting};
use crate::infra::db::Database;
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardMarkup, ParseMode};

struct SubDisplayInfo {
    telegram_id: i64,
    display_name: String,
    tt_username: Option<String>,
}

pub async fn send_subscribers_list(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    db: &Database,
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
        bot.send_message(
            chat_id,
            locales::get_text(lang.as_str(), "list-subs-empty", None),
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
                parts.push(format!("TT: {}", tt));
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
            "btn-back-menu",
            CallbackAction::Menu(MenuAction::Who),
        )),
        lang,
    );

    bot.send_message(
        chat_id,
        locales::get_text(lang.as_str(), "list-subs-title", None),
    )
    .reply_markup(keyboard)
    .await?;
    Ok(())
}

pub async fn edit_subscribers_list(
    bot: &Bot,
    msg: &Message,
    db: &Database,
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
            locales::get_text(lang.as_str(), "list-subs-empty", None),
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
                parts.push(format!("TT: {}", tt));
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
            "btn-back-menu",
            CallbackAction::Menu(MenuAction::Who),
        )),
        lang,
    );

    bot.edit_message_text(
        msg.chat.id,
        msg.id,
        locales::get_text(lang.as_str(), "list-subs-title", None),
    )
    .reply_markup(keyboard)
    .await?;
    Ok(())
}

async fn prepare_display_list(
    bot: &Bot,
    subs: Vec<crate::infra::db::types::SubscriberInfo>,
) -> Vec<SubDisplayInfo> {
    let mut display_list = Vec::new();
    for sub in subs {
        let display_name = match bot.get_chat(teloxide::types::ChatId(sub.telegram_id)).await {
            Ok(chat) => format_tg_user(&chat),
            Err(e) => {
                tracing::error!(
                    telegram_id = sub.telegram_id,
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

pub async fn send_subscriber_details(
    bot: &Bot,
    msg: &Message,
    db: &Database,
    lang: LanguageCode,
    sub_id: i64,
    return_page: usize,
) -> ResponseResult<()> {
    let settings = db
        .get_or_create_user(sub_id, LanguageCode::En)
        .await
        .unwrap_or_else(|e| {
            tracing::error!(
                sub_id,
                error = %e,
                "Failed to load subscriber settings"
            );
            crate::infra::db::types::UserSettings {
                telegram_id: sub_id,
                language_code: "en".to_string(),
                notification_settings: "all".to_string(),
                mute_list_mode: "blacklist".to_string(),
                teamtalk_username: None,
                not_on_online_enabled: false,
                not_on_online_confirmed: false,
            }
        });

    let display_name = match bot.get_chat(teloxide::types::ChatId(sub_id)).await {
        Ok(chat) => format_tg_user(&chat),
        Err(_) => sub_id.to_string(),
    };

    let notif_setting =
        user_settings_service::parse_notification_setting(&settings.notification_settings);
    let notif_text = match notif_setting {
        NotificationSetting::All => {
            locales::get_text(lang.as_str(), "btn-sub-all", args!(marker = "").as_ref())
        }
        NotificationSetting::JoinOff => {
            locales::get_text(lang.as_str(), "btn-sub-join", args!(marker = "").as_ref())
        }
        NotificationSetting::LeaveOff => {
            locales::get_text(lang.as_str(), "btn-sub-leave", args!(marker = "").as_ref())
        }
        NotificationSetting::None => {
            locales::get_text(lang.as_str(), "btn-sub-none", args!(marker = "").as_ref())
        }
    };

    let mute_mode = user_settings_service::parse_mute_list_mode(&settings.mute_list_mode);
    let mode_text = match mute_mode {
        MuteListMode::Blacklist => locales::get_text(lang.as_str(), "mode-blacklist", None),
        MuteListMode::Whitelist => locales::get_text(lang.as_str(), "mode-whitelist", None),
    };
    let sub_lang = LanguageCode::from_str_or_default(&settings.language_code, LanguageCode::En);

    let args = args!(
        name = display_name,
        tt_user = settings
            .teamtalk_username
            .unwrap_or_else(|| locales::get_text(lang.as_str(), "val-none", None)),
        lang = sub_lang.as_str(),
        noon = if settings.not_on_online_enabled {
            locales::get_text(lang.as_str(), "status-enabled", None)
        } else {
            locales::get_text(lang.as_str(), "status-disabled", None)
        },
        notif = notif_text,
        mode = mode_text
    );

    let text = locales::get_text(lang.as_str(), "sub-details-title", args.as_ref());

    let btn = |text_key: &str, action: SubAction| {
        callback_button(
            locales::get_text(lang.as_str(), text_key, None),
            CallbackAction::Subscriber(action),
        )
    };

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![btn(
            "btn-sub-delete",
            SubAction::Delete {
                sub_id,
                page: return_page,
            },
        )],
        vec![btn(
            "btn-sub-ban",
            SubAction::Ban {
                sub_id,
                page: return_page,
            },
        )],
        vec![btn(
            "btn-sub-manage-tt",
            SubAction::ManageTt {
                sub_id,
                page: return_page,
            },
        )],
        vec![btn(
            "btn-sub-lang",
            SubAction::LangMenu {
                sub_id,
                page: return_page,
            },
        )],
        vec![btn(
            "btn-sub-noon",
            SubAction::NoonToggle {
                sub_id,
                page: return_page,
            },
        )],
        vec![btn(
            "btn-sub-notif",
            SubAction::NotifMenu {
                sub_id,
                page: return_page,
            },
        )],
        vec![btn(
            "btn-sub-mute-mode",
            SubAction::ModeMenu {
                sub_id,
                page: return_page,
            },
        )],
        vec![btn(
            "btn-sub-view-mute",
            SubAction::MuteView {
                sub_id,
                page: return_page,
                view_page: 0,
            },
        )],
        vec![back_button(
            lang,
            "btn-back-subs",
            CallbackAction::Admin(AdminAction::SubsList { page: return_page }),
        )],
    ]);

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}
