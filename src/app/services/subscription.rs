use crate::infra::db::Database;
use anyhow::Result;

#[derive(Debug, Clone)]
pub enum SubscribeOutcome {
    BannedUser,
    BannedTeamTalk { username: String },
    SubscribedLinked,
    SubscribedGuest,
}

pub async fn subscribe_via_deeplink(
    db: &Database,
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

pub async fn unsubscribe(db: &Database, telegram_id: i64) -> Result<()> {
    db.delete_user_profile(telegram_id).await
}

pub async fn is_subscribed(db: &Database, telegram_id: i64) -> Result<bool> {
    db.is_subscribed(telegram_id).await
}
