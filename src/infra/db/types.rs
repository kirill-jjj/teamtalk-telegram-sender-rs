use chrono::NaiveDateTime;

#[derive(sqlx::FromRow, Debug)]
pub struct UserSettings {
    pub telegram_id: i64,
    pub language_code: String,
    pub notification_settings: String,
    pub mute_list_mode: String,
    pub teamtalk_username: Option<String>,
    pub not_on_online_enabled: bool,
    pub not_on_online_confirmed: bool,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Deeplink {
    #[allow(dead_code)]
    pub token: String,
    pub action: String,
    pub payload: Option<String>,
    pub expected_telegram_id: Option<i64>,
    pub expiry_time: NaiveDateTime,
}

#[derive(sqlx::FromRow, Debug)]
pub struct BanEntry {
    pub id: i64,
    pub telegram_id: Option<i64>,
    pub teamtalk_username: Option<String>,
    #[allow(dead_code)]
    pub ban_reason: Option<String>,
    #[allow(dead_code)]
    pub banned_at: NaiveDateTime,
}

#[derive(sqlx::FromRow, Debug)]
pub struct SubscriberInfo {
    pub telegram_id: i64,
    pub teamtalk_username: Option<String>,
    #[allow(dead_code)]
    pub language_code: String,
}
