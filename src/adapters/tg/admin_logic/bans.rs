use crate::adapters::tg::keyboards::{back_btn, create_user_list_keyboard};
use crate::core::callbacks::{AdminAction, CallbackAction, MenuAction};
use crate::core::types::LanguageCode;
use crate::infra::db::Database;
use crate::infra::locales;
use teloxide::prelude::*;

pub async fn send_unban_list(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    db: &Database,
    lang: LanguageCode,
    page: usize,
) -> ResponseResult<()> {
    let entries = match db.get_banned_users().await {
        Ok(list) => list,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load banned users");
            Vec::new()
        }
    };

    if entries.is_empty() {
        bot.send_message(
            chat_id,
            locales::get_text(lang.as_str(), "list-ban-empty", None),
        )
        .await?;
        return Ok(());
    }

    let keyboard = create_user_list_keyboard(
        &entries,
        page,
        |e| {
            let name = e.telegram_id.map_or_else(
                || {
                    e.teamtalk_username
                        .clone()
                        .unwrap_or_else(|| "Unknown".to_string())
                },
                |tg| format!("{tg}"),
            );
            (
                name,
                CallbackAction::Admin(AdminAction::UnbanPerform {
                    ban_db_id: e.id,
                    page,
                }),
            )
        },
        |p| CallbackAction::Admin(AdminAction::UnbanList { page: p }),
        Some(back_btn(
            lang,
            "btn-back-menu",
            CallbackAction::Menu(MenuAction::Who),
        )),
        lang,
    );

    bot.send_message(
        chat_id,
        locales::get_text(lang.as_str(), "list-unban-title", None),
    )
    .reply_markup(keyboard)
    .await?;
    Ok(())
}

pub async fn edit_unban_list(
    bot: &Bot,
    msg: &Message,
    db: &Database,
    lang: LanguageCode,
    page: usize,
) -> ResponseResult<()> {
    let entries = match db.get_banned_users().await {
        Ok(list) => list,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load banned users");
            Vec::new()
        }
    };

    if entries.is_empty() {
        bot.edit_message_text(
            msg.chat.id,
            msg.id,
            locales::get_text(lang.as_str(), "list-ban-empty", None),
        )
        .await?;
        return Ok(());
    }

    let keyboard = create_user_list_keyboard(
        &entries,
        page,
        |e| {
            let name = e.telegram_id.map_or_else(
                || {
                    e.teamtalk_username
                        .clone()
                        .unwrap_or_else(|| "Unknown".to_string())
                },
                |tg| format!("{tg}"),
            );
            (
                name,
                CallbackAction::Admin(AdminAction::UnbanPerform {
                    ban_db_id: e.id,
                    page,
                }),
            )
        },
        |p| CallbackAction::Admin(AdminAction::UnbanList { page: p }),
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
        locales::get_text(lang.as_str(), "list-unban-title", None),
    )
    .reply_markup(keyboard)
    .await?;
    Ok(())
}
