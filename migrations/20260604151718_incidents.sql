CREATE TABLE incidents (
    id               TEXT     NOT NULL PRIMARY KEY,  -- UUID stored as text
    monitor_id       TEXT     NOT NULL,
    started_at       TEXT     NOT NULL,              -- ISO 8601 / RFC 3339 stored as text
    resolved_at      TEXT                             -- ISO 8601 / RFC 3339 stored as text, NULL while open
);
