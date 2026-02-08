use crate::core::types::{DeeplinkAction, TelegramId};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait DeeplinkRepo: Sync {
    async fn resolve_deeplink(
        &self,
        token: &str,
    ) -> Result<Option<crate::infra::db::types::Deeplink>>;
    async fn create_deeplink(
        &self,
        token: &str,
        action: DeeplinkAction,
        payload: Option<&str>,
        expected_telegram_id: Option<TelegramId>,
        ttl_seconds: i64,
    ) -> Result<()>;
    async fn consume_deeplink(&self, token: &str) -> Result<bool>;
}

#[derive(Debug, Clone)]
pub struct ResolvedDeeplink {
    pub action: DeeplinkAction,
    pub payload: Option<String>,
}

pub async fn resolve_for_user(
    db: &impl DeeplinkRepo,
    token: &str,
    telegram_id: TelegramId,
) -> Result<Option<ResolvedDeeplink>> {
    let Some(deeplink) = db.resolve_deeplink(token).await? else {
        return Ok(None);
    };

    if let Some(expected_id) = deeplink.expected_telegram_id
        && expected_id != telegram_id
    {
        return Ok(None);
    }

    Ok(Some(ResolvedDeeplink {
        action: deeplink.action,
        payload: deeplink.payload,
    }))
}

pub async fn create(
    db: &impl DeeplinkRepo,
    token: &str,
    action: DeeplinkAction,
    payload: Option<&str>,
    expected_telegram_id: Option<TelegramId>,
    ttl_seconds: i64,
) -> Result<()> {
    db.create_deeplink(token, action, payload, expected_telegram_id, ttl_seconds)
        .await
}

pub async fn consume(db: &impl DeeplinkRepo, token: &str) -> Result<bool> {
    db.consume_deeplink(token).await
}

#[async_trait]
impl DeeplinkRepo for crate::infra::db::Database {
    async fn resolve_deeplink(
        &self,
        token: &str,
    ) -> Result<Option<crate::infra::db::types::Deeplink>> {
        self.resolve_deeplink(token).await
    }

    async fn create_deeplink(
        &self,
        token: &str,
        action: DeeplinkAction,
        payload: Option<&str>,
        expected_telegram_id: Option<TelegramId>,
        ttl_seconds: i64,
    ) -> Result<()> {
        self.create_deeplink(token, action, payload, expected_telegram_id, ttl_seconds)
            .await
    }

    async fn consume_deeplink(&self, token: &str) -> Result<bool> {
        self.consume_deeplink(token).await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app_deeplink.rs"]
mod tests;
