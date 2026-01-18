use super::Database;
use crate::core::types::{LanguageCode, MuteListMode};

#[tokio::test]
async fn toggle_mute_list() {
    let (db, path) = setup_db().await;
    db.get_or_create_user(1, LanguageCode::En).await.unwrap();

    db.toggle_muted_user(1, MuteListMode::Blacklist, "alice")
        .await
        .unwrap();
    let list = db
        .get_muted_users_list(1, MuteListMode::Blacklist)
        .await
        .unwrap();
    assert_eq!(list, vec!["alice".to_string()]);

    db.toggle_muted_user(1, MuteListMode::Blacklist, "alice")
        .await
        .unwrap();
    let list = db
        .get_muted_users_list(1, MuteListMode::Blacklist)
        .await
        .unwrap();
    assert!(list.is_empty());

    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn mute_payload_is_literal() {
    let (db, path) = setup_db().await;
    db.get_or_create_user(9, LanguageCode::En).await.unwrap();
    let payload = "x'); DELETE FROM muted_users; --";
    db.toggle_muted_user(9, MuteListMode::Blacklist, payload)
        .await
        .unwrap();
    let list = db
        .get_muted_users_list(9, MuteListMode::Blacklist)
        .await
        .unwrap();
    assert_eq!(list, vec![payload.to_string()]);

    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn update_mute_mode() {
    let (db, path) = setup_db().await;
    db.get_or_create_user(2, LanguageCode::En).await.unwrap();
    db.update_mute_mode(2, MuteListMode::Whitelist)
        .await
        .unwrap();

    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn muted_list_is_empty_for_new_user() {
    let (db, path) = setup_db().await;
    db.get_or_create_user(15, LanguageCode::En).await.unwrap();
    let list = db
        .get_muted_users_list(15, MuteListMode::Blacklist)
        .await
        .unwrap();
    assert!(list.is_empty());
    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn lists_are_separate_between_modes() {
    let (db, path) = setup_db().await;
    db.get_or_create_user(20, LanguageCode::En).await.unwrap();
    db.toggle_muted_user(20, MuteListMode::Blacklist, "alice")
        .await
        .unwrap();
    db.toggle_muted_user(20, MuteListMode::Whitelist, "bob")
        .await
        .unwrap();

    let blacklist = db
        .get_muted_users_list(20, MuteListMode::Blacklist)
        .await
        .unwrap();
    let whitelist = db
        .get_muted_users_list(20, MuteListMode::Whitelist)
        .await
        .unwrap();

    assert_eq!(blacklist, vec!["alice".to_string()]);
    assert_eq!(whitelist, vec!["bob".to_string()]);

    db.close().await;
    let _ = std::fs::remove_file(path);
}

async fn setup_db() -> (Database, std::path::PathBuf) {
    let mut path = std::env::temp_dir();
    path.push(format!("tt_tg_mutes_{}.db", uuid::Uuid::now_v7()));
    let db = Database::new(path.to_str().unwrap()).await.unwrap();
    (db, path)
}
