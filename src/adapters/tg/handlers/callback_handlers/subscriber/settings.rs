use crate::adapters::tg::presenter::admin::subscriber_settings::{
    send_sub_lang_menu, send_sub_mute_mode_menu, send_sub_notif_menu,
};
use crate::adapters::tg::presenter::admin::subscribers::send_subscriber_details;
use crate::adapters::tg::subscriber_notify::SubscriberChangeKind;
use crate::adapters::tg::utils::answer_callback;
use crate::app::services::tg_sub_settings as tg_sub_settings_service;
use crate::args;
use crate::core::types::{
    AdminErrorContext, LanguageCode, MuteListMode, NotificationSetting, TelegramId,
};
use crate::infra::locales;
use teloxide_ng::prelude::*;

use super::SubCtx;

pub(super) async fn lang_menu(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
) -> ResponseResult<()> {
    send_sub_lang_menu(ctx.bot, ctx.msg, ctx.lang, sub_id, page).await
}

pub(super) async fn lang_set(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
    new_lang: LanguageCode,
) -> ResponseResult<()> {
    if ctx
        .errors()
        .check_db_err(
            &ctx.q_id.0,
            tg_sub_settings_service::update_language(ctx.db, sub_id, new_lang).await,
            AdminErrorContext::Callback,
        )
        .await?
    {
        return Ok(());
    }
    ctx.notify_change(sub_id, SubscriberChangeKind::Language(new_lang))
        .await;
    answer_callback(
        ctx.bot,
        ctx.q_id,
        locales::get_text(
            ctx.lang.as_str(),
            locales::LocaleKey::ToastLangSet,
            args!(id = sub_id.to_string(), lang = new_lang.as_str()).as_ref(),
        ),
        false,
    )
    .await?;
    send_subscriber_details(ctx.sub_details_args(sub_id, page).await).await?;
    Ok(())
}

pub(super) async fn notif_menu(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
) -> ResponseResult<()> {
    send_sub_notif_menu(ctx.bot, ctx.msg, ctx.lang, sub_id, page).await
}

pub(super) async fn notif_set(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
    val: NotificationSetting,
) -> ResponseResult<()> {
    if ctx
        .errors()
        .check_db_err(
            &ctx.q_id.0,
            tg_sub_settings_service::update_notifications(ctx.db, sub_id, val.clone()).await,
            AdminErrorContext::Callback,
        )
        .await?
    {
        return Ok(());
    }
    ctx.notify_change(sub_id, SubscriberChangeKind::Notifications(val.clone()))
        .await;
    answer_callback(
        ctx.bot,
        ctx.q_id,
        locales::get_text(
            ctx.lang.as_str(),
            locales::LocaleKey::ToastNotifSet,
            args!(id = sub_id.to_string(), val = val.to_string()).as_ref(),
        ),
        false,
    )
    .await?;
    send_subscriber_details(ctx.sub_details_args(sub_id, page).await).await?;
    Ok(())
}

pub(super) async fn noon_toggle(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
) -> ResponseResult<()> {
    let enabled = match tg_sub_settings_service::toggle_noon(ctx.db, sub_id).await {
        Ok(enabled) => enabled,
        Err(error) => {
            let _ = ctx
                .errors()
                .check_db_err(&ctx.q_id.0, Err(error), AdminErrorContext::Callback)
                .await?;
            return Ok(());
        }
    };
    ctx.notify_change(sub_id, SubscriberChangeKind::OfflineOnly(enabled))
        .await;
    answer_callback(
        ctx.bot,
        ctx.q_id,
        locales::get_text(
            ctx.lang.as_str(),
            locales::LocaleKey::ToastNoonToggled,
            args!(
                id = sub_id.to_string(),
                status = if enabled {
                    locales::get_text(ctx.lang.as_str(), locales::LocaleKey::StatusEnabled, None)
                } else {
                    locales::get_text(ctx.lang.as_str(), locales::LocaleKey::StatusDisabled, None)
                }
            )
            .as_ref(),
        ),
        false,
    )
    .await?;
    send_subscriber_details(ctx.sub_details_args(sub_id, page).await).await?;
    Ok(())
}

pub(super) async fn mode_menu(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
) -> ResponseResult<()> {
    send_sub_mute_mode_menu(ctx.bot, ctx.msg, ctx.lang, sub_id, page).await
}

pub(super) async fn mode_set(
    ctx: &SubCtx<'_>,
    sub_id: TelegramId,
    page: usize,
    mode: MuteListMode,
) -> ResponseResult<()> {
    if ctx
        .errors()
        .check_db_err(
            &ctx.q_id.0,
            tg_sub_settings_service::update_mute_mode(ctx.db, sub_id, mode.clone()).await,
            AdminErrorContext::Callback,
        )
        .await?
    {
        return Ok(());
    }
    ctx.notify_change(sub_id, SubscriberChangeKind::MuteMode(mode.clone()))
        .await;
    answer_callback(
        ctx.bot,
        ctx.q_id,
        locales::get_text(
            ctx.lang.as_str(),
            locales::LocaleKey::ToastMuteModeSubSet,
            args!(id = sub_id.to_string(), val = mode.to_string()).as_ref(),
        ),
        false,
    )
    .await?;
    send_subscriber_details(ctx.sub_details_args(sub_id, page).await).await?;
    Ok(())
}
