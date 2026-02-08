use crate::adapters::tg::presenter::admin::subscribers::send_subscriber_details;
use crate::adapters::tg::presenter::keyboards::confirm_cancel_keyboard;
use crate::adapters::tg::subscriber_notify::SubscriberChangeKind;
use crate::adapters::tg::utils::{answer_callback, answer_callback_empty};
use crate::app::services::tg_admin as tg_admin_service;
use crate::core::callbacks::{CallbackAction, SubAction};
use crate::core::types::{AdminErrorContext, TelegramId};
use crate::infra::locales;
use teloxide::prelude::*;

use super::SubCtx;

pub(super) async fn admin_add_confirm(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
) -> ResponseResult<()> {
    let is_main_admin = ctx.admin_chat_id == ctx.config.telegram.admin_chat_id;
    if !is_main_admin {
        answer_callback_empty(ctx.bot, ctx.q_id).await?;
        return Ok(());
    }
    let text = locales::get_text(ctx.lang.as_str(), locales::LocaleKey::ConfirmAdminAdd, None);
    let keyboard = confirm_cancel_keyboard(
        ctx.lang,
        locales::LocaleKey::BtnYes,
        CallbackAction::Subscriber(SubAction::AdminAdd { sub_id, page }),
        locales::LocaleKey::BtnNo,
        CallbackAction::Subscriber(SubAction::Details { sub_id, page }),
    );
    ctx.bot
        .edit_message_text(ctx.msg.chat.id, ctx.msg.id, text)
        .reply_markup(keyboard)
        .await?;
    answer_callback_empty(ctx.bot, ctx.q_id).await?;
    Ok(())
}

pub(super) async fn admin_remove_confirm(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
) -> ResponseResult<()> {
    let is_main_admin = ctx.admin_chat_id == ctx.config.telegram.admin_chat_id;
    if !is_main_admin {
        answer_callback_empty(ctx.bot, ctx.q_id).await?;
        return Ok(());
    }
    let text = locales::get_text(
        ctx.lang.as_str(),
        locales::LocaleKey::ConfirmAdminRemove,
        None,
    );
    let keyboard = confirm_cancel_keyboard(
        ctx.lang,
        locales::LocaleKey::BtnYes,
        CallbackAction::Subscriber(SubAction::AdminRemove { sub_id, page }),
        locales::LocaleKey::BtnNo,
        CallbackAction::Subscriber(SubAction::Details { sub_id, page }),
    );
    ctx.bot
        .edit_message_text(ctx.msg.chat.id, ctx.msg.id, text)
        .reply_markup(keyboard)
        .await?;
    answer_callback_empty(ctx.bot, ctx.q_id).await?;
    Ok(())
}

pub(super) async fn admin_add(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
) -> ResponseResult<()> {
    let is_main_admin = ctx.admin_chat_id == ctx.config.telegram.admin_chat_id;
    if !is_main_admin {
        answer_callback_empty(ctx.bot, ctx.q_id).await?;
        return Ok(());
    }
    if ctx
        .errors()
        .check_db_err(
            &ctx.q_id.0,
            tg_admin_service::add_admin(ctx.db, sub_id)
                .await
                .map_err(crate::app::services::tg_admin::AdminError::into_error),
            AdminErrorContext::Callback,
        )
        .await?
    {
        return Ok(());
    }
    ctx.notify_change(sub_id, SubscriberChangeKind::AdminAdded)
        .await;
    answer_callback(
        ctx.bot,
        ctx.q_id,
        locales::get_text(ctx.lang.as_str(), locales::LocaleKey::ToastAdminAdded, None),
        true,
    )
    .await?;
    send_subscriber_details(ctx.sub_details_args(sub_id, page).await).await?;
    Ok(())
}

pub(super) async fn admin_remove(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
) -> ResponseResult<()> {
    let is_main_admin = ctx.admin_chat_id == ctx.config.telegram.admin_chat_id;
    if !is_main_admin {
        answer_callback_empty(ctx.bot, ctx.q_id).await?;
        return Ok(());
    }
    if ctx
        .errors()
        .check_db_err(
            &ctx.q_id.0,
            tg_admin_service::remove_admin(ctx.db, sub_id)
                .await
                .map_err(crate::app::services::tg_admin::AdminError::into_error),
            AdminErrorContext::Callback,
        )
        .await?
    {
        return Ok(());
    }
    ctx.notify_change(sub_id, SubscriberChangeKind::AdminRemoved)
        .await;
    answer_callback(
        ctx.bot,
        ctx.q_id,
        locales::get_text(
            ctx.lang.as_str(),
            locales::LocaleKey::ToastAdminRemoved,
            None,
        ),
        true,
    )
    .await?;
    send_subscriber_details(ctx.sub_details_args(sub_id, page).await).await?;
    Ok(())
}
