use crate::adapters::tg::presenter::admin::subscriber_settings as subscriber_settings_logic;
use crate::adapters::tg::presenter::admin::subscribers as subscribers_logic;
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::subscriber_notify::{AdminActor, SubscriberChangeKind};
use crate::adapters::tg::utils::TgErrorReporter;
use crate::app::services::tg_search_actions as tg_search_actions_service;
use crate::args;
use crate::core::callbacks::{CallbackAction, SubAction};
use crate::core::types::{AdminErrorContext, LanguageCode, TelegramId};
use crate::infra::locales;
use teloxide_ng::prelude::*;
use teloxide_ng::sugar::request::RequestReplyExt;

use crate::adapters::tg::handlers::search::context::SearchCandidate;

pub(super) async fn handle_subscribers(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    candidate: &SearchCandidate,
    lang: LanguageCode,
) -> ResponseResult<bool> {
    let CallbackAction::Subscriber(SubAction::Details { sub_id, page }) = &candidate.action else {
        return Ok(false);
    };
    let Some(requester_id) = super::requester_id_from_message(msg, "search_subscribers") else {
        return Ok(true);
    };
    let is_main_admin = requester_id == state.config.telegram.admin_chat_id;
    let mut settings = subscribers_logic::default_user_settings(*sub_id);
    let mut is_admin = false;
    match tg_search_actions_service::load_subscriber_details(
        &state.db,
        *sub_id,
        state.config.general.default_lang,
    )
    .await
    {
        Ok(details) => {
            settings = details.settings;
            is_admin = details.is_admin;
        }
        Err(e) => {
            TgErrorReporter::new(bot, &state.config, requester_id, lang)
                .notify(AdminErrorContext::Callback, &e.to_string())
                .await;
        }
    }
    subscribers_logic::send_subscriber_details(subscribers_logic::SubscriberDetailsArgs {
        bot,
        msg,
        lang,
        sub_id: *sub_id,
        return_page: *page,
        is_main_admin,
        admin_chat_id: state.config.telegram.admin_chat_id,
        settings,
        is_admin,
    })
    .await?;
    Ok(true)
}

pub(super) async fn handle_link_list(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    candidate: &SearchCandidate,
    sub_id: TelegramId,
    page: usize,
    lang: LanguageCode,
) -> ResponseResult<bool> {
    let CallbackAction::Subscriber(SubAction::LinkPerform { username, .. }) = &candidate.action
    else {
        return Ok(false);
    };
    if let Err(e) = tg_search_actions_service::link_tt(&state.db, sub_id, username).await {
        if let Some(requester_id) = super::requester_id_from_message(msg, "search_link_tt") {
            TgErrorReporter::new(bot, &state.config, requester_id, lang)
                .notify(AdminErrorContext::Callback, &e.to_string())
                .await;
        }
        return Ok(true);
    }
    let actor = msg
        .from
        .as_ref()
        .and_then(AdminActor::from_telegram_user)
        .or_else(|| {
            super::requester_id_from_message(msg, "search_link_tt_actor").map(AdminActor::fallback)
        });
    if let Some(actor) = actor {
        crate::adapters::tg::subscriber_notify::notify_subscriber_change(
            bot,
            &state.db,
            sub_id,
            &actor,
            SubscriberChangeKind::Linked(username.clone()),
        )
        .await;
    } else {
        tracing::warn!("Skipping subscriber change notify: actor is missing");
    }
    let args = args!(user = username.to_string());
    let text = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::ToastAccountLinked,
        args.as_ref(),
    );
    bot.send_message(msg.chat.id, text).reply_to(msg.id).await?;
    let settings = match tg_search_actions_service::load_user_settings(
        &state.db,
        sub_id,
        state.config.general.default_lang,
    )
    .await
    {
        Ok(s) => s,
        Err(e) => {
            if let Some(requester_id) =
                super::requester_id_from_message(msg, "search_load_settings")
            {
                TgErrorReporter::new(bot, &state.config, requester_id, lang)
                    .notify(AdminErrorContext::Callback, &e.to_string())
                    .await;
            }
            return Ok(true);
        }
    };
    subscriber_settings_logic::send_sub_manage_tt_menu(
        bot,
        msg,
        lang,
        sub_id,
        page,
        settings.teamtalk_username,
    )
    .await?;
    Ok(true)
}
