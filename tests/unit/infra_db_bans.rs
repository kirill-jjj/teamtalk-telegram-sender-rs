use super::Database;

#[tokio::test]
async fn ban_lifecycle() {
    let (db, path) = setup_db().await;
    db.add_ban(
        Some(10),
        Some("user1".to_string()),
        Some("reason".to_string()),
    )
    .await
    .unwrap();

    assert!(db.is_telegram_id_banned(10).await.unwrap());
    assert!(db.is_teamtalk_username_banned("user1").await.unwrap());
    assert!(db.is_teamtalk_username_banned("USER1").await.unwrap());

    let list = db.get_banned_users().await.unwrap();
    assert_eq!(list.len(), 1);
    let ban_id = list[0].id;

    db.remove_ban_by_id(ban_id).await.unwrap();
    assert!(!db.is_telegram_id_banned(10).await.unwrap());

    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn ban_checks_false_for_unknown() {
    let (db, path) = setup_db().await;
    assert!(!db.is_telegram_id_banned(404).await.unwrap());
    assert!(!db.is_teamtalk_username_banned("nobody").await.unwrap());
    db.close().await;
    let _ = std::fs::remove_file(path);
}

async fn setup_db() -> (Database, std::path::PathBuf) {
    let mut path = std::env::temp_dir();
    path.push(format!("tt_tg_bans_{}.db", uuid::Uuid::now_v7()));
    let db = Database::new(path.to_str().unwrap()).await.unwrap();
    (db, path)
}
