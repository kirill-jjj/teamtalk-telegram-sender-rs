use crate::app::services::pending::{
    PendingRepo, get_pending_channel_reply, get_pending_reply, touch_pending_channel_reply,
    touch_pending_reply,
};
use crate::core::types::TtUsername;
use anyhow::Result;

#[derive(Default)]
struct FakePendingRepo {
    reply: Option<(i32, Option<TtUsername>)>,
    channel_reply: Option<(i32, String, String, String)>,
}

#[allow(async_fn_in_trait)]
impl PendingRepo for FakePendingRepo {
    async fn get_pending_reply(&self, _reply_id: i64) -> Result<Option<(i32, Option<TtUsername>)>> {
        Ok(self.reply.clone())
    }

    async fn touch_pending_reply(&self, _reply_id: i64) -> Result<()> {
        Ok(())
    }

    async fn get_pending_channel_reply(
        &self,
        _reply_id: i64,
    ) -> Result<Option<(i32, String, String, String)>> {
        Ok(self.channel_reply.clone())
    }

    async fn touch_pending_channel_reply(&self, _reply_id: i64) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn pending_reply_delegates() {
    let repo = FakePendingRepo {
        reply: Some((7, Some(TtUsername::new("hi")))),
        ..Default::default()
    };
    let res = get_pending_reply(&repo, 1).await.unwrap();
    assert_eq!(res, Some((7, Some(TtUsername::new("hi")))));
    touch_pending_reply(&repo, 1).await.unwrap();
}

#[tokio::test]
async fn pending_channel_reply_delegates() {
    let repo = FakePendingRepo {
        channel_reply: Some((9, "chan".to_string(), "srv".to_string(), "msg".to_string())),
        ..Default::default()
    };
    let res = get_pending_channel_reply(&repo, 1).await.unwrap();
    assert_eq!(
        res,
        Some((9, "chan".to_string(), "srv".to_string(), "msg".to_string()))
    );
    touch_pending_channel_reply(&repo, 1).await.unwrap();
}
