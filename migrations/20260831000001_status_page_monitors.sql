CREATE TABLE status_page_monitors (
    status_page_id TEXT NOT NULL REFERENCES status_pages(id) ON DELETE CASCADE,
    monitor_id     TEXT NOT NULL REFERENCES monitors(id) ON DELETE CASCADE,
    created_at     TEXT NOT NULL,
    PRIMARY KEY (status_page_id, monitor_id)
);
