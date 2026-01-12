use crate::infra::db::Database;
use anyhow::Result;

pub async fn get_user_lang_by_tt_user(
    db: &Database,
    tt_username: &str,
) -> Option<crate::core::types::LanguageCode> {
    db.get_user_lang_by_tt_user(tt_username).await
}

pub async fn get_telegram_id_by_tt_user(db: &Database, tt_username: &str) -> Option<i64> {
    db.get_telegram_id_by_tt_user(tt_username).await
}

pub async fn list_admins(db: &Database) -> Result<Vec<i64>> {
    db.get_all_admins().await
}
