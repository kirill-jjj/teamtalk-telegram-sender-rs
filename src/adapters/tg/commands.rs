use crate::adapters::tg::admin_logic::bans::send_unban_list;
use crate::adapters::tg::admin_logic::subscribers::send_subscribers_list;
use crate::adapters::tg::keyboards::{
    confirm_cancel_keyboard, create_main_menu_keyboard, create_user_list_keyboard,
};
use crate::adapters::tg::search::{
    SearchContext, SearchListType, append_search_hint, maybe_handle_search_message,
    set_search_context,
};
use crate::adapters::tg::settings_logic::send_main_settings;
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{ensure_subscribed, notify_admin_error, send_text_key};
use crate::app::services::deeplink as deeplink_service;
use crate::app::services::pending as pending_service;
use crate::app::services::reply_queue as reply_queue_service;
use crate::app::services::subscription as subscription_service;
use crate::app::services::user_settings as user_settings_service;
use crate::args;
use crate::bootstrap::config::Config;
use crate::core::callbacks::{AdminAction, CallbackAction, UnsubAction};
use crate::core::types::{
    AdminErrorContext, DeeplinkAction, LanguageCode, LiteUser, TelegramId, TtCommand, TtUsername,
};
use crate::infra::db::Database;
use crate::infra::locales;
use std::time::{SystemTime, UNIX_EPOCH};
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::types::{ParseMode, Voice};
use teloxide::utils::command::BotCommands;
use tokio::fs::File;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "Available Commands:")]
pub enum Command {
    #[command(description = "Start")]
    Start(String),
    #[command(description = "Main Menu")]
    Menu,
    #[command(description = "Help")]
    Help,
    #[command(description = "Who is online")]
    Who,
    #[command(description = "Settings")]
    Settings,
    #[command(description = "Unsubscribe")]
    Unsub,
    #[command(description = "Kick (Admin)")]
    Kick,
    #[command(description = "Ban (Admin)")]
    Ban,
    #[command(description = "Unban (Admin)")]
    Unban,
    #[command(description = "Subscribers (Admin)")]
    Subscribers,
    #[command(description = "Exit (Admin)")]
    Exit,
    #[command(description = "Broadcast (Admin)")]
    Broadcast(String),
    #[command(description = "Message (Admin)")]
    Message(String),
    #[command(description = "Reply Queue")]
    Queue(String),
}

pub async fn answer_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: AppState,
) -> ResponseResult<()> {
    let Some(user) = &msg.from else {
        return Ok(());
    };
    let telegram_id = tg_user_id_i64(user.id.0);
    let Some(ctx) = CommandCtx::new(&bot, &msg, &state, telegram_id).await? else {
        return Ok(());
    };
    ctx.dispatch(cmd).await?;
    Ok(())
}

fn tg_user_id_i64(user_id: u64) -> TelegramId {
    TelegramId::from(i64::try_from(user_id).unwrap_or(i64::MAX))
}

struct CommandCtx<'a> {
    bot: &'a Bot,
    msg: &'a Message,
    state: &'a AppState,
    db: &'a crate::infra::db::Database,
    config: &'a crate::bootstrap::config::Config,
    tx_tt: &'a tokio::sync::mpsc::Sender<TtCommand>,
    telegram_id: TelegramId,
    lang: LanguageCode,
    is_admin: bool,
}

impl<'a> CommandCtx<'a> {
    async fn new(
        bot: &'a Bot,
        msg: &'a Message,
        state: &'a AppState,
        telegram_id: TelegramId,
    ) -> ResponseResult<Option<Self>> {
        let db = &state.db;
        let config = &state.config;
        let default_lang = config.general.default_lang;
        let settings = match user_settings_service::get_or_create(db, telegram_id, default_lang)
            .await
        {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(telegram_id = telegram_id.as_i64(), error = %e, "Failed to get or create user");
                notify_admin_error(
                    bot,
                    config,
                    telegram_id,
                    AdminErrorContext::Command,
                    &e.to_string(),
                    default_lang,
                )
                .await;
                send_text_key(
                    bot,
                    msg.chat.id,
                    default_lang,
                    locales::LocaleKey::CmdError,
                    Some(msg.id),
                )
                .await?;
                return Ok(None);
            }
        };
        let lang = settings.language_code;
        let is_admin = if telegram_id == config.telegram.admin_chat_id {
            true
        } else {
            match db.get_all_admins().await {
                Ok(admins) => admins.contains(&telegram_id),
                Err(e) => {
                    tracing::error!(error = %e, "Failed to load admin list");
                    false
                }
            }
        };

