pub mod admin_logic;
pub mod callback_handlers;
pub mod callbacks;
pub mod callbacks_types;
pub mod commands;
pub mod keyboards;
pub mod settings_logic;
pub mod state;
pub mod utils;

use crate::config::Config;
use crate::db::Database;
use crate::locales;
use crate::types::{LiteUser, TtCommand};
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
        log::error!("Failed to set bot commands: {}", e);
    }

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(commands::answer_command),
        )
        .branch(Update::filter_callback_query().endpoint(callbacks::answer_callback));

    Dispatcher::builder(event_bot, handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .error_handler(std::sync::Arc::new(
            |err: teloxide::errors::RequestError| async move {
                let err_str = err.to_string();
                if !err_str.contains("TerminatedByOtherGetUpdates") {
                    log::error!("❌ [TELEGRAM] Update listener error: {}", err);
                }
            },
        ))
        .build()
        .dispatch()
        .await;
}

async fn set_bot_commands(
    bot: &Bot,
    db: &Database,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let languages = vec!["en", "ru"];

    let default_lang = &config.general.default_lang;
    let global_commands = get_user_commands(default_lang);
    bot.set_my_commands(global_commands)
        .scope(BotCommandScope::AllPrivateChats)
        .await?;

    for lang in &languages {
        if lang == default_lang {
            continue;
        }
        let cmds = get_user_commands(lang);
        bot.set_my_commands(cmds)
            .scope(BotCommandScope::AllPrivateChats)
            .language_code(*lang)
            .await?;
    }

    let admin_ids = db.get_all_admins().await.unwrap_or_default();
    for admin_id in admin_ids {
        let user_settings = db
            .get_or_create_user(admin_id, default_lang)
            .await
            .unwrap_or_else(|_| crate::db::types::UserSettings {
                telegram_id: admin_id,
                language_code: default_lang.clone(),
                notification_settings: "all".to_string(),
                mute_list_mode: "blacklist".to_string(),
                teamtalk_username: None,
                not_on_online_enabled: false,
                not_on_online_confirmed: false,
            });

        let admin_cmds = get_admin_commands(&user_settings.language_code);

        bot.set_my_commands(admin_cmds)
            .scope(BotCommandScope::Chat {
                chat_id: Recipient::Id(teloxide::types::ChatId(admin_id)),
            })
            .await
            .ok();
    }

    Ok(())
}

fn get_user_commands(lang: &str) -> Vec<BotCommand> {
    vec![
        BotCommand::new("menu", locales::get_text(lang, "cmd-desc-menu", None)),
        BotCommand::new("who", locales::get_text(lang, "cmd-desc-who", None)),
        BotCommand::new(
            "settings",
            locales::get_text(lang, "cmd-desc-settings", None),
        ),
        BotCommand::new("unsub", locales::get_text(lang, "cmd-desc-unsub", None)),
        BotCommand::new("help", locales::get_text(lang, "cmd-desc-help", None)),
    ]
}

fn get_admin_commands(lang: &str) -> Vec<BotCommand> {
    let mut cmds = get_user_commands(lang);
    cmds.extend(vec![
        BotCommand::new("kick", locales::get_text(lang, "cmd-desc-kick", None)),
        BotCommand::new("ban", locales::get_text(lang, "cmd-desc-ban", None)),
        BotCommand::new("unban", locales::get_text(lang, "cmd-desc-unban", None)),
        BotCommand::new(
            "subscribers",
            locales::get_text(lang, "cmd-desc-subscribers", None),
        ),
        BotCommand::new("exit", locales::get_text(lang, "cmd-desc-exit", None)),
    ]);
    cmds
}
