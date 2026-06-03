CREATE TABLE monitors (
    id          TEXT        NOT NULL PRIMARY KEY,  -- UUID stored as text
    name        TEXT        NOT NULL,
    config      TEXT        NOT NULL,              -- JSON, e.g. {"type":"http","url":"..."}
    interval    INTEGER     NOT NULL DEFAULT 60,   -- check interval in seconds
    timeout     INTEGER     NOT NULL DEFAULT 10,   -- request timeout in seconds
    enabled     INTEGER     NOT NULL DEFAULT 1,    -- SQLite has no BOOLEAN; 1=true, 0=false
    created_at  TEXT        NOT NULL,              -- ISO 8601 / RFC 3339 stored as text
    updated_at  TEXT        NOT NULL
);