        Ok(Some(Self {
            bot,
            msg,
            state,
            db,
            config,
            tx_tt: &state.tx_tt,
            telegram_id,
            lang,
            is_admin,
        }))
    }

    async fn dispatch(&self, cmd: Command) -> ResponseResult<()> {
        match cmd {
            Command::Start(token) => self.start(token).await,
            Command::Menu => self.menu().await,
            Command::Help => self.help().await,
            Command::Who => self.who().await,
            Command::Settings => self.settings().await,
            Command::Unsub => self.unsub().await,
            Command::Kick | Command::Ban => self.kick_or_ban(cmd).await,
            Command::Unban => self.unban().await,
            Command::Subscribers => self.subscribers().await,
            Command::Exit => self.exit().await,
            Command::Broadcast(text) => self.broadcast(text).await,
            Command::Message(text) => self.message(text).await,
            Command::Queue(text) => self.queue(text).await,
        }
    }

    async fn start(&self, token: String) -> ResponseResult<()> {
        if token.is_empty() {
            return send_text_key(
                self.bot,
                self.msg.chat.id,
                self.lang,
                locales::LocaleKey::HelloStart,
                Some(self.msg.id),
            )
            .await;
        }
        match deeplink_service::resolve_for_user(self.db, &token, self.telegram_id).await {
            Ok(Some(deeplink)) => match deeplink.action {
                DeeplinkAction::Subscribe => self.handle_subscribe(deeplink.payload).await,
                DeeplinkAction::Unsubscribe => self.handle_unsubscribe().await,
            },
            Ok(None) => {
                send_text_key(
                    self.bot,
                    self.msg.chat.id,
                    self.lang,
                    locales::LocaleKey::CmdInvalidDeeplink,
                    Some(self.msg.id),
                )
                .await
            }
            Err(e) => {
                tracing::error!(error = %e, "DB error resolving deeplink");
                notify_admin_error(
                    self.bot,
                    self.config,
                    self.telegram_id,
                    AdminErrorContext::Command,
                    &e.to_string(),
                    self.lang,
                )
                .await;
                send_text_key(
                    self.bot,
                    self.msg.chat.id,
                    self.lang,
                    locales::LocaleKey::CmdError,
                    Some(self.msg.id),
                )
                .await
            }
        }
    }

    async fn handle_subscribe(&self, payload: Option<String>) -> ResponseResult<()> {
        match subscription_service::subscribe_via_deeplink(self.db, self.telegram_id, payload).await
        {
            Ok(subscription_service::SubscribeOutcome::BannedUser) => {
                send_text_key(
                    self.bot,
                    self.msg.chat.id,
                    self.lang,
                    locales::LocaleKey::CmdUserBanned,
                    Some(self.msg.id),
                )
                .await
            }
            Ok(subscription_service::SubscribeOutcome::BannedTeamTalk { username }) => {
                let args = args!(name = username.to_string());
                self.bot
                    .send_message(
                        self.msg.chat.id,
                        locales::get_text(
                            self.lang.as_str(),
                            locales::LocaleKey::CmdTtBanned,
                            args.as_ref(),
                        ),
                    )
                    .reply_to(self.msg.id)
                    .await?;
                Ok(())
            }
            Ok(subscription_service::SubscribeOutcome::SubscribedLinked) => {
                send_text_key(
                    self.bot,
                    self.msg.chat.id,
                    self.lang,
                    locales::LocaleKey::CmdSuccessSub,
                    Some(self.msg.id),
                )
                .await
            }
            Ok(subscription_service::SubscribeOutcome::SubscribedGuest) => {
                self.bot
                    .send_message(
                        self.msg.chat.id,
                        locales::get_text(
                            self.lang.as_str(),
                            locales::LocaleKey::CmdSuccessSubGuest,
                            None,
                        ),
                    )
                    .parse_mode(ParseMode::Html)
                    .reply_to(self.msg.id)
                    .await?;
                Ok(())
            }
            Err(e) => {
                tracing::error!(error = %e, "DB error adding subscriber");
                notify_admin_error(
                    self.bot,
                    self.config,
                    self.telegram_id,
                    AdminErrorContext::Command,
                    &e.to_string(),
                    self.lang,
                )
                .await;
                send_text_key(
                    self.bot,
                    self.msg.chat.id,
                    self.lang,
                    locales::LocaleKey::CmdError,
                    Some(self.msg.id),
                )
                .await
            }
        }
    }

    async fn handle_unsubscribe(&self) -> ResponseResult<()> {
        if let Err(e) = subscription_service::unsubscribe(self.db, self.telegram_id).await {
            tracing::error!(error = %e, "DB error unsubscribing");
            notify_admin_error(
                self.bot,
                self.config,
                self.telegram_id,
                AdminErrorContext::Command,
                &e.to_string(),
                self.lang,
            )
            .await;
            return send_text_key(
                self.bot,
                self.msg.chat.id,
                self.lang,
                locales::LocaleKey::CmdError,
                Some(self.msg.id),
            )
            .await;
        }
        send_text_key(
            self.bot,
            self.msg.chat.id,
            self.lang,
            locales::LocaleKey::CmdSuccessUnsub,
            Some(self.msg.id),
        )
        .await
    }

    async fn menu(&self) -> ResponseResult<()> {
        if !ensure_subscribed(self.bot, self.msg, self.db, self.config, self.lang).await {
            return Ok(());
        }
        let keyboard = create_main_menu_keyboard(self.lang, self.is_admin);
        self.bot
            .send_message(
                self.msg.chat.id,
                locales::get_text(self.lang.as_str(), locales::LocaleKey::MenuTitle, None),
            )
            .parse_mode(ParseMode::Html)
            .reply_to(self.msg.id)
            .reply_markup(keyboard)
            .await?;
        Ok(())
    }

    async fn help(&self) -> ResponseResult<()> {
        if !ensure_subscribed(self.bot, self.msg, self.db, self.config, self.lang).await {
            return Ok(());
        }
        self.bot
            .send_message(
                self.msg.chat.id,
                locales::get_text(self.lang.as_str(), locales::LocaleKey::HelpText, None),
            )
            .parse_mode(ParseMode::Html)
            .reply_to(self.msg.id)
            .await?;
        Ok(())
    }

    async fn who(&self) -> ResponseResult<()> {
        if !ensure_subscribed(self.bot, self.msg, self.db, self.config, self.lang).await {
            return Ok(());
        }
        if let Err(e) = self
            .tx_tt
            .send(TtCommand::Who {
                chat_id: crate::core::types::TgChatId::from(self.msg.chat.id.0),
                lang: self.lang,
                reply_to: Some(crate::core::types::TgMessageId::from(self.msg.id.0)),
            })
            .await
        {
            tracing::error!(error = %e, "Failed to send TT who command");
            notify_admin_error(
                self.bot,
                self.config,
                self.telegram_id,
                AdminErrorContext::TtCommand,
                &e.to_string(),
                self.lang,
            )
            .await;
        }
        Ok(())
    }

    async fn settings(&self) -> ResponseResult<()> {
        if !ensure_subscribed(self.bot, self.msg, self.db, self.config, self.lang).await {
            return Ok(());
        }
        send_main_settings(self.bot, self.msg.chat.id, self.lang, Some(self.msg.id)).await
    }

    async fn unsub(&self) -> ResponseResult<()> {
        if !ensure_subscribed(self.bot, self.msg, self.db, self.config, self.lang).await {
            return Ok(());
        }
        let text = locales::get_text(
            self.lang.as_str(),
            locales::LocaleKey::UnsubConfirmText,
            None,
        );
        let keyboard = confirm_cancel_keyboard(
            self.lang,
            locales::LocaleKey::BtnYes,
            CallbackAction::Unsub(UnsubAction::Confirm),
            locales::LocaleKey::BtnNo,
            CallbackAction::Unsub(UnsubAction::Cancel),
        );
        self.bot
            .send_message(self.msg.chat.id, text)
            .reply_to(self.msg.id)
            .reply_markup(keyboard)
            .await?;
        Ok(())
    }

    async fn kick_or_ban(&self, cmd: Command) -> ResponseResult<()> {
        if !self.is_admin {
            send_text_key(
                self.bot,
                self.msg.chat.id,
                self.lang,
                locales::LocaleKey::CmdUnauth,
                Some(self.msg.id),
            )
            .await?;
            return Ok(());
        }
        let users: Vec<LiteUser> = self
            .state
            .state
            .online_users_sorted()
            .await
            .unwrap_or_default();

        let is_kick = matches!(cmd, Command::Kick);
        let title_key = if is_kick {
            locales::LocaleKey::ListKickTitle
        } else {
            locales::LocaleKey::ListBanTitle
        };

        let args = args!(server = self.config.teamtalk.display_name().to_string());
        let base = locales::get_text(self.lang.as_str(), title_key, args.as_ref());
        let title = append_search_hint(&base, self.lang);

        let keyboard = create_user_list_keyboard(
            &users,
            0,
            move |u| {
                let action = if is_kick {
                    AdminAction::KickPerform { user_id: u.id }
                } else {
                    AdminAction::BanPerform { user_id: u.id }
                };
                (u.nickname.clone(), CallbackAction::Admin(action))
            },
            move |p| {
                let action = if is_kick {
                    AdminAction::KickList { page: p }
                } else {
                    AdminAction::BanList { page: p }
                };
                CallbackAction::Admin(action)
            },
            None,
            self.lang,
        );

        let sent = self
            .bot
            .send_message(self.msg.chat.id, title)
            .reply_to(self.msg.id)
            .reply_markup(keyboard)
            .await?;
        let list_type = if is_kick {
            SearchListType::Kick
        } else {
            SearchListType::Ban
        };
        set_search_context(
            self.state,
            sent.chat.id,
            SearchContext {
                message_id: sent.id,
                list_type,
            },
        )
        .await;
        Ok(())
    }

    async fn unban(&self) -> ResponseResult<()> {
        if !self.is_admin {
            send_text_key(
                self.bot,
                self.msg.chat.id,
                self.lang,
                locales::LocaleKey::CmdUnauth,
                Some(self.msg.id),
            )
            .await?;
            return Ok(());
        }
        send_unban_list(
            self.bot,
            self.msg.chat.id,
            self.db,
            &self.state.search_contexts,
            self.lang,
            0,
            Some(self.msg.id),
        )
        .await
    }

    async fn subscribers(&self) -> ResponseResult<()> {
        if !self.is_admin {
            send_text_key(
                self.bot,
                self.msg.chat.id,
                self.lang,
                locales::LocaleKey::CmdUnauth,
                Some(self.msg.id),
            )
            .await?;
            return Ok(());
        }
        send_subscribers_list(
            self.bot,
            self.msg.chat.id,
            self.db,
            &self.state.search_contexts,
            self.lang,
            0,
            Some(self.msg.id),
        )
        .await
    }

    async fn exit(&self) -> ResponseResult<()> {
        if !self.is_admin {
            send_text_key(
                self.bot,
                self.msg.chat.id,
                self.lang,
                locales::LocaleKey::CmdUnauth,
                Some(self.msg.id),
            )
            .await?;
            return Ok(());
        }
        self.bot
            .send_message(
                self.msg.chat.id,
                locales::get_text(
                    self.lang.as_str(),
                    locales::LocaleKey::CmdShuttingDown,
                    None,
                ),
            )
            .reply_to(self.msg.id)
            .await?;
        let _ = self.state.tx_tt.send(TtCommand::Shutdown).await;
        self.state.cancel_token.cancel();
        Ok(())
    }

    async fn broadcast(&self, text: String) -> ResponseResult<()> {
        if !self.is_admin {
            send_text_key(
                self.bot,
                self.msg.chat.id,
                self.lang,
                locales::LocaleKey::CmdUnauth,
                Some(self.msg.id),
            )
            .await?;
            return Ok(());
        }

        let text = text.trim().to_string();
        if text.is_empty() {
            send_text_key(
                self.bot,
                self.msg.chat.id,
                self.lang,
                locales::LocaleKey::CmdBroadcastEmpty,
                Some(self.msg.id),
            )
            .await?;
            return Ok(());
        }

        if let Err(e) = self.tx_tt.send(TtCommand::Broadcast { text }).await {
            tracing::error!(error = %e, "Failed to send broadcast command");
            notify_admin_error(
                self.bot,
                self.config,
                self.telegram_id,
                AdminErrorContext::Command,
                &e.to_string(),
                self.lang,
            )
            .await;
            send_text_key(
                self.bot,
                self.msg.chat.id,
                self.lang,
                locales::LocaleKey::CmdError,
                Some(self.msg.id),
            )
            .await?;
            return Ok(());
        }

        send_text_key(
            self.bot,
            self.msg.chat.id,
            self.lang,
            locales::LocaleKey::CmdBroadcastSent,
            Some(self.msg.id),
        )
        .await
    }

    async fn message(&self, text: String) -> ResponseResult<()> {
        if !self.is_admin {
            send_text_key(
                self.bot,
                self.msg.chat.id,
                self.lang,
                locales::LocaleKey::CmdUnauth,
                Some(self.msg.id),
            )
            .await?;
            return Ok(());
        }

        let text = text.trim().to_string();
        if text.is_empty() {
            send_text_key(
                self.bot,
                self.msg.chat.id,
                self.lang,
                locales::LocaleKey::CmdMessageEmpty,
                Some(self.msg.id),
            )
            .await?;
            return Ok(());
        }

        let subs = match self.db.get_subscribers().await {
            Ok(subs) => subs,
            Err(e) => {
                tracing::error!(error = %e, "Failed to load subscribers");
                notify_admin_error(
                    self.bot,
                    self.config,
                    self.telegram_id,
                    AdminErrorContext::Command,
                    &e.to_string(),
                    self.lang,
                )
                .await;
                send_text_key(
                    self.bot,
                    self.msg.chat.id,
                    self.lang,
                    locales::LocaleKey::CmdError,
                    Some(self.msg.id),
                )
                .await?;
                return Ok(());
            }
        };

        let mut sent = 0usize;
        let mut failed = 0usize;
        for sub in subs {
            if sub.telegram_id == self.telegram_id {
                continue;
            }
            let chat_id = ChatId(sub.telegram_id.as_i64());
            match self.bot.send_message(chat_id, text.clone()).await {
                Ok(_) => sent += 1,
                Err(e) => {
                    failed += 1;
                    tracing::warn!(
                        telegram_id = sub.telegram_id.as_i64(),
                        error = %e,
                        "Failed to send broadcast message"
                    );
                }
            }
        }

        let args = args!(sent = sent, failed = failed);
        let reply = locales::get_text(
            self.lang.as_str(),
            locales::LocaleKey::CmdMessageSent,
            args.as_ref(),
        );
        self.bot
            .send_message(self.msg.chat.id, reply)
            .reply_to(self.msg.id)
            .await?;
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    async fn queue(&self, text: String) -> ResponseResult<()> {
        let text = text.trim();
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.is_empty() {
            let help =
                locales::get_text(self.lang.as_str(), locales::LocaleKey::CmdQueueHelp, None);
            self.bot
                .send_message(self.msg.chat.id, help)
                .reply_to(self.msg.id)
                .await?;
            return Ok(());
        }

        match parts.as_slice() {
            ["on" | "off"] => {
                if !self.is_admin {
                    send_text_key(
                        self.bot,
                        self.msg.chat.id,
                        self.lang,
                        locales::LocaleKey::CmdUnauth,
                        Some(self.msg.id),
                    )
                    .await?;
                    return Ok(());
                }
                let enabled = parts[0] == "on";
                let current =
                    match reply_queue_service::get_reply_queue_global_enabled(self.db).await {
                        Ok(val) => val,
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to load global queue setting");
                            notify_admin_error(
                                self.bot,
                                self.config,
                                self.telegram_id,
                                AdminErrorContext::Command,
                                &e.to_string(),
                                self.lang,
                            )
                            .await;
                            send_text_key(
                                self.bot,
                                self.msg.chat.id,
                                self.lang,
                                locales::LocaleKey::CmdError,
                                Some(self.msg.id),
                            )
                            .await?;
                            return Ok(());
                        }
                    };
                if current == enabled {
                    let status_key = if enabled {
                        locales::LocaleKey::RespQueueGlobalAlreadyEnabled
                    } else {
                        locales::LocaleKey::RespQueueGlobalAlreadyDisabled
                    };
                    send_text_key(
                        self.bot,
                        self.msg.chat.id,
                        self.lang,
                        status_key,
                        Some(self.msg.id),
                    )
                    .await?;
                    return Ok(());
                }
                if let Err(e) =
                    reply_queue_service::set_reply_queue_global_enabled(self.db, enabled).await
                {
                    tracing::error!(error = %e, "Failed to update global queue setting");
                    notify_admin_error(
                        self.bot,
                        self.config,
                        self.telegram_id,
                        AdminErrorContext::Command,
                        &e.to_string(),
                        self.lang,
                    )
                    .await;
                    send_text_key(
                        self.bot,
                        self.msg.chat.id,
                        self.lang,
                        locales::LocaleKey::CmdError,
                        Some(self.msg.id),
                    )
                    .await?;
                    return Ok(());
                }
                let status_key = if enabled {
                    locales::LocaleKey::RespQueueGlobalEnabled
                } else {
                    locales::LocaleKey::RespQueueGlobalDisabled
                };
                send_text_key(
                    self.bot,
                    self.msg.chat.id,
                    self.lang,
                    status_key,
                    Some(self.msg.id),
                )
                .await?;
            }
            ["me", "on" | "off"] => {
                let enabled = parts[1] == "on";
                let tt_username = match self
                    .db
                    .get_tt_username_by_telegram_id(self.telegram_id)
                    .await
                {
                    Ok(val) => val,
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to load TT username");
                        send_text_key(
                            self.bot,
                            self.msg.chat.id,
                            self.lang,
                            locales::LocaleKey::CmdError,
                            Some(self.msg.id),
                        )
                        .await?;
                        return Ok(());
                    }
                };
                let Some(_tt_username) = tt_username else {
                    send_text_key(
                        self.bot,
                        self.msg.chat.id,
                        self.lang,
                        locales::LocaleKey::CmdQueueNoLink,
                        Some(self.msg.id),
                    )
                    .await?;
                    return Ok(());
                };
                let global_enabled =
                    match reply_queue_service::get_reply_queue_global_enabled(self.db).await {
                        Ok(val) => val,
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to load global queue setting");
                            send_text_key(
                                self.bot,
                                self.msg.chat.id,
                                self.lang,
                                locales::LocaleKey::CmdError,
                                Some(self.msg.id),
                            )
                            .await?;
                            return Ok(());
                        }
                    };
                if !global_enabled {
                    send_text_key(
                        self.bot,
                        self.msg.chat.id,
                        self.lang,
                        locales::LocaleKey::RespQueueGlobalDisabledUser,
                        Some(self.msg.id),
                    )
                    .await?;
                    return Ok(());
                }
                let current = match reply_queue_service::get_reply_queue_user_enabled(
                    self.db,
                    self.telegram_id,
                )
                .await
                {
                    Ok(val) => val,
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to load queue setting");
                        send_text_key(
                            self.bot,
                            self.msg.chat.id,
                            self.lang,
                            locales::LocaleKey::CmdError,
                            Some(self.msg.id),
                        )
                        .await?;
                        return Ok(());
                    }
                };
                if current == enabled {
                    let status_key = if enabled {
                        locales::LocaleKey::RespQueueUserAlreadyEnabled
                    } else {
                        locales::LocaleKey::RespQueueUserAlreadyDisabled
                    };
                    send_text_key(
                        self.bot,
                        self.msg.chat.id,
                        self.lang,
                        status_key,
                        Some(self.msg.id),
                    )
                    .await?;
                    return Ok(());
                }
                if let Err(e) = reply_queue_service::set_reply_queue_user_enabled(
                    self.db,
                    self.telegram_id,
                    enabled,
                )
                .await
                {
                    tracing::error!(error = %e, "Failed to update queue setting");
                    notify_admin_error(
                        self.bot,
                        self.config,
                        self.telegram_id,
                        AdminErrorContext::Command,
                        &e.to_string(),
                        self.lang,
                    )
                    .await;
                    send_text_key(
                        self.bot,
                        self.msg.chat.id,
                        self.lang,
                        locales::LocaleKey::CmdError,
                        Some(self.msg.id),
                    )
                    .await?;
                    return Ok(());
                }
                let status_key = if enabled {
                    locales::LocaleKey::RespQueueUserEnabled
                } else {
                    locales::LocaleKey::RespQueueUserDisabled
                };
                send_text_key(
                    self.bot,
                    self.msg.chat.id,
                    self.lang,
                    status_key,
                    Some(self.msg.id),
                )
                .await?;
            }
            ["clear"] | ["clear", "all"] => {
                if parts.len() == 2 && parts[1] == "all" {
                    if !self.is_admin {
                        send_text_key(
                            self.bot,
                            self.msg.chat.id,
                            self.lang,
                            locales::LocaleKey::CmdUnauth,
                            Some(self.msg.id),
                        )
                        .await?;
                        return Ok(());
                    }
                    let cleared = match self.db.clear_reply_queue_all().await {
                        Ok(count) => count,
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to clear reply queue");
                            notify_admin_error(
                                self.bot,
                                self.config,
                                self.telegram_id,
                                AdminErrorContext::Command,
                                &e.to_string(),
                                self.lang,
                            )
                            .await;
                            send_text_key(
                                self.bot,
                                self.msg.chat.id,
                                self.lang,
                                locales::LocaleKey::CmdError,
                                Some(self.msg.id),
                            )
                            .await?;
                            return Ok(());
                        }
                    };
                    let text = locales::get_text(
                        self.lang.as_str(),
                        locales::LocaleKey::RespQueueClearedAll,
                        args!(count = cleared).as_ref(),
                    );
                    self.bot
                        .send_message(self.msg.chat.id, text)
                        .reply_to(self.msg.id)
                        .await?;
                } else {
                    let tt_username = match self
                        .db
                        .get_tt_username_by_telegram_id(self.telegram_id)
                        .await
                    {
                        Ok(val) => val,
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to load TT username");
                            send_text_key(
                                self.bot,
                                self.msg.chat.id,
                                self.lang,
                                locales::LocaleKey::CmdError,
                                Some(self.msg.id),
                            )
                            .await?;
                            return Ok(());
                        }
                    };
                    let Some(tt_username) = tt_username else {
                        send_text_key(
                            self.bot,
                            self.msg.chat.id,
                            self.lang,
                            locales::LocaleKey::CmdQueueNoLink,
                            Some(self.msg.id),
                        )
                        .await?;
                        return Ok(());
                    };
                    let cleared = match self.db.clear_reply_queue_for_user(&tt_username).await {
                        Ok(count) => count,
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to clear reply queue");
                            notify_admin_error(
                                self.bot,
                                self.config,
                                self.telegram_id,
                                AdminErrorContext::Command,
                                &e.to_string(),
                                self.lang,
                            )
                            .await;
                            send_text_key(
                                self.bot,
                                self.msg.chat.id,
                                self.lang,
                                locales::LocaleKey::CmdError,
                                Some(self.msg.id),
                            )
                            .await?;
                            return Ok(());
                        }
                    };
                    let text = locales::get_text(
                        self.lang.as_str(),
                        locales::LocaleKey::RespQueueCleared,
                        args!(count = cleared).as_ref(),
                    );
                    self.bot
                        .send_message(self.msg.chat.id, text)
                        .reply_to(self.msg.id)
                        .await?;
                }
            }
            _ => {
                let help =
                    locales::get_text(self.lang.as_str(), locales::LocaleKey::CmdQueueHelp, None);
                self.bot
                    .send_message(self.msg.chat.id, help)
                    .reply_to(self.msg.id)
                    .await?;
            }
        }

        Ok(())
    }
}

