ALTER TABLE user_settings
ADD COLUMN admin_sub_events_enabled BOOLEAN NOT NULL DEFAULT 0
    CHECK (admin_sub_events_enabled IN (0, 1));

UPDATE user_settings
SET admin_sub_events_enabled = 1
WHERE EXISTS (
    SELECT 1 FROM app_settings
    WHERE key = 'admin_sub_events_enabled'
      AND value IN ('1', 'true', 'on', 'yes')
);

DELETE FROM app_settings
WHERE key = 'admin_sub_events_enabled';
