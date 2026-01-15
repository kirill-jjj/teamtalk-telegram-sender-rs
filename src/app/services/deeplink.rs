use crate::core::types::DeeplinkAction;
use crate::infra::db::Database;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ResolvedDeeplink {
    pub action: DeeplinkAction,
    pub payload: Option<String>,
}

pub async fn resolve_for_user(
    db: &Database,
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

    let Ok(action) = DeeplinkAction::try_from(deeplink.action.as_str()) else {
        return Ok(None);
    };

    Ok(Some(ResolvedDeeplink {
        action,
        payload: deeplink.payload,
    }))
}
