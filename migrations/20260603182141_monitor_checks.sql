CREATE TABLE monitor_checks (
    id               TEXT     NOT NULL PRIMARY KEY,  -- UUID stored as text
    monitor_id       TEXT     NOT NULL,
    status           TEXT     NOT NULL,
    status_code      INTEGER,
    response_time_ms INTEGER  NOT NULL,
    error_message    TEXT,
    checked_at       TEXT     NOT NULL               -- ISO 8601 / RFC 3339 stored as text
);