pub async fn answer_message(bot: Bot, msg: Message, state: AppState) -> ResponseResult<()> {
    let Some(user) = &msg.from else {
        return Ok(());
    };
    let telegram_id = tg_user_id_i64(user.id.0);
    let config = &state.config;
    let db = &state.db;

    let default_lang = config.general.default_lang;
    let admin_lang = user_settings_service::get_or_create(db, telegram_id, default_lang)
        .await
        .map(|u| u.language_code)
        .unwrap_or(default_lang);

    if maybe_handle_search_message(&bot, &msg, &state, admin_lang).await? {
        return Ok(());
    }

    if !is_admin(db, config, telegram_id).await {
        return Ok(());
    }

    handle_admin_reply(&bot, &msg, &state, telegram_id, admin_lang).await
}

async fn is_admin(
    db: &crate::infra::db::Database,
    config: &crate::bootstrap::config::Config,
    telegram_id: TelegramId,
) -> bool {
    if telegram_id == config.telegram.admin_chat_id {
        return true;
    }
    match db.get_all_admins().await {
        Ok(admins) => admins.contains(&telegram_id),
        Err(e) => {
            tracing::error!(error = %e, "Failed to load admin list");
            false
        }
    }
}

async fn handle_admin_reply(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    telegram_id: TelegramId,
    admin_lang: LanguageCode,
) -> ResponseResult<()> {
    let config = &state.config;
    let reply_to = msg.reply_to_message();
    let text = msg.text();
    let voice = msg.voice();

    if reply_to.is_none() {
        if let Some(voice) = voice {
            let reply_key = match stream_voice(bot, state, None, voice).await {
                Ok(()) => locales::LocaleKey::TgReplySent,
                Err(e) => {
                    notify_admin_error(
                        bot,
                        config,
                        telegram_id,
                        AdminErrorContext::Command,
                        &e,
                        admin_lang,
                    )
                    .await;
                    locales::LocaleKey::TgReplyFailed
                }
            };
            let reply_text = locales::get_text(admin_lang.as_str(), reply_key, None);
            let _ = bot
                .send_message(msg.chat.id, reply_text)
                .reply_to(msg.id)
                .await;
        }
        return Ok(());
    }

    let Some(reply_to) = reply_to else {
        return Ok(());
    };
    let reply_id = crate::core::types::TgMessageId::from(reply_to.id.0);

    if handle_channel_reply(
        ChannelReplyCtx {
            bot,
            msg,
            state,
            telegram_id,
            admin_lang,
        },
        ChannelReplyInput {
            reply_id,
            text,
            voice,
        },
    )
    .await?
    {
        return Ok(());
    }

    let Some(text) = text else {
        return Ok(());
    };
    handle_user_reply(bot, msg, state, telegram_id, admin_lang, reply_id, text).await
}

