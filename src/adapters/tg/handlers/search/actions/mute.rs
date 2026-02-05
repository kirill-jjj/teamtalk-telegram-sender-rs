use crate::adapters::tg::presenter::settings::{RenderMuteListArgs, RenderMuteListStringsArgs};
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::TgErrorReporter;
use crate::app::services::tg_search_actions as tg_search_actions_service;
use crate::args;
use crate::core::callbacks::{CallbackAction, MuteAction};
use crate::core::types::{
    ActionStatus, AdminErrorContext, LanguageCode, MuteListMode, TelegramId, TtUsername,
};
use crate::infra::locales;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;

use crate::adapters::tg::handlers::search::context::SearchCandidate;

pub(super) struct HandleMuteArgs<'a> {
    pub(super) bot: &'a Bot,
    pub(super) msg: &'a Message,
    pub(super) state: &'a AppState,
    pub(super) candidate: &'a SearchCandidate,
    pub(super) telegram_id: TelegramId,
    pub(super) mode: MuteListMode,
    pub(super) page: usize,
    pub(super) server_list: bool,
    pub(super) lang: LanguageCode,
}

struct ToggleMuteArgs {
    telegram_id: TelegramId,
    mode: MuteListMode,
    username: TtUsername,
    page: usize,
    server_list: bool,
    lang: LanguageCode,
}

pub(super) async fn handle_mute(args: HandleMuteArgs<'_>) -> ResponseResult<bool> {
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
