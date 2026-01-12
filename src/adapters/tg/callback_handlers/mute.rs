use crate::adapters::tg::settings_logic::{
    RenderMuteListArgs, RenderMuteListStringsArgs, render_mute_list, render_mute_list_strings,
    send_mute_menu,
};
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{answer_callback, check_db_err, notify_admin_error};
use crate::app::services::mute as mute_service;
use crate::args;
use crate::core::callbacks::MuteAction;
use crate::core::types::{AdminErrorContext, LanguageCode, TtCommand};
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
                mute_service::update_mode(db, telegram_id, mode.clone()).await,
                config,
                telegram_id,
                AdminErrorContext::Callback,
                lang,
            )
            .await?
            {
                return Ok(());
            }
            answer_callback(
                &bot,
                &q.id,
                locales::get_text(
                    lang.as_str(),
                    "toast-mute-mode-set",
                    args!(mode = mode.to_string()).as_ref(),
                ),
                false,
            )
            .await?;
            send_mute_menu(&bot, msg, lang, mode).await?;
        }
        MuteAction::Menu { mode } => {
            send_mute_menu(&bot, msg, lang, mode).await?;
        }
        MuteAction::List { page } => {
            let muted = match mute_service::list_muted_users(db, telegram_id).await {
                Ok(list) => list,
                Err(e) => {
                    tracing::error!("Failed to load muted users for {}: {}", telegram_id, e);
                    Vec::new()
                }
            };
            let guest_username = state.config.teamtalk.guest_username.as_deref();
            render_mute_list_strings(RenderMuteListStringsArgs {
                bot: &bot,
                msg,
                lang,
                items: &muted,
                page,
                title_key: "list-mute-title",
                guest_username,
            })
            .await?;
        }
        MuteAction::Toggle { username, page } => {
            if let Err(e) = mute_service::toggle_mute(db, telegram_id, username.as_str()).await {
                check_db_err(
                    &bot,
                    &q.id.0,
                    Err(e),
                    config,
                    telegram_id,
                    AdminErrorContext::Callback,
                    lang,
                )
                .await?;
                return Ok(());
            }

            let args = args!(user = username.to_string(), action = "toggled");
            answer_callback(
                &bot,
                &q.id,
                locales::get_text(lang.as_str(), "toast-user-muted", args.as_ref()),
                false,
            )
            .await?;

            let muted = mute_service::list_muted_users(db, telegram_id)
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to load muted users for {}: {}", telegram_id, e);
                    Vec::new()
                });
            let guest_username = state.config.teamtalk.guest_username.as_deref();
            render_mute_list_strings(RenderMuteListStringsArgs {
                bot: &bot,
                msg,
                lang,
                items: &muted,
                page,
                title_key: "list-mute-title",
                guest_username,
            })
            .await?;
        }
        MuteAction::ServerList { page } => {
            if let Err(e) = state.tx_tt.send(TtCommand::LoadAccounts) {
                tracing::error!("Failed to request TT accounts: {}", e);
                notify_admin_error(
                    &bot,
                    config,
                    telegram_id,
                    AdminErrorContext::TtCommand,
                    &e.to_string(),
                    lang,
                )
                .await;
            }
            let user_accounts = &state.user_accounts;
            let mut accounts: Vec<UserAccount> = user_accounts
                .read()
                .unwrap_or_else(|e| e.into_inner())
                .values()
                .cloned()
                .collect();
            accounts.sort_by(|a, b| a.username.to_lowercase().cmp(&b.username.to_lowercase()));

            let guest_username = state.config.teamtalk.guest_username.as_deref();
            render_mute_list(RenderMuteListArgs {
                bot: &bot,
                msg,
                db,
                telegram_id,
                lang,
                accounts: &accounts,
                page,
                title_key: "list-all-accs-title",
                guest_username,
            })
            .await?;
        }
        MuteAction::ServerToggle { username, page } => {
            if let Err(e) = mute_service::toggle_mute(db, telegram_id, username.as_str()).await {
                check_db_err(
                    &bot,
                    &q.id.0,
                    Err(e),
                    config,
                    telegram_id,
                    AdminErrorContext::Callback,
                    lang,
                )
                .await?;
                return Ok(());
            }

            let args = args!(user = username.to_string(), action = "toggled");
            answer_callback(
                &bot,
                &q.id,
                locales::get_text(lang.as_str(), "toast-user-muted", args.as_ref()),
                false,
            )
            .await?;

            let user_accounts = &state.user_accounts;
            let mut accounts: Vec<UserAccount> = user_accounts
                .read()
                .unwrap_or_else(|e| e.into_inner())
                .values()
                .cloned()
                .collect();
            accounts.sort_by(|a, b| a.username.to_lowercase().cmp(&b.username.to_lowercase()));
            let guest_username = state.config.teamtalk.guest_username.as_deref();

            render_mute_list(RenderMuteListArgs {
                bot: &bot,
                msg,
                db,
                telegram_id,
                lang,
                accounts: &accounts,
                page,
                title_key: "list-all-accs-title",
                guest_username,
            })
            .await?;
        }
    }

    Ok(())
}