struct ChannelReplyCtx<'a> {
    bot: &'a Bot,
    msg: &'a Message,
    state: &'a AppState,
    telegram_id: TelegramId,
    admin_lang: LanguageCode,
}

struct ChannelReplyInput<'a> {
    reply_id: crate::core::types::TgMessageId,
    text: Option<&'a str>,
    voice: Option<&'a Voice>,
}

async fn handle_channel_reply(
    ctx: ChannelReplyCtx<'_>,
    input: ChannelReplyInput<'_>,
) -> ResponseResult<bool> {
    let db = &ctx.state.db;
    let config = &ctx.state.config;
    if let Ok(Some((channel_id, _channel_name, _server_name, original_text))) =
        pending_service::get_pending_channel_reply(db, input.reply_id).await
    {
        let mut reply_key = locales::LocaleKey::TgReplySent;
        if let Some(voice) = input.voice {
            let duration = format_duration(voice.duration.seconds());
            let args = args!(msg = original_text.clone(), duration = duration);
            let announce_text = locales::get_text(
                ctx.admin_lang.as_str(),
                locales::LocaleKey::TtChannelReply,
                args.as_ref(),
            );
            if let Err(e) =
                stream_voice(ctx.bot, ctx.state, Some((channel_id, announce_text)), voice).await
            {
                notify_admin_error(
                    ctx.bot,
                    config,
                    ctx.telegram_id,
                    AdminErrorContext::Command,
                    &e,
                    ctx.admin_lang,
                )
                .await;
                reply_key = locales::LocaleKey::TgReplyFailed;
            }
        } else if let Some(text) = input.text {
            let args = args!(msg = original_text.clone(), reply = text.to_string());
            let channel_text = locales::get_text(
                ctx.admin_lang.as_str(),
                locales::LocaleKey::TtChannelReplyText,
                args.as_ref(),
            );
            if let Err(e) = ctx
                .state
                .tx_tt
                .send(TtCommand::SendToChannel {
                    channel_id,
                    text: channel_text,
                })
                .await
            {
                tracing::error!(
                    channel_id = channel_id.as_i32(),
                    error = %e,
                    "Failed to send TT channel reply"
                );
                notify_admin_error(
                    ctx.bot,
                    config,
                    ctx.telegram_id,
                    AdminErrorContext::Command,
                    &e.to_string(),
                    ctx.admin_lang,
                )
                .await;
                reply_key = locales::LocaleKey::TgReplyFailed;
            }
        } else {
            return Ok(true);
        }

        let reply_text = locales::get_text(ctx.admin_lang.as_str(), reply_key, None);
        let _ = ctx
            .bot
            .send_message(ctx.msg.chat.id, reply_text)
            .reply_to(ctx.msg.id)
            .await;

        if let Err(e) = pending_service::touch_pending_channel_reply(db, input.reply_id).await {
            tracing::error!(
                reply_id = input.reply_id.as_i32(),
                error = %e,
                "Failed to update pending channel reply"
            );
        }

        return Ok(true);
    }
    Ok(false)
}

