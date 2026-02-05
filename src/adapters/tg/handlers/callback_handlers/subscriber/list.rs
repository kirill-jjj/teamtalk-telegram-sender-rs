use crate::adapters::tg::presenter::admin::subscribers::prepare_display_list;
use crate::adapters::tg::presenter::admin::subscribers::{
    edit_subscribers_list, send_subscriber_details,
};
use crate::adapters::tg::utils::{answer_callback, answer_callback_empty, check_db_err};
use crate::app::services::tg_admin as tg_admin_service;
use crate::app::services::tg_subscribers as tg_subscribers_service;
use crate::core::types::{AdminErrorContext, TelegramId};
use crate::infra::locales;
use teloxide::prelude::*;

use super::SubCtx;

pub(super) async fn details(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
) -> ResponseResult<()> {
    send_subscriber_details(ctx.sub_details_args(sub_id, page).await).await?;
    answer_callback_empty(ctx.bot, ctx.q_id).await?;
    Ok(())
}

pub(super) async fn delete(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
) -> ResponseResult<()> {
    if check_db_err(
        ctx.bot,
        &ctx.q_id.0,
        tg_subscribers_service::delete_subscriber(ctx.db, sub_id).await,
        ctx.config,
        ctx.admin_chat_id,
        AdminErrorContext::Callback,
        ctx.lang,
    )
    .await?
    {
        return Ok(());
    }
    answer_callback(
        ctx.bot,
        ctx.q_id,
        locales::get_text(
            ctx.lang.as_str(),
            locales::LocaleKey::ToastSubscriberDeleted,
            None,
        ),
        true,
    )
    .await?;
    edit_subscribers_list(
        ctx.bot,
        ctx.msg,
        prepare_display_list(
            ctx.bot,
            tg_admin_service::list_subscribers(ctx.db)
                .await
                .unwrap_or_default(),
        )
        .await,
        ctx.search_contexts,
        ctx.lang,
        page,
    )
    .await?;
    Ok(())
}

pub(super) async fn ban(ctx: &SubCtx<'_>, sub_id: TelegramId, page: usize) -> ResponseResult<()> {
    if check_db_err(
        ctx.bot,
        &ctx.q_id.0,
        tg_subscribers_service::ban_subscriber(ctx.db, sub_id).await,
        ctx.config,
        ctx.admin_chat_id,
        AdminErrorContext::Callback,
        ctx.lang,
    )
    .await?
    {
        return Ok(());
    }

    answer_callback(
        ctx.bot,
        ctx.q_id,
        locales::get_text(ctx.lang.as_str(), locales::LocaleKey::ToastUserBanned, None),
        true,
    )
    .await?;
    edit_subscribers_list(
        ctx.bot,
        ctx.msg,
        prepare_display_list(
            ctx.bot,
            tg_admin_service::list_subscribers(ctx.db)
                .await
                .unwrap_or_default(),
        )
        .await,
        ctx.search_contexts,
        ctx.lang,
        page,
    )
    .await?;
    Ok(())
}
