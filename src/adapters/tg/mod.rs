pub mod admin_logic;
pub mod callback_handlers;
pub mod callbacks;
pub mod commands;
pub mod keyboards;
pub mod settings_logic;
pub mod state;
pub mod utils;

use crate::adapters::tg::utils::notify_admin_error;
use crate::app::services::user_settings as user_settings_service;
use crate::bootstrap::config::Config;
use crate::core::types::{AdminErrorContext, LanguageCode, LiteUser, TtCommand};
use crate::infra::db::Database;
use crate::infra::locales;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use teamtalk::types::UserAccount;
use teloxide::error_handlers::ErrorHandler;
use teloxide::{
    prelude::*,
    types::{BotCommand, BotCommandScope, Recipient},
};
use tokio::sync::mpsc::Sender;

use self::commands::Command;
use self::state::AppState;

pub struct TgRunArgs {
    pub event_bot: Bot,
    pub message_bot: Option<Bot>,
    pub db: Database,
    pub online_users: Arc<RwLock<HashMap<i32, LiteUser>>>,
    pub user_accounts: Arc<RwLock<HashMap<String, UserAccount>>>,
    pub tx_tt_cmd: Sender<TtCommand>,
    pub config: Arc<Config>,
    pub cancel_token: tokio_util::sync::CancellationToken,
}

pub async fn run_tg_bot(args: TgRunArgs) {
    let TgRunArgs {
        event_bot,
        message_bot,
        db,
        online_users,
        user_accounts,
        tx_tt_cmd,
        config,
        cancel_token,
    } = args;
    let state = build_state(
        db.clone(),
        online_users,
        user_accounts,
        tx_tt_cmd,
        &config,
        &cancel_token,
    );

    if let Err(e) = set_bot_commands(&event_bot, &db, &config).await {
        tracing::error!(error = %e, "Failed to set bot commands");
    }

    let msg_handle = message_bot
        .map(|bot| spawn_message_bot(bot, state.clone(), config.clone(), cancel_token.clone()));
    run_event_bot(event_bot, state, config, cancel_token).await;

    if let Some(handle) = msg_handle {
        handle.abort();
    }
}

fn build_state(
    db: Database,
    online_users: Arc<RwLock<HashMap<i32, LiteUser>>>,
    user_accounts: Arc<RwLock<HashMap<String, UserAccount>>>,
    tx_tt_cmd: Sender<TtCommand>,
    config: &Arc<Config>,
    cancel_token: &tokio_util::sync::CancellationToken,
) -> AppState {
    AppState {
        db,
        online_users,
        user_accounts,
        tx_tt: tx_tt_cmd,
        config: config.clone(),
        cancel_token: cancel_token.clone(),
    }
}

fn make_error_handler(
    admin_bot: Bot,
    admin_config: Arc<Config>,
) -> std::sync::Arc<dyn ErrorHandler<teloxide::errors::RequestError> + Send + Sync> {
    std::sync::Arc::new(move |err: teloxide::errors::RequestError| {
        let admin_bot = admin_bot.clone();
        let admin_config = admin_config.clone();
        async move {
            let err_str = err.to_string();
            if !err_str.contains("TerminatedByOtherGetUpdates")
                && !err_str.contains("message is not modified")
            {
                tracing::error!(
                    component = "telegram",
                    error = %err,
                    "Update listener error"
                );
                let default_lang = admin_config.general.default_lang;
                notify_admin_error(
                    &admin_bot,
                    &admin_config,
                    0,
                    AdminErrorContext::UpdateListener,
                    &err_str,
                    default_lang,
                )
                .await;
            }
        }
    })
}

fn spawn_message_bot(
    message_bot: Bot,
    state: AppState,
    config: Arc<Config>,
    cancel_token: tokio_util::sync::CancellationToken,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let msg_handler =
            dptree::entry().branch(Update::filter_message().endpoint(commands::answer_message));
        let mut dispatcher = Dispatcher::builder(message_bot.clone(), msg_handler)
            .dependencies(dptree::deps![state])
            .error_handler(make_error_handler(message_bot.clone(), config.clone()))
            .build();
        run_dispatcher(&mut dispatcher, cancel_token).await;
    })
}

async fn run_event_bot(
    event_bot: Bot,
    state: AppState,
    config: Arc<Config>,
    cancel_token: tokio_util::sync::CancellationToken,
) {
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(commands::answer_command),
        )
        .branch(Update::filter_message().endpoint(commands::answer_message))
        .branch(Update::filter_callback_query().endpoint(callbacks::answer_callback));
    let mut dispatcher = Dispatcher::builder(event_bot.clone(), handler)
        .dependencies(dptree::deps![state])
        .error_handler(make_error_handler(event_bot.clone(), config))
        .build();
    run_dispatcher(&mut dispatcher, cancel_token).await;
}

