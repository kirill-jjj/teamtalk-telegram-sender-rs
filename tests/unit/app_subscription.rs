use crate::app::services::subscription::{
    SubscribeOutcome, SubscriptionRepo, is_subscribed, subscribe_via_deeplink, unsubscribe,
};
use crate::core::types::TtUsername;
use anyhow::Result;
use async_trait::async_trait;

#[derive(Default)]
struct FakeRepo {
    banned_tg: bool,
    banned_tt: bool,
    subscribed: bool,
}

#[async_trait]
impl SubscriptionRepo for FakeRepo {
    async fn is_telegram_id_banned(&self, _telegram_id: i64) -> Result<bool> {
        Ok(self.banned_tg)
    }

    async fn is_teamtalk_username_banned(&self, _username: &TtUsername) -> Result<bool> {
        Ok(self.banned_tt)
    }

    async fn add_subscriber(&self, _telegram_id: i64) -> Result<()> {
        Ok(())
    }

    async fn link_tt_account(&self, _telegram_id: i64, _tt_username: &TtUsername) -> Result<()> {
        Ok(())
    }

    async fn delete_user_profile(&self, _telegram_id: i64) -> Result<()> {
        Ok(())
    }

    async fn is_subscribed(&self, _telegram_id: i64) -> Result<bool> {
        Ok(self.subscribed)
    }
}

#[tokio::test]
async fn subscribe_banned_user_short_circuits() {
    let repo = FakeRepo {
        banned_tg: true,
        ..Default::default()
    };
    let outcome = subscribe_via_deeplink(&repo, 1, None).await.unwrap();
    assert!(matches!(outcome, SubscribeOutcome::BannedUser));
}

#[tokio::test]
async fn subscribe_banned_teamtalk_user_short_circuits() {
    let repo = FakeRepo {
        banned_tt: true,
        ..Default::default()
    };
    let outcome = subscribe_via_deeplink(&repo, 1, Some("tt".to_string()))
        .await
        .unwrap();
    assert!(matches!(outcome, SubscribeOutcome::BannedTeamTalk { .. }));
}

#[tokio::test]
async fn subscribe_guest_when_no_payload() {
    let repo = FakeRepo::default();
    let outcome = subscribe_via_deeplink(&repo, 1, None).await.unwrap();
    assert!(matches!(outcome, SubscribeOutcome::SubscribedGuest));
}

#[tokio::test]
async fn subscribe_linked_when_payload_present() {
    let repo = FakeRepo::default();
    let outcome = subscribe_via_deeplink(&repo, 1, Some("tt".to_string()))
        .await
        .unwrap();
    assert!(matches!(outcome, SubscribeOutcome::SubscribedLinked));
}

#[tokio::test]
async fn is_subscribed_delegates() {
    let repo = FakeRepo {
        subscribed: true,
        ..Default::default()
    };
    assert!(is_subscribed(&repo, 1).await.unwrap());
}

#[tokio::test]
async fn unsubscribe_delegates() {
    let repo = FakeRepo::default();
    unsubscribe(&repo, 1).await.unwrap();
}
