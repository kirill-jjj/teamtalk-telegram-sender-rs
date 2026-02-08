use crate::adapters::tg::presenter::admin::subscriber_settings::{
    SubLinkAccountListArgs, send_sub_link_account_list, send_sub_manage_tt_menu,
};
use crate::adapters::tg::subscriber_notify::SubscriberChangeKind;
use crate::adapters::tg::utils::{answer_callback, answer_callback_empty};
use crate::app::services::tg_sub_links as tg_sub_links_service;
use crate::args;
use crate::core::types::{AdminErrorContext, LanguageCode, TelegramId, TtUsername};
use crate::infra::locales;
use teloxide::prelude::*;

use super::SubCtx;

pub(super) async fn manage_tt(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
) -> ResponseResult<()> {
    let settings = match tg_sub_links_service::load_settings(ctx.db, sub_id, LanguageCode::En).await
    {
        Ok(s) => s,
        Err(e) => {
            ctx.errors()
                .check_db_err(&ctx.q_id.0, Err(e), AdminErrorContext::Callback)
                .await?;
            return Ok(());
        }
    };
    send_sub_manage_tt_menu(
        ctx.bot,
        ctx.msg,
        ctx.lang,
        sub_id,
        page,
        settings.teamtalk_username,
    )
    .await
}

pub(super) async fn unlink(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
) -> ResponseResult<()> {
    if ctx
        .errors()
        .check_db_err(
            &ctx.q_id.0,
            tg_sub_links_service::unlink_tt(ctx.db, sub_id).await,
            AdminErrorContext::Callback,
        )
        .await?
    {
        return Ok(());
    }
    ctx.notify_change(sub_id, SubscriberChangeKind::Unlinked)
        .await;
    answer_callback(
        ctx.bot,
        ctx.q_id,
        locales::get_text(
            ctx.lang.as_str(),
            locales::LocaleKey::ToastAccountUnlinked,
            args!(user = sub_id.to_string()).as_ref(),
        ),
        true,
    )
    .await?;
    let settings = match tg_sub_links_service::load_settings(ctx.db, sub_id, LanguageCode::En).await
    {
        Ok(s) => s,
        Err(e) => {
            ctx.errors()
                .check_db_err(&ctx.q_id.0, Err(e), AdminErrorContext::Callback)
                .await?;
            return Ok(());
        }
    };
    send_sub_manage_tt_menu(
        ctx.bot,
        ctx.msg,
        ctx.lang,
        sub_id,
        page,
        settings.teamtalk_username,
    )
    .await?;
    Ok(())
}

pub(super) async fn link_list(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
    list_page: usize,
) -> ResponseResult<()> {
    let accounts = tg_sub_links_service::load_accounts(ctx.tx_tt, ctx.state_handle).await;
    send_sub_link_account_list(SubLinkAccountListArgs {
        bot: ctx.bot,
        msg: ctx.msg,
        accounts,
        search_contexts: ctx.search_contexts,
        lang: ctx.lang,
        target_id: sub_id,
        sub_page: page,
        page: list_page,
    })
    .await?;
    answer_callback_empty(ctx.bot, ctx.q_id).await?;
    Ok(())
}

pub(super) async fn link_perform(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
    username: TtUsername,
) -> ResponseResult<()> {
    if ctx
        .errors()
        .check_db_err(
            &ctx.q_id.0,
            tg_sub_links_service::link_tt(ctx.db, sub_id, &username).await,
            AdminErrorContext::Callback,
        )
        .await?
    {
        return Ok(());
    }
    ctx.notify_change(sub_id, SubscriberChangeKind::Linked(username.clone()))
        .await;
    answer_callback(
        ctx.bot,
        ctx.q_id,
        locales::get_text(
            ctx.lang.as_str(),
            locales::LocaleKey::ToastAccountLinked,
            args!(user = username.to_string()).as_ref(),
        ),
        true,
    )
    .await?;
    let settings = match tg_sub_links_service::load_settings(ctx.db, sub_id, LanguageCode::En).await
    {
        Ok(s) => s,
        Err(e) => {
            ctx.errors()
                .check_db_err(&ctx.q_id.0, Err(e), AdminErrorContext::Callback)
                .await?;
            return Ok(());
        }
    };
    send_sub_manage_tt_menu(
        ctx.bot,
        ctx.msg,
        ctx.lang,
        sub_id,
        page,
        settings.teamtalk_username,
    )
    .await?;
    Ok(())
}
