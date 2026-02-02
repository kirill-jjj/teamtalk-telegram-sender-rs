use anyhow::Result;

#[allow(async_fn_in_trait)]
pub trait PendingRepo: Sync {
    async fn get_pending_reply(&self, reply_id: i64) -> Result<Option<(i32, Option<String>)>>;
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
) -> Result<Option<(i32, Option<String>)>> {
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

impl PendingRepo for crate::infra::db::Database {
    async fn get_pending_reply(&self, reply_id: i64) -> Result<Option<(i32, Option<String>)>> {
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
