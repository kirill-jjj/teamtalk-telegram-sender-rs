use crate::app::services::subscription as subscriptions_service;
use crate::bootstrap::config::Config;
use crate::core::types::{AdminErrorContext, LanguageCode, TelegramId};
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
        check_db_err(
            self.bot,
            query_id,
            result,
            self.config,
            self.user_id,
            context,
            self.lang,
        )
        .await
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

pub async fn check_db_err(
    bot: &Bot,
    query_id: &str,
    result: anyhow::Result<()>,
    config: &Config,
    user_id: TelegramId,
    context: AdminErrorContext,
    lang: LanguageCode,
) -> ResponseResult<bool> {
    if let Err(e) = result {
        tracing::error!(error = ?e, "Database error");
        notify_admin_error(bot, config, user_id, context, &e.to_string(), lang).await;

        bot.answer_callback_query(teloxide::types::CallbackQueryId(query_id.to_string()))
            .text(cmd_error_text(lang))
            .show_alert(true)
            .await?;

        return Ok(true);
    }
    Ok(false)
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
