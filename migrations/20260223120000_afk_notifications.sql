CREATE TABLE IF NOT EXISTS afk_user_settings (
    telegram_id INTEGER PRIMARY KEY,
    enabled BOOLEAN NOT NULL DEFAULT 0,
    threshold_minutes INTEGER NOT NULL DEFAULT 10 CHECK (threshold_minutes BETWEEN 1 AND 1440),
    list_mode TEXT NOT NULL DEFAULT 'none' CHECK (list_mode IN ('none','blacklist','whitelist')),
    cooldown_seconds INTEGER NOT NULL DEFAULT 0 CHECK (cooldown_seconds >= 0),
    FOREIGN KEY(telegram_id) REFERENCES user_settings(telegram_id)
);

CREATE TABLE IF NOT EXISTS afk_tracked_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_settings_telegram_id INTEGER NOT NULL,
    tt_username TEXT NOT NULL,
    list_mode TEXT NOT NULL CHECK (list_mode IN ('blacklist','whitelist')),
    FOREIGN KEY(user_settings_telegram_id) REFERENCES user_settings(telegram_id),
    UNIQUE(user_settings_telegram_id, tt_username, list_mode)
);

CREATE INDEX IF NOT EXISTS idx_afk_tracked_users_user_mode
    ON afk_tracked_users(user_settings_telegram_id, list_mode);

CREATE TABLE IF NOT EXISTS afk_threshold_overrides (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_settings_telegram_id INTEGER NOT NULL,
    tt_username TEXT NOT NULL,
    threshold_minutes INTEGER NOT NULL CHECK (threshold_minutes BETWEEN 1 AND 1440),
    FOREIGN KEY(user_settings_telegram_id) REFERENCES user_settings(telegram_id),
    UNIQUE(user_settings_telegram_id, tt_username)
);

CREATE INDEX IF NOT EXISTS idx_afk_threshold_overrides_user
    ON afk_threshold_overrides(user_settings_telegram_id);

INSERT OR IGNORE INTO app_settings (key, value) VALUES ('afk_default_enabled', 'false');
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('afk_default_threshold_minutes', '10');
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('afk_default_list_mode', 'none');
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('afk_default_cooldown_seconds', '0');
