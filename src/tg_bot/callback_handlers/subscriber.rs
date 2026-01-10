use crate::tg_bot::admin_logic::subscriber_settings::{
    send_sub_lang_menu, send_sub_link_account_list, send_sub_manage_tt_menu, send_sub_mute_list,
    send_sub_mute_mode_menu, send_sub_notif_menu,
};
use crate::tg_bot::admin_logic::subscribers::{edit_subscribers_list, send_subscriber_details};
use crate::tg_bot::callbacks_types::SubAction;
use crate::tg_bot::state::AppState;
use crate::types::{MuteListMode, NotificationSetting, TtCommand};
use crate::{args, locales};
use teloxide::prelude::*;

pub async fn handle_subscriber_actions(
    bot: Bot,
    q: CallbackQuery,
    state: AppState,
    action: SubAction,
    lang: &str,
) -> ResponseResult<()> {
    let msg = match q.message {
        Some(teloxide::types::MaybeInaccessibleMessage::Regular(m)) => m,
        _ => return Ok(()),
    };
    let db = &state.db;
    let user_accounts = &state.user_accounts;
    let tx_tt = &state.tx_tt;

    match action {
        SubAction::Details { sub_id, page } => {
            send_subscriber_details(&bot, &msg, db, lang, sub_id, page).await?;
            bot.answer_callback_query(q.id).await?;
        }
        SubAction::Delete { sub_id, page } => {
            db.delete_user_profile(sub_id).await.ok();
            bot.answer_callback_query(q.id)
                .text(locales::get_text(lang, "toast-subscriber-deleted", None))
                .show_alert(true)
                .await?;
            edit_subscribers_list(&bot, &msg, db, lang, page).await?;
        }
        SubAction::Ban { sub_id, page } => {
            let tt_user = sqlx::query_scalar::<_, String>(
                "SELECT teamtalk_username FROM user_settings WHERE telegram_id = ?",
            )
            .bind(sub_id)
            .fetch_optional(&db.pool)
            .await
            .unwrap_or(None);

            db.add_ban(Some(sub_id), tt_user, Some("Admin Ban".to_string()))
                .await
                .ok();
            db.delete_user_profile(sub_id).await.ok();

            bot.answer_callback_query(q.id)
                .text(locales::get_text(lang, "toast-user-banned", None))
                .show_alert(true)
                .await?;
            edit_subscribers_list(&bot, &msg, db, lang, page).await?;
        }
        SubAction::ManageTt { sub_id, page } => {
            send_sub_manage_tt_menu(&bot, &msg, db, lang, sub_id, page).await?;
        }
        SubAction::Unlink { sub_id, page } => {
            db.unlink_tt_account(sub_id).await.ok();
            bot.answer_callback_query(q.id)
                .text(locales::get_text(
                    lang,
                    "toast-account-unlinked",
                    args!(user = sub_id.to_string()).as_ref(),
                ))
                .show_alert(true)
                .await?;
            send_sub_manage_tt_menu(&bot, &msg, db, lang, sub_id, page).await?;
        }
        SubAction::LinkList {
            sub_id,
            page,
            list_page,
        } => {
            tx_tt.send(TtCommand::LoadAccounts).ok();
            send_sub_link_account_list(&bot, &msg, user_accounts, lang, sub_id, page, list_page)
                .await?;
        }
        SubAction::LinkPerform {
            sub_id,
            page,
            username,
        } => {
            db.link_tt_account(sub_id, &username).await.ok();
            bot.answer_callback_query(q.id)
                .text(locales::get_text(
                    lang,
                    "toast-account-linked",
                    args!(user = username).as_ref(),
                ))
                .show_alert(true)
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
            db.update_language(sub_id, &new_lang).await.ok();
            bot.answer_callback_query(q.id)
                .text(locales::get_text(
                    lang,
                    "toast-lang-set",
                    args!(id = sub_id.to_string(), lang = new_lang).as_ref(),
                ))
                .await?;
            send_subscriber_details(&bot, &msg, db, lang, sub_id, page).await?;
        }
        SubAction::NotifMenu { sub_id, page } => {
            send_sub_notif_menu(&bot, &msg, lang, sub_id, page).await?;
        }
        SubAction::NotifSet { sub_id, page, val } => {
            db.update_notification_setting(sub_id, NotificationSetting::from(val.as_str()))
                .await
                .ok();
            bot.answer_callback_query(q.id)
                .text(locales::get_text(
                    lang,
                    "toast-notif-set",
                    args!(id = sub_id.to_string(), val = val).as_ref(),
                ))
                .await?;
            send_subscriber_details(&bot, &msg, db, lang, sub_id, page).await?;
        }
        SubAction::NoonToggle { sub_id, page } => {
            db.toggle_noon(sub_id).await.ok();
            let status = "toggled";
            bot.answer_callback_query(q.id)
                .text(locales::get_text(
                    lang,
                    "toast-noon-toggled",
                    args!(id = sub_id.to_string(), status = status).as_ref(),
                ))
                .await?;
            send_subscriber_details(&bot, &msg, db, lang, sub_id, page).await?;
        }
        SubAction::ModeMenu { sub_id, page } => {
            send_sub_mute_mode_menu(&bot, &msg, lang, sub_id, page).await?;
        }
        SubAction::ModeSet { sub_id, page, mode } => {
            db.update_mute_mode(sub_id, MuteListMode::from(mode.as_str()))
                .await
                .ok();
            bot.answer_callback_query(q.id)
                .text(locales::get_text(
                    lang,
                    "toast-mute-mode-sub-set",
                    args!(id = sub_id.to_string(), val = mode).as_ref(),
                ))
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
