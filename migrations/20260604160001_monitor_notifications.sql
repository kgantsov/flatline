CREATE TABLE monitor_notifications (
    id          TEXT    NOT NULL PRIMARY KEY,
    monitor_id  TEXT    NOT NULL REFERENCES monitors(id) ON DELETE CASCADE,
    channel_id  TEXT    NOT NULL REFERENCES notification_channels(id) ON DELETE CASCADE,
    on_recovery INTEGER NOT NULL DEFAULT 0,  -- 0 = false, 1 = true
    created_at  TEXT    NOT NULL,            -- RFC3339
    UNIQUE(monitor_id, channel_id)
);
