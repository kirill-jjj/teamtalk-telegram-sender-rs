use crate::app::services::deeplink as deeplink_service;
use crate::app::services::subscribers as subscribers_service;
use crate::args;
use crate::core::types::DeeplinkAction;
use crate::infra::locales;
use uuid::Uuid;

use super::user::UserCtx;

pub(super) async fn handle_sub(ctx: &UserCtx) {
    if let Some(bot_user) = &ctx.bot_username {
        let is_guest = ctx.username.as_str().is_empty()
            || ctx
                .tt_config
                .guest_username
                .as_ref()
                .is_some_and(|g| g == &ctx.username);

        let payload = if is_guest {
            None
        } else {
            Some(ctx.username.as_str())
        };

        let token = Uuid::now_v7().to_string().replace('-', "");
        let expected_telegram_id = if ctx.username.as_str().is_empty() {
            None
        } else {
            subscribers_service::get_telegram_id_by_tt_user(&ctx.services.db, &ctx.username).await
        };
        let res = deeplink_service::create(
            &ctx.services.db,
            &token,
            DeeplinkAction::Subscribe,
            payload,
            expected_telegram_id,
            ctx.deeplink_ttl,
        )
        .await;

        if res.is_ok() {
            let link = format!("https://t.me/{bot_user}?start={token}");
            let text = locales::get_text_or_log(
                ctx.reply_lang.as_str(),
                locales::LocaleKey::TtSubLink,
                args!(link = link).as_ref(),
            );
            ctx.send_reply(text).await;
        } else {
            let text = locales::get_text_or_log(
                ctx.reply_lang.as_str(),
                locales::LocaleKey::TtErrorGeneric,
                None,
            );
            ctx.send_reply(text).await;
        }
    } else {
        ctx.send_reply(
            "Telegram integration is currently disabled (Event Token missing).".to_string(),
        )
        .await;
    }
}

pub(super) async fn handle_unsub(ctx: &UserCtx) {
    if let Some(bot_user) = &ctx.bot_username {
        let token = Uuid::now_v7().to_string().replace('-', "");
        let expected_telegram_id = if ctx.username.as_str().is_empty() {
            None
        } else {
            subscribers_service::get_telegram_id_by_tt_user(&ctx.services.db, &ctx.username).await
        };
        let res = deeplink_service::create(
            &ctx.services.db,
            &token,
            DeeplinkAction::Unsubscribe,
            None,
            expected_telegram_id,
            ctx.deeplink_ttl,
        )
        .await;

        if res.is_ok() {
            let link = format!("https://t.me/{bot_user}?start={token}");
            let text = locales::get_text_or_log(
                ctx.reply_lang.as_str(),
                locales::LocaleKey::TtUnsubLink,
                args!(link = link).as_ref(),
            );
            ctx.send_reply(text).await;
        } else {
            let text = locales::get_text_or_log(
                ctx.reply_lang.as_str(),
                locales::LocaleKey::TtErrorGeneric,
                None,
            );
            ctx.send_reply(text).await;
        }
    } else {
        ctx.send_reply(
            "Telegram integration is currently disabled (Event Token missing).".to_string(),
        )
        .await;
    }
}
