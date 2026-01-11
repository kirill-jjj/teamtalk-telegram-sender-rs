use crate::config::Config;
use crate::db::Database;
use crate::locales;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub async fn ensure_subscribed(
    bot: &Bot,
    msg: &Message,
    db: &Database,
    config: &Config,
    lang: &str,
) -> bool {
    match db.is_subscribed(msg.chat.id.0).await {
        Ok(true) => true,
        Ok(false) => {
            if let Err(e) = bot
                .send_message(
                    msg.chat.id,
                    locales::get_text(lang, "cmd-not-subscribed", None),
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
                .send_message(msg.chat.id, locales::get_text(lang, "cmd-error", None))
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
    lang: &str,
) -> ResponseResult<bool> {
    if let Err(e) = result {
        tracing::error!("? Database Error: {:?}", e);
        notify_admin_error(bot, config, user_id, context, &e.to_string(), lang).await;

        let error_text = locales::get_text(lang, "cmd-error", None);
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
    lang: &str,
) {
    let admin_chat_id = teloxide::types::ChatId(config.telegram.admin_chat_id);
    let context_text = locales::get_text(lang, context, None);
    let args = crate::args!(
        user_id = user_id.to_string(),
        context = context_text,
        error = error.to_string()
    );
    let text = locales::get_text(lang, "admin-error-user", args.as_ref());
    if let Err(e) = bot.send_message(admin_chat_id, text).await {
        tracing::error!("Failed to notify admin about error: {}", e);
    }
}
