use crate::adapters::tg::keyboards::{
    back_btn, back_button, callback_button, create_user_list_keyboard,
};
use crate::app::services::user_settings as user_settings_service;
use crate::args;
use crate::core::callbacks::{CallbackAction, SubAction};
use crate::core::types::{LanguageCode, MuteListMode, NotificationSetting, TtUsername};
use crate::infra::db::Database;
use crate::infra::locales;
use teamtalk::types::UserAccount;
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardMarkup;

pub async fn send_sub_manage_tt_menu(
    bot: &Bot,
    msg: &Message,
    db: &Database,
    lang: LanguageCode,
    sub_id: i64,
    return_page: usize,
) -> ResponseResult<()> {
    let settings = match user_settings_service::get_or_create(db, sub_id, LanguageCode::En).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to get or create user {}: {}", sub_id, e);
            bot.edit_message_text(
                msg.chat.id,
                msg.id,
                locales::get_text(lang.as_str(), "cmd-error", None),
            )
            .await?;
            return Ok(());
        }
    };
    let tt_user = settings.teamtalk_username;

    let args = args!(id = sub_id.to_string());
    let text = locales::get_text(lang.as_str(), "sub-manage-tt-title", args.as_ref());

    let mut buttons = vec![];
    if let Some(user) = tt_user {
        let args_btn = args!(user = user);
        buttons.push(vec![callback_button(
            locales::get_text(lang.as_str(), "btn-unlink", args_btn.as_ref()),
            CallbackAction::Subscriber(SubAction::Unlink {
                sub_id,
                page: return_page,
            }),
        )]);
    }
    buttons.push(vec![callback_button(
        locales::get_text(lang.as_str(), "btn-link-new", None),
        CallbackAction::Subscriber(SubAction::LinkList {
            sub_id,
            page: return_page,
            list_page: 0,
        }),
    )]);
    buttons.push(vec![back_button(
        lang,
        "btn-back-user-actions",
        CallbackAction::Subscriber(SubAction::Details {
            sub_id,
            page: return_page,
        }),
    )]);

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(InlineKeyboardMarkup::new(buttons))
        .await?;
    Ok(())
}

pub async fn send_sub_link_account_list(
    bot: &Bot,
    msg: &Message,
    user_accounts: &std::sync::Arc<
        std::sync::RwLock<std::collections::HashMap<String, UserAccount>>,
    >,
    lang: LanguageCode,
    target_id: i64,
    sub_page: usize,
    page: usize,
) -> ResponseResult<()> {
    let mut accounts: Vec<UserAccount> = user_accounts
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .values()
        .cloned()
        .collect();
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
                    username: TtUsername::new(acc.username.clone()),
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
        Some(back_btn(
            lang,
            "btn-back-manage-acc",
            CallbackAction::Subscriber(SubAction::ManageTt {
                sub_id: target_id,
                page: sub_page,
            }),
        )),
        lang,
    );

    let args = args!(id = target_id.to_string());
    let text = locales::get_text(lang.as_str(), "list-link-title", args.as_ref());

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

pub async fn send_sub_lang_menu(
    bot: &Bot,
    msg: &Message,
    lang: LanguageCode,
    target_id: i64,
    return_page: usize,
) -> ResponseResult<()> {
    let args = args!(id = target_id.to_string());
    let text = locales::get_text(lang.as_str(), "sub-lang-title", args.as_ref());

    let mk_btn = |lbl: &str, l_code: &str| {
        callback_button(
            lbl,
            CallbackAction::Subscriber(SubAction::LangSet {
                sub_id: target_id,
                page: return_page,
                lang: match l_code {
                    "ru" => LanguageCode::Ru,
                    _ => LanguageCode::En,
                },
            }),
        )
    };

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![mk_btn("🇷🇺 Русский", "ru")],
        vec![mk_btn("🇬🇧 English", "en")],
        vec![back_button(
            lang,
            "btn-back-user-actions",
            CallbackAction::Subscriber(SubAction::Details {
                sub_id: target_id,
                page: return_page,
            }),
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
    lang: LanguageCode,
    target_id: i64,
    return_page: usize,
) -> ResponseResult<()> {
    let args = args!(id = target_id.to_string());
    let text = locales::get_text(lang.as_str(), "sub-notif-title", args.as_ref());

    let marker_args = args!(marker = "");

    let btn_all = locales::get_text(lang.as_str(), "btn-sub-all", marker_args.as_ref());
    let btn_join = locales::get_text(lang.as_str(), "btn-sub-join", marker_args.as_ref());
    let btn_leave = locales::get_text(lang.as_str(), "btn-sub-leave", marker_args.as_ref());
    let btn_none = locales::get_text(lang.as_str(), "btn-sub-none", marker_args.as_ref());

    let mk_act = |val: NotificationSetting| {
        CallbackAction::Subscriber(SubAction::NotifSet {
            sub_id: target_id,
            page: return_page,
            val,
        })
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
            "btn-back-user-actions",
            CallbackAction::Subscriber(SubAction::Details {
                sub_id: target_id,
                page: return_page,
            }),
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
    lang: LanguageCode,
    target_id: i64,
    return_page: usize,
) -> ResponseResult<()> {
    let args = args!(id = target_id.to_string());
    let text = locales::get_text(lang.as_str(), "sub-mode-title", args.as_ref());

    let bl_text = locales::get_text(lang.as_str(), "mode-blacklist", None);
    let wl_text = locales::get_text(lang.as_str(), "mode-whitelist", None);

    let mk_act = |mode: MuteListMode| {
        CallbackAction::Subscriber(SubAction::ModeSet {
            sub_id: target_id,
            page: return_page,
            mode,
        })
    };

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![callback_button(bl_text, mk_act(MuteListMode::Blacklist))],
        vec![callback_button(wl_text, mk_act(MuteListMode::Whitelist))],
        vec![back_button(
            lang,
            "btn-back-user-actions",
            CallbackAction::Subscriber(SubAction::Details {
                sub_id: target_id,
                page: return_page,
            }),
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
    lang: LanguageCode,
    target_id: i64,
    sub_page: usize,
    page: usize,
) -> ResponseResult<()> {
    let muted: Vec<String> = match db.get_muted_users_list(target_id).await {
        Ok(list) => list,
        Err(e) => {
            tracing::error!("Failed to load muted users for {}: {}", target_id, e);
            Vec::new()
        }
    };

    let user_name = format!("{}", target_id);
    let args = args!(name = user_name);
    let title = locales::get_text(lang.as_str(), "list-mute-title", args.as_ref());

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
        Some(back_btn(
            lang,
            "btn-back-user-actions",
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
