CREATE TABLE muted_users_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    muted_teamtalk_username TEXT NOT NULL,
    user_settings_telegram_id INTEGER NOT NULL,
    list_mode TEXT NOT NULL DEFAULT 'blacklist',
    FOREIGN KEY(user_settings_telegram_id) REFERENCES user_settings(telegram_id),
    UNIQUE(user_settings_telegram_id, muted_teamtalk_username, list_mode)
);

INSERT INTO muted_users_new (id, muted_teamtalk_username, user_settings_telegram_id, list_mode)
SELECT mu.id, mu.muted_teamtalk_username, mu.user_settings_telegram_id, us.mute_list_mode
FROM muted_users mu
JOIN user_settings us ON us.telegram_id = mu.user_settings_telegram_id;

DROP TABLE muted_users;
ALTER TABLE muted_users_new RENAME TO muted_users;

CREATE INDEX IF NOT EXISTS idx_muted_users_telegram_id ON muted_users(user_settings_telegram_id);
