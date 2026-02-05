use crate::adapters::tg::presenter::admin::subscriber_settings::{
    SubMuteListArgs, send_sub_mute_list,
};
use crate::app::services::tg_search as tg_search_service;
use crate::core::types::TelegramId;
use teloxide::prelude::*;

use super::SubCtx;

pub(super) async fn mute_view(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
    view_page: usize,
) -> ResponseResult<()> {
    let mode = tg_search_service::resolve_mute_mode(ctx.db, sub_id, ctx.lang).await;
    let muted = tg_search_service::list_muted_users(ctx.db, sub_id, mode.clone()).await;
    send_sub_mute_list(SubMuteListArgs {
        bot: ctx.bot,
        msg: ctx.msg,
        search_contexts: ctx.search_contexts,
        lang: ctx.lang,
        target_id: sub_id,
        sub_page: page,
        page: view_page,
        muted,
    })
    .await
}
