use crate::app::services::subscription as subscriptions_service;
use crate::app::services::tg_settings as tg_settings_service;
use crate::app::services::user_settings as user_settings_service;
use crate::bootstrap::config::Config;
use crate::core::types::{AdminErrorContext, LanguageCode, TelegramId, TtUsername};
use crate::infra::db::Database;
use crate::infra::locales;
use crate::infra::locales::LocaleKey;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestReplyExt;
use teloxide::types::ParseMode;

pub async fn ensure_subscribed(
    bot: &Bot,
    msg: &Message,
    db: &Database,
    config: &Config,
    lang: LanguageCode,
) -> bool {
    match subscriptions_service::is_subscribed(db, TelegramId::from(msg.chat.id.0)).await {
        Ok(true) => true,
        Ok(false) => {
            if let Err(e) = bot
                .send_message(
                    msg.chat.id,
                    locales::get_text(lang.as_str(), locales::LocaleKey::CmdNotSubscribed, None),
                )
                .parse_mode(ParseMode::Html)
                .reply_to(msg.id)
                .await
            {
                tracing::error!(
                    error = %e,
                    chat_id = msg.chat.id.0,
                    "Failed to send not-subscribed message"
                );
            }
            false
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                chat_id = msg.chat.id.0,
                "Database error checking subscription"
            );
            notify_admin_error(
                bot,
                config,
                TelegramId::from(msg.chat.id.0),
                AdminErrorContext::Subscription,
                &e.to_string(),
                lang,
            )
            .await;
            if let Err(e) = bot
                .send_message(
                    msg.chat.id,
                    locales::get_text(lang.as_str(), locales::LocaleKey::CmdError, None),
                )
                .parse_mode(ParseMode::Html)
                .reply_to(msg.id)
                .await
            {
                tracing::error!(
                    error = %e,
                    chat_id = msg.chat.id.0,
                    "Failed to send error message"
                );
            }
            false
        }
    }
}

pub fn telegram_id_from_user_id(user_id: u64, context: &'static str) -> Option<TelegramId> {
    TelegramId::try_from(user_id).map_or_else(
        |_| {
            tracing::error!(user_id, context, "Telegram user id does not fit into i64");
            None
        },
        Some,
    )
}

pub fn telegram_id_from_callback_query(
    q: &teloxide::types::CallbackQuery,
    context: &'static str,
) -> Option<TelegramId> {
    telegram_id_from_user_id(q.from.id.0, context)
}

pub fn telegram_id_from_user(
    user: &teloxide::types::User,
    context: &'static str,
) -> Option<TelegramId> {
    telegram_id_from_user_id(user.id.0, context)
}

pub fn cmd_error_text(lang: LanguageCode) -> String {
    locales::get_text(lang.as_str(), locales::LocaleKey::CmdError, None)
}

pub struct TgErrorReporter<'a> {
    bot: &'a Bot,
    config: &'a Config,
    user_id: TelegramId,
    lang: LanguageCode,
}

impl<'a> TgErrorReporter<'a> {
    pub const fn new(
        bot: &'a Bot,
        config: &'a Config,
        user_id: TelegramId,
        lang: LanguageCode,
    ) -> Self {
        Self {
            bot,
            config,
            user_id,
            lang,
        }
    }

    pub async fn notify(&self, context: AdminErrorContext, error: &str) {
        notify_admin_error(
            self.bot,
            self.config,
            self.user_id,
            context,
            error,
            self.lang,
        )
        .await;
    }

    pub const fn user_id(&self) -> TelegramId {
        self.user_id
    }

    pub async fn check_db_err(
        &self,
        query_id: &str,
        result: anyhow::Result<()>,
        context: AdminErrorContext,
    ) -> ResponseResult<bool> {
        if let Err(e) = result {
            tracing::error!(error = ?e, "Database error");
            notify_admin_error(
                self.bot,
                self.config,
                self.user_id,
                context,
                &e.to_string(),
                self.lang,
            )
            .await;

            self.bot
                .answer_callback_query(teloxide::types::CallbackQueryId(query_id.to_string()))
                .text(cmd_error_text(self.lang))
                .show_alert(true)
                .await?;

            return Ok(true);
        }
        Ok(false)
    }
}

pub async fn answer_cmd_error_callback(
    bot: &Bot,
    query_id: &teloxide::types::CallbackQueryId,
    lang: LanguageCode,
    show_alert: bool,
) -> ResponseResult<()> {
    answer_callback(bot, query_id, cmd_error_text(lang), show_alert).await
}

