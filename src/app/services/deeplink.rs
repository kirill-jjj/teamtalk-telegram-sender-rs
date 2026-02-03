use crate::core::types::DeeplinkAction;
use anyhow::Result;

#[allow(async_fn_in_trait)]
pub trait DeeplinkRepo: Sync {
    async fn resolve_deeplink(
        &self,
        token: &str,
    ) -> Result<Option<crate::infra::db::types::Deeplink>>;
}

#[derive(Debug, Clone)]
pub struct ResolvedDeeplink {
    pub action: DeeplinkAction,
    pub payload: Option<String>,
}

pub async fn resolve_for_user(
    db: &impl DeeplinkRepo,
    token: &str,
    telegram_id: i64,
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

impl DeeplinkRepo for crate::infra::db::Database {
    async fn resolve_deeplink(
        &self,
        token: &str,
    ) -> Result<Option<crate::infra::db::types::Deeplink>> {
        self.resolve_deeplink(token).await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app_deeplink.rs"]
mod tests;
