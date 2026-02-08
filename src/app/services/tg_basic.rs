use crate::app::services::{deeplink as deeplink_service, subscription as subscription_service};
use crate::core::types::{DeeplinkAction, TelegramId, TtUsername};
use anyhow::Result;

#[derive(Debug, Clone)]
pub enum StartOutcome {
    NoToken,
    InvalidToken,
    SubscribeBannedUser,
    SubscribeBannedTeamTalk { username: TtUsername },
    SubscribeLinked,
    SubscribeGuest,
    Unsubscribe,
}

pub async fn resolve_start(
    db: &(impl deeplink_service::DeeplinkRepo + subscription_service::SubscriptionRepo),
    telegram_id: TelegramId,
    token: &str,
) -> Result<StartOutcome> {
    if token.is_empty() {
        return Ok(StartOutcome::NoToken);
    }

    let Some(deeplink) = deeplink_service::resolve_for_user(db, token, telegram_id).await? else {
        return Ok(StartOutcome::InvalidToken);
    };

    let outcome = match deeplink.action {
        DeeplinkAction::Subscribe => {
            let outcome =
                subscription_service::subscribe_via_deeplink(db, telegram_id, deeplink.payload)
                    .await?;
            match outcome {
                subscription_service::SubscribeOutcome::BannedUser => {
                    StartOutcome::SubscribeBannedUser
                }
                subscription_service::SubscribeOutcome::BannedTeamTalk { username } => {
                    StartOutcome::SubscribeBannedTeamTalk { username }
                }
                subscription_service::SubscribeOutcome::SubscribedLinked => {
                    StartOutcome::SubscribeLinked
                }
                subscription_service::SubscribeOutcome::SubscribedGuest => {
                    StartOutcome::SubscribeGuest
                }
            }
        }
        DeeplinkAction::Unsubscribe => StartOutcome::Unsubscribe,
    };

    deeplink_service::consume(db, token).await?;
    Ok(outcome)
}

pub async fn unsubscribe(
    db: &impl subscription_service::SubscriptionRepo,
    telegram_id: TelegramId,
) -> Result<()> {
    subscription_service::unsubscribe(db, telegram_id).await
}
