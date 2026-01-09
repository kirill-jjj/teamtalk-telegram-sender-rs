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
