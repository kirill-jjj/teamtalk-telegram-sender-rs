use crate::core::types::TgChatId;
use std::path::PathBuf;
use teloxide_ng::payloads::SendDocumentSetters;
use teloxide_ng::prelude::Requester;
use teloxide_ng::types::{ChatId, InputFile};

use super::BridgeDeps;

pub(super) async fn handle_tg_document(
    deps: &BridgeDeps<'_>,
    chat_id: TgChatId,
    file_path: PathBuf,
    caption: Option<String>,
    delete_after_send: bool,
) {
    let bot = deps.msg_bot.or(if deps.message_token_present {
        deps.event_bot
    } else {
        None
    });
    let Some(bot) = bot else {
        tracing::debug!(
            component = "bridge",
            "Skipping document send: message_token not configured"
        );
        return;
    };

    let mut request =
        bot.send_document(ChatId(chat_id.as_i64()), InputFile::file(file_path.clone()));
    if let Some(caption) = caption {
        request = request.caption(caption);
    }

    if let Err(error) = request.await {
        tracing::error!(
            component = "bridge",
            chat_id = chat_id.as_i64(),
            path = %file_path.display(),
            error = %error,
            "Failed to send document to Telegram"
        );
        return;
    }

    if delete_after_send {
        let delete_path = file_path.clone();
        tokio::task::spawn_blocking(move || {
            if let Err(error) = std::fs::remove_file(&delete_path) {
                tracing::warn!(
                    component = "bridge",
                    path = %delete_path.display(),
                    error = %error,
                    "Failed to delete sent recording file"
                );
            }
        });
    }
}