async fn load_pending_reply(
    bot: &Bot,
    db: &Database,
    config: &Config,
    telegram_id: TelegramId,
    reply_id: crate::core::types::TgMessageId,
) -> ResponseResult<Option<(crate::core::types::TtUserId, Option<TtUsername>)>> {
    match pending_service::get_pending_reply(db, reply_id).await {
        Ok(Some(data)) => Ok(Some(data)),
        Ok(None) => Ok(None),
        Err(e) => {
            tracing::error!(
                reply_id = reply_id.as_i32(),
                error = %e,
                "Failed to load pending reply"
            );
            notify_admin_error(
                bot,
                config,
                telegram_id,
                AdminErrorContext::Command,
                &e.to_string(),
                config.general.default_lang,
            )
            .await;
            Ok(None)
        }
    }
}

async fn resolve_current_tt_user_id(
    state: &AppState,
    tt_username: Option<&TtUsername>,
    tt_user_id: crate::core::types::TtUserId,
) -> Option<crate::core::types::TtUserId> {
    if let Some(username) = tt_username {
        state
            .state
            .user_id_by_username(username)
            .await
            .ok()
            .flatten()
            .or(Some(tt_user_id))
    } else {
        Some(tt_user_id)
    }
}

async fn is_tt_user_online(state: &AppState, user_id: crate::core::types::TtUserId) -> bool {
    state
        .state
        .online_user_by_id(user_id)
        .await
        .ok()
        .flatten()
        .is_some()
}

