use crate::db::Database;
use crate::locales;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub async fn ensure_subscribed(bot: &Bot, msg: &Message, db: &Database, lang: &str) -> bool {
    if let Ok(true) = db.is_subscribed(msg.chat.id.0).await {
        true
    } else {
        bot.send_message(
            msg.chat.id,
            locales::get_text(lang, "cmd-not-subscribed", None),
        )
        .parse_mode(ParseMode::Html)
        .await
        .ok();
        false
    }
}

pub async fn check_db_err(
    bot: &Bot,
    query_id: &str,
    result: anyhow::Result<()>,
    lang: &str,
) -> ResponseResult<bool> {
    if let Err(e) = result {
        tracing::error!("❌ Database Error: {:?}", e);

        let error_text = locales::get_text(lang, "cmd-error", None);
        bot.answer_callback_query(teloxide::types::CallbackQueryId(query_id.to_string()))
            .text(error_text)
            .show_alert(true)
            .await?;

        return Ok(true);
    }
    Ok(false)
}
