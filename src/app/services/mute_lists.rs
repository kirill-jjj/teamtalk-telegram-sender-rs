use crate::core::types::{MuteListMode, TelegramId, TtUsername};
use crate::infra::db::Database;
use anyhow::Result;

pub async fn get_muted_users_list(
    db: &Database,
    telegram_id: TelegramId,
    mode: MuteListMode,
) -> Result<Vec<TtUsername>> {
    db.get_muted_users_list(telegram_id, mode).await
}

pub async fn toggle_muted_user(
    db: &Database,
    telegram_id: TelegramId,
    mode: MuteListMode,
    username: &TtUsername,
) -> Result<()> {
    db.toggle_muted_user(telegram_id, mode, username).await
}
