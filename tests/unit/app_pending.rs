use crate::app::services::pending::{
    PendingRepo, get_pending_channel_reply, get_pending_reply, touch_pending_channel_reply,
    touch_pending_reply,
};
use crate::core::types::{TgMessageId, TtChannelId, TtUserId, TtUsername};
use anyhow::Result;
use async_trait::async_trait;

#[derive(Default)]
struct FakePendingRepo {
    reply: Option<(TtUserId, Option<TtUsername>)>,
    channel_reply: Option<(TtChannelId, String, String, String)>,
}

#[async_trait]
impl PendingRepo for FakePendingRepo {
    async fn get_pending_reply(
        &self,
        _reply_id: TgMessageId,
    ) -> Result<Option<(TtUserId, Option<TtUsername>)>> {
        Ok(self.reply.clone())
    }

    async fn touch_pending_reply(&self, _reply_id: TgMessageId) -> Result<()> {
        Ok(())
    }

    async fn get_pending_channel_reply(
        &self,
        _reply_id: TgMessageId,
    ) -> Result<Option<(TtChannelId, String, String, String)>> {
        Ok(self.channel_reply.clone())
    }

    async fn touch_pending_channel_reply(&self, _reply_id: TgMessageId) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn pending_reply_delegates() {
    let repo = FakePendingRepo {
        reply: Some((TtUserId::from(7), Some(TtUsername::new("hi")))),
        ..Default::default()
    };
    let res = get_pending_reply(&repo, TgMessageId::from(1))
        .await
        .unwrap();
    assert_eq!(res, Some((TtUserId::from(7), Some(TtUsername::new("hi")))));
    touch_pending_reply(&repo, TgMessageId::from(1))
        .await
        .unwrap();
}

#[tokio::test]
async fn pending_channel_reply_delegates() {
    let repo = FakePendingRepo {
        channel_reply: Some((
            TtChannelId::from(9),
            "chan".to_string(),
            "srv".to_string(),
            "msg".to_string(),
        )),
        ..Default::default()
    };
    let res = get_pending_channel_reply(&repo, TgMessageId::from(1))
        .await
        .unwrap();
    assert_eq!(
        res,
        Some((
            TtChannelId::from(9),
            "chan".to_string(),
            "srv".to_string(),
            "msg".to_string()
        ))
    );
    touch_pending_channel_reply(&repo, TgMessageId::from(1))
        .await
        .unwrap();
}
