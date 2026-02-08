use super::*;
use crate::core::types::DeeplinkAction;
use crate::core::types::TelegramId;
use crate::infra::db::{Database, types::Deeplink};
use async_trait::async_trait;
use std::path::PathBuf;

async fn setup_db() -> (Database, PathBuf) {
    let db_path =
        std::env::temp_dir().join(format!("teamtalk_bot_test_{}.db", uuid::Uuid::now_v7()));
    let db = Database::new(db_path.to_str().unwrap())
        .await
        .expect("db init");
    (db, db_path)
}

#[tokio::test]
async fn resolve_for_user_honors_expected_id() {
    let (db, db_path) = setup_db().await;

    db.create_deeplink(
        "token123",
        DeeplinkAction::Subscribe,
        Some("payload"),
        Some(TelegramId::from(42)),
        60,
    )
    .await
    .expect("insert deeplink");

    let denied = resolve_for_user(&db, "token123", TelegramId::from(7))
        .await
        .expect("resolve");
    assert!(denied.is_none());

    let allowed = resolve_for_user(&db, "token123", TelegramId::from(42))
        .await
        .expect("resolve")
        .expect("expected deeplink");

    assert_eq!(allowed.action, DeeplinkAction::Subscribe);
    assert_eq!(allowed.payload.as_deref(), Some("payload"));

    db.close().await;
    let _ = std::fs::remove_file(db_path);
}

#[derive(Default)]
struct FakeDeeplinkRepo {
    deeplink: Option<Deeplink>,
}

#[async_trait]
impl DeeplinkRepo for FakeDeeplinkRepo {
    async fn resolve_deeplink(&self, _token: &str) -> anyhow::Result<Option<Deeplink>> {
        Ok(self.deeplink.clone())
    }

    async fn create_deeplink(
        &self,
        _token: &str,
        _action: DeeplinkAction,
        _payload: Option<&str>,
        _expected_telegram_id: Option<TelegramId>,
        _ttl_seconds: i64,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn consume_deeplink(&self, _token: &str) -> anyhow::Result<bool> {
        Ok(true)
    }
}

#[tokio::test]
async fn resolve_rejects_wrong_expected_id() {
    let repo = FakeDeeplinkRepo {
        deeplink: Some(Deeplink {
            action: DeeplinkAction::Subscribe,
            payload: Some("payload".to_string()),
            expected_telegram_id: Some(TelegramId::from(10)),
            expiry_time: chrono::Utc::now().naive_utc(),
        }),
    };
    let res = resolve_for_user(&repo, "t", TelegramId::from(42))
        .await
        .unwrap();
    assert!(res.is_none());
}
