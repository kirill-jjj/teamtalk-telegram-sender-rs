-- Clean up orphaned muted_users rows before FK-constrained migrations run.
DELETE FROM muted_users
WHERE user_settings_telegram_id NOT IN (SELECT telegram_id FROM user_settings);