pub async fn notify_admin_error(
    bot: &Bot,
    config: &Config,
    user_id: TelegramId,
    context: AdminErrorContext,
    error: &str,
    lang: LanguageCode,
) {
    let admin_chat_id = teloxide::types::ChatId(config.telegram.admin_chat_id.as_i64());
    let context_key = match context {
        AdminErrorContext::Command => LocaleKey::AdminErrorContextCommand,
        AdminErrorContext::Callback => LocaleKey::AdminErrorContextCallback,
        AdminErrorContext::Subscription => LocaleKey::AdminErrorContextSubscription,
        AdminErrorContext::TtCommand => LocaleKey::AdminErrorContextTtCommand,
        AdminErrorContext::UpdateListener => LocaleKey::AdminErrorContextUpdateListener,
    };
    let context_text = locales::get_text(lang.as_str(), context_key, None);
    let args = crate::args!(
        user_id = user_id.to_string(),
        context = context_text,
        error = error.to_string()
    );
    let text = locales::get_text(
        lang.as_str(),
        locales::LocaleKey::AdminErrorUser,
        args.as_ref(),
    );
    if let Err(e) = bot.send_message(admin_chat_id, text).await {
        tracing::error!(error = %e, "Failed to notify admin about error");
    }
}

pub async fn answer_callback(
    bot: &Bot,
    query_id: &teloxide::types::CallbackQueryId,
    text: String,
    alert: bool,
) -> ResponseResult<()> {
    let req = bot.answer_callback_query(query_id.clone()).text(text);
    if alert {
        req.show_alert(true).await?;
    } else {
        req.await?;
    }
    Ok(())
}

pub async fn answer_callback_empty(
    bot: &Bot,
    query_id: &teloxide::types::CallbackQueryId,
) -> ResponseResult<()> {
    bot.answer_callback_query(query_id.clone()).await?;
    Ok(())
}

pub async fn send_text_key(
    bot: &Bot,
    chat_id: teloxide::types::ChatId,
    lang: LanguageCode,
    key: LocaleKey,
    reply_to: Option<teloxide::types::MessageId>,
) -> ResponseResult<()> {
    let req = bot.send_message(chat_id, locales::get_text(lang.as_str(), key, None));
    if let Some(reply_to) = reply_to {
        req.reply_to(reply_to).await?;
    } else {
        req.await?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
pub enum AdminSubEventKind {
    Subscribed,
    Unsubscribed,
}

pub async fn notify_admins_subscription_event(
    bot: &Bot,
    db: &Database,
    admin_chat_id: TelegramId,
    actor_id: TelegramId,
    tt_username: Option<&TtUsername>,
    event: AdminSubEventKind,
) {
    let mut admin_ids = match db.get_all_admins().await {
        Ok(ids) => ids,
        Err(err) => {
            tracing::error!(error = %err, "Failed to load admin list for subscription event notify");
            Vec::new()
        }
    };
    if !admin_ids.contains(&admin_chat_id) {
        admin_ids.push(admin_chat_id);
    }

    for admin_id in admin_ids {
        let enabled = match tg_settings_service::admin_sub_events_enabled(db, admin_id).await {
            Ok(enabled) => enabled,
            Err(err) => {
                tracing::error!(
                    error = %err,
                    admin_id = admin_id.as_i64(),
                    "Failed to read admin subscription events setting"
                );
                false
            }
        };
        if !enabled {
            continue;
        }

        let admin_lang = match user_settings_service::get_or_create(db, admin_id, LanguageCode::En)
            .await
        {
            Ok(settings) => settings.language_code,
            Err(err) => {
                tracing::error!(error = %err, admin_id = admin_id.as_i64(), "Failed to load admin language for subscription event notify");
                LanguageCode::En
            }
        };

        let tt_username_value = tt_username.map(TtUsername::as_str).map_or_else(
            || locales::get_text(admin_lang.as_str(), LocaleKey::ValNone, None),
            str::to_string,
        );

        let key = match event {
            AdminSubEventKind::Subscribed => LocaleKey::AdminSubEventSubscribed,
            AdminSubEventKind::Unsubscribed => LocaleKey::AdminSubEventUnsubscribed,
        };
        let text = locales::get_text(
            admin_lang.as_str(),
            key,
            crate::args!(
                user_id = actor_id.as_i64().to_string(),
                tt_username = tt_username_value
            )
            .as_ref(),
        );
        if let Err(err) = bot
            .send_message(teloxide::types::ChatId(admin_id.as_i64()), text)
            .parse_mode(ParseMode::Html)
            .await
        {
            tracing::error!(
                error = %err,
                admin_id = admin_id.as_i64(),
                actor_id = actor_id.as_i64(),
                "Failed to send admin subscription event notification"
            );
        }
    }
}
