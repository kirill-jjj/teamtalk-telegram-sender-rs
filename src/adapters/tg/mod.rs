pub mod admin_logic;
pub mod callback_handlers;
pub mod callbacks;
pub mod commands;
pub mod keyboards;
pub mod settings_logic;
pub mod state;
pub mod utils;

use crate::adapters::tg::utils::notify_admin_error;
use crate::app::services::admin as admin_service;
use crate::app::services::user_settings as user_settings_service;
use crate::bootstrap::config::Config;
use crate::core::types::{AdminErrorContext, LanguageCode, LiteUser, TtCommand};
use crate::infra::db::Database;
use crate::infra::locales;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use teamtalk::types::UserAccount;
use teloxide::{
    prelude::*,
    types::{BotCommand, BotCommandScope, Recipient},
};

use self::commands::Command;
use self::state::AppState;

pub async fn run_tg_bot(
    event_bot: Bot,
    db: Database,
    online_users: Arc<DashMap<i32, LiteUser>>,
    user_accounts: Arc<DashMap<String, UserAccount>>,
    tx_tt_cmd: Sender<TtCommand>,
    config: Arc<Config>,
) {
    let state = AppState {
        db: db.clone(),
        online_users,
        user_accounts,
        tx_tt: tx_tt_cmd,
        config: config.clone(),
    };

    if let Err(e) = set_bot_commands(&event_bot, &db, &config).await {
        tracing::error!("Failed to set bot commands: {}", e);
    }

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(commands::answer_command),
        )
        .branch(Update::filter_message().endpoint(commands::answer_message))
        .branch(Update::filter_callback_query().endpoint(callbacks::answer_callback));

    let admin_bot = event_bot.clone();
    let admin_config = config.clone();
    Dispatcher::builder(event_bot, handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .error_handler(std::sync::Arc::new({
            let admin_bot = admin_bot.clone();
            let admin_config = admin_config.clone();
            move |err: teloxide::errors::RequestError| {
                let admin_bot = admin_bot.clone();
                let admin_config = admin_config.clone();
                async move {
                    let err_str = err.to_string();
                    if !err_str.contains("TerminatedByOtherGetUpdates") {
                        tracing::error!("[TELEGRAM] Update listener error: {}", err);
                        let default_lang = LanguageCode::from_str_or_default(
                            &admin_config.general.default_lang,
                            LanguageCode::En,
                        );
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
            }
        }))
        .build()
        .dispatch()
        .await;
}

async fn set_bot_commands(
    bot: &Bot,
    db: &Database,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let languages = vec![LanguageCode::En, LanguageCode::Ru];

    let default_lang =
        LanguageCode::from_str_or_default(&config.general.default_lang, LanguageCode::En);
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

    let mut admin_ids = match admin_service::list_admins(db).await {
        Ok(ids) => ids,
        Err(e) => {
            tracing::error!("Failed to load admin list: {}", e);
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
                tracing::error!("Failed to load admin settings for {}: {}", admin_id, e);
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
                tracing::error!("Failed to set admin commands for {}: {}", admin_id, e);
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
    ]);
    cmds
}
