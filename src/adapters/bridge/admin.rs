use crate::app::services::pending as pending_service;
use crate::app::services::tt_users as tt_users_service;
use crate::app::services::user_settings as user_settings_service;
use crate::args;
use crate::core::types::{LanguageCode, TtNickname, TtServerName, TtUserId, TtUsername};
use crate::infra::locales;
use crate::infra::locales::LocaleKey;
use teloxide_ng::payloads::SendMessageSetters;
use teloxide_ng::prelude::Requester;
use teloxide_ng::utils::html;

use super::BridgeDeps;

pub(super) async fn handle_to_admin(
    deps: &BridgeDeps<'_>,
    user_id: TtUserId,
    nick: TtNickname,
    tt_username: TtUsername,
    msg_content: String,
    server_name: TtServerName,
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
            LanguageCode::En
        }
    };

    let args_admin = args!(
        server = html::escape(server_name.as_str()),
        nick = html::escape(nick.as_str()),
        msg = html::escape(&msg_content)
    );
    let text_admin = locales::get_text(
        admin_lang.as_str(),
        LocaleKey::AdminAlert,
        args_admin.as_ref(),
    );

    let res = bot
        .send_message(deps.admin_id, &text_admin)
        .parse_mode(teloxide_ng::types::ParseMode::Html)
        .await;
    if let Ok(msg) = &res
        && let Err(e) = pending_service::add_pending_reply(
            &deps.services.db,
            crate::core::types::TgMessageId::from(msg.id.0),
            user_id,
            Some(&tt_username),
        )
        .await
    {
        tracing::error!(
            component = "bridge",
            message_id = msg.id.0,
            tt_username = %tt_username,
            error = %e,
            "Failed to save pending reply"
        );
    }

    let reply_lang =
        tt_users_service::get_user_lang_by_tt_user(&deps.services, &tt_username, deps.default_lang)
            .await;

    let key_reply = if res.is_ok() {
        LocaleKey::TtMsgSent
    } else {
        LocaleKey::TtMsgFailed
    };
    let reply_text = locales::get_text(reply_lang.as_str(), key_reply, None);

    if let Err(e) = deps
        .tx_tt_cmd
        .send(crate::core::types::TtCommand::ReplyToUser {
            user_id,
            text: reply_text,
        })
        .await
    {
        tracing::error!(
            component = "bridge",
            user_id = user_id.as_i32(),
            tt_username = %tt_username,
            error = %e,
            "Failed to send TT reply command"
        );
    }
}
