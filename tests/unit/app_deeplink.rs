use super::*;
use crate::core::types::DeeplinkAction;
use crate::infra::db::Database;
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
        Some(42),
        60,
    )
    .await
    .expect("insert deeplink");

    let denied = resolve_for_user(&db, "token123", 7).await.expect("resolve");
    assert!(denied.is_none());

    db.create_deeplink(
        "token123",
        DeeplinkAction::Subscribe,
        Some("payload"),
        Some(42),
        60,
    )
    .await
    .expect("insert deeplink");

    let allowed = resolve_for_user(&db, "token123", 42)
        .await
        .expect("resolve")
        .expect("expected deeplink");

    assert_eq!(allowed.action, DeeplinkAction::Subscribe);
    assert_eq!(allowed.payload.as_deref(), Some("payload"));

    db.close().await;
    let _ = std::fs::remove_file(db_path);
}
