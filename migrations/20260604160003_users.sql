CREATE TABLE IF NOT EXISTS users (
    id         TEXT PRIMARY KEY,
    sub        TEXT NOT NULL UNIQUE,
    email      TEXT,
    name       TEXT,
    created_at TEXT NOT NULL
);
