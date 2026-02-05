use crate::infra::locales;

use super::user::UserCtx;

pub(super) async fn handle_help(ctx: &UserCtx) {
    let is_main_admin = ctx
        .admin_username
        .as_ref()
        .is_some_and(|u| u == &ctx.username);
    let mut help_msg =
        locales::get_text_or_log(ctx.reply_lang.as_str(), locales::LocaleKey::HelpText, None);
    if is_main_admin {
        let header = locales::get_text_or_log(
            ctx.reply_lang.as_str(),
            locales::LocaleKey::TtAdminHelpHeader,
            None,
        );
        let cmds = locales::get_text_or_log(
            ctx.reply_lang.as_str(),
            locales::LocaleKey::TtAdminHelpCmds,
            None,
        );
        help_msg.push_str(&header);
        help_msg.push_str(&cmds);
    }
    ctx.send_reply(help_msg).await;
}
