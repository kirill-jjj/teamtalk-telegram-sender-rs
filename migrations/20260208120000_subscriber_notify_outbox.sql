CREATE TABLE IF NOT EXISTS subscriber_notify_outbox (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    target_telegram_id INTEGER NOT NULL,
    message_text TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    next_retry_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_error TEXT
);

CREATE INDEX IF NOT EXISTS idx_subscriber_notify_outbox_next_retry
    ON subscriber_notify_outbox(next_retry_at, id);
