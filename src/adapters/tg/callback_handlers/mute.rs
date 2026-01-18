use crate::adapters::tg::settings_logic::{
    RenderMuteListArgs, RenderMuteListStringsArgs, render_mute_list, render_mute_list_strings,
    send_mute_menu,
};
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{answer_callback, check_db_err, notify_admin_error};
use crate::app::services::user_settings as user_settings_service;
use crate::args;
use crate::core::callbacks::MuteAction;
use crate::core::types::{AdminErrorContext, LanguageCode, MuteListMode, TtCommand};
use crate::infra::locales;
use teamtalk::types::UserAccount;
use teloxide::prelude::*;

pub async fn handle_mute(
    bot: Bot,
    q: CallbackQuery,
    state: AppState,
    action: MuteAction,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let Some(teloxide::types::MaybeInaccessibleMessage::Regular(msg)) = &q.message else {
        return Ok(());
    };
    let telegram_id = tg_user_id_i64(q.from.id.0);
    let ctx = MuteCtx {
        bot: &bot,
        q: &q,
        msg,
        state: &state,
        telegram_id,
        lang,
    };

    match action {
        MuteAction::ModeSet { mode } => {
            handle_mode_set(&bot, &q, &state, msg, telegram_id, lang, mode).await?;
        }
        MuteAction::Menu { mode } => {
            let has_guest = state.config.teamtalk.guest_username.is_some();
            send_mute_menu(&bot, msg, lang, mode, has_guest).await?;
        }
        MuteAction::List { page } => {
            handle_list(&bot, msg, &state, telegram_id, lang, page).await?;
        }
        MuteAction::Toggle { username, page } => {
            handle_toggle(&ctx, username.to_string(), page).await?;
        }
        MuteAction::ServerList { page } => {
            handle_server_list(&bot, msg, &state, telegram_id, lang, page).await?;
        }
        MuteAction::ServerToggle { username, page } => {
            handle_server_toggle(&ctx, username.to_string(), page).await?;
        }
    }

    Ok(())
}

async fn handle_mode_set(
    bot: &Bot,
    q: &CallbackQuery,
    state: &AppState,
    msg: &Message,
    telegram_id: i64,
    lang: LanguageCode,
    mode: crate::core::types::MuteListMode,
) -> ResponseResult<()> {
    if check_db_err(
        bot,
        &q.id.0,
        state.db.update_mute_mode(telegram_id, mode.clone()).await,
        &state.config,
        telegram_id,
        AdminErrorContext::Callback,
        lang,
    )
    .await?
    {
        return Ok(());
    }
    answer_callback(
        bot,
        &q.id,
        locales::get_text(
            lang.as_str(),
            "toast-mute-mode-set",
            args!(mode = mode.to_string()).as_ref(),
        ),
        false,
    )
    .await?;
    let has_guest = state.config.teamtalk.guest_username.is_some();
    send_mute_menu(bot, msg, lang, mode, has_guest).await
}

async fn handle_list(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    telegram_id: i64,
    lang: LanguageCode,
    page: usize,
) -> ResponseResult<()> {
    let muted = load_muted_users(&state.db, telegram_id).await;
    let mode = load_mute_mode(&state.db, telegram_id, lang).await;
    let guest_username = state.config.teamtalk.guest_username.as_deref();
    render_mute_list_strings(RenderMuteListStringsArgs {
        bot,
        msg,
        lang,
        items: &muted,
        page,
        title_key: "list-mute-title",
        guest_username,
        mode,
    })
    .await
}

async fn handle_toggle(ctx: &MuteCtx<'_>, username: String, page: usize) -> ResponseResult<()> {
    if let Err(e) = ctx
        .state
        .db
        .toggle_muted_user(ctx.telegram_id, username.as_str())
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

    let args = args!(user = username.clone(), action = "toggled");
    answer_callback(
        ctx.bot,
        &ctx.q.id,
        locales::get_text(ctx.lang.as_str(), "toast-user-muted", args.as_ref()),
        false,
    )
    .await?;

    let muted = load_muted_users(&ctx.state.db, ctx.telegram_id).await;
    let mode = load_mute_mode(&ctx.state.db, ctx.telegram_id, ctx.lang).await;
    let guest_username = ctx.state.config.teamtalk.guest_username.as_deref();
    render_mute_list_strings(RenderMuteListStringsArgs {
        bot: ctx.bot,
        msg: ctx.msg,
        lang: ctx.lang,
        items: &muted,
        page,
        title_key: "list-mute-title",
        guest_username,
        mode,
    })
    .await
}

