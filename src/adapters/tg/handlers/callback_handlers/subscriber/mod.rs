use crate::adapters::tg::presenter::admin::subscribers::{
    SubscriberDetailsArgs, default_user_settings,
};
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{TgErrorReporter, telegram_id_from_callback_query};
use crate::app::services::tg_search_actions as tg_search_actions_service;
use crate::core::callbacks::SubAction;
use crate::core::types::AdminErrorContext;
use crate::core::types::{LanguageCode, TelegramId, TtCommand};
use crate::infra::db::Database;
use teloxide::prelude::*;

mod admin;
mod links;
mod list;
mod mute;
mod settings;

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
                crate::adapters::tg::handlers::search::SearchContext,
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
    state: &AppState,
    action: SubAction,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let Some(admin_chat_id) = telegram_id_from_callback_query(&q, "handle_subscriber_actions")
    else {
        return Ok(());
    };
    let Some(teloxide::types::MaybeInaccessibleMessage::Regular(msg)) = q.message else {
        return Ok(());
    };
    let db = &state.db;
    let tx_tt = &state.tx_tt;
    let config = &state.config;
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
    const fn errors(&self) -> TgErrorReporter<'_> {
        TgErrorReporter::new(self.bot, self.config, self.admin_chat_id, self.lang)
    }

    async fn dispatch(&self, action: SubAction) -> ResponseResult<()> {
        match action {
            SubAction::Details { sub_id, page } => list::details(self, sub_id, page).await,
            SubAction::AdminAddConfirm { sub_id, page } => {
                admin::admin_add_confirm(self, sub_id, page).await
            }
            SubAction::AdminAdd { sub_id, page } => admin::admin_add(self, sub_id, page).await,
            SubAction::AdminRemoveConfirm { sub_id, page } => {
                admin::admin_remove_confirm(self, sub_id, page).await
            }
            SubAction::AdminRemove { sub_id, page } => {
                admin::admin_remove(self, sub_id, page).await
            }
            SubAction::Delete { sub_id, page } => list::delete(self, sub_id, page).await,
            SubAction::Ban { sub_id, page } => list::ban(self, sub_id, page).await,
            SubAction::ManageTt { sub_id, page } => links::manage_tt(self, sub_id, page).await,
            SubAction::Unlink { sub_id, page } => links::unlink(self, sub_id, page).await,
            SubAction::LinkList {
                sub_id,
                page,
                list_page,
            } => links::link_list(self, sub_id, page, list_page).await,
            SubAction::LinkPerform {
                sub_id,
                page,
                username,
            } => links::link_perform(self, sub_id, page, username).await,
            SubAction::LangMenu { sub_id, page } => settings::lang_menu(self, sub_id, page).await,
            SubAction::LangSet { sub_id, page, lang } => {
                settings::lang_set(self, sub_id, page, lang).await
            }
            SubAction::NotifMenu { sub_id, page } => settings::notif_menu(self, sub_id, page).await,
            SubAction::NotifSet { sub_id, page, val } => {
                settings::notif_set(self, sub_id, page, val).await
            }
            SubAction::NoonToggle { sub_id, page } => {
                settings::noon_toggle(self, sub_id, page).await
            }
            SubAction::ModeMenu { sub_id, page } => settings::mode_menu(self, sub_id, page).await,
            SubAction::ModeSet { sub_id, page, mode } => {
                settings::mode_set(self, sub_id, page, mode).await
            }
            SubAction::MuteView {
                sub_id,
                page,
                view_page,
            } => mute::mute_view(self, sub_id, page, view_page).await,
        }
    }
    async fn sub_details_args(&self, sub_id: TelegramId, page: usize) -> SubscriberDetailsArgs<'_> {
        let is_main_admin = self.admin_chat_id == self.config.telegram.admin_chat_id;
        let mut is_admin = false;
        let mut settings = default_user_settings(sub_id);
        match tg_search_actions_service::load_subscriber_details(self.db, sub_id, LanguageCode::En)
            .await
        {
            Ok(details) => {
                settings = details.settings;
                is_admin = details.is_admin;
            }
            Err(e) => {
                let _ = self
                    .errors()
                    .check_db_err(&self.q_id.0, Err(e), AdminErrorContext::Callback)
                    .await;
            }
        }
        SubscriberDetailsArgs {
            bot: self.bot,
            msg: self.msg,
            lang: self.lang,
            sub_id,
            return_page: page,
            is_main_admin,
            admin_chat_id: self.config.telegram.admin_chat_id,
            settings,
            is_admin,
        }
    }
}
