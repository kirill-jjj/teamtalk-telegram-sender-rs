use crate::args;
use crate::locales;
use crate::tg_bot::callbacks_types::MuteAction;
use crate::tg_bot::settings_logic::{render_mute_list, render_mute_list_strings, send_mute_menu};
use crate::tg_bot::state::AppState;
use crate::tg_bot::utils::{check_db_err, notify_admin_error};
use crate::types::{LanguageCode, TtCommand};
use teamtalk::types::UserAccount;
use teloxide::prelude::*;

pub async fn handle_mute(
    bot: Bot,
    q: CallbackQuery,
    state: AppState,
    action: MuteAction,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let msg = match &q.message {
        Some(teloxide::types::MaybeInaccessibleMessage::Regular(m)) => m,
        _ => return Ok(()),
    };
    let telegram_id = q.from.id.0 as i64;
    let db = &state.db;
    let config = &state.config;

    match action {
        MuteAction::ModeSet { mode } => {
            if check_db_err(
                &bot,
                &q.id.0,
                db.update_mute_mode(telegram_id, mode.clone()).await,
                config,
                telegram_id,
                "admin-error-context-callback",
                lang,
            )
            .await?
            {
                return Ok(());
            }
            bot.answer_callback_query(q.id)
                .text(locales::get_text(
                    lang.as_str(),
                    "toast-mute-mode-set",
                    args!(mode = mode.to_string()).as_ref(),
                ))
                .await?;
            send_mute_menu(&bot, msg, lang, mode).await?;
        }
        MuteAction::Menu { mode } => {
            send_mute_menu(&bot, msg, lang, mode).await?;
        }
        MuteAction::List { page } => {
            let muted = match db.get_muted_users_list(telegram_id).await {
                Ok(list) => list,
                Err(e) => {
                    tracing::error!("Failed to load muted users for {}: {}", telegram_id, e);
                    Vec::new()
                }
            };
            let guest_username = state.config.teamtalk.guest_username.as_deref();
            render_mute_list_strings(
                &bot,
                msg,
                telegram_id,
                lang,
                &muted,
                page,
                false,
                "list-mute-title",
                guest_username,
            )
            .await?;
        }
        MuteAction::Toggle { username, page } => {
            if let Err(e) = toggle_mute(db, telegram_id, &username).await {
                check_db_err(
                    &bot,
                    &q.id.0,
                    Err(e),
                    config,
                    telegram_id,
                    "admin-error-context-callback",
                    lang,
                )
                .await?;
                return Ok(());
            }

            let args = args!(user = username.clone(), action = "toggled");
            bot.answer_callback_query(q.id)
                .text(locales::get_text(
                    lang.as_str(),
                    "toast-user-muted",
                    args.as_ref(),
                ))
                .await?;

            let muted = db
                .get_muted_users_list(telegram_id)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to load muted users for {}: {}", telegram_id, e);
                    Vec::new()
                });
            let guest_username = state.config.teamtalk.guest_username.as_deref();
            render_mute_list_strings(
                &bot,
                msg,
                telegram_id,
                lang,
                &muted,
                page,
                false,
                "list-mute-title",
                guest_username,
            )
            .await?;
        }
        MuteAction::ServerList { page } => {
            if let Err(e) = state.tx_tt.send(TtCommand::LoadAccounts) {
                tracing::error!("Failed to request TT accounts: {}", e);
                notify_admin_error(
                    &bot,
                    config,
                    telegram_id,
                    "admin-error-context-tt-command",
                    &e.to_string(),
                    lang,
                )
                .await;
            }
            let user_accounts = &state.user_accounts;
            let mut accounts: Vec<UserAccount> =
                user_accounts.iter().map(|kv| kv.value().clone()).collect();
            accounts.sort_by(|a, b| a.username.to_lowercase().cmp(&b.username.to_lowercase()));

            let guest_username = state.config.teamtalk.guest_username.as_deref();
            render_mute_list(
                &bot,
                msg,
                db,
                telegram_id,
                lang,
                &accounts,
                page,
                "list-all-accs-title",
                guest_username,
            )
            .await?;
        }
        MuteAction::ServerToggle { username, page } => {
            if let Err(e) = toggle_mute(db, telegram_id, &username).await {
                check_db_err(
                    &bot,
                    &q.id.0,
                    Err(e),
                    config,
                    telegram_id,
                    "admin-error-context-callback",
                    lang,
                )
                .await?;
                return Ok(());
            }

            let args = args!(user = username.clone(), action = "toggled");
            bot.answer_callback_query(q.id)
                .text(locales::get_text(
                    lang.as_str(),
                    "toast-user-muted",
                    args.as_ref(),
                ))
                .await?;

            let user_accounts = &state.user_accounts;
            let mut accounts: Vec<UserAccount> =
                user_accounts.iter().map(|kv| kv.value().clone()).collect();
            accounts.sort_by(|a, b| a.username.to_lowercase().cmp(&b.username.to_lowercase()));
            let guest_username = state.config.teamtalk.guest_username.as_deref();

            render_mute_list(
                &bot,
                msg,
                db,
                telegram_id,
                lang,
                &accounts,
                page,
                "list-all-accs-title",
                guest_username,
            )
            .await?;
        }
    }

    Ok(())
}

async fn toggle_mute(
    db: &crate::db::Database,
    telegram_id: i64,
    username: &str,
) -> anyhow::Result<()> {
    let count: i32 = sqlx::query_scalar("SELECT count(*) FROM muted_users WHERE user_settings_telegram_id = ? AND muted_teamtalk_username = ?")
        .bind(telegram_id).bind(username).fetch_one(&db.pool).await?;

    let is_muted = count > 0;

    let query = if is_muted {
        "DELETE FROM muted_users WHERE user_settings_telegram_id = ? AND muted_teamtalk_username = ?"
    } else {
        "INSERT INTO muted_users (user_settings_telegram_id, muted_teamtalk_username) VALUES (?, ?)"
    };

    sqlx::query(query)
        .bind(telegram_id)
        .bind(username)
        .execute(&db.pool)
        .await?;

    Ok(())
}
