use crate::adapters::tg::presenter::settings::send_mute_menu;
use crate::adapters::tg::state::AppState;
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
    let telegram_id = tg_user_id_i64(q.from.id.0);
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

fn tg_user_id_i64(user_id: u64) -> TelegramId {
    TelegramId::from(i64::try_from(user_id).unwrap_or(i64::MAX))
}
