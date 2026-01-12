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
    let deeplink = match db.resolve_deeplink(token).await? {
        Some(val) => val,
        None => return Ok(None),
    };

    if let Some(expected_id) = deeplink.expected_telegram_id
        && expected_id != telegram_id
    {
        return Ok(None);
    }

    let action = match DeeplinkAction::try_from(deeplink.action.as_str()) {
        Ok(val) => val,
        Err(_) => return Ok(None),
    };

    Ok(Some(ResolvedDeeplink {
        action,
        payload: deeplink.payload,
    }))
}
