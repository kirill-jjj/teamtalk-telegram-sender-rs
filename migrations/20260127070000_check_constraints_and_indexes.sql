-- Strengthen data integrity with CHECK constraints and add missing indexes.
-- This migration rebuilds affected tables and preserves existing data.

PRAGMA foreign_keys = OFF;

-- 1) user_settings: constrain enums and booleans.
CREATE TABLE user_settings_new (
    telegram_id INTEGER PRIMARY KEY,
    language_code TEXT NOT NULL DEFAULT 'en'
        CHECK (language_code IN ('en', 'ru')),
    notification_settings TEXT NOT NULL DEFAULT 'all'
        CHECK (notification_settings IN ('all', 'join_off', 'leave_off', 'none')),
    mute_list_mode TEXT NOT NULL DEFAULT 'blacklist'
        CHECK (mute_list_mode IN ('blacklist', 'whitelist')),
    teamtalk_username TEXT,
    not_on_online_enabled BOOLEAN NOT NULL DEFAULT 0
        CHECK (not_on_online_enabled IN (0, 1)),
    not_on_online_confirmed BOOLEAN NOT NULL DEFAULT 0
        CHECK (not_on_online_confirmed IN (0, 1))
);

INSERT INTO user_settings_new (
    telegram_id,
    language_code,
    notification_settings,
    mute_list_mode,
    teamtalk_username,
    not_on_online_enabled,
    not_on_online_confirmed
)
SELECT
    telegram_id,
    CASE language_code
        WHEN 'en' THEN 'en'
        WHEN 'ru' THEN 'ru'
        ELSE 'en'
    END AS language_code,
    CASE notification_settings
        WHEN 'all' THEN 'all'
        WHEN 'join_off' THEN 'join_off'
        WHEN 'leave_off' THEN 'leave_off'
        WHEN 'none' THEN 'none'
        ELSE 'all'
    END AS notification_settings,
    CASE mute_list_mode
        WHEN 'blacklist' THEN 'blacklist'
        WHEN 'whitelist' THEN 'whitelist'
        ELSE 'blacklist'
    END AS mute_list_mode,
    teamtalk_username,
    CASE
        WHEN not_on_online_enabled IN (1, '1', true, 'true') THEN 1
        ELSE 0
    END AS not_on_online_enabled,
    CASE
        WHEN not_on_online_confirmed IN (1, '1', true, 'true') THEN 1
        ELSE 0
    END AS not_on_online_confirmed
FROM user_settings;

DROP TABLE user_settings;
ALTER TABLE user_settings_new RENAME TO user_settings;

-- Re-create indexes that existed previously.
CREATE INDEX IF NOT EXISTS idx_user_settings_tt_username ON user_settings(teamtalk_username);
CREATE INDEX IF NOT EXISTS idx_user_settings_tt_username_nocase ON user_settings(teamtalk_username COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_user_settings_notif ON user_settings(notification_settings);
CREATE INDEX IF NOT EXISTS idx_user_settings_mute_mode ON user_settings(mute_list_mode);

-- Useful additional indexes.
CREATE INDEX IF NOT EXISTS idx_user_settings_language_code ON user_settings(language_code);
CREATE INDEX IF NOT EXISTS idx_user_settings_not_on_online_enabled ON user_settings(not_on_online_enabled);

-- 2) muted_users: constrain list_mode and keep uniqueness.
CREATE TABLE muted_users_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    muted_teamtalk_username TEXT NOT NULL,
    user_settings_telegram_id INTEGER NOT NULL,
    list_mode TEXT NOT NULL DEFAULT 'blacklist'
        CHECK (list_mode IN ('blacklist', 'whitelist')),
    FOREIGN KEY(user_settings_telegram_id) REFERENCES user_settings(telegram_id),
    UNIQUE(user_settings_telegram_id, muted_teamtalk_username, list_mode)
);

INSERT INTO muted_users_new (id, muted_teamtalk_username, user_settings_telegram_id, list_mode)
SELECT
    id,
    muted_teamtalk_username,
    user_settings_telegram_id,
    CASE list_mode
        WHEN 'blacklist' THEN 'blacklist'
        WHEN 'whitelist' THEN 'whitelist'
        ELSE 'blacklist'
    END AS list_mode
FROM muted_users;

DROP TABLE muted_users;
ALTER TABLE muted_users_new RENAME TO muted_users;

CREATE INDEX IF NOT EXISTS idx_muted_users_telegram_id ON muted_users(user_settings_telegram_id);
CREATE INDEX IF NOT EXISTS idx_muted_users_list_mode ON muted_users(list_mode);
CREATE INDEX IF NOT EXISTS idx_muted_users_user_mode ON muted_users(user_settings_telegram_id, list_mode);

-- 3) deeplinks: constrain action and add lookup/cleanup indexes.
CREATE TABLE deeplinks_new (
    token TEXT PRIMARY KEY,
    action TEXT NOT NULL
        CHECK (action IN ('subscribe', 'unsubscribe')),
    payload TEXT,
    expected_telegram_id INTEGER,
    expiry_time DATETIME NOT NULL
);

INSERT INTO deeplinks_new (token, action, payload, expected_telegram_id, expiry_time)
SELECT
    token,
    CASE action
        WHEN 'subscribe' THEN 'subscribe'
        WHEN 'unsubscribe' THEN 'unsubscribe'
        ELSE 'subscribe'
    END AS action,
    payload,
    expected_telegram_id,
    expiry_time
FROM deeplinks;

DROP TABLE deeplinks;
ALTER TABLE deeplinks_new RENAME TO deeplinks;

CREATE INDEX IF NOT EXISTS idx_deeplinks_expiry_time ON deeplinks(expiry_time);
CREATE INDEX IF NOT EXISTS idx_deeplinks_expected_telegram_id ON deeplinks(expected_telegram_id);
CREATE INDEX IF NOT EXISTS idx_deeplinks_action ON deeplinks(action);

-- 4) ban_list: add helpful index for cleanup/reporting.
CREATE INDEX IF NOT EXISTS idx_ban_list_banned_at ON ban_list(banned_at);

PRAGMA foreign_keys = ON;