async fn handle_server_list(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    telegram_id: i64,
    lang: LanguageCode,
    page: usize,
) -> ResponseResult<()> {
    request_accounts(bot, state, telegram_id, lang).await;

    let accounts = load_accounts(&state.user_accounts);
    let guest_username = state.config.teamtalk.guest_username.as_deref();
    let mode = load_mute_mode(&state.db, telegram_id, lang).await;
    render_mute_list(RenderMuteListArgs {
        bot,
        msg,
        db: &state.db,
        telegram_id,
        lang,
        accounts: &accounts,
        page,
        title_key: "list-all-accs-title",
        guest_username,
        mode,
    })
    .await
}

async fn handle_server_toggle(
    ctx: &MuteCtx<'_>,
    username: String,
    page: usize,
) -> ResponseResult<()> {
    if let Err(e) = ctx
        .state
        .db
        .toggle_muted_user(ctx.telegram_id, username.as_str())
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

    let args = args!(user = username.clone(), action = "toggled");
    answer_callback(
        ctx.bot,
        &ctx.q.id,
        locales::get_text(ctx.lang.as_str(), "toast-user-muted", args.as_ref()),
        false,
    )
    .await?;

    let accounts = load_accounts(&ctx.state.user_accounts);
    let guest_username = ctx.state.config.teamtalk.guest_username.as_deref();
    let mode = load_mute_mode(&ctx.state.db, ctx.telegram_id, ctx.lang).await;
    render_mute_list(RenderMuteListArgs {
        bot: ctx.bot,
        msg: ctx.msg,
        db: &ctx.state.db,
        telegram_id: ctx.telegram_id,
        lang: ctx.lang,
        accounts: &accounts,
        page,
        title_key: "list-all-accs-title",
        guest_username,
        mode,
    })
    .await
}

struct MuteCtx<'a> {
    bot: &'a Bot,
    q: &'a CallbackQuery,
    msg: &'a Message,
    state: &'a AppState,
    telegram_id: i64,
    lang: LanguageCode,
}

async fn request_accounts(bot: &Bot, state: &AppState, telegram_id: i64, lang: LanguageCode) {
    if let Err(e) = state.tx_tt.send(TtCommand::LoadAccounts) {
        tracing::error!(error = %e, "Failed to request TT accounts");
        notify_admin_error(
            bot,
            &state.config,
            telegram_id,
            AdminErrorContext::TtCommand,
            &e.to_string(),
            lang,
        )
        .await;
    }
}

async fn load_muted_users(db: &crate::infra::db::Database, telegram_id: i64) -> Vec<String> {
    db.get_muted_users_list(telegram_id)
        .await
        .unwrap_or_else(|e| {
            tracing::error!(telegram_id, error = %e, "Failed to load muted users");
            Vec::new()
        })
}

async fn load_mute_mode(
    db: &crate::infra::db::Database,
    telegram_id: i64,
    lang: LanguageCode,
) -> MuteListMode {
    let settings = user_settings_service::get_or_create(db, telegram_id, lang).await;
    settings
        .map(|s| user_settings_service::parse_mute_list_mode(&s.mute_list_mode))
        .unwrap_or(MuteListMode::Blacklist)
}

fn load_accounts(
    user_accounts: &std::sync::RwLock<std::collections::HashMap<String, UserAccount>>,
) -> Vec<UserAccount> {
    let mut accounts: Vec<UserAccount> = user_accounts
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .values()
        .cloned()
        .collect();
    accounts.sort_by(|a, b| a.username.to_lowercase().cmp(&b.username.to_lowercase()));
    accounts
}

fn tg_user_id_i64(user_id: u64) -> i64 {
    i64::try_from(user_id).unwrap_or(i64::MAX)
}
