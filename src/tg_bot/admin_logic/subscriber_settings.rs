use crate::args;
use crate::db::Database;
use crate::locales;
use crate::tg_bot::callbacks_types::{CallbackAction, SubAction};
use crate::tg_bot::keyboards::create_user_list_keyboard;
use teamtalk::types::UserAccount;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub async fn send_sub_manage_tt_menu(
    bot: &Bot,
    msg: &Message,
    db: &Database,
    lang: &str,
    sub_id: i64,
    return_page: usize,
) -> ResponseResult<()> {
    let settings = match db.get_or_create_user(sub_id, "en").await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to get or create user {}: {}", sub_id, e);
            bot.edit_message_text(
                msg.chat.id,
                msg.id,
                locales::get_text(lang, "cmd-error", None),
            )
            .await?;
            return Ok(());
        }
    };
    let tt_user = settings.teamtalk_username;

    let args = args!(id = sub_id.to_string());
    let text = locales::get_text(lang, "sub-manage-tt-title", args.as_ref());

    let mut buttons = vec![];
    if let Some(user) = tt_user {
        let args_btn = args!(user = user);
        buttons.push(vec![InlineKeyboardButton::callback(
            locales::get_text(lang, "btn-unlink", args_btn.as_ref()),
            CallbackAction::Subscriber(SubAction::Unlink {
                sub_id,
                page: return_page,
            })
            .to_string(),
        )]);
    }
    buttons.push(vec![InlineKeyboardButton::callback(
        locales::get_text(lang, "btn-link-new", None),
        CallbackAction::Subscriber(SubAction::LinkList {
            sub_id,
            page: return_page,
            list_page: 0,
        })
        .to_string(),
    )]);
    buttons.push(vec![InlineKeyboardButton::callback(
        locales::get_text(lang, "btn-back-user-actions", None),
        CallbackAction::Subscriber(SubAction::Details {
            sub_id,
            page: return_page,
        })
        .to_string(),
    )]);

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(InlineKeyboardMarkup::new(buttons))
        .await?;
    Ok(())
}

pub async fn send_sub_link_account_list(
    bot: &Bot,
    msg: &Message,
    user_accounts: &std::sync::Arc<dashmap::DashMap<String, UserAccount>>,
    lang: &str,
    target_id: i64,
    sub_page: usize,
    page: usize,
) -> ResponseResult<()> {
    let mut accounts: Vec<UserAccount> =
        user_accounts.iter().map(|kv| kv.value().clone()).collect();
    accounts.sort_by(|a, b| a.username.to_lowercase().cmp(&b.username.to_lowercase()));

    let keyboard = create_user_list_keyboard(
        &accounts,
        page,
        |acc| {
            (
                acc.username.clone(),
                CallbackAction::Subscriber(SubAction::LinkPerform {
                    sub_id: target_id,
                    page: sub_page,
                    username: acc.username.clone(),
                }),
            )
        },
        |p| {
            CallbackAction::Subscriber(SubAction::LinkList {
                sub_id: target_id,
                page: sub_page,
                list_page: p,
            })
        },
        Some((
            locales::get_text(lang, "btn-back-manage-acc", None),
            CallbackAction::Subscriber(SubAction::ManageTt {
                sub_id: target_id,
                page: sub_page,
            }),
        )),
        lang,
    );

    let args = args!(id = target_id.to_string());
    let text = locales::get_text(lang, "list-link-title", args.as_ref());

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

pub async fn send_sub_lang_menu(
    bot: &Bot,
    msg: &Message,
    lang: &str,
    target_id: i64,
    return_page: usize,
) -> ResponseResult<()> {
    let args = args!(id = target_id.to_string());
    let text = locales::get_text(lang, "sub-lang-title", args.as_ref());

    let mk_btn = |lbl: &str, l_code: &str| {
        InlineKeyboardButton::callback(
            lbl,
            CallbackAction::Subscriber(SubAction::LangSet {
                sub_id: target_id,
                page: return_page,
                lang: l_code.to_string(),
            })
            .to_string(),
        )
    };

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![mk_btn("🇷🇺 Русский", "ru")],
        vec![mk_btn("🇬🇧 English", "en")],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang, "btn-back-user-actions", None),
            CallbackAction::Subscriber(SubAction::Details {
                sub_id: target_id,
                page: return_page,
            })
            .to_string(),
        )],
    ]);

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

