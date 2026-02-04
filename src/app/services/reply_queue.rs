use crate::core::types::{LanguageCode, TelegramId, TtUsername};
use crate::infra::db::app_settings::AppSettingKey;
use crate::infra::db::reply_queue::ReplyQueueItem;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Datelike, Local, NaiveDateTime, Timelike, Utc};

#[async_trait]
pub trait ReplyQueueRepo: Sync {
    async fn get_app_setting(&self, key: AppSettingKey) -> Result<Option<String>>;
    async fn set_app_setting(&self, key: AppSettingKey, value: &str) -> Result<()>;
    async fn get_or_create_user(
        &self,
        telegram_id: TelegramId,
        default_lang: LanguageCode,
    ) -> Result<crate::infra::db::types::UserSettings>;
    async fn update_reply_queue_enabled(
        &self,
        telegram_id: TelegramId,
        enabled: bool,
    ) -> Result<()>;
    async fn get_telegram_id_by_tt_user(&self, tt_username: &TtUsername) -> Option<TelegramId>;
}

pub async fn get_reply_queue_global_enabled(db: &impl ReplyQueueRepo) -> Result<bool> {
    let value = db
        .get_app_setting(AppSettingKey::ReplyQueueEnabledGlobal)
        .await?;
    Ok(matches!(value.as_deref(), Some("1" | "true" | "on")))
}

pub async fn set_reply_queue_global_enabled(db: &impl ReplyQueueRepo, enabled: bool) -> Result<()> {
    let val = if enabled { "1" } else { "0" };
    db.set_app_setting(AppSettingKey::ReplyQueueEnabledGlobal, val)
        .await
}

pub async fn get_reply_queue_user_enabled(
    db: &impl ReplyQueueRepo,
    telegram_id: TelegramId,
) -> Result<bool> {
    let user = db.get_or_create_user(telegram_id, LanguageCode::En).await?;
    Ok(user.reply_queue_enabled)
}

pub async fn set_reply_queue_user_enabled(
    db: &impl ReplyQueueRepo,
    telegram_id: TelegramId,
    enabled: bool,
) -> Result<()> {
    db.update_reply_queue_enabled(telegram_id, enabled).await
}

pub async fn is_reply_queue_enabled_for_tt_user(
    db: &impl ReplyQueueRepo,
    tt_username: &TtUsername,
) -> Result<bool> {
    if !get_reply_queue_global_enabled(db).await? {
        return Ok(false);
    }
    let Some(tg_id) = db.get_telegram_id_by_tt_user(tt_username).await else {
        return Ok(false);
    };
    get_reply_queue_user_enabled(db, tg_id).await
}

#[async_trait]
impl ReplyQueueRepo for crate::infra::db::Database {
    async fn get_app_setting(&self, key: AppSettingKey) -> Result<Option<String>> {
        self.get_app_setting(key).await
    }

    async fn set_app_setting(&self, key: AppSettingKey, value: &str) -> Result<()> {
        self.set_app_setting(key, value).await
    }

    async fn get_or_create_user(
        &self,
        telegram_id: TelegramId,
        default_lang: LanguageCode,
    ) -> Result<crate::infra::db::types::UserSettings> {
        self.get_or_create_user(telegram_id, default_lang).await
    }

    async fn update_reply_queue_enabled(
        &self,
        telegram_id: TelegramId,
        enabled: bool,
    ) -> Result<()> {
        self.update_reply_queue_enabled(telegram_id, enabled).await
    }

    #[allow(clippy::use_self)]
    async fn get_telegram_id_by_tt_user(&self, tt_username: &TtUsername) -> Option<TelegramId> {
        crate::infra::db::Database::get_telegram_id_by_tt_user(self, tt_username).await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app_reply_queue.rs"]
mod tests;

pub fn format_queue_message(
    lang: LanguageCode,
    created_at: NaiveDateTime,
    now: DateTime<Utc>,
    text: &str,
) -> String {
    let created_utc = DateTime::<Utc>::from_naive_utc_and_offset(created_at, Utc);
    let now_local: DateTime<Local> = now.with_timezone(&Local);
    let created_local: DateTime<Local> = created_utc.with_timezone(&Local);

    let delta = now.signed_duration_since(created_utc);
    let header = if delta.num_minutes() < 60 {
        let minutes = delta.num_minutes().max(0);
        format_relative_header(lang, minutes, TimeUnit::Minutes)
    } else if delta.num_hours() < 24 {
        let hours = delta.num_hours().max(0);
        format_relative_header(lang, hours, TimeUnit::Hours)
    } else if now_local.year() == created_local.year() {
        format_absolute_header(lang, created_local, false)
    } else {
        format_absolute_header(lang, created_local, true)
    };

    format!("{header}\n{text}")
}

pub fn queue_items_sorted(items: &mut [ReplyQueueItem]) {
    items.sort_by(|a, b| (a.created_at, a.id).cmp(&(b.created_at, b.id)));
}

#[derive(Clone, Copy)]
enum TimeUnit {
    Minutes,
    Hours,
}

fn format_relative_header(lang: LanguageCode, count: i64, unit: TimeUnit) -> String {
    match lang {
        LanguageCode::En => match unit {
            TimeUnit::Minutes => {
                let unit = if count == 1 { "minute" } else { "minutes" };
                format!("Message sent {count} {unit} ago:")
            }
            TimeUnit::Hours => {
                let unit = if count == 1 { "hour" } else { "hours" };
                format!("Message sent {count} {unit} ago:")
            }
        },
        LanguageCode::Ru => match unit {
            TimeUnit::Minutes => {
                let unit = ru_plural(count, "минуту", "минуты", "минут");
                format!("Сообщение было отправлено {count} {unit} назад:")
            }
            TimeUnit::Hours => {
                let unit = ru_plural(count, "час", "часа", "часов");
                format!("Сообщение было отправлено {count} {unit} назад:")
            }
        },
    }
}

fn format_absolute_header(lang: LanguageCode, dt: DateTime<Local>, with_year: bool) -> String {
    match lang {
        LanguageCode::En => {
            let month = en_month(dt.month());
            let year = if with_year {
                format!(" {}", dt.year())
            } else {
                String::new()
            };
            format!(
                "Message sent {month} {}{year} at {:02}:{:02}:",
                dt.day(),
                dt.hour(),
                dt.minute()
            )
        }
        LanguageCode::Ru => {
            let month = ru_month_genitive(dt.month());
            let year = if with_year {
                format!(" {} года", dt.year())
            } else {
                String::new()
            };
            format!(
                "Сообщение было отправлено {} {month}{year} в {:02}:{:02}:",
                dt.day(),
                dt.hour(),
                dt.minute()
            )
        }
    }
}

fn ru_plural(value: i64, one: &str, few: &str, many: &str) -> String {
    let value = value.abs() % 100;
    let rem = value % 10;
    if (11..=14).contains(&value) {
        return many.to_string();
    }
    match rem {
        1 => one.to_string(),
        2..=4 => few.to_string(),
        _ => many.to_string(),
    }
}

const fn ru_month_genitive(month: u32) -> &'static str {
    match month {
        1 => "января",
        2 => "февраля",
        3 => "марта",
        4 => "апреля",
        5 => "мая",
        6 => "июня",
        7 => "июля",
        8 => "августа",
        9 => "сентября",
        10 => "октября",
        11 => "ноября",
        12 => "декабря",
        _ => "",
    }
}

const fn en_month(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}