async fn handle_user_reply(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    telegram_id: TelegramId,
    admin_lang: LanguageCode,
    reply_id: crate::core::types::TgMessageId,
    text: &str,
) -> ResponseResult<()> {
    let db = &state.db;
    let config = &state.config;
    let Some((tt_user_id, tt_username)) =
        load_pending_reply(bot, db, config, telegram_id, reply_id).await?
    else {
        return Ok(());
    };

    let current_tt_user_id =
        resolve_current_tt_user_id(state, tt_username.as_ref(), tt_user_id).await;
    let is_online = match current_tt_user_id {
        Some(id) => is_tt_user_online(state, id).await,
        None => false,
    };

    let reply_key = if is_online {
        let Some(target_id) = current_tt_user_id else {
            return Ok(());
        };
        let send_res = state
            .tx_tt
            .send(TtCommand::ReplyToUser {
                user_id: target_id,
                text: text.to_string(),
            })
            .await;
        if let Err(e) = send_res {
            tracing::error!(
                tt_user_id = tt_user_id.as_i32(),
                error = %e,
                "Failed to send TT reply command"
            );
            notify_admin_error(
                bot,
                config,
                telegram_id,
                AdminErrorContext::Command,
                &e.to_string(),
                admin_lang,
            )
            .await;
            locales::LocaleKey::TgReplyFailed
        } else {
            locales::LocaleKey::TgReplySent
        }
    } else if let Some(tt_username) = tt_username.as_ref() {
        match reply_queue_service::is_reply_queue_enabled_for_tt_user(db, tt_username).await {
            Ok(true) => {
                if let Err(e) = db
                    .add_reply_queue_item(tt_username, telegram_id, text)
                    .await
                {
                    tracing::error!(
                        tt_username = %tt_username,
                        error = %e,
                        "Failed to queue reply"
                    );
                    locales::LocaleKey::TgReplyFailed
                } else {
                    locales::LocaleKey::TgReplyQueued
                }
            }
            Ok(false) => locales::LocaleKey::TgReplyOffline,
            Err(e) => {
                tracing::error!(error = %e, "Failed to check reply queue status");
                locales::LocaleKey::TgReplyFailed
            }
        }
    } else {
        locales::LocaleKey::TgReplyOffline
    };
    let reply_text = locales::get_text(admin_lang.as_str(), reply_key, None);
    let _ = bot
        .send_message(msg.chat.id, reply_text)
        .reply_to(msg.id)
        .await;

    if let Err(e) = pending_service::touch_pending_reply(db, reply_id).await {
        tracing::error!(
            reply_id = reply_id.as_i32(),
            error = %e,
            "Failed to update pending reply"
        );
    }

    Ok(())
}

