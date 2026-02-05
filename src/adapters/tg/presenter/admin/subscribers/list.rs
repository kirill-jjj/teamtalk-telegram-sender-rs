use crate::adapters::tg::handlers::search::{
    SearchContext, SearchListType, append_search_hint, set_search_context_raw,
};
use crate::adapters::tg::presenter::keyboards::{back_btn, create_user_list_keyboard};
use crate::core::callbacks::{AdminAction, CallbackAction, MenuAction, SubAction};
use crate::core::types::LanguageCode;
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;

use super::display::SubDisplayInfo;

pub async fn send_subscribers_list(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    display_list: Vec<SubDisplayInfo>,
    search_contexts: &std::sync::Arc<
        tokio::sync::Mutex<
            std::collections::HashMap<
                teloxide::types::ChatId,
                crate::adapters::tg::handlers::search::SearchContext,
            >,
        >,
    >,
    lang: LanguageCode,
    page: usize,
    reply_to: Option<teloxide::types::MessageId>,
) -> ResponseResult<()> {
    if display_list.is_empty() {
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

    let keyboard = create_user_list_keyboard(
        &display_list,
        page,
        |s| build_list_entry(s, page),
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
    display_list: Vec<SubDisplayInfo>,
    search_contexts: &std::sync::Arc<
        tokio::sync::Mutex<
            std::collections::HashMap<
                teloxide::types::ChatId,
                crate::adapters::tg::handlers::search::SearchContext,
            >,
        >,
    >,
    lang: LanguageCode,
    page: usize,
) -> ResponseResult<()> {
    if display_list.is_empty() {
        bot.edit_message_text(
            msg.chat.id,
            msg.id,
            locales::get_text(lang.as_str(), locales::LocaleKey::ListSubsEmpty, None),
        )
        .await?;
        return Ok(());
    }

    let keyboard = create_user_list_keyboard(
        &display_list,
        page,
        |s| build_list_entry(s, page),
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

fn build_list_entry(s: &SubDisplayInfo, page: usize) -> (String, CallbackAction) {
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
}
