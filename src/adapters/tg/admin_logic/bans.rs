use crate::adapters::tg::keyboards::{back_btn, create_user_list_keyboard};
use crate::adapters::tg::search::{
    SearchContext, SearchListType, append_search_hint, set_search_context_raw,
};
use crate::core::callbacks::{AdminAction, CallbackAction, MenuAction};
use crate::core::types::LanguageCode;
use crate::infra::db::Database;
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;

pub async fn send_unban_list(
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
    let entries = match db.get_banned_users().await {
        Ok(list) => list,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load banned users");
            Vec::new()
        }
    };

    if entries.is_empty() {
        let req = bot.send_message(
            chat_id,
            locales::get_text(lang.as_str(), "list-ban-empty", None),
        );
        if let Some(reply_to) = reply_to {
            req.reply_to(reply_to).await?;
        } else {
            req.await?;
        }
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

    let base = locales::get_text(lang.as_str(), "list-unban-title", None);
    let text = append_search_hint(&base, lang);
    let req = bot.send_message(chat_id, text).reply_markup(keyboard);
    if let Some(reply_to) = reply_to {
        let msg = req.reply_to(reply_to).await?;
        set_search_context_raw(
            search_contexts,
            msg.chat.id,
            SearchContext {
                message_id: msg.id,
                list_type: SearchListType::Unban,
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
                list_type: SearchListType::Unban,
            },
        )
        .await;
    }
    Ok(())
}

pub async fn edit_unban_list(
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

    let base = locales::get_text(lang.as_str(), "list-unban-title", None);
    let text = append_search_hint(&base, lang);
    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .await?;
    set_search_context_raw(
        search_contexts,
        msg.chat.id,
        SearchContext {
            message_id: msg.id,
            list_type: SearchListType::Unban,
        },
    )
    .await;
    Ok(())
}
