use crate::core::types::BridgeEvent;

use super::user::UserCtx;

pub(super) async fn handle_admin_bridge(ctx: &UserCtx) {
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
