use crate::adapters::tg::handlers::search::{SearchContext, SearchListType, set_search_context};
use crate::adapters::tg::presenter::settings::{
    RenderMuteListStringsArgs, render_mute_list_strings,
};
use crate::adapters::tg::utils::{TgErrorReporter, answer_callback};
use crate::app::services::tg_search_actions as tg_search_actions_service;
use crate::args;
use crate::core::types::{
    ActionStatus, AdminErrorContext, LanguageCode, MuteListMode, TelegramId, TtUsername,
};
use crate::infra::locales;
use teloxide::prelude::*;

use super::{AppState, MuteCtx};

pub(super) async fn handle_list(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    telegram_id: TelegramId,
    lang: LanguageCode,
    mode: MuteListMode,
    page: usize,
) -> ResponseResult<()> {
    let muted = load_muted_users(&state.db, telegram_id, mode.clone()).await;
    let guest_username = state
        .config
        .teamtalk
        .guest_username
        .as_ref()
        .map(TtUsername::as_str);
    render_mute_list_strings(RenderMuteListStringsArgs {
        bot,
        msg,
        lang,
        items: &muted,
        page,
        title_key: locales::LocaleKey::ListMuteTitle,
        guest_username,
        mode: mode.clone(),
    })
    .await?;
    set_search_context(
        state,
        msg.chat.id,
        SearchContext {
            message_id: msg.id,
            list_type: SearchListType::MuteLocal {
                telegram_id,
                mode: mode.clone(),
                page,
            },
        },
    )
    .await;
    Ok(())
}

pub(super) async fn handle_toggle(
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
        TgErrorReporter::new(ctx.bot, &ctx.state.config, ctx.telegram_id, ctx.lang)
            .check_db_err(&ctx.q.id.0, Err(e), AdminErrorContext::Callback)
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

    let muted = load_muted_users(&ctx.state.db, ctx.telegram_id, mode.clone()).await;
    let guest_username = ctx
        .state
        .config
        .teamtalk
        .guest_username
        .as_ref()
        .map(TtUsername::as_str);
    render_mute_list_strings(RenderMuteListStringsArgs {
        bot: ctx.bot,
        msg: ctx.msg,
        lang: ctx.lang,
        items: &muted,
        page,
        title_key: locales::LocaleKey::ListMuteTitle,
        guest_username,
        mode,
    })
    .await
}

async fn load_muted_users(
    db: &crate::infra::db::Database,
    telegram_id: TelegramId,
    mode: MuteListMode,
) -> Vec<TtUsername> {
    tg_search_actions_service::list_muted_users(db, telegram_id, mode).await
}
