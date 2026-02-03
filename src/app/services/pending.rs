use crate::core::types::TtUsername;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait PendingRepo: Sync {
    async fn get_pending_reply(&self, reply_id: i64) -> Result<Option<(i32, Option<TtUsername>)>>;
    async fn touch_pending_reply(&self, reply_id: i64) -> Result<()>;
    async fn get_pending_channel_reply(
        &self,
        reply_id: i64,
    ) -> Result<Option<(i32, String, String, String)>>;
    async fn touch_pending_channel_reply(&self, reply_id: i64) -> Result<()>;
}

pub async fn get_pending_reply(
    db: &impl PendingRepo,
    reply_id: i64,
) -> Result<Option<(i32, Option<TtUsername>)>> {
    db.get_pending_reply(reply_id).await
}

pub async fn touch_pending_reply(db: &impl PendingRepo, reply_id: i64) -> Result<()> {
    db.touch_pending_reply(reply_id).await
}

pub async fn get_pending_channel_reply(
    db: &impl PendingRepo,
    reply_id: i64,
) -> Result<Option<(i32, String, String, String)>> {
    db.get_pending_channel_reply(reply_id).await
}

pub async fn touch_pending_channel_reply(db: &impl PendingRepo, reply_id: i64) -> Result<()> {
    db.touch_pending_channel_reply(reply_id).await
}

#[async_trait]
impl PendingRepo for crate::infra::db::Database {
    async fn get_pending_reply(&self, reply_id: i64) -> Result<Option<(i32, Option<TtUsername>)>> {
        self.get_pending_reply(reply_id).await
    }

    async fn touch_pending_reply(&self, reply_id: i64) -> Result<()> {
        self.touch_pending_reply(reply_id).await
    }

    async fn get_pending_channel_reply(
        &self,
        reply_id: i64,
    ) -> Result<Option<(i32, String, String, String)>> {
        self.get_pending_channel_reply(reply_id).await
    }

    async fn touch_pending_channel_reply(&self, reply_id: i64) -> Result<()> {
        self.touch_pending_channel_reply(reply_id).await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app_pending.rs"]
mod tests;
