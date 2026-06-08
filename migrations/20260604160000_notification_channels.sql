CREATE TABLE notification_channels (
    id         TEXT NOT NULL PRIMARY KEY,
    name       TEXT NOT NULL,
    config     TEXT NOT NULL,  -- JSON
    created_at TEXT NOT NULL,  -- RFC3339
    updated_at TEXT NOT NULL   -- RFC3339
);
