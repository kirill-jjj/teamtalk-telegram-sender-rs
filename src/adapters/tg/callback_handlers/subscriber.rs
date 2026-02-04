use crate::adapters::tg::admin_logic::subscriber_settings::{
    send_sub_lang_menu, send_sub_link_account_list, send_sub_manage_tt_menu, send_sub_mute_list,
    send_sub_mute_mode_menu, send_sub_notif_menu,
};
use crate::adapters::tg::admin_logic::subscribers::{
    SubscriberDetailsArgs, edit_subscribers_list, send_subscriber_details,
};
use crate::adapters::tg::keyboards::confirm_cancel_keyboard;
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{
    answer_callback, answer_callback_empty, check_db_err, notify_admin_error,
};
use crate::app::services::subscriber_actions as subscriber_actions_service;
use crate::args;
use crate::core::callbacks::{CallbackAction, SubAction};
use crate::core::types::TelegramId;
use crate::core::types::{AdminErrorContext, LanguageCode, TtCommand};
use crate::infra::db::Database;
use crate::infra::locales;
use teloxide::prelude::*;

struct SubCtx<'a> {
    bot: &'a Bot,
    msg: &'a Message,
    db: &'a Database,
    config: &'a crate::bootstrap::config::Config,
    state_handle: &'a crate::app::state::StateHandle,
    tx_tt: &'a tokio::sync::mpsc::Sender<TtCommand>,
    search_contexts: &'a std::sync::Arc<
        tokio::sync::Mutex<
            std::collections::HashMap<
                teloxide::types::ChatId,
                crate::adapters::tg::search::SearchContext,
            >,
        >,
    >,
    lang: LanguageCode,
    q_id: &'a teloxide::types::CallbackQueryId,
    admin_chat_id: TelegramId,
}

pub async fn handle_subscriber_actions(
    bot: Bot,
    q: CallbackQuery,
    state: AppState,
    action: SubAction,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let Some(teloxide::types::MaybeInaccessibleMessage::Regular(msg)) = q.message else {
        return Ok(());
    };
    let db = &state.db;
    let tx_tt = &state.tx_tt;
    let config = &state.config;
    let admin_chat_id = tg_user_id_i64(q.from.id.0);

    let ctx = SubCtx {
        bot: &bot,
        msg: &msg,
        db,
        config,
        state_handle: &state.state,
        tx_tt,
        search_contexts: &state.search_contexts,
        lang,
        q_id: &q.id,
        admin_chat_id,
    };
    ctx.dispatch(action).await?;
    Ok(())
}

