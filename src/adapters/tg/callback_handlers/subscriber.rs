use crate::adapters::tg::admin_logic::subscriber_settings::{
    send_sub_lang_menu, send_sub_link_account_list, send_sub_manage_tt_menu, send_sub_mute_list,
    send_sub_mute_mode_menu, send_sub_notif_menu,
};
use crate::adapters::tg::admin_logic::subscribers::{
    edit_subscribers_list, send_subscriber_details,
};
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{
    answer_callback, answer_callback_empty, check_db_err, notify_admin_error,
};
use crate::app::services::bans as bans_service;
use crate::app::services::settings as settings_service;
use crate::app::services::subscriber_actions as subscriber_actions_service;
use crate::args;
use crate::core::callbacks::SubAction;
use crate::core::types::{AdminErrorContext, LanguageCode, TtCommand};
use crate::infra::locales;
use teloxide::prelude::*;

pub async fn handle_subscriber_actions(
    bot: Bot,
    q: CallbackQuery,
    state: AppState,
    action: SubAction,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let msg = match q.message {
        Some(teloxide::types::MaybeInaccessibleMessage::Regular(m)) => m,
        _ => return Ok(()),
    };
    let db = &state.db;
    let user_accounts = &state.user_accounts;
    let tx_tt = &state.tx_tt;
    let config = &state.config;

    match action {
        SubAction::Details { sub_id, page } => {
            send_subscriber_details(&bot, &msg, db, lang, sub_id, page).await?;
            answer_callback_empty(&bot, &q.id).await?;
        }
        SubAction::Delete { sub_id, page } => {
            if check_db_err(
                &bot,
                &q.id.0,
                subscriber_actions_service::delete_user(db, sub_id).await,
                config,
                q.from.id.0 as i64,
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
                locales::get_text(lang.as_str(), "toast-subscriber-deleted", None),
                true,
            )
            .await?;
            edit_subscribers_list(&bot, &msg, db, lang, page).await?;
        }
        SubAction::Ban { sub_id, page } => {
            let tt_user_res = bans_service::get_tt_username_by_telegram_id(db, sub_id).await;
            let tt_user = match tt_user_res {
                Ok(u) => u,
                Err(e) => {
                    check_db_err(
                        &bot,
                        &q.id.0,
                        Err(e),
                        config,
                        q.from.id.0 as i64,
                        AdminErrorContext::Callback,
                        lang,
                    )
                    .await?;
                    return Ok(());
                }
            };

            if let Err(e) =
                bans_service::add_ban(db, Some(sub_id), tt_user, Some("Admin Ban".to_string()))
                    .await
            {
                check_db_err(
                    &bot,
                    &q.id.0,
                    Err(e),
                    config,
                    q.from.id.0 as i64,
                    AdminErrorContext::Callback,
                    lang,
                )
                .await?;
                return Ok(());
            }

            if let Err(e) = subscriber_actions_service::delete_user(db, sub_id).await {
                tracing::error!("Partial failure during ban: {}", e);
            }

            answer_callback(
                &bot,
                &q.id,
                locales::get_text(lang.as_str(), "toast-user-banned", None),
                true,
            )
            .await?;
            edit_subscribers_list(&bot, &msg, db, lang, page).await?;
        }
        SubAction::ManageTt { sub_id, page } => {
            send_sub_manage_tt_menu(&bot, &msg, db, lang, sub_id, page).await?;
        }
        SubAction::Unlink { sub_id, page } => {
            if check_db_err(
                &bot,
                &q.id.0,
                subscriber_actions_service::unlink_tt(db, sub_id).await,
                config,
                q.from.id.0 as i64,
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
                    "toast-account-unlinked",
                    args!(user = sub_id.to_string()).as_ref(),
                ),
                true,
            )
            .await?;
            send_sub_manage_tt_menu(&bot, &msg, db, lang, sub_id, page).await?;
        }
        SubAction::LinkList {
            sub_id,
            page,
            list_page,
        } => {
            if let Err(e) = tx_tt.send(TtCommand::LoadAccounts) {
                tracing::error!("Failed to request TT accounts: {}", e);
                notify_admin_error(
                    &bot,
                    config,
                    q.from.id.0 as i64,
                    AdminErrorContext::TtCommand,
                    &e.to_string(),
                    lang,
                )
                .await;
            }
            send_sub_link_account_list(&bot, &msg, user_accounts, lang, sub_id, page, list_page)
                .await?;
        }
        SubAction::LinkPerform {
            sub_id,
            page,
            username,
        } => {
            if check_db_err(
                &bot,
                &q.id.0,
                subscriber_actions_service::link_tt(db, sub_id, username.as_str()).await,
                config,
                q.from.id.0 as i64,
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
                    "toast-account-linked",
                    args!(user = username.to_string()).as_ref(),
                ),
                true,
            )
            .await?;
            send_sub_manage_tt_menu(&bot, &msg, db, lang, sub_id, page).await?;
        }
        SubAction::LangMenu { sub_id, page } => {
            send_sub_lang_menu(&bot, &msg, lang, sub_id, page).await?;
        }
        SubAction::LangSet {
            sub_id,
            page,
            lang: new_lang,
        } => {
            if check_db_err(
                &bot,
                &q.id.0,
                settings_service::update_language(db, sub_id, new_lang).await,
                config,
                q.from.id.0 as i64,
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
                    "toast-lang-set",
                    args!(id = sub_id.to_string(), lang = new_lang.as_str()).as_ref(),
                ),
                false,
            )
            .await?;
            send_subscriber_details(&bot, &msg, db, lang, sub_id, page).await?;
        }
        SubAction::NotifMenu { sub_id, page } => {
            send_sub_notif_menu(&bot, &msg, lang, sub_id, page).await?;
        }
        SubAction::NotifSet { sub_id, page, val } => {
            if check_db_err(
                &bot,
                &q.id.0,
                subscriber_actions_service::update_notifications(db, sub_id, val.clone()).await,
                config,
                q.from.id.0 as i64,
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
                    "toast-notif-set",
                    args!(id = sub_id.to_string(), val = val.to_string()).as_ref(),
                ),
                false,
            )
            .await?;
            send_subscriber_details(&bot, &msg, db, lang, sub_id, page).await?;
        }
        SubAction::NoonToggle { sub_id, page } => {
            if check_db_err(
                &bot,
                &q.id.0,
                settings_service::toggle_noon(db, sub_id).await.map(|_| ()),
                config,
                q.from.id.0 as i64,
                AdminErrorContext::Callback,
                lang,
            )
            .await?
            {
                return Ok(());
            }
            let status = "toggled";
            answer_callback(
                &bot,
                &q.id,
                locales::get_text(
                    lang.as_str(),
                    "toast-noon-toggled",
                    args!(id = sub_id.to_string(), status = status).as_ref(),
                ),
                false,
            )
            .await?;
            send_subscriber_details(&bot, &msg, db, lang, sub_id, page).await?;
        }
        SubAction::ModeMenu { sub_id, page } => {
            send_sub_mute_mode_menu(&bot, &msg, lang, sub_id, page).await?;
        }
        SubAction::ModeSet { sub_id, page, mode } => {
            if check_db_err(
                &bot,
                &q.id.0,
                subscriber_actions_service::update_mute_mode(db, sub_id, mode.clone()).await,
                config,
                q.from.id.0 as i64,
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
                    "toast-mute-mode-sub-set",
                    args!(id = sub_id.to_string(), val = mode.to_string()).as_ref(),
                ),
                false,
            )
            .await?;
            send_subscriber_details(&bot, &msg, db, lang, sub_id, page).await?;
        }
        SubAction::MuteView {
            sub_id,
            page,
            view_page,
        } => {
            send_sub_mute_list(&bot, &msg, db, lang, sub_id, page, view_page).await?;
        }
    }
    Ok(())
}
