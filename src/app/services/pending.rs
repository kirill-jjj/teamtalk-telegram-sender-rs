use crate::infra::db::Database;
use anyhow::Result;

pub async fn get_pending_reply_user_id(db: &Database, reply_id: i64) -> Result<Option<i32>> {
    db.get_pending_reply_user_id(reply_id).await
}

pub async fn touch_pending_reply(db: &Database, reply_id: i64) -> Result<()> {
    db.touch_pending_reply(reply_id).await
}

pub async fn get_pending_channel_reply(
    db: &Database,
    reply_id: i64,
) -> Result<Option<(i32, String, String, String)>> {
    db.get_pending_channel_reply(reply_id).await
}

pub async fn touch_pending_channel_reply(db: &Database, reply_id: i64) -> Result<()> {
    db.touch_pending_channel_reply(reply_id).await
}
