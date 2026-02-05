use crate::adapters::tg::presenter::admin::bans as bans_logic;
use crate::adapters::tg::presenter::admin::subscriber_settings as subscriber_settings_logic;
use crate::adapters::tg::presenter::admin::subscribers as subscribers_logic;
use crate::adapters::tg::presenter::settings::{RenderMuteListArgs, RenderMuteListStringsArgs};
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{TgErrorReporter, send_text_key, telegram_id_from_user};
use crate::app::services::tg_admin as tg_admin_service;
use crate::app::services::tg_search_actions as tg_search_actions_service;
use crate::args;
use crate::core::callbacks::{AdminAction, CallbackAction, MuteAction, SubAction};
use crate::core::types::{
    ActionStatus, AdminErrorContext, LanguageCode, MuteListMode, TelegramId, TtCommand, TtUsername,
};
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;

use super::context::{SearchCandidate, SearchContext, SearchListType};

pub(super) async fn handle_single_match(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    ctx: &SearchContext,
    candidate: &SearchCandidate,
    lang: LanguageCode,
) -> ResponseResult<bool> {
    match &ctx.list_type {
        SearchListType::Kick => handle_kick(bot, msg, state, candidate, lang).await,
        SearchListType::Ban => handle_ban(bot, msg, state, candidate, lang).await,
        SearchListType::Unban => handle_unban(bot, msg, state, candidate, lang).await,
        SearchListType::Subscribers => handle_subscribers(bot, msg, state, candidate, lang).await,
        SearchListType::MuteServer {
            telegram_id,
            mode,
            page,
        } => {
            handle_mute(HandleMuteArgs {
                bot,
                msg,
                state,
                candidate,
                telegram_id: *telegram_id,
                mode: mode.clone(),
                page: *page,
                server_list: true,
                lang,
            })
            .await
        }
        SearchListType::MuteLocal {
            telegram_id,
            mode,
            page,
        } => {
            handle_mute(HandleMuteArgs {
                bot,
                msg,
                state,
                candidate,
                telegram_id: *telegram_id,
                mode: mode.clone(),
                page: *page,
                server_list: false,
                lang,
            })
            .await
        }
        SearchListType::SubMuteView { .. } => Ok(false),
        SearchListType::LinkList { sub_id, page } => {
            handle_link_list(bot, msg, state, candidate, *sub_id, *page, lang).await
        }
    }
}

async fn handle_kick(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    candidate: &SearchCandidate,
    lang: LanguageCode,
) -> ResponseResult<bool> {
    let CallbackAction::Admin(AdminAction::KickPerform { user_id }) = &candidate.action else {
        return Ok(false);
    };
    send_tt_command(
        bot,
        msg,
        state,
        TtCommand::KickUser { user_id: *user_id },
        lang,
    )
    .await?;
    Ok(true)
}

async fn handle_ban(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    candidate: &SearchCandidate,
    lang: LanguageCode,
) -> ResponseResult<bool> {
    let CallbackAction::Admin(AdminAction::BanPerform { user_id }) = &candidate.action else {
        return Ok(false);
    };
    send_tt_command(
        bot,
        msg,
        state,
        TtCommand::BanUser { user_id: *user_id },
        lang,
    )
    .await?;
    Ok(true)
}

async fn handle_unban(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    candidate: &SearchCandidate,
    lang: LanguageCode,
) -> ResponseResult<bool> {
    let CallbackAction::Admin(AdminAction::UnbanPerform { ban_db_id, page }) = &candidate.action
    else {
        return Ok(false);
    };
    if let Err(e) = tg_search_actions_service::remove_ban(&state.db, *ban_db_id).await {
        if let Some(requester_id) = requester_id_from_message(msg, "search_unban") {
            TgErrorReporter::new(bot, &state.config, requester_id, lang)
                .notify(AdminErrorContext::Callback, &e.to_string())
                .await;
        }
        return Ok(true);
    }
    send_text_key(
        bot,
        msg.chat.id,
        lang,
        locales::LocaleKey::ToastUserUnbanned,
        Some(msg.id),
    )
    .await?;
    bans_logic::edit_unban_list(
        bot,
        msg,
        match tg_admin_service::list_ban_entries(&state.db).await {
            Ok(entries) => entries,
            Err(err) => {
                if let Some(requester_id) = requester_id_from_message(msg, "search_bans_reload") {
                    TgErrorReporter::new(bot, &state.config, requester_id, lang)
                        .notify(AdminErrorContext::Callback, &err.into_error().to_string())
                        .await;
                }
                Vec::new()
            }
        },
        &state.search_contexts,
        lang,
        *page,
    )
    .await?;
    Ok(true)
}

async fn handle_subscribers(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    candidate: &SearchCandidate,
    lang: LanguageCode,
) -> ResponseResult<bool> {
    let CallbackAction::Subscriber(SubAction::Details { sub_id, page }) = &candidate.action else {
        return Ok(false);
    };
    let Some(requester_id) = requester_id_from_message(msg, "search_subscribers") else {
        return Ok(true);
    };
    let is_main_admin = requester_id == state.config.telegram.admin_chat_id;
    let mut settings = subscribers_logic::default_user_settings(*sub_id);
    let mut is_admin = false;
    match tg_search_actions_service::load_subscriber_details(&state.db, *sub_id, LanguageCode::En)
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

struct HandleMuteArgs<'a> {
    bot: &'a Bot,
    msg: &'a Message,
    state: &'a AppState,
    candidate: &'a SearchCandidate,
    telegram_id: TelegramId,
    mode: MuteListMode,
    page: usize,
    server_list: bool,
    lang: LanguageCode,
}