impl SubCtx<'_> {
    async fn dispatch(&self, action: SubAction) -> ResponseResult<()> {
        match action {
            SubAction::Details { sub_id, page } => self.details(sub_id, page).await,
            SubAction::AdminAddConfirm { sub_id, page } => {
                self.admin_add_confirm(sub_id, page).await
            }
            SubAction::AdminAdd { sub_id, page } => self.admin_add(sub_id, page).await,
            SubAction::AdminRemoveConfirm { sub_id, page } => {
                self.admin_remove_confirm(sub_id, page).await
            }
            SubAction::AdminRemove { sub_id, page } => self.admin_remove(sub_id, page).await,
            SubAction::Delete { sub_id, page } => self.delete(sub_id, page).await,
            SubAction::Ban { sub_id, page } => self.ban(sub_id, page).await,
            SubAction::ManageTt { sub_id, page } => self.manage_tt(sub_id, page).await,
            SubAction::Unlink { sub_id, page } => self.unlink(sub_id, page).await,
            SubAction::LinkList {
                sub_id,
                page,
                list_page,
            } => self.link_list(sub_id, page, list_page).await,
            SubAction::LinkPerform {
                sub_id,
                page,
                username,
            } => self.link_perform(sub_id, page, username).await,
            SubAction::LangMenu { sub_id, page } => self.lang_menu(sub_id, page).await,
            SubAction::LangSet { sub_id, page, lang } => self.lang_set(sub_id, page, lang).await,
            SubAction::NotifMenu { sub_id, page } => self.notif_menu(sub_id, page).await,
            SubAction::NotifSet { sub_id, page, val } => self.notif_set(sub_id, page, val).await,
            SubAction::NoonToggle { sub_id, page } => self.noon_toggle(sub_id, page).await,
            SubAction::ModeMenu { sub_id, page } => self.mode_menu(sub_id, page).await,
            SubAction::ModeSet { sub_id, page, mode } => self.mode_set(sub_id, page, mode).await,
            SubAction::MuteView {
                sub_id,
                page,
                view_page,
            } => self.mute_view(sub_id, page, view_page).await,
        }
    }

    async fn details(&self, sub_id: TelegramId, page: usize) -> ResponseResult<()> {
        send_subscriber_details(self.sub_details_args(sub_id, page)).await?;
        answer_callback_empty(self.bot, self.q_id).await?;
        Ok(())
    }

    async fn admin_add_confirm(&self, sub_id: TelegramId, page: usize) -> ResponseResult<()> {
        if !self.is_main_admin() {
            answer_callback_empty(self.bot, self.q_id).await?;
            return Ok(());
        }
        let text = locales::get_text(
            self.lang.as_str(),
            locales::LocaleKey::ConfirmAdminAdd,
            None,
        );
        let keyboard = confirm_cancel_keyboard(
            self.lang,
            locales::LocaleKey::BtnYes,
            CallbackAction::Subscriber(SubAction::AdminAdd { sub_id, page }),
            locales::LocaleKey::BtnNo,
            CallbackAction::Subscriber(SubAction::Details { sub_id, page }),
        );
        self.bot
            .edit_message_text(self.msg.chat.id, self.msg.id, text)
            .reply_markup(keyboard)
            .await?;
        answer_callback_empty(self.bot, self.q_id).await?;
        Ok(())
    }

    async fn admin_remove_confirm(&self, sub_id: TelegramId, page: usize) -> ResponseResult<()> {
        if !self.is_main_admin() {
            answer_callback_empty(self.bot, self.q_id).await?;
            return Ok(());
        }
        let text = locales::get_text(
            self.lang.as_str(),
            locales::LocaleKey::ConfirmAdminRemove,
            None,
        );
        let keyboard = confirm_cancel_keyboard(
            self.lang,
            locales::LocaleKey::BtnYes,
            CallbackAction::Subscriber(SubAction::AdminRemove { sub_id, page }),
            locales::LocaleKey::BtnNo,
            CallbackAction::Subscriber(SubAction::Details { sub_id, page }),
        );
        self.bot
            .edit_message_text(self.msg.chat.id, self.msg.id, text)
            .reply_markup(keyboard)
            .await?;
        answer_callback_empty(self.bot, self.q_id).await?;
        Ok(())
    }

    async fn admin_add(&self, sub_id: TelegramId, page: usize) -> ResponseResult<()> {
        if !self.is_main_admin() {
            answer_callback_empty(self.bot, self.q_id).await?;
            return Ok(());
        }
        if check_db_err(
            self.bot,
            &self.q_id.0,
            self.db.add_admin(sub_id).await.map(|_| ()),
            self.config,
            self.admin_chat_id,
            AdminErrorContext::Callback,
            self.lang,
        )
        .await?
        {
            return Ok(());
        }
        answer_callback(
            self.bot,
            self.q_id,
            locales::get_text(
                self.lang.as_str(),
                locales::LocaleKey::ToastAdminAdded,
                None,
            ),
            true,
        )
        .await?;
        send_subscriber_details(self.sub_details_args(sub_id, page)).await?;
        Ok(())
    }

    async fn admin_remove(&self, sub_id: TelegramId, page: usize) -> ResponseResult<()> {
        if !self.is_main_admin() {
            answer_callback_empty(self.bot, self.q_id).await?;
            return Ok(());
        }
        if check_db_err(
            self.bot,
            &self.q_id.0,
            self.db.remove_admin(sub_id).await.map(|_| ()),
            self.config,
            self.admin_chat_id,
            AdminErrorContext::Callback,
            self.lang,
        )
        .await?
        {
            return Ok(());
        }
        answer_callback(
            self.bot,
            self.q_id,
            locales::get_text(
                self.lang.as_str(),
                locales::LocaleKey::ToastAdminRemoved,
                None,
            ),
            true,
        )
        .await?;
        send_subscriber_details(self.sub_details_args(sub_id, page)).await?;
        Ok(())
    }

    async fn delete(&self, sub_id: TelegramId, page: usize) -> ResponseResult<()> {
        if check_db_err(
            self.bot,
            &self.q_id.0,
            subscriber_actions_service::delete_user(self.db, sub_id).await,
            self.config,
            self.admin_chat_id,
            AdminErrorContext::Callback,
            self.lang,
        )
        .await?
        {
            return Ok(());
        }
        answer_callback(
            self.bot,
            self.q_id,
            locales::get_text(
                self.lang.as_str(),
                locales::LocaleKey::ToastSubscriberDeleted,
                None,
            ),
            true,
        )
        .await?;
        edit_subscribers_list(
            self.bot,
            self.msg,
            self.db,
            self.search_contexts,
            self.lang,
            page,
        )
        .await?;
        Ok(())
    }

    async fn ban(&self, sub_id: TelegramId, page: usize) -> ResponseResult<()> {
        let tt_user = match self.db.get_tt_username_by_telegram_id(sub_id).await {
            Ok(u) => u,
            Err(e) => {
                check_db_err(
                    self.bot,
                    &self.q_id.0,
                    Err(e),
                    self.config,
                    self.admin_chat_id,
                    AdminErrorContext::Callback,
                    self.lang,
                )
                .await?;
                return Ok(());
            }
        };

        if let Err(e) = self
            .db
            .add_ban(Some(sub_id), tt_user.clone(), Some("Admin Ban".to_string()))
            .await
        {
            check_db_err(
                self.bot,
                &self.q_id.0,
                Err(e),
                self.config,
                self.admin_chat_id,
                AdminErrorContext::Callback,
                self.lang,
            )
            .await?;
            return Ok(());
        }

        if let Err(e) = subscriber_actions_service::delete_user(self.db, sub_id).await {
            tracing::error!(
                telegram_id = sub_id.as_i64(),
                tt_username = ?tt_user,
                error = %e,
                "Partial failure during ban"
            );
        }

        answer_callback(
            self.bot,
            self.q_id,
            locales::get_text(
                self.lang.as_str(),
                locales::LocaleKey::ToastUserBanned,
                None,
            ),
            true,
        )
        .await?;
        edit_subscribers_list(
            self.bot,
            self.msg,
            self.db,
            self.search_contexts,
            self.lang,
            page,
        )
        .await?;
        Ok(())
    }

    async fn manage_tt(&self, sub_id: TelegramId, page: usize) -> ResponseResult<()> {
        send_sub_manage_tt_menu(self.bot, self.msg, self.db, self.lang, sub_id, page).await
    }

    async fn unlink(&self, sub_id: TelegramId, page: usize) -> ResponseResult<()> {
        if check_db_err(
            self.bot,
            &self.q_id.0,
            subscriber_actions_service::unlink_tt(self.db, sub_id).await,
            self.config,
            self.admin_chat_id,
            AdminErrorContext::Callback,
            self.lang,
        )
        .await?
        {
            return Ok(());
        }
        answer_callback(
            self.bot,
            self.q_id,
            locales::get_text(
                self.lang.as_str(),
                locales::LocaleKey::ToastAccountUnlinked,
                args!(user = sub_id.to_string()).as_ref(),
            ),
            true,
        )
        .await?;
        send_sub_manage_tt_menu(self.bot, self.msg, self.db, self.lang, sub_id, page).await?;
        Ok(())
    }

    async fn link_list(
        &self,
        sub_id: TelegramId,
        page: usize,
        list_page: usize,
    ) -> ResponseResult<()> {
        if let Err(e) = self.tx_tt.send(TtCommand::LoadAccounts).await {
            tracing::error!(error = %e, "Failed to request TT accounts");
            notify_admin_error(
                self.bot,
                self.config,
                self.admin_chat_id,
                AdminErrorContext::TtCommand,
                &e.to_string(),
                self.lang,
            )
            .await;
        }
        let accounts = self
            .state_handle
            .user_accounts_sorted()
            .await
            .unwrap_or_default();
        send_sub_link_account_list(
            self.bot,
            self.msg,
            accounts,
            self.search_contexts,
            self.lang,
            sub_id,
            page,
            list_page,
        )
        .await?;
        answer_callback_empty(self.bot, self.q_id).await?;
        Ok(())
    }

    async fn link_perform(
        &self,
        sub_id: TelegramId,
        page: usize,
        username: crate::core::types::TtUsername,
    ) -> ResponseResult<()> {
        if check_db_err(
            self.bot,
            &self.q_id.0,
            subscriber_actions_service::link_tt(self.db, sub_id, &username).await,
            self.config,
            self.admin_chat_id,
            AdminErrorContext::Callback,
            self.lang,
        )
        .await?
        {
            return Ok(());
        }
        answer_callback(
            self.bot,
            self.q_id,
            locales::get_text(
                self.lang.as_str(),
                locales::LocaleKey::ToastAccountLinked,
                args!(user = username.to_string()).as_ref(),
            ),
            true,
        )
        .await?;
        send_sub_manage_tt_menu(self.bot, self.msg, self.db, self.lang, sub_id, page).await?;
        Ok(())
    }

    async fn lang_menu(&self, sub_id: TelegramId, page: usize) -> ResponseResult<()> {
        send_sub_lang_menu(self.bot, self.msg, self.lang, sub_id, page).await
    }

    async fn lang_set(
        &self,
        sub_id: TelegramId,
        page: usize,
        new_lang: LanguageCode,
    ) -> ResponseResult<()> {
        if check_db_err(
            self.bot,
            &self.q_id.0,
            self.db.update_language(sub_id, new_lang).await,
            self.config,
            self.admin_chat_id,
            AdminErrorContext::Callback,
            self.lang,
        )
        .await?
        {
            return Ok(());
        }
        answer_callback(
            self.bot,
            self.q_id,
            locales::get_text(
                self.lang.as_str(),
                locales::LocaleKey::ToastLangSet,
                args!(id = sub_id.to_string(), lang = new_lang.as_str()).as_ref(),
            ),
            false,
        )
        .await?;
        send_subscriber_details(self.sub_details_args(sub_id, page)).await?;
        Ok(())
    }

    async fn notif_menu(&self, sub_id: TelegramId, page: usize) -> ResponseResult<()> {
        send_sub_notif_menu(self.bot, self.msg, self.lang, sub_id, page).await
    }

    async fn notif_set(
        &self,
        sub_id: TelegramId,
        page: usize,
        val: crate::core::types::NotificationSetting,
    ) -> ResponseResult<()> {
        if check_db_err(
            self.bot,
            &self.q_id.0,
            subscriber_actions_service::update_notifications(self.db, sub_id, val.clone()).await,
            self.config,
            self.admin_chat_id,
            AdminErrorContext::Callback,
            self.lang,
        )
        .await?
        {
            return Ok(());
        }
        answer_callback(
            self.bot,
            self.q_id,
            locales::get_text(
                self.lang.as_str(),
                locales::LocaleKey::ToastNotifSet,
                args!(id = sub_id.to_string(), val = val.to_string()).as_ref(),
            ),
            false,
        )
        .await?;
        send_subscriber_details(self.sub_details_args(sub_id, page)).await?;
        Ok(())
    }

    async fn noon_toggle(&self, sub_id: TelegramId, page: usize) -> ResponseResult<()> {
        if check_db_err(
            self.bot,
            &self.q_id.0,
            {
                let res: anyhow::Result<()> = self.db.toggle_noon(sub_id).await.map(|_| ());
                res
            },
            self.config,
            self.admin_chat_id,
            AdminErrorContext::Callback,
            self.lang,
        )
        .await?
        {
            return Ok(());
        }
        answer_callback(
            self.bot,
            self.q_id,
            locales::get_text(
                self.lang.as_str(),
                locales::LocaleKey::ToastNoonToggled,
                args!(id = sub_id.to_string(), status = "toggled").as_ref(),
            ),
            false,
        )
        .await?;
        send_subscriber_details(self.sub_details_args(sub_id, page)).await?;
        Ok(())
    }

    async fn mode_menu(&self, sub_id: TelegramId, page: usize) -> ResponseResult<()> {
        send_sub_mute_mode_menu(self.bot, self.msg, self.lang, sub_id, page).await
    }

    async fn mode_set(
        &self,
        sub_id: TelegramId,
        page: usize,
        mode: crate::core::types::MuteListMode,
    ) -> ResponseResult<()> {
        if check_db_err(
            self.bot,
            &self.q_id.0,
            subscriber_actions_service::update_mute_mode(self.db, sub_id, mode.clone()).await,
            self.config,
            self.admin_chat_id,
            AdminErrorContext::Callback,
            self.lang,
        )
        .await?
        {
            return Ok(());
        }
        answer_callback(
            self.bot,
            self.q_id,
            locales::get_text(
                self.lang.as_str(),
                locales::LocaleKey::ToastMuteModeSubSet,
                args!(id = sub_id.to_string(), val = mode.to_string()).as_ref(),
            ),
            false,
        )
        .await?;
        send_subscriber_details(self.sub_details_args(sub_id, page)).await?;
        Ok(())
    }

    async fn mute_view(
        &self,
        sub_id: TelegramId,
        page: usize,
        view_page: usize,
    ) -> ResponseResult<()> {
        send_sub_mute_list(
            self.bot,
            self.msg,
            self.db,
            self.search_contexts,
            self.lang,
            sub_id,
            page,
            view_page,
        )
        .await
    }

    #[allow(clippy::missing_const_for_fn)]
    fn is_main_admin(&self) -> bool {
        self.admin_chat_id == self.config.telegram.admin_chat_id
    }

    fn sub_details_args(&self, sub_id: TelegramId, page: usize) -> SubscriberDetailsArgs<'_> {
        SubscriberDetailsArgs {
            bot: self.bot,
            msg: self.msg,
            db: self.db,
            lang: self.lang,
            sub_id,
            return_page: page,
            is_main_admin: self.is_main_admin(),
            admin_chat_id: self.config.telegram.admin_chat_id,
        }
    }
}

fn tg_user_id_i64(user_id: u64) -> TelegramId {
    TelegramId::from(i64::try_from(user_id).unwrap_or(i64::MAX))
}
