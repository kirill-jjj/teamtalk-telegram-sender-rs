use anyhow::Result;

#[allow(async_fn_in_trait)]
pub trait SubscriptionRepo: Sync {
    async fn is_telegram_id_banned(&self, telegram_id: i64) -> Result<bool>;
    async fn is_teamtalk_username_banned(&self, username: &str) -> Result<bool>;
    async fn add_subscriber(&self, telegram_id: i64) -> Result<()>;
    async fn link_tt_account(&self, telegram_id: i64, tt_username: &str) -> Result<()>;
    async fn delete_user_profile(&self, telegram_id: i64) -> Result<()>;
    async fn is_subscribed(&self, telegram_id: i64) -> Result<bool>;
}

#[derive(Debug, Clone)]
pub enum SubscribeOutcome {
    BannedUser,
    BannedTeamTalk { username: String },
    SubscribedLinked,
    SubscribedGuest,
}

pub async fn subscribe_via_deeplink(
    db: &impl SubscriptionRepo,
    telegram_id: i64,
    payload: Option<String>,
) -> Result<SubscribeOutcome> {
    if db.is_telegram_id_banned(telegram_id).await? {
        return Ok(SubscribeOutcome::BannedUser);
    }

    if let Some(tt_username) = payload.as_deref()
        && db.is_teamtalk_username_banned(tt_username).await?
    {
        return Ok(SubscribeOutcome::BannedTeamTalk {
            username: tt_username.to_string(),
        });
    }

    db.add_subscriber(telegram_id).await?;

    if let Some(tt_username) = payload {
        db.link_tt_account(telegram_id, &tt_username).await?;
        Ok(SubscribeOutcome::SubscribedLinked)
    } else {
        Ok(SubscribeOutcome::SubscribedGuest)
    }
}

pub async fn unsubscribe(db: &impl SubscriptionRepo, telegram_id: i64) -> Result<()> {
    db.delete_user_profile(telegram_id).await
}

pub async fn is_subscribed(db: &impl SubscriptionRepo, telegram_id: i64) -> Result<bool> {
    db.is_subscribed(telegram_id).await
}

impl SubscriptionRepo for crate::infra::db::Database {
    async fn is_telegram_id_banned(&self, telegram_id: i64) -> Result<bool> {
        self.is_telegram_id_banned(telegram_id).await
    }

    async fn is_teamtalk_username_banned(&self, username: &str) -> Result<bool> {
        self.is_teamtalk_username_banned(username).await
    }

    async fn add_subscriber(&self, telegram_id: i64) -> Result<()> {
        self.add_subscriber(telegram_id).await
    }

    async fn link_tt_account(&self, telegram_id: i64, tt_username: &str) -> Result<()> {
        self.link_tt_account(telegram_id, tt_username).await
    }

    async fn delete_user_profile(&self, telegram_id: i64) -> Result<()> {
        self.delete_user_profile(telegram_id).await
    }

    async fn is_subscribed(&self, telegram_id: i64) -> Result<bool> {
        self.is_subscribed(telegram_id).await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app_subscription.rs"]
mod tests;
