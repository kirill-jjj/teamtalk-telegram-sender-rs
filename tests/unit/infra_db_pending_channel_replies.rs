use super::Database;
use crate::core::types::{TgMessageId, TtChannelId};

#[tokio::test]
async fn pending_channel_reply_roundtrip() {
    let (db, path) = setup_db().await;
    db.add_pending_channel_reply(
        TgMessageId::from(10),
        TtChannelId::from(1),
        "chan",
        "srv",
        "text",
    )
        .await
        .unwrap();
    let data = db
        .get_pending_channel_reply(TgMessageId::from(10))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(data.0, TtChannelId::from(1));
    assert_eq!(data.1, "chan");
    assert_eq!(data.2, "srv");
    assert_eq!(data.3, "text");

    db.touch_pending_channel_reply(TgMessageId::from(10))
        .await
        .unwrap();
    let removed = db.cleanup_pending_channel_replies(0).await.unwrap();
    assert!(removed <= 1);

    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn cleanup_keeps_recent_when_ttl_large() {
    let (db, path) = setup_db().await;
    db.add_pending_channel_reply(
        TgMessageId::from(11),
        TtChannelId::from(2),
        "chan2",
        "srv2",
        "text2",
    )
        .await
        .unwrap();
    let removed = db.cleanup_pending_channel_replies(10_000).await.unwrap();
    assert_eq!(removed, 0);
    let data = db
        .get_pending_channel_reply(TgMessageId::from(11))
        .await
        .unwrap();
    assert!(data.is_some());
    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn cleanup_empty_returns_zero() {
    let (db, path) = setup_db().await;
    let removed = db.cleanup_pending_channel_replies(0).await.unwrap();
    assert_eq!(removed, 0);
    db.close().await;
    let _ = std::fs::remove_file(path);
}

async fn setup_db() -> (Database, std::path::PathBuf) {
    let mut path = std::env::temp_dir();
    path.push(format!("tt_tg_pending_chan_{}.db", uuid::Uuid::now_v7()));
    let db = Database::new(path.to_str().unwrap()).await.unwrap();
    (db, path)
}
