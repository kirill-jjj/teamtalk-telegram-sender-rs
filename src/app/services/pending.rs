use crate::core::types::{
    TgMessageId, TtChannelId, TtChannelName, TtServerName, TtUserId, TtUsername,
};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait PendingRepo: Sync {
    async fn add_pending_reply(
        &self,
        reply_id: TgMessageId,
        tt_user_id: TtUserId,
        tt_username: Option<&TtUsername>,
    ) -> Result<()>;
    async fn add_pending_channel_reply(
        &self,
        reply_id: TgMessageId,
        channel_id: TtChannelId,
        channel_name: &TtChannelName,
        server_name: &TtServerName,
        original_text: &str,
    ) -> Result<()>;
    async fn get_pending_reply(
        &self,
        reply_id: TgMessageId,
    ) -> Result<Option<(TtUserId, Option<TtUsername>)>>;
    async fn touch_pending_reply(&self, reply_id: TgMessageId) -> Result<()>;
    async fn get_pending_channel_reply(
        &self,
        reply_id: TgMessageId,
    ) -> Result<Option<(TtChannelId, TtChannelName, TtServerName, String)>>;
    async fn touch_pending_channel_reply(&self, reply_id: TgMessageId) -> Result<()>;
}

pub async fn get_pending_reply(
    db: &impl PendingRepo,
    reply_id: TgMessageId,
) -> Result<Option<(TtUserId, Option<TtUsername>)>> {
    db.get_pending_reply(reply_id).await
}

pub async fn add_pending_reply(
    db: &impl PendingRepo,
    reply_id: TgMessageId,
    tt_user_id: TtUserId,
    tt_username: Option<&TtUsername>,
) -> Result<()> {
    db.add_pending_reply(reply_id, tt_user_id, tt_username)
        .await
}

pub async fn add_pending_channel_reply(
    db: &impl PendingRepo,
    reply_id: TgMessageId,
    channel_id: TtChannelId,
    channel_name: &TtChannelName,
    server_name: &TtServerName,
    original_text: &str,
) -> Result<()> {
    db.add_pending_channel_reply(
        reply_id,
        channel_id,
        channel_name,
        server_name,
        original_text,
    )
    .await
}

pub async fn touch_pending_reply(db: &impl PendingRepo, reply_id: TgMessageId) -> Result<()> {
    db.touch_pending_reply(reply_id).await
}

pub async fn get_pending_channel_reply(
    db: &impl PendingRepo,
    reply_id: TgMessageId,
) -> Result<Option<(TtChannelId, TtChannelName, TtServerName, String)>> {
    db.get_pending_channel_reply(reply_id).await
}

pub async fn touch_pending_channel_reply(
    db: &impl PendingRepo,
    reply_id: TgMessageId,
) -> Result<()> {
    db.touch_pending_channel_reply(reply_id).await
}

#[async_trait]
impl PendingRepo for crate::infra::db::Database {
    async fn add_pending_reply(
        &self,
        reply_id: TgMessageId,
        tt_user_id: TtUserId,
        tt_username: Option<&TtUsername>,
    ) -> Result<()> {
        self.add_pending_reply(reply_id, tt_user_id, tt_username)
            .await
    }

    async fn add_pending_channel_reply(
        &self,
        reply_id: TgMessageId,
        channel_id: TtChannelId,
        channel_name: &TtChannelName,
        server_name: &TtServerName,
        original_text: &str,
    ) -> Result<()> {
        self.add_pending_channel_reply(
            reply_id,
            channel_id,
            channel_name,
            server_name,
            original_text,
        )
        .await
    }

    async fn get_pending_reply(
        &self,
        reply_id: TgMessageId,
    ) -> Result<Option<(TtUserId, Option<TtUsername>)>> {
        self.get_pending_reply(reply_id).await
    }

    async fn touch_pending_reply(&self, reply_id: TgMessageId) -> Result<()> {
        self.touch_pending_reply(reply_id).await
    }

    async fn get_pending_channel_reply(
        &self,
        reply_id: TgMessageId,
    ) -> Result<Option<(TtChannelId, TtChannelName, TtServerName, String)>> {
        self.get_pending_channel_reply(reply_id).await
    }

    async fn touch_pending_channel_reply(&self, reply_id: TgMessageId) -> Result<()> {
        self.touch_pending_channel_reply(reply_id).await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app_pending.rs"]
mod tests;
