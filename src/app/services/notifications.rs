use crate::core::types::{NotificationType, TtUsername};
use crate::infra::db::types::UserSettings;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait NotificationsRepo: Sync {
    async fn get_recipients_for_event(
        &self,
        tt_username: &TtUsername,
        event_type: NotificationType,
    ) -> Result<Vec<UserSettings>>;
}

pub async fn get_recipients_for_event(
    db: &impl NotificationsRepo,
    tt_username: &TtUsername,
    event_type: NotificationType,
) -> Result<Vec<UserSettings>> {
    db.get_recipients_for_event(tt_username, event_type).await
}

#[async_trait]
impl NotificationsRepo for crate::infra::db::Database {
    async fn get_recipients_for_event(
        &self,
        tt_username: &TtUsername,
        event_type: NotificationType,
    ) -> Result<Vec<UserSettings>> {
        self.get_recipients_for_event(tt_username, event_type).await
    }
}
