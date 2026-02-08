use super::Database;
use crate::core::types::TelegramId;

#[tokio::test]
async fn outbox_add_and_get_due_items() {
    let (db, path) = setup_db().await;
    db.add_subscriber_notify_outbox_item(TelegramId::from(101), "first")
        .await
        .unwrap();
    db.add_subscriber_notify_outbox_item(TelegramId::from(202), "second")
        .await
        .unwrap();

    let items = db.get_due_subscriber_notify_outbox_items(10).await.unwrap();
    assert_eq!(items.len(), 2);
    assert!(items[0].id < items[1].id);
    assert_eq!(items[0].target_telegram_id, TelegramId::from(101));
    assert_eq!(items[1].target_telegram_id, TelegramId::from(202));
    assert_eq!(items[0].message_text, "first");
    assert_eq!(items[1].message_text, "second");
    assert_eq!(items[0].attempts, 0);
    assert_eq!(items[1].attempts, 0);

    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn outbox_retry_updates_attempts_and_delete_removes_item() {
    let (db, path) = setup_db().await;
    db.add_subscriber_notify_outbox_item(TelegramId::from(777), "payload")
        .await
        .unwrap();

    let item = db
        .get_due_subscriber_notify_outbox_items(1)
        .await
        .unwrap()
        .pop()
        .unwrap();
    db.mark_subscriber_notify_outbox_retry(item.id, "network error", 60)
        .await
        .unwrap();

    let attempts: i64 =
        sqlx::query_scalar("SELECT attempts FROM subscriber_notify_outbox WHERE id = ?")
            .bind(item.id)
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert_eq!(attempts, 1);

    let last_error: String =
        sqlx::query_scalar("SELECT last_error FROM subscriber_notify_outbox WHERE id = ?")
            .bind(item.id)
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert_eq!(last_error, "network error");

    let due_now = db.get_due_subscriber_notify_outbox_items(10).await.unwrap();
    assert!(due_now.is_empty());

    let deleted = db
        .delete_subscriber_notify_outbox_item(item.id)
        .await
        .unwrap();
    assert_eq!(deleted, 1);
    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM subscriber_notify_outbox")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(remaining, 0);

    db.close().await;
    let _ = std::fs::remove_file(path);
}

async fn setup_db() -> (Database, std::path::PathBuf) {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "tt_tg_subscriber_notify_outbox_{}.db",
        uuid::Uuid::now_v7()
    ));
    let db = Database::new(path.to_str().unwrap()).await.unwrap();
    (db, path)
}
