use crate::adapters::tg::handlers::search::{
    SearchContext, SearchListType, append_search_hint, set_search_context_raw,
};
use crate::adapters::tg::presenter::keyboards::{
    back_btn, back_button, callback_button, create_user_list_keyboard,
};
use crate::args;
use crate::core::callbacks::{CallbackAction, SubAction};
use crate::core::types::{LanguageCode, TelegramId, TtUsername};
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardMarkup;

pub async fn send_sub_manage_tt_menu(
    bot: &Bot,
    msg: &Message,
    lang: LanguageCode,
    sub_id: TelegramId,
    return_page: usize,
    tt_user: Option<TtUsername>,
) -> ResponseResult<()> {
    let args = args!(id = sub_id.to_string());
    let text = locales::get_text_or_log(
        lang.as_str(),
        locales::LocaleKey::SubManageTtTitle,
        args.as_ref(),
    );

    let mut buttons = vec![];
    if let Some(user) = tt_user {
        let args_btn = args!(user = user.to_string());
        buttons.push(vec![callback_button(
            locales::get_text_or_log(
                lang.as_str(),
                locales::LocaleKey::BtnUnlink,
                args_btn.as_ref(),
            ),
            CallbackAction::Subscriber(SubAction::Unlink {
                sub_id,
                page: return_page,
            }),
        )]);
    }
    buttons.push(vec![callback_button(
        locales::get_text_or_log(lang.as_str(), locales::LocaleKey::BtnLinkNew, None),
        CallbackAction::Subscriber(SubAction::LinkList {
            sub_id,
            page: return_page,
            list_page: 0,
        }),
    )]);
    buttons.push(vec![back_button(
        lang,
        locales::LocaleKey::BtnBackUserActions,
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

pub struct SubLinkAccountListArgs<'a> {
    pub bot: &'a Bot,
    pub msg: &'a Message,
    pub accounts: Vec<teamtalk::types::UserAccount>,
    pub search_contexts: &'a std::sync::Arc<
        tokio::sync::Mutex<
            std::collections::HashMap<
                teloxide::types::ChatId,
                crate::adapters::tg::handlers::search::SearchContext,
            >,
        >,
    >,
    pub lang: LanguageCode,
    pub target_id: TelegramId,
    pub sub_page: usize,
    pub page: usize,
}

pub async fn send_sub_link_account_list(args: SubLinkAccountListArgs<'_>) -> ResponseResult<()> {
    let SubLinkAccountListArgs {
        bot,
        msg,
        accounts,
        search_contexts,
        lang,
        target_id,
        sub_page,
        page,
    } = args;
    let mut accounts = accounts;
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
            locales::LocaleKey::BtnBackManageAcc,
            CallbackAction::Subscriber(SubAction::ManageTt {
                sub_id: target_id,
                page: sub_page,
            }),
        )),
        lang,
    );

    let args = args!(id = target_id.to_string());
    let base = locales::get_text_or_log(
        lang.as_str(),
        locales::LocaleKey::ListLinkTitle,
        args.as_ref(),
    );
    let text = append_search_hint(&base, lang);

    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(keyboard)
        .await?;
    set_search_context_raw(
        search_contexts,
        msg.chat.id,
        SearchContext {
            message_id: msg.id,
            list_type: SearchListType::LinkList {
                sub_id: target_id,
                page: sub_page,
            },
        },
    )
    .await;
    Ok(())
}

pub struct SubMuteListArgs<'a> {
    pub bot: &'a Bot,
    pub msg: &'a Message,
    pub search_contexts: &'a std::sync::Arc<
        tokio::sync::Mutex<
            std::collections::HashMap<
                teloxide::types::ChatId,
                crate::adapters::tg::handlers::search::SearchContext,
            >,
        >,
    >,
    pub lang: LanguageCode,
    pub target_id: TelegramId,
    pub sub_page: usize,
    pub page: usize,
    pub muted: Vec<TtUsername>,
}

pub async fn send_sub_mute_list(args: SubMuteListArgs<'_>) -> ResponseResult<()> {
    let SubMuteListArgs {
        bot,
        msg,
        search_contexts,
        lang,
        target_id,
        sub_page,
        page,
        muted,
    } = args;
    let user_name = format!("{target_id}");
    let args = args!(name = user_name);
    let base = locales::get_text_or_log(
        lang.as_str(),
        locales::LocaleKey::ListMuteTitleFor,
        args.as_ref(),
    );
    let title = append_search_hint(&base, lang);

    let keyboard = create_user_list_keyboard(
        &muted,
        page,
        |username| (username.to_string(), CallbackAction::NoOp),
        |p| {
            CallbackAction::Subscriber(SubAction::MuteView {
                sub_id: target_id,
                page: sub_page,
                view_page: p,
            })
        },
        Some(back_btn(
            lang,
            locales::LocaleKey::BtnBackUserActions,
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
    set_search_context_raw(
        search_contexts,
        msg.chat.id,
        SearchContext {
            message_id: msg.id,
            list_type: SearchListType::SubMuteView {
                sub_id: target_id,
                sub_page,
                view_page: page,
            },
        },
    )
    .await;
    Ok(())
}
