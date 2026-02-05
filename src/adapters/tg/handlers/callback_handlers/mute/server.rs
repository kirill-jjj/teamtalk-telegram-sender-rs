use crate::adapters::tg::handlers::search::{SearchContext, SearchListType, set_search_context};
use crate::adapters::tg::presenter::settings::{RenderMuteListArgs, render_mute_list};
use crate::adapters::tg::utils::{answer_callback, check_db_err};
use crate::app::services::tg_search_actions as tg_search_actions_service;
use crate::app::services::tg_sub_links as tg_sub_links_service;
use crate::args;
use crate::core::types::{
    ActionStatus, AdminErrorContext, LanguageCode, MuteListMode, TelegramId, TtUsername,
};
use crate::infra::locales;
use teloxide::prelude::*;

use super::{AppState, MuteCtx};

pub(super) async fn handle_server_list(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    telegram_id: TelegramId,
    lang: LanguageCode,
    mode: MuteListMode,
    page: usize,
) -> ResponseResult<()> {
    let accounts = tg_sub_links_service::load_accounts(&state.tx_tt, &state.state).await;
    let muted_users =
        tg_search_actions_service::list_muted_users(&state.db, telegram_id, mode.clone()).await;
    let guest_username = state
        .config
        .teamtalk
        .guest_username
        .as_ref()
        .map(TtUsername::as_str);
    render_mute_list(RenderMuteListArgs {
        bot,
        msg,
        lang,
        accounts: &accounts,
        page,
        title_key: locales::LocaleKey::ListAllAccsTitle,
        guest_username,
        mode: mode.clone(),
        muted_users: &muted_users,
    })
    .await?;
    set_search_context(
        state,
        msg.chat.id,
        SearchContext {
            message_id: msg.id,
            list_type: SearchListType::MuteServer {
                telegram_id,
                mode: mode.clone(),
                page,
            },
        },
    )
    .await;
    Ok(())
}

pub(super) async fn handle_server_toggle(
    ctx: &MuteCtx<'_>,
    mode: MuteListMode,
    username: TtUsername,
    page: usize,
) -> ResponseResult<()> {
    if let Err(e) = tg_search_actions_service::toggle_mute(
        &ctx.state.db,
        ctx.telegram_id,
        mode.clone(),
        &username,
    )
    .await
    {
        check_db_err(
            ctx.bot,
            &ctx.q.id.0,
            Err(e),
            &ctx.state.config,
            ctx.telegram_id,
            AdminErrorContext::Callback,
            ctx.lang,
        )
        .await?;
        return Ok(());
    }

    let args = args!(
        user = username.to_string(),
        action = ActionStatus::Toggled.as_str()
    );
    answer_callback(
        ctx.bot,
        &ctx.q.id,
        locales::get_text(
            ctx.lang.as_str(),
            locales::LocaleKey::ToastUserMuted,
            args.as_ref(),
        ),
        false,
    )
    .await?;

    let accounts = tg_sub_links_service::load_accounts(&ctx.state.tx_tt, &ctx.state.state).await;
    let muted_users =
        tg_search_actions_service::list_muted_users(&ctx.state.db, ctx.telegram_id, mode.clone())
            .await;
    let guest_username = ctx
        .state
        .config
        .teamtalk
        .guest_username
        .as_ref()
        .map(TtUsername::as_str);
    render_mute_list(RenderMuteListArgs {
        bot: ctx.bot,
        msg: ctx.msg,
        lang: ctx.lang,
        accounts: &accounts,
        page,
        title_key: locales::LocaleKey::ListAllAccsTitle,
        guest_username,
        mode,
        muted_users: &muted_users,
    })
    .await
}