fn format_duration(duration_secs: u32) -> String {
    let minutes = duration_secs / 60;
    let seconds = duration_secs % 60;
    format!("{minutes:02}:{seconds:02}")
}

async fn stream_voice(
    bot: &Bot,
    state: &AppState,
    announce: Option<(crate::core::types::TtChannelId, String)>,
    voice: &Voice,
) -> Result<(), String> {
    let file_info = bot
        .get_file(voice.file.id.clone())
        .await
        .map_err(|e| e.to_string())?;
    let mut temp_path = std::env::temp_dir();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    temp_path.push(format!("tg-voice-{}-{}.ogg", voice.file.id, now));
    let mut dst = File::create(&temp_path).await.map_err(|e| e.to_string())?;
    bot.download_file(&file_info.path, &mut dst)
        .await
        .map_err(|e| e.to_string())?;

    let duration_ms = voice.duration.seconds().saturating_mul(1000);
    let (channel_id, announce_text) =
        announce.map_or_else(|| (crate::core::types::TtChannelId::from(0), None), |(id, text)| {
            (id, Some(text))
        });
    state
        .tx_tt
        .send(TtCommand::EnqueueStream {
            channel_id,
            file_path: temp_path.to_string_lossy().to_string(),
            duration_ms,
            announce_text,
        })
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
