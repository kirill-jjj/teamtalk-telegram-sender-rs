mod admin;
mod afk;
mod basic;
mod queue;
mod replies;
mod reply_users;
mod voice;

use crate::adapters::tg::handlers::search::maybe_handle_search_message;
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::{TgErrorReporter, send_text_key, telegram_id_from_user};
use crate::app::plugins::{TgCommandContext, parse_command_text};
use crate::app::services::tg_admin as tg_admin_service;
use crate::app::services::tg_commands as tg_commands_service;
use crate::core::types::{AdminErrorContext, LanguageCode, TelegramId, TtCommand};
use crate::infra::locales;
use serde_json::json;
use std::sync::Arc;
use teloxide_ng::prelude::*;
use teloxide_ng::sugar::request::RequestReplyExt;
use teloxide_ng::utils::command::BotCommands;

use self::replies::{ChannelReplyCtx, ChannelReplyInput, handle_channel_reply};
use self::reply_users::handle_user_reply;
use self::voice::stream_voice;

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
    #[command(description = "AFK notifications")]
    Afk(String),
    #[command(description = "Plugins (Admin)")]
    Plugins(String),
}

pub async fn answer_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: Arc<AppState>,
) -> ResponseResult<()> {
    let Some(user) = &msg.from else {
        return Ok(());
    };
    let Some(telegram_id) = telegram_id_from_user(user, "answer_command") else {
        return Ok(());
    };
    if let Some((command, args)) = parse_command_from_enum(&cmd) {
        let is_admin = tg_commands_service::is_admin(&state.db, &state.config, telegram_id).await;
        if state
            .plugins
            .dispatch_tg_command(
                &command,
                &args,
                TgCommandContext {
                    chat_id: msg.chat.id.0,
                    user_id: telegram_id.as_i64(),
                    is_admin,
                    text: msg.text().unwrap_or_default().to_string(),
                },
            )
            .await
        {
            return Ok(());
        }
    }
    let Some(ctx) = CommandCtx::new(&bot, &msg, state.as_ref(), telegram_id).await? else {
        return Ok(());
    };
    ctx.dispatch(cmd).await?;
    Ok(())
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
    const fn errors(&self) -> TgErrorReporter<'_> {
        TgErrorReporter::new(self.bot, self.config, self.telegram_id, self.lang)
    }

    async fn new(
        bot: &'a Bot,
        msg: &'a Message,
        state: &'a AppState,
        telegram_id: TelegramId,
    ) -> ResponseResult<Option<Self>> {
        let db = &state.db;
        let config = &state.config;
        let default_lang = config.general.default_lang;
        let lang = match tg_commands_service::load_user_lang(db, telegram_id, default_lang).await {
            Ok(lang) => lang,
            Err(err) => {
                let error = err.into_error();
                tracing::error!(
                    telegram_id = telegram_id.as_i64(),
                    error = %error,
                    "Failed to get or create user"
                );
                TgErrorReporter::new(bot, config, telegram_id, default_lang)
                    .notify(AdminErrorContext::Command, &error.to_string())
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
        let is_admin = if telegram_id == config.telegram.admin_chat_id {
            true
        } else {
            match tg_admin_service::list_admins(db).await {
                Ok(admins) => admins.contains(&telegram_id),
                Err(e) => {
                    let error = e.into_error();
                    tracing::error!(error = %error, "Failed to load admin list");
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
            Command::Afk(text) => self.afk(text).await,
            Command::Plugins(text) => self.plugins(text).await,
        }
    }

    async fn start(&self, token: String) -> ResponseResult<()> {
        basic::handle_start(self, token).await
    }

    async fn menu(&self) -> ResponseResult<()> {
        basic::handle_menu(self).await
    }

    async fn help(&self) -> ResponseResult<()> {
        basic::handle_help(self).await
    }

    async fn who(&self) -> ResponseResult<()> {
        basic::handle_who(self).await
    }

    async fn settings(&self) -> ResponseResult<()> {
        basic::handle_settings(self).await
    }

    async fn unsub(&self) -> ResponseResult<()> {
        basic::handle_unsub(self).await
    }

    async fn kick_or_ban(&self, cmd: Command) -> ResponseResult<()> {
        admin::handle_kick_or_ban(self, cmd).await
    }

    async fn unban(&self) -> ResponseResult<()> {
        admin::handle_unban(self).await
    }

    async fn subscribers(&self) -> ResponseResult<()> {
        admin::handle_subscribers(self).await
    }

    async fn exit(&self) -> ResponseResult<()> {
        admin::handle_exit(self).await
    }

    async fn broadcast(&self, text: String) -> ResponseResult<()> {
        admin::handle_broadcast(self, text).await
    }

    async fn message(&self, text: String) -> ResponseResult<()> {
        admin::handle_message(self, text).await
    }

    async fn queue(&self, text: String) -> ResponseResult<()> {
        queue::handle_queue(self, text).await
    }

    async fn afk(&self, text: String) -> ResponseResult<()> {
        afk::handle_afk(self, text).await
    }

    async fn plugins(&self, text: String) -> ResponseResult<()> {
        admin::handle_plugins(self, text).await
    }
}

pub async fn answer_message(bot: Bot, msg: Message, state: Arc<AppState>) -> ResponseResult<()> {
    let Some(user) = &msg.from else {
        return Ok(());
    };
    let Some(telegram_id) = telegram_id_from_user(user, "answer_message") else {
        return Ok(());
    };
    let config = &state.config;
    let db = &state.db;

    let default_lang = config.general.default_lang;
    let admin_lang = match tg_commands_service::load_user_lang(db, telegram_id, default_lang).await
    {
        Ok(lang) => lang,
        Err(err) => {
            tracing::error!(
                telegram_id = telegram_id.as_i64(),
                error = ?err,
                "Failed to load admin language, using default"
            );
            default_lang
        }
    };

    state
        .plugins
        .dispatch_event(crate::app::plugins::PluginEvent {
            name: "Message".to_string(),
            source: "tg".to_string(),
            normalized: json!({
                "chat_id": msg.chat.id.0,
                "user_id": telegram_id.as_i64(),
                "has_text": msg.text().is_some(),
                "is_reply": msg.reply_to_message().is_some(),
            }),
            raw: json!({
                "text": msg.text(),
                "message_id": msg.id.0,
                "chat_id": msg.chat.id.0,
            }),
        })
        .await;

    if maybe_handle_search_message(&bot, &msg, state.as_ref(), admin_lang).await? {
        return Ok(());
    }

    if let Some(text) = msg.text()
        && text.starts_with('/')
        && let Some((command, args)) = parse_command_text(text)
    {
        let is_admin = tg_commands_service::is_admin(db, config, telegram_id).await;
        if state
            .plugins
            .dispatch_tg_command(
                &command,
                &args,
                TgCommandContext {
                    chat_id: msg.chat.id.0,
                    user_id: telegram_id.as_i64(),
                    is_admin,
                    text: text.to_string(),
                },
            )
            .await
        {
            return Ok(());
        }
    }

    if !tg_commands_service::is_admin(db, config, telegram_id).await {
        return Ok(());
    }

    handle_admin_reply(&bot, &msg, state.as_ref(), telegram_id, admin_lang).await
}

async fn handle_admin_reply(
    bot: &Bot,
    msg: &Message,
    state: &AppState,
    telegram_id: TelegramId,
    admin_lang: LanguageCode,
) -> ResponseResult<()> {
    let errors = TgErrorReporter::new(bot, &state.config, telegram_id, admin_lang);
    let reply_to_id = msg
        .reply_to_message()
        .map(|reply_msg| crate::core::types::TgMessageId::from(reply_msg.id.0));
    let text = msg.text().map(str::to_string);
    let voice = msg.voice().cloned();

    if reply_to_id.is_none() {
        if let Some(voice) = voice.as_ref() {
            let reply_key = match stream_voice(bot, state, None, voice).await {
                Ok(()) => locales::LocaleKey::TgReplySent,
                Err(e) => {
                    errors
                        .notify(AdminErrorContext::Command, &e.to_string())
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

    let Some(reply_id) = reply_to_id else {
        return Ok(());
    };

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
            text: text.as_deref(),
            voice: voice.as_ref(),
        },
    )
    .await?
    {
        return Ok(());
    }

    let Some(text) = text.as_deref() else {
        return Ok(());
    };
    handle_user_reply(bot, msg, state, telegram_id, admin_lang, reply_id, text).await
}

fn format_duration(duration_secs: u32) -> String {
    let minutes = duration_secs / 60;
    let seconds = duration_secs % 60;
    format!("{minutes:02}:{seconds:02}")
}

fn parse_command_from_enum(command: &Command) -> Option<(String, Vec<String>)> {
    match command {
        Command::Start(token) => Some(("start".to_string(), vec![token.clone()])),
        Command::Menu => Some(("menu".to_string(), Vec::new())),
        Command::Help => Some(("help".to_string(), Vec::new())),
        Command::Who => Some(("who".to_string(), Vec::new())),
        Command::Settings => Some(("settings".to_string(), Vec::new())),
        Command::Unsub => Some(("unsub".to_string(), Vec::new())),
        Command::Kick => Some(("kick".to_string(), Vec::new())),
        Command::Ban => Some(("ban".to_string(), Vec::new())),
        Command::Unban => Some(("unban".to_string(), Vec::new())),
        Command::Subscribers => Some(("subscribers".to_string(), Vec::new())),
        Command::Exit => Some(("exit".to_string(), Vec::new())),
        Command::Broadcast(text) => Some(("broadcast".to_string(), vec![text.clone()])),
        Command::Message(text) => Some(("message".to_string(), vec![text.clone()])),
        Command::Queue(text) => parse_command_text(&format!("/queue {text}")),
        Command::Afk(text) => parse_command_text(&format!("/afk {text}")),
        Command::Plugins(text) => parse_command_text(&format!("/plugins {text}")),
    }
}
