use crate::config::Config;
use crate::db::Database;
use crate::locales;
use crate::types::LanguageCode;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub async fn ensure_subscribed(
    bot: &Bot,
    msg: &Message,
    db: &Database,
    config: &Config,
    lang: LanguageCode,
) -> bool {
    match db.is_subscribed(msg.chat.id.0).await {
        Ok(true) => true,
        Ok(false) => {
            if let Err(e) = bot
                .send_message(
                    msg.chat.id,
                    locales::get_text(lang.as_str(), "cmd-not-subscribed", None),
                )
                .parse_mode(ParseMode::Html)
                .await
            {
                tracing::error!("Failed to send not-subscribed message: {}", e);
            }
            false
        }
        Err(e) => {
            tracing::error!(
                "Database error checking subscription for {}: {}",
                msg.chat.id.0,
                e
            );
            notify_admin_error(
                bot,
                config,
                msg.chat.id.0,
                "admin-error-context-subscription",
                &e.to_string(),
                lang,
            )
            .await;
            if let Err(e) = bot
                .send_message(
                    msg.chat.id,
                    locales::get_text(lang.as_str(), "cmd-error", None),
                )
                .parse_mode(ParseMode::Html)
                .await
            {
                tracing::error!("Failed to send not-subscribed message: {}", e);
            }
            false
        }
    }
}

pub async fn check_db_err(
    bot: &Bot,
    query_id: &str,
    result: anyhow::Result<()>,
    config: &Config,
    user_id: i64,
    context: &str,
    lang: LanguageCode,
) -> ResponseResult<bool> {
    if let Err(e) = result {
        tracing::error!("? Database Error: {:?}", e);
        notify_admin_error(bot, config, user_id, context, &e.to_string(), lang).await;

        let error_text = locales::get_text(lang.as_str(), "cmd-error", None);
        bot.answer_callback_query(teloxide::types::CallbackQueryId(query_id.to_string()))
            .text(error_text)
            .show_alert(true)
            .await?;

        return Ok(true);
    }
    Ok(false)
}

pub async fn notify_admin_error(
    bot: &Bot,
    config: &Config,
    user_id: i64,
    context: &str,
    error: &str,
    lang: LanguageCode,
) {
    let admin_chat_id = teloxide::types::ChatId(config.telegram.admin_chat_id);
    let context_text = locales::get_text(lang.as_str(), context, None);
    let args = crate::args!(
        user_id = user_id.to_string(),
        context = context_text,
        error = error.to_string()
    );
    let text = locales::get_text(lang.as_str(), "admin-error-user", args.as_ref());
    if let Err(e) = bot.send_message(admin_chat_id, text).await {
        tracing::error!("Failed to notify admin about error: {}", e);
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
    key: &str,
) -> ResponseResult<()> {
    bot.send_message(chat_id, locales::get_text(lang.as_str(), key, None))
        .await?;
    Ok(())
}
