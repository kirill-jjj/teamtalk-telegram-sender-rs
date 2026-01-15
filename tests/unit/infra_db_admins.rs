use super::Database;

#[tokio::test]
async fn add_and_remove_admin() {
    let (db, path) = setup_db().await;
    assert!(db.add_admin(100).await.unwrap());
    assert!(!db.add_admin(100).await.unwrap());

    let admins = db.get_all_admins().await.unwrap();
    assert!(admins.contains(&100));

    assert!(db.remove_admin(100).await.unwrap());
    assert!(!db.remove_admin(100).await.unwrap());

    db.close().await;
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn add_extreme_admin_ids() {
    let (db, path) = setup_db().await;
    assert!(db.add_admin(i64::MAX).await.unwrap());
    assert!(db.add_admin(i64::MIN).await.unwrap());
    let admins = db.get_all_admins().await.unwrap();
    assert!(admins.contains(&i64::MAX));
    assert!(admins.contains(&i64::MIN));
    db.close().await;
    let _ = std::fs::remove_file(path);
}

async fn setup_db() -> (Database, std::path::PathBuf) {
    let mut path = std::env::temp_dir();
    path.push(format!("tt_tg_admins_{}.db", uuid::Uuid::now_v7()));
    let db = Database::new(path.to_str().unwrap()).await.unwrap();
    (db, path)
}
