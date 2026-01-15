use super::*;
use crate::core::types::LanguageCode;
use crate::infra::db::Database;
use std::path::PathBuf;

async fn setup_db() -> (Database, PathBuf) {
    let db_path = std::env::temp_dir().join(format!(
        "teamtalk_bot_admin_cleanup_{}.db",
        uuid::Uuid::now_v7()
    ));
    let db = Database::new(db_path.to_str().unwrap())
        .await
        .expect("db init");
    (db, db_path)
}

#[tokio::test]
async fn cleanup_deleted_user_removes_profile() {
    let (db, db_path) = setup_db().await;
    db.get_or_create_user(55, LanguageCode::En)
        .await
        .expect("create user");
    db.link_tt_account(55, "sergey").await.expect("link tt");
    assert_eq!(get_telegram_id_by_tt_user(&db, "sergey").await, Some(55));

    cleanup_deleted_banned_user(&db, 55).await.expect("cleanup");
    assert_eq!(get_telegram_id_by_tt_user(&db, "sergey").await, None);

    db.close().await;
    let _ = std::fs::remove_file(db_path);
}
