use crate::core::types::{TelegramId, TtUsername};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait SubscriptionRepo: Sync {
    async fn is_telegram_id_banned(&self, telegram_id: TelegramId) -> Result<bool>;
    async fn is_teamtalk_username_banned(&self, username: &TtUsername) -> Result<bool>;
    async fn add_subscriber(&self, telegram_id: TelegramId) -> Result<()>;
    async fn link_tt_account(
        &self,
        telegram_id: TelegramId,
        tt_username: &TtUsername,
    ) -> Result<()>;
    async fn delete_user_profile(&self, telegram_id: TelegramId) -> Result<()>;
    async fn is_subscribed(&self, telegram_id: TelegramId) -> Result<bool>;
}

#[derive(Debug, Clone)]
pub enum SubscribeOutcome {
    BannedUser,
    BannedTeamTalk { username: TtUsername },
    SubscribedLinked,
    SubscribedGuest,
}

pub async fn subscribe_via_deeplink(
    db: &impl SubscriptionRepo,
    telegram_id: TelegramId,
    payload: Option<String>,
) -> Result<SubscribeOutcome> {
    if db.is_telegram_id_banned(telegram_id).await? {
        return Ok(SubscribeOutcome::BannedUser);
    }

    if let Some(tt_username) = payload.as_deref().map(TtUsername::from)
        && db.is_teamtalk_username_banned(&tt_username).await?
    {
        return Ok(SubscribeOutcome::BannedTeamTalk {
            username: tt_username,
        });
    }

    db.add_subscriber(telegram_id).await?;

    if let Some(tt_username) = payload.map(TtUsername::from) {
        db.link_tt_account(telegram_id, &tt_username).await?;
        Ok(SubscribeOutcome::SubscribedLinked)
    } else {
        Ok(SubscribeOutcome::SubscribedGuest)
    }
}

pub async fn unsubscribe(db: &impl SubscriptionRepo, telegram_id: TelegramId) -> Result<()> {
    db.delete_user_profile(telegram_id).await
}

pub async fn is_subscribed(db: &impl SubscriptionRepo, telegram_id: TelegramId) -> Result<bool> {
    db.is_subscribed(telegram_id).await
}

#[async_trait]
impl SubscriptionRepo for crate::infra::db::Database {
    async fn is_telegram_id_banned(&self, telegram_id: TelegramId) -> Result<bool> {
        self.is_telegram_id_banned(telegram_id).await
    }

    async fn is_teamtalk_username_banned(&self, username: &TtUsername) -> Result<bool> {
        self.is_teamtalk_username_banned(username).await
    }

    async fn add_subscriber(&self, telegram_id: TelegramId) -> Result<()> {
        self.add_subscriber(telegram_id).await
    }

    async fn link_tt_account(
        &self,
        telegram_id: TelegramId,
        tt_username: &TtUsername,
    ) -> Result<()> {
        self.link_tt_account(telegram_id, tt_username).await
    }

    async fn delete_user_profile(&self, telegram_id: TelegramId) -> Result<()> {
        self.delete_user_profile(telegram_id).await
    }

    async fn is_subscribed(&self, telegram_id: TelegramId) -> Result<bool> {
        self.is_subscribed(telegram_id).await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app_subscription.rs"]
mod tests;
