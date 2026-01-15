use super::Database;
use crate::core::types::{LanguageCode, NotificationSetting};

#[tokio::test]
async fn subscriptions_basic_flow() {
    let (db, path) = setup_db().await;
    db.add_subscriber(10).await.unwrap();
    assert!(db.is_subscribed(10).await.unwrap());

    let subs = db.get_subscribers().await.unwrap();
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].telegram_id, 10);

    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn add_subscriber_is_idempotent() {
    let (db, path) = setup_db().await;
    db.add_subscriber(99).await.unwrap();
    db.add_subscriber(99).await.unwrap();
    let subs = db.get_subscribers().await.unwrap();
    assert_eq!(subs.len(), 1);
    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn recipients_respect_notification_settings() {
    let (db, path) = setup_db().await;
    db.get_or_create_user(20, LanguageCode::En).await.unwrap();
    db.add_subscriber(20).await.unwrap();

    db.update_notification_setting(20, NotificationSetting::JoinOff)
        .await
        .unwrap();
    let join = db
        .get_recipients_for_event("user", crate::core::types::NotificationType::Join)
        .await
        .unwrap();
    assert!(join.is_empty());

    let leave = db
        .get_recipients_for_event("user", crate::core::types::NotificationType::Leave)
        .await
        .unwrap();
    assert_eq!(leave.len(), 1);

    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn recipients_respect_mute_modes() {
    let (db, path) = setup_db().await;
    db.get_or_create_user(30, LanguageCode::En).await.unwrap();
    db.add_subscriber(30).await.unwrap();

    db.update_notification_setting(30, NotificationSetting::All)
        .await
        .unwrap();

    // blacklist + muted user => excluded
    db.toggle_muted_user(30, "bob").await.unwrap();
    let join = db
        .get_recipients_for_event("bob", crate::core::types::NotificationType::Join)
        .await
        .unwrap();
    assert!(join.is_empty());

    // whitelist + muted user => included
    db.update_mute_mode(30, crate::core::types::MuteListMode::Whitelist)
        .await
        .unwrap();
    let join = db
        .get_recipients_for_event("bob", crate::core::types::NotificationType::Join)
        .await
        .unwrap();
    assert_eq!(join.len(), 1);

    db.close().await;
    let _ = std::fs::remove_file(path);
}

async fn setup_db() -> (Database, std::path::PathBuf) {
    let mut path = std::env::temp_dir();
    path.push(format!("tt_tg_subs_{}.db", uuid::Uuid::now_v7()));
    let db = Database::new(path.to_str().unwrap()).await.unwrap();
    (db, path)
}
