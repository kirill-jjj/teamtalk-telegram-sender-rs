use crate::adapters::tg::handlers::search::{
    SearchContext, SearchListType, append_search_hint, set_search_context_raw,
};
use crate::adapters::tg::presenter::keyboards::{back_btn, create_user_list_keyboard};
use crate::core::callbacks::{AdminAction, CallbackAction, MenuAction};
use crate::core::types::LanguageCode;
use crate::infra::db::types::BanEntry;
use crate::infra::locales;
use teloxide_ng::prelude::*;
use teloxide_ng::sugar::request::RequestReplyExt;

pub async fn send_unban_list(
    bot: &Bot,
    chat_id: teloxide_ng::types::ChatId,
    entries: Vec<BanEntry>,
    search_contexts: &std::sync::Arc<
        tokio::sync::Mutex<
            std::collections::HashMap<
                teloxide_ng::types::ChatId,
                crate::adapters::tg::handlers::search::SearchContext,
            >,
        >,
    >,
    lang: LanguageCode,
    page: usize,
    reply_to: Option<teloxide_ng::types::MessageId>,
) -> ResponseResult<()> {
    if entries.is_empty() {
        let req = bot.send_message(
            chat_id,
            locales::get_text(lang.as_str(), locales::LocaleKey::ListBanEmpty, None),
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
                        .as_ref()
                        .map_or_else(|| "Unknown".to_string(), ToString::to_string)
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
            locales::LocaleKey::BtnBackMenu,
            CallbackAction::Menu(MenuAction::Who),
        )),
        lang,
    );

    let base = locales::get_text(lang.as_str(), locales::LocaleKey::ListUnbanTitle, None);
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
    entries: Vec<BanEntry>,
    search_contexts: &std::sync::Arc<
        tokio::sync::Mutex<
            std::collections::HashMap<
                teloxide_ng::types::ChatId,
                crate::adapters::tg::handlers::search::SearchContext,
            >,
        >,
    >,
    lang: LanguageCode,
    page: usize,
) -> ResponseResult<()> {
    if entries.is_empty() {
        bot.edit_message_text(
            msg.chat.id,
            msg.id,
            locales::get_text(lang.as_str(), locales::LocaleKey::ListBanEmpty, None),
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
                        .as_ref()
                        .map_or_else(|| "Unknown".to_string(), ToString::to_string)
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
            locales::LocaleKey::BtnBackMenu,
            CallbackAction::Menu(MenuAction::Who),
        )),
        lang,
    );

    let base = locales::get_text(lang.as_str(), locales::LocaleKey::ListUnbanTitle, None);
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