async fn run_dispatcher(
    dispatcher: &mut Dispatcher<
        Bot,
        teloxide::errors::RequestError,
        teloxide::dispatching::DefaultKey,
    >,
    cancel_token: tokio_util::sync::CancellationToken,
) {
    let shutdown_token = dispatcher.shutdown_token();
    let shutdown_task = tokio::spawn(async move {
        cancel_token.cancelled().await;
        if let Ok(fut) = shutdown_token.shutdown() {
            fut.await;
        }
    });
    dispatcher.dispatch().await;
    shutdown_task.abort();
}

async fn set_bot_commands(
    bot: &Bot,
    db: &Database,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let languages = vec![LanguageCode::En, LanguageCode::Ru];

    let default_lang = config.general.default_lang;
    let global_commands = get_user_commands(default_lang);
    bot.set_my_commands(global_commands)
        .scope(BotCommandScope::AllPrivateChats)
        .await?;

    for lang in &languages {
        if *lang == default_lang {
            continue;
        }
        let cmds = get_user_commands(*lang);
        bot.set_my_commands(cmds)
            .scope(BotCommandScope::AllPrivateChats)
            .language_code(lang.as_str())
            .await?;
    }

    let mut admin_ids = match db.get_all_admins().await {
        Ok(ids) => ids,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load admin list");
            Vec::new()
        }
    };
    let config_admin_id = config.telegram.admin_chat_id;
    if !admin_ids.contains(&config_admin_id) {
        admin_ids.push(config_admin_id);
    }
    for admin_id in admin_ids {
        let user_settings = user_settings_service::get_or_create(db, admin_id, default_lang)
            .await
            .unwrap_or_else(|e| {
                tracing::error!(
                    admin_id,
                    error = %e,
                    "Failed to load admin settings"
                );
                crate::infra::db::types::UserSettings {
                    telegram_id: admin_id,
                    language_code: default_lang.as_str().to_string(),
                    notification_settings: "all".to_string(),
                    mute_list_mode: "blacklist".to_string(),
                    teamtalk_username: None,
                    not_on_online_enabled: false,
                    not_on_online_confirmed: false,
                }
            });

        let admin_lang =
            LanguageCode::from_str_or_default(&user_settings.language_code, default_lang);
        let admin_cmds = get_admin_commands(admin_lang);

        bot.set_my_commands(admin_cmds)
            .scope(BotCommandScope::Chat {
                chat_id: Recipient::Id(teloxide::types::ChatId(admin_id)),
            })
            .await
            .unwrap_or_else(|e| {
                tracing::error!(
                    admin_id,
                    error = %e,
                    "Failed to set admin commands"
                );
                teloxide::types::True
            });
    }

    Ok(())
}

fn get_user_commands(lang: LanguageCode) -> Vec<BotCommand> {
    vec![
        BotCommand::new(
            "menu",
            locales::get_text(lang.as_str(), "cmd-desc-menu", None),
        ),
        BotCommand::new(
            "who",
            locales::get_text(lang.as_str(), "cmd-desc-who", None),
        ),
        BotCommand::new(
            "settings",
            locales::get_text(lang.as_str(), "cmd-desc-settings", None),
        ),
        BotCommand::new(
            "unsub",
            locales::get_text(lang.as_str(), "cmd-desc-unsub", None),
        ),
        BotCommand::new(
            "help",
            locales::get_text(lang.as_str(), "cmd-desc-help", None),
        ),
    ]
}

fn get_admin_commands(lang: LanguageCode) -> Vec<BotCommand> {
    let mut cmds = get_user_commands(lang);
    cmds.extend(vec![
        BotCommand::new(
            "kick",
            locales::get_text(lang.as_str(), "cmd-desc-kick", None),
        ),
        BotCommand::new(
            "ban",
            locales::get_text(lang.as_str(), "cmd-desc-ban", None),
        ),
        BotCommand::new(
            "unban",
            locales::get_text(lang.as_str(), "cmd-desc-unban", None),
        ),
        BotCommand::new(
            "subscribers",
            locales::get_text(lang.as_str(), "cmd-desc-subscribers", None),
        ),
        BotCommand::new(
            "exit",
            locales::get_text(lang.as_str(), "cmd-desc-exit", None),
        ),
        BotCommand::new(
            "broadcast",
            locales::get_text(lang.as_str(), "cmd-desc-broadcast", None),
        ),
        BotCommand::new(
            "message",
            locales::get_text(lang.as_str(), "cmd-desc-message", None),
        ),
    ]);
    cmds
}
