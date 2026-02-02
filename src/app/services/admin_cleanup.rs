use anyhow::Result;

#[allow(async_fn_in_trait)]
pub trait AdminCleanupRepo: Sync {
    async fn delete_user_profile(&self, telegram_id: i64) -> Result<()>;
    async fn get_telegram_id_by_tt_user(&self, tt_username: &str) -> Option<i64>;
}

pub async fn cleanup_deleted_banned_user(
    db: &impl AdminCleanupRepo,
    telegram_id: i64,
) -> Result<()> {
    db.delete_user_profile(telegram_id).await
}

pub async fn get_telegram_id_by_tt_user(
    db: &impl AdminCleanupRepo,
    tt_username: &str,
) -> Option<i64> {
    db.get_telegram_id_by_tt_user(tt_username).await
}

impl AdminCleanupRepo for crate::infra::db::Database {
    async fn delete_user_profile(&self, telegram_id: i64) -> Result<()> {
        self.delete_user_profile(telegram_id).await
    }

    async fn get_telegram_id_by_tt_user(&self, tt_username: &str) -> Option<i64> {
        self.get_telegram_id_by_tt_user(tt_username).await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app_admin_cleanup.rs"]
mod tests;
