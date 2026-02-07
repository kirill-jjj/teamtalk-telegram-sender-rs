use crate::core::types::{NotificationType, TelegramId};
use anyhow::Result;

use super::{
    Database,
    types::{SubscriberInfo, UserSettings},
};

impl Database {
    pub async fn add_subscriber(&self, telegram_id: TelegramId) -> Result<()> {
        sqlx::query!(
            "INSERT OR IGNORE INTO subscribed_users (telegram_id) VALUES (?)",
            telegram_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn is_subscribed(&self, telegram_id: TelegramId) -> Result<bool> {
        let record = sqlx::query!(
            "SELECT count(*) as count FROM subscribed_users WHERE telegram_id = ?",
            telegram_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(record.count > 0)
    }

    pub async fn get_subscribers(&self) -> Result<Vec<SubscriberInfo>> {
        let rows = sqlx::query_as!(
            SubscriberInfo,
            r#"
            SELECT
                su.telegram_id as "telegram_id!: TelegramId",
                us.teamtalk_username as "teamtalk_username?: crate::core::types::TtUsername"
            FROM subscribed_users su
            LEFT JOIN user_settings us ON su.telegram_id = us.telegram_id
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_recipients_for_event(
        &self,
        tt_username: &crate::core::types::TtUsername,
        event_type: NotificationType,
    ) -> Result<Vec<UserSettings>> {
        let event_tag = match event_type {
            NotificationType::Join => "join",
            NotificationType::Leave => "leave",
        };
        let tt_username = tt_username.as_str();

        let recipients = sqlx::query_as!(
            UserSettings,
            r#"
            SELECT
                us.telegram_id as "telegram_id!: TelegramId",
                us.language_code as "language_code!: crate::core::types::LanguageCode",
                us.notification_settings as "notification_settings!: crate::core::types::NotificationSetting",
                us.mute_list_mode as "mute_list_mode!: crate::core::types::MuteListMode",
                us.teamtalk_username as "teamtalk_username?: crate::core::types::TtUsername",
                us.not_on_online_enabled as "not_on_online_enabled!",
                us.not_on_online_confirmed as "not_on_online_confirmed!",
                us.reply_queue_enabled as "reply_queue_enabled!",
                us.admin_sub_events_enabled as "admin_sub_events_enabled!"
            FROM user_settings us
            JOIN subscribed_users su ON us.telegram_id = su.telegram_id
            LEFT JOIN muted_users mu ON us.telegram_id = mu.user_settings_telegram_id
                AND mu.muted_teamtalk_username = ?
                AND mu.list_mode = us.mute_list_mode
            WHERE us.notification_settings != 'none'
            AND (
                (? = 'join' AND us.notification_settings != 'join_off')
                OR
                (? = 'leave' AND us.notification_settings != 'leave_off')
            )
            AND (
                (us.mute_list_mode = 'blacklist' AND mu.id IS NULL)
                OR
                (us.mute_list_mode = 'whitelist' AND mu.id IS NOT NULL)
            )
            "#,
            tt_username,
            event_tag,
            event_tag
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(recipients)
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/infra_db_subscriptions.rs"]
mod tests;
