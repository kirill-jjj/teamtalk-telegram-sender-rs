CREATE TABLE IF NOT EXISTS user_settings (
    telegram_id INTEGER PRIMARY KEY,
    language_code TEXT NOT NULL DEFAULT 'en',
    notification_settings TEXT NOT NULL DEFAULT 'all',
    mute_list_mode TEXT NOT NULL DEFAULT 'blacklist',
    teamtalk_username TEXT,
    not_on_online_enabled BOOLEAN NOT NULL DEFAULT 0,
    not_on_online_confirmed BOOLEAN NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS muted_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    muted_teamtalk_username TEXT NOT NULL,
    user_settings_telegram_id INTEGER NOT NULL,
    FOREIGN KEY(user_settings_telegram_id) REFERENCES user_settings(telegram_id),
    UNIQUE(user_settings_telegram_id, muted_teamtalk_username)
);
CREATE TABLE IF NOT EXISTS subscribed_users (
    telegram_id INTEGER PRIMARY KEY
);
CREATE TABLE IF NOT EXISTS admins (
    telegram_id INTEGER PRIMARY KEY
);
CREATE TABLE IF NOT EXISTS deeplinks (
    token TEXT PRIMARY KEY,
    action TEXT NOT NULL,
    payload TEXT,
    expected_telegram_id INTEGER,
    expiry_time DATETIME NOT NULL
);
CREATE TABLE IF NOT EXISTS ban_list (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    telegram_id INTEGER,
    teamtalk_username TEXT,
    ban_reason TEXT,
    banned_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_user_settings_tt_username ON user_settings(teamtalk_username);
CREATE INDEX IF NOT EXISTS idx_user_settings_notif ON user_settings(notification_settings);
CREATE INDEX IF NOT EXISTS idx_user_settings_mute_mode ON user_settings(mute_list_mode);
CREATE INDEX IF NOT EXISTS idx_ban_list_tg_id ON ban_list(telegram_id);
CREATE INDEX IF NOT EXISTS idx_ban_list_tt_username ON ban_list(teamtalk_username COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_muted_users_telegram_id ON muted_users(user_settings_telegram_id);
