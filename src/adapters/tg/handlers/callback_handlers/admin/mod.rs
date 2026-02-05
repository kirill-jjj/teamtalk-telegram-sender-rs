use crate::adapters::tg::state::AppState;
use crate::core::callbacks::AdminAction;
use crate::core::types::LanguageCode;
use teloxide::prelude::*;

mod lists;
mod perform;
mod subs;
mod unban;

pub async fn handle_admin(
    bot: Bot,
    q: CallbackQuery,
    state: &AppState,
    action: AdminAction,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let Some(teloxide::types::MaybeInaccessibleMessage::Regular(ref msg)) = q.message else {
        return Ok(());
    };
    let msg = msg.as_ref();
    match action {
        AdminAction::KickList { page } => {
            lists::handle_kick_list(&bot, &q, state, msg, page, lang).await?;
        }
        AdminAction::BanList { page } => {
            lists::handle_ban_list(&bot, &q, state, msg, page, lang).await?;
        }
        AdminAction::KickPerform { user_id } => {
            perform::handle_kick_perform(&bot, &q, state, user_id, lang).await?;
        }
        AdminAction::BanPerform { user_id } => {
            perform::handle_ban_perform(&bot, &q, state, user_id, lang).await?;
        }
        AdminAction::UnbanList { page } => {
            unban::handle_unban_list(&bot, &q, state, msg, page, lang).await?;
        }
        AdminAction::UnbanPerform { ban_db_id, page } => {
            unban::handle_unban_perform(&bot, &q, state, msg, ban_db_id, page, lang).await?;
        }
        AdminAction::SubsList { page } => {
            subs::handle_subs_list(&bot, &q, state, msg, page, lang).await?;
        }
    }
    Ok(())
}
