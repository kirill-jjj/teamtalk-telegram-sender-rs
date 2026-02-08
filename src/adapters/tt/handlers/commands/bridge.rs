use crate::core::types::BridgeEvent;
use crate::infra::locales;
use std::time::{Duration, Instant};

use super::user::UserCtx;

pub(super) async fn handle_admin_bridge(ctx: &UserCtx) {
    if !ctx.message_token_present {
        let now = Instant::now();
        let should_reply = {
            let mut state = ctx.tt_bridge_disabled_reply_state.lock().await;
            let last_sent_at = state.get(&ctx.from_uid).copied();
            if should_send_disabled_hint(last_sent_at, now, ctx.tt_bridge_disabled_reply_cooldown) {
                state.insert(ctx.from_uid, now);
                true
            } else {
                false
            }
        };
        if should_reply {
            let text = locales::get_text(
                ctx.reply_lang.as_str(),
                locales::LocaleKey::TtBridgeDisabledUser,
                None,
            );
            ctx.send_reply(text).await;
            tracing::info!(
                user_id = ctx.from_uid.as_i32(),
                cooldown_seconds = ctx.tt_bridge_disabled_reply_cooldown.as_secs(),
                "Skipped TT->TG admin bridge: message_token not configured"
            );
        } else {
            tracing::debug!(
                user_id = ctx.from_uid.as_i32(),
                "Suppressed repeated TT->TG disabled notice by cooldown"
            );
        }
        return;
    }
    let server_name = ctx.server_name();
    if let Err(e) = ctx
        .tx_bridge
        .send(BridgeEvent::ToAdmin {
            user_id: ctx.from_uid,
            nick: ctx.nick.clone(),
            tt_username: ctx.username.clone(),
            msg_content: ctx.content.clone(),
            server_name,
        })
        .await
    {
        tracing::error!(error = %e, "Failed to send admin bridge event");
    }
}

fn should_send_disabled_hint(
    last_sent_at: Option<Instant>,
    now: Instant,
    cooldown: Duration,
) -> bool {
    last_sent_at.is_none_or(|last_sent_at| now.saturating_duration_since(last_sent_at) >= cooldown)
}

#[cfg(test)]
#[path = "../../../../../tests/unit/tt_bridge_command.rs"]
mod tests;
