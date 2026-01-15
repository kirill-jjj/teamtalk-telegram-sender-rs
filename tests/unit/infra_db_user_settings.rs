use super::Database;
use crate::core::types::{LanguageCode, NotificationSetting};

#[tokio::test]
async fn get_or_create_and_updates() {
    let (db, path) = setup_db().await;
    let user = db.get_or_create_user(1, LanguageCode::En).await.unwrap();
    assert_eq!(user.telegram_id, 1);

    db.update_language(1, LanguageCode::Ru).await.unwrap();
    let lang = db.get_user_lang_by_tt_user("missing").await;
    assert!(lang.is_none());

    db.update_notification_setting(1, NotificationSetting::LeaveOff)
        .await
        .unwrap();

    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn toggle_noon_and_linking() {
    let (db, path) = setup_db().await;
    db.get_or_create_user(2, LanguageCode::En).await.unwrap();
    let first = db.toggle_noon(2).await.unwrap();
    let second = db.toggle_noon(2).await.unwrap();
    assert_ne!(first, second);

    db.link_tt_account(2, "bob").await.unwrap();
    let username = db.get_tt_username_by_telegram_id(2).await.unwrap();
    assert_eq!(username.as_deref(), Some("bob"));
    let lang = db.get_user_lang_by_tt_user("bob").await;
    assert!(lang.is_some());

    let confirmed: i64 = sqlx::query_scalar(
        "SELECT CAST(not_on_online_confirmed AS INTEGER) FROM user_settings WHERE telegram_id = ?",
    )
    .bind(2_i64)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert!(confirmed >= 0);

    db.unlink_tt_account(2).await.unwrap();
    let username = db.get_tt_username_by_telegram_id(2).await.unwrap();
    assert!(username.is_none());

    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn delete_user_profile_clears_relations() {
    let (db, path) = setup_db().await;
    db.get_or_create_user(3, LanguageCode::En).await.unwrap();
    db.add_subscriber(3).await.unwrap();
    db.add_admin(3).await.unwrap();
    db.toggle_muted_user(3, "user").await.unwrap();
    db.delete_user_profile(3).await.unwrap();

    assert!(!db.is_subscribed(3).await.unwrap());
    assert!(!db.get_all_admins().await.unwrap().contains(&3));
    assert!(db.get_muted_users_list(3).await.unwrap().is_empty());

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM user_settings WHERE telegram_id = ?")
        .bind(3_i64)
        .fetch_one(&db.pool)
        .await
        .unwrap_or(0);
    assert_eq!(count, 0);

    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn tt_username_payload_is_literal() {
    let (db, path) = setup_db().await;
    db.get_or_create_user(11, LanguageCode::En).await.unwrap();
    let payload = "bob'; DROP TABLE user_settings; --";
    db.link_tt_account(11, payload).await.unwrap();
    let fetched = db.get_telegram_id_by_tt_user(payload).await;
    assert_eq!(fetched, Some(11));

    let other = db.get_or_create_user(12, LanguageCode::En).await.unwrap();
    assert_eq!(other.telegram_id, 12);

    db.close().await;
    let _ = std::fs::remove_file(path);
}

async fn setup_db() -> (Database, std::path::PathBuf) {
    let mut path = std::env::temp_dir();
    path.push(format!("tt_tg_user_settings_{}.db", uuid::Uuid::now_v7()));
    let db = Database::new(path.to_str().unwrap()).await.unwrap();
    (db, path)
}