pub async fn send_sub_notif_menu(
    bot: &Bot,
    msg: &Message,
    lang: &str,
    target_id: i64,
    return_page: usize,
) -> ResponseResult<()> {
    let args = args!(id = target_id.to_string());
    let text = locales::get_text(lang, "sub-notif-title", args.as_ref());

    let marker_args = args!(marker = "");

    let btn_all = locales::get_text(lang, "btn-sub-all", marker_args.as_ref());
    let btn_join = locales::get_text(lang, "btn-sub-join", marker_args.as_ref());
    let btn_leave = locales::get_text(lang, "btn-sub-leave", marker_args.as_ref());
    let btn_none = locales::get_text(lang, "btn-sub-none", marker_args.as_ref());

    let mk_act = |val: &str| {
        CallbackAction::Subscriber(SubAction::NotifSet {
            sub_id: target_id,
            page: return_page,
            val: val.to_string(),
        })
        .to_string()
    };

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(btn_all, mk_act("all"))],
        vec![InlineKeyboardButton::callback(btn_join, mk_act("join_off"))],
        vec![InlineKeyboardButton::callback(
            btn_leave,
            mk_act("leave_off"),
        )],
        vec![InlineKeyboardButton::callback(btn_none, mk_act("none"))],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang, "btn-back-user-actions", None),
            CallbackAction::Subscriber(SubAction::Details {
                sub_id: target_id,
                page: return_page,
            })
            .to_string(),
        )],
    ]);

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

pub async fn send_sub_mute_mode_menu(
    bot: &Bot,
    msg: &Message,
    lang: &str,
    target_id: i64,
    return_page: usize,
) -> ResponseResult<()> {
    let args = args!(id = target_id.to_string());
    let text = locales::get_text(lang, "sub-mode-title", args.as_ref());

    let bl_text = locales::get_text(lang, "mode-blacklist", None);
    let wl_text = locales::get_text(lang, "mode-whitelist", None);

    let mk_act = |mode: &str| {
        CallbackAction::Subscriber(SubAction::ModeSet {
            sub_id: target_id,
            page: return_page,
            mode: mode.to_string(),
        })
        .to_string()
    };

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(bl_text, mk_act("blacklist"))],
        vec![InlineKeyboardButton::callback(wl_text, mk_act("whitelist"))],
        vec![InlineKeyboardButton::callback(
            locales::get_text(lang, "btn-back-user-actions", None),
            CallbackAction::Subscriber(SubAction::Details {
                sub_id: target_id,
                page: return_page,
            })
            .to_string(),
        )],
    ]);

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

pub async fn send_sub_mute_list(
    bot: &Bot,
    msg: &Message,
    db: &Database,
    lang: &str,
    target_id: i64,
    sub_page: usize,
    page: usize,
) -> ResponseResult<()> {
    let muted: Vec<String> = db.get_muted_users_list(target_id).await.unwrap_or_default();

    let user_name = format!("{}", target_id);
    let args = args!(name = user_name);
    let title = locales::get_text(lang, "list-mute-title", args.as_ref());

    let keyboard = create_user_list_keyboard(
        &muted,
        page,
        |username| (username.clone(), CallbackAction::NoOp), // Список просмотра, действия не нужны
        |p| {
            CallbackAction::Subscriber(SubAction::MuteView {
                sub_id: target_id,
                page: sub_page,
                view_page: p,
            })
        },
        Some((
            locales::get_text(lang, "btn-back-user-actions", None),
            CallbackAction::Subscriber(SubAction::Details {
                sub_id: target_id,
                page: sub_page,
            }),
        )),
        lang,
    );

    bot.edit_message_text(msg.chat.id, msg.id, title)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}
