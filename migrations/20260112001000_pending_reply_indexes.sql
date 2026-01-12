CREATE INDEX IF NOT EXISTS idx_pending_replies_last_used ON pending_replies(last_used_at);
CREATE INDEX IF NOT EXISTS idx_pending_replies_tt_user_id ON pending_replies(tt_user_id);

CREATE TABLE IF NOT EXISTS pending_channel_replies (
    tg_message_id INTEGER PRIMARY KEY,
    channel_id INTEGER NOT NULL,
    channel_name TEXT NOT NULL,
    server_name TEXT NOT NULL,
    original_text TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_pending_channel_replies_last_used ON pending_channel_replies(last_used_at);
CREATE INDEX IF NOT EXISTS idx_pending_channel_replies_channel_id ON pending_channel_replies(channel_id);
