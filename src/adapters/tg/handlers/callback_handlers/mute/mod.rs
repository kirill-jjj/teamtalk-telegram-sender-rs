use crate::adapters::tg::presenter::settings::send_mute_menu;
use crate::adapters::tg::state::AppState;
use crate::adapters::tg::utils::telegram_id_from_user_id;
use crate::core::callbacks::MuteAction;
use crate::core::types::{LanguageCode, TelegramId};
use teloxide::prelude::*;

mod list;
mod mode;
mod server;

pub async fn handle_mute(
    bot: Bot,
    q: CallbackQuery,
    state: &AppState,
    action: MuteAction,
    lang: LanguageCode,
) -> ResponseResult<()> {
    let Some(teloxide::types::MaybeInaccessibleMessage::Regular(msg)) = &q.message else {
        return Ok(());
    };
    let Some(telegram_id) = telegram_id_from_user_id(q.from.id.0, "handle_mute") else {
        return Ok(());
    };
    let ctx = MuteCtx {
        bot: &bot,
        q: &q,
        msg,
        state,
        telegram_id,
        lang,
    };

    match action {
        MuteAction::ModeSet { mode } => {
            mode::handle_mode_set(&bot, &q, state, msg, telegram_id, lang, mode).await?;
        }
        MuteAction::Menu { mode } => {
            let has_guest = state.config.teamtalk.guest_username.is_some();
            send_mute_menu(&bot, msg, lang, mode, has_guest).await?;
        }
        MuteAction::List { mode, page } => {
            list::handle_list(&bot, msg, state, telegram_id, lang, mode, page).await?;
        }
        MuteAction::Toggle {
            mode,
            username,
            page,
        } => {
            list::handle_toggle(&ctx, mode, username, page).await?;
        }
        MuteAction::ServerList { mode, page } => {
            server::handle_server_list(&bot, msg, state, telegram_id, lang, mode, page).await?;
        }
        MuteAction::ServerToggle {
            mode,
            username,
            page,
        } => {
            server::handle_server_toggle(&ctx, mode, username, page).await?;
        }
    }

    Ok(())
}

struct MuteCtx<'a> {
    bot: &'a Bot,
    q: &'a CallbackQuery,
    msg: &'a Message,
    state: &'a AppState,
    telegram_id: TelegramId,
    lang: LanguageCode,
}
