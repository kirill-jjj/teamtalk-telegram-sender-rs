use crate::app::services::user_settings as user_settings_service;
use crate::args;
use crate::core::types::{LanguageCode, MuteListMode, NotificationSetting, TelegramId, TtUsername};
use crate::infra::db::Database;
use crate::infra::locales;
use crate::infra::locales::LocaleKey;
use teloxide_ng::prelude::Requester;
use teloxide_ng::types::ChatId;

const OUTBOX_BATCH_LIMIT: i64 = 100;

#[derive(Clone, Debug)]
pub struct AdminActor {
    pub telegram_id: TelegramId,
    pub full_name: String,
    pub username: Option<String>,
}

impl AdminActor {
    pub fn from_telegram_user(user: &teloxide_ng::types::User) -> Option<Self> {
        let telegram_id = TelegramId::try_from(user.id.0).ok()?;
        let full_name = user.last_name.as_ref().map_or_else(
            || user.first_name.clone(),
            |last_name| format!("{} {}", user.first_name, last_name),
        );
        Some(Self {
            telegram_id,
            full_name,
            username: user.username.clone(),
        })
    }

    pub const fn fallback(telegram_id: TelegramId) -> Self {
        Self {
            telegram_id,
            full_name: String::new(),
            username: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SubscriberChangeKind {
    Language(LanguageCode),
    Notifications(NotificationSetting),
    OfflineOnly(bool),
    MuteMode(MuteListMode),
    AdminAdded,
    AdminRemoved,
    Linked(TtUsername),
    Unlinked,
    Deleted,
    Banned,
}

pub async fn notify_subscriber_change(
    bot: &teloxide_ng::Bot,
    db: &Database,
    target_telegram_id: TelegramId,
    actor: &AdminActor,
    change: SubscriberChangeKind,
) {
    let settings = match user_settings_service::get_or_create(
        db,
        target_telegram_id,
        LanguageCode::En,
    )
    .await
    {
        Ok(settings) => settings,
        Err(error) => {
            tracing::error!(
                error = %error,
                target_telegram_id = target_telegram_id.as_i64(),
                "Failed to load target user settings for subscriber notification"
            );
            return;
        }
    };
    let text = render_subscriber_change_message(settings.language_code, &change, actor);
    if let Err(error) = bot
        .send_message(ChatId(target_telegram_id.as_i64()), text.clone())
        .await
    {
        tracing::warn!(
            error = %error,
            target_telegram_id = target_telegram_id.as_i64(),
            "Immediate subscriber notification failed; enqueueing retry"
        );
        if let Err(queue_error) = db
            .add_subscriber_notify_outbox_item(target_telegram_id, &text)
            .await
        {
            tracing::error!(
                error = %queue_error,
                target_telegram_id = target_telegram_id.as_i64(),
                "Failed to enqueue subscriber notification retry"
            );
        }
    }
}

pub fn spawn_subscriber_notify_retry_worker(
    bot: teloxide_ng::Bot,
    db: Database,
    retry_interval_seconds: u64,
    retry_backoff_seconds: u64,
    max_attempts: u32,
    cancel_token: tokio_util::sync::CancellationToken,
) {
    tokio::spawn(async move {
        let interval_seconds = retry_interval_seconds.max(1);
        let backoff_seconds = retry_backoff_seconds.max(1);
        let max_attempts = max_attempts.max(1);
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_seconds));
        loop {
            tokio::select! {
                () = cancel_token.cancelled() => break,
                _ = interval.tick() => {}
            }
            flush_subscriber_notify_outbox(&bot, &db, backoff_seconds, max_attempts).await;
        }
    });
}

async fn flush_subscriber_notify_outbox(
    bot: &teloxide_ng::Bot,
    db: &Database,
    backoff_seconds: u64,
    max_attempts: u32,
) {
    let items = match db
        .get_due_subscriber_notify_outbox_items(OUTBOX_BATCH_LIMIT)
        .await
    {
        Ok(items) => items,
        Err(error) => {
            tracing::error!(error = %error, "Failed to load subscriber notification outbox");
            return;
        }
    };
    for item in items {
        let send_result = bot
            .send_message(
                ChatId(item.target_telegram_id.as_i64()),
                item.message_text.clone(),
            )
            .await;
        match send_result {
            Ok(_) => {
                if let Err(error) = db.delete_subscriber_notify_outbox_item(item.id).await {
                    tracing::error!(
                        error = %error,
                        outbox_id = item.id,
                        "Failed to delete delivered subscriber notification outbox item"
                    );
                } else {
                    tracing::info!(
                        outbox_id = item.id,
                        target_telegram_id = item.target_telegram_id.as_i64(),
                        "Delivered subscriber notification from outbox"
                    );
                }
            }
            Err(error) => {
                let next_attempt = item.attempts.saturating_add(1);
                if next_attempt >= i64::from(max_attempts) {
                    if let Err(delete_error) =
                        db.delete_subscriber_notify_outbox_item(item.id).await
                    {
                        tracing::error!(
                            error = %delete_error,
                            outbox_id = item.id,
                            "Failed to delete exhausted subscriber notification outbox item"
                        );
                    } else {
                        tracing::warn!(
                            outbox_id = item.id,
                            target_telegram_id = item.target_telegram_id.as_i64(),
                            attempts = next_attempt,
                            error = %error,
                            "Dropped subscriber notification outbox item after max attempts"
                        );
                    }
                    continue;
                }
                let attempt_u64 = u64::try_from(next_attempt).unwrap_or(u64::MAX);
                let delay_seconds = backoff_seconds.saturating_mul(attempt_u64.max(1));
                if let Err(update_error) = db
                    .mark_subscriber_notify_outbox_retry(item.id, &error.to_string(), delay_seconds)
                    .await
                {
                    tracing::error!(
                        error = %update_error,
                        outbox_id = item.id,
                        "Failed to reschedule subscriber notification outbox item"
                    );
                } else {
                    tracing::warn!(
                        outbox_id = item.id,
                        target_telegram_id = item.target_telegram_id.as_i64(),
                        attempts = next_attempt,
                        retry_delay_seconds = delay_seconds,
                        error = %error,
                        "Rescheduled subscriber notification outbox item"
                    );
                }
            }
        }
    }
}

pub fn render_subscriber_change_message(
    lang: LanguageCode,
    change: &SubscriberChangeKind,
    actor: &AdminActor,
) -> String {
    let line_1 = match change {
        SubscriberChangeKind::Language(new_lang) => locales::get_text(
            lang.as_str(),
            LocaleKey::SubUserNotifyLang,
            args!(value = format_language_name(lang, *new_lang)).as_ref(),
        ),
        SubscriberChangeKind::Notifications(setting) => locales::get_text(
            lang.as_str(),
            LocaleKey::SubUserNotifyNotif,
            args!(value = format_notification_setting(lang, setting)).as_ref(),
        ),
        SubscriberChangeKind::OfflineOnly(enabled) => locales::get_text(
            lang.as_str(),
            LocaleKey::SubUserNotifyNoon,
            args!(value = format_status(lang, *enabled)).as_ref(),
        ),
        SubscriberChangeKind::MuteMode(mode) => locales::get_text(
            lang.as_str(),
            LocaleKey::SubUserNotifyMuteMode,
            args!(value = format_mute_mode(lang, mode)).as_ref(),
        ),
        SubscriberChangeKind::AdminAdded => {
            locales::get_text(lang.as_str(), LocaleKey::SubUserNotifyAdminAdded, None)
        }
        SubscriberChangeKind::AdminRemoved => {
            locales::get_text(lang.as_str(), LocaleKey::SubUserNotifyAdminRemoved, None)
        }
        SubscriberChangeKind::Linked(username) => locales::get_text(
            lang.as_str(),
            LocaleKey::SubUserNotifyLinked,
            args!(value = username.to_string()).as_ref(),
        ),
        SubscriberChangeKind::Unlinked => {
            locales::get_text(lang.as_str(), LocaleKey::SubUserNotifyUnlinked, None)
        }
        SubscriberChangeKind::Deleted => {
            locales::get_text(lang.as_str(), LocaleKey::SubUserNotifyDeleted, None)
        }
        SubscriberChangeKind::Banned => {
            locales::get_text(lang.as_str(), LocaleKey::SubUserNotifyBanned, None)
        }
    };
    let actor_name = if actor.full_name.trim().is_empty() {
        actor.telegram_id.as_i64().to_string()
    } else {
        actor.full_name.trim().to_string()
    };
    let line_2 = if let Some(username) = actor.username.as_ref().filter(|u| !u.is_empty()) {
        let prefixed = if username.starts_with('@') {
            username.clone()
        } else {
            format!("@{username}")
        };
        locales::get_text(
            lang.as_str(),
            LocaleKey::SubUserNotifyActorUsername,
            args!(name = actor_name, username = prefixed).as_ref(),
        )
    } else {
        locales::get_text(
            lang.as_str(),
            LocaleKey::SubUserNotifyActor,
            args!(name = actor_name).as_ref(),
        )
    };
    format!("{line_1}\n{line_2}")
}

fn format_status(lang: LanguageCode, enabled: bool) -> String {
    locales::get_text(
        lang.as_str(),
        if enabled {
            LocaleKey::StatusEnabled
        } else {
            LocaleKey::StatusDisabled
        },
        None,
    )
}

fn format_notification_setting(lang: LanguageCode, setting: &NotificationSetting) -> String {
    let key = match setting {
        NotificationSetting::All => LocaleKey::BtnSubAll,
        NotificationSetting::JoinOff => LocaleKey::BtnSubLeave,
        NotificationSetting::LeaveOff => LocaleKey::BtnSubJoin,
        NotificationSetting::None => LocaleKey::BtnSubNone,
    };
    locales::get_text(lang.as_str(), key, args!(marker = "").as_ref())
        .trim()
        .to_string()
}

fn format_mute_mode(lang: LanguageCode, mode: &MuteListMode) -> String {
    locales::get_text(
        lang.as_str(),
        match mode {
            MuteListMode::Blacklist => LocaleKey::ModeBlacklist,
            MuteListMode::Whitelist => LocaleKey::ModeWhitelist,
        },
        None,
    )
}

fn format_language_name(lang: LanguageCode, value: LanguageCode) -> String {
    let key = match value {
        LanguageCode::En => LocaleKey::ValLangEn,
        LanguageCode::Ru => LocaleKey::ValLangRu,
    };
    locales::get_text(lang.as_str(), key, None)
}

#[cfg(test)]
#[path = "../../../tests/unit/tg_subscriber_notify.rs"]
mod tests;
