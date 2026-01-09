use crate::args;
use crate::db::Database;
use crate::locales;
use crate::tg_bot::admin_logic::utils::format_tg_user;
use crate::tg_bot::callbacks_types::{AdminAction, CallbackAction, MenuAction, SubAction};
use crate::tg_bot::keyboards::create_user_list_keyboard;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};

struct SubDisplayInfo {
    telegram_id: i64,
    display_name: String,
    tt_username: Option<String>,
}

pub async fn send_subscribers_list(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    db: &Database,
    lang: &str,
    page: usize,
) -> ResponseResult<()> {
    let subs = db.get_subscribers().await.unwrap_or_default();

    if subs.is_empty() {
        bot.send_message(chat_id, locales::get_text(lang, "list-subs-empty", None))
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
        Some((
            locales::get_text(lang, "btn-back-menu", None),
            CallbackAction::Menu(MenuAction::Who),
        )),
        lang,
    );

    bot.send_message(chat_id, locales::get_text(lang, "list-subs-title", None))
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

pub async fn edit_subscribers_list(
    bot: &Bot,
    msg: &Message,
    db: &Database,
    lang: &str,
    page: usize,
) -> ResponseResult<()> {
    let subs = db.get_subscribers().await.unwrap_or_default();

    if subs.is_empty() {
        bot.edit_message_text(
            msg.chat.id,
            msg.id,
            locales::get_text(lang, "list-subs-empty", None),
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
        Some((
            locales::get_text(lang, "btn-back-menu", None),
            CallbackAction::Menu(MenuAction::Who),
        )),
        lang,
    );

    bot.edit_message_text(
        msg.chat.id,
        msg.id,
        locales::get_text(lang, "list-subs-title", None),
    )
    .reply_markup(keyboard)
    .await?;
    Ok(())
}

async fn prepare_display_list(
    bot: &Bot,
    subs: Vec<crate::db::types::SubscriberInfo>,
) -> Vec<SubDisplayInfo> {
    let mut display_list = Vec::new();
    for sub in subs {
        let display_name = match bot.get_chat(teloxide::types::ChatId(sub.telegram_id)).await {
            Ok(chat) => format_tg_user(&chat),
            Err(_) => sub.telegram_id.to_string(),
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
    lang: &str,
    sub_id: i64,
    return_page: usize,
) -> ResponseResult<()> {
    let settings = db
        .get_or_create_user(sub_id, "en")
        .await
        .unwrap_or_else(|_| crate::db::types::UserSettings {
            telegram_id: sub_id,
            language_code: "en".to_string(),
            notification_settings: "all".to_string(),
            mute_list_mode: "blacklist".to_string(),
            teamtalk_username: None,
            not_on_online_enabled: false,
            not_on_online_confirmed: false,
        });

    let display_name = match bot.get_chat(teloxide::types::ChatId(sub_id)).await {
        Ok(chat) => format_tg_user(&chat),
        Err(_) => sub_id.to_string(),
    };

    let notif_map = |s: &str| match s {
        "all" => locales::get_text(lang, "btn-sub-all", args!(marker = "").as_ref()),
        "join_off" => locales::get_text(lang, "btn-sub-leave", args!(marker = "").as_ref()),
        "leave_off" => locales::get_text(lang, "btn-sub-join", args!(marker = "").as_ref()),
        "none" => locales::get_text(lang, "btn-sub-none", args!(marker = "").as_ref()),
        _ => s.to_string(),
    };

    let mode_map = |s: &str| match s {
        "blacklist" => locales::get_text(lang, "mode-blacklist", None),
        "whitelist" => locales::get_text(lang, "mode-whitelist", None),
        _ => s.to_string(),
    };

    let args = args!(
        name = display_name,
        tt_user = settings
            .teamtalk_username
            .unwrap_or_else(|| locales::get_text(lang, "val-none", None)),
        lang = settings.language_code,
        noon = if settings.not_on_online_enabled {
            locales::get_text(lang, "status-enabled", None)
        } else {
            locales::get_text(lang, "status-disabled", None)
        },
        notif = notif_map(&settings.notification_settings),
        mode = mode_map(&settings.mute_list_mode)
    );

    let text = locales::get_text(lang, "sub-details-title", args.as_ref());

    let btn = |text_key: &str, action: SubAction| {
        InlineKeyboardButton::callback(
            locales::get_text(lang, text_key, None),
            CallbackAction::Subscriber(action).to_string(),
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
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang, "btn-back-subs", None),
            CallbackAction::Admin(AdminAction::SubsList { page: return_page }).to_string(),
        )],
    ]);

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}
