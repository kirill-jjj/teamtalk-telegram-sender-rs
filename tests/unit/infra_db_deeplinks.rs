use super::Database;
use crate::core::types::DeeplinkAction;

async fn setup_db() -> (Database, std::path::PathBuf) {
    let mut path = std::env::temp_dir();
    path.push(format!("tt_tg_deeplinks_{}.db", uuid::Uuid::now_v7()));
    let db = Database::new(path.to_str().unwrap()).await.unwrap();
    (db, path)
}

#[tokio::test]
async fn resolve_deeplink_is_one_time() {
    let (db, path) = setup_db().await;
    db.create_deeplink("token", DeeplinkAction::Subscribe, None, None, 60)
        .await
        .unwrap();

    let first = db.resolve_deeplink("token").await.unwrap();
    assert!(first.is_some());
    let second = db.resolve_deeplink("token").await.unwrap();
    assert!(second.is_none());

    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn cleanup_removes_expired() {
    let (db, path) = setup_db().await;
    db.create_deeplink("expired", DeeplinkAction::Subscribe, None, None, -1)
        .await
        .unwrap();
    let removed = db.cleanup_expired_deeplinks().await.unwrap();
    assert!(removed <= 1);
    let res = db.resolve_deeplink("expired").await.unwrap();
    assert!(res.is_none());

    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn resolve_returns_none_for_missing_token() {
    let (db, path) = setup_db().await;
    let res = db.resolve_deeplink("missing").await.unwrap();
    assert!(res.is_none());
    db.close().await;
    let _ = std::fs::remove_file(path);
}
