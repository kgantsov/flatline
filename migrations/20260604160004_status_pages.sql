CREATE TABLE status_pages (
    id               TEXT    NOT NULL PRIMARY KEY,
    name             TEXT    NOT NULL,
    description      TEXT,
    slug             TEXT    NOT NULL UNIQUE,
    refresh_interval INTEGER NOT NULL DEFAULT 60,
    created_at       TEXT    NOT NULL,
    updated_at       TEXT    NOT NULL
);
