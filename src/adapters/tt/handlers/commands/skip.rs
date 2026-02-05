use crate::core::types::TtCommand;
use crate::infra::locales;

use super::user::UserCtx;

pub(super) async fn handle_skip(ctx: &UserCtx) {
    if !ctx.is_admin().await {
        let text = locales::get_text_or_log(ctx.reply_lang.as_str(), locales::LocaleKey::CmdUnauth, None);
        ctx.send_reply(text).await;
        return;
    }
    if let Err(e) = ctx.tx_tt_cmd.send(TtCommand::SkipStream).await {
        tracing::error!(
            tt_username = %ctx.username,
            error = %e,
            "Failed to send TT skip command"
        );
        let text = locales::get_text_or_log(
            ctx.reply_lang.as_str(),
            locales::LocaleKey::TtErrorGeneric,
            None,
        );
        ctx.send_reply(text).await;
        return;
    }
    let text = locales::get_text_or_log(
        ctx.reply_lang.as_str(),
        locales::LocaleKey::TtSkipSent,
        None,
    );
    ctx.send_reply(text).await;
}
