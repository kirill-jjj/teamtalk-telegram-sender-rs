use super::Database;

#[tokio::test]
async fn pending_reply_roundtrip() {
    let (db, path) = setup_db().await;
    db.add_pending_reply(1, 42).await.unwrap();
    let user_id = db.get_pending_reply_user_id(1).await.unwrap();
    assert_eq!(user_id, Some(42));

    db.touch_pending_reply(1).await.unwrap();
    let removed = db.cleanup_pending_replies(0).await.unwrap();
    assert!(removed <= 1);

    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn cleanup_keeps_recent_when_ttl_large() {
    let (db, path) = setup_db().await;
    db.add_pending_reply(2, 99).await.unwrap();
    let removed = db.cleanup_pending_replies(10_000).await.unwrap();
    assert_eq!(removed, 0);
    let user_id = db.get_pending_reply_user_id(2).await.unwrap();
    assert_eq!(user_id, Some(99));
    db.close().await;
    let _ = std::fs::remove_file(path);
}

async fn setup_db() -> (Database, std::path::PathBuf) {
    let mut path = std::env::temp_dir();
    path.push(format!("tt_tg_pending_{}.db", uuid::Uuid::now_v7()));
    let db = Database::new(path.to_str().unwrap()).await.unwrap();
    (db, path)
}
