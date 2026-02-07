use crate::adapters::tg::utils::{AdminSubEventKind, notify_admins_subscription_event};
use crate::app::services::notifications as notifications_service;
use crate::app::services::pending as pending_service;
use crate::app::services::subscription as subscription_service;
use crate::args;
use crate::core::types::{JoinGender, LanguageCode, TtNickname, TtServerName, TtUsername};
use crate::infra::db::types::UserSettings;
use crate::infra::locales;
use crate::infra::locales::LocaleKey;
use std::collections::HashMap;
use teloxide::ApiError;
use teloxide::RequestError;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Bot, Requester};
use tokio::task::JoinSet;

use super::BridgeDeps;

pub(super) async fn handle_broadcast(
    deps: &BridgeDeps<'_>,
    event_type: crate::core::types::NotificationType,
    nickname: TtNickname,
    server_name: TtServerName,
    related_tt_username: TtUsername,
    gender: JoinGender,
) {
    let Some(bot) = deps.event_bot else {
        return;
    };

    let recipients = match notifications_service::get_recipients_for_event(
        &deps.services.db,
        &related_tt_username,
        event_type,
    )
    .await
    {
        Ok(r) if !r.is_empty() => r,
        Ok(_) => return,
        Err(e) => {
            tracing::error!(
                component = "bridge",
                event_type = ?event_type,
                tt_username = %related_tt_username,
                error = %e,
                "Failed to load recipients"
            );
            return;
        }
    };

    let escaped_server = teloxide::utils::html::escape(server_name.as_str());

    let key = match event_type {
        crate::core::types::NotificationType::Join => match gender {
            JoinGender::Female => LocaleKey::EventJoinFemale,
            JoinGender::Male => LocaleKey::EventJoinMale,
            JoinGender::Neutral => LocaleKey::EventJoinNeutral,
        },
        crate::core::types::NotificationType::Leave => match gender {
            JoinGender::Female => LocaleKey::EventLeaveFemale,
            JoinGender::Male => LocaleKey::EventLeaveMale,
            JoinGender::Neutral => LocaleKey::EventLeaveNeutral,
        },
    };

    let mut rendered_text_cache: HashMap<LanguageCode, String> = HashMap::new();
    let mut set = JoinSet::new();

    for sub in recipients {
        let task_ctx = BroadcastTaskCtx {
            bot: bot.clone(),
            state: deps.state.clone(),
            services: deps.services.clone(),
            admin_id: deps.admin_id,
            related_tt_username: related_tt_username.clone(),
        };

        let lang = sub.language_code;
        let escaped_nick = if !nickname.as_str().trim().is_empty() {
            teloxide::utils::html::escape(nickname.as_str())
        } else if !related_tt_username.as_str().trim().is_empty() {
            teloxide::utils::html::escape(related_tt_username.as_str())
        } else {
            locales::get_text(lang.as_str(), LocaleKey::DisplayUnknownUser, None)
        };
        let text = rendered_text_cache
            .entry(lang)
            .or_insert_with(|| {
                let args = args!(
                    nickname = escaped_nick.clone(),
                    server = escaped_server.clone()
                );
                locales::get_text(lang.as_str(), key, args.as_ref())
            })
            .clone();

        set.spawn(async move {
            send_broadcast_to_recipient(task_ctx, sub, text).await;
        });
    }

    while let Some(res) = set.join_next().await {
        if let Err(e) = res {
            tracing::error!(component = "bridge", error = ?e, "Notification task failed after join");
        }
    }
}

struct BroadcastTaskCtx {
    bot: Bot,
    state: crate::app::state::StateHandle,
    services: crate::app::services::tt_context::TtServiceContext,
    admin_id: teloxide::types::ChatId,
    related_tt_username: TtUsername,
}

async fn send_broadcast_to_recipient(ctx: BroadcastTaskCtx, sub: UserSettings, text: String) {
    let mut send_silent = false;

    if sub.not_on_online_enabled
        && sub.not_on_online_confirmed
        && let Some(linked_tt) = &sub.teamtalk_username
    {
        let is_online = match ctx.state.is_username_online(linked_tt).await {
            Ok(value) => value,
            Err(err) => {
                tracing::error!(
                    component = "bridge",
                    tt_username = %linked_tt,
                    error = %err,
                    "Failed to resolve online status for silent notification"
                );
                false
            }
        };
        if is_online {
            send_silent = true;
        }
    }

    let res = ctx
        .bot
        .send_message(teloxide::types::ChatId(sub.telegram_id.as_i64()), text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .disable_notification(send_silent)
        .await;

    if let Ok(msg) = &res
        && sub.telegram_id.as_i64() == ctx.admin_id.0
        && !ctx.related_tt_username.as_str().is_empty()
        && let Err(e) = pending_service::add_pending_reply(
            &ctx.services.db,
            crate::core::types::TgMessageId::from(msg.id.0),
            crate::core::types::TtUserId::from(0),
            Some(&ctx.related_tt_username),
        )
        .await
    {
        tracing::error!(
            component = "bridge",
            message_id = msg.id.0,
            tt_username = %ctx.related_tt_username,
            error = %e,
            "Failed to save pending reply for broadcast"
        );
    }

    if let Err(e) = res {
        tracing::warn!(
            component = "bridge",
            telegram_id = sub.telegram_id.as_i64(),
            tt_username = ?sub.teamtalk_username,
            error = %e,
            "Failed to send notification"
        );

        if let RequestError::Api(api_err) = e {
            match api_err {
                ApiError::BotBlocked | ApiError::UserDeactivated | ApiError::ChatNotFound => {
                    tracing::info!(
                        component = "bridge",
                        telegram_id = sub.telegram_id.as_i64(),
                        tt_username = ?sub.teamtalk_username,
                        api_error = ?api_err,
                        "Cleaning up unreachable user"
                    );

                    if let Err(db_err) =
                        subscription_service::unsubscribe(&ctx.services.db, sub.telegram_id).await
                    {
                        tracing::error!(
                            component = "bridge",
                            telegram_id = sub.telegram_id.as_i64(),
                            tt_username = ?sub.teamtalk_username,
                            error = %db_err,
                            "DB error during auto-cleanup"
                        );
                    } else {
                        tracing::info!(
                            component = "bridge",
                            telegram_id = sub.telegram_id.as_i64(),
                            tt_username = ?sub.teamtalk_username,
                            "Profile removed successfully"
                        );
                        notify_admins_subscription_event(
                            &ctx.bot,
                            &ctx.services.db,
                            crate::core::types::TelegramId::from(ctx.admin_id.0),
                            sub.telegram_id,
                            sub.teamtalk_username.as_ref(),
                            AdminSubEventKind::Unsubscribed,
                        )
                        .await;
                    }
                }
                _ => {}
            }
        }
    }
}
