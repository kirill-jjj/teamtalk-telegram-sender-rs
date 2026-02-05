use crate::app::services::pending as pending_service;
use crate::app::services::user_settings as user_settings_service;
use crate::args;
use crate::core::types::{TtChannelId, TtChannelName, TtServerName};
use crate::infra::locales;
use crate::infra::locales::LocaleKey;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::utils::html;

use super::BridgeDeps;

pub(super) async fn handle_to_admin_channel(
    deps: &BridgeDeps<'_>,
    channel_id: TtChannelId,
    channel_name: TtChannelName,
    server_name: TtServerName,
    msg_content: String,
) {
    let bot = deps.msg_bot.or(if deps.message_token_present {
        deps.event_bot
    } else {
        None
    });
    let Some(bot) = bot else {
        tracing::debug!(
            component = "bridge",
            "Skipping admin alert: message_token not configured"
        );
        return;
    };

    let admin_settings = user_settings_service::get_or_create(
        &deps.services.db,
        crate::core::types::TelegramId::from(deps.admin_id.0),
        deps.default_lang,
    )
    .await;
    let admin_lang = match admin_settings {
        Ok(u) => u.language_code,
        Err(e) => {
            tracing::error!(
                component = "bridge",
                error = %e,
                "Failed to get admin settings; defaulting to 'en'"
            );
            crate::core::types::LanguageCode::En
        }
    };

    let args_admin = args!(
        server = html::escape(server_name.as_str()),
        channel = html::escape(channel_name.as_str()),
        msg = html::escape(&msg_content)
    );
    let text_admin = locales::get_text_or_log(
        admin_lang.as_str(),
        LocaleKey::AdminChannelPm,
        args_admin.as_ref(),
    );

    let res = bot
        .send_message(deps.admin_id, &text_admin)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await;
    if let Ok(msg) = &res
        && let Err(e) = pending_service::add_pending_channel_reply(
            &deps.services.db,
            crate::core::types::TgMessageId::from(msg.id.0),
            channel_id,
            &channel_name,
            &server_name,
            &msg_content,
        )
        .await
    {
        tracing::error!(
            component = "bridge",
            message_id = msg.id.0,
            error = %e,
            "Failed to save pending channel reply"
        );
    }
}
