use crate::core::types::{TgChatId, TgMessageId};
use teloxide_ng::payloads::SendMessageSetters;
use teloxide_ng::prelude::Requester;
use teloxide_ng::sugar::request::RequestReplyExt;

use super::BridgeDeps;

pub(super) async fn handle_who_report(
    deps: &BridgeDeps<'_>,
    chat_id: TgChatId,
    text: String,
    reply_to: Option<TgMessageId>,
) {
    if let Some(bot) = deps.event_bot
        && let Err(e) = {
            let req = bot
                .send_message(teloxide_ng::types::ChatId(chat_id.as_i64()), &text)
                .parse_mode(teloxide_ng::types::ParseMode::Html);
            if let Some(reply_to) = reply_to {
                req.reply_to(teloxide_ng::types::MessageId(reply_to.as_i32()))
                    .await
            } else {
                req.await
            }
        }
    {
        tracing::error!(
            component = "bridge",
            chat_id = chat_id.as_i64(),
            error = %e,
            "Failed to send who report"
        );
    }
}
