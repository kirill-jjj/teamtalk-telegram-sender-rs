use super::Database;
use crate::core::types::{TgMessageId, TtUserId, TtUsername};

#[tokio::test]
async fn pending_reply_roundtrip() {
    let (db, path) = setup_db().await;
    let alpha = TtUsername::new("alpha");
    db.add_pending_reply(TgMessageId::from(1), TtUserId::from(42), Some(&alpha))
        .await
        .unwrap();
    let reply = db.get_pending_reply(TgMessageId::from(1)).await.unwrap();
    assert_eq!(reply, Some((TtUserId::from(42), Some(alpha))));

    db.touch_pending_reply(TgMessageId::from(1)).await.unwrap();
    let removed = db.cleanup_pending_replies(0).await.unwrap();
    assert!(removed <= 1);

    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn cleanup_keeps_recent_when_ttl_large() {
    let (db, path) = setup_db().await;
    db.add_pending_reply(TgMessageId::from(2), TtUserId::from(99), None)
        .await
        .unwrap();
    let removed = db.cleanup_pending_replies(10_000).await.unwrap();
    assert_eq!(removed, 0);
    let reply = db.get_pending_reply(TgMessageId::from(2)).await.unwrap();
    assert_eq!(reply, Some((TtUserId::from(99), None)));
    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn cleanup_empty_returns_zero() {
    let (db, path) = setup_db().await;
    let removed = db.cleanup_pending_replies(0).await.unwrap();
    assert_eq!(removed, 0);
    db.close().await;
    let _ = std::fs::remove_file(path);
}

async fn setup_db() -> (Database, std::path::PathBuf) {
    let mut path = std::env::temp_dir();
    path.push(format!("tt_tg_pending_{}.db", uuid::Uuid::now_v7()));
    let db = Database::new(path.to_str().unwrap()).await.unwrap();
    (db, path)
}