async fn handle_mute(args: HandleMuteArgs<'_>) -> ResponseResult<bool> {
    let username = match &args.candidate.action {
        CallbackAction::Mute(
            MuteAction::ServerToggle { username, .. } | MuteAction::Toggle { username, .. },
        ) => username.clone(),
        _ => return Ok(false),
    };
    toggle_mute_and_render(
        args.bot,
        args.msg,
        args.state,
        ToggleMuteArgs {
            telegram_id: args.telegram_id,
            mode: args.mode,
            username,
            page: args.page,
            server_list: args.server_list,
            lang: args.lang,
        },
    )
    .await?;
    Ok(true)
}

async fn handle_link_list(
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
        if let Some(requester_id) = requester_id_from_message(msg, "search_link_tt") {
            TgErrorReporter::new(bot, &state.config, requester_id, lang)
                .notify(AdminErrorContext::Callback, &e.to_string())
                .await;
        }
        return Ok(true);
    }
    let args = args!(user = username.to_string());
    let text = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::ToastAccountLinked,
        args.as_ref(),
    );
    bot.send_message(msg.chat.id, text).reply_to(msg.id).await?;
    let settings =
        match tg_search_actions_service::load_user_settings(&state.db, sub_id, LanguageCode::En)
            .await
        {
            Ok(s) => s,
            Err(e) => {
                if let Some(requester_id) = requester_id_from_message(msg, "search_load_settings") {
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

struct ToggleMuteArgs {
    telegram_id: TelegramId,
    mode: MuteListMode,
    username: TtUsername,
    page: usize,
    server_list: bool,
    lang: LanguageCode,
}

async fn toggle_mute_and_render(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    args: ToggleMuteArgs,
) -> ResponseResult<()> {
    if let Err(e) = tg_search_actions_service::toggle_mute(
        &state.db,
        args.telegram_id,
        args.mode.clone(),
        &args.username,
    )
    .await
    {
        TgErrorReporter::new(bot, &state.config, args.telegram_id, args.lang)
            .notify(AdminErrorContext::Callback, &e.to_string())
            .await;
        return Ok(());
    }

    let fmt_args = args!(
        user = args.username.to_string(),
        action = ActionStatus::Toggled.as_str()
    );
    let text = locales::get_text(
        args.lang.as_str(),
        locales::LocaleKey::ToastUserMuted,
        fmt_args.as_ref(),
    );
    bot.send_message(msg.chat.id, text).reply_to(msg.id).await?;

    if args.server_list {
        let accounts = tg_search_actions_service::list_user_accounts(&state.state).await;
        let muted_users = tg_search_actions_service::list_muted_users(
            &state.db,
            args.telegram_id,
            args.mode.clone(),
        )
        .await;
        let guest_username = state
            .config
            .teamtalk
            .guest_username
            .as_ref()
            .map(TtUsername::as_str);
        let render_args = RenderMuteListArgs {
            bot,
            msg,
            lang: args.lang,
            accounts: &accounts,
            page: args.page,
            title_key: locales::LocaleKey::ListAllAccsTitle,
            guest_username,
            mode: args.mode,
            muted_users: &muted_users,
        };
        crate::adapters::tg::presenter::settings::render_mute_list(render_args).await?;
    } else {
        let muted = tg_search_actions_service::list_muted_users(
            &state.db,
            args.telegram_id,
            args.mode.clone(),
        )
        .await;
        let guest_username = state
            .config
            .teamtalk
            .guest_username
            .as_ref()
            .map(TtUsername::as_str);
        let render_args = RenderMuteListStringsArgs {
            bot,
            msg,
            lang: args.lang,
            items: &muted,
            page: args.page,
            title_key: locales::LocaleKey::ListMuteTitle,
            guest_username,
            mode: args.mode,
        };
        crate::adapters::tg::presenter::settings::render_mute_list_strings(render_args).await?;
    }
    Ok(())
}

async fn send_tt_command(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    cmd: TtCommand,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let admin_id = requester_id_from_message(msg, "search_send_tt_command");
    if let Err(e) = state.tx_tt.send(cmd).await
        && let Some(admin_id) = admin_id
    {
        TgErrorReporter::new(bot, &state.config, admin_id, lang)
            .notify(AdminErrorContext::TtCommand, &e.to_string())
            .await;
    }
    send_text_key(
        bot,
        msg.chat.id,
        lang,
        locales::LocaleKey::ToastCommandSent,
        Some(msg.id),
    )
    .await?;
    Ok(())
}

fn requester_id_from_message(msg: &Message, context: &'static str) -> Option<TelegramId> {
    let user = msg.from.as_ref()?;
    telegram_id_from_user(user, context)
}
