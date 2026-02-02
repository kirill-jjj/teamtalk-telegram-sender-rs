CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT OR IGNORE INTO app_settings (key, value)
VALUES ('reply_queue_enabled_global', '0');

ALTER TABLE user_settings
ADD COLUMN reply_queue_enabled BOOLEAN NOT NULL DEFAULT 0
    CHECK (reply_queue_enabled IN (0, 1));

CREATE TABLE IF NOT EXISTS reply_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tt_username TEXT NOT NULL,
    admin_telegram_id INTEGER NOT NULL,
    message_text TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_reply_queue_tt_username ON reply_queue(tt_username);
CREATE INDEX IF NOT EXISTS idx_reply_queue_created_at ON reply_queue(created_at);